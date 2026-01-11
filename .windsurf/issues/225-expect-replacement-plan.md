# Issue #225: Replace expect() calls with proper error handling

## Summary

Replace `.expect()` calls in production code with proper error handling using `Result` types, `unwrap_or_else()`, or documented `unreachable!()` where appropriate.

## Scope

Focus on **production code only** - `.expect()` in tests is acceptable as panicking on failure is expected behavior.

## Files to Update

### Phase 1: Strategy Files (High Priority)
These files have multiple `.expect()` calls in strategy constructors:

| File | Count | Priority |
|------|-------|----------|
| `src/strategies/iron_condor.rs` | 5 | High |
| `src/strategies/iron_butterfly.rs` | 5 | High |
| `src/strategies/bull_call_spread.rs` | 3 | High |
| `src/strategies/bull_put_spread.rs` | 3 | High |
| `src/strategies/bear_call_spread.rs` | 3 | High |
| `src/strategies/bear_put_spread.rs` | 3 | High |
| `src/strategies/long_strangle.rs` | 3 | Medium |
| `src/strategies/short_strangle.rs` | 3 | Medium |
| `src/strategies/long_straddle.rs` | 3 | Medium |
| `src/strategies/short_straddle.rs` | 3 | Medium |
| `src/strategies/call_butterfly.rs` | 4 | Medium |
| `src/strategies/long_butterfly_spread.rs` | 2 | Medium |
| `src/strategies/short_butterfly_spread.rs` | 2 | Medium |
| `src/strategies/long_call.rs` | 2 | Medium |
| `src/strategies/short_call.rs` | 2 | Medium |
| `src/strategies/short_put.rs` | 2 | Medium |
| `src/strategies/covered_call.rs` | 1 | Low |
| `src/strategies/custom.rs` | 1 | Low |
| `src/strategies/poor_mans_covered_call.rs` | 2 | Medium |

### Phase 2: Model Files
| File | Count | Priority |
|------|-------|----------|
| `src/model/format.rs` | 10 | High |
| `src/model/decimal.rs` | 1 | Low |
| `src/model/trade.rs` | 1 | Low |

### Phase 3: Other Files
| File | Count | Priority |
|------|-------|----------|
| `src/chains/chain.rs` | 2 | Medium |
| `src/surfaces/surface.rs` | 1 | Low |
| `src/curves/curve.rs` | 1 | Low |
| `src/utils/logger.rs` | 2 | Low |
| `src/pricing/monte_carlo.rs` | 1 | Medium |

## Replacement Patterns

### Pattern 1: Return Result instead of panic
```rust
// Before
fn new(...) -> Self {
    self.add_position(...).expect("Invalid position");
}

// After
fn new(...) -> Result<Self, StrategyError> {
    self.add_position(...)?;
    Ok(self)
}
```

### Pattern 2: Use unwrap_or_else with fallback
```rust
// Before
let value = result.expect("message");

// After
let value = result.unwrap_or_else(|e| {
    tracing::warn!("Failed: {}", e);
    default_value
});
```

### Pattern 3: Document unreachable cases
```rust
// Before
let value = result.expect("should never fail");

// After - only when truly unreachable
let value = result.unwrap_or_else(|_| {
    unreachable!("This case is impossible because [documented reason]")
});
```

### Pattern 4: Keep expect() with better message (acceptable cases)
```rust
// Acceptable in initialization/static contexts
lazy_static! {
    static ref REGEX: Regex = Regex::new(r"^\d+$")
        .expect("regex pattern is valid and compile-time constant");
}
```

## Implementation Tasks

### Phase 1: Strategy Files
- [ ] Update `iron_condor.rs` - convert constructor to return `Result`
- [ ] Update `iron_butterfly.rs` - convert constructor to return `Result`
- [ ] Update spread strategies (bull_call, bull_put, bear_call, bear_put)
- [ ] Update strangle/straddle strategies
- [ ] Update butterfly strategies
- [ ] Update single-leg strategies (long_call, short_call, short_put)

### Phase 2: Model Files
- [ ] Update `format.rs` - handle date/time parsing errors
- [ ] Review `decimal.rs` - document why Normal distribution is always valid
- [ ] Review `trade.rs` - handle trade() method properly

### Phase 3: Other Files
- [ ] Update `chains/chain.rs` - handle empty strike_prices
- [ ] Update `surfaces/surface.rs` - handle index bounds
- [ ] Update `curves/curve.rs` - handle index bounds
- [ ] Review `utils/logger.rs` - initialization errors
- [ ] Update `pricing/monte_carlo.rs` - handle payoff calculation

## Acceptance Criteria

- [ ] No `.expect()` calls in production code paths (tests excluded)
- [ ] All replaced calls use appropriate error handling pattern
- [ ] No compilation errors
- [ ] All tests pass
- [ ] `make lint-fix` passes
- [ ] `make pre-push` passes

## Notes

- Tests can keep `.expect()` as panicking on failure is expected
- Initialization code (lazy_static, one-time setup) may keep `.expect()` with documented reason
- Progress bar template failures can use `unwrap_or_else` with fallback

## Related Issues

- #227 - Replace unwrap() calls (completed)
- #220 - Add error context using anyhow at API boundaries
