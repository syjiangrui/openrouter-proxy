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
    // 加载配置
    let config = config::Config::new();

    // verbose 模式强制开启 debug 日志，否则使用 RUST_LOG 环境变量，默认 info
    let env_filter = if config.verbose {
        EnvFilter::new("debug")
    } else {
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"))
    };

    // 初始化日志
    fmt().with_env_filter(env_filter).with_target(true).init();

    // 运行应用
    app::run(config).await?;

    Ok(())
}
