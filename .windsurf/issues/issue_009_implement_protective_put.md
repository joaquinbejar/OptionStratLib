# Issue #9: Implement Protective Put strategy

## Title
`feat: Complete implementation of Protective Put strategy`

## Labels
- `enhancement`
- `strategies`
- `priority-medium`

## Description

The file `src/strategies/protective_put.rs` is almost empty (**358 bytes**). The Protective Put is a fundamental hedging strategy that should be fully implemented.

### What is a Protective Put?

A Protective Put (also known as a "married put") is a hedging strategy that involves:
1. **Long stock position** (or equivalent)
2. **Long put** at or below the current price

The strategy provides downside protection while maintaining unlimited upside potential.

### Current State
- File exists but is essentially empty
- Strategy is not functional
- Missing from available strategies

### Target State
- Fully functional Protective Put strategy
- All required traits implemented
- Comprehensive tests and documentation

## Tasks

- [ ] Implement `ProtectivePut` struct with all required fields:
  - Underlying position (stock/spot)
  - Long put position
  - Break-even points
- [ ] Implement `Strategable` trait
- [ ] Implement `StrategyConstructor` trait
- [ ] Implement `Profit` calculations
- [ ] Implement `Greeks` calculations
- [ ] Implement `DeltaNeutrality` trait
- [ ] Implement `Graph` trait for visualization
- [ ] Implement `ProbabilityAnalysis` trait
- [ ] Add comprehensive tests
- [ ] Add documentation with examples
- [ ] Run `make lint-fix` and `make pre-push` to verify

## Acceptance Criteria

- [ ] Protective Put strategy is fully functional
- [ ] All trait implementations are complete
- [ ] Tests cover edge cases
- [ ] Documentation includes usage examples
- [ ] Strategy appears in prelude exports

## Technical Notes

### Protective Put Structure

```rust
pub struct ProtectivePut {
    /// Name/identifier for the strategy
    pub name: String,
    /// Underlying asset symbol
    pub symbol: String,
    /// Current underlying price
    pub underlying_price: Positive,
    /// Put strike price
    pub put_strike: Positive,
    /// Expiration date
    pub expiration: ExpirationDate,
    /// Implied volatility
    pub implied_volatility: Positive,
    /// Quantity of underlying shares
    pub quantity: Positive,
    /// Long put position
    pub long_put: Position,
    /// Underlying position (spot leg)
    pub underlying: Position,
    /// Break-even point
    pub break_even_points: Vec<Positive>,
}
```

### Profit/Loss Profile

```
Profit
  ^
  |                    /
  |                   /
  |                  /  <- Unlimited upside
  |                 /
  |-----------------/---> Underlying Price
  |________________/
  |                 <- Max loss = Entry - Strike + Premium
```

### Key Formulas

```
Max Profit = Unlimited
Max Loss = (Entry Price - Put Strike) + Put Premium
Break-even = Entry Price + Put Premium
```

### Files to Create/Update
- `src/strategies/protective_put.rs` (implement)
- `src/strategies/mod.rs` (export)
- `src/prelude.rs` (add to prelude)
- `tests/unit/strategies/protective_put_test.rs` (tests)

## Estimated Effort

**Medium (4-6 hours)**

## Dependencies

- Requires spot/underlying leg support (Issue #198)

## Related Issues

- Issue #8: Implement Collar strategy
- Issue #198: Base asset legs implementation
