# Issue #6: Extract common logic from strategy files

## Title
`refactor: Extract common strategy logic to reduce file sizes`

## Labels
- `refactor`
- `strategies`
- `priority-medium`

## Description

Several strategy files are excessively large due to duplicated logic:

| File | Size |
|------|------|
| `short_strangle.rs` | 129KB |
| `iron_condor.rs` | 126KB |
| `iron_butterfly.rs` | 113KB |
| `long_butterfly_spread.rs` | 112KB |
| `bear_call_spread.rs` | 101KB |
| `bear_put_spread.rs` | 100KB |

### Current State
- Large files that are difficult to maintain
- Significant code duplication across strategies
- Similar patterns repeated in each strategy

### Target State
- Each strategy file under 50KB
- Common logic extracted to shared modules
- Clear separation between strategy-specific and shared code

## Tasks

- [ ] Identify common patterns across strategy implementations:
  - Break-even calculations
  - Profit/loss calculations
  - Greeks aggregation
  - Validation logic
  - Optimization routines
- [ ] Create shared traits for strategy categories:
  - `SpreadStrategy` for vertical spreads
  - `ButterflyStrategy` for butterfly patterns
  - `CondorStrategy` for condor patterns
  - `StraddleStrategy` for straddle/strangle patterns
- [ ] Extract common calculation methods to `strategies/utils.rs`
- [ ] Create helper macros in `strategies/macros.rs` for repetitive code
- [ ] Refactor strategies to use shared traits and utilities
- [ ] Ensure all tests pass after refactoring
- [ ] Run `make lint-fix` and `make pre-push` to verify

## Acceptance Criteria

- [ ] Each strategy file is under 50KB
- [ ] No functionality is lost
- [ ] Code duplication is significantly reduced
- [ ] All tests pass
- [ ] New shared code is well documented

## Technical Notes

### Proposed Trait Hierarchy

```rust
/// Base trait for all spread strategies
pub trait SpreadStrategy: Strategable {
    /// Returns the lower strike price
    fn lower_strike(&self) -> Positive;
    
    /// Returns the upper strike price
    fn upper_strike(&self) -> Positive;
    
    /// Returns the spread width
    fn spread_width(&self) -> Positive {
        self.upper_strike() - self.lower_strike()
    }
}

/// Trait for butterfly-type strategies
pub trait ButterflyStrategy: Strategable {
    /// Returns the wing strikes (lower, upper)
    fn wing_strikes(&self) -> (Positive, Positive);
    
    /// Returns the body (middle) strike
    fn body_strike(&self) -> Positive;
    
    /// Returns the wing width
    fn wing_width(&self) -> Positive {
        let (lower, upper) = self.wing_strikes();
        (upper - lower) / dec!(2)
    }
}

/// Trait for condor-type strategies
pub trait CondorStrategy: Strategable {
    /// Returns all four strikes (lowest to highest)
    fn strikes(&self) -> (Positive, Positive, Positive);
    
    /// Returns the inner spread width
    fn inner_width(&self) -> Positive;
    
    /// Returns the outer spread width
    fn outer_width(&self) -> Positive;
}
```

### Common Utility Functions to Extract

```rust
// strategies/utils.rs

/// Calculate break-even for a two-leg spread
pub fn spread_break_even(
    lower_strike: Positive,
    upper_strike: Positive,
    net_premium: Decimal,
    is_credit: bool,
) -> Vec<Positive>;

/// Aggregate Greeks from multiple positions
pub fn aggregate_greeks(positions: &[Position]) -> Result<Greeks, GreeksError>;

/// Calculate max profit/loss for bounded strategies
pub fn calculate_bounded_pnl(
    positions: &[Position],
    lower_bound: Positive,
    upper_bound: Positive,
) -> (Option<Decimal>, Option<Decimal>);
```

### Files to Update
- `src/strategies/utils.rs` (add shared utilities)
- `src/strategies/macros.rs` (add helper macros)
- All strategy files (refactor to use shared code)
- `src/strategies/mod.rs` (export new traits)

## Estimated Effort

**High (8-12 hours)**

## Dependencies

None

## Related Issues

- Issue #7: Reduce unnecessary clone() calls
