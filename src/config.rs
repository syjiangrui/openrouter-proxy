use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug, Clone)]
#[command(about = "OpenRouter API 代理服务器 (伪装为 OpenAI API)")]
pub struct Config {
    /// 是否启用 HTTPS
    #[arg(long, default_value_t = false)]
    pub https: bool,

    /// SSL 证书路径
    #[arg(long)]
    pub cert_path: Option<PathBuf>,

    /// SSL 私钥路径
    #[arg(long)]
    pub key_path: Option<PathBuf>,

    /// 端口号
    #[arg(short, long, default_value_t = 3000)]
    pub port: u16,

    /// OpenRouter 基础 URL
    #[arg(long, default_value = "https://openrouter.ai/api/v1")]
    pub openrouter_base_url: String,
}

impl Config {
    pub fn new() -> Self {
        Config::parse()
    }
}
