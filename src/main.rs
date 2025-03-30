mod app;
mod config;
mod error;
mod handlers;
mod models;
mod services;
mod utils;

use tracing_subscriber::{fmt, EnvFilter};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 从环境变量初始化日志级别
    // 使用 RUST_LOG 环境变量控制日志级别
    // 例如: RUST_LOG=debug ./target/release/openrouter-proxy
    // 如果未设置，默认使用 info 级别
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    // 初始化日志
    fmt().with_env_filter(env_filter).with_target(true).init();

    // 加载配置
    let config = config::Config::new();

    // 运行应用
    app::run(config).await?;

    Ok(())
}
