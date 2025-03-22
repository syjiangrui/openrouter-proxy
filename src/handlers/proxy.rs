use crate::{error::AppError, services::openrouter::SharedOpenRouterService};
use axum::{
    extract::State,
    http::{HeaderMap, Method},
    response::Response,
};
use bytes::Bytes;

// 不带提供商的端点处理函数
pub async fn proxy_chat_completions(
    State(service): State<SharedOpenRouterService>,
    method: Method,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Response, AppError> {
    service
        .proxy_request("chat/completions", method, &headers, body)
        .await
}

// embeddings 端点
pub async fn proxy_embeddings(
    State(service): State<SharedOpenRouterService>,
    method: Method,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Response, AppError> {
    service
        .proxy_request("embeddings", method, &headers, body)
        .await
}

// models 端点
pub async fn proxy_models(
    State(service): State<SharedOpenRouterService>,
    method: Method,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Response, AppError> {
    service
        .proxy_request("models", method, &headers, body)
        .await
}
