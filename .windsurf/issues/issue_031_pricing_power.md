# Issue #31: Implement Power option pricing

## Title
`feat: Implement Power option pricing model`

## Labels
- `enhancement`
- `pricing`
- `priority-low`

## Description

Implement pricing support for Power options. Power options have payoffs that are a power function of the underlying asset price.

### Current State
- Power options return `PricingError::UnsupportedOptionType`
- No pricing model for non-linear payoffs

### Target State
- Fully functional Power option pricing
- Support for various power exponents
- Accurate pricing of non-linear payoffs

## Tasks

- [ ] Implement standard power option pricing
- [ ] Implement capped power options
- [ ] Support both call and put variants
- [ ] Handle various power exponents (n = 2, 3, etc.)
- [ ] Implement Greeks calculations
- [ ] Add comprehensive tests
- [ ] Add documentation with examples
- [ ] Run `make lint-fix` and `make pre-push` to verify

## Acceptance Criteria

- [ ] Power options are priced correctly
- [ ] Various exponents are supported
- [ ] Tests validate against known benchmark values
- [ ] Documentation explains the pricing methodology

## Technical Notes

### Power Option Payoffs

**Standard Power Call**
```
Payoff = max(S^n - K, 0)
```

**Standard Power Put**
```
Payoff = max(K - S^n, 0)
```

**Powered Payoff (Alternative)**
```
Call: Payoff = [max(S - K, 0)]^n
Put: Payoff = [max(K - S, 0)]^n
```

### Pricing Formula (Power Call, n = 2)

For a squared power call:
```
C = e^(-rT) * E[max(S_T² - K, 0)]

Using log-normal properties:
S_T² is log-normal with adjusted parameters
μ_adj = 2μ + σ²
σ_adj = 2σ
```

### General Formula

```
E[S^n] = S^n * e^(n*(r-q)*T + n*(n-1)*σ²*T/2)
```

### Considerations
- Higher powers increase leverage and risk
- Greeks can be very large for high powers
- Often capped to limit extreme payoffs

### Files to Update
- `src/pricing/black_scholes_model.rs`
- `src/pricing/power.rs` (new module)
- `tests/unit/pricing/power_test.rs` (new)

## Estimated Effort

**Medium (6-8 hours)**

## Dependencies

None

## Related Issues

- Issue #214: Resolve TODOs in black_scholes_model.rs (completed)
