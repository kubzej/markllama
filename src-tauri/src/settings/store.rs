use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

const SETTINGS_FILE_NAME: &str = "settings.json";

/// A user-written alias/description for a model, keyed by the model's exact Ollama name
/// (e.g. `"qwen3.5:9b"`) in `Settings.model_notes`. Purely local organization — never sent to
/// Ollama or read by the prompt-building code.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct ModelNote {
    pub alias: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct InstructionPreset {
    pub id: String,
    pub name: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    pub last_model: Option<String>,
    pub thinking_default: bool,
    pub web_search_default: bool,
    pub selected_instruction_id: Option<String>,
    pub model_notes: HashMap<String, ModelNote>,
    /// Per-model `num_ctx` override, keyed by exact Ollama model name. Absent entry means "use
    /// Ollama's own default" — never sent to `/api/chat` as an explicit value in that case.
    pub num_ctx_overrides: HashMap<String, u32>,
    /// User-managed behavior instructions, selected independently from the model. Empty means
    /// the toolbar still offers "No instructions" and no extra system guidance is sent.
    pub instruction_presets: Vec<InstructionPreset>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            last_model: None,
            thinking_default: false,
            web_search_default: false,
            selected_instruction_id: None,
            model_notes: HashMap::new(),
            num_ctx_overrides: HashMap::new(),
            instruction_presets: Vec::new(),
        }
    }
}

fn settings_path(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_config_dir()
        .map_err(|err| format!("Could not resolve config directory: {err}"))?;
    std::fs::create_dir_all(&dir)
        .map_err(|err| format!("Could not create config directory: {err}"))?;
    Ok(dir.join(SETTINGS_FILE_NAME))
}

#[tauri::command]
pub fn get_settings(app: AppHandle) -> Result<Settings, String> {
    let path = settings_path(&app)?;
    if !path.exists() {
        return Ok(Settings::default());
    }
    let raw =
        std::fs::read_to_string(&path).map_err(|err| format!("Could not read settings: {err}"))?;
    // A corrupted settings.json (e.g. truncated by a crash mid-write, before the atomic write
    // below existed) should degrade to defaults rather than permanently locking the user out of
    // the app on every subsequent launch.
    match serde_json::from_str(&raw) {
        Ok(settings) => Ok(settings),
        Err(err) => {
            log::warn!("Could not parse settings.json ({err}); falling back to defaults");
            Ok(Settings::default())
        }
    }
}

#[tauri::command]
pub fn set_settings(app: AppHandle, settings: Settings) -> Result<(), String> {
    let path = settings_path(&app)?;
    let raw = serde_json::to_string_pretty(&settings)
        .map_err(|err| format!("Could not serialize settings: {err}"))?;
    crate::fs::write_atomically(&path, raw.as_bytes())
        .map_err(|err| format!("Could not write settings: {err}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A settings.json written before `model_notes` existed must still load — `#[serde(default)]`
    /// should backfill an empty map rather than failing to parse.
    #[test]
    fn old_settings_without_model_notes_still_parse() {
        let old = r#"{"lastModel":"qwen3.5:9b","thinkingDefault":true,"webSearchDefault":false}"#;
        let parsed: Settings = serde_json::from_str(old).expect("should parse");
        assert_eq!(parsed.last_model, Some("qwen3.5:9b".to_string()));
        assert!(parsed.model_notes.is_empty());
    }

    #[test]
    fn model_notes_round_trip_through_json() {
        let mut settings = Settings::default();
        settings.model_notes.insert(
            "qwen3.5:9b".to_string(),
            ModelNote {
                alias: "Reasoner".to_string(),
                description: "Good at multi-step edits".to_string(),
            },
        );

        let json = serde_json::to_string(&settings).expect("should serialize");
        let parsed: Settings = serde_json::from_str(&json).expect("should parse");
        let note = parsed.model_notes.get("qwen3.5:9b").expect("note present");
        assert_eq!(note.alias, "Reasoner");
        assert_eq!(note.description, "Good at multi-step edits");
    }

    /// A settings.json written before `num_ctx_overrides` existed must still load —
    /// `#[serde(default)]` should backfill an empty map rather than failing to parse.
    #[test]
    fn old_settings_without_num_ctx_overrides_still_parse() {
        let old = r#"{"lastModel":"qwen3.5:9b","thinkingDefault":true,"webSearchDefault":false}"#;
        let parsed: Settings = serde_json::from_str(old).expect("should parse");
        assert!(parsed.num_ctx_overrides.is_empty());
    }

    #[test]
    fn num_ctx_overrides_round_trip_through_json() {
        let mut settings = Settings::default();
        settings
            .num_ctx_overrides
            .insert("qwen3.5:9b".to_string(), 8192);

        let json = serde_json::to_string(&settings).expect("should serialize");
        let parsed: Settings = serde_json::from_str(&json).expect("should parse");
        assert_eq!(parsed.num_ctx_overrides.get("qwen3.5:9b"), Some(&8192));
    }

    #[test]
    fn old_settings_without_instruction_presets_still_parse() {
        let old = r#"{"lastModel":"qwen3.5:9b","thinkingDefault":true,"webSearchDefault":false}"#;
        let parsed: Settings = serde_json::from_str(old).expect("should parse");
        assert!(parsed.instruction_presets.is_empty());
        assert!(parsed.selected_instruction_id.is_none());
    }

    #[test]
    fn instruction_presets_round_trip_through_json() {
        let mut settings = Settings::default();
        settings.selected_instruction_id = Some("strict".to_string());
        settings.instruction_presets.push(InstructionPreset {
            id: "strict".to_string(),
            name: "Strict editor".to_string(),
            text: "Be concise and edit directly.".to_string(),
        });

        let json = serde_json::to_string(&settings).expect("should serialize");
        let parsed: Settings = serde_json::from_str(&json).expect("should parse");
        assert_eq!(parsed.selected_instruction_id, Some("strict".to_string()));
        assert_eq!(parsed.instruction_presets[0].name, "Strict editor");
        assert_eq!(
            parsed.instruction_presets[0].text,
            "Be concise and edit directly."
        );
    }

    /// Truncated/corrupted JSON (e.g. a crash mid-write) must not fail to deserialize into
    /// `Settings` in a way callers can't recover from — `get_settings`'s fallback-to-default
    /// path is what actually handles this at the command level; this test just confirms the
    /// underlying parse genuinely fails on garbage input, which is the precondition that path
    /// relies on.
    #[test]
    fn truncated_json_fails_to_parse() {
        let truncated = r#"{"lastModel":"qwen3.5:9b","thinkingDef"#;
        assert!(serde_json::from_str::<Settings>(truncated).is_err());
    }
}
