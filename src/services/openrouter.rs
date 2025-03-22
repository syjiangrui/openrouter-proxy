use crate::{config::Config, error::AppError, models::request};
use axum::{
    body::{Body, Bytes},
    http::{HeaderMap, Method},
    response::{IntoResponse, Response},
};
use futures_util::TryStreamExt;
use reqwest::Client;
use std::sync::Arc;

pub struct OpenRouterService {
    client: Client,
    base_url: String,
    config: Config,
}

impl OpenRouterService {
    pub fn new(config: Config) -> Self {
        let client = Client::builder()
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            base_url: config.openrouter_base_url.clone(),
            config,
        }
    }

    // 从请求头中提取API密钥
    pub fn extract_api_key(&self, headers: &HeaderMap) -> Result<String, AppError> {
        let auth_header = headers
            .get("authorization")
            .ok_or_else(|| AppError::Auth("缺少 Authorization 请求头".into()))?;

        let auth_str = auth_header
            .to_str()
            .map_err(|_| AppError::Auth("Authorization 请求头格式无效".into()))?;

        if !auth_str.starts_with("Bearer ") {
            return Err(AppError::Auth(
                "无效的 Authorization 格式，预期为 'Bearer YOUR_API_KEY'".into(),
            ));
        }

        Ok(auth_str[7..].to_string())
    }

    pub async fn proxy_request(
        &self,
        path: &str,
        method: Method,
        headers: &HeaderMap,
        body: Bytes,
    ) -> Result<Response, AppError> {
        // 提取API密钥
        let api_key = self.extract_api_key(headers)?;

        // 处理请求体
        let processed_body = self.process_request_body(&body, path)?;

        // 发送代理请求
        self.send_proxy_request(path, method, headers, processed_body, &api_key)
            .await
    }

    // 处理请求体，根据模型名称自动设置提供商
    pub fn process_request_body(&self, body: &[u8], path: &str) -> Result<Vec<u8>, AppError> {
        // 解析JSON
        let mut json_body: serde_json::Value = serde_json::from_slice(body)
            .map_err(|_| AppError::Parse("无效的 JSON 请求体".into()))?;

        // 仅在chat/completions和embeddings请求中检查模型
        if path.contains("chat/completions") || path.contains("embeddings") {
            // 尝试从请求中获取模型名称
            if let Some(model) = json_body.get("model").and_then(|v| v.as_str()) {
                // 查找配置中该模型对应的提供商
                if let Some(providers) = self.config.find_providers_for_model(model) {
                    tracing::info!("为模型 {} 设置提供商: {:?}", model, providers);
                    // 使用辅助函数设置提供商
                    request::set_providers(&mut json_body, providers);
                }
            }
        }
        //
        tracing::info!("修改后的请求体: {}", json_body);
        // 转回字节
        serde_json::to_vec(&json_body)
            .map_err(|_| AppError::Parse("无法序列化修改后的请求体".into()))
    }

    // 发送代理请求到OpenRouter
    pub async fn send_proxy_request(
        &self,
        path: &str,
        method: Method,
        headers: &HeaderMap,
        body: Vec<u8>,
        api_key: &str,
    ) -> Result<Response, AppError> {
        let url = format!("{}/{}", self.base_url, path);
        tracing::debug!("转发请求到: {}", url);

        // 构建请求
        let mut req_builder = self.client.request(method.clone(), &url);

        // 添加API密钥
        req_builder = req_builder.header("Authorization", format!("Bearer {}", api_key));

        // 复制其他请求头
        for (name, value) in headers.iter() {
            // 避免复制已处理的头
            if name != "host" && name != "authorization" && name != "content-length" {
                req_builder = req_builder.header(name, value);
            }
        }

        // 添加请求体
        if !body.is_empty() {
            req_builder = req_builder.body(body);
        }

        // 发送请求
        let resp = req_builder.send().await?;
        let status = resp.status();
        let headers = resp.headers().clone();

        // 将 reqwest 流映射错误为 std::io::Error
        let stream = resp
            .bytes_stream()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()));

        // 创建响应并将其转换为 Axum 期望的类型
        let mut response = Response::new(Body::wrap_stream(stream));
        *response.status_mut() = status;

        // 复制必要的响应头
        for (name, value) in headers.iter() {
            if name != "transfer-encoding" && name != "connection" {
                response.headers_mut().insert(name, value.clone());
            }
        }

        // 使用 into_response() 转换为 Axum 期望的类型
        return Ok(response.into_response());
    }
}

// 创建共享服务实例
pub type SharedOpenRouterService = Arc<OpenRouterService>;

pub fn create_service(config: Config) -> SharedOpenRouterService {
    Arc::new(OpenRouterService::new(config))
}
