# Issue #30: Implement Exchange option pricing

## Title
`feat: Implement Exchange option pricing model`

## Labels
- `enhancement`
- `pricing`
- `priority-medium`

## Description

Implement pricing support for Exchange options (also known as Margrabe options). Exchange options give the holder the right to exchange one asset for another.

### Current State
- Exchange options return `PricingError::UnsupportedOptionType`
- No pricing model for asset exchange

### Target State
- Fully functional Exchange option pricing
- Proper correlation handling between assets
- Support for dividend-paying assets

## Tasks

- [ ] Implement Margrabe's formula for exchange options
- [ ] Handle correlation between underlying assets
- [ ] Support dividend yields on both assets
- [ ] Implement Greeks calculations
- [ ] Add comprehensive tests
- [ ] Add documentation with examples
- [ ] Run `make lint-fix` and `make pre-push` to verify

## Acceptance Criteria

- [ ] Exchange options are priced correctly
- [ ] Correlation is handled properly
- [ ] Tests validate against known benchmark values
- [ ] Documentation explains the pricing methodology

## Technical Notes

### Exchange Option Payoff

```
Payoff = max(S1 - S2, 0)
```

The holder has the right to exchange asset 2 for asset 1.

### Margrabe's Formula (1978)

```
C = S1*e^(-q1*T)*N(d1) - S2*e^(-q2*T)*N(d2)

where:
d1 = [ln(S1/S2) + (q2 - q1 + σ²/2)*T] / (σ*√T)
d2 = d1 - σ*√T
σ = √(σ1² + σ2² - 2*ρ*σ1*σ2)
```

### Key Properties
- No strike price (K = 0 effectively)
- Special case of spread option
- Symmetric: exchange(S1, S2) + exchange(S2, S1) = S1 + S2

### Applications
- Stock-for-stock mergers
- Switching options in energy markets
- Outperformance options

### Files to Update
- `src/pricing/black_scholes_model.rs`
- `src/pricing/exchange.rs` (new module)
- `tests/unit/pricing/exchange_test.rs` (new)

## Estimated Effort

**Medium (4-6 hours)**

## Dependencies

None

## Related Issues

- Issue #214: Resolve TODOs in black_scholes_model.rs (completed)
- Issue #28: Spread option pricing (related)
