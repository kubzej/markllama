use base64::{engine::general_purpose::STANDARD, Engine};
use std::fs;

/// No real vision model benefits from images anywhere near this size, and without a cap a user
/// picking an arbitrarily large file would have it fully read into memory, base64-encoded (~33%
/// larger again), and shipped to Ollama with no warning.
const MAX_IMAGE_BYTES: u64 = 20 * 1024 * 1024;

#[tauri::command]
pub fn read_image_base64(path: String) -> Result<String, String> {
    let metadata = fs::metadata(&path).map_err(|err| format!("Could not read {path}: {err}"))?;
    if metadata.len() > MAX_IMAGE_BYTES {
        return Err(format!(
            "{path} is too large ({:.1} MB) — the limit is {} MB",
            metadata.len() as f64 / (1024.0 * 1024.0),
            MAX_IMAGE_BYTES / (1024 * 1024)
        ));
    }
    let bytes = fs::read(&path).map_err(|err| format!("Could not read {path}: {err}"))?;
    Ok(STANDARD.encode(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_bytes_through_read_and_base64_decode() {
        let path = std::env::temp_dir()
            .join(format!("markllama-image-test-{}.bin", std::process::id()))
            .to_string_lossy()
            .into_owned();
        let original_bytes: Vec<u8> = vec![0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a];

        fs::write(&path, &original_bytes).expect("write should succeed");
        let encoded = read_image_base64(path.clone()).expect("read should succeed");
        let decoded = STANDARD.decode(encoded).expect("should decode as base64");
        assert_eq!(decoded, original_bytes);

        fs::remove_file(&path).expect("cleanup should succeed");
    }

    #[test]
    fn read_image_base64_reports_missing_file() {
        let missing = "/nonexistent/markllama-test-missing.png".to_string();
        assert!(read_image_base64(missing).is_err());
    }

    #[test]
    fn rejects_files_over_the_size_cap() {
        let path = std::env::temp_dir()
            .join(format!("markllama-image-oversize-test-{}.bin", std::process::id()))
            .to_string_lossy()
            .into_owned();

        // A sparse file (no actual disk space used for the zero bytes) just over the cap —
        // enough to exercise the size check without writing 20MB+ to disk in a test.
        let file = fs::File::create(&path).expect("create should succeed");
        file.set_len(MAX_IMAGE_BYTES + 1).expect("set_len should succeed");
        drop(file);

        let result = read_image_base64(path.clone());
        assert!(result.is_err());

        fs::remove_file(&path).expect("cleanup should succeed");
    }
}
