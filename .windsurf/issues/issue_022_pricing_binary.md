# Issue #22: Implement Binary option pricing

## Title
`feat: Implement Binary option pricing model`

## Labels
- `enhancement`
- `pricing`
- `priority-medium`

## Description

Implement pricing support for Binary options (also known as Digital options). Binary options have a fixed payout if the option expires in-the-money, regardless of how far in-the-money it is.

### Current State
- Binary options return `PricingError::UnsupportedOptionType`
- No pricing model for fixed payoff options

### Target State
- Fully functional Binary option pricing
- Support for cash-or-nothing and asset-or-nothing variants
- Support for both calls and puts

## Tasks

- [ ] Implement cash-or-nothing binary options
- [ ] Implement asset-or-nothing binary options
- [ ] Support both call and put variants
- [ ] Implement Greeks calculations (note: delta is discontinuous)
- [ ] Add comprehensive tests
- [ ] Add documentation with examples
- [ ] Run `make lint-fix` and `make pre-push` to verify

## Acceptance Criteria

- [ ] Binary options are priced correctly
- [ ] Both payout types are supported
- [ ] Tests validate against known benchmark values
- [ ] Documentation explains the pricing methodology

## Technical Notes

### Binary Option Types

**Cash-or-Nothing**
- Pays fixed cash amount Q if in-the-money
- Call: pays Q if S > K at expiration
- Put: pays Q if S < K at expiration

**Asset-or-Nothing**
- Pays the asset value if in-the-money
- Call: pays S if S > K at expiration
- Put: pays S if S < K at expiration

### Pricing Formulas

**Cash-or-Nothing Call**
```
C = Q * e^(-rT) * N(d2)
```

**Cash-or-Nothing Put**
```
P = Q * e^(-rT) * N(-d2)
```

**Asset-or-Nothing Call**
```
C = S * e^(-qT) * N(d1)
```

**Asset-or-Nothing Put**
```
P = S * e^(-qT) * N(-d1)
```

### Greeks Considerations
- Delta is discontinuous at the strike
- Gamma can be very large near expiration
- Vega changes sign near the strike

### Files to Update
- `src/pricing/black_scholes_model.rs`
- `src/pricing/binary.rs` (new module)
- `tests/unit/pricing/binary_test.rs` (new)

## Estimated Effort

**Medium (4-6 hours)**

## Dependencies

None

## Related Issues

- Issue #214: Resolve TODOs in black_scholes_model.rs (completed)
