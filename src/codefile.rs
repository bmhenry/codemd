//!
//! Push code chunks into a file according to their mode of operation
//!

use std::collections::HashMap;
use std::io::Write;

use color_eyre::eyre::{eyre, Result};

use crate::codechunk::*;



#[derive(Debug)]
pub struct CodeFile {
    name: Option<String>,
    lines: Vec<String>,
}

impl CodeFile {
    pub fn new(chunk: &CodeChunk) -> CodeFile {
        CodeFile {
            name: chunk.file.clone(),
            lines: chunk.code.clone()
        }
    }

    pub fn from_blocks(blocks: &Vec<CodeChunk>) -> Result<Vec<CodeFile>> {
        if blocks.is_empty() {
            return Err(eyre!("No code blocks found"));
        }

        let mut cfs = HashMap::<Option<String>, CodeFile>::new();

        for block in blocks {
            if cfs.contains_key(&block.file) {
                cfs.get_mut(&block.file).unwrap().add(block)?;
            } else {
                cfs.insert(block.file.clone(), CodeFile::new(block));
            }
        }

        Ok(cfs.into_values().collect())
    }

    /// Perform line removals, processing from highest to lowest line numbers
    /// to avoid index shifting issues. Detects and errors on overlapping ranges.
    pub fn perform_removals(&mut self, removals: &Vec<CodeRemove>) -> Result<()> {
        if removals.is_empty() {
            return Ok(());
        }

        // Sort removals by first line number (ascending) for overlap detection
        let mut sorted_removals: Vec<&CodeRemove> = removals.iter().collect();
        sorted_removals.sort_by_key(|r| r.first);

        // Check for overlapping ranges
        for i in 0..sorted_removals.len() - 1 {
            let current = sorted_removals[i];
            let next = sorted_removals[i + 1];
            if current.last >= next.first {
                return Err(eyre!(
                    "Overlapping removal ranges detected: lines {}-{} overlaps with lines {}-{}",
                    current.first, current.last, next.first, next.last
                ));
            }
        }

        // Process removals in reverse order (highest line numbers first)
        // to avoid index shifting issues
        for r in sorted_removals.into_iter().rev() {
            // Remove lines from last to first within each range
            for rdx in (r.first..=r.last).rev() {
                if rdx < self.lines.len() {
                    self.lines.remove(rdx);
                }
            }
        }

        Ok(())
    }

    pub fn add(&mut self, chunk: &CodeChunk) -> Result<()> {
        if chunk.file != self.name {
            return Err(eyre!("Filename doesn't match"));
        }

        match &chunk.meta {
            CodeMeta::Append(a) => {
                self.lines.append(&mut chunk.code.clone());
                self.perform_removals(&a.removals)?;
            },
            CodeMeta::Insert(i) => {
                for (idx, code) in std::iter::zip(i.line..i.line + chunk.code.len(), &chunk.code) {
                    self.lines.insert(idx, code.to_string());
                }
                self.perform_removals(&i.removals)?;
            },
            CodeMeta::Diff(d) => {
                for _ in d.first..=d.last {
                    self.lines.remove(d.first);
                }
                for (idx, code) in std::iter::zip(d.first..d.first + chunk.code.len(), &chunk.code) {
                    self.lines.insert(idx, code.to_string());
                }
                self.perform_removals(&d.removals)?;
            },
        }

        Ok(())
    }

    pub fn write_to_file(&self, output_dir: &str, default_filename: &str) -> Result<()> {
        let filename = match &self.name {
            Some(n) => n,
            None => default_filename
        };
        let path = format!("{}/{}", output_dir, filename);

        let mut f = std::fs::File::create(path)?;
        for line in &self.lines {
            f.write_all(line.as_bytes())?;
            f.write_all(b"\n")?;
        }

        Ok(())
    }

    #[allow(dead_code)]
    pub fn name(&self) -> &Option<String> {
        &self.name
    }

    #[allow(dead_code)]
    pub fn lines(&self) -> &Vec<String> {
        &self.lines
    }
    
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn append_codefile() {
        let mut cb0 = CodeChunk::new("rust", CodeMeta::Append(CodeAppend{ file: None, removals: vec!()}));
        cb0.code.push("line 0".to_string());
        cb0.code.push("line 1".to_string());

        let mut cb1 = CodeChunk::new("rust", CodeMeta::Append(CodeAppend{ file: None, removals: vec!()}));
        cb1.code.push("line 2".to_string());
        cb1.code.push("line 3".to_string());

        let mut cf = CodeFile::new(&cb0);
        cf.add(&cb1).unwrap();

        //println!("{:#?}", cf);

        assert!(cf.lines.len() == 4);
    }

    #[test]
    fn removals_sorted_correctly() {
        // Create a file with 10 lines
        let mut cb = CodeChunk::new("rust", CodeMeta::Append(CodeAppend{ file: None, removals: vec!()}));
        for i in 0..10 {
            cb.code.push(format!("line {}", i));
        }
        let mut cf = CodeFile::new(&cb);

        // Remove lines out of order: remove lines 7-8, then lines 2-3
        // If not sorted and processed in reverse, this would fail
        let removals = vec![
            CodeRemove { first: 7, last: 8 },
            CodeRemove { first: 2, last: 3 },
        ];
        let result = cf.perform_removals(&removals);
        assert!(result.is_ok());

        // Should have 6 lines left: 0, 1, 4, 5, 6, 9
        assert_eq!(cf.lines.len(), 6);
        assert_eq!(cf.lines[0], "line 0");
        assert_eq!(cf.lines[1], "line 1");
        assert_eq!(cf.lines[2], "line 4");
        assert_eq!(cf.lines[3], "line 5");
        assert_eq!(cf.lines[4], "line 6");
        assert_eq!(cf.lines[5], "line 9");
    }

    #[test]
    fn removals_multiple_ranges_reverse_order() {
        // Test removals specified in reverse order (high to low)
        let mut cb = CodeChunk::new("rust", CodeMeta::Append(CodeAppend{ file: None, removals: vec!()}));
        for i in 0..10 {
            cb.code.push(format!("line {}", i));
        }
        let mut cf = CodeFile::new(&cb);

        // Removals in reverse order
        let removals = vec![
            CodeRemove { first: 1, last: 2 },
            CodeRemove { first: 5, last: 6 },
        ];
        let result = cf.perform_removals(&removals);
        assert!(result.is_ok());

        // Should have 6 lines left: 0, 3, 4, 7, 8, 9
        assert_eq!(cf.lines.len(), 6);
        assert_eq!(cf.lines[0], "line 0");
        assert_eq!(cf.lines[1], "line 3");
        assert_eq!(cf.lines[2], "line 4");
        assert_eq!(cf.lines[3], "line 7");
    }

    #[test]
    fn overlapping_removals_error() {
        let mut cb = CodeChunk::new("rust", CodeMeta::Append(CodeAppend{ file: None, removals: vec!()}));
        for i in 0..10 {
            cb.code.push(format!("line {}", i));
        }
        let mut cf = CodeFile::new(&cb);

        // Overlapping ranges: 2-5 and 4-7 overlap at lines 4-5
        let removals = vec![
            CodeRemove { first: 2, last: 5 },
            CodeRemove { first: 4, last: 7 },
        ];
        let result = cf.perform_removals(&removals);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Overlapping removal ranges"));
    }

    #[test]
    fn adjacent_removals_ok() {
        // Adjacent but non-overlapping ranges should work
        let mut cb = CodeChunk::new("rust", CodeMeta::Append(CodeAppend{ file: None, removals: vec!()}));
        for i in 0..10 {
            cb.code.push(format!("line {}", i));
        }
        let mut cf = CodeFile::new(&cb);

        // Adjacent ranges: 2-3 and 4-5 (line 3 ends, line 4 starts)
        let removals = vec![
            CodeRemove { first: 2, last: 3 },
            CodeRemove { first: 4, last: 5 },
        ];
        let result = cf.perform_removals(&removals);
        assert!(result.is_ok());
        assert_eq!(cf.lines.len(), 6);
    }
}

