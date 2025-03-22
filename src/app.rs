use crate::{
    config::Config,
    error::AppError,
    handlers::{health::health_check, proxy},
    services::openrouter,
};
use axum::{
    routing::{any, get},
    Router,
};
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
use tracing::info;

pub async fn run(config: Config) -> Result<(), AppError> {
    // 创建 OpenRouter 服务
    let service = openrouter::create_service(config.clone());

    // 配置 CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // 创建路由
    let app = Router::new()
        .route("/", get(health_check))
        // 只提供标准路径
        .route(
            "/api/v1/chat/completions",
            any(proxy::proxy_chat_completions),
        )
        .route("/api/v1/embeddings", any(proxy::proxy_embeddings))
        .route("/api/v1/models", any(proxy::proxy_models))
        .layer(cors)
        .with_state(service);

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));

    // 显示模型提供商映射配置
    if !config.model_provider_mapping.is_empty() {
        info!("模型到提供商的映射配置:");
        for (pattern, providers) in &config.model_provider_mapping {
            info!("  模式 '{}' -> 提供商: {:?}", pattern, providers);
        }
    } else {
        info!("未配置模型到提供商的映射");
    }

    // 使用 axum-server 启动 HTTP 或 HTTPS 服务器
    if config.https {
        let cert_path = config
            .cert_path
            .as_ref()
            .ok_or_else(|| AppError::Tls("使用 HTTPS 需要提供 --cert-path".into()))?;
        let key_path = config
            .key_path
            .as_ref()
            .ok_or_else(|| AppError::Tls("使用 HTTPS 需要提供 --key-path".into()))?;

        info!("API 代理服务运行在 https://{}", addr);

        // 使用 axum-server 启动 HTTPS 服务器
        let tls_config = axum_server::tls_rustls::RustlsConfig::from_pem_file(cert_path, key_path)
            .await
            .map_err(|e| AppError::Tls(format!("加载 TLS 配置失败: {}", e)))?;

        axum_server::bind_rustls(addr, tls_config)
            .serve(app.into_make_service())
            .await
            .map_err(|e| AppError::Tls(format!("HTTPS 服务器错误: {}", e)))?;
    } else {
        info!("API 代理服务运行在 http://{}", addr);
        // 使用 axum-server 启动 HTTP 服务器
        axum_server::bind(addr)
            .serve(app.into_make_service())
            .await
            .map_err(|e| AppError::Tls(format!("HTTP 服务器错误: {}", e)))?;
    }

    Ok(())
}
