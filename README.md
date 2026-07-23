<p align="center">
  <img src="src-tauri/icons/128x128@2x.png" width="96" height="96" alt="Markllama logo">
</p>

<h1 align="center">Markllama</h1>

<p align="center">
  <img src="https://img.shields.io/badge/platform-macOS-CC5F40" alt="macOS">
  <img src="https://img.shields.io/badge/built%20with-Tauri%202-CC5F40" alt="Tauri 2">
  <img src="https://img.shields.io/badge/license-MIT-CC5F40" alt="MIT License">
</p>

A lightweight bridge between [Ollama](https://ollama.com) and your Markdown files, with web
search and thinking-model support built in. Point it at a `.md` file, or a whole folder of them,
for brainstorming, planning, or quick research — write an instruction, Generate, and review the
answer as a diff against the document. Nothing is applied or saved until you say so. Markdown is
the new programming language; this is an editor for it.

Local inference is free, but it's not fast — and it gets slower the more context you hand it. So
every request stays as small as it can possibly be: a fixed system prompt, the current document,
and your instruction. Nothing more — no accumulated chat history, no whole-project context, no
agentic planning step. Every Generate is one fresh, single-turn request built from whatever's on
screen right now, no matter how long you've been chatting or how many files are in the folder.
The chat log is local and disposable, for your own reference only — it's never fed back into a
prompt.

## Requirements

- macOS (Apple Silicon), macOS 13 (Ventura) or later.
- [Ollama](https://ollama.com) installed and running locally (`ollama serve`, or the Ollama
  menu-bar app), with at least one model pulled (`ollama pull <model>`).
- Optional: an [Ollama Web Search](https://ollama.com/blog/web-search) API key if you want the
  Web Search toggle — inference itself always stays local; only that one optional feature talks
  to `ollama.com`. The key is stored in the macOS Keychain, never on disk or in logs.

## Development

| Command | What it does |
| --- | --- |
| `npm install` | Installs frontend dependencies. |
| `npm run tauri dev` | Runs the app with hot reload (starts the Vite dev server + Tauri). |
| `npm run check` | Type-checks the Svelte/TypeScript frontend. |
| `npm run lint` | Prettier + ESLint over the frontend. |
| `cargo check` (in `src-tauri/`) | Type-checks the Rust backend. |
| `cargo test` (in `src-tauri/`) | Runs the Rust test suite (Ollama client, diff engine, settings, project scan). |
| `npm run tauri build` | Produces a release `.app`/`.dmg` in `src-tauri/target/release/bundle/`. |

## License

MIT — see [LICENSE](LICENSE).
