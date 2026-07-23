use base64::{engine::general_purpose::STANDARD, Engine};
use std::fs;

#[tauri::command]
pub fn read_image_base64(path: String) -> Result<String, String> {
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
}
