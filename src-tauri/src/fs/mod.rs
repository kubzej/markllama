pub mod document;
pub mod image;
pub mod project;

use std::path::Path;

/// Writes `contents` to `path` atomically: writes to a temp file in the same directory first,
/// then renames it into place. A crash or power loss mid-write leaves either the old file intact
/// or the new one complete — never a truncated/corrupted file the way a direct `fs::write` would
/// if interrupted partway through.
pub fn write_atomically(path: &Path, contents: &[u8]) -> std::io::Result<()> {
    let dir = path.parent().filter(|p| !p.as_os_str().is_empty()).unwrap_or_else(|| Path::new("."));
    let file_name = path.file_name().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, "path has no file name")
    })?;
    let tmp_path = dir.join(format!(
        ".{}.tmp-{}",
        file_name.to_string_lossy(),
        std::process::id()
    ));
    std::fs::write(&tmp_path, contents)?;
    std::fs::rename(&tmp_path, path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_atomically_round_trips_content() {
        let path = std::env::temp_dir().join(format!("markllama-atomic-test-{}.txt", std::process::id()));
        write_atomically(&path, b"hello").expect("write should succeed");
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "hello");
        // Overwriting an existing file must also succeed (rename replaces the destination).
        write_atomically(&path, b"updated").expect("overwrite should succeed");
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "updated");
        std::fs::remove_file(&path).unwrap();
    }
}
