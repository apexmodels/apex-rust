use apex_core::models::{ChatMessage, ThinkingDepth};
use futures::Stream;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::pin::Pin;

#[derive(Debug, Clone, Serialize)]
pub struct OllamaChatOptions {
    pub num_predict: u32,
    pub temperature: f32,
}

#[derive(Debug, Clone, Serialize)]
pub struct OllamaChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub options: OllamaChatOptions,
    pub stream: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OllamaMessagePayload {
    #[serde(default)]
    pub role: String,
    #[serde(default)]
    pub content: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OllamaChatChunk {
    pub message: Option<OllamaMessagePayload>,
    #[serde(default)]
    pub done: bool,
}

pub struct OllamaClient {
    client: Client,
    base_url: String,
    model: String,
}

impl Default for OllamaClient {
    fn default() -> Self {
        Self::new("http://localhost:11434", "apex-phi-think-14b:latest")
    }
}

impl OllamaClient {
    pub fn new(base_url: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.into(),
            model: model.into(),
        }
    }

    pub async fn stream_chat(
        &self,
        messages: Vec<ChatMessage>,
        depth: ThinkingDepth,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<String, String>> + Send>>, String> {
        let endpoint = format!("{}/api/chat", self.base_url.trim_end_matches('/'));
        let request_payload = OllamaChatRequest {
            model: self.model.clone(),
            messages,
            options: OllamaChatOptions {
                num_predict: depth.num_predict(),
                temperature: depth.temperature(),
            },
            stream: true,
        };

        let response = self
            .client
            .post(&endpoint)
            .json(&request_payload)
            .send()
            .await
            .map_err(|e| format!("Failed to connect to Ollama at {}: {}", endpoint, e))?;

        if !response.status().is_success() {
            let status = response.status();
            let err_body = response.text().await.unwrap_or_default();
            return Err(format!("Ollama returned HTTP {}: {}", status, err_body));
        }

        let byte_stream = response.bytes_stream();
        let s = async_stream::stream! {
            use futures::StreamExt;
            let mut stream = byte_stream;
            let mut line_buf = String::new();

            while let Some(item) = stream.next().await {
                match item {
                    Ok(bytes) => {
                        line_buf.push_str(&String::from_utf8_lossy(&bytes));
                        while let Some(newline_pos) = line_buf.find('\n') {
                            let line = line_buf[..newline_pos].trim().to_string();
                            line_buf = line_buf[newline_pos + 1..].to_string();
                            if !line.is_empty() {
                                if let Ok(chunk) = serde_json::from_str::<OllamaChatChunk>(&line) {
                                    if let Some(msg) = chunk.message {
                                        if !msg.content.is_empty() {
                                            yield Ok(msg.content);
                                        }
                                    }
                                    if chunk.done {
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        yield Err(format!("Network chunk error: {}", e));
                        break;
                    }
                }
            }
        };

        Ok(Box::pin(s))
    }
}
