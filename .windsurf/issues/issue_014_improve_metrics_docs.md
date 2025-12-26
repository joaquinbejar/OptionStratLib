# Issue #14: Improve documentation for metrics modules

## Title
`docs: Add comprehensive documentation for metrics modules`

## Labels
- `documentation`
- `priority-low`

## Description

The new metrics modules (`composite/`, `liquidity/`, `stress/`, `temporal/`) need more comprehensive documentation with examples.

### Current State
- Basic documentation exists
- Limited examples
- Mathematical background could be expanded

### Target State
- All public items have documentation
- Examples are runnable
- Mathematical formulas are explained

## Tasks

- [ ] Add module-level documentation with overview for:
  - `src/metrics/composite/` (Vanna-Volga, Delta-Gamma, Smile Dynamics)
  - `src/metrics/liquidity/` (Bid-Ask Spread, Volume Profile, Open Interest)
  - `src/metrics/stress/` (Volatility Sensitivity, Time Decay, Price Shock)
  - `src/metrics/temporal/` (Theta, Charm, Color)
- [ ] Add examples for each trait implementation
- [ ] Add mathematical background where appropriate
- [ ] Add usage examples in doc comments
- [ ] Verify examples compile with `cargo test --doc`
- [ ] Run `make lint-fix` and `make pre-push` to verify

## Acceptance Criteria

- [ ] All public items have documentation
- [ ] Examples are runnable (`cargo test --doc` passes)
- [ ] Mathematical formulas are explained
- [ ] Documentation follows library style

## Technical Notes

### Documentation Template

```rust
/// # Vanna-Volga Hedge Surface
///
/// Computes the Vanna-Volga hedge cost surface across price and volatility.
///
/// ## Mathematical Background
///
/// The Vanna-Volga method adjusts Black-Scholes prices to account for
/// the volatility smile using three benchmark options:
///
/// ```text
/// Price_VV = Price_BS + Vanna × (σ_market - σ_ATM) × S × √T
///          + 0.5 × Volga × (σ_market - σ_ATM)²
/// ```
///
/// ## Example
///
/// ```rust
/// use optionstratlib::prelude::*;
/// use optionstratlib::metrics::VannaVolgaSurface;
///
/// let chain = OptionChain::new("SPY", pos!(450.0), "2024-12-31".to_string(), None, None);
/// let price_range = (pos!(400.0), pos!(500.0));
/// let vol_range = (pos!(0.10), pos!(0.40));
///
/// let surface = chain.vanna_volga_surface(price_range, vol_range, 20, 20)?;
/// println!("Surface has {} points", surface.points.len());
/// # Ok::<(), optionstratlib::error::SurfaceError>(())
/// ```
///
/// ## Returns
///
/// A `Surface` where:
/// - **X-axis**: Underlying price in currency units
/// - **Y-axis**: Implied volatility as a decimal
/// - **Z-axis**: Vanna-Volga hedge cost/adjustment
pub trait VannaVolgaSurface {
    // ...
}
```

### Files to Update
- `src/metrics/composite/mod.rs`
- `src/metrics/composite/vanna_volga.rs`
- `src/metrics/composite/delta_gamma_profile.rs`
- `src/metrics/composite/smile_dynamics.rs`
- `src/metrics/liquidity/mod.rs`
- `src/metrics/liquidity/bid_ask_spread.rs`
- `src/metrics/liquidity/volume_profile.rs`
- `src/metrics/liquidity/open_interest.rs`
- `src/metrics/stress/mod.rs`
- `src/metrics/stress/volatility_sensitivity.rs`
- `src/metrics/stress/time_decay.rs`
- `src/metrics/stress/price_shock.rs`
- `src/metrics/temporal/mod.rs`
- `src/metrics/temporal/theta.rs`
- `src/metrics/temporal/charm.rs`
- `src/metrics/temporal/color.rs`

## Estimated Effort

**Low (2-4 hours)**

## Dependencies

None

## Related Issues

None
