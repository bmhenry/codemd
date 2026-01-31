# Add `codemd validate` subcommand

## Summary
Add a `validate` subcommand that checks markdown files for valid CodeMD syntax without extracting or writing any files.

## Motivation
Documentation authors need a way to verify their CodeMD annotations are correct before running extraction. This is useful for:
- **Pre-commit hooks**: Catch syntax errors before committing
- **CI pipelines**: Fast validation step before full extraction
- **Editor integration**: Quick feedback during authoring
- **Debugging**: Isolate parsing issues from file operation issues

## Proposed CLI Interface
```bash
# Validate a single file
codemd validate -f docs/tutorial.md

# Validate multiple files
codemd validate -f docs/part1.md -f docs/part2.md

# Validate all markdown in a directory (future enhancement)
codemd validate -d docs/
```

## Proposed Output

### Success
```
$ codemd validate -f docs/tutorial.md
✓ docs/tutorial.md: 5 code blocks valid
  - main.rs: 3 blocks (2 append, 1 diff)
  - utils.rs: 2 blocks (2 append)
```

### Failure
```
$ codemd validate -f docs/tutorial.md
✗ docs/tutorial.md: 2 errors found

  Line 45: Invalid JSON in code block metadata
    ```rust { "append": { file: "main.rs" } }
                         ^^^^
    Expected quoted string for key

  Line 89: Unknown operation type
    ```rust { "replace": { "file": "main.rs" } }
              ^^^^^^^^^
    Valid operations: append, insert, diff
```

## Validation Checks
1. **JSON syntax**: Valid JSON in metadata block
2. **Required fields**: Operation has required fields (e.g., `first`/`last` for diff)
3. **Operation type**: Must be `append`, `insert`, or `diff`
4. **Field types**: Correct types (numbers for line fields, strings for file)
5. **Logical consistency**: `first <= last` for ranges

## Exit Codes
- `0`: All files valid
- `1`: Validation errors found
- `2`: File not found or read error

## Implementation Notes
- Add subcommand support using `argh` subcommands
- Reuse `find_codeblocks()` from `codegrab.rs`
- Add structured error collection for reporting
- Consider `--json` flag for machine-readable output
