# Add line number validation before operations

## Summary
Validate that line numbers referenced in `insert`, `diff`, and `removals` operations exist in the target file before attempting to modify it.

## Motivation
Currently, if a code block specifies an invalid line number (e.g., `"line": 100` when the file only has 50 lines), the operation either:
- Panics at runtime with an unclear error
- Silently produces incorrect output
- Causes index out-of-bounds errors

This makes it difficult to diagnose issues in documentation, especially when line numbers drift as files are modified.

## Proposed Behavior

### Validation Checks
1. **Insert operations**: Verify `line` is within valid range (0 to current_lines)
2. **Diff operations**: Verify `first` and `last` exist, and `first <= last`
3. **Removal operations**: Verify `first` and `last` exist, and `first <= last`

### Error Messages
Provide clear, actionable error messages:
```
Error: Invalid line number in diff operation
  File: main.rs (currently 45 lines)
  Requested: lines 50-55
  Hint: Line numbers are 0-indexed
```

### Validation Modes
Consider two modes:
- **Strict (default)**: Error on invalid line numbers
- **Lenient (`--lenient`)**: Warn but continue, clamping to valid ranges

## Implementation Notes
- Add validation in `CodeFile::add()` before performing operations
- Include current file line count in error messages
- Consider adding a `--strict` / `--lenient` flag for user control

## Example
```markdown
```rust { "diff": { "file": "main.rs", "first": 100, "last": 105 } }
// This should fail with clear error if main.rs has < 100 lines
```
```
