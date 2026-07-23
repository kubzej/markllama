use std::process::Child;
use std::sync::Mutex;

use super::client::OllamaClient;

/// Common install locations for the `ollama` binary. Tried in order because GUI apps launched
/// from Finder/Launchpad don't inherit a shell's `PATH` (Homebrew's `/opt/homebrew/bin` in
/// particular is routinely missing), so a bare `Command::new("ollama")` can fail even though
/// `ollama` works fine from a terminal.
const OLLAMA_BINARY_CANDIDATES: [&str; 3] =
    ["ollama", "/opt/homebrew/bin/ollama", "/usr/local/bin/ollama"];

/// Tracks whether *this* Markllama process spawned `ollama serve`, so it only ever stops the
/// instance it started itself. An Ollama that was already running (started by the user, another
/// app, or a background service) is never touched — Markllama has no business killing a process
/// it doesn't own.
pub struct OllamaProcess {
    child: Mutex<Option<Child>>,
}

impl OllamaProcess {
    pub fn new() -> Self {
        Self {
            child: Mutex::new(None),
        }
    }

    /// Starts `ollama serve` if nothing is listening on `:11434` yet. No-op if Ollama is already
    /// reachable — that instance isn't ours, so it isn't tracked for shutdown either.
    pub async fn ensure_running(&self, client: &OllamaClient) {
        if client.detect().await {
            return;
        }

        match spawn_ollama_serve() {
            Ok(child) => {
                *self.child.lock().unwrap() = Some(child);
            }
            Err(err) => {
                log::warn!("Could not start `ollama serve` automatically: {err}");
            }
        }
    }

    /// Stops the process this instance spawned, if any — called on app exit. See the `Exit`/
    /// `ExitRequested` handling in `lib.rs` for why both events are hooked.
    pub fn stop_if_owned(&self) {
        if let Some(mut child) = self.child.lock().unwrap().take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

impl Default for OllamaProcess {
    fn default() -> Self {
        Self::new()
    }
}

fn spawn_ollama_serve() -> std::io::Result<Child> {
    let mut last_err = None;

    for candidate in OLLAMA_BINARY_CANDIDATES {
        match std::process::Command::new(candidate).arg("serve").spawn() {
            Ok(child) => return Ok(child),
            Err(err) => last_err = Some(err),
        }
    }

    Err(last_err.expect("OLLAMA_BINARY_CANDIDATES is non-empty"))
}
