use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::application::errors::AppError;

const OPENROUTER_URL: &str = "https://openrouter.ai/api/v1/chat/completions";
const USER_AGENT: &str = "Brewlog/1.0";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(90);

const ROASTER_PROMPT: &str = r#"Extract coffee roaster information from this input. Use web search to look up any details you cannot determine from the input alone (e.g. the roaster's website, location, or background). Return a JSON object with these fields (only include fields you can identify with confidence):
- "name": the roaster's name
- "country": the country the roaster is based in
- "city": the city the roaster is based in
- "homepage": the roaster's website URL

Return ONLY the JSON object, no other text."#;

const ROAST_PROMPT: &str = r#"Extract coffee roast information from this input. Use web search to look up any details you cannot determine from the input alone (e.g. origin, region, producer, processing method, tasting notes). Return a JSON object with these fields (only include fields you can identify with confidence):
- "roaster_name": the name of the roaster
- "name": the name of this specific coffee/roast
- "origin": the country of origin of the coffee beans
- "region": the region within the origin country
- "producer": the farm, estate, or cooperative that produced the beans
- "process": the processing method (e.g. Washed, Natural, Honey, Anaerobic)
- "tasting_notes": an array of flavour/tasting notes in Title Case (e.g. ["Blueberry", "Jasmine", "Dark Chocolate"])

Return ONLY the JSON object, no other text."#;

const SCAN_PROMPT: &str = r#"Extract both the coffee roaster and the roast information from this input. Use web search to look up any details you cannot determine from the input alone (e.g. the roaster's website, location, tasting notes, processing method). Return a JSON object with two top-level keys:

{
  "roaster": {
    "name": "the roaster's name",
    "country": "country the roaster is based in",
    "city": "city the roaster is based in",
    "homepage": "the roaster's website URL"
  },
  "roast": {
    "name": "the name of this specific coffee/roast",
    "origin": "the country of origin of the beans",
    "region": "the region within the origin country",
    "producer": "the farm, estate, or cooperative",
    "process": "processing method (e.g. Washed, Natural, Honey, Anaerobic)",
    "tasting_notes": ["Array", "Of", "Flavour Notes In Title Case"]
  }
}

Only include fields you can identify with confidence. Each tasting note must be in Title Case. Return ONLY the JSON object, no other text."#;

// --- Public types ---

#[derive(Debug, Deserialize)]
pub struct ExtractionInput {
    pub image: Option<String>,
    pub prompt: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedRoaster {
    pub name: Option<String>,
    pub country: Option<String>,
    pub city: Option<String>,
    pub homepage: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedRoast {
    pub roaster_name: Option<String>,
    pub name: Option<String>,
    pub origin: Option<String>,
    pub region: Option<String>,
    pub producer: Option<String>,
    pub process: Option<String>,
    pub tasting_notes: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedBagScan {
    pub roaster: ExtractedRoaster,
    pub roast: ExtractedRoast,
}

// --- Public functions ---

pub async fn extract_roaster(
    client: &reqwest::Client,
    api_key: &str,
    model: &str,
    input: &ExtractionInput,
) -> Result<ExtractedRoaster, AppError> {
    let content = call_openrouter(client, api_key, model, ROASTER_PROMPT, input).await?;
    let json = extract_json(&content);

    serde_json::from_str(json).map_err(|e| {
        AppError::unexpected(format!("Failed to parse AI response as roaster data: {e}"))
    })
}

pub async fn extract_roast(
    client: &reqwest::Client,
    api_key: &str,
    model: &str,
    input: &ExtractionInput,
) -> Result<ExtractedRoast, AppError> {
    let content = call_openrouter(client, api_key, model, ROAST_PROMPT, input).await?;
    let json = extract_json(&content);

    serde_json::from_str(json).map_err(|e| {
        AppError::unexpected(format!("Failed to parse AI response as roast data: {e}"))
    })
}

pub async fn extract_bag_scan(
    client: &reqwest::Client,
    api_key: &str,
    model: &str,
    input: &ExtractionInput,
) -> Result<ExtractedBagScan, AppError> {
    let content = call_openrouter(client, api_key, model, SCAN_PROMPT, input).await?;
    let json = extract_json(&content);

    serde_json::from_str(json).map_err(|e| {
        AppError::unexpected(format!("Failed to parse AI response as bag scan data: {e}"))
    })
}

// --- Internal helpers ---

async fn call_openrouter(
    client: &reqwest::Client,
    api_key: &str,
    model: &str,
    system_prompt: &str,
    input: &ExtractionInput,
) -> Result<String, AppError> {
    let has_image = input.image.as_ref().is_some_and(|s| !s.trim().is_empty());
    let has_prompt = input.prompt.as_ref().is_some_and(|s| !s.trim().is_empty());

    if !has_image && !has_prompt {
        return Err(AppError::validation(
            "Provide either an image or a text prompt",
        ));
    }

    let mut content_parts = vec![ContentPart::Text {
        text: system_prompt.to_string(),
    }];

    if let Some(image) = &input.image
        && !image.trim().is_empty()
    {
        content_parts.push(ContentPart::ImageUrl {
            image_url: ImageUrlDetail { url: image.clone() },
        });
    }

    if let Some(prompt) = &input.prompt
        && !prompt.trim().is_empty()
    {
        content_parts.push(ContentPart::Text {
            text: prompt.clone(),
        });
    }

    let request_body = ChatRequest {
        model: model.to_string(),
        messages: vec![Message {
            role: "user".to_string(),
            content: content_parts,
        }],
    };

    let response = client
        .post(OPENROUTER_URL)
        .header("User-Agent", USER_AGENT)
        .header("Authorization", format!("Bearer {api_key}"))
        .timeout(REQUEST_TIMEOUT)
        .json(&request_body)
        .send()
        .await
        .map_err(|e| AppError::unexpected(format!("OpenRouter request failed: {e}")))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response
            .text()
            .await
            .unwrap_or_else(|_| "(unreadable body)".to_string());
        return Err(AppError::unexpected(format!(
            "OpenRouter returned status {status}: {body}"
        )));
    }

    let body = response.text().await.map_err(|e| {
        AppError::unexpected(format!("Failed to read OpenRouter response body: {e}"))
    })?;

    let chat_response: ChatResponse = serde_json::from_str(&body)
        .map_err(|e| AppError::unexpected(format!("Failed to parse OpenRouter response: {e}")))?;

    let content = chat_response
        .choices
        .into_iter()
        .next()
        .map(|c| c.message.content)
        .unwrap_or_default();

    if content.trim().is_empty() {
        return Err(AppError::unexpected(
            "OpenRouter returned an empty response".to_string(),
        ));
    }

    Ok(content)
}

/// Extract a JSON object from a model response that may contain markdown
/// fences (```json ... ```) or surrounding prose.
fn extract_json(raw: &str) -> &str {
    let trimmed = raw.trim();

    // Strip ```json ... ``` or ``` ... ``` fences
    if let Some(after) = trimmed.strip_prefix("```json")
        && let Some(inner) = after.strip_suffix("```")
    {
        return inner.trim();
    }
    if let Some(after) = trimmed.strip_prefix("```")
        && let Some(inner) = after.strip_suffix("```")
    {
        return inner.trim();
    }

    // Find the first '{' and last '}' to extract the JSON object
    if let (Some(start), Some(end)) = (trimmed.find('{'), trimmed.rfind('}'))
        && start < end
    {
        return &trimmed[start..=end];
    }

    trimmed
}

// --- OpenRouter API types ---

#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
}

#[derive(Debug, Serialize)]
struct Message {
    role: String,
    content: Vec<ContentPart>,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
enum ContentPart {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image_url")]
    ImageUrl { image_url: ImageUrlDetail },
}

#[derive(Debug, Serialize)]
struct ImageUrlDetail {
    url: String,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: ResponseMessage,
}

#[derive(Debug, Deserialize)]
struct ResponseMessage {
    content: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_chat_response() {
        let json = r#"{
            "id": "gen-abc123",
            "model": "openrouter/free",
            "choices": [
                {
                    "index": 0,
                    "message": {
                        "role": "assistant",
                        "content": "{\"name\": \"Square Mile\", \"country\": \"United Kingdom\", \"city\": \"London\"}"
                    },
                    "finish_reason": "stop"
                }
            ]
        }"#;

        let response: ChatResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.choices.len(), 1);

        let content = &response.choices[0].message.content;
        let roaster: ExtractedRoaster = serde_json::from_str(content).unwrap();
        assert_eq!(roaster.name.as_deref(), Some("Square Mile"));
        assert_eq!(roaster.country.as_deref(), Some("United Kingdom"));
        assert_eq!(roaster.city.as_deref(), Some("London"));
        assert!(roaster.homepage.is_none());
    }

    #[test]
    fn parse_roast_extraction() {
        let json = r#"{
            "roaster_name": "Square Mile",
            "name": "Red Brick",
            "origin": "Brazil",
            "region": "Cerrado Mineiro",
            "producer": "Fazenda Pinhal",
            "process": "Natural",
            "tasting_notes": ["Chocolate", "Hazelnut", "Caramel"]
        }"#;

        let roast: ExtractedRoast = serde_json::from_str(json).unwrap();
        assert_eq!(roast.roaster_name.as_deref(), Some("Square Mile"));
        assert_eq!(roast.name.as_deref(), Some("Red Brick"));
        assert_eq!(roast.origin.as_deref(), Some("Brazil"));
        assert_eq!(
            roast.tasting_notes.as_deref(),
            Some(&["Chocolate", "Hazelnut", "Caramel"].map(String::from)[..])
        );
    }

    #[test]
    fn parse_partial_roast_extraction() {
        let json = r#"{"name": "Ethiopia Yirgacheffe", "origin": "Ethiopia"}"#;

        let roast: ExtractedRoast = serde_json::from_str(json).unwrap();
        assert_eq!(roast.name.as_deref(), Some("Ethiopia Yirgacheffe"));
        assert_eq!(roast.origin.as_deref(), Some("Ethiopia"));
        assert!(roast.roaster_name.is_none());
        assert!(roast.region.is_none());
        assert!(roast.tasting_notes.is_none());
    }

    #[test]
    fn serialize_chat_request_with_image() {
        let request = ChatRequest {
            model: "test-model".to_string(),
            messages: vec![Message {
                role: "user".to_string(),
                content: vec![
                    ContentPart::Text {
                        text: "Extract info".to_string(),
                    },
                    ContentPart::ImageUrl {
                        image_url: ImageUrlDetail {
                            url: "data:image/jpeg;base64,/9j/4AAQ".to_string(),
                        },
                    },
                ],
            }],
        };

        let json = serde_json::to_value(&request).unwrap();
        assert_eq!(json["model"], "test-model");
        assert_eq!(json["messages"][0]["content"][0]["type"], "text");
        assert_eq!(json["messages"][0]["content"][1]["type"], "image_url");
    }

    #[test]
    fn extract_json_from_plain_json() {
        let raw = r#"{"name": "Square Mile"}"#;
        assert_eq!(extract_json(raw), raw);
    }

    #[test]
    fn extract_json_from_markdown_fence() {
        let raw = "```json\n{\"name\": \"Square Mile\"}\n```";
        assert_eq!(extract_json(raw), r#"{"name": "Square Mile"}"#);
    }

    #[test]
    fn extract_json_from_prose() {
        let raw = "Here is the data:\n{\"name\": \"Square Mile\"}\nHope that helps!";
        assert_eq!(extract_json(raw), r#"{"name": "Square Mile"}"#);
    }
}
