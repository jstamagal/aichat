use crate::client::model::{ModelData, ProviderModels};
use anyhow::{Context, Result};
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::time::{Duration, SystemTime};

const MODELS_DEV_API_URL: &str = "https://models.dev/api.json";
const CACHE_TTL_SECONDS: u64 = 3600; // 1 hour

#[derive(Debug, Clone, Deserialize)]
pub struct ModelsDevResponse {
    #[serde(flatten)]
    pub providers: HashMap<String, ProviderData>,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)] // Fields are needed for deserialization
pub struct ProviderData {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub env: Vec<String>,
    pub npm: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub doc: Option<String>,
    pub models: HashMap<String, ModelDataDev>,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)] // Fields are needed for deserialization
pub struct ModelDataDev {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub family: Option<String>,
    #[serde(default)]
    pub attachment: bool,
    #[serde(default)]
    pub reasoning: bool,
    #[serde(default, rename = "tool_call")]
    pub tool_call: bool,
    #[serde(default)]
    pub temperature: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub knowledge: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_updated: Option<String>,
    #[serde(default)]
    pub modalities: ModelModalities,
    #[serde(default)]
    pub open_weights: bool,
    #[serde(default)]
    pub cost: ModelCost,
    #[serde(default)]
    pub limit: ModelLimit,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[allow(dead_code)] // Fields may not be used in all contexts
pub struct ModelModalities {
    #[serde(default)]
    pub input: Vec<String>,
    #[serde(default)]
    pub output: Vec<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[allow(dead_code)] // Some cost fields may not be used yet
pub struct ModelCost {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_read: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_write: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_audio: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_audio: Option<f64>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[allow(dead_code)] // output field may not be used in all contexts
pub struct ModelLimit {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<u64>,
}

#[derive(Debug, Clone)]
struct CachedData {
    data: Vec<ProviderModels>,
    fetched_at: SystemTime,
}

static CACHE: std::sync::LazyLock<std::sync::Mutex<Option<CachedData>>> =
    std::sync::LazyLock::new(|| std::sync::Mutex::new(None));

/// Provider name mapping from models.dev IDs to aichat provider names
fn map_provider_name(models_dev_id: &str) -> String {
    match models_dev_id {
        "anthropic" => "claude".to_string(),
        "google" => "gemini".to_string(),
        "amazon-bedrock" => "bedrock".to_string(),
        "azure" => "azure-openai".to_string(),
        "cloudflare-workers-ai" => "cloudflare".to_string(),
        "moonshotai" => "moonshot".to_string(),
        "moonshotai-cn" => "moonshot".to_string(),
        "alibaba" => "qianwen".to_string(),
        "alibaba-cn" => "qianwen".to_string(),
        "zai" | "zai-coding-plan" => "zhipuai".to_string(),
        _ => models_dev_id.to_string(),
    }
}

/// Determine model type from models.dev data
fn determine_model_type(model: &ModelDataDev) -> String {
    // Check if it's an embedding model - typically has "embed" in name or specific patterns
    let name_lower = model.name.to_lowercase();
    let id_lower = model.id.to_lowercase();
    
    if name_lower.contains("embed") 
        || id_lower.contains("embed") 
        || name_lower.contains("embedding")
        || id_lower.contains("embedding") {
        return "embedding".to_string();
    }
    
    // Check if it's a reranker model
    if name_lower.contains("rerank") || id_lower.contains("rerank") {
        return "reranker".to_string();
    }
    
    // Default to chat
    "chat".to_string()
}

/// Convert models.dev format to aichat's ProviderModels format
pub fn convert_models_dev_to_provider_models(
    response: &ModelsDevResponse,
) -> Vec<ProviderModels> {
    let mut result = Vec::new();
    
    for (provider_id, provider_data) in &response.providers {
        // Skip deprecated providers
        if provider_data.models.is_empty() {
            continue;
        }
        
        let aichat_provider = map_provider_name(provider_id);
        let mut models = Vec::new();
        
        for (model_id, model_dev) in &provider_data.models {
            // Skip deprecated models
            if model_dev.status.as_deref() == Some("deprecated") {
                continue;
            }
            
            let model_type = determine_model_type(model_dev);
            
            // Determine max_input_tokens from limit.context or limit.input
            let max_input_tokens = model_dev
                .limit
                .input
                .or(model_dev.limit.context)
                .map(|v| v as usize);
            
            // Determine max_output_tokens from limit.output
            let max_output_tokens = model_dev
                .limit
                .output
                .map(|v| v as isize);
            
            // Check if supports vision (image in input modalities)
            let supports_vision = model_dev.modalities.input.contains(&"image".to_string())
                || model_dev.modalities.input.contains(&"vision".to_string());
            
            // Check if supports function calling
            let supports_function_calling = model_dev.tool_call;
            
            // Convert prices (already per million tokens in models.dev)
            let input_price = model_dev.cost.input;
            let output_price = model_dev.cost.output;
            
            // Create model data using serde_json to work around private fields
            let model_json = serde_json::json!({
                "name": model_id,
                "type": model_type,
                "max_input_tokens": max_input_tokens,
                "input_price": input_price,
                "output_price": output_price,
                "max_output_tokens": max_output_tokens,
                "require_max_tokens": false,
                "supports_vision": supports_vision,
                "supports_function_calling": supports_function_calling,
                "max_tokens_per_chunk": None::<usize>,
                "default_chunk_size": None::<usize>,
                "max_batch_size": None::<usize>,
            });
            
            let mut model_data: ModelData = serde_json::from_value(model_json)
                .unwrap_or_else(|_| ModelData::new(model_id));
            
            // Set embedding-specific fields if this is an embedding model
            if model_data.model_type == "embedding" {
                // Use context limit as max_tokens_per_chunk if available
                if let Some(context) = model_dev.limit.context {
                    model_data.max_tokens_per_chunk = Some(context as usize);
                } else if let Some(input) = model_dev.limit.input {
                    model_data.max_tokens_per_chunk = Some(input as usize);
                }
                
                // Set defaults for embedding models
                if model_data.default_chunk_size.is_none() {
                    model_data.default_chunk_size = Some(1000);
                }
                if model_data.max_batch_size.is_none() {
                    model_data.max_batch_size = Some(100);
                }
            }
            
            models.push(model_data);
        }
        
        if !models.is_empty() {
            result.push(ProviderModels {
                provider: aichat_provider,
                models,
            });
        }
    }
    
    result
}

/// Fetch models from models.dev API
pub async fn fetch_models_dev(url: &str) -> Result<ModelsDevResponse> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .context("Failed to create HTTP client")?;
    
    let response = client
        .get(url)
        .send()
        .await
        .with_context(|| format!("Failed to fetch models from '{}'", url))?;
    
    if !response.status().is_success() {
        anyhow::bail!(
            "HTTP error {} when fetching models from '{}'",
            response.status(),
            url
        );
    }
    
    let json: Value = response
        .json()
        .await
        .context("Failed to parse JSON response from models.dev")?;
    
    // The API returns a flat object with provider IDs as keys
    // We need to deserialize it properly
    let providers: HashMap<String, ProviderData> = serde_json::from_value(json)
        .context("Failed to deserialize models.dev response")?;
    
    Ok(ModelsDevResponse { providers })
}

/// Get cached models or fetch fresh ones
pub async fn get_models_dev(url: Option<&str>) -> Result<Vec<ProviderModels>> {
    let url = url.unwrap_or(MODELS_DEV_API_URL);
    
    // Check cache first
    {
        let cache_guard = CACHE.lock().unwrap();
        if let Some(cached) = cache_guard.as_ref() {
            if let Ok(elapsed) = cached.fetched_at.elapsed() {
                if elapsed.as_secs() < CACHE_TTL_SECONDS {
                    return Ok(cached.data.clone());
                }
            }
        }
    }
    
    // Fetch fresh data
    let response = fetch_models_dev(url).await?;
    let provider_models = convert_models_dev_to_provider_models(&response);
    
    // Update cache
    {
        let mut cache_guard = CACHE.lock().unwrap();
        *cache_guard = Some(CachedData {
            data: provider_models.clone(),
            fetched_at: SystemTime::now(),
        });
    }
    
    Ok(provider_models)
}

/// Clear the cache (useful for testing or manual refresh)
#[allow(dead_code)] // May be used by external code or future features
pub fn clear_cache() {
    let mut cache_guard = CACHE.lock().unwrap();
    *cache_guard = None;
}

