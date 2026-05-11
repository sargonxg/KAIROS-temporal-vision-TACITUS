use crate::{KairosError, Result};
use reqwest::Client;
use serde_json::{json, Value};
use std::env;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LlmProvider {
    Gemini,
    Ollama,
    Mock,
}

#[derive(Clone)]
pub struct LlmClient {
    provider: LlmProvider,
    http: Client,
    gemini_api_key: Option<String>,
    gemini_model: Option<String>,
}

impl LlmClient {
    pub fn from_env() -> Self {
        let provider = match env::var("KAIROS_LLM")
            .unwrap_or_else(|_| "gemini".to_string())
            .to_lowercase()
            .as_str()
        {
            "mock" => LlmProvider::Mock,
            "ollama" => LlmProvider::Ollama,
            _ => LlmProvider::Gemini,
        };
        Self {
            provider,
            http: Client::new(),
            gemini_api_key: None,
            gemini_model: None,
        }
    }

    pub fn mock() -> Self {
        Self {
            provider: LlmProvider::Mock,
            http: Client::new(),
            gemini_api_key: None,
            gemini_model: None,
        }
    }

    pub fn gemini_with_key(api_key: impl Into<String>, model: Option<String>) -> Self {
        Self {
            provider: LlmProvider::Gemini,
            http: Client::new(),
            gemini_api_key: Some(api_key.into()),
            gemini_model: model,
        }
    }

    pub fn is_mock(&self) -> bool {
        self.provider == LlmProvider::Mock
    }

    pub fn provider_name(&self) -> &'static str {
        match self.provider {
            LlmProvider::Gemini => "gemini",
            LlmProvider::Ollama => "ollama",
            LlmProvider::Mock => "mock",
        }
    }

    pub fn model_name(&self) -> String {
        match self.provider {
            LlmProvider::Gemini => self
                .gemini_model
                .as_deref()
                .map(str::trim)
                .filter(|model| !model.is_empty())
                .map(ToString::to_string)
                .or_else(|| env::var("KAIROS_GEMINI_MODEL").ok())
                .unwrap_or_else(|| "gemini-2.5-flash".to_string()),
            LlmProvider::Ollama => {
                env::var("KAIROS_OLLAMA_MODEL").unwrap_or_else(|_| "gemma2:2b".to_string())
            }
            LlmProvider::Mock => "deterministic-demo".to_string(),
        }
    }

    pub async fn complete_json(&self, prompt: &str) -> Result<Value> {
        match self.provider {
            LlmProvider::Mock => Ok(json!([])),
            LlmProvider::Gemini => self.complete_gemini(prompt).await,
            LlmProvider::Ollama => self.complete_ollama(prompt).await,
        }
    }

    pub async fn complete_json_with_schema(&self, prompt: &str, schema: Value) -> Result<Value> {
        match self.provider {
            LlmProvider::Mock => Ok(json!([])),
            LlmProvider::Gemini => self.complete_gemini_with_schema(prompt, Some(schema)).await,
            LlmProvider::Ollama => self.complete_ollama(prompt).await,
        }
    }

    async fn complete_gemini(&self, prompt: &str) -> Result<Value> {
        self.complete_gemini_with_schema(prompt, None).await
    }

    async fn complete_gemini_with_schema(
        &self,
        prompt: &str,
        schema: Option<Value>,
    ) -> Result<Value> {
        let key = match &self.gemini_api_key {
            Some(key) if !key.trim().is_empty() => key.trim().to_string(),
            _ => env::var("GEMINI_API_KEY").map_err(|_| {
                KairosError::Llm("GEMINI_API_KEY is required for KAIROS_LLM=gemini".to_string())
            })?,
        };
        let model = self.model_name();
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{model}:generateContent"
        );
        let mut generation_config = json!({
            "temperature": 0.1,
            "responseMimeType": "application/json"
        });
        if let Some(schema) = schema {
            generation_config["responseSchema"] = schema;
        }
        let body = json!({
            "contents": [{"parts": [{"text": prompt}]}],
            "generationConfig": generation_config
        });
        let res: Value = self
            .http
            .post(url)
            .header("x-goog-api-key", key)
            .json(&body)
            .send()
            .await
            .map_err(|e| KairosError::Llm(e.to_string()))?
            .error_for_status()
            .map_err(|e| KairosError::Llm(e.to_string()))?
            .json()
            .await
            .map_err(|e| KairosError::Llm(e.to_string()))?;
        let text = res["candidates"][0]["content"]["parts"][0]["text"]
            .as_str()
            .ok_or_else(|| KairosError::Llm(format!("Gemini returned no JSON text: {res}")))?;
        parse_jsonish(text)
    }

    async fn complete_ollama(&self, prompt: &str) -> Result<Value> {
        let host = env::var("OLLAMA_HOST").unwrap_or_else(|_| "http://localhost:11434".to_string());
        let model = env::var("KAIROS_OLLAMA_MODEL").unwrap_or_else(|_| "gemma2:2b".to_string());
        let res: Value = self
            .http
            .post(format!("{}/api/generate", host.trim_end_matches('/')))
            .json(&json!({"model": model, "prompt": prompt, "stream": false, "format": "json"}))
            .send()
            .await
            .map_err(|e| KairosError::Llm(e.to_string()))?
            .error_for_status()
            .map_err(|e| KairosError::Llm(e.to_string()))?
            .json()
            .await
            .map_err(|e| KairosError::Llm(e.to_string()))?;
        let text = res["response"]
            .as_str()
            .ok_or_else(|| KairosError::Llm(format!("Ollama returned no response: {res}")))?;
        parse_jsonish(text)
    }
}

fn parse_jsonish(text: &str) -> Result<Value> {
    let trimmed = text.trim();
    let stripped = trimmed
        .strip_prefix("```json")
        .or_else(|| trimmed.strip_prefix("```"))
        .unwrap_or(trimmed)
        .trim()
        .strip_suffix("```")
        .unwrap_or(trimmed)
        .trim();
    serde_json::from_str(stripped).map_err(|e| {
        let preview: String = stripped.chars().take(500).collect();
        KairosError::Llm(format!("invalid JSON from model: {e}; preview: {preview}"))
    })
}
