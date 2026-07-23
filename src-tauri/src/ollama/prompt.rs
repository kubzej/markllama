//! # Context policy — read this before touching `build_chat_request`/`build_write_request`
//!
//! Markllama supports a real project-wide chat: you can discuss and research across a project's
//! markdown files with the model keeping the thread, and only *explicitly* ask it to write
//! something into a specific document when you're ready. Three rules keep that from becoming
//! unbounded context growth or cross-chat contamination:
//!   1. History is only ever what the user's own current chat actually contains. Nothing is
//!      auto-expanded, summarized in, or pulled in from anywhere else.
//!   2. A file's content enters a request *only* because the user explicitly attached it (or, for
//!      a "write" turn, because it's the explicit target document) — never automatically because
//!      it happens to exist in the project.
//!   3. Building a request for one chat must never be able to see another chat's turns. Chats are
//!      stored as separate files (`chat/store.rs`) specifically so there's no shared mutable state
//!      that could leak between them — `build_chat_request`/`build_write_request` only ever see
//!      whatever `history` the caller explicitly hands them for *that* chat.
//! If a future change needs more than this — e.g. pulling in content the user never attached —
//! that is an architecture change. Ask first.

use serde::Serialize;

const CHAT_SYSTEM_PROMPT: &str = "You are a helpful assistant discussing Markdown documents the \
user is working on. Answer questions, discuss ideas, and use any attached file content or web \
search results as reference. This is a conversation, not a document-editing turn — do not \
produce or rewrite a full document unless explicitly asked to.";

const WRITE_SYSTEM_PROMPT: &str = "You are a Markdown editing assistant. You are given the \
conversation so far, the current contents of the target document, and an instruction. Respond \
with the complete, updated Markdown document and nothing else: no explanations, no code fences, \
no commentary before or after. Keep everything the instruction and conversation don't ask you to \
change.";

/// Used when "Write to document" is clicked with no fresh instruction typed — the conversation
/// itself is the instruction in that case.
const DEFAULT_WRITE_INSTRUCTION: &str = "Based on our conversation, write the complete updated document.";

#[derive(Serialize, Clone)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub images: Vec<String>,
}

impl ChatMessage {
    fn new(role: &str, content: String, images: Vec<String>) -> Self {
        Self {
            role: role.to_string(),
            content,
            images,
        }
    }
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

/// Everything both `build_chat_request` and `build_write_request` need, bundled into one struct
/// rather than a long positional parameter list, which would otherwise drift as new fields
/// (images, num_ctx, ...) get added over time.
pub struct ChatContext {
    /// Prior turns in this chat, already compacted by the caller (e.g. a past "write" turn is
    /// represented as a short marker, not its full document text) — see `conversation.svelte.ts`.
    pub history: Vec<ChatMessage>,
    /// Explicitly attached files for *this* turn only: (project-relative path, content).
    pub attached_files: Vec<(String, String)>,
    /// Images attached to *this* turn only (base64, no `data:` prefix).
    pub images: Vec<String>,
    pub instruction: String,
    pub num_ctx: Option<u32>,
    pub thinking: bool,
    pub web_context: Option<String>,
}

fn attached_files_block(attached_files: &[(String, String)]) -> String {
    let mut block = String::new();
    for (path, content) in attached_files {
        block.push_str(&format!("Attached: {path}\n\n{content}\n\n---\n\n"));
    }
    block
}

fn append_web_context(content: &mut String, web_context: &Option<String>) {
    if let Some(context) = web_context {
        content.push_str("\n\nWeb search results (reference only, use if relevant):\n");
        content.push_str(context);
    }
}

/// Builds a request for a plain conversational turn — no document is read or written. See the
/// module-level context policy above before changing what goes into `messages`.
pub fn build_chat_request(model: String, ctx: ChatContext) -> ChatRequest {
    let mut content = attached_files_block(&ctx.attached_files);
    content.push_str(&ctx.instruction);
    append_web_context(&mut content, &ctx.web_context);

    let mut messages = vec![ChatMessage::new(
        "system",
        CHAT_SYSTEM_PROMPT.to_string(),
        Vec::new(),
    )];
    messages.extend(ctx.history);
    messages.push(ChatMessage::new("user", content, ctx.images));

    ChatRequest {
        model,
        messages,
        stream: true,
        think: ctx.thinking,
        options: ctx.num_ctx.map(|n| ChatOptions { num_ctx: n }),
    }
}

/// Builds a request for an explicit "write to document" turn — the one place a document's
/// content still enters a request automatically, because it's the explicit target being edited,
/// not an incidental project file. See the module-level context policy above.
pub fn build_write_request(model: String, target_document: &str, ctx: ChatContext) -> ChatRequest {
    let mut content = format!("Document:\n\n{target_document}\n\n");
    content.push_str(&attached_files_block(&ctx.attached_files));
    let instruction = if ctx.instruction.trim().is_empty() {
        DEFAULT_WRITE_INSTRUCTION
    } else {
        ctx.instruction.as_str()
    };
    content.push_str(&format!("Instruction: {instruction}"));
    append_web_context(&mut content, &ctx.web_context);

    let mut messages = vec![ChatMessage::new(
        "system",
        WRITE_SYSTEM_PROMPT.to_string(),
        Vec::new(),
    )];
    messages.extend(ctx.history);
    messages.push(ChatMessage::new("user", content, ctx.images));

    ChatRequest {
        model,
        messages,
        stream: true,
        think: ctx.thinking,
        options: ctx.num_ctx.map(|n| ChatOptions { num_ctx: n }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_ctx(instruction: &str) -> ChatContext {
        ChatContext {
            history: Vec::new(),
            attached_files: Vec::new(),
            images: Vec::new(),
            instruction: instruction.to_string(),
            num_ctx: None,
            thinking: false,
            web_context: None,
        }
    }

    #[test]
    fn chat_request_with_no_history_is_exactly_system_plus_user() {
        let request = build_chat_request("qwen3.5:9b".to_string(), empty_ctx("hello"));
        assert_eq!(request.messages.len(), 2);
        assert_eq!(request.messages[0].role, "system");
        assert_eq!(request.messages[1].role, "user");
        assert!(request.messages[1].content.contains("hello"));
    }

    /// History passed in for chat A must never appear when building a request for chat B — since
    /// there's no shared mutable state in this module, two independent calls can't leak into each
    /// other regardless of what history each one carries.
    #[test]
    fn history_from_one_chat_never_leaks_into_another_chats_request() {
        let chat_a_history = vec![
            ChatMessage::new("user", "chat A question".to_string(), Vec::new()),
            ChatMessage::new("assistant", "chat A answer".to_string(), Vec::new()),
        ];
        let mut ctx_a = empty_ctx("chat A follow-up");
        ctx_a.history = chat_a_history;
        let request_a = build_chat_request("qwen3.5:9b".to_string(), ctx_a);

        let request_b = build_chat_request("qwen3.5:9b".to_string(), empty_ctx("chat B question"));

        assert!(request_a
            .messages
            .iter()
            .any(|m| m.content.contains("chat A question")));
        assert!(!request_b
            .messages
            .iter()
            .any(|m| m.content.contains("chat A question") || m.content.contains("chat A answer")));
    }

    #[test]
    fn history_messages_appear_in_order_between_system_and_the_new_message() {
        let mut ctx = empty_ctx("latest message");
        ctx.history = vec![
            ChatMessage::new("user", "first".to_string(), Vec::new()),
            ChatMessage::new("assistant", "second".to_string(), Vec::new()),
            ChatMessage::new("user", "third".to_string(), Vec::new()),
            ChatMessage::new("assistant", "fourth".to_string(), Vec::new()),
        ];
        let request = build_chat_request("qwen3.5:9b".to_string(), ctx);

        let roles_and_content: Vec<(&str, &str)> = request
            .messages
            .iter()
            .map(|m| (m.role.as_str(), m.content.as_str()))
            .collect();
        assert_eq!(
            roles_and_content,
            vec![
                ("system", CHAT_SYSTEM_PROMPT),
                ("user", "first"),
                ("assistant", "second"),
                ("user", "third"),
                ("assistant", "fourth"),
                ("user", "latest message"),
            ]
        );
    }

    #[test]
    fn explicitly_attached_files_are_included_but_only_on_the_new_user_message() {
        let mut ctx = empty_ctx("what does this mean?");
        ctx.attached_files = vec![("notes.md".to_string(), "# Notes\n\nSome content".to_string())];
        let request = build_chat_request("qwen3.5:9b".to_string(), ctx);

        assert!(request.messages[0].content.is_empty() == false); // system prompt, sanity check
        let user_message = request.messages.last().unwrap();
        assert!(user_message.content.contains("Attached: notes.md"));
        assert!(user_message.content.contains("Some content"));
    }

    #[test]
    fn write_request_includes_the_target_document_and_a_write_specific_system_prompt() {
        let request = build_write_request(
            "qwen3.5:9b".to_string(),
            "# Existing doc",
            empty_ctx("add a summary"),
        );
        assert_eq!(request.messages[0].content, WRITE_SYSTEM_PROMPT);
        let user_message = &request.messages[1];
        assert!(user_message.content.contains("# Existing doc"));
        assert!(user_message.content.contains("add a summary"));
    }

    #[test]
    fn write_request_with_no_typed_instruction_falls_back_to_a_default() {
        let request = build_write_request(
            "qwen3.5:9b".to_string(),
            "# Existing doc",
            empty_ctx(""),
        );
        let user_message = &request.messages[1];
        assert!(user_message.content.contains(DEFAULT_WRITE_INSTRUCTION));
    }

    #[test]
    fn images_attach_only_to_the_new_user_message_not_history_or_system() {
        let mut ctx = empty_ctx("describe this image");
        ctx.images = vec!["base64data".to_string()];
        ctx.history = vec![ChatMessage::new("user", "earlier text-only message".to_string(), Vec::new())];
        let request = build_chat_request("qwen3.5:9b".to_string(), ctx);

        assert!(request.messages[0].images.is_empty());
        assert!(request.messages[1].images.is_empty());
        assert_eq!(request.messages.last().unwrap().images, vec!["base64data".to_string()]);
    }

    #[test]
    fn num_ctx_override_becomes_request_options() {
        let mut ctx = empty_ctx("do something");
        ctx.num_ctx = Some(8192);
        let with_override = build_chat_request("qwen3.5:9b".to_string(), ctx);
        assert_eq!(with_override.options, Some(ChatOptions { num_ctx: 8192 }));

        let without_override = build_chat_request("qwen3.5:9b".to_string(), empty_ctx("do something"));
        assert!(without_override.options.is_none());
    }
}
