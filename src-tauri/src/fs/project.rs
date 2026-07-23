use serde::Serialize;
use std::fs;
use std::path::Path;

#[derive(Serialize, Clone)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum ProjectNode {
    File {
        name: String,
        path: String,
    },
    Dir {
        name: String,
        path: String,
        children: Vec<ProjectNode>,
    },
}

#[tauri::command]
pub fn scan_project(root: String) -> Result<ProjectNode, String> {
    let root_path = Path::new(&root);
    let name = root_path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| root.clone());
    let children = scan_dir(root_path).map_err(|err| format!("Could not scan {root}: {err}"))?;
    Ok(ProjectNode::Dir {
        name,
        path: root,
        children,
    })
}

/// Recursively scans `dir`. Returns only markdown files and directories that contain at least
/// one markdown file anywhere in their subtree — directories with none anywhere inside are
/// pruned entirely (bottom-up: a subdirectory is only kept if its own scan came back non-empty).
/// Symlinks and dot-prefixed directories (.git, .obsidian, …) are skipped.
fn scan_dir(dir: &Path) -> std::io::Result<Vec<ProjectNode>> {
    let mut dirs = Vec::new();
    let mut files = Vec::new();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        if file_type.is_symlink() {
            continue;
        }
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();

        if file_type.is_dir() {
            if name.starts_with('.') {
                continue;
            }
            let children = scan_dir(&path)?;
            if !children.is_empty() {
                dirs.push(ProjectNode::Dir {
                    name,
                    path: path.to_string_lossy().into_owned(),
                    children,
                });
            }
        } else if file_type.is_file() && is_markdown(&name) {
            files.push(ProjectNode::File {
                name,
                path: path.to_string_lossy().into_owned(),
            });
        }
    }

    dirs.sort_by_key(|n| node_name(n).to_lowercase());
    files.sort_by_key(|n| node_name(n).to_lowercase());
    dirs.extend(files);
    Ok(dirs)
}

fn is_markdown(name: &str) -> bool {
    let lower = name.to_lowercase();
    lower.ends_with(".md") || lower.ends_with(".markdown")
}

fn node_name(node: &ProjectNode) -> &str {
    match node {
        ProjectNode::File { name, .. } | ProjectNode::Dir { name, .. } => name,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs as stdfs;

    struct TempDir {
        path: std::path::PathBuf,
    }

    impl TempDir {
        fn new(label: &str) -> Self {
            let path = std::env::temp_dir().join(format!(
                "markllama-project-test-{label}-{}",
                std::process::id()
            ));
            let _ = stdfs::remove_dir_all(&path);
            stdfs::create_dir_all(&path).expect("create temp dir");
            TempDir { path }
        }

        fn join(&self, rel: &str) -> std::path::PathBuf {
            self.path.join(rel)
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = stdfs::remove_dir_all(&self.path);
        }
    }

    fn names(nodes: &[ProjectNode]) -> Vec<&str> {
        nodes.iter().map(node_name).collect()
    }

    #[test]
    fn folder_with_no_markdown_anywhere_is_pruned() {
        let tmp = TempDir::new("prune");
        stdfs::create_dir_all(tmp.join("empty_sub")).unwrap();
        stdfs::write(tmp.join("empty_sub/notes.txt"), "x").unwrap();

        let result = scan_dir(&tmp.path).unwrap();
        assert!(names(&result).is_empty());
    }

    #[test]
    fn folder_with_deeply_nested_markdown_is_kept_with_intermediate_dirs() {
        let tmp = TempDir::new("nested");
        stdfs::create_dir_all(tmp.join("a/b/c")).unwrap();
        stdfs::write(tmp.join("a/b/c/deep.md"), "# deep").unwrap();

        let result = scan_dir(&tmp.path).unwrap();
        assert_eq!(names(&result), vec!["a"]);
        let ProjectNode::Dir { children: b_children, .. } = &result[0] else {
            panic!("expected dir");
        };
        assert_eq!(names(b_children), vec!["b"]);
    }

    #[test]
    fn only_markdown_files_are_included() {
        let tmp = TempDir::new("filter");
        stdfs::write(tmp.join("keep.md"), "# keep").unwrap();
        stdfs::write(tmp.join("keep2.markdown"), "# keep2").unwrap();
        stdfs::write(tmp.join("skip.txt"), "skip").unwrap();

        let result = scan_dir(&tmp.path).unwrap();
        assert_eq!(names(&result), vec!["keep.md", "keep2.markdown"]);
    }

    #[test]
    fn dot_prefixed_directories_are_skipped_even_with_markdown_inside() {
        let tmp = TempDir::new("dotdir");
        stdfs::create_dir_all(tmp.join(".git")).unwrap();
        stdfs::write(tmp.join(".git/HEAD.md"), "# nope").unwrap();

        let result = scan_dir(&tmp.path).unwrap();
        assert!(names(&result).is_empty());
    }

    #[test]
    fn directories_sort_before_files_both_case_insensitive() {
        let tmp = TempDir::new("sort");
        stdfs::write(tmp.join("Zebra.md"), "z").unwrap();
        stdfs::write(tmp.join("apple.md"), "a").unwrap();
        stdfs::create_dir_all(tmp.join("Beta")).unwrap();
        stdfs::write(tmp.join("Beta/x.md"), "x").unwrap();
        stdfs::create_dir_all(tmp.join("alpha")).unwrap();
        stdfs::write(tmp.join("alpha/y.md"), "y").unwrap();

        let result = scan_dir(&tmp.path).unwrap();
        assert_eq!(names(&result), vec!["alpha", "Beta", "apple.md", "Zebra.md"]);
    }

    #[test]
    fn scan_project_wraps_root_as_dir_node() {
        let tmp = TempDir::new("root");
        stdfs::write(tmp.join("a.md"), "a").unwrap();

        let root = scan_project(tmp.path.to_string_lossy().into_owned()).unwrap();
        match root {
            ProjectNode::Dir { children, .. } => assert_eq!(names(&children), vec!["a.md"]),
            _ => panic!("expected root Dir node"),
        }
    }
}
