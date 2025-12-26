# Issue #15: Reduce `.expect()` usage in production code

## Title
`refactor: Replace expect() calls with proper error handling`

## Labels
- `refactor`
- `error-handling`
- `priority-medium`

## Description

The codebase contains **163 occurrences** of `.expect()`. While better than `.unwrap()`, these should still be replaced with proper error handling in production code.

### Current State
- 163 `.expect()` calls across the codebase
- Better than `.unwrap()` but still can panic
- Error messages are static strings

### Target State
- `.expect()` only used in tests and initialization code
- Production code uses proper error handling
- Error messages are preserved or improved

### Top Files by Expect Count

| File | Count |
|------|-------|
| `pnl/metrics.rs` | 37 |
| `model/format.rs` | 14 |
| `strategies/iron_butterfly.rs` | 8 |
| `strategies/iron_condor.rs` | 8 |
| `curves/visualization/plotters.rs` | 7 |

## Tasks

- [ ] Audit `.expect()` calls in production code
- [ ] Focus on high-impact files first:
  - `pnl/metrics.rs` (37 occurrences)
  - `model/format.rs` (14 occurrences)
- [ ] Categorize by:
  - Can be replaced with `?` operator
  - Can use `unwrap_or_else()` with logging
  - Is truly unreachable (document with `unreachable!()`)
- [ ] Replace with `?` operator or proper error handling
- [ ] Keep `.expect()` only in tests and initialization code
- [ ] Run `make lint-fix` and `make pre-push` to verify

## Acceptance Criteria

- [ ] `.expect()` only used in tests and initialization
- [ ] All existing tests pass
- [ ] Error messages are preserved or improved
- [ ] No panics in production code paths

## Technical Notes

### Replacement Patterns

```rust
// Before: expect in production code
let value = some_result.expect("Failed to get value");

// After Option 1: Propagate error
let value = some_result.map_err(|e| MyError::from(e))?;

// After Option 2: Log and use default
let value = some_result.unwrap_or_else(|e| {
    tracing::warn!("Failed to get value: {}", e);
    default_value()
});

// After Option 3: Document unreachable
let value = some_result.unwrap_or_else(|_| {
    unreachable!("This case is impossible because...")
});
```

### When `.expect()` is Acceptable

1. **Tests**: Panicking on failure is expected behavior
2. **Initialization**: One-time setup that must succeed
3. **Truly unreachable**: When logic guarantees success (document why)

```rust
// Acceptable in tests
#[test]
fn test_something() {
    let result = do_thing().expect("should succeed in test");
    assert_eq!(result, expected);
}

// Acceptable in initialization
lazy_static! {
    static ref REGEX: Regex = Regex::new(r"^\d+$").expect("valid regex");
}
```

### Files to Update
- `src/pnl/metrics.rs`
- `src/model/format.rs`
- `src/strategies/iron_butterfly.rs`
- `src/strategies/iron_condor.rs`
- Other files with `.expect()` calls

## Estimated Effort

**Medium (3-5 hours)**

## Dependencies

None

## Related Issues

- Issue #1: Reduce unwrap() in chains/chain.rs
- Issue #2: Reduce unwrap() in greeks/equations.rs
- Issue #3: Reduce unwrap() in model/option.rs
