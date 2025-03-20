use std::{
    convert::Infallible,
    net::SocketAddr,
    time::{Duration, Instant},
};

use axum::{
    Json, Router, body, debug_handler,
    extract::{ConnectInfo, State},
    http::StatusCode,
    response::{
        IntoResponse, Sse,
        sse::{Event, KeepAlive},
    },
    routing::{get, post},
};
use axum_extra::{TypedHeader, extract::Query, headers::ContentType};
use futures::stream::{self, Stream, StreamExt};
use mime::Mime;

use serde_json::json;

use tokio_stream::wrappers::BroadcastStream;
use tower_http::services::ServeDir;

use crate::{
    error::AppError,
    msg::Msg,
    req::{PubReq, Req, SubReq, SubReqGuard},
    state::AppState,
};

/// Builds the axum router for the application.
pub fn router(state: &AppState) -> Router<AppState> {
    let mut router = Router::new()
        .route(&state.pub_path, post(publish))
        .route(&state.sub_path, get(subscribe));

    // Serve static files from the specified directory.
    if let Some(serve_static_dir) = &state.serve_static_dir {
        router = router.nest_service(&state.serve_static_path, ServeDir::new(serve_static_dir))
    }

    router
}

/// Utility function to decode raw body based on content type.
///
/// Supported content types:
///   - application/json
///   - application/x-www-form-urlencoded
fn decode_raw_body(mime: &Mime, raw: &body::Bytes) -> Result<Msg, AppError> {
    Ok(match (mime.type_(), mime.subtype()) {
        (mime::APPLICATION, mime::JSON) => {
            serde_json::from_slice(raw).map_err(|e| AppError::BadRequest(e.to_string()))?
        }
        (mime::APPLICATION, mime::WWW_FORM_URLENCODED) => {
            // serde_html_form supports repeated keys as arrays
            serde_html_form::from_bytes(raw).map_err(|e| AppError::BadRequest(e.to_string()))?
        }
        _ => {
            return Err(AppError::UnsupportedMediaType(format!(
                r#"unsupported media type "{mime}""#
            )));
        }
    })
}

#[debug_handler]
async fn publish(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    TypedHeader(content_type): TypedHeader<ContentType>,
    axum_req: axum::extract::Request,
) -> Result<impl IntoResponse, AppError> {
    let req = Req::new(addr, &axum_req);
    let raw = body::to_bytes(axum_req.into_body(), state.max_body_size.as_u64() as usize)
        .await
        .map_err(|e| {
            if let Some(src) = std::error::Error::source(&e) {
                if src.is::<http_body_util::LengthLimitError>() {
                    return AppError::PayloadTooLarge(e.to_string());
                }
            }

            AppError::Internal(e.into())
        })?;
    let msg = decode_raw_body(&content_type.into(), &raw)?;
    let pub_req = PubReq::new(req, msg);

    if let Some(pub_req) = state.script.publish(pub_req).await? {
        let subs = state.broadcast.send(pub_req).unwrap_or(0);

        Ok((
            StatusCode::ACCEPTED,
            Json(json!({
                "subscribers": subs,
                "queued": state.broadcast.len(),
            })),
        ))
    } else {
        Ok((
            StatusCode::FORBIDDEN,
            Json(json!({"error": "message rejected by script"})),
        ))
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct LastEventIdQuery {
    last_event_id: Option<String>,
}

#[debug_handler]
async fn subscribe(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Query(LastEventIdQuery { last_event_id }): Query<LastEventIdQuery>,
    axum_req: axum::extract::Request,
) -> Result<impl IntoResponse, AppError> {
    // Header takes precedence over query parameter
    let last_event_id = axum_req
        .headers()
        .get("last-event-id")
        .and_then(|id| id.to_str().ok().map(String::from))
        .or(last_event_id);

    let req = Req::new(addr, &axum_req);
    let sub_req = SubReq::new(req, last_event_id.clone());

    match state.script.subscribe(sub_req).await? {
        Some(sub_req) => Ok(sse_subscribe(state, sub_req, last_event_id).await),
        None => Err(AppError::Forbidden("subscribe rejected by script".into())),
    }
}

async fn sse_subscribe(
    state: AppState,
    sub_req: SubReq,
    _last_event_id: Option<String>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let start = Instant::now();
    let keep_alive = KeepAlive::new()
        .interval(state.keep_alive)
        .text(state.keep_alive_text.clone());

    let events = async_stream::stream! {
        let event_stream = stream::once(async { Ok(Event::default().comment("ok")) }).chain(
            BroadcastStream::new(state.broadcast.subscribe()).filter_map(async |pub_req| { match pub_req {
                Ok(pub_req) if !pub_req.msg().is_empty() => {
                    match state.script.message(pub_req, &sub_req).await {
                        Ok(Some(pub_req)) if !pub_req.msg().is_empty() => {
                            Some(Ok(pub_req.msg().clone().into()))
                        },
                        Ok(_) => {
                            tracing::debug!("received empty message from script");
                            None
                        },
                        Err(e) => {
                            tracing::error!("{e}");
                            None
                        }
                    }
                },
                Ok(_) => {
                    tracing::debug!("received empty message");
                    None
                }
                Err(e) => {
                    tracing::error!("{e}");
                    None
                }
            }}),
        );
        tokio::pin!(event_stream);

        let timeout = if state.timeout.as_millis() > 0 {
            tokio::time::sleep(state.timeout)
        } else {
            // Effectively no timeout
            tokio::time::sleep(Duration::from_millis(u64::MAX))
        };
        tokio::pin!(timeout);

        // Unsubscribe on guard drop
        let _guard = SubReqGuard::new(&state, sub_req.clone());

        loop {
            tokio::select! {
                event = event_stream.next() => {
                    if let Some(event) = event {
                        yield event;
                    }
                },
                _ = &mut timeout => {
                    let retry = match state.script.timeout(&sub_req, &start.elapsed()).await {
                        Ok(Some(retry)) => Duration::from_millis(retry as u64),
                        Ok(None) => state.timeout_retry,
                        Err(e) => {
                            tracing::error!("{e}");
                            state.timeout_retry
                        }
                    };

                    yield Ok(Event::default().comment("timeout").retry(retry));
                    break;
                }
            }
        }

        // _guard dropped here. Unsubscribe called.
    };

    Sse::new(events).keep_alive(keep_alive)
}
