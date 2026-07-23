# Markllama

A lightweight native macOS desktop client for [Ollama](https://ollama.com), focused on editing
Markdown documents. It sits between a plain chat client and a full AI agent: you write an
instruction, a local model proposes a full replacement document, Markllama computes a local diff,
and nothing changes until you explicitly **Apply**. The model never touches your files directly.

## Core workflow

1. Open a Markdown file (or just start typing).
2. Edit it manually if you want.
3. Write an instruction in the bar at the bottom.
4. Generate — the current document + your instruction are sent to a local Ollama model and
   streamed back.
5. Review the diff in the right-hand panel.
6. **Apply** to accept the change into the editor, or **Discard** to drop it. Either way, nothing
   is written to disk until you explicitly Save.

Every request sends exactly one thing: a small fixed system prompt, the current document, and
your instruction. No hidden conversation history, no project context, no planning step — each
Generate is a fresh, single-turn request built from whatever is currently in the editor.

## Requirements

- macOS (Apple Silicon), macOS 13 (Ventura) or later.
- [Ollama](https://ollama.com) installed and running locally (`ollama serve`, or the Ollama
  menu-bar app), with at least one model pulled (`ollama pull <model>`).
- Optional: an [Ollama Web Search](https://ollama.com/blog/web-search) API key if you want the
  Web Search toggle — inference itself always stays local; only that one optional feature talks
  to `ollama.com`. The key is stored in the macOS Keychain, never on disk or in logs.

## Development

```sh
npm install
npm run tauri dev
```

Useful commands:

| Command | What it does |
| --- | --- |
| `npm run tauri dev` | Runs the app with hot reload (starts the Vite dev server + Tauri). |
| `npm run check` | Type-checks the Svelte/TypeScript frontend. |
| `npm run lint` | Prettier + ESLint over the frontend. |
| `cargo check` (in `src-tauri/`) | Type-checks the Rust backend. |
| `cargo test` (in `src-tauri/`) | Runs the Rust test suite (Ollama client, diff engine, settings, Keychain). |
| `npm run tauri build` | Produces a release `.app`/`.dmg` in `src-tauri/target/release/bundle/`. |

## Architecture

```
src-tauri/src/
  ollama/       # HTTP client to localhost:11434, prompt building, web search
  fs/           # Open/save markdown documents
  diff/         # Line + word-level diff engine (similar crate)
  settings/     # Persisted preferences + Keychain-backed API key
  menu.rs       # Native macOS menu bar (File/Edit/App)

src/lib/
  components/   # Toolbar, Editor (CodeMirror 6), InstructionBar, DiffPanel, SettingsModal
  stores/       # Svelte 5 rune-based state: document, session, generation, ui
  tauri/        # Typed wrappers around Tauri commands/events — the only place the
                # frontend talks to Rust
  actions/      # Higher-level orchestration shared between UI buttons and the native menu
```

The frontend never calls Ollama directly — every network call and disk write happens in Rust,
exposed to the UI only through Tauri commands. That's what keeps "the AI never edits files
directly" an actual guarantee rather than a convention.

## Design principles

Native, fast, minimal, keyboard-friendly, offline-first, no telemetry, no cloud dependency except
the optional Web Search toggle.

## License

MIT — see [LICENSE](LICENSE).
