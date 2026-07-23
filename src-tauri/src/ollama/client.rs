use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::{AppHandle, Emitter};

use super::prompt::{build_chat_request, build_write_request, ChatContext, ChatMessage};
use super::websearch::{format_results_for_prompt, web_search};
use crate::settings::keychain::get_api_key;

const OLLAMA_BASE_URL: &str = "http://localhost:11434";
/// For quick status calls only (detect/list/capabilities) — these should fail fast if Ollama
/// isn't responsive. Streaming generation deliberately has no such ceiling: a thinking model can
/// legitimately take well over a minute, and a client-wide timeout would abort the connection
/// mid-stream (this happened — it surfaced as a confusing "error decoding response body").
const QUICK_REQUEST_TIMEOUT: Duration = Duration::from_secs(5);
/// Safety net for generation only, so a truly stalled connection doesn't hang forever.
const GENERATION_TIMEOUT: Duration = Duration::from_secs(600);
/// How often the streaming loop wakes up to recheck the cancel flag even when no new bytes have
/// arrived — without this, a stalled/idle connection would only ever notice Cancel between
/// chunks, i.e. never, until `GENERATION_TIMEOUT`.
const STREAM_POLL_INTERVAL: Duration = Duration::from_millis(500);
const GENERATION_CHUNK_EVENT: &str = "generation:chunk";
const GENERATION_THINKING_EVENT: &str = "generation:thinking";
const GENERATION_PROMPT_EVAL_EVENT: &str = "generation:promptEvalCount";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaModel {
    pub name: String,
}

#[derive(Debug, Deserialize)]
struct TagsResponse {
    models: Vec<TagsModel>,
}

#[derive(Debug, Deserialize)]
struct TagsModel {
    name: String,
}

#[derive(Debug, Default, Deserialize)]
struct ShowDetails {
    #[serde(default)]
    family: String,
    #[serde(default)]
    parameter_size: String,
    #[serde(default)]
    quantization_level: String,
}

#[derive(Debug, Deserialize)]
struct ShowResponse {
    #[serde(default)]
    capabilities: Vec<String>,
    #[serde(default)]
    details: ShowDetails,
    #[serde(default)]
    model_info: std::collections::HashMap<String, serde_json::Value>,
    #[serde(default)]
    parameters: String,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ModelParameter {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Serialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct ModelInfo {
    pub architecture: String,
    pub parameter_size: String,
    pub quantization: String,
    pub context_length: Option<u64>,
    pub capabilities: Vec<String>,
    pub parameters: Vec<ModelParameter>,
}

#[derive(Debug, Deserialize)]
struct ChatStreamLine {
    message: Option<ChatStreamMessage>,
    #[serde(default)]
    done: bool,
    #[serde(default)]
    error: Option<String>,
    #[serde(default)]
    prompt_eval_count: Option<u32>,
}

/// Emitted payloads for `generation:chunk`/`generation:thinking`. Tagged with the generation's id
/// (the frontend's chat-turn id, echoed back unchanged) so a listener that outlives its own
/// generation — e.g. because the user switched files before the old request's HTTP stream fully
/// closed — can tell the difference and ignore events that belong to an abandoned generation
/// instead of letting them bleed into whatever turn is now active.
#[derive(Serialize, Clone)]
struct GenerationEvent<'a> {
    id: &'a str,
    chunk: &'a str,
}

/// Emitted once, when Ollama reports how many prompt tokens the just-completed request actually
/// used — a real number to replace the frontend's pre-send character-based estimate once a turn
/// finishes.
#[derive(Serialize, Clone)]
struct PromptEvalEvent<'a> {
    id: &'a str,
    count: u32,
}

#[derive(Debug, Deserialize)]
struct ChatStreamMessage {
    content: String,
    #[serde(default)]
    thinking: Option<String>,
}

/// One message in `GenerateChatTurnRequest.history` — the frontend has already compacted this
/// (e.g. a past "write" turn becomes a short marker, not its full document text) before sending
/// it; this module doesn't interpret history, only relays it into the request unchanged.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatMessageInput {
    pub role: String,
    pub content: String,
    #[serde(default)]
    pub images: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachedFileInput {
    pub path: String,
    pub content: String,
}

/// A single struct rather than a long positional parameter list for the same reason
/// `ollama/prompt.rs`'s `ChatContext` is a struct — this command has grown once already
/// (`generate_edit` → this) and a struct keeps the next addition from being another silent
/// positional insertion.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateChatTurnRequest {
    pub generation_id: String,
    pub model: String,
    /// `"chat"` (plain conversational reply) or `"write"` (diffed against `target_document`).
    pub mode: String,
    #[serde(default)]
    pub history: Vec<ChatMessageInput>,
    pub target_document: Option<String>,
    #[serde(default)]
    pub attached_files: Vec<AttachedFileInput>,
    pub instruction: String,
    #[serde(default)]
    pub images: Vec<String>,
    pub num_ctx: Option<u32>,
    pub thinking: bool,
    pub web_search: bool,
}

/// Drains complete NDJSON lines out of `buffer`, leaving any trailing partial line (a chunk can
/// split a JSON object across two network reads) in place for the next call.
fn drain_complete_lines(buffer: &mut String) -> Result<Vec<ChatStreamLine>, String> {
    let mut lines = Vec::new();

    while let Some(newline_pos) = buffer.find('\n') {
        let line = buffer[..newline_pos].to_string();
        buffer.drain(..=newline_pos);

        if line.trim().is_empty() {
            continue;
        }

        let parsed: ChatStreamLine = serde_json::from_str(&line)
            .map_err(|err| format!("Unexpected response from Ollama: {err}"))?;
        lines.push(parsed);
    }

    Ok(lines)
}

/// `model_info` keys are architecture-prefixed (e.g. `"qwen3.context_length"`) — search by
/// suffix rather than constructing the exact key, so this works regardless of model family.
fn extract_context_length(model_info: &std::collections::HashMap<String, serde_json::Value>) -> Option<u64> {
    model_info
        .iter()
        .find(|(key, _)| key.ends_with(".context_length"))
        .and_then(|(_, value)| value.as_u64())
}

/// Ollama's `/api/show` returns default sampling parameters as a raw multi-line string (one
/// `key value` pair per line, e.g. `"temperature 1\ntop_p 0.95"`) rather than structured JSON.
fn parse_parameters(raw: &str) -> Vec<ModelParameter> {
    raw.lines()
        .filter_map(|line| {
            let mut parts = line.split_whitespace();
            let key = parts.next()?.to_string();
            let value: Vec<&str> = parts.collect();
            if value.is_empty() {
                return None;
            }
            Some(ModelParameter {
                key,
                value: value.join(" "),
            })
        })
        .collect()
}

pub struct OllamaClient {
    http: reqwest::Client,
    /// Flag for the currently in-flight generation, if any. Only one generation runs at a time
    /// (the UI disables Send while busy), so a single slot is enough — a fresh flag replaces the
    /// old one at the start of each `generate_chat_turn` call.
    cancel_flag: Mutex<Option<Arc<AtomicBool>>>,
}

impl OllamaClient {
    pub fn new() -> Self {
        let http = reqwest::Client::builder()
            .build()
            .expect("failed to build reqwest client");
        Self {
            http,
            cancel_flag: Mutex::new(None),
        }
    }

    /// Signals the in-flight generation (if any) to stop. The streaming loop checks this between
    /// chunks and bails out, which drops the response body and closes the connection — Ollama
    /// notices the closed connection and stops generating on its end too, not just on ours.
    pub fn cancel_generation(&self) {
        if let Some(flag) = self.cancel_flag.lock().unwrap().as_ref() {
            flag.store(true, Ordering::Relaxed);
        }
    }

    pub async fn detect(&self) -> bool {
        self.http
            .get(format!("{OLLAMA_BASE_URL}/api/tags"))
            .timeout(QUICK_REQUEST_TIMEOUT)
            .send()
            .await
            .map(|response| response.status().is_success())
            .unwrap_or(false)
    }

    pub async fn list_models(&self) -> Result<Vec<OllamaModel>, String> {
        let response = self
            .http
            .get(format!("{OLLAMA_BASE_URL}/api/tags"))
            .timeout(QUICK_REQUEST_TIMEOUT)
            .send()
            .await
            .map_err(|err| format!("Could not reach Ollama: {err}"))?;

        let tags: TagsResponse = response
            .json()
            .await
            .map_err(|err| format!("Unexpected response from Ollama: {err}"))?;

        Ok(tags
            .models
            .into_iter()
            .map(|model| OllamaModel { name: model.name })
            .collect())
    }

    /// Shared by `supports_thinking`/`supports_vision`/`get_model_info` — they all just need a
    /// different slice of the same `/api/show` response, so there's no reason for each to fire
    /// its own independent request against Ollama.
    async fn fetch_show(&self, model: &str) -> Result<ShowResponse, String> {
        let response = self
            .http
            .post(format!("{OLLAMA_BASE_URL}/api/show"))
            .json(&serde_json::json!({ "model": model }))
            .timeout(QUICK_REQUEST_TIMEOUT)
            .send()
            .await
            .map_err(|err| format!("Could not reach Ollama: {err}"))?;

        response
            .json()
            .await
            .map_err(|err| format!("Unexpected response from Ollama: {err}"))
    }

    /// Whether the model advertises the `thinking` capability via `/api/show`. Models that
    /// don't support it should have the thinking toggle disabled rather than silently ignoring
    /// `think: true`.
    pub async fn supports_thinking(&self, model: &str) -> Result<bool, String> {
        let show = self.fetch_show(model).await?;
        Ok(show.capabilities.iter().any(|cap| cap == "thinking"))
    }

    /// Whether the model advertises the `vision` capability via `/api/show` — gates whether the
    /// UI lets the user attach images to a turn at all.
    pub async fn supports_vision(&self, model: &str) -> Result<bool, String> {
        let show = self.fetch_show(model).await?;
        Ok(show.capabilities.iter().any(|cap| cap == "vision"))
    }

    /// Fetches the richer model detail shown in the info dialog — architecture, parameter size,
    /// quantization, context length, capabilities, and default sampling parameters. Separate from
    /// `supports_thinking`/`supports_vision` (which only need `capabilities`) since this is only
    /// fetched lazily when the user opens the info dialog, not on every model switch.
    pub async fn get_model_info(&self, model: &str) -> Result<ModelInfo, String> {
        let show = self.fetch_show(model).await?;
        Ok(ModelInfo {
            architecture: show.details.family,
            parameter_size: show.details.parameter_size,
            quantization: show.details.quantization_level,
            context_length: extract_context_length(&show.model_info),
            capabilities: show.capabilities,
            parameters: parse_parameters(&show.parameters),
        })
    }

    /// Sends a chat-turn request (either a plain conversational "chat" turn, or a "write" turn
    /// diffed against a target document — see `ollama/prompt.rs`) and streams the response as
    /// `generation:chunk`/`generation:thinking` events, plus a one-shot `generation:promptEvalCount`
    /// once Ollama reports how many prompt tokens it actually used. Returns the full assembled
    /// answer text once Ollama reports `done` — the thinking trace is never included in it.
    pub async fn generate_chat_turn(
        &self,
        app: &AppHandle,
        request: GenerateChatTurnRequest,
    ) -> Result<String, String> {
        let cancel = Arc::new(AtomicBool::new(false));
        *self.cancel_flag.lock().unwrap() = Some(cancel.clone());

        let result = self.generate_chat_turn_inner(app, &request, &cancel).await;

        // Only clear the slot if it still points to *this* generation's flag — a newer
        // generation may already have replaced it (e.g. this one was cancelled in favor of a
        // fresh one started before this call had a chance to unwind). Blindly nulling the slot
        // here would silently strand the newer generation's Cancel button.
        let mut guard = self.cancel_flag.lock().unwrap();
        if guard.as_ref().is_some_and(|current| Arc::ptr_eq(current, &cancel)) {
            *guard = None;
        }
        drop(guard);
        result
    }

    async fn generate_chat_turn_inner(
        &self,
        app: &AppHandle,
        request: &GenerateChatTurnRequest,
        cancel: &Arc<AtomicBool>,
    ) -> Result<String, String> {
        let web_context = if request.web_search {
            // Keychain access can block on a macOS permission prompt (see settings/keychain.rs) —
            // must not tie up this async worker thread while waiting on it.
            let api_key = tauri::async_runtime::spawn_blocking(get_api_key)
                .await
                .map_err(|err| format!("Keychain task failed: {err}"))??
                .ok_or_else(|| {
                    "Web Search is on but no Ollama API key is configured. Add one in Settings."
                        .to_string()
                })?;
            let results = web_search(&self.http, &api_key, &request.instruction).await?;
            Some(format_results_for_prompt(&results))
        } else {
            None
        };

        let history: Vec<ChatMessage> = request
            .history
            .iter()
            .map(|m| ChatMessage {
                role: m.role.clone(),
                content: m.content.clone(),
                images: m.images.clone(),
            })
            .collect();
        let attached_files: Vec<(String, String)> = request
            .attached_files
            .iter()
            .map(|f| (f.path.clone(), f.content.clone()))
            .collect();

        let ctx = ChatContext {
            history,
            attached_files,
            images: request.images.clone(),
            instruction: request.instruction.clone(),
            num_ctx: request.num_ctx,
            thinking: request.thinking,
            web_context,
        };

        let chat_request = if request.mode == "write" {
            build_write_request(
                request.model.clone(),
                request.target_document.as_deref().unwrap_or(""),
                ctx,
            )
        } else {
            build_chat_request(request.model.clone(), ctx)
        };

        let response = self
            .http
            .post(format!("{OLLAMA_BASE_URL}/api/chat"))
            .json(&chat_request)
            .timeout(GENERATION_TIMEOUT)
            .send()
            .await
            .map_err(|err| format!("Could not reach Ollama: {err}"))?;

        if !response.status().is_success() {
            return Err(format!("Ollama returned {}", response.status()));
        }

        let mut byte_stream = response.bytes_stream();
        let mut buffer = String::new();
        let mut full_text = String::new();

        loop {
            // Checked every poll tick, not just when a chunk actually arrives — a stalled
            // connection (model hung, Ollama process wedged) would otherwise block on
            // `byte_stream.next()` forever and never notice a Cancel click until
            // `GENERATION_TIMEOUT` eventually fires.
            if cancel.load(Ordering::Relaxed) {
                // Dropping `byte_stream`/`response` here (function return) closes the
                // connection; Ollama notices and stops generating on its end too.
                return Err("Generation cancelled".to_string());
            }

            let next = match tokio::time::timeout(STREAM_POLL_INTERVAL, byte_stream.next()).await
            {
                Ok(next) => next,
                Err(_) => continue, // no data within this tick — loop back and recheck cancel
            };

            let Some(chunk) = next else {
                break; // stream ended
            };

            let chunk = chunk.map_err(|err| format!("Stream error: {err}"))?;
            buffer.push_str(&String::from_utf8_lossy(&chunk));

            for parsed in drain_complete_lines(&mut buffer)? {
                if let Some(error) = parsed.error {
                    return Err(format!("Ollama reported an error: {error}"));
                }

                if let Some(message) = parsed.message {
                    if let Some(thinking) = message.thinking.filter(|text| !text.is_empty()) {
                        let _ = app.emit(
                            GENERATION_THINKING_EVENT,
                            GenerationEvent {
                                id: &request.generation_id,
                                chunk: &thinking,
                            },
                        );
                    }
                    if !message.content.is_empty() {
                        full_text.push_str(&message.content);
                        let _ = app.emit(
                            GENERATION_CHUNK_EVENT,
                            GenerationEvent {
                                id: &request.generation_id,
                                chunk: &message.content,
                            },
                        );
                    }
                }

                if parsed.done {
                    if let Some(count) = parsed.prompt_eval_count {
                        let _ = app.emit(
                            GENERATION_PROMPT_EVAL_EVENT,
                            PromptEvalEvent {
                                id: &request.generation_id,
                                count,
                            },
                        );
                    }
                    return Ok(full_text);
                }
            }
        }

        Ok(full_text)
    }
}

impl Default for OllamaClient {
    fn default() -> Self {
        Self::new()
    }
}

#[tauri::command]
pub async fn ollama_detect(client: tauri::State<'_, OllamaClient>) -> Result<bool, ()> {
    Ok(client.detect().await)
}

#[tauri::command]
pub async fn ollama_list_models(
    client: tauri::State<'_, OllamaClient>,
) -> Result<Vec<OllamaModel>, String> {
    client.list_models().await
}

#[tauri::command]
pub async fn ollama_supports_thinking(
    client: tauri::State<'_, OllamaClient>,
    model: String,
) -> Result<bool, String> {
    client.supports_thinking(&model).await
}

#[tauri::command]
pub async fn ollama_supports_vision(
    client: tauri::State<'_, OllamaClient>,
    model: String,
) -> Result<bool, String> {
    client.supports_vision(&model).await
}

#[tauri::command]
pub async fn ollama_get_model_info(
    client: tauri::State<'_, OllamaClient>,
    model: String,
) -> Result<ModelInfo, String> {
    client.get_model_info(&model).await
}

#[tauri::command]
pub fn cancel_generation(client: tauri::State<'_, OllamaClient>) {
    client.cancel_generation();
}

#[tauri::command]
pub async fn generate_chat_turn(
    app: AppHandle,
    client: tauri::State<'_, OllamaClient>,
    request: GenerateChatTurnRequest,
) -> Result<String, String> {
    client.generate_chat_turn(&app, request).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_real_ollama_tags_response_shape() {
        // Trimmed sample of what a real `/api/tags` response looks like.
        let body = r#"{
            "models": [
                {
                    "name": "qwen3.5:9b",
                    "model": "qwen3.5:9b",
                    "modified_at": "2026-07-22T20:00:00Z",
                    "size": 6600000000,
                    "digest": "6488c96fa5fa"
                }
            ]
        }"#;

        let parsed: TagsResponse = serde_json::from_str(body).expect("should parse");
        assert_eq!(parsed.models.len(), 1);
        assert_eq!(parsed.models[0].name, "qwen3.5:9b");
    }

    #[test]
    fn drains_only_complete_lines_and_keeps_a_split_line_buffered() {
        let mut buffer = String::from(
            "{\"message\":{\"content\":\"# Good\"},\"done\":false}\n\
             {\"message\":{\"content\":\"bye\"},\"do",
        );

        let lines = drain_complete_lines(&mut buffer).expect("should parse complete lines");

        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].message.as_ref().unwrap().content, "# Good");
        // The second, still-incomplete line stays in the buffer rather than being dropped.
        assert_eq!(buffer, "{\"message\":{\"content\":\"bye\"},\"do");

        buffer.push_str("ne\":true}\n");
        let lines = drain_complete_lines(&mut buffer).expect("should parse the completed line");
        assert_eq!(lines.len(), 1);
        assert!(lines[0].done);
        assert_eq!(buffer, "");
    }

    /// A normal NDJSON line has no `error` field, so it must default to `None` rather than
    /// failing to parse — this is what lets `error: Option<String>` be added without breaking
    /// every existing stream sample.
    #[test]
    fn lines_without_an_error_field_default_to_none() {
        let mut buffer = String::from("{\"message\":{\"content\":\"hi\"},\"done\":false}\n");
        let lines = drain_complete_lines(&mut buffer).expect("should parse");
        assert_eq!(lines[0].error, None);
    }

    /// Ollama can emit a mid-stream error line after the HTTP status was already a successful
    /// 200 (e.g. the model crashed partway through). It must be recognized as an error rather
    /// than silently ignored as a contentless line — a caller checks `parsed.error` per drained
    /// line and turns it into an `Err` instead of treating the generation as having succeeded.
    #[test]
    fn a_mid_stream_error_line_parses_with_its_message() {
        let mut buffer = String::from("{\"error\":\"model runner crashed\",\"done\":true}\n");
        let lines = drain_complete_lines(&mut buffer).expect("should parse");
        assert_eq!(lines[0].error.as_deref(), Some("model runner crashed"));
    }

    #[test]
    fn assembles_full_text_from_real_captured_stream_sample() {
        // Reduced sample matching a real `/api/chat` stream: a thinking-only chunk (empty
        // content, ignored for the assembled text) followed by real content chunks and a
        // trailing `done` line with empty content and extra stats fields.
        let raw = "{\"message\":{\"content\":\"\",\"thinking\":\"ok\"},\"done\":false}\n\
                   {\"message\":{\"content\":\"# Goodbye\"},\"done\":false}\n\
                   {\"message\":{\"content\":\"\\n\\nThis is a test doc.\"},\"done\":false}\n\
                   {\"message\":{\"content\":\"\"},\"done\":true,\"done_reason\":\"stop\",\"eval_count\":583}\n";

        let mut buffer = String::from(raw);
        let mut full_text = String::new();
        let mut saw_done = false;

        for parsed in drain_complete_lines(&mut buffer).expect("should parse") {
            if let Some(message) = parsed.message {
                full_text.push_str(&message.content);
            }
            saw_done |= parsed.done;
        }

        assert!(saw_done);
        assert_eq!(full_text, "# Goodbye\n\nThis is a test doc.");
    }

    #[test]
    fn thinking_text_is_separate_from_content_and_never_joins_the_answer() {
        let raw = "{\"message\":{\"content\":\"\",\"thinking\":\"Let me think\"},\"done\":false}\n\
                   {\"message\":{\"content\":\"\",\"thinking\":\" about it.\"},\"done\":false}\n\
                   {\"message\":{\"content\":\"Done.\"},\"done\":false}\n\
                   {\"message\":{\"content\":\"\"},\"done\":true}\n";

        let mut buffer = String::from(raw);
        let mut full_text = String::new();
        let mut thinking_text = String::new();

        for parsed in drain_complete_lines(&mut buffer).expect("should parse") {
            if let Some(message) = parsed.message {
                if let Some(thinking) = message.thinking {
                    thinking_text.push_str(&thinking);
                }
                full_text.push_str(&message.content);
            }
        }

        assert_eq!(thinking_text, "Let me think about it.");
        assert_eq!(full_text, "Done.");
    }

    /// The final `done` line can carry a real `prompt_eval_count` — must parse and be available
    /// for the caller to turn into the one-shot `generation:promptEvalCount` event.
    #[test]
    fn done_line_with_prompt_eval_count_parses_it() {
        let mut buffer = String::from(
            "{\"message\":{\"content\":\"\"},\"done\":true,\"prompt_eval_count\":1234}\n",
        );
        let lines = drain_complete_lines(&mut buffer).expect("should parse");
        assert_eq!(lines[0].prompt_eval_count, Some(1234));
    }

    #[test]
    fn done_line_without_prompt_eval_count_defaults_to_none() {
        let mut buffer = String::from("{\"message\":{\"content\":\"\"},\"done\":true}\n");
        let lines = drain_complete_lines(&mut buffer).expect("should parse");
        assert_eq!(lines[0].prompt_eval_count, None);
    }

    #[test]
    fn parses_show_response_capabilities() {
        let body = r#"{"capabilities": ["completion", "vision", "tools", "thinking"]}"#;
        let parsed: ShowResponse = serde_json::from_str(body).expect("should parse");
        assert!(parsed.capabilities.iter().any(|cap| cap == "thinking"));

        let body_without_thinking = r#"{"capabilities": ["completion"]}"#;
        let parsed: ShowResponse =
            serde_json::from_str(body_without_thinking).expect("should parse");
        assert!(!parsed.capabilities.iter().any(|cap| cap == "thinking"));
    }

    #[test]
    fn parses_full_show_response_into_model_info_fields() {
        let body = r#"{
            "capabilities": ["completion", "vision", "thinking"],
            "details": {
                "family": "qwen3",
                "parameter_size": "9.7B",
                "quantization_level": "Q4_K_M"
            },
            "model_info": {
                "general.architecture": "qwen3",
                "qwen3.context_length": 262144,
                "qwen3.embedding_length": 4096
            },
            "parameters": "temperature 1\ntop_p 0.95\ntop_k 20"
        }"#;

        let show: ShowResponse = serde_json::from_str(body).expect("should parse");
        assert_eq!(show.details.family, "qwen3");
        assert_eq!(show.details.parameter_size, "9.7B");
        assert_eq!(show.details.quantization_level, "Q4_K_M");
        assert_eq!(extract_context_length(&show.model_info), Some(262144));

        let params = parse_parameters(&show.parameters);
        assert_eq!(params.len(), 3);
        assert_eq!(params[0].key, "temperature");
        assert_eq!(params[0].value, "1");
        assert_eq!(params[1].key, "top_p");
        assert_eq!(params[1].value, "0.95");
    }

    #[test]
    fn extract_context_length_returns_none_when_absent() {
        let model_info = std::collections::HashMap::new();
        assert_eq!(extract_context_length(&model_info), None);
    }

    #[test]
    fn parse_parameters_ignores_blank_lines_and_joins_multi_word_values() {
        let raw = "temperature 1\n\nstop \"<|eot_id|>\" extra\n";
        let params = parse_parameters(raw);
        assert_eq!(params.len(), 2);
        assert_eq!(params[0].key, "temperature");
        assert_eq!(params[1].key, "stop");
        assert_eq!(params[1].value, "\"<|eot_id|>\" extra");
    }
}
