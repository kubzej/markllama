use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::{AppHandle, Emitter};

use super::prompt::build_edit_request;
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
const GENERATION_CHUNK_EVENT: &str = "generation:chunk";
const GENERATION_THINKING_EVENT: &str = "generation:thinking";

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

#[derive(Debug, Deserialize)]
struct ShowResponse {
    #[serde(default)]
    capabilities: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct ChatStreamLine {
    message: Option<ChatStreamMessage>,
    #[serde(default)]
    done: bool,
}

#[derive(Debug, Deserialize)]
struct ChatStreamMessage {
    content: String,
    #[serde(default)]
    thinking: Option<String>,
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

pub struct OllamaClient {
    http: reqwest::Client,
    /// Flag for the currently in-flight generation, if any. Only one generation runs at a time
    /// (the UI disables Send while busy), so a single slot is enough — a fresh flag replaces the
    /// old one at the start of each `generate_edit` call.
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

    /// Whether the model advertises the `thinking` capability via `/api/show`. Models that
    /// don't support it should have the thinking toggle disabled rather than silently ignoring
    /// `think: true`.
    pub async fn supports_thinking(&self, model: &str) -> Result<bool, String> {
        let response = self
            .http
            .post(format!("{OLLAMA_BASE_URL}/api/show"))
            .json(&serde_json::json!({ "model": model }))
            .timeout(QUICK_REQUEST_TIMEOUT)
            .send()
            .await
            .map_err(|err| format!("Could not reach Ollama: {err}"))?;

        let show: ShowResponse = response
            .json()
            .await
            .map_err(|err| format!("Unexpected response from Ollama: {err}"))?;

        Ok(show.capabilities.iter().any(|cap| cap == "thinking"))
    }

    /// Sends the single-turn `{ system, markdown, instruction }` request and streams the
    /// response as `generation:chunk` (and, when `thinking` is enabled, `generation:thinking`)
    /// events. Returns the full assembled answer text once Ollama reports `done` — the thinking
    /// trace is never included in the returned text.
    pub async fn generate_edit(
        &self,
        app: &AppHandle,
        model: String,
        markdown: &str,
        instruction: &str,
        thinking: bool,
        web_search_enabled: bool,
    ) -> Result<String, String> {
        let cancel = Arc::new(AtomicBool::new(false));
        *self.cancel_flag.lock().unwrap() = Some(cancel.clone());

        let result = self
            .generate_edit_inner(
                app,
                model,
                markdown,
                instruction,
                thinking,
                web_search_enabled,
                &cancel,
            )
            .await;

        // Clear the slot so a cancel() call after this generation has already finished doesn't
        // reach into the *next* one.
        *self.cancel_flag.lock().unwrap() = None;
        result
    }

    async fn generate_edit_inner(
        &self,
        app: &AppHandle,
        model: String,
        markdown: &str,
        instruction: &str,
        thinking: bool,
        web_search_enabled: bool,
        cancel: &Arc<AtomicBool>,
    ) -> Result<String, String> {
        let web_context = if web_search_enabled {
            let api_key = get_api_key()?.ok_or_else(|| {
                "Web Search is on but no Ollama API key is configured. Add one in Settings."
                    .to_string()
            })?;
            let results = web_search(&self.http, &api_key, instruction).await?;
            Some(format_results_for_prompt(&results))
        } else {
            None
        };

        let request =
            build_edit_request(model, markdown, instruction, thinking, web_context.as_deref());

        let response = self
            .http
            .post(format!("{OLLAMA_BASE_URL}/api/chat"))
            .json(&request)
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

        while let Some(chunk) = byte_stream.next().await {
            if cancel.load(Ordering::Relaxed) {
                // Dropping `byte_stream`/`response` here (function return) closes the
                // connection; Ollama notices and stops generating on its end too.
                return Err("Generation cancelled".to_string());
            }

            let chunk = chunk.map_err(|err| format!("Stream error: {err}"))?;
            buffer.push_str(&String::from_utf8_lossy(&chunk));

            for parsed in drain_complete_lines(&mut buffer)? {
                if let Some(message) = parsed.message {
                    if let Some(thinking) = message.thinking.filter(|text| !text.is_empty()) {
                        let _ = app.emit(GENERATION_THINKING_EVENT, &thinking);
                    }
                    if !message.content.is_empty() {
                        full_text.push_str(&message.content);
                        let _ = app.emit(GENERATION_CHUNK_EVENT, &message.content);
                    }
                }

                if parsed.done {
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
pub fn cancel_generation(client: tauri::State<'_, OllamaClient>) {
    client.cancel_generation();
}

#[tauri::command]
pub async fn generate_edit(
    app: AppHandle,
    client: tauri::State<'_, OllamaClient>,
    model: String,
    markdown: String,
    instruction: String,
    thinking: bool,
    web_search: bool,
) -> Result<String, String> {
    client
        .generate_edit(&app, model, &markdown, &instruction, thinking, web_search)
        .await
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
}
