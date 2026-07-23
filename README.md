<p align="center">
  <img src="src-tauri/icons/128x128@2x.png" width="96" height="96" alt="Markllama logo">
</p>

<h1 align="center">Markllama</h1>

<p align="center">
  <img src="https://img.shields.io/badge/platform-macOS-CC5F40" alt="macOS">
  <img src="https://img.shields.io/badge/built%20with-Tauri%202-CC5F40" alt="Tauri 2">
  <img src="https://img.shields.io/badge/license-MIT-CC5F40" alt="MIT License">
</p>

A lightweight, local-first bridge between Ollama and your Markdown files. It runs on models you already have pulled and keeps their context as small as the job needs. Built for working with Markdown with AI's help — not running an agent. Web search, independent instruction presets, and Ollama thinking-model support included.

## Minimal Context Design

Markllama is built around a small, explicit request instead of an agent loop. It does not scan your
project, build an index, retrieve hidden snippets, or carry a huge background prompt. The model
gets only the pieces needed for the current turn.

| Context piece            | When it is sent                                                                                 |
| ------------------------ | ----------------------------------------------------------------------------------------------- |
| Markllama system prompt  | Always. A short mode prompt: chat about Markdown, or return the complete updated Markdown file. |
| Instruction preset       | Only when you pick one. `No instructions` sends nothing extra.                                  |
| Current chat history     | Only from the active chat for this file/project. Switching chats switches the history.          |
| Active Markdown document | In chat mode, the currently open document is in scope automatically.                            |
| Write target document    | In write mode, the target document is sent as the document to edit.                             |
| Extra files              | Only files you explicitly attach for that turn.                                                 |
| Images                   | Only images attached to the current turn; old images are not replayed through history.          |
| Web search results       | Only when Web Search is enabled for that request.                                               |
| Thinking                 | Sent as Ollama's `think` flag for models that support it; it is not extra document context.     |
| Context window override  | Only when you set `num_ctx` for a model; otherwise Ollama's own default is used.                |

Markllama also avoids resending unchanged file content. If the active document or an attached file
has already been sent in this chat and has not changed, the next request can rely on that earlier
turn instead of sending the same text again. Past write turns are compacted to short markers like
`[Wrote changes to notes.md]`, not replayed as full old documents.

That is the point of the app: Markdown work with local models, without an always-on agent mode or
thousands of hidden tokens. You can add context deliberately; the default stays plain.

## Requirements

- macOS (Apple Silicon), macOS 13 (Ventura) or later.
- [Ollama](https://ollama.com) installed and running locally (`ollama serve`, or the Ollama
  menu-bar app), with at least one model pulled (`ollama pull <model>`).
- Optional: an [Ollama Web Search](https://ollama.com/blog/web-search) API key if you want the
  Web Search toggle — inference itself always stays local; only that one optional feature talks
  to `ollama.com`. The key is stored in the macOS Keychain, never on disk or in logs.

## Development

| Command                         | What it does                                                                                 |
| ------------------------------- | -------------------------------------------------------------------------------------------- |
| `npm install`                   | Installs frontend dependencies.                                                              |
| `npm run tauri dev`             | Runs the app with hot reload (starts the Vite dev server + Tauri).                           |
| `npm run check`                 | Type-checks the Svelte/TypeScript frontend.                                                  |
| `npm run lint`                  | Prettier + ESLint over the frontend.                                                         |
| `cargo check` (in `src-tauri/`) | Type-checks the Rust backend.                                                                |
| `cargo test` (in `src-tauri/`)  | Runs the Rust test suite (Ollama client, diff engine, settings, project scan, chat storage). |
| `npm run tauri build`           | Produces a release `.app`/`.dmg` in `src-tauri/target/release/bundle/`.                      |

## License

MIT — see [LICENSE](LICENSE).
