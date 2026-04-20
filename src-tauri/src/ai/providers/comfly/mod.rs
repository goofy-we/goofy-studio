use std::process::Command;
use base64::Engine;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use async_trait::async_trait;
use tracing::info;

use crate::ai::{AIProvider, AIError, GenerateRequest};
use crate::ai::providers::COMFLY_BASE_URL;

#[derive(Debug, Serialize)]
struct ImageRequest {
    model: String,
    prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    response_format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    aspect_ratio: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    image: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    image_size: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ImageResponse {
    data: Vec<ImageData>,
}

#[derive(Debug, Deserialize)]
struct ImageData {
    url: Option<String>,
    #[serde(rename = "b64_json")]
    b64_json: Option<String>,
}

pub struct ComflyProvider {
    api_key: std::sync::RwLock<Option<String>>,
}

impl ComflyProvider {
    pub fn new() -> Self {
        Self {
            api_key: std::sync::RwLock::new(None),
        }
    }

    fn is_grok_image_model(model: &str) -> bool {
        model.starts_with("grok-")
    }

    fn mime_type_from_content_type(content_type: Option<&str>) -> &'static str {
        match content_type
            .and_then(|value| value.split(';').next())
            .map(str::trim)
            .unwrap_or_default()
            .to_ascii_lowercase()
            .as_str()
        {
            "image/jpeg" | "image/jpg" => "image/jpeg",
            "image/webp" => "image/webp",
            "image/gif" => "image/gif",
            "image/bmp" => "image/bmp",
            "image/svg+xml" => "image/svg+xml",
            _ => "image/png",
        }
    }

    fn mime_type_from_url(url: &str) -> &'static str {
        let lower = url.to_ascii_lowercase();
        if lower.contains(".jpg") || lower.contains(".jpeg") {
            return "image/jpeg";
        }
        if lower.contains(".webp") {
            return "image/webp";
        }
        if lower.contains(".gif") {
            return "image/gif";
        }
        if lower.contains(".bmp") {
            return "image/bmp";
        }
        if lower.contains(".svg") {
            return "image/svg+xml";
        }
        "image/png"
    }

    fn download_image_via_curl(source_url: &str) -> Result<String, String> {
        let output = Command::new("curl.exe")
            .args(["-L", "--silent", "--show-error", source_url])
            .output()
            .map_err(|e| format!("Failed to spawn curl.exe for generated image: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            return Err(if stderr.is_empty() {
                format!("curl.exe failed with status {}", output.status)
            } else {
                format!("curl.exe failed: {}", stderr)
            });
        }

        if output.stdout.is_empty() {
            return Err("curl.exe returned empty image payload".to_string());
        }

        Ok(format!(
            "data:{};base64,{}",
            Self::mime_type_from_url(source_url),
            base64::engine::general_purpose::STANDARD.encode(&output.stdout)
        ))
    }

    async fn download_image_as_data_url(source_url: &str) -> Result<String, AIError> {
        let client = Client::builder()
            .user_agent("Storyboard-Copilot/0.1")
            .build()
            .map_err(|e| AIError::Provider(format!("Failed to build image download client: {}", e)))?;

        let response = match client
            .get(source_url)
            .send()
            .await
        {
            Ok(response) => response,
            Err(reqwest_error) => {
                return Self::download_image_via_curl(source_url).map_err(|curl_error| {
                    AIError::Provider(format!(
                        "Failed to fetch generated image: {}; fallback download also failed: {}",
                        reqwest_error, curl_error
                    ))
                });
            }
        };

        if !response.status().is_success() {
            return Err(AIError::Provider(format!("Failed to fetch generated image: HTTP {}", response.status())));
        }

        let mime_type = Self::mime_type_from_content_type(
            response
                .headers()
                .get(reqwest::header::CONTENT_TYPE)
                .and_then(|value| value.to_str().ok()),
        );

        let bytes = match response.bytes().await {
            Ok(bytes) => bytes,
            Err(reqwest_error) => {
                return Self::download_image_via_curl(source_url).map_err(|curl_error| {
                    AIError::Provider(format!(
                        "Failed to read generated image bytes: {}; fallback download also failed: {}",
                        reqwest_error, curl_error
                    ))
                });
            }
        };

        Ok(format!(
            "data:{};base64,{}",
            mime_type,
            base64::engine::general_purpose::STANDARD.encode(bytes)
        ))
    }
}

impl Default for ComflyProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AIProvider for ComflyProvider {
    fn name(&self) -> &str {
        "comfly"
    }

    fn supports_model(&self, _model: &str) -> bool {
        true
    }

    fn list_models(&self) -> Vec<String> {
        vec![
            "grok-4.2-image".to_string(),
            "grok-4.1-image".to_string(),
            "nano-banana-2".to_string(),
            "gemini-3.1-flash-image-preview".to_string(),
        ]
    }

    async fn set_api_key(&self, api_key: String) -> Result<(), AIError> {
        let mut key = self.api_key.write().unwrap();
        *key = Some(api_key);
        info!("API key set for comfly provider");
        Ok(())
    }

    async fn generate(&self, request: GenerateRequest) -> Result<String, AIError> {
        let api_key = {
            let key = self.api_key.read().unwrap();
            key.clone()
        };

        let api_key = api_key.ok_or_else(|| AIError::Provider("API key not set".to_string()))?;

        let client = Client::new();
        let is_grok_model = Self::is_grok_image_model(&request.model);

        let image_request = ImageRequest {
            model: request.model.clone(),
            prompt: request.prompt,
            response_format: if is_grok_model {
                None
            } else {
                Some("url".to_string())
            },
            aspect_ratio: if request.aspect_ratio.is_empty() {
                None
            } else {
                Some(request.aspect_ratio.clone())
            },
            image: request.reference_images.filter(|values| !values.is_empty()),
            image_size: if is_grok_model {
                None
            } else {
                Some(request.size.clone())
            },
        };

        let response = client
            .post(format!("{}{}", COMFLY_BASE_URL, "/v1/images/generations"))
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&image_request)
            .send()
            .await
            .map_err(|e| AIError::Network(e))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(AIError::Provider(format!("API error: {}", error_text)));
        }

        let image_response: ImageResponse = response.json().await.map_err(AIError::Network)?;

        let image = image_response
            .data
            .first()
            .ok_or_else(|| AIError::Provider("No image from API".to_string()))?;

        if let Some(url) = image.url.as_deref().filter(|value| !value.trim().is_empty()) {
            return Self::download_image_as_data_url(url).await;
        }

        if let Some(raw_b64) = image.b64_json.as_deref().filter(|value| !value.trim().is_empty()) {
            return Ok(format!("data:image/png;base64,{}", raw_b64));
        }

        Err(AIError::Provider("No image payload from API".to_string()))
    }
}