<p align="center">
  <img src="src-tauri/icons/128x128@2x.png" width="96" height="96" alt="Markllama logo">
</p>

<h1 align="center">Markllama</h1>

<p align="center">
  <img src="https://img.shields.io/badge/platform-macOS-CC5F40" alt="macOS">
  <img src="https://img.shields.io/badge/built%20with-Tauri%202-CC5F40" alt="Tauri 2">
  <img src="https://img.shields.io/badge/license-MIT-CC5F40" alt="MIT License">
</p>

A lightweight, local-first bridge between Ollama and your Markdown files. It runs on models you already have pulled and keeps their context as small as the job needs. Built for working with Markdown with AI's help — not running an agent. Web search and thinking-model support included.

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
