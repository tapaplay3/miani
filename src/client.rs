use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tokio_stream::StreamExt;
use crate::config::Provider;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
    stream: bool,
    temperature: f32,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    delta: Option<Delta>,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Delta {
    content: Option<String>,
}

pub struct LLMClient {
    client: reqwest::Client,
    provider: Provider,
    endpoint: String,
    model: String,
}

impl LLMClient {
    pub fn new(provider: Provider, endpoint: String, model: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            provider,
            endpoint,
            model,
        }
    }

    pub fn provider(&self) -> Provider {
        self.provider
    }

    pub async fn check_connection(&self) -> Result<bool> {
        let url = format!("{}/models", self.endpoint);
        let response = self.client.get(&url).send().await;

        Ok(response.is_ok())
    }

    pub async fn get_loaded_model(&self) -> Result<String> {
        let url = format!("{}/models", self.endpoint);
        let response = self.client.get(&url).send().await?;

        #[derive(Deserialize)]
        struct ModelsResponse {
            data: Vec<ModelInfo>,
        }

        #[derive(Deserialize)]
        struct ModelInfo {
            id: String,
        }

        let models: ModelsResponse = response.json().await?;

        if let Some(model) = models.data.first() {
            Ok(model.id.clone())
        } else {
            Ok("No model loaded".to_string())
        }
    }

    pub fn get_model_name(&self) -> &str {
        &self.model
    }

    pub fn get_provider_name(&self) -> &str {
        self.provider.name()
    }

    pub fn get_setup_hint(&self) -> &str {
        self.provider.setup_hint()
    }

    pub async fn chat_stream<F>(&self, messages: Vec<Message>, mut on_chunk: F) -> Result<String>
    where
        F: FnMut(&str),
    {
        let url = format!("{}/chat/completions", self.endpoint);
        let request = ChatRequest {
            model: self.model.clone(),
            messages,
            stream: true,
            temperature: 0.7,
        };

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .context(format!("Failed to send request to {}", self.provider.name()))?;

        if !response.status().is_success() {
            anyhow::bail!("{} returned error: {}", self.provider.name(), response.status());
        }

        let mut stream = response.bytes_stream();
        let mut full_response = String::new();
        let mut buffer = String::new();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.context("Error reading stream")?;
            let text = String::from_utf8_lossy(&chunk);
            buffer.push_str(&text);

            // Process complete lines (SSE format: "data: {json}\n\n")
            while let Some(newline_pos) = buffer.find("\n\n") {
                let line = buffer[..newline_pos].trim().to_string();
                buffer = buffer[newline_pos + 2..].to_string();

                if line.is_empty() || line == "data: [DONE]" {
                    continue;
                }

                // Remove "data: " prefix
                let json_str = line.strip_prefix("data: ").unwrap_or(&line);

                if let Ok(response) = serde_json::from_str::<ChatResponse>(json_str) {
                    if let Some(choice) = response.choices.first() {
                        if let Some(delta) = &choice.delta {
                            if let Some(content) = &delta.content {
                                if !content.is_empty() {
                                    on_chunk(content);
                                    full_response.push_str(content);
                                }
                            }
                        }

                        if choice.finish_reason.is_some() {
                            return Ok(full_response);
                        }
                    }
                }
            }
        }

        Ok(full_response)
    }
}
