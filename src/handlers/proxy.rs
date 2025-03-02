use crate::{error::AppError, services::openrouter::SharedOpenRouterService};
use axum::{
    extract::{Path, State},
    http::{HeaderMap, Method},
    response::Response,
};
use bytes::Bytes;

// 带提供商的端点处理函数
pub async fn proxy_chat_completions_with_provider(
    State(service): State<SharedOpenRouterService>,
    Path(provider): Path<String>,
    method: Method,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Response, AppError> {
    // 提取API密钥
    let api_key = service.extract_api_key(&headers)?;
    // 处理请求体
    let processed_body = service.process_request_body(&body, Some(&provider), true)?;
    // 发送代理请求
    service
        .send_proxy_request(
            &"chat/completions",
            method,
            &headers,
            processed_body,
            &api_key,
        )
        .await
}

// embeddings 端点处理函数
pub async fn proxy_embeddings_with_provider(
    State(service): State<SharedOpenRouterService>,
    Path(provider): Path<String>,
    method: Method,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Response, AppError> {
    // 提取API密钥
    let api_key = service.extract_api_key(&headers)?;
    // 处理请求体
    let processed_body = service.process_request_body(&body, Some(&provider), true)?;
    // 发送代理请求
    service
        .send_proxy_request(&"embeddings", method, &headers, processed_body, &api_key)
        .await
}

// 不带提供商的端点处理函数
pub async fn proxy_chat_completions(
    State(service): State<SharedOpenRouterService>,
    method: Method,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Response, AppError> {
    // 使用硬编码的路径
    service
        .proxy_request(None, "chat/completions", method, &headers, body)
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
        .proxy_request(None, "embeddings", method, &headers, body)
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
        .proxy_request(None, "models", method, &headers, body)
        .await
}
