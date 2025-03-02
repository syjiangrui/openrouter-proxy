use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("认证错误: {0}")]
    Auth(String),

    #[error("解析错误: {0}")]
    Parse(String),

    #[error("代理错误: {0}")]
    Proxy(String),

    #[error("TLS 错误: {0}")]
    Tls(String),

    #[error("请求错误: {0}")]
    Request(#[from] reqwest::Error),

    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),

    #[error("服务器错误: {0}")]
    Server(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_type, message) = match &self {
            AppError::Auth(msg) => (StatusCode::UNAUTHORIZED, "auth_error", msg),
            AppError::Parse(msg) => (StatusCode::BAD_REQUEST, "parse_error", msg),
            AppError::Proxy(msg) => (StatusCode::BAD_GATEWAY, "proxy_error", msg),
            AppError::Tls(msg) => (StatusCode::INTERNAL_SERVER_ERROR, "tls_error", msg),
            AppError::Request(e) => (StatusCode::BAD_GATEWAY, "request_error", &e.to_string()),
            AppError::Io(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "io_error",
                &e.to_string(),
            ),
            AppError::Server(msg) => (StatusCode::INTERNAL_SERVER_ERROR, "server_error", msg),
        };

        let body = json!({
            "error": {
                "message": message,
                "type": error_type,
            }
        });

        let body_string = serde_json::to_string(&body).unwrap_or_else(|_| {
            r#"{"error":{"message":"Failed to serialize error response","type":"internal_error"}}"#.to_string()
        });

        (
            status,
            [(axum::http::header::CONTENT_TYPE, "application/json")],
            body_string,
        )
            .into_response()
    }
}
