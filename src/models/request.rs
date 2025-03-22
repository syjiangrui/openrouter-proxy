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

// 辅助方法用于设置多个提供商
pub fn set_providers(json: &mut Value, providers: Vec<String>) {
    // 将String转换为Value
    let provider_values: Vec<Value> = providers
        .into_iter()
        .map(|p| serde_json::Value::String(p))
        .collect();

    // 检查是否已有 provider 字段
    if let Some(provider_obj) = json.get_mut("provider") {
        // 如果已经有 provider 对象，则更新 order 字段
        if let Some(provider_obj) = provider_obj.as_object_mut() {
            provider_obj.insert(
                "order".to_string(),
                serde_json::Value::Array(provider_values),
            );
        } else {
            // 如果 provider 不是对象，则替换它
            *provider_obj = serde_json::json!({
                "order": provider_values
            });
        }
    } else {
        // 如果没有 provider 字段，创建一个新的
        json["provider"] = serde_json::json!({
            "order": provider_values
        });
    }
}
