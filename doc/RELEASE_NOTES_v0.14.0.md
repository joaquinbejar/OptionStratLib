# Release Notes - OptionStratLib v0.14.2

**Release Date:** January 2025

This is a major release that introduces comprehensive exotic option pricing support, new metrics modules, improved error handling, and significant code quality improvements.

---

## Highlights

- **Complete Exotic Options Support**: All 14 exotic option types now have full pricing implementations
- **New Metrics Framework**: Comprehensive metrics system with curves and surfaces for risk analysis
- **Improved Error Handling**: Systematic replacement of `unwrap()` calls with proper error handling
- **New Strategies**: CoveredCall, Collar, and ProtectivePut strategies fully implemented
- **Multi-Instrument Support**: New `Leg` enum for multi-asset strategy support

---

## New Features

### Exotic Option Pricing Models

Complete pricing implementations for all exotic option types:

| Issue | Option Type | Description |
|-------|-------------|-------------|
| #235 | American | Barone-Adesi-Whaley approximation for early exercise |
| #236 | Bermuda | Discrete exercise dates pricing |
| #237 | Asian | Arithmetic and geometric average price options |
| #238 | Barrier | Up/Down, In/Out barrier options with numerical Greeks |
| #239 | Binary | Cash-or-nothing and asset-or-nothing options |
| #240 | Lookback | Fixed and floating strike lookback options |
| #241 | Compound | Options on options pricing |
| #242 | Chooser | Options to choose call or put at future date |
| #243 | Cliquet | Forward-starting options with local caps/floors |
| #244 | Rainbow | Multi-asset best-of/worst-of options |
| #245 | Spread | Kirk's approximation and Margrabe's formula |
| #246 | Quanto | Currency-protected options |
| #247 | Exchange | Margrabe's formula for asset exchange |
| #248 | Power | Non-linear payoff options with adjusted volatility |

### Metrics Framework

New comprehensive metrics system for option chain analysis:

#### Risk Metrics (#139)
- `ImpliedVolatilityCurve` / `ImpliedVolatilitySurface`
- `RiskReversalCurve`
- `DollarGammaCurve`

#### Volatility Metrics (#136)
- `VannaCurve` / `VannaSurface`
- `VolgaCurve` / `VolgaSurface`
- `VommaCurve` / `VommaSurface`
- `VetaCurve` / `VetaSurface`

#### Temporal Metrics (#137)
- `ThetaCurve` / `ThetaSurface`
- `CharmCurve` / `CharmSurface`
- `ColorCurve` / `ColorSurface`

#### Price Metrics (#138)
- `VolatilitySkewCurve`
- `PutCallRatioCurve`
- `StrikeConcentrationCurve`

#### Composite Metrics (#140)
- `SmileDynamics`
- `DeltaGammaProfile`
- `VannaVolga`

#### Liquidity Metrics (#141)
- `BidAskSpreadCurve`
- `VolumeProfileCurve` / `VolumeProfileSurface`
- `OpenInterestCurve`

#### Stress Metrics (#142)
- `VolatilitySensitivityCurve` / `VolatilitySensitivitySurface`
- `TimeDecayCurve` / `TimeDecaySurface`
- `PriceShockCurve` / `PriceShockSurface`

### New Strategies

| Issue | Strategy | Description |
|-------|----------|-------------|
| #21 | CoveredCall | Long stock + short call with spot leg support |
| #20 | Collar | Protective put + covered call combination |
| #25 | ProtectivePut | Long stock + long put protection |

### Multi-Instrument Support (#198)

- New `Leg` enum for unified position types
- `SpotPosition` for underlying asset positions
- `FuturePosition` for futures contracts
- `PerpetualPosition` for perpetual swaps

### Delta-Neutral Adjustments (#155, #187)

- `AdjustmentOptimizer` for portfolio rebalancing
- `PortfolioGreeks` for aggregate risk metrics
- Multiple adjustment strategies support

### Async Feature (#223)

- New `async` feature flag for asynchronous I/O operations
- Async support for `OptionChain` and OHLCV data loading

---

## Improvements

### Error Handling Refactoring

Systematic replacement of `unwrap()` and `expect()` calls with proper error handling:

| Issue | Scope |
|-------|-------|
| #213 | `model/option.rs` |
| #212 | `greeks/equations.rs` |
| #211 | `chains/chain.rs` |
| #225 | All `expect()` calls |
| #226 | `Positive` type safety improvements |
| #227 | Remaining `unwrap()` in `src/model/` |

### Code Quality

| Issue | Description |
|-------|-------------|
| #214 | Resolved 14 TODO/FIXME items in `black_scholes_model.rs` |
| #215 | Resolved 4 TODO/FIXME items in `model/position.rs` |
| #216 | Extracted common strategy logic to shared traits |
| #217 | Reduced unnecessary `clone()` calls |
| #220 | Added error context using `anyhow` at API boundaries |

### Testing

| Issue | Description |
|-------|-------------|
| #221 | Comprehensive benchmarks for critical code paths |
| #222 | Property-based testing with `proptest` |

### Documentation (#224)

- Comprehensive documentation for all metrics modules
- Executable examples for each metric type
- Updated README with temporal metrics documentation

---

## Bug Fixes

| Issue | Description |
|-------|-------------|
| #167 | Static image export hangs on Apple-Silicon macOS with kaleido |
| #176 | Implied volatility calculation improvements |
| #184 | Unit tests for threshold pricing model errors |
| #191 | `build_chain` function chain_size parameter handling |

### Other Fixes

- Division by zero in `premium_weighted_pcr`
- PNG rendering tests gracefully handle CI failures
- Documentation typos corrected

---

## Dependencies Updated (#257)

| Package | Version |
|---------|---------|
| `zip` | 7.0 |
| `plotly` | 0.14 |
| `positive` | 0.3.0 |

---

## Breaking Changes

### Positive Type Migration

The internal `Positive` type has been migrated to the external `positive` crate:

```rust
// Before (v0.13.x)
use optionstratlib::Positive;
use optionstratlib::pos!;

// After (v0.14.2)
use positive::{Positive, pos_or_panic};
```

### Error Handling

Many functions that previously returned values directly now return `Result<T, Error>`:

```rust
// Before
fn calculate_price(&self) -> Decimal;

// After
fn calculate_price(&self) -> Result<Decimal, PricingError>;
```

---

## Migration Guide

### From v0.13.x to v0.14.2

1. **Update Positive imports**:
   ```rust
   // Add to Cargo.toml
   positive = "0.3"
   
   // Update imports
   use positive::{Positive, pos_or_panic};
   ```

2. **Handle new Result types**:
   ```rust
   // Before
   let price = option.calculate_price();
   
   // After
   let price = option.calculate_price()?;
   ```

3. **Use new exotic option pricing**:
   ```rust
   use optionstratlib::pricing::black_scholes;
   
   let option = Options::new(
       OptionType::Asian { averaging_type: AveragingType::Arithmetic },
       // ... other parameters
   );
   let price = black_scholes(&option)?;
   ```

---

## Statistics

- **86 commits** since v0.13.1
- **70+ issues closed**
- **50+ pull requests merged**
- **14 exotic option types** now fully supported
- **7 metrics categories** with curves and surfaces

---

## Contributors

- Joaquín Béjar García (@joaquinbejar)

---

## Full Changelog

See the [GitHub Releases](https://github.com/joaquinbejar/OptionStratLib/releases) page for the complete list of changes.

**Issues Closed**: #20, #21, #25, #38, #72, #73, #75, #77, #78, #83, #84, #85, #89, #91, #96, #97, #104, #105, #108, #113, #116, #119, #120, #124, #126, #127, #128, #129, #130, #131, #134, #135, #136, #137, #138, #139, #140, #141, #142, #155, #167, #176, #184, #187, #191, #198, #211, #212, #213, #214, #215, #216, #217, #218, #219, #220, #221, #222, #223, #224, #225, #226, #227, #235, #236, #237, #238, #239, #240, #241, #242, #243, #244, #245, #246, #247, #248, #257
