use crate::{error::AppError, services::openrouter::SharedOpenRouterService};
use axum::{
    extract::{Path, State},
    http::{HeaderMap, Method},
    response::Response,
};
use bytes::Bytes;
use tracing::debug;

// 带提供商的端点处理函数
pub async fn proxy_with_provider(
    State(service): State<SharedOpenRouterService>,
    Path((provider, path)): Path<(String, String)>,
    method: Method,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Response, AppError> {
    let should_modify_model = path.contains("chat/completions") || path.contains("embeddings");

    debug!("代理请求: {:?} /{} (提供商: {})", method, path, provider);

    // 提取API密钥
    let api_key = service.extract_api_key(&headers)?;

    // 处理请求体
    let processed_body =
        service.process_request_body(&body, Some(&provider), should_modify_model)?;

    // 发送代理请求
    service
        .send_proxy_request(&path, method, &headers, processed_body, &api_key)
        .await
}

// 不带提供商的端点处理函数
pub async fn proxy_without_provider(
    State(service): State<SharedOpenRouterService>,
    Path(path): Path<String>,
    method: Method,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Response, AppError> {
    debug!("代理请求: {:?} /{} (无提供商)", method, path);

    // 提取API密钥
    let api_key = service.extract_api_key(&headers)?;

    // 处理请求体（不修改模型名称）
    let processed_body = service.process_request_body(&body, None, false)?;

    // 发送代理请求
    service
        .send_proxy_request(&path, method, &headers, processed_body, &api_key)
        .await
}
