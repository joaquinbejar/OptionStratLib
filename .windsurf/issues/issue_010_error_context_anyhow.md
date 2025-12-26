# Issue #10: Improve error context with anyhow

## Title
`refactor: Add error context using anyhow at API boundaries`

## Labels
- `refactor`
- `error-handling`
- `priority-low`

## Description

While the project uses `thiserror` for typed errors, adding `anyhow::Context` at API boundaries would improve error messages for users.

### Current State
- Errors use `thiserror` for type safety
- Error messages may lack context about what operation failed
- Users may struggle to debug issues

### Target State
- Error messages include context about what operation failed
- Existing error types are preserved
- Better debugging experience for library users

## Tasks

- [ ] Add `anyhow` as a dependency in `Cargo.toml`
- [ ] Identify public API boundaries where context would help
- [ ] Add `.context()` calls to provide meaningful error messages
- [ ] Update error documentation
- [ ] Run `make lint-fix` and `make pre-push` to verify

## Acceptance Criteria

- [ ] Error messages include context about what operation failed
- [ ] Existing error types are preserved
- [ ] No breaking changes to public API
- [ ] Documentation updated with error handling examples

## Technical Notes

### Adding Context Pattern

```rust
use anyhow::{Context, Result};

// Before
pub fn load_chain(path: &str) -> Result<OptionChain, ChainError> {
    let data = std::fs::read_to_string(path)?;
    serde_json::from_str(&data)?
}

// After
pub fn load_chain(path: &str) -> Result<OptionChain, anyhow::Error> {
    let data = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read option chain from '{}'", path))?;
    
    serde_json::from_str(&data)
        .with_context(|| format!("Failed to parse option chain JSON from '{}'", path))
}
```

### Where to Add Context
- File I/O operations
- Network requests (if any)
- Complex calculations that can fail
- Strategy construction
- Chain loading/parsing

### Files to Update
- `Cargo.toml` (add dependency)
- Public API functions across modules
- Error documentation

## Estimated Effort

**Low (2-3 hours)**

## Dependencies

None

## Related Issues

- Issue #1-#3: Error handling improvements
