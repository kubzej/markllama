//! # CORE INVARIANT — DO NOT ACCUMULATE HISTORY HERE
//!
//! The whole point of Markllama is to keep the local model's context window small and cheap.
//! Every single call to `build_edit_request` MUST produce a request containing exactly:
//!   1. the fixed system prompt,
//!   2. the CURRENT markdown document (as it is right now, not any prior version),
//!   3. the CURRENT instruction (not any prior instruction).
//! That's it. Two messages, always. Never append previous instructions, previous model
//! responses, or a running conversation log to `messages`. The chat log the frontend shows
//! (`conversation.svelte.ts`) is a local, in-memory, UI-only display for the user's own
//! reference — it is never read back into this function, and it must stay that way.
//!
//! If a future change needs the model to see more than "current doc + current instruction",
//! that is an explicit, user-approved architecture change, not something to slip in while
//! adding an unrelated feature. Ask first.
//!
//! Attached images follow the same rule: they belong to the current turn's request only, exactly
//! like the instruction text — never accumulated or carried over from a previous turn.

use serde::Serialize;

const SYSTEM_PROMPT: &str = "You are a Markdown editing assistant. You are given the full \
contents of a Markdown document and an instruction describing a change to make to it. Respond \
with the complete, updated Markdown document and nothing else: no explanations, no code fences, \
no commentary before or after. Keep everything from the original document that the instruction \
does not ask you to change.";

#[derive(Serialize)]
pub struct ChatMessage {
    pub role: &'static str,
    pub content: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub images: Vec<String>,
}

#[derive(Serialize, Debug, PartialEq)]
pub struct ChatOptions {
    pub num_ctx: u32,
}

#[derive(Serialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub stream: bool,
    pub think: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<ChatOptions>,
}

/// Builds a single-turn request: system prompt + current markdown + current instruction, and
/// nothing else. See the module-level CORE INVARIANT note above before touching this function.
pub fn build_edit_request(
    model: String,
    markdown: &str,
    instruction: &str,
    images: Vec<String>,
    num_ctx: Option<u32>,
    thinking: bool,
    web_context: Option<&str>,
) -> ChatRequest {
    let mut user_content = format!("Document:\n\n{markdown}\n\nInstruction: {instruction}");
    if let Some(context) = web_context {
        user_content
            .push_str("\n\nWeb search results (reference only, use if relevant):\n");
        user_content.push_str(context);
    }

    ChatRequest {
        model,
        messages: vec![
            ChatMessage {
                role: "system",
                content: SYSTEM_PROMPT.to_string(),
                images: Vec::new(),
            },
            ChatMessage {
                role: "user",
                content: user_content,
                images,
            },
        ],
        stream: true,
        think: thinking,
        options: num_ctx.map(|n| ChatOptions { num_ctx: n }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Guards the CORE INVARIANT documented at the top of this module: exactly two messages
    /// (system + user), no assistant/history messages, ever — regardless of how many times this
    /// gets called or with what arguments. If this test fails, something started accumulating
    /// conversation history into the prompt, which is the one thing this app must never do.
    #[test]
    fn every_request_is_exactly_system_plus_current_turn_no_history() {
        let request = build_edit_request(
            "qwen3.5:9b".to_string(),
            "# Doc",
            "do something",
            Vec::new(),
            None,
            false,
            None,
        );

        assert_eq!(request.messages.len(), 2, "must be exactly system + user, no more");
        assert_eq!(request.messages[0].role, "system");
        assert_eq!(request.messages[1].role, "user");
        assert!(
            !request.messages.iter().any(|m| m.role == "assistant"),
            "a request must never carry a prior assistant/model turn"
        );
    }

    /// Simulates the exact regression this invariant protects against: two independent calls
    /// (as if the user sent two separate instructions in the chat) must never leak one call's
    /// instruction or document into the other's request.
    #[test]
    fn independent_calls_never_leak_into_each_other() {
        let first = build_edit_request(
            "qwen3.5:9b".to_string(),
            "# First document",
            "first instruction",
            Vec::new(),
            None,
            false,
            None,
        );
        let second = build_edit_request(
            "qwen3.5:9b".to_string(),
            "# Second document",
            "second instruction",
            Vec::new(),
            None,
            false,
            None,
        );

        assert_eq!(second.messages.len(), 2);
        assert!(!second.messages[1].content.contains("first instruction"));
        assert!(!second.messages[1].content.contains("First document"));
        assert!(second.messages[1].content.contains("second instruction"));
        assert!(second.messages[1].content.contains("Second document"));

        // The first call's own request is untouched by the second call (no shared mutable state).
        assert!(first.messages[1].content.contains("first instruction"));
        assert!(!first.messages[1].content.contains("second instruction"));
    }

    /// Attached images ride along with the current turn's request only, same as the
    /// instruction — they must land on the user message, never on the system message.
    #[test]
    fn images_attach_only_to_the_user_message() {
        let request = build_edit_request(
            "qwen3.5:9b".to_string(),
            "# Doc",
            "describe this image",
            vec!["base64data".to_string()],
            None,
            false,
            None,
        );

        assert!(request.messages[0].images.is_empty());
        assert_eq!(request.messages[1].images, vec!["base64data".to_string()]);
    }

    /// An explicit `num_ctx` override becomes the request's `options`; leaving it unset omits
    /// `options` entirely rather than sending an explicit "use the default" value.
    #[test]
    fn num_ctx_override_becomes_request_options() {
        let with_override = build_edit_request(
            "qwen3.5:9b".to_string(),
            "# Doc",
            "do something",
            Vec::new(),
            Some(8192),
            false,
            None,
        );
        assert_eq!(with_override.options, Some(ChatOptions { num_ctx: 8192 }));

        let without_override = build_edit_request(
            "qwen3.5:9b".to_string(),
            "# Doc",
            "do something",
            Vec::new(),
            None,
            false,
            None,
        );
        assert!(without_override.options.is_none());
    }
}
