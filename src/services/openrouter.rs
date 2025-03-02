use crate::error::AppError;
use axum::{
    body::Body,
    http::{HeaderMap, Method},
    response::{IntoResponse, Response},
};
use futures_util::TryStreamExt;
use reqwest::Client;
use serde_json::Value;
use std::sync::Arc;

pub struct OpenRouterService {
    client: Client,
    base_url: String,
}

impl OpenRouterService {
    pub fn new(base_url: String) -> Self {
        let client = Client::builder()
            .build()
            .expect("Failed to create HTTP client");

        Self { client, base_url }
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

    // 处理请求体，如果需要可以修改模型名称
    pub fn process_request_body(
        &self,
        body: &[u8],
        provider: Option<&str>,
        should_modify_model: bool,
    ) -> Result<Vec<u8>, AppError> {
        // 如果不需要修改或者没有提供商，则直接返回原始请求体
        if !should_modify_model || provider.is_none() {
            return Ok(body.to_vec());
        }

        // 解析JSON
        let mut json_body: Value = serde_json::from_slice(body)
            .map_err(|_| AppError::Parse("无效的 JSON 请求体".into()))?;

        // 修改模型名称
        if let Some(model) = json_body.get("model").and_then(|m| m.as_str()) {
            if !model.contains("/") {
                let model_name = format!("{}/{}", provider.unwrap(), model);
                // 修改这行代码，使用正确的 serde_json::Value
                json_body["model"] = serde_json::Value::String(model_name);
            }
        }

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
            if name != "host" && name != "authorization" {
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

        // 处理流式响应
        if path.contains("chat/completions")
            && headers
                .get("content-type")
                .and_then(|v| v.to_str().ok())
                .map_or(false, |v| v.contains("text/event-stream"))
        {
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

        // 处理非流式响应
        match resp.bytes().await {
            Ok(bytes) => {
                let mut response = Response::new(Body::from(bytes));
                *response.status_mut() = status;

                // 复制必要的响应头
                for (name, value) in headers.iter() {
                    if name != "transfer-encoding" && name != "connection" {
                        response.headers_mut().insert(name, value.clone());
                    }
                }

                // 使用 into_response() 转换为 Axum 期望的类型
                Ok(response.into_response())
            }
            Err(e) => Err(AppError::Proxy(format!("读取API响应出错: {}", e))),
        }
    }
}

// 创建共享服务实例
pub type SharedOpenRouterService = Arc<OpenRouterService>;

pub fn create_service(base_url: String) -> SharedOpenRouterService {
    Arc::new(OpenRouterService::new(base_url))
}
