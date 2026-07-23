use std::fs;
use std::path::Path;

#[tauri::command]
pub fn read_document(path: String) -> Result<String, String> {
    fs::read_to_string(&path).map_err(|err| format!("Could not read {path}: {err}"))
}

#[tauri::command]
pub fn write_document(path: String, content: String) -> Result<(), String> {
    super::write_atomically(Path::new(&path), content.as_bytes())
        .map_err(|err| format!("Could not write {path}: {err}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_content_through_write_and_read() {
        let path = std::env::temp_dir()
            .join(format!("markllama-test-{}.md", std::process::id()))
            .to_string_lossy()
            .into_owned();

        write_document(path.clone(), "# Hello".into()).expect("write should succeed");
        let read_back = read_document(path.clone()).expect("read should succeed");
        assert_eq!(read_back, "# Hello");

        fs::remove_file(&path).expect("cleanup should succeed");
    }

    #[test]
    fn read_document_reports_missing_file() {
        let missing = "/nonexistent/markllama-test-missing.md".to_string();
        assert!(read_document(missing).is_err());
    }
}
