use axum::{
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
use axum_extra::headers::ContentType;

pub enum AppError {
    Internal(anyhow::Error),
    BadRequest(String),
    UnsupportedMediaType(String),
}

impl AppError {
    fn to_json_response<S>(status_code: StatusCode, s: S) -> Response
    where
        S: Into<String>,
    {
        (
            status_code,
            [(header::CONTENT_TYPE, ContentType::json().to_string())],
            serde_json::json!({"error": s.into()}).to_string(),
        )
            .into_response()
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match &self {
            Self::Internal(e) => {
                Self::to_json_response(StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
            }

            Self::BadRequest(s) => Self::to_json_response(StatusCode::BAD_REQUEST, s),

            Self::UnsupportedMediaType(s) => {
                Self::to_json_response(StatusCode::UNSUPPORTED_MEDIA_TYPE, s)
            }
        }
    }
}

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(e: E) -> Self {
        Self::Internal(e.into())
    }
}