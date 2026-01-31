# Add --verbose flag for detailed output

## Summary
Add a `--verbose` (or `-v`) flag to provide detailed output during code extraction, helping users debug issues and understand what operations are being performed.

## Motivation
Currently, CodeMD runs silently and only outputs errors. When processing complex markdown files with multiple code blocks, users have no visibility into:
- Which code blocks are being processed
- What operations (append/insert/diff) are being applied
- Which files are being written
- Line number changes from removals

This makes debugging failures difficult, especially in CI pipelines.

## Proposed Behavior
When `--verbose` is enabled, output should include:
- Each code block found (language, operation type, target file)
- Line operations being performed (insertions, replacements, removals)
- Final file write confirmations with line counts
- Summary statistics (blocks processed, files written)

## Example Output
```
$ codemd -f docs/tutorial.md -o build/ --verbose
[1/3] Found rust block → append to main.rs
[2/3] Found rust block → insert at line 5 in main.rs
[3/3] Found rust block → diff lines 10-15 in main.rs
      Removing lines 20-22
Writing main.rs (45 lines)
Done: 3 blocks processed, 1 file written
```

## Implementation Notes
- Add `--verbose` / `-v` flag to `CodeMdArgs` in `main.rs`
- Pass verbose flag through to processing functions
- Consider using a simple println! approach or a logging crate
