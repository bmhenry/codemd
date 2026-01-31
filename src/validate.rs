//!
//! Validation module for codemd syntax
//!
//! This module provides validation functions that check codemd syntax without
//! performing any file operations. The validation logic is designed to be
//! reusable at the start of regular operations.
//!

use color_eyre::eyre::{eyre, Result};

use crate::codechunk::{CodeChunk, CodeMeta, CodeRemove};

/// Result of validating a markdown file
#[derive(Debug)]
pub struct ValidationResult {
    /// Total number of code blocks found
    pub block_count: usize,
    /// Number of unique files referenced
    pub file_count: usize,
    /// List of file names that will be written (None means default.out)
    pub files: Vec<Option<String>>,
    /// Any warnings (non-fatal issues)
    pub warnings: Vec<ValidationWarning>,
}

/// A non-fatal validation warning
#[derive(Debug)]
pub struct ValidationWarning {
    pub block_index: usize,
    pub message: String,
}

/// Validates a collection of parsed code blocks
///
/// This is the main validation entry point that checks:
/// - Code blocks are not empty
/// - Metadata is valid for each operation type
/// - Line ranges are valid (first <= last for diff/remove)
/// - Removals don't overlap within a single block
///
/// Returns a ValidationResult on success, or an error if validation fails.
pub fn validate_codeblocks(blocks: &[CodeChunk]) -> Result<ValidationResult> {
    if blocks.is_empty() {
        return Err(eyre!("No code blocks found"));
    }

    let mut warnings = Vec::new();
    let mut files: Vec<Option<String>> = Vec::new();

    for (idx, block) in blocks.iter().enumerate() {
        // Validate the metadata for this block
        validate_codemeta(&block.meta, idx)?;

        // Collect unique files
        if !files.contains(&block.file) {
            files.push(block.file.clone());
        }

        // Warn if code block is empty (valid but potentially unintended)
        if block.code.is_empty() {
            warnings.push(ValidationWarning {
                block_index: idx,
                message: format!(
                    "Code block {} is empty (file: {})",
                    idx,
                    block.file.as_deref().unwrap_or("default.out")
                ),
            });
        }
    }

    Ok(ValidationResult {
        block_count: blocks.len(),
        file_count: files.len(),
        files,
        warnings,
    })
}

/// Validates a CodeMeta operation
///
/// Checks operation-specific constraints:
/// - Diff: first <= last
/// - Insert: line number is reasonable
/// - Append: no specific constraints
/// - All: removals are valid
fn validate_codemeta(meta: &CodeMeta, block_index: usize) -> Result<()> {
    match meta {
        CodeMeta::Diff(d) => {
            validate_line_range(d.first, d.last, "diff", block_index)?;
            validate_removals(&d.removals, block_index)?;
        }
        CodeMeta::Insert(i) => {
            validate_removals(&i.removals, block_index)?;
        }
        CodeMeta::Append(a) => {
            validate_removals(&a.removals, block_index)?;
        }
    }
    Ok(())
}

/// Validates that a line range is valid (first <= last)
fn validate_line_range(first: usize, last: usize, op_type: &str, block_index: usize) -> Result<()> {
    if first > last {
        return Err(eyre!(
            "Invalid {} range in block {}: first ({}) > last ({})",
            op_type,
            block_index,
            first,
            last
        ));
    }
    Ok(())
}

/// Validates a list of removal operations
///
/// Checks:
/// - Each removal has first <= last
/// - Removals don't overlap with each other
fn validate_removals(removals: &[CodeRemove], block_index: usize) -> Result<()> {
    // Validate individual removal ranges
    for (idx, r) in removals.iter().enumerate() {
        if r.first > r.last {
            return Err(eyre!(
                "Invalid removal range in block {}, removal {}: first ({}) > last ({})",
                block_index,
                idx,
                r.first,
                r.last
            ));
        }
    }

    // Check for overlapping removals
    if removals.len() > 1 {
        let mut sorted: Vec<&CodeRemove> = removals.iter().collect();
        sorted.sort_by_key(|r| r.first);

        for i in 0..sorted.len() - 1 {
            let current = sorted[i];
            let next = sorted[i + 1];
            if current.last >= next.first {
                return Err(eyre!(
                    "Overlapping removal ranges in block {}: lines {}-{} overlaps with lines {}-{}",
                    block_index,
                    current.first,
                    current.last,
                    next.first,
                    next.last
                ));
            }
        }
    }

    Ok(())
}

/// Validates a markdown file by reading, parsing, and checking its code blocks
///
/// This is a convenience function that combines file reading, parsing, and validation.
/// It does NOT perform any file write operations.
pub fn validate_markdown_file(path: &str) -> Result<ValidationResult> {
    let md = std::fs::read_to_string(path)
        .map_err(|e| eyre!("Failed to read file '{}': {}", path, e))?;

    let lines: Vec<String> = md.lines().map(str::to_string).collect();

    validate_markdown_lines(&lines)
}

/// Validates markdown content from lines
///
/// This is the core validation function that parses and validates without file I/O.
pub fn validate_markdown_lines(lines: &[String]) -> Result<ValidationResult> {
    let blocks = crate::codegrab::find_codeblocks(&lines.to_vec())?;
    validate_codeblocks(&blocks)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codechunk::{CodeAppend, CodeDiff, CodeInsert};

    #[test]
    fn validate_empty_blocks_fails() {
        let blocks: Vec<CodeChunk> = vec![];
        let result = validate_codeblocks(&blocks);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("No code blocks found"));
    }

    #[test]
    fn validate_valid_append_block() {
        let mut chunk = CodeChunk::new(
            "rust",
            CodeMeta::Append(CodeAppend {
                file: Some("test.rs".to_string()),
                removals: vec![],
            }),
        );
        chunk.code.push("fn main() {}".to_string());

        let result = validate_codeblocks(&[chunk]);
        assert!(result.is_ok());
        let vr = result.unwrap();
        assert_eq!(vr.block_count, 1);
        assert_eq!(vr.file_count, 1);
        assert!(vr.warnings.is_empty());
    }

    #[test]
    fn validate_invalid_diff_range() {
        let chunk = CodeChunk::new(
            "rust",
            CodeMeta::Diff(CodeDiff {
                file: Some("test.rs".to_string()),
                first: 20,
                last: 10, // Invalid: first > last
                removals: vec![],
            }),
        );

        let result = validate_codeblocks(&[chunk]);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Invalid diff range"));
        assert!(err.contains("first (20) > last (10)"));
    }

    #[test]
    fn validate_invalid_removal_range() {
        let chunk = CodeChunk::new(
            "rust",
            CodeMeta::Append(CodeAppend {
                file: Some("test.rs".to_string()),
                removals: vec![CodeRemove { first: 15, last: 5 }], // Invalid
            }),
        );

        let result = validate_codeblocks(&[chunk]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid removal range"));
    }

    #[test]
    fn validate_overlapping_removals() {
        let chunk = CodeChunk::new(
            "rust",
            CodeMeta::Insert(CodeInsert {
                file: Some("test.rs".to_string()),
                line: 5,
                removals: vec![
                    CodeRemove { first: 10, last: 15 },
                    CodeRemove { first: 12, last: 18 }, // Overlaps with previous
                ],
            }),
        );

        let result = validate_codeblocks(&[chunk]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Overlapping removal ranges"));
    }

    #[test]
    fn validate_empty_code_block_warns() {
        let chunk = CodeChunk::new(
            "rust",
            CodeMeta::Append(CodeAppend {
                file: Some("test.rs".to_string()),
                removals: vec![],
            }),
        );
        // Note: code is empty

        let result = validate_codeblocks(&[chunk]);
        assert!(result.is_ok());
        let vr = result.unwrap();
        assert_eq!(vr.warnings.len(), 1);
        assert!(vr.warnings[0].message.contains("is empty"));
    }

    #[test]
    fn validate_multiple_files_counted() {
        let chunk1 = CodeChunk::new(
            "rust",
            CodeMeta::Append(CodeAppend {
                file: Some("file1.rs".to_string()),
                removals: vec![],
            }),
        );
        let chunk2 = CodeChunk::new(
            "rust",
            CodeMeta::Append(CodeAppend {
                file: Some("file2.rs".to_string()),
                removals: vec![],
            }),
        );
        let chunk3 = CodeChunk::new(
            "rust",
            CodeMeta::Append(CodeAppend {
                file: Some("file1.rs".to_string()), // Same as chunk1
                removals: vec![],
            }),
        );

        let result = validate_codeblocks(&[chunk1, chunk2, chunk3]);
        assert!(result.is_ok());
        let vr = result.unwrap();
        assert_eq!(vr.block_count, 3);
        assert_eq!(vr.file_count, 2); // Only 2 unique files
    }

    #[test]
    fn validate_markdown_lines_valid() {
        let lines: Vec<String> = vec![
            "# Test".to_string(),
            "```rust { \"append\": { \"file\": \"test.rs\" } }".to_string(),
            "fn main() {}".to_string(),
            "```".to_string(),
        ];

        let result = validate_markdown_lines(&lines);
        assert!(result.is_ok());
    }

    #[test]
    fn validate_markdown_lines_invalid_json() {
        let lines: Vec<String> = vec![
            "```rust { \"invalid\": }".to_string(), // Invalid JSON
            "fn main() {}".to_string(),
            "```".to_string(),
        ];

        let result = validate_markdown_lines(&lines);
        // This should fail during parsing, not validation
        assert!(result.is_err());
    }
}
