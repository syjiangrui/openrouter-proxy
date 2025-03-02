mod app;
mod config;
mod error;
mod handlers;
mod models;
mod services;
mod utils;

use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    let subscriber = FmtSubscriber::builder()
        .with_max_level(tracing::Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("设置日志记录器失败");

    // 加载配置
    let config = config::Config::new();

    // 运行应用
    app::run(config).await?;

    Ok(())
}
