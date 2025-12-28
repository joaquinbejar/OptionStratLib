# Issue #24: Implement Compound option pricing

## Title
`feat: Implement Compound option pricing model`

## Labels
- `enhancement`
- `pricing`
- `priority-low`

## Description

Implement pricing support for Compound options. Compound options are options on options, where the underlying asset is itself an option.

### Current State
- Compound options return `PricingError::UnsupportedOptionType`
- No pricing model for options on options

### Target State
- Fully functional Compound option pricing
- Support for all four compound types
- Accurate pricing of nested option structures

## Tasks

- [ ] Implement call-on-call compound options
- [ ] Implement call-on-put compound options
- [ ] Implement put-on-call compound options
- [ ] Implement put-on-put compound options
- [ ] Implement Greeks calculations
- [ ] Add comprehensive tests
- [ ] Add documentation with examples
- [ ] Run `make lint-fix` and `make pre-push` to verify

## Acceptance Criteria

- [ ] All compound option types are priced correctly
- [ ] Tests validate against known benchmark values
- [ ] Documentation explains the pricing methodology

## Technical Notes

### Compound Option Types

| Type | Description |
|------|-------------|
| Call on Call | Right to buy a call option |
| Call on Put | Right to buy a put option |
| Put on Call | Right to sell a call option |
| Put on Put | Right to sell a put option |

### Pricing Formula (Geske 1979)

Compound options require bivariate normal distribution:

```
Call on Call = S*M(a1, b1; ρ) - K2*e^(-rT2)*M(a2, b2; ρ) - K1*e^(-rT1)*N(a2)

where M() is the bivariate cumulative normal distribution
ρ = √(T1/T2)
```

### Key Parameters
- K1: Strike of the compound option
- K2: Strike of the underlying option
- T1: Time to expiration of compound option
- T2: Time to expiration of underlying option

### Files to Update
- `src/pricing/black_scholes_model.rs`
- `src/pricing/compound.rs` (new module)
- `tests/unit/pricing/compound_test.rs` (new)

## Estimated Effort

**High (8-12 hours)**

## Dependencies

- Requires bivariate normal distribution implementation

## Related Issues

- Issue #214: Resolve TODOs in black_scholes_model.rs (completed)
