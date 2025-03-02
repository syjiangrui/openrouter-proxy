use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<Provider>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(flatten)]
    pub other: HashMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
    #[serde(flatten)]
    pub other: HashMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingRequest {
    pub model: String,
    pub input: Value,
    #[serde(flatten)]
    pub other: HashMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provider {
    pub order: Vec<String>,
}

// 辅助方法用于设置提供商
pub fn set_provider(json: &mut Value, provider: &str) {
    // 检查是否已有 provider 字段
    if let Some(provider_obj) = json.get_mut("provider") {
        // 如果已经有 provider 对象，则更新 order 字段
        if let Some(provider_obj) = provider_obj.as_object_mut() {
            provider_obj.insert(
                "order".to_string(),
                serde_json::Value::Array(vec![serde_json::Value::String(provider.to_string())]),
            );
        } else {
            // 如果 provider 不是对象，则替换它
            *provider_obj = serde_json::json!({
                "order": [provider]
            });
        }
    } else {
        // 如果没有 provider 字段，创建一个新的
        json["provider"] = serde_json::json!({
            "order": [provider]
        });
    }
}
