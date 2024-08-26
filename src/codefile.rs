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
                println!("Creating new file: {:?}", block.file);
                cfs.insert(block.file.clone(), CodeFile::new(block));
            }
        }

        Ok(cfs.into_values().collect())
    }

    pub fn perform_removals(&mut self, removals: &Vec<CodeRemove>) {
        for r in removals {
            for rdx in r.first..=r.last {
                self.lines.remove(rdx);
            }
        }
    }

    pub fn add(&mut self, chunk: &CodeChunk) -> Result<()> {
        if chunk.file != self.name {
            return Err(eyre!("Filename doesn't match"));
        }

        match &chunk.meta {
            CodeMeta::Append(a) => {
                self.lines.append(&mut chunk.code.clone());
                self.perform_removals(&a.removals);
            },
            CodeMeta::Insert(i) => {
                for (idx, code) in std::iter::zip(i.line..i.line + chunk.code.len(), &chunk.code) {
                    self.lines.insert(idx, code.to_string());
                }
                self.perform_removals(&i.removals);
            },
            CodeMeta::Diff(d) => {
                for _ in d.first..=d.last {
                    self.lines.remove(d.first);
                }
                for (idx, code) in std::iter::zip(d.first..d.first + chunk.code.len(), &chunk.code) {
                    self.lines.insert(idx, code.to_string());
                }
                self.perform_removals(&d.removals);
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

        println!("{:#?}", cf);

        assert!(cf.lines.len() == 4);
    }
}

