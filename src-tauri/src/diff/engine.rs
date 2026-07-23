use serde::Serialize;
use similar::{DiffOp, TextDiff};

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct WordSpan {
    pub text: String,
    pub changed: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum DiffLine {
    Unchanged { text: String },
    Removed { text: String },
    Added { text: String },
    Changed { old: Vec<WordSpan>, new: Vec<WordSpan> },
}

/// Line-level diff with inline word-level spans on changed lines, so prose edits stay readable
/// instead of showing a whole reflowed paragraph as removed+added.
pub fn diff_markdown(old: &str, new: &str) -> Vec<DiffLine> {
    let line_diff = TextDiff::from_lines(old, new);
    let old_lines = line_diff.old_slices();
    let new_lines = line_diff.new_slices();
    let mut result = Vec::new();

    for op in line_diff.ops() {
        match *op {
            DiffOp::Equal { old_index, len, .. } => {
                for i in 0..len {
                    result.push(DiffLine::Unchanged {
                        text: strip_newline(old_lines[old_index + i]),
                    });
                }
            }
            DiffOp::Delete { old_index, old_len, .. } => {
                for i in 0..old_len {
                    result.push(DiffLine::Removed {
                        text: strip_newline(old_lines[old_index + i]),
                    });
                }
            }
            DiffOp::Insert { new_index, new_len, .. } => {
                for i in 0..new_len {
                    result.push(DiffLine::Added {
                        text: strip_newline(new_lines[new_index + i]),
                    });
                }
            }
            DiffOp::Replace {
                old_index,
                old_len,
                new_index,
                new_len,
            } => {
                if old_len == 1 && new_len == 1 {
                    let old_line = strip_newline(old_lines[old_index]);
                    let new_line = strip_newline(new_lines[new_index]);
                    let (old_spans, new_spans) = diff_line_words(&old_line, &new_line);
                    result.push(DiffLine::Changed {
                        old: old_spans,
                        new: new_spans,
                    });
                } else {
                    // A multi-line replace group doesn't map 1:1 for word-level pairing —
                    // fall back to whole-line removed/added rather than guessing pairs.
                    for i in 0..old_len {
                        result.push(DiffLine::Removed {
                            text: strip_newline(old_lines[old_index + i]),
                        });
                    }
                    for i in 0..new_len {
                        result.push(DiffLine::Added {
                            text: strip_newline(new_lines[new_index + i]),
                        });
                    }
                }
            }
        }
    }

    result
}

fn strip_newline(line: &str) -> String {
    line.strip_suffix('\n').unwrap_or(line).to_string()
}

fn diff_line_words(old_line: &str, new_line: &str) -> (Vec<WordSpan>, Vec<WordSpan>) {
    let word_diff = TextDiff::from_words(old_line, new_line);
    let old_words = word_diff.old_slices();
    let new_words = word_diff.new_slices();
    let mut old_spans = Vec::new();
    let mut new_spans = Vec::new();

    for op in word_diff.ops() {
        match *op {
            DiffOp::Equal {
                old_index,
                new_index,
                len,
            } => {
                let old_text: String = old_words[old_index..old_index + len].concat();
                let new_text: String = new_words[new_index..new_index + len].concat();
                old_spans.push(WordSpan {
                    text: old_text,
                    changed: false,
                });
                new_spans.push(WordSpan {
                    text: new_text,
                    changed: false,
                });
            }
            DiffOp::Delete {
                old_index, old_len, ..
            } => {
                let text: String = old_words[old_index..old_index + old_len].concat();
                old_spans.push(WordSpan { text, changed: true });
            }
            DiffOp::Insert {
                new_index, new_len, ..
            } => {
                let text: String = new_words[new_index..new_index + new_len].concat();
                new_spans.push(WordSpan { text, changed: true });
            }
            DiffOp::Replace {
                old_index,
                old_len,
                new_index,
                new_len,
            } => {
                let old_text: String = old_words[old_index..old_index + old_len].concat();
                let new_text: String = new_words[new_index..new_index + new_len].concat();
                old_spans.push(WordSpan {
                    text: old_text,
                    changed: true,
                });
                new_spans.push(WordSpan {
                    text: new_text,
                    changed: true,
                });
            }
        }
    }

    (old_spans, new_spans)
}

#[tauri::command]
pub fn diff_documents(old: String, new: String) -> Vec<DiffLine> {
    diff_markdown(&old, &new)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unchanged_document_has_only_unchanged_lines() {
        let doc = "# Title\n\nSome text.\n";
        let result = diff_markdown(doc, doc);
        assert!(result.iter().all(|line| matches!(line, DiffLine::Unchanged { .. })));
    }

    #[test]
    fn single_line_edit_produces_a_word_level_changed_line() {
        let old = "# Hello\n\nBody text.\n";
        let new = "# Goodbye\n\nBody text.\n";
        let result = diff_markdown(old, new);

        let changed: Vec<_> = result
            .iter()
            .filter(|line| matches!(line, DiffLine::Changed { .. }))
            .collect();
        assert_eq!(changed.len(), 1);

        let DiffLine::Changed { old, new } = changed[0] else {
            unreachable!()
        };
        assert!(old.iter().any(|span| span.changed && span.text.contains("Hello")));
        assert!(new.iter().any(|span| span.changed && span.text.contains("Goodbye")));
        // The unchanged "# " prefix should not be marked as changed.
        assert!(old.iter().any(|span| !span.changed));
    }

    #[test]
    fn appended_paragraph_is_a_pure_addition() {
        let old = "# Title\n";
        let new = "# Title\n\nNew paragraph.\n";
        let result = diff_markdown(old, new);

        assert!(result.iter().any(|line| matches!(line, DiffLine::Unchanged { .. })));
        assert!(result
            .iter()
            .any(|line| matches!(line, DiffLine::Added { text } if text == "New paragraph.")));
        assert!(!result.iter().any(|line| matches!(line, DiffLine::Removed { .. })));
    }

    #[test]
    fn removed_paragraph_is_a_pure_removal() {
        let old = "# Title\n\nOld paragraph.\n";
        let new = "# Title\n";
        let result = diff_markdown(old, new);

        assert!(result
            .iter()
            .any(|line| matches!(line, DiffLine::Removed { text } if text == "Old paragraph.")));
        assert!(!result.iter().any(|line| matches!(line, DiffLine::Added { .. })));
    }
}
