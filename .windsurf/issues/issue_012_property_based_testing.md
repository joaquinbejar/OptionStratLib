# Issue #12: Add property-based testing

## Title
`test: Add property-based testing for mathematical invariants`

## Labels
- `testing`
- `quality`
- `priority-low`

## Description

Add property-based testing using `proptest` to validate mathematical invariants like Put-Call Parity and Greeks bounds.

### Current State
- Tests use specific example values
- Mathematical invariants not systematically tested
- Edge cases may be missed

### Target State
- Property-based tests cover key mathematical invariants
- Tests run as part of the test suite
- Higher confidence in mathematical correctness

## Tasks

- [ ] Add `proptest` as a dev dependency
- [ ] Add property tests for Put-Call Parity:
  - `C - P = S - K * e^(-rT)`
- [ ] Add property tests for Greeks bounds:
  - `0 ≤ delta ≤ 1` for calls
  - `-1 ≤ delta ≤ 0` for puts
  - `gamma ≥ 0` for all options
- [ ] Add property tests for arbitrage-free pricing:
  - `C ≥ max(0, S - K * e^(-rT))`
  - `P ≥ max(0, K * e^(-rT) - S)`
- [ ] Add property tests for Positive type invariants:
  - Value is always > 0
  - Operations preserve positivity where expected
- [ ] Add property tests for strategy invariants:
  - Break-even points are within valid range
  - Max profit/loss are consistent
- [ ] Run `make lint-fix` and `make pre-push` to verify

## Acceptance Criteria

- [ ] Property tests cover key mathematical invariants
- [ ] Tests run as part of the test suite
- [ ] No false positives in normal operation
- [ ] Edge cases are properly handled

## Technical Notes

### Put-Call Parity Test

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_put_call_parity(
        s in 10.0f64..1000.0,
        k in 10.0f64..1000.0,
        r in 0.0f64..0.2,
        sigma in 0.05f64..1.0,
        t in 0.01f64..2.0,
    ) {
        let s = pos!(s);
        let k = pos!(k);
        let sigma = pos!(sigma);
        let t = pos!(t);
        
        let call = black_scholes_call(s, k, r, sigma, t);
        let put = black_scholes_put(s, k, r, sigma, t);
        
        let parity_lhs = call - put;
        let parity_rhs = s.to_dec() - k.to_dec() * (-r * t.to_dec()).exp();
        
        prop_assert!((parity_lhs - parity_rhs).abs() < dec!(0.0001));
    }
}
```

### Greeks Bounds Test

```rust
proptest! {
    #[test]
    fn test_call_delta_bounds(
        s in 10.0f64..1000.0,
        k in 10.0f64..1000.0,
        sigma in 0.05f64..1.0,
        t in 0.01f64..2.0,
    ) {
        let option = create_call_option(s, k, sigma, t);
        let delta = option.delta().unwrap();
        
        prop_assert!(delta >= dec!(0.0));
        prop_assert!(delta <= dec!(1.0));
    }
    
    #[test]
    fn test_gamma_non_negative(
        s in 10.0f64..1000.0,
        k in 10.0f64..1000.0,
        sigma in 0.05f64..1.0,
        t in 0.01f64..2.0,
    ) {
        let option = create_call_option(s, k, sigma, t);
        let gamma = option.gamma().unwrap();
        
        prop_assert!(gamma >= dec!(0.0));
    }
}
```

### Files to Create/Update
- `Cargo.toml` (add proptest dependency)
- `tests/property/mod.rs`
- `tests/property/put_call_parity_test.rs`
- `tests/property/greeks_bounds_test.rs`
- `tests/property/positive_invariants_test.rs`

## Estimated Effort

**Medium (4-6 hours)**

## Dependencies

None

## Related Issues

- Issue #11: Add benchmarks for critical paths
