# Issue #7: Reduce unnecessary `.clone()` calls

## Title
`perf: Reduce unnecessary clone() calls across the codebase`

## Labels
- `performance`
- `refactor`
- `priority-medium`

## Description

The codebase contains **795 occurrences** of `.clone()`. Many of these could be avoided with better use of references, `Cow<T>`, or `Arc<T>`.

### Current State
- 795 `.clone()` calls across the codebase
- Unnecessary memory allocations
- Potential performance impact in hot paths

### Target State
- Clone count reduced by at least 50%
- No unnecessary memory allocations
- Improved performance in critical paths

### Top Files by Clone Count

| File | Count |
|------|-------|
| `chains/chain.rs` | 55 |
| `strategies/short_strangle.rs` | 49 |
| `strategies/iron_condor.rs` | 46 |
| `strategies/call_butterfly.rs` | 39 |
| `strategies/iron_butterfly.rs` | 38 |

## Tasks

- [ ] Audit `.clone()` calls in high-frequency code paths
- [ ] Categorize clones by necessity:
  - Required for ownership transfer
  - Can be replaced with reference
  - Can use `Cow<'_, T>`
  - Can use `Arc<T>` for shared data
- [ ] Replace unnecessary clones with references
- [ ] Use `Cow<'_, T>` for data that is rarely modified
- [ ] Use `Arc<T>` for shared data in multi-threaded contexts
- [ ] Benchmark critical paths before and after
- [ ] Run `make lint-fix` and `make pre-push` to verify

## Acceptance Criteria

- [ ] Clone count reduced by at least 50%
- [ ] No performance regression (verified by benchmarks)
- [ ] All tests pass
- [ ] Code remains readable and maintainable

## Technical Notes

### Common Patterns to Apply

```rust
// Before: Unnecessary clone
fn process(data: &MyStruct) {
    let owned = data.clone();
    do_something(&owned);
}

// After: Use reference directly
fn process(data: &MyStruct) {
    do_something(data);
}
```

```rust
// Before: Clone for potential modification
fn maybe_modify(data: String) -> String {
    if should_modify() {
        data.to_uppercase()
    } else {
        data
    }
}

// After: Use Cow for lazy cloning
fn maybe_modify(data: &str) -> Cow<'_, str> {
    if should_modify() {
        Cow::Owned(data.to_uppercase())
    } else {
        Cow::Borrowed(data)
    }
}
```

```rust
// Before: Clone for sharing
let config = config.clone();
thread::spawn(move || use_config(&config));

// After: Use Arc for sharing
let config = Arc::new(config);
let config_clone = Arc::clone(&config);
thread::spawn(move || use_config(&config_clone));
```

### Files to Update
- `src/chains/chain.rs`
- `src/strategies/*.rs`
- Other files with high clone counts

## Estimated Effort

**Medium (4-6 hours)**

## Dependencies

- Issue #6 (may reduce clones as side effect)

## Related Issues

- Issue #6: Extract common strategy logic
