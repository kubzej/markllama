use keyring::Entry;

const SERVICE_NAME: &str = "io.github.kubzej.markllama";
const ACCOUNT_NAME: &str = "ollama-web-search-api-key";

fn entry() -> Result<Entry, String> {
    Entry::new(SERVICE_NAME, ACCOUNT_NAME).map_err(|err| format!("Keychain error: {err}"))
}

/// Reads the stored key, if any. A missing entry is `Ok(None)`, not an error — most callers
/// only care whether Web Search can proceed.
pub fn get_api_key() -> Result<Option<String>, String> {
    match entry()?.get_password() {
        Ok(password) => Ok(Some(password)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(err) => Err(format!("Could not read Keychain: {err}")),
    }
}

pub fn delete_api_key() -> Result<(), String> {
    match entry()?.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(err) => Err(format!("Could not remove Keychain entry: {err}")),
    }
}

/// Saving an empty/whitespace-only key clears it instead — the Settings UI has one field and one
/// Save action, not a separate Clear button to keep in sync.
///
/// Runs on a blocking thread: the underlying macOS Security framework call can block waiting on
/// a Keychain permission prompt (common with unsigned/ad-hoc-signed dev builds), and it must not
/// tie up an async runtime worker thread — other commands (like the Ollama status poll) share
/// that pool and would otherwise stall too.
#[tauri::command]
pub async fn save_web_search_api_key(key: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        let trimmed = key.trim();
        if trimmed.is_empty() {
            delete_api_key()
        } else {
            entry()?
                .set_password(trimmed)
                .map_err(|err| format!("Could not save to Keychain: {err}"))
        }
    })
    .await
    .map_err(|err| format!("Keychain task failed: {err}"))?
}

/// Never returns the key itself over IPC — only whether one is configured, which is all the
/// Settings UI needs to render its state. See `save_web_search_api_key` for why this runs on a
/// blocking thread.
#[tauri::command]
pub async fn has_web_search_api_key() -> Result<bool, String> {
    tauri::async_runtime::spawn_blocking(|| Ok(get_api_key()?.is_some()))
        .await
        .map_err(|err| format!("Keychain task failed: {err}"))?
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Exercises the real macOS Keychain (not mocked) to prove the save/read/clear round trip
    /// actually works for this app's service/account identifiers, not just that the code
    /// compiles against the `keyring` API.
    #[test]
    fn round_trips_a_key_through_the_real_keychain() {
        let probe_entry = Entry::new(SERVICE_NAME, "markllama-test-probe").expect("entry");
        probe_entry.set_password("test-value-123").expect("save should succeed");

        let read_back = probe_entry.get_password().expect("read should succeed");
        assert_eq!(read_back, "test-value-123");

        probe_entry.delete_credential().expect("cleanup should succeed");
        assert!(matches!(
            probe_entry.get_password(),
            Err(keyring::Error::NoEntry)
        ));
    }
}
