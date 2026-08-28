use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::error::Error;

/// Represents a chat message in the conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String, // "system", "user", "assistant"
    pub content: String,
}

/// Represents a response from the LLM provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    pub content: String,
    pub model: String,
    pub usage: Option<UsageInfo>,
}

/// Token usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageInfo {
    pub prompt_tokens: Option<u32>,
    pub completion_tokens: Option<u32>,
    pub total_tokens: Option<u32>,
}

/// Configuration for LLM providers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub api_key: String,
    pub base_url: Option<String>,
    pub model: String,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub top_p: Option<f32>,
    pub stream: bool,
}

/// Abstract trait for LLM providers
#[async_trait]
pub trait LLMProvider: Send + Sync {
    /// Send a chat completion request to the provider
    async fn chat_completion(
        &self,
        messages: Vec<ChatMessage>,
        config: &ProviderConfig,
    ) -> Result<ChatResponse, Box<dyn Error + Send + Sync>>;

    /// Get the provider name
    fn name(&self) -> &'static str;
}

/// OpenRouter provider implementation
pub struct OpenRouterProvider;

#[async_trait]
impl LLMProvider for OpenRouterProvider {
    async fn chat_completion(
        &self,
        messages: Vec<ChatMessage>,
        config: &ProviderConfig,
    ) -> Result<ChatResponse, Box<dyn Error + Send + Sync>> {
        // OpenRouter API endpoint
        let base_url = config.base_url.as_deref().unwrap_or("https://openrouter.ai/api/v1");
        let url = format!("{}/chat/completions", base_url);

        // Prepare request
        let client = reqwest::Client::new();
        let request_body = serde_json::json!({
            "model": config.model,
            "messages": messages,
            "temperature": config.temperature.unwrap_or(0.7),
            "max_tokens": config.max_tokens,
            "top_p": config.top_p.unwrap_or(1.0),
            "stream": config.stream,
        });

        // Send request
        let response = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", config.api_key))
            .header("HTTP-Referer", "https://agent-os.local") // Optional, for OpenRouter rankings
            .header("X-Title", "Agent OS")
            .json(&request_body)
            .send()
            .await?
            .error_for_status()?;

        // Parse response
        let chat_response: serde_json::Value = response.json().await?;

        Ok(ChatResponse {
            content: chat_response["choices"][0]["message"]["content"]
                .as_str()
                .unwrap_or("")
                .to_string(),
            model: chat_response["model"]
                .as_str()
                .unwrap_or(&config.model)
                .to_string(),
            usage: Some(UsageInfo {
                prompt_tokens: chat_response["usage"]["prompt_tokens"].as_u64().map(|v| v as u32),
                completion_tokens: chat_response["usage"]["completion_tokens"]
                    .as_u64()
                    .map(|v| v as u32),
                total_tokens: chat_response["usage"]["total_tokens"].as_u64().map(|v| v as u32),
            }),
        })
    }

    fn name(&self) -> &'static str {
        "openrouter"
    }
}

/// NVIDIA NIM provider implementation
pub struct NIMProvider;

#[async_trait]
impl LLMProvider for NIMProvider {
    async fn chat_completion(
        &self,
        messages: Vec<ChatMessage>,
        config: &ProviderConfig,
    ) -> Result<ChatResponse, Box<dyn Error + Send + Sync>> {
        // NIM API endpoint format: https://ai.api.nvidia.com/v1/{model}/chat/completions
        let base_url = config.base_url.as_deref().unwrap_or("https://ai.api.nvidia.com/v1");
        let url = format!("{}/chat/completions", base_url);

        // Prepare request
        let client = reqwest::Client::new();
        let request_body = serde_json::json!({
            "model": config.model,
            "messages": messages,
            "temperature": config.temperature.unwrap_or(0.7),
            "max_tokens": config.max_tokens,
            "top_p": config.top_p.unwrap_or(1.0),
            "stream": config.stream,
        });

        // Send request
        let response = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", config.api_key))
            .header("Accept", "application/json")
            .json(&request_body)
            .send()
            .await?
            .error_for_status()?;

        // Parse response
        let chat_response: serde_json::Value = response.json().await?;

        Ok(ChatResponse {
            content: chat_response["choices"][0]["message"]["content"]
                .as_str()
                .unwrap_or("")
                .to_string(),
            model: chat_response["model"]
                .as_str()
                .unwrap_or(&config.model)
                .to_string(),
            usage: Some(UsageInfo {
                prompt_tokens: chat_response["usage"]["prompt_tokens"].as_u64().map(|v| v as u32),
                completion_tokens: chat_response["usage"]["completion_tokens"]
                    .as_u64()
                    .map(|v| v as u32),
                total_tokens: chat_response["usage"]["total_tokens"].as_u64().map(|v| v as u32),
            }),
        })
    }

    fn name(&self) -> &'static str {
        "nim"
    }
}

/// Factory to create provider instances
pub struct ProviderFactory;

impl ProviderFactory {
    pub fn create_provider(provider_type: &str) -> Result<Box<dyn LLMProvider>, String> {
        match provider_type.to_lowercase().as_str() {
            "openrouter" => Ok(Box::new(OpenRouterProvider)),
            "nim" | "nvidia" | "nvidia-nim" => Ok(Box::new(NIMProvider)),
            _ => Err(format!("Unknown provider type: {}", provider_type)),
        }
    }
}

/// Provider manager that handles provider selection and fallback
pub struct ProviderManager {
    providers: std::collections::HashMap<String, Box<dyn LLMProvider>>,
}

impl ProviderManager {
    pub fn new() -> Self {
        let mut managers = ProviderManager {
            providers: std::collections::HashMap::new(),
        };

        // Register built-in providers
        managers.register_provider("openrouter", Box::new(OpenRouterProvider));
        managers.register_provider("nim", Box::new(NIMProvider));
        managers.register_provider("nvidia", Box::new(NIMProvider));
        managers.register_provider("nvidia-nim", Box::new(NIMProvider));

        managers
    }

    pub fn register_provider(&mut self, name: &str, provider: Box<dyn LLMProvider>) {
        self.providers.insert(name.to_string(), provider);
    }

    pub fn get_provider(&self, name: &str) -> Option<&Box<dyn LLMProvider>> {
        self.providers.get(name)
    }

    pub fn list_providers(&self) -> Vec<String> {
        self.providers.keys().cloned().collect()
    }
}

// Unit tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_factory() {
        let openrouter = ProviderFactory::create_provider("openrouter").unwrap();
        assert_eq!(openrouter.name(), "openrouter");

        let nim = ProviderFactory::create_provider("nim").unwrap();
        assert_eq!(nim.name(), "nim");

        let nvidia = ProviderFactory::create_provider("nvidia").unwrap();
        assert_eq!(nvidia.name(), "nim");

        let result = ProviderFactory::create_provider("unknown");
        assert!(result.is_err());
    }

    #[test]
    fn test_provider_manager() {
        let manager = ProviderManager::new();
        assert!(manager.get_provider("openrouter").is_some());
        assert!(manager.get_provider("nim").is_some());
        assert!(manager.get_provider("nvidia").is_some());

        let providers = manager.list_providers();
        assert!(providers.contains(&"openrouter".to_string()));
        assert!(providers.contains(&"nim".to_string()));
        assert!(providers.contains(&"nvidia".to_string()));
    }
}