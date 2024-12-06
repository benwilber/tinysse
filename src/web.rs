use std::{convert::Infallible, net::SocketAddr};

use axum::{
    body, debug_handler,
    extract::{ConnectInfo, State},
    http::StatusCode,
    response::{
        sse::{Event, KeepAlive},
        IntoResponse, Sse,
    },
    routing::{get, post},
    Json, Router,
};
use axum_extra::{headers::ContentType, TypedHeader};
use futures::stream::{self, Stream, StreamExt as _};
use mime::Mime;

use serde_json::json;

use tokio_stream::wrappers::BroadcastStream;
use tracing::{debug, error};

use crate::{
    error::AppError,
    msg::Message,
    req::{PublishRequest, Request, SubscribeRequest},
    state::AppState,
};

pub fn router(_state: &AppState) -> Router<AppState> {
    Router::new()
        .route("/publish", post(publish))
        .route("/subscribe", get(subscribe))
}

fn decode_raw_body(mime: &Mime, raw: &body::Bytes) -> Result<Message, AppError> {
    Ok(match (mime.type_(), mime.subtype()) {
        (mime::APPLICATION, mime::JSON) => {
            serde_json::from_slice(raw).map_err(|e| AppError::BadRequest(e.to_string()))?
        }
        (mime::APPLICATION, mime::WWW_FORM_URLENCODED) => {
            serde_html_form::from_bytes(raw).map_err(|e| AppError::BadRequest(e.to_string()))?
        }
        _ => {
            return Err(AppError::UnsupportedMediaType(format!(
                r#"unsupported media type "{mime}""#
            )))
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
    let req = Request::new(addr, &axum_req);
    let raw = body::to_bytes(axum_req.into_body(), usize::MAX).await?;
    let msg = decode_raw_body(&content_type.into(), &raw)?;
    let pub_req = PublishRequest::new(req, msg);

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

#[debug_handler]
async fn subscribe(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    axum_req: axum::extract::Request,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let req = Request::new(addr, &axum_req);
    let sub_req = SubscribeRequest::new(req);
    let sub_req = state.script.subscribe(sub_req).await.unwrap().unwrap();

    let events = async_stream::stream! {
        let event_stream = stream::once(async { Ok(Event::default().comment("ok")) }).chain(
            BroadcastStream::new(state.broadcast.subscribe()).filter_map(|pub_req| async { match pub_req {
                Ok(pub_req) if !pub_req.msg().is_empty() => {
                    match state.script.message(pub_req, &sub_req).await {
                        Ok(Some(pub_req)) if !pub_req.msg().is_empty() => {
                            Some(Ok(pub_req.msg().clone().into()))
                        },
                        Ok(_) => {
                            debug!("received empty message from script");
                            None
                        },
                        Err(e) => {
                            error!("{e:?}");
                            None
                        }
                    }
                },
                Ok(_) => {
                    debug!("received empty message");
                    None
                }
                Err(e) => {
                    error!("{e:?}");
                    None
                }
            }}),
        );
        tokio::pin!(event_stream);

        let timeout = tokio::time::sleep(state.timeout);
        tokio::pin!(timeout);

        loop {
            tokio::select! {
                event = event_stream.next() => {
                    if let Some(event) = event {
                        yield event;
                    }
                },
                _ = &mut timeout => {
                    yield Ok(Event::default().comment("timeout").retry(state.timeout_retry));
                    break;
                }
            }
        }
    };

    Sse::new(events).keep_alive(
        KeepAlive::new()
            .interval(state.keep_alive)
            .text(&state.keep_alive_text),
    )
}
