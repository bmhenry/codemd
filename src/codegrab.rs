//!
//! Find code blocks in the markdown file
//!

use color_eyre::Result;
use regex::Regex;

use crate::codechunk::*;


pub fn find_codeblocks(lines: &Vec<String>) -> Result<Vec<CodeChunk>> {
    let mut chunks = vec!();

    // regex for opening code block
    let opener = Regex::new(r#"^ *```[ \t]*(\w+)[ \t]*(\{[\w \t='":,.\-_{}]*\})"#).unwrap();
    let closer = Regex::new(r"^ *```").unwrap();

    let mut block: Option<CodeChunk> = None;

    for line in lines {
        if block.is_some() {
            if closer.is_match(line) {
                chunks.push(block.unwrap());
                block = None;
            } else if let Some(b) = block.as_mut() {
                b.code.push(line.to_string());
            }
        } else if let Some(caps) = opener.captures(line) {
            block = Some(CodeChunk::new(
                caps.get(1).unwrap().as_str(),
                CodeMeta::from_json(caps.get(2).unwrap().as_str())?
            ));
        }
        
    }

    Ok(chunks)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_one_cb() {
        let code = "```rust { \"diff\": { \"first\": 10, \"last\": 20 } }\n\
                    some code\n\
                    some more code\n\
                    ```";
        let lines = code.lines().map(str::to_string).collect::<Vec<String>>();
        let blocks_r = find_codeblocks(&lines);
        assert!(blocks_r.is_ok());
        let blocks = blocks_r.unwrap();
        assert!(blocks.len() == 1);
        assert!(blocks[0].lang == "rust");
        assert!(match &blocks[0].meta {
            CodeMeta::Diff(d) => d.first == 10 && d.last == 20,
            _ => false
        });
    }
}

