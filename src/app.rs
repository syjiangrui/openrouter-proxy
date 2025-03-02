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
use std::{net::SocketAddr, sync::Arc};
use tower_http::cors::{Any, CorsLayer};
use tracing::info;

pub async fn run(config: Config) -> Result<(), AppError> {
    // 创建 OpenRouter 服务
    let service = openrouter::create_service(config.openrouter_base_url.clone());

    // 配置 CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // 创建路由
    let app = Router::new()
        .route("/", get(health_check))
        // 带提供商路径
        .route(
            "/v1/:provider/chat/completions",
            any(proxy::proxy_with_provider),
        )
        .route("/v1/:provider/embeddings", any(proxy::proxy_with_provider))
        .route("/v1/:provider/models", any(proxy::proxy_with_provider))
        .route("/v1/:provider/:path", any(proxy::proxy_with_provider))
        // 不带提供商路径
        .route("/v1/chat/completions", any(proxy::proxy_without_provider))
        .route("/v1/embeddings", any(proxy::proxy_without_provider))
        .route("/v1/models", any(proxy::proxy_without_provider))
        .route("/v1/:path", any(proxy::proxy_without_provider))
        .layer(cors)
        .with_state(service);

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));

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
