use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::application::errors::AppError;

pub const OPENROUTER_URL: &str = "https://openrouter.ai/api/v1/chat/completions";
const USER_AGENT: &str = "Brewlog/1.0";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(90);

const ROASTER_PROMPT: &str = r#"Resolve the input as a coffee roaster lookup, then extract information about the roaster. The input may be only a short or ambiguous brand name (for example, "Prolog"). Always use web search to identify the coffee-roasting business that best matches the input and verify its details, preferring the roaster's official website. Use coffee-specific context to disambiguate it from unrelated businesses or meanings. If no coffee roaster can be identified with confidence, return an empty JSON object.

Return a JSON object with these fields (only include fields you can identify with confidence):
- "name": the roaster's name
- "country": the country the roaster is based in
- "city": the city the roaster is based in
- "homepage": the roaster's website URL

Use UK/English names for countries and cities (e.g. United Kingdom, not UK; Montreal, Gothenburg, Copenhagen, Torrevieja, Vienna). Preserve a district qualifier if it is part of the verified location.

Return ONLY the JSON object, no other text."#;

const ROAST_PROMPT: &str = r#"Resolve the input as a coffee roaster or specific coffee lookup, then extract information about the coffee. The input may contain only a short or ambiguous product or roaster name. Always use web search to identify the coffee product that best matches the input and verify its details, preferring the roaster's official product page. Use coffee-specific context to disambiguate it from unrelated products or meanings. If no specific coffee can be identified with confidence, return an empty JSON object.

Return a JSON object with these fields (only include fields you can identify with confidence):
- "roaster_name": the name of the roaster
- "name": the name of this specific coffee/roast
- "origin": the country (or countries, comma-separated) of origin of the coffee beans (e.g. "Ethiopia" or "Ethiopia, Colombia")
- "region": the region within the origin country
- "producer": the farm, estate, or cooperative that produced the beans
- "process": the processing method (e.g. Washed, Natural, Honey, Anaerobic)
- "tasting_notes": an array of flavour/tasting notes in Title Case (e.g. ["Blueberry", "Jasmine", "Dark Chocolate"])

Use UK/English country names for origin. The name identifies the coffee: prefer its distinctive product, farm, lot, or producer identity. Do not repeat origin, producer, or process in the name when another distinctive identifier remains; retain the producer when removing it would leave only a generic variety or descriptor. Preserve established product names, and keep decaf and flavour qualifiers when they distinguish the coffee. Use a spaced hyphen ( - ), not an em dash, if two identifying parts are needed.
For process, use Anaerobic Natural and Anaerobic Washed (not reversed word order), and spell flavoured co-ferments as Co-ferment (e.g. Peach Co-ferment). Keep the underlying method and flavour where verified. Do not conflate distinct processing techniques just to standardise wording. Do not infer a process from the name alone; omit fields you cannot verify rather than copying or guessing missing details.

Return ONLY the JSON object, no other text."#;

const SCAN_PROMPT: &str = r#"Resolve the input as a coffee bag lookup, then extract both the roaster and coffee information. The input may be an image, a short name, or incomplete bag text. Always use web search to identify and verify the best coffee-specific match, preferring the roaster's official website or product page. Use visible bag details and coffee-specific context to disambiguate it from unrelated products or meanings. If no coffee product can be identified with confidence, return empty "roaster" and "roast" objects.

Return a JSON object with two top-level keys:

{
  "roaster": {
    "name": "the roaster's name",
    "country": "country the roaster is based in",
    "city": "city the roaster is based in",
    "homepage": "the roaster's website URL"
  },
  "roast": {
    "name": "the name of this specific coffee/roast",
    "origin": "the country (or countries, comma-separated) of origin of the beans (e.g. 'Ethiopia' or 'Ethiopia, Colombia')",
    "region": "the region within the origin country",
    "producer": "the farm, estate, or cooperative",
    "process": "processing method (e.g. Washed, Natural, Honey, Anaerobic)",
    "tasting_notes": ["Array", "Of", "Flavour Notes In Title Case"]
  }
}

Use UK/English names for countries and cities (e.g. United Kingdom, not UK; Montreal, Gothenburg, Copenhagen, Torrevieja, Vienna). Preserve a district qualifier if it is part of the verified location. Use UK/English country names for roast origin.
The roast name identifies the coffee: prefer its distinctive product, farm, lot, or producer identity. Do not repeat origin, producer, or process in the name when another distinctive identifier remains; retain the producer when removing it would leave only a generic variety or descriptor. Preserve established product names, and keep decaf and flavour qualifiers when they distinguish the coffee. Use a spaced hyphen ( - ), not an em dash, if two identifying parts are needed.
For process, use Anaerobic Natural and Anaerobic Washed (not reversed word order), and spell flavoured co-ferments as Co-ferment (e.g. Peach Co-ferment). Keep the underlying method and flavour where verified. Do not conflate distinct processing techniques just to standardise wording. Do not infer a process from the name alone; omit fields you cannot verify rather than copying or guessing missing details.
Only include fields you can identify with confidence. Each tasting note must be in Title Case. Return ONLY the JSON object, no other text."#;

// --- Public types ---

#[derive(Debug, Clone, Deserialize)]
pub struct Usage {
    pub prompt_tokens: i64,
    pub completion_tokens: i64,
    pub total_tokens: i64,
    pub cost: f64,
}

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
    url: &str,
    api_key: &str,
    model: &str,
    input: &ExtractionInput,
) -> Result<(ExtractedRoaster, Option<Usage>), AppError> {
    let (content, usage) =
        call_openrouter(client, url, api_key, model, ROASTER_PROMPT, input).await?;
    let json = extract_json(&content);

    let extracted = serde_json::from_str(json).map_err(|e| {
        AppError::unexpected(format!("Failed to parse AI response as roaster data: {e}"))
    })?;
    Ok((extracted, usage))
}

pub async fn extract_roast(
    client: &reqwest::Client,
    url: &str,
    api_key: &str,
    model: &str,
    input: &ExtractionInput,
) -> Result<(ExtractedRoast, Option<Usage>), AppError> {
    let (content, usage) =
        call_openrouter(client, url, api_key, model, ROAST_PROMPT, input).await?;
    let json = extract_json(&content);

    let extracted = serde_json::from_str(json).map_err(|e| {
        AppError::unexpected(format!("Failed to parse AI response as roast data: {e}"))
    })?;
    Ok((extracted, usage))
}

pub async fn extract_bag_scan(
    client: &reqwest::Client,
    url: &str,
    api_key: &str,
    model: &str,
    input: &ExtractionInput,
) -> Result<(ExtractedBagScan, Option<Usage>), AppError> {
    let (content, usage) = call_openrouter(client, url, api_key, model, SCAN_PROMPT, input).await?;
    let json = extract_json(&content);

    let extracted = serde_json::from_str(json).map_err(|e| {
        AppError::unexpected(format!("Failed to parse AI response as bag scan data: {e}"))
    })?;
    Ok((extracted, usage))
}

// --- Internal helpers ---

async fn call_openrouter(
    client: &reqwest::Client,
    url: &str,
    api_key: &str,
    model: &str,
    system_prompt: &str,
    input: &ExtractionInput,
) -> Result<(String, Option<Usage>), AppError> {
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
        tools: vec![ServerTool {
            tool_type: "openrouter:web_search",
        }],
    };

    let response = client
        .post(url)
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

    response_content(chat_response)
}

fn response_content(response: ChatResponse) -> Result<(String, Option<Usage>), AppError> {
    let ChatResponse { choices, usage } = response;
    let choice = choices
        .into_iter()
        .next()
        .ok_or_else(|| AppError::unexpected("OpenRouter returned no choices"))?;

    let content = choice
        .message
        .content
        .filter(|content| !content.trim().is_empty())
        .ok_or_else(|| {
            AppError::unexpected(format!(
                "OpenRouter returned no content (finish_reason={}, native_finish_reason={})",
                choice.finish_reason.as_deref().unwrap_or("unknown"),
                choice.native_finish_reason.as_deref().unwrap_or("unknown")
            ))
        })?;

    Ok((content, usage))
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
    tools: Vec<ServerTool>,
}

#[derive(Debug, Serialize)]
struct ServerTool {
    #[serde(rename = "type")]
    tool_type: &'static str,
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
    usage: Option<Usage>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: ResponseMessage,
    finish_reason: Option<String>,
    native_finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ResponseMessage {
    content: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roaster_extraction_uses_english_place_names() {
        for prompt in [ROASTER_PROMPT, SCAN_PROMPT] {
            assert!(prompt.contains("UK/English names for countries and cities"));
            assert!(prompt.contains("United Kingdom, not UK"));
            assert!(prompt.contains("Montreal, Gothenburg, Copenhagen"));
        }
    }

    #[test]
    fn both_roast_extractions_share_name_and_process_conventions() {
        for prompt in [ROAST_PROMPT, SCAN_PROMPT] {
            assert!(prompt.contains("name identifies the coffee"));
            assert!(prompt.contains("Do not repeat origin, producer, or process"));
            assert!(prompt.contains("keep decaf and flavour qualifiers"));
            assert!(prompt.contains("spaced hyphen ( - ), not an em dash"));
            assert!(prompt.contains("Anaerobic Natural and Anaerobic Washed"));
            assert!(prompt.contains("Co-ferment"));
            assert!(prompt.contains("Do not conflate distinct processing techniques"));
        }
    }

    #[test]
    fn roast_extraction_keeps_unknown_details_unknown() {
        for prompt in [ROAST_PROMPT, SCAN_PROMPT] {
            assert!(prompt.contains("Do not infer a process from the name alone"));
            assert!(prompt.contains("omit fields you cannot verify"));
        }
    }

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
            ],
            "usage": {
                "prompt_tokens": 194,
                "completion_tokens": 42,
                "total_tokens": 236,
                "cost": 0.0012
            }
        }"#;

        let response: ChatResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.choices.len(), 1);

        let content = response.choices[0].message.content.as_deref().unwrap();
        let roaster: ExtractedRoaster = serde_json::from_str(content).unwrap();
        assert_eq!(roaster.name.as_deref(), Some("Square Mile"));
        assert_eq!(roaster.country.as_deref(), Some("United Kingdom"));
        assert_eq!(roaster.city.as_deref(), Some("London"));
        assert!(roaster.homepage.is_none());

        let usage = response.usage.unwrap();
        assert_eq!(usage.prompt_tokens, 194);
        assert_eq!(usage.completion_tokens, 42);
        assert_eq!(usage.total_tokens, 236);
        assert!((usage.cost - 0.0012).abs() < f64::EPSILON);
    }

    #[test]
    fn parse_chat_response_without_usage() {
        let json = r#"{
            "id": "gen-abc123",
            "model": "openrouter/free",
            "choices": [
                {
                    "index": 0,
                    "message": {
                        "role": "assistant",
                        "content": "{\"name\": \"Square Mile\"}"
                    },
                    "finish_reason": "stop"
                }
            ]
        }"#;

        let response: ChatResponse = serde_json::from_str(json).unwrap();
        assert!(response.usage.is_none());
    }

    #[test]
    fn report_finish_reasons_when_response_content_is_null() {
        let json = r#"{
            "choices": [{
                "message": { "role": "assistant", "content": null },
                "finish_reason": "error",
                "native_finish_reason": "MALFORMED_FUNCTION_CALL"
            }]
        }"#;

        let response: ChatResponse = serde_json::from_str(json).unwrap();
        let error = response_content(response).unwrap_err();

        assert!(error.to_string().contains("finish_reason=error"));
        assert!(
            error
                .to_string()
                .contains("native_finish_reason=MALFORMED_FUNCTION_CALL")
        );
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
            tools: vec![ServerTool {
                tool_type: "openrouter:web_search",
            }],
        };

        let json = serde_json::to_value(&request).unwrap();
        assert_eq!(json["model"], "test-model");
        assert_eq!(json["messages"][0]["content"][0]["type"], "text");
        assert_eq!(json["messages"][0]["content"][1]["type"], "image_url");
        assert_eq!(json["tools"][0]["type"], "openrouter:web_search");
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
