use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug, Clone)]
#[command(about = "OpenRouter API 代理服务器 (OpenAI API)")]
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

    /// 监听的 IP 地址
    #[arg(short, long, default_value = "0.0.0.0")]
    pub ip: String,

    /// 端口号
    #[arg(short, long, default_value_t = 3000)]
    pub port: u16,

    /// OpenRouter 基础 URL
    #[arg(long, default_value = "https://openrouter.ai/api/v1")]
    pub openrouter_base_url: String,

    /// 模型到提供商的映射配置 (格式: 模式=提供商1,提供商2)
    #[arg(long, value_parser = parse_model_provider_mapping)]
    pub model_provider_mapping: Vec<(String, Vec<String>)>,
}

// 解析模型到提供商的映射配置
fn parse_model_provider_mapping(s: &str) -> Result<(String, Vec<String>), String> {
    let parts: Vec<&str> = s.split('=').collect();
    if parts.len() != 2 {
        return Err("模型提供商映射格式应为: 模式=提供商1,提供商2".to_string());
    }

    let model_pattern = parts[0].trim().to_string();
    let providers: Vec<String> = parts[1].split(',').map(|p| p.trim().to_string()).collect();

    if providers.is_empty() {
        return Err("至少需要指定一个提供商".to_string());
    }

    Ok((model_pattern, providers))
}

impl Config {
    pub fn new() -> Self {
        Config::parse()
    }

    // 为给定模型查找提供商
    pub fn find_providers_for_model(&self, model: &str) -> Option<Vec<String>> {
        for (pattern, providers) in &self.model_provider_mapping {
            if model_matches_pattern(model, pattern) {
                return Some(providers.clone());
            }
        }
        None
    }
}

// 检查模型名称是否匹配模式
fn model_matches_pattern(model: &str, pattern: &str) -> bool {
    // 处理简单的通配符匹配
    // 如 *anthropic/claude* 会匹配任何包含 "anthropic/claude" 的模型名称
    if pattern.starts_with('*') && pattern.ends_with('*') {
        let inner = &pattern[1..pattern.len() - 1];
        model.contains(inner)
    } else if pattern.starts_with('*') {
        let suffix = &pattern[1..];
        model.ends_with(suffix)
    } else if pattern.ends_with('*') {
        let prefix = &pattern[..pattern.len() - 1];
        model.starts_with(prefix)
    } else {
        model == pattern
    }
}
