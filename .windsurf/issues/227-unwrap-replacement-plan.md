# Issue #227: Replace unwrap() calls with proper error handling in src/model

> **GitHub Issue:** [#227](https://github.com/joaquinbejar/OptionStratLib/issues/227)  
> **Priority:** High  
> **Total Estimated Effort:** 12-18 hours  
> **Created:** 2026-01-11

## Overview

This document outlines the phased implementation plan for replacing `unwrap()` calls with proper error handling in the `src/model` directory.

### Current State Analysis

| File | Total `unwrap()` | In Tests | In Production Code |
|------|------------------|----------|-------------------|
| `option.rs` | 122 | ~100 | ~22 |
| `position.rs` | 109 | ~100 | ~9 |
| `expiration.rs` | 72 | ~60 | ~12 |
| `trade.rs` | 22 | ~18 | ~4 |
| `decimal.rs` | 12 | ~10 | ~2 |
| `profit_range.rs` | 9 | ~7 | ~2 |
| `utils.rs` | 7 | ~5 | ~2 |
| `leg/*.rs` | 18 | ~15 | ~3 |
| Other files | 8 | ~6 | ~2 |
| **Total** | **379** | **~321** | **~58** |

---

## Implementation Phases

### Phase 1: Critical Production Code in `option.rs`
**Effort:** 4-6 hours  
**Status:** ⬜ Not Started

#### Scope
Replace `unwrap()` calls in production code within `src/model/option.rs`:

- [ ] Pricing methods: `calculate_price_binomial`, `calculate_price_black_scholes`
- [ ] Time calculations: `time_to_expiration`
- [ ] Value calculations: `time_value`, `intrinsic_value`
- [ ] Payoff calculations: `payoff`, `payoff_at_price`
- [ ] Numeric conversions: `.to_f64().unwrap()` patterns

#### Replacement Strategy
| Pattern | Replacement |
|---------|-------------|
| `.to_f64().unwrap()` | `.to_f64_lossy()` or propagate with `?` |
| `Positive::try_from().unwrap()` | Propagate error with `?` |
| `DateTime::parse().unwrap()` | Return `Result` with descriptive error |

#### Files to Modify
- `src/model/option.rs`

#### Acceptance Criteria
- [ ] No `unwrap()` in non-test code in `option.rs`
- [ ] All existing tests pass
- [ ] `make lint-fix` passes
- [ ] `make pre-push` passes

---

### Phase 2: Critical Production Code in `position.rs` and `expiration.rs`
**Effort:** 3-4 hours  
**Status:** ⬜ Not Started

#### Scope
Replace `unwrap()` calls in production code:

**position.rs:**
- [ ] `days_held()` - uses `pos_or_panic!`
- [ ] `days_to_expiration()` - uses `pos_or_panic!`
- [ ] PnL calculations
- [ ] Break-even calculations

**expiration.rs:**
- [ ] Date parsing methods
- [ ] Time conversion methods
- [ ] `get_date()` method
- [ ] Duration calculations

#### Replacement Strategy
| Pattern | Replacement |
|---------|-------------|
| `pos_or_panic!(value)` | `Positive::try_from(value)?` |
| `datetime.unwrap()` | Propagate with `?` or use `ok_or_else()` |
| `.num_days() as f64` | Safe conversion with bounds checking |

#### Files to Modify
- `src/model/position.rs`
- `src/model/expiration.rs`

#### Acceptance Criteria
- [ ] No `unwrap()` in non-test code in these files
- [ ] All existing tests pass
- [ ] `make lint-fix` passes
- [ ] `make pre-push` passes

---

### Phase 3: Secondary Production Code
**Effort:** 2-3 hours  
**Status:** ⬜ Not Started

#### Scope
Replace `unwrap()` calls in remaining production code:

- [ ] `trade.rs` (~4 unwraps)
- [ ] `decimal.rs` (~2 unwraps)
- [ ] `profit_range.rs` (~2 unwraps)
- [ ] `utils.rs` (~2 unwraps)
- [ ] `leg/leg_enum.rs` (~3 unwraps)
- [ ] `leg/spot.rs` (~2 unwraps)
- [ ] `leg/future.rs` (~1 unwrap)
- [ ] `leg/perpetual.rs` (~1 unwrap)

#### Files to Modify
- `src/model/trade.rs`
- `src/model/decimal.rs`
- `src/model/profit_range.rs`
- `src/model/utils.rs`
- `src/model/leg/*.rs`

#### Acceptance Criteria
- [ ] No `unwrap()` in non-test code in these files
- [ ] All existing tests pass
- [ ] `make lint-fix` passes
- [ ] `make pre-push` passes

---

### Phase 4: Test Code Improvements (Optional)
**Effort:** 6-8 hours  
**Status:** ⬜ Not Started

#### Scope
Improve error handling in test code (~321 `unwrap()` calls):

**Option A (Recommended):** Replace with `expect("descriptive message")`
- Provides better error messages on test failures
- Minimal code changes

**Option B:** Use `?` with test functions returning `Result`
- More idiomatic but requires changing test signatures

#### Priority Order
1. `option.rs` tests (~100 unwraps)
2. `position.rs` tests (~100 unwraps)
3. `expiration.rs` tests (~60 unwraps)
4. Remaining test files

#### Acceptance Criteria
- [ ] All `unwrap()` replaced with `expect()` or `?`
- [ ] All tests pass
- [ ] Error messages are descriptive

---

## Error Types Reference

The codebase already has well-defined error types to use:

```rust
// From src/error/
PositionError          // For position-related errors
PricingError           // For pricing calculation errors
GreeksError            // For Greeks calculation errors
StrategyError          // For strategy-related errors
TradeError             // For trade-related errors
TransactionError       // For transaction errors
```

### Error Conversion Pattern

```rust
// Before
let value = some_option.unwrap();

// After - Option A: Using ok_or_else
let value = some_option.ok_or_else(|| {
    PricingError::calculation_error("descriptive message")
})?;

// After - Option B: Using map_err
let value = some_result.map_err(|e| {
    PositionError::from(e)
})?;
```

---

## Progress Tracking

### Commits
| Phase | Commit Hash | Date | Notes |
|-------|-------------|------|-------|
| 1 | - | - | - |
| 2 | - | - | - |
| 3 | - | - | - |
| 4 | - | - | - |

### Verification Commands

```bash
# After each phase, run:
make lint-fix
make pre-push

# To check remaining unwraps in production code:
grep -r "\.unwrap()" src/model/*.rs | grep -v "#\[cfg(test)\]" | grep -v "mod tests"

# To count unwraps per file:
for f in src/model/*.rs; do echo "$f: $(grep -c '\.unwrap()' $f 2>/dev/null || echo 0)"; done
```

---

## Notes

- **Tests:** `unwrap()` in tests is generally acceptable but `expect()` provides better debugging
- **pos_or_panic!:** This macro is used throughout and should be replaced with fallible alternatives
- **Backward Compatibility:** Some public methods may need signature changes from `-> T` to `-> Result<T, Error>`

---

*Last Updated: 2026-01-11*
