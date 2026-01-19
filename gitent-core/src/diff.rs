use crate::error::Result;
use crate::models::Change;
use similar::{ChangeTag, TextDiff};

#[derive(Debug, Clone)]
pub struct FileDiff {
    pub path: String,
    pub old_content: Option<String>,
    pub new_content: Option<String>,
    pub diff_lines: Vec<DiffLine>,
}

#[derive(Debug, Clone)]
pub struct DiffLine {
    pub line_type: DiffLineType,
    pub content: String,
    pub old_line_number: Option<usize>,
    pub new_line_number: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffLineType {
    Context,
    Addition,
    Deletion,
}

impl FileDiff {
    pub fn from_change(change: &Change) -> Result<Self> {
        let old_content = change
            .content_before
            .as_ref()
            .and_then(|bytes| String::from_utf8(bytes.clone()).ok());

        let new_content = change
            .content_after
            .as_ref()
            .and_then(|bytes| String::from_utf8(bytes.clone()).ok());

        let diff_lines = if let (Some(old), Some(new)) = (&old_content, &new_content) {
            Self::compute_diff(old, new)
        } else {
            Vec::new()
        };

        Ok(FileDiff {
            path: change.path.to_string_lossy().to_string(),
            old_content,
            new_content,
            diff_lines,
        })
    }

    fn compute_diff(old_text: &str, new_text: &str) -> Vec<DiffLine> {
        let diff = TextDiff::from_lines(old_text, new_text);
        let mut lines = Vec::new();
        let mut old_line_num = 1;
        let mut new_line_num = 1;

        for change in diff.iter_all_changes() {
            let (line_type, old_num, new_num) = match change.tag() {
                ChangeTag::Delete => {
                    let num = old_line_num;
                    old_line_num += 1;
                    (DiffLineType::Deletion, Some(num), None)
                }
                ChangeTag::Insert => {
                    let num = new_line_num;
                    new_line_num += 1;
                    (DiffLineType::Addition, None, Some(num))
                }
                ChangeTag::Equal => {
                    let old_num = old_line_num;
                    let new_num = new_line_num;
                    old_line_num += 1;
                    new_line_num += 1;
                    (DiffLineType::Context, Some(old_num), Some(new_num))
                }
            };

            lines.push(DiffLine {
                line_type,
                content: change.to_string(),
                old_line_number: old_num,
                new_line_number: new_num,
            });
        }

        lines
    }

    pub fn format_unified(&self, context_lines: usize) -> String {
        let mut output = String::new();

        output.push_str(&format!("--- {}\n", self.path));
        output.push_str(&format!("+++ {}\n", self.path));

        let mut in_hunk = false;
        let mut hunk_start = 0;
        let mut hunk_lines = Vec::new();

        for (i, line) in self.diff_lines.iter().enumerate() {
            if line.line_type != DiffLineType::Context || in_hunk {
                if !in_hunk {
                    in_hunk = true;
                    hunk_start = i.saturating_sub(context_lines);
                }

                let prefix = match line.line_type {
                    DiffLineType::Addition => "+",
                    DiffLineType::Deletion => "-",
                    DiffLineType::Context => " ",
                };

                hunk_lines.push(format!("{}{}", prefix, line.content));

                // Check if we should close the hunk
                if i + context_lines >= self.diff_lines.len() - 1 {
                    if !hunk_lines.is_empty() {
                        output.push_str(&format!(
                            "@@ -{},{} +{},{} @@\n",
                            self.diff_lines[hunk_start].old_line_number.unwrap_or(0),
                            hunk_lines.len(),
                            self.diff_lines[hunk_start].new_line_number.unwrap_or(0),
                            hunk_lines.len()
                        ));
                        output.push_str(&hunk_lines.join(""));
                        hunk_lines.clear();
                    }
                    in_hunk = false;
                }
            }
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ChangeType;
    use std::path::PathBuf;
    use uuid::Uuid;

    #[test]
    fn test_diff_computation() {
        let old_text = "line 1\nline 2\nline 3\n";
        let new_text = "line 1\nline 2 modified\nline 3\nline 4\n";

        let diff_lines = FileDiff::compute_diff(old_text, new_text);

        assert!(!diff_lines.is_empty());
        assert!(diff_lines
            .iter()
            .any(|l| l.line_type == DiffLineType::Addition));
        assert!(diff_lines
            .iter()
            .any(|l| l.line_type == DiffLineType::Deletion));
    }

    #[test]
    fn test_file_diff_from_change() {
        let session_id = Uuid::new_v4();
        let change = Change::new(ChangeType::Modify, PathBuf::from("test.txt"), session_id)
            .with_content_before(b"Hello\nWorld\n".to_vec())
            .with_content_after(b"Hello\nRust\nWorld\n".to_vec());

        let file_diff = FileDiff::from_change(&change).unwrap();

        assert_eq!(file_diff.path, "test.txt");
        assert!(file_diff.old_content.is_some());
        assert!(file_diff.new_content.is_some());
        assert!(!file_diff.diff_lines.is_empty());
    }
}
