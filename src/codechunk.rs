//!
//! Pick code chunks out of a file using an extension on the markdown syntax
//!
//! Example:
//! 
//! '''
//! ```rust { file = 'filename', mode = 'diff' }
//! ...
//! ```
//! '''
//!

#![allow(unused)]

use color_eyre::eyre::Result;
use serde::Deserialize;



#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum CodeMeta {
    /// Add the chunk as a diff, replacing the given set of lines with the lines in the chunk
    Diff(CodeDiff),
    /// Insert the chunk into the existing file starting on the given line
    Insert(CodeInsert),
    // This one NEEDS to go at the end, so that serde attempts to deserialize Diff and Insert
    // first. Append has no unique attributes, so it'll always be picked
    /// Simply add this chunk to the end of the file
    Append(CodeAppend),
}

impl CodeMeta {
    pub fn from_json(j: &str) -> Result<CodeMeta> {
        let cm: CodeMeta = serde_json::from_str(j)?;
        Ok(cm)
    }
}

#[derive(Debug, Deserialize)]
pub struct CodeAppend {
    /// A filename, allowing multiple code files to be modified within the same markdown doc
    pub file: Option<String>,
    /// Allow some additional code sections to be removed, which won't be replaced with this code
    #[serde(default)]
    pub removals: Vec<CodeRemove>,
}

#[derive(Debug, Deserialize)]
pub struct CodeInsert {
    /// A filename, allowing multiple code files to be modified within the same markdown doc
    pub file: Option<String>,
    /// The line at which this code chunk will be inserted, bumping any code currently on
    /// that line down
    pub line: usize,
    /// Allow some additional code sections to be removed, which won't be replaced with this code
    #[serde(default)]
    pub removals: Vec<CodeRemove>,
}

#[derive(Debug, Deserialize)]
pub struct CodeDiff {
    /// A filename, allowing multiple code files to be modified within the same markdown doc
    pub file: Option<String>,
    /// First line (inclusive) to replace with the code chunk
    pub first: usize,
    /// Last line (inclusive) to replace with the code chunk
    pub last: usize,
    /// Allow some additional code sections to be removed, which won't be replaced with this code
    #[serde(default)]
    pub removals: Vec<CodeRemove>,
}

/// Allows simply deleting code without requiring a chunk to replace it with
#[derive(Debug, Deserialize)]
pub struct CodeRemove {
    // /// A filename, allowing multiple code files to be modified within the same markdown doc
    // file: Option<String>,
    /// First line (inclusive) to remove
    pub first: usize,
    /// Last line (inclusive) to remove
    pub last: usize,
}

#[derive(Deserialize)]
pub enum BlockMeta {
    /// Metadata may be provided as a list
    List(Vec<CodeMeta>),
    Single(CodeMeta)    
}

/// Represents a single chunk of code, including language & other metadata
/// from the opening code block
#[derive(Debug)]
pub struct CodeChunk {
    pub file: Option<String>,
    pub lang: String,
    pub meta: CodeMeta,
    pub code: Vec<String>
}

impl CodeChunk {
    pub fn new(lang: &str, meta: CodeMeta) -> CodeChunk {
        CodeChunk {
            lang: lang.to_string(),
            code: vec!(),
            file: match &meta {
                CodeMeta::Diff(d) => d.file.clone(),
                CodeMeta::Insert(i) => i.file.clone(),
                CodeMeta::Append(a) => a.file.clone(),
            },
            meta,
        }
        
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deser_remove() {
        let rem_str = "{ \"first\": 10, \"last\": 20 }";
        let rem: CodeRemove = serde_json::from_str(rem_str).unwrap();

        assert!(rem.first == 10 && rem.last == 20);
    }

    #[test]
    fn deser_diff() {
        let diff_meta = "{ \"first\": 10, \"last\": 20, \"removals\": [{ \"first\": 24, \"last\": 27 }, { \"first\": 30, \"last\": 31 }] }";
        let d: CodeDiff = serde_json::from_str(diff_meta).unwrap();
        assert!(d.first == 10 && d.removals[0].first == 24 && d.removals[1].last == 31);

        let cm: CodeMeta = serde_json::from_str(diff_meta).unwrap();
        assert!(match cm {
            CodeMeta::Diff(d) => d.first == 10 && d.removals[0].first == 24 && d.removals[1].last == 31,
            _ => false
        });
    }
}


