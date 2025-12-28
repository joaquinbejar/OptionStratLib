# Issue #25: Implement Chooser option pricing

## Title
`feat: Implement Chooser option pricing model`

## Labels
- `enhancement`
- `pricing`
- `priority-low`

## Description

Implement pricing support for Chooser options. Chooser options allow the holder to choose whether the option becomes a call or a put at a specified date before expiration.

### Current State
- Chooser options return `PricingError::UnsupportedOptionType`
- No pricing model for choice flexibility

### Target State
- Fully functional Chooser option pricing
- Support for simple and complex choosers
- Accurate pricing of the choice flexibility

## Tasks

- [ ] Implement simple chooser options (same strike and expiration)
- [ ] Implement complex chooser options (different strikes/expirations)
- [ ] Implement Greeks calculations
- [ ] Add comprehensive tests
- [ ] Add documentation with examples
- [ ] Run `make lint-fix` and `make pre-push` to verify

## Acceptance Criteria

- [ ] Chooser options are priced correctly
- [ ] Both simple and complex variants are supported
- [ ] Tests validate against known benchmark values
- [ ] Documentation explains the pricing methodology

## Technical Notes

### Chooser Option Types

**Simple Chooser**
- Same strike K and expiration T for both call and put
- Choice date t < T

**Complex Chooser**
- Different strikes (K_c, K_p) and/or expirations (T_c, T_p)
- More flexible but harder to price

### Pricing Formula (Simple Chooser - Rubinstein 1991)

```
Chooser = S*e^(-qT)*N(d) - K*e^(-rT)*N(d - σ√T)
        + K*e^(-rT)*N(-d + σ√t) - S*e^(-qT)*N(-d)

where d = [ln(S/K) + (r-q+σ²/2)T] / (σ√T)
```

Alternative decomposition:
```
Simple Chooser = Call(K, T) + Put(K, t) * e^(-(r-q)(T-t))
```

### Files to Update
- `src/pricing/black_scholes_model.rs`
- `src/pricing/chooser.rs` (new module)
- `tests/unit/pricing/chooser_test.rs` (new)

## Estimated Effort

**Medium (6-8 hours)**

## Dependencies

None

## Related Issues

- Issue #214: Resolve TODOs in black_scholes_model.rs (completed)
