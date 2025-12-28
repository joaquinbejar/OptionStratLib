# Issue #21: Implement Barrier option pricing

## Title
`feat: Implement Barrier option pricing model`

## Labels
- `enhancement`
- `pricing`
- `priority-medium`

## Description

Implement pricing support for Barrier options. Barrier options are path-dependent options that are activated or deactivated when the underlying price crosses a specified barrier level.

### Current State
- Barrier options return `PricingError::UnsupportedOptionType`
- No pricing model for barrier monitoring

### Target State
- Fully functional Barrier option pricing
- Support for all barrier types (knock-in, knock-out)
- Support for up and down barriers

## Tasks

- [ ] Implement knock-in barrier options (up-and-in, down-and-in)
- [ ] Implement knock-out barrier options (up-and-out, down-and-out)
- [ ] Support continuous and discrete barrier monitoring
- [ ] Implement rebate payments for knock-out options
- [ ] Implement Greeks calculations
- [ ] Add comprehensive tests
- [ ] Add documentation with examples
- [ ] Run `make lint-fix` and `make pre-push` to verify

## Acceptance Criteria

- [ ] All barrier types are priced correctly
- [ ] Barrier monitoring is handled properly
- [ ] Tests validate against known benchmark values
- [ ] Documentation explains the pricing methodology

## Technical Notes

### Barrier Option Types

| Type | Description |
|------|-------------|
| Up-and-Out | Knocked out if price rises above barrier |
| Up-and-In | Activated if price rises above barrier |
| Down-and-Out | Knocked out if price falls below barrier |
| Down-and-In | Activated if price falls below barrier |

### Pricing Formulas (Continuous Monitoring)

Closed-form solutions exist for continuously monitored barriers:

```
Down-and-Out Call:
C_do = C_bs - (S/H)^(2λ) * C_bs(H²/S)

where λ = (r - q + σ²/2) / σ²
```

### In-Out Parity
```
Knock-In + Knock-Out = Vanilla Option
```

### Files to Update
- `src/pricing/black_scholes_model.rs`
- `src/pricing/barrier.rs` (new module)
- `tests/unit/pricing/barrier_test.rs` (new)

## Estimated Effort

**High (8-12 hours)**

## Dependencies

None

## Related Issues

- Issue #214: Resolve TODOs in black_scholes_model.rs (completed)
