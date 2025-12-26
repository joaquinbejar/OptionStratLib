# Issue #8: Implement Collar strategy

## Title
`feat: Complete implementation of Collar strategy`

## Labels
- `enhancement`
- `strategies`
- `priority-medium`

## Description

The file `src/strategies/collar.rs` is almost empty (**388 bytes**). The Collar strategy is an important protective strategy that should be fully implemented.

### What is a Collar?

A Collar is a protective options strategy that involves:
1. **Long stock position** (or equivalent)
2. **Long put** (protective put) at a lower strike
3. **Short call** (covered call) at a higher strike

The strategy limits both upside and downside, creating a "collar" around the current price.

### Current State
- File exists but is essentially empty
- Strategy is not functional
- Missing from available strategies

### Target State
- Fully functional Collar strategy
- All required traits implemented
- Comprehensive tests and documentation

## Tasks

- [ ] Implement `Collar` struct with all required fields:
  - Underlying position (stock/spot)
  - Long put position
  - Short call position
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

- [ ] Collar strategy is fully functional
- [ ] All trait implementations are complete
- [ ] Tests cover edge cases
- [ ] Documentation includes usage examples
- [ ] Strategy appears in prelude exports

## Technical Notes

### Collar Structure

```rust
pub struct Collar {
    /// Name/identifier for the strategy
    pub name: String,
    /// Underlying asset symbol
    pub symbol: String,
    /// Current underlying price
    pub underlying_price: Positive,
    /// Long put strike (lower)
    pub put_strike: Positive,
    /// Short call strike (upper)
    pub call_strike: Positive,
    /// Expiration date
    pub expiration: ExpirationDate,
    /// Implied volatility
    pub implied_volatility: Positive,
    /// Quantity of underlying shares
    pub quantity: Positive,
    /// Long put position
    pub long_put: Position,
    /// Short call position
    pub short_call: Position,
    /// Underlying position (spot leg)
    pub underlying: Position,
    /// Break-even points
    pub break_even_points: Vec<Positive>,
}
```

### Profit/Loss Profile

```
Profit
  ^
  |     ___________  <- Max profit (capped by short call)
  |    /
  |   /
  |--/-------------> Underlying Price
  | /
  |/______________ <- Max loss (limited by long put)
  |
```

### Key Formulas

```
Max Profit = Call Strike - Entry Price + Net Premium
Max Loss = Entry Price - Put Strike - Net Premium
Break-even = Entry Price - Net Premium (if credit)
           = Entry Price + Net Premium (if debit)
```

### Files to Create/Update
- `src/strategies/collar.rs` (implement)
- `src/strategies/mod.rs` (export)
- `src/prelude.rs` (add to prelude)
- `tests/unit/strategies/collar_test.rs` (tests)

## Estimated Effort

**High (6-8 hours)**

## Dependencies

- Requires spot/underlying leg support (Issue #198)

## Related Issues

- Issue #9: Implement Protective Put strategy
- Issue #198: Base asset legs implementation
