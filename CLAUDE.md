# CLAUDE.md - AI Assistant Guide for CodeMD

## Project Overview

CodeMD is a Rust CLI tool that extracts and assembles code blocks from Markdown documentation. It enables testing documentation code by pulling annotated code blocks and concatenating them into compilable files.

**Primary Use Cases:**
- Extract code examples from tutorial-style documentation
- Test that documentation code actually compiles/runs
- Support CI pipelines that verify documentation accuracy
- Language-agnostic code extraction (works with any programming language)

## Quick Reference

```bash
# Build
cargo build                    # Development build
cargo build --release          # Release build

# Test
cargo test                     # Run all tests (4 unit tests)

# Run
cargo run -- -f <file.md>              # Extract to current directory
cargo run -- -f <file.md> -o <outdir>  # Extract to specified directory

# Example
cargo run -- -f examples/example_cpp.md -o examples/
```

## Architecture

```
src/
├── main.rs        # CLI entry point, argument parsing (argh)
├── codegrab.rs    # Markdown parsing with regex
├── codechunk.rs   # Data structures for code metadata (serde)
└── codefile.rs    # File writing and code manipulation
```

### Module Responsibilities

| Module | Purpose | Key Types |
|--------|---------|-----------|
| `main.rs` | CLI args, orchestration | `CodeMdArgs` |
| `codegrab.rs` | Parse markdown, extract blocks | `find_codeblocks()` |
| `codechunk.rs` | Metadata structures | `CodeMeta`, `CodeChunk`, `CodeDiff`, `CodeInsert`, `CodeAppend` |
| `codefile.rs` | Build and write output files | `CodeFile` |

### Data Flow

1. `main.rs` reads markdown file into lines
2. `codegrab::find_codeblocks()` parses lines, extracts `CodeChunk`s
3. `CodeFile::from_blocks()` groups chunks by filename, applies operations
4. `CodeFile::write_to_file()` writes assembled code to disk

## Key Concepts

### CodeMeta Enum (codechunk.rs:21)

Three operation types for code blocks:

- **`Append`** - Add code to end of file
- **`Insert`** - Insert code at specific line number
- **`Diff`** - Replace a range of lines with new code

All operations support optional `removals` for deleting additional lines.

### Markdown Syntax

Code blocks use JSON metadata after the language identifier:

```
```lang { "command": { "key": "value" } }
code here
```
```

**Examples:**
```
```cpp { "append": { "file": "main.cpp" } }
```cpp { "insert": { "file": "main.cpp", "line": 5 } }
```cpp { "diff": { "file": "main.cpp", "first": 10, "last": 15 } }
```

### Line Indexing

Lines are **0-indexed** throughout the codebase.

## Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `argh` | 0.1.12 | CLI argument parsing |
| `color-eyre` | 0.6.3 | Error handling with colored output |
| `regex` | 1.10.6 | Markdown code block pattern matching |
| `serde` | 1.0 | Serialization/deserialization |
| `serde_json` | 1.0 | JSON parsing for metadata |

## Code Conventions

### Error Handling
- Use `color_eyre::eyre::Result<T>` for all fallible functions
- Use `eyre!()` macro for custom error messages
- Propagate errors with `?` operator

### Serde Patterns
- Use `#[serde(alias = "...")]` for case-flexible JSON keys
- Use `#[serde(default)]` for optional Vec fields

### Testing
- Unit tests are inline with `#[cfg(test)]` modules
- Test names describe the operation being tested
- Tests exist in: `codechunk.rs`, `codefile.rs`, `codegrab.rs`

### Style
- Module-level doc comments with `//!`
- Use `#[allow(unused)]` or `#[allow(dead_code)]` for intentionally unused items
- Prefer `Vec<String>` for line-based text manipulation

## Testing

Run all tests:
```bash
cargo test
```

Current test coverage:
- `codechunk::tests::deser_remove` - CodeRemove deserialization
- `codechunk::tests::deser_diff` - CodeDiff and CodeMeta deserialization
- `codefile::tests::append_codefile` - CodeFile append operation
- `codegrab::tests::get_one_cb` - Code block parsing from markdown

## Important Implementation Details

### Regex Pattern (codegrab.rs:15)
```rust
r#"^ *```[ \t]*(\w+)[ \t]*(\{[\w \t='":,.\-_{}]*\})"#
```
Captures: (1) language identifier, (2) JSON metadata block

### File Operations (codefile.rs)
- Uses `HashMap<Option<String>, CodeFile>` to group blocks by filename
- `None` filename uses `default.out` as output
- Removals are processed **after** each operation completes

### Removal Bug Warning
The `perform_removals()` function at `codefile.rs:46` has a potential issue: it removes lines in a loop without adjusting indices, which can cause incorrect behavior when multiple removals exist. Consider this when modifying removal logic.

## Common Tasks

### Adding a New Operation Type
1. Add variant to `CodeMeta` enum in `codechunk.rs`
2. Create corresponding struct (like `CodeDiff`, `CodeInsert`)
3. Update `CodeChunk::new()` to extract filename from new variant
4. Add handling in `CodeFile::add()` match statement
5. Add tests for deserialization and file operations

### Modifying the Regex
The code block regex in `codegrab.rs:15` must:
- Allow leading whitespace before ```
- Capture the language identifier
- Capture the JSON metadata block
- Handle various whitespace patterns

### Adding CLI Options
Use `argh` derive macros in `main.rs`:
```rust
#[argh(option, short = 'x')]
new_option: Option<String>,
```

## File Locations

| What | Where |
|------|-------|
| CLI args struct | `src/main.rs:14` |
| Code block regex | `src/codegrab.rs:15` |
| Operation enums | `src/codechunk.rs:20-31` |
| File writing | `src/codefile.rs:84` |
| Example markdown | `examples/example_cpp.md` |
