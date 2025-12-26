# OptionStratLib Implementation Study v2

## Executive Summary

After reviewing the actual codebase structure, the implementation strategy changes significantly. The library already has robust infrastructure for curves, surfaces, and interpolation that we should leverage rather than recreate.

---

## Existing Infrastructure Analysis

### What Already Exists

```
src/curves/          → Curve types, traits, and visualization (plotters)
src/surfaces/        → Surface types, traits, and visualization
src/geometrics/
  └── interpolation/ → bilinear, cubic, linear, spline implementations
src/metrics/
  └── price/         → volatility_skew.rs (starting point)
src/strategies/
  └── delta_neutral/ → model.rs (base for Issue #187)
src/greeks/          → equations.rs, utils.rs
```

### Key Insight

The architecture follows a clear pattern:
- **Domain types** live in dedicated modules (`curves/`, `surfaces/`)
- **Metrics** is already structured to hold analytical functions by category (`metrics/price/`)
- **Strategies** have their own submodules for complex features (`delta_neutral/`)

---

## Revised Architecture Proposal

### Extend `src/metrics/` Module

```
src/metrics/
├── mod.rs                      # Re-export all metric categories
├── traits.rs                   # NEW: Common metric traits
│
├── price/                      # EXISTING
│   ├── mod.rs
│   └── volatility_skew.rs
│
├── temporal/                   # NEW: Issue #137
│   ├── mod.rs
│   ├── theta.rs                # Theta curves/surfaces
│   ├── charm.rs                # Charm (delta decay)
│   └── color.rs                # Color (gamma decay)
│
├── risk/                       # NEW: Issue #139
│   ├── mod.rs
│   ├── implied_volatility.rs   # IV surface generation
│   ├── risk_reversal.rs        # Risk reversal curve
│   └── dollar_gamma.rs         # Dollar gamma curve
│
├── composite/                  # NEW: Issue #140
│   ├── mod.rs
│   ├── vanna_volga.rs          # Vanna-Volga hedge surface
│   ├── delta_gamma_profile.rs  # Combined exposure analysis
│   └── smile_dynamics.rs       # Smile evolution over time
│
├── liquidity/                  # NEW: Issue #141
│   ├── mod.rs
│   ├── spread.rs               # Bid-ask spread curve
│   ├── volume.rs               # Volume profile curve/surface
│   └── open_interest.rs        # OI distribution curve
│
└── stress/                     # NEW: Issue #142
    ├── mod.rs
    ├── vol_sensitivity.rs      # Vega exposure analysis
    ├── time_decay.rs           # Theta profile surface
    └── price_shock.rs          # Scenario matrix generation
```

### Extend `src/strategies/delta_neutral/` for Issue #187

```
src/strategies/delta_neutral/
├── mod.rs                      # EXISTING: add new exports
├── model.rs                    # EXISTING: DeltaAdjustment enum
├── adjustment.rs               # NEW: Extended adjustment actions
├── portfolio.rs                # NEW: Portfolio-level Greeks aggregation
└── optimizer.rs                # NEW: Multi-leg adjustment optimization
```

### Add Error Types to `src/error/`

```
src/error/
├── metrics.rs                  # EXISTING: extend with new error variants
└── adjustment.rs               # NEW: adjustment-specific errors
```

---

## Integration with Existing Types

### Using Existing Curve Infrastructure

```rust
// src/curves/curve.rs already provides the foundation
// We extend it by implementing traits for our metrics

use crate::curves::{Curve, CurvePoint};
use crate::curves::traits::CurveOperations;

// Example: ThetaCurve generates a standard Curve
pub struct ThetaCurveGenerator {
    // configuration
}

impl ThetaCurveGenerator {
    pub fn generate(&self, chain: &OptionChain) -> Result<Curve, CurveError> {
        let points: Vec<CurvePoint> = chain
            .options_iter()
            .filter_map(|opt| {
                opt.theta().ok().map(|theta| {
                    CurvePoint::new(opt.strike_price, theta)
                })
            })
            .collect();
        
        Curve::from_points(points)
    }
}
```

### Using Existing Surface Infrastructure

```rust
// src/surfaces/surface.rs provides Surface type
use crate::surfaces::{Surface, SurfacePoint};
use crate::surfaces::traits::SurfaceOperations;

pub struct IVSurfaceGenerator {
    // configuration
}

impl IVSurfaceGenerator {
    pub fn generate(&self, chain: &OptionChain) -> Result<Surface, SurfaceError> {
        // Group by expiration, create surface points
        let points: Vec<SurfacePoint> = chain
            .options_iter()
            .map(|opt| {
                SurfacePoint::new(
                    opt.strike_price,                    // x
                    opt.expiration_date.get_years(),     // y  
                    opt.implied_volatility,              // z
                )
            })
            .collect();
        
        Surface::from_points(points)
    }
}
```

### Using Existing Interpolation

```rust
// src/geometrics/interpolation/ already has what we need
use crate::geometrics::interpolation::{
    CubicSplineInterpolator,
    BilinearInterpolator,
    InterpolationMethod,
};

impl Surface {
    pub fn interpolate_at(&self, x: Positive, y: Positive) -> Result<Decimal, InterpolationError> {
        let interpolator = BilinearInterpolator::new(&self.points)?;
        interpolator.interpolate(x, y)
    }
}
```

---

## Detailed Implementation by Issue

### Issue #137: Temporal Metrics

**Files to create:**
- `src/metrics/temporal/mod.rs`
- `src/metrics/temporal/theta.rs`
- `src/metrics/temporal/charm.rs`
- `src/metrics/temporal/color.rs`

**Files to modify:**
- `src/metrics/mod.rs` - add `pub mod temporal;`
- `src/model/option.rs` - add helper methods if needed

```rust
// src/metrics/temporal/mod.rs
mod theta;
mod charm;
mod color;

pub use theta::{ThetaCurveGenerator, ThetaSurfaceGenerator};
pub use charm::{CharmCalculator, CharmCurveGenerator};
pub use color::{ColorCalculator, ColorCurveGenerator};

// src/metrics/temporal/theta.rs
use crate::chains::OptionChain;
use crate::curves::Curve;
use crate::surfaces::Surface;
use crate::error::MetricError;
use crate::greeks::Greeks;

/// Generates theta curve across strikes at fixed expiration
pub struct ThetaCurveGenerator {
    option_style: Option<OptionStyle>,  // Filter calls/puts
}

impl ThetaCurveGenerator {
    pub fn new() -> Self {
        Self { option_style: None }
    }
    
    pub fn with_style(mut self, style: OptionStyle) -> Self {
        self.option_style = Some(style);
        self
    }
    
    pub fn generate(&self, chain: &OptionChain) -> Result<Curve, MetricError> {
        let mut points = Vec::new();
        
        for option_data in chain.options_iter() {
            // Apply filter if set
            if let Some(style) = &self.option_style {
                if option_data.option_style != *style {
                    continue;
                }
            }
            
            // Build Options struct to calculate theta
            let option = option_data.to_option(chain.underlying_price)?;
            let theta = option.theta()?;
            
            points.push((option_data.strike_price, theta));
        }
        
        // Sort by strike
        points.sort_by(|a, b| a.0.cmp(&b.0));
        
        Ok(Curve::from_xy_pairs(points)?)
    }
}

/// Generates theta surface: strike × time → theta
pub struct ThetaSurfaceGenerator {
    time_steps: Vec<Positive>,  // Days to expiration values
}

impl ThetaSurfaceGenerator {
    pub fn new(time_steps: Vec<Positive>) -> Self {
        Self { time_steps }
    }
    
    pub fn with_range(start_days: Positive, end_days: Positive, step: Positive) -> Self {
        let mut time_steps = Vec::new();
        let mut current = start_days;
        while current <= end_days {
            time_steps.push(current);
            current = current + step;
        }
        Self { time_steps }
    }
    
    pub fn generate(&self, chain: &OptionChain) -> Result<Surface, MetricError> {
        let mut surface_points = Vec::new();
        
        for &days in &self.time_steps {
            for option_data in chain.options_iter() {
                // Create option with modified expiration
                let option = option_data
                    .to_option(chain.underlying_price)?
                    .with_expiration(ExpirationDate::Days(days));
                
                let theta = option.theta()?;
                
                surface_points.push((
                    option_data.strike_price,  // x: strike
                    days,                       // y: time
                    theta,                      // z: theta value
                ));
            }
        }
        
        Ok(Surface::from_xyz_points(surface_points)?)
    }
}

// src/metrics/temporal/charm.rs
use crate::model::Options;
use crate::error::MetricError;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

/// Charm = ∂Δ/∂t (rate of change of delta with time)
pub struct CharmCalculator;

impl CharmCalculator {
    /// Calculate charm using finite difference approximation
    /// 
    /// # Arguments
    /// * `option` - The option to analyze
    /// * `dt` - Time step in years (default: 1/365 for 1 day)
    pub fn calculate(option: &Options, dt: Option<Decimal>) -> Result<Decimal, MetricError> {
        let dt = dt.unwrap_or(dec!(0.00274));  // ~1 day in years
        
        let delta_now = option.delta()?;
        
        // Shift time forward (less time to expiry)
        let option_later = option.with_time_shift(-dt)?;
        let delta_later = option_later.delta()?;
        
        // Charm = (delta_later - delta_now) / dt
        Ok((delta_later - delta_now) / dt)
    }
    
    /// Calculate charm for an option (using 1-day default)
    pub fn charm(option: &Options) -> Result<Decimal, MetricError> {
        Self::calculate(option, None)
    }
}

/// Generates charm curve across strikes
pub struct CharmCurveGenerator {
    dt: Decimal,
}

impl CharmCurveGenerator {
    pub fn new() -> Self {
        Self { dt: dec!(0.00274) }
    }
    
    pub fn with_time_step(mut self, dt: Decimal) -> Self {
        self.dt = dt;
        self
    }
    
    pub fn generate(&self, chain: &OptionChain) -> Result<Curve, MetricError> {
        let mut points = Vec::new();
        
        for option_data in chain.options_iter() {
            let option = option_data.to_option(chain.underlying_price)?;
            let charm = CharmCalculator::calculate(&option, Some(self.dt))?;
            points.push((option_data.strike_price, charm));
        }
        
        points.sort_by(|a, b| a.0.cmp(&b.0));
        Ok(Curve::from_xy_pairs(points)?)
    }
}

// src/metrics/temporal/color.rs
/// Color = ∂Γ/∂t (rate of change of gamma with time)
pub struct ColorCalculator;

impl ColorCalculator {
    pub fn calculate(option: &Options, dt: Option<Decimal>) -> Result<Decimal, MetricError> {
        let dt = dt.unwrap_or(dec!(0.00274));
        
        let gamma_now = option.gamma()?;
        let option_later = option.with_time_shift(-dt)?;
        let gamma_later = option_later.gamma()?;
        
        Ok((gamma_later - gamma_now) / dt)
    }
    
    pub fn color(option: &Options) -> Result<Decimal, MetricError> {
        Self::calculate(option, None)
    }
}
```

---

### Issue #139: Risk Metrics

**Files to create:**
- `src/metrics/risk/mod.rs`
- `src/metrics/risk/implied_volatility.rs`
- `src/metrics/risk/risk_reversal.rs`
- `src/metrics/risk/dollar_gamma.rs`

```rust
// src/metrics/risk/implied_volatility.rs
use crate::chains::OptionChain;
use crate::surfaces::Surface;
use crate::error::MetricError;

/// Generates implied volatility surface from option chain
pub struct IVSurfaceGenerator {
    moneyness_centered: bool,  // Use moneyness instead of absolute strike
}

impl IVSurfaceGenerator {
    pub fn new() -> Self {
        Self { moneyness_centered: false }
    }
    
    /// Use log-moneyness (ln(K/S)) instead of absolute strike
    pub fn with_moneyness(mut self) -> Self {
        self.moneyness_centered = true;
        self
    }
    
    pub fn generate(&self, chain: &OptionChain) -> Result<Surface, MetricError> {
        let spot = chain.underlying_price;
        let mut points = Vec::new();
        
        // Group options by expiration to build surface
        let by_expiry = chain.group_by_expiration();
        
        for (expiry, options) in by_expiry {
            let time_to_expiry = expiry.get_years();
            
            for opt in options {
                let x = if self.moneyness_centered {
                    // Log-moneyness: ln(K/S)
                    (opt.strike_price / spot).ln()
                } else {
                    opt.strike_price
                };
                
                points.push((
                    x,
                    Positive::from(time_to_expiry * 365.0),  // days
                    opt.implied_volatility,
                ));
            }
        }
        
        Surface::from_xyz_points(points)
    }
}

// src/metrics/risk/risk_reversal.rs
use crate::chains::OptionChain;
use crate::curves::Curve;
use crate::error::MetricError;
use crate::model::OptionStyle;

/// Risk Reversal = IV(OTM Call at Δ) - IV(OTM Put at same Δ)
/// Measures market skew/sentiment
pub struct RiskReversal;

impl RiskReversal {
    /// Calculate risk reversal at a specific delta level
    pub fn at_delta(chain: &OptionChain, target_delta: Positive) -> Result<Decimal, MetricError> {
        let call = chain.find_option_at_delta(target_delta, OptionStyle::Call)?;
        let put = chain.find_option_at_delta(target_delta, OptionStyle::Put)?;
        
        Ok(call.implied_volatility.to_decimal() - put.implied_volatility.to_decimal())
    }
    
    /// Generate risk reversal curve across standard deltas
    pub fn curve(chain: &OptionChain) -> Result<Curve, MetricError> {
        let deltas = [
            pos!(0.10), pos!(0.15), pos!(0.20), pos!(0.25),
            pos!(0.30), pos!(0.35), pos!(0.40), pos!(0.45),
        ];
        
        let mut points = Vec::new();
        
        for delta in deltas {
            match Self::at_delta(chain, delta) {
                Ok(rr) => points.push((delta, rr)),
                Err(_) => continue,  // Skip if can't find options at this delta
            }
        }
        
        if points.is_empty() {
            return Err(MetricError::InsufficientData("No valid delta points found".into()));
        }
        
        Curve::from_xy_pairs(points)
    }
}

// src/metrics/risk/dollar_gamma.rs
use crate::model::{Options, Positive};
use crate::chains::OptionChain;
use crate::curves::Curve;
use crate::error::MetricError;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

/// Dollar Gamma = Γ × S² / 100
/// Represents P&L for a 1% move in underlying
pub struct DollarGamma;

impl DollarGamma {
    /// Calculate dollar gamma for a single option
    pub fn calculate(option: &Options) -> Result<Decimal, MetricError> {
        let gamma = option.gamma()?;
        let spot = option.underlying_price.to_decimal();
        
        // Dollar gamma formula
        Ok(gamma * spot * spot / dec!(100))
    }
    
    /// Generate dollar gamma curve across strikes
    pub fn curve(chain: &OptionChain) -> Result<Curve, MetricError> {
        let mut points = Vec::new();
        
        for option_data in chain.options_iter() {
            let option = option_data.to_option(chain.underlying_price)?;
            let dg = Self::calculate(&option)?;
            points.push((option_data.strike_price, dg));
        }
        
        points.sort_by(|a, b| a.0.cmp(&b.0));
        Curve::from_xy_pairs(points)
    }
    
    /// Calculate total dollar gamma for a portfolio
    pub fn portfolio_total(positions: &[Position]) -> Result<Decimal, MetricError> {
        let mut total = Decimal::ZERO;
        
        for pos in positions {
            let dg = Self::calculate(&pos.option)?;
            let sign = if pos.option.is_long() { dec!(1) } else { dec!(-1) };
            total += dg * pos.option.quantity.to_decimal() * sign;
        }
        
        Ok(total)
    }
}
```

---

### Issue #140: Composite Metrics

```rust
// src/metrics/composite/delta_gamma_profile.rs
use crate::model::Position;
use crate::curves::Curve;
use crate::surfaces::Surface;
use crate::error::MetricError;

/// Combined delta-gamma exposure analysis
#[derive(Debug, Clone)]
pub struct DeltaGammaPoint {
    pub price: Positive,
    pub delta_exposure: Decimal,
    pub gamma_exposure: Decimal,
    pub dollar_delta: Decimal,
    pub dollar_gamma: Decimal,
}

pub struct DeltaGammaProfile;

impl DeltaGammaProfile {
    /// Calculate profile at a specific underlying price
    pub fn at_price(
        positions: &[Position],
        underlying_price: Positive,
    ) -> Result<DeltaGammaPoint, MetricError> {
        let mut delta_exp = Decimal::ZERO;
        let mut gamma_exp = Decimal::ZERO;
        
        for pos in positions {
            // Recalculate with new underlying price
            let option = pos.option.with_underlying_price(underlying_price);
            let qty = pos.option.quantity.to_decimal();
            let sign = if pos.option.is_long() { dec!(1) } else { dec!(-1) };
            
            delta_exp += option.delta()? * qty * sign;
            gamma_exp += option.gamma()? * qty * sign;
        }
        
        let spot = underlying_price.to_decimal();
        
        Ok(DeltaGammaPoint {
            price: underlying_price,
            delta_exposure: delta_exp,
            gamma_exposure: gamma_exp,
            dollar_delta: delta_exp * spot,
            dollar_gamma: gamma_exp * spot * spot / dec!(100),
        })
    }
    
    /// Generate exposure curve across price range
    pub fn curve(
        positions: &[Position],
        price_range: (Positive, Positive),
        steps: usize,
    ) -> Result<Curve, MetricError> {
        let (min_price, max_price) = price_range;
        let step_size = (max_price - min_price) / Positive::from(steps as f64);
        
        let mut points = Vec::new();
        let mut price = min_price;
        
        for _ in 0..=steps {
            let profile = Self::at_price(positions, price)?;
            points.push((price, profile.delta_exposure));
            price = price + step_size;
        }
        
        Curve::from_xy_pairs(points)
    }
    
    /// Generate surface: price × time → delta exposure
    pub fn surface(
        positions: &[Position],
        price_range: (Positive, Positive),
        time_range: (Positive, Positive),  // days
        price_steps: usize,
        time_steps: usize,
    ) -> Result<Surface, MetricError> {
        let mut points = Vec::new();
        
        let price_step = (price_range.1 - price_range.0) / Positive::from(price_steps as f64);
        let time_step = (time_range.1 - time_range.0) / Positive::from(time_steps as f64);
        
        for t in 0..=time_steps {
            let days = time_range.0 + time_step * Positive::from(t as f64);
            
            // Shift all positions to this time
            let shifted_positions: Vec<Position> = positions
                .iter()
                .filter_map(|p| p.with_days_to_expiry(days).ok())
                .collect();
            
            for p in 0..=price_steps {
                let price = price_range.0 + price_step * Positive::from(p as f64);
                let profile = Self::at_price(&shifted_positions, price)?;
                
                points.push((price, days, profile.delta_exposure));
            }
        }
        
        Surface::from_xyz_points(points)
    }
}

// src/metrics/composite/smile_dynamics.rs
use crate::chains::OptionChain;
use crate::curves::Curve;
use crate::surfaces::Surface;
use crate::error::MetricError;

/// Tracks how the volatility smile evolves over time
#[derive(Debug, Clone)]
pub struct SmileShift {
    pub parallel_shift: Decimal,    // ATM vol change
    pub slope_change: Decimal,      // Skew change  
    pub curvature_change: Decimal,  // Smile curvature change
}

pub struct SmileDynamics;

impl SmileDynamics {
    /// Extract smile parameters from a chain at single expiry
    fn extract_smile_params(chain: &OptionChain) -> Result<SmileParams, MetricError> {
        let atm_vol = chain.atm_implied_volatility()?;
        
        // Use 25-delta to measure skew
        let call_25d = chain.find_option_at_delta(pos!(0.25), OptionStyle::Call)?;
        let put_25d = chain.find_option_at_delta(pos!(0.25), OptionStyle::Put)?;
        
        let skew = call_25d.implied_volatility.to_decimal() 
                 - put_25d.implied_volatility.to_decimal();
        
        // Butterfly for curvature
        let butterfly = (call_25d.implied_volatility.to_decimal() 
                       + put_25d.implied_volatility.to_decimal()) / dec!(2)
                       - atm_vol.to_decimal();
        
        Ok(SmileParams {
            atm_vol,
            skew,
            curvature: butterfly,
        })
    }
    
    /// Compare two snapshots of the smile
    pub fn calculate_shift(
        chain_before: &OptionChain,
        chain_after: &OptionChain,
    ) -> Result<SmileShift, MetricError> {
        let params_before = Self::extract_smile_params(chain_before)?;
        let params_after = Self::extract_smile_params(chain_after)?;
        
        Ok(SmileShift {
            parallel_shift: params_after.atm_vol.to_decimal() - params_before.atm_vol.to_decimal(),
            slope_change: params_after.skew - params_before.skew,
            curvature_change: params_after.curvature - params_before.curvature,
        })
    }
    
    /// Generate smile curve at specific expiry
    pub fn smile_curve(chain: &OptionChain) -> Result<Curve, MetricError> {
        let spot = chain.underlying_price;
        let mut points = Vec::new();
        
        for opt in chain.options_iter() {
            // Use log-moneyness for x-axis
            let moneyness = (opt.strike_price / spot).ln();
            points.push((moneyness, opt.implied_volatility));
        }
        
        points.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        Curve::from_xy_pairs(points)
    }
    
    /// Generate surface: moneyness × expiry → IV
    pub fn surface(chains_by_expiry: &[(ExpirationDate, OptionChain)]) -> Result<Surface, MetricError> {
        let mut points = Vec::new();
        
        for (expiry, chain) in chains_by_expiry {
            let time = expiry.get_years() * dec!(365);
            let spot = chain.underlying_price;
            
            for opt in chain.options_iter() {
                let moneyness = (opt.strike_price / spot).ln();
                points.push((moneyness, time, opt.implied_volatility));
            }
        }
        
        Surface::from_xyz_points(points)
    }
}

#[derive(Debug, Clone)]
struct SmileParams {
    atm_vol: Positive,
    skew: Decimal,
    curvature: Decimal,
}
```

---

### Issue #141: Liquidity Metrics

```rust
// src/metrics/liquidity/mod.rs

/// Trait for market data providers (bid/ask, volume, OI)
pub trait MarketDataSource {
    fn bid(&self, option: &OptionData) -> Option<Positive>;
    fn ask(&self, option: &OptionData) -> Option<Positive>;
    fn volume(&self, option: &OptionData) -> Option<u64>;
    fn open_interest(&self, option: &OptionData) -> Option<u64>;
}

// src/metrics/liquidity/spread.rs
use crate::chains::OptionChain;
use crate::curves::Curve;
use crate::error::MetricError;

pub struct BidAskSpread<D: MarketDataSource> {
    data_source: D,
}

impl<D: MarketDataSource> BidAskSpread<D> {
    pub fn new(data_source: D) -> Self {
        Self { data_source }
    }
    
    /// Absolute spread in price terms
    pub fn absolute(&self, option: &OptionData) -> Option<Positive> {
        let bid = self.data_source.bid(option)?;
        let ask = self.data_source.ask(option)?;
        Some(ask - bid)
    }
    
    /// Relative spread as percentage of mid price
    pub fn relative(&self, option: &OptionData) -> Option<Positive> {
        let bid = self.data_source.bid(option)?;
        let ask = self.data_source.ask(option)?;
        let mid = (bid + ask) / pos!(2.0);
        
        if mid > Positive::ZERO {
            Some((ask - bid) / mid)
        } else {
            None
        }
    }
    
    /// Generate spread curve by strike
    pub fn curve(&self, chain: &OptionChain) -> Result<Curve, MetricError> {
        let mut points = Vec::new();
        
        for opt in chain.options_iter() {
            if let Some(spread) = self.relative(opt) {
                points.push((opt.strike_price, spread));
            }
        }
        
        if points.is_empty() {
            return Err(MetricError::InsufficientData("No spread data available".into()));
        }
        
        points.sort_by(|a, b| a.0.cmp(&b.0));
        Curve::from_xy_pairs(points)
    }
}

// src/metrics/liquidity/open_interest.rs
pub struct OpenInterestDistribution<D: MarketDataSource> {
    data_source: D,
}

impl<D: MarketDataSource> OpenInterestDistribution<D> {
    pub fn new(data_source: D) -> Self {
        Self { data_source }
    }
    
    /// Generate OI curve by strike
    pub fn curve(&self, chain: &OptionChain) -> Result<Curve, MetricError> {
        let mut points = Vec::new();
        
        for opt in chain.options_iter() {
            if let Some(oi) = self.data_source.open_interest(opt) {
                points.push((opt.strike_price, Decimal::from(oi)));
            }
        }
        
        points.sort_by(|a, b| a.0.cmp(&b.0));
        Curve::from_xy_pairs(points)
    }
    
    /// Find strike with maximum open interest (potential support/resistance)
    pub fn max_pain_strike(&self, chain: &OptionChain) -> Result<Positive, MetricError> {
        let curve = self.curve(chain)?;
        curve.max_point().map(|(strike, _)| strike)
    }
    
    /// Put/Call OI ratio by strike
    pub fn put_call_ratio(&self, chain: &OptionChain) -> Result<Curve, MetricError> {
        let mut ratios = Vec::new();
        
        let by_strike = chain.group_by_strike();
        
        for (strike, options) in by_strike {
            let call_oi: u64 = options.iter()
                .filter(|o| o.option_style == OptionStyle::Call)
                .filter_map(|o| self.data_source.open_interest(o))
                .sum();
                
            let put_oi: u64 = options.iter()
                .filter(|o| o.option_style == OptionStyle::Put)
                .filter_map(|o| self.data_source.open_interest(o))
                .sum();
            
            if call_oi > 0 {
                let ratio = Decimal::from(put_oi) / Decimal::from(call_oi);
                ratios.push((strike, ratio));
            }
        }
        
        Curve::from_xy_pairs(ratios)
    }
}
```

---

### Issue #142: Stress Metrics

```rust
// src/metrics/stress/vol_sensitivity.rs
use crate::model::Position;
use crate::curves::Curve;
use crate::surfaces::Surface;
use crate::error::MetricError;

/// Analyzes portfolio sensitivity to volatility changes
pub struct VolatilitySensitivity;

impl VolatilitySensitivity {
    /// Calculate P&L for a given volatility shock
    pub fn pnl_for_shock(positions: &[Position], vol_shock: Decimal) -> Result<Decimal, MetricError> {
        let mut total_pnl = Decimal::ZERO;
        
        for pos in positions {
            let vega = pos.option.vega()?;
            let vomma = pos.option.vomma()?;
            let qty = pos.option.quantity.to_decimal();
            let sign = if pos.option.is_long() { dec!(1) } else { dec!(-1) };
            
            // Taylor expansion: Vega * Δσ + 0.5 * Vomma * Δσ²
            let pnl = (vega * vol_shock + dec!(0.5) * vomma * vol_shock * vol_shock) * qty * sign;
            total_pnl += pnl;
        }
        
        Ok(total_pnl)
    }
    
    /// Generate sensitivity curve across vol shocks
    pub fn curve(
        positions: &[Position],
        shock_range: (Decimal, Decimal),
        steps: usize,
    ) -> Result<Curve, MetricError> {
        let (min_shock, max_shock) = shock_range;
        let step_size = (max_shock - min_shock) / Decimal::from(steps);
        
        let mut points = Vec::new();
        
        for i in 0..=steps {
            let shock = min_shock + step_size * Decimal::from(i);
            let pnl = Self::pnl_for_shock(positions, shock)?;
            points.push((shock, pnl));
        }
        
        Curve::from_xy_pairs(points)
    }
}

// src/metrics/stress/price_shock.rs
use crate::model::Position;
use crate::error::MetricError;

/// Scenario matrix for combined price and volatility shocks
#[derive(Debug, Clone)]
pub struct ScenarioMatrix {
    pub price_shocks: Vec<Decimal>,   // e.g., [-20%, -10%, 0%, +10%, +20%]
    pub vol_shocks: Vec<Decimal>,     // e.g., [-25%, 0%, +25%, +50%]
    pub pnl_matrix: Vec<Vec<Decimal>>, // price_shock × vol_shock → P&L
}

impl ScenarioMatrix {
    /// Pretty print the matrix
    pub fn display(&self) -> String {
        let mut output = String::new();
        
        // Header row
        output.push_str("Price\\Vol\t");
        for vs in &self.vol_shocks {
            output.push_str(&format!("{:.0}%\t", vs * dec!(100)));
        }
        output.push('\n');
        
        // Data rows
        for (i, ps) in self.price_shocks.iter().enumerate() {
            output.push_str(&format!("{:.0}%\t", ps * dec!(100)));
            for pnl in &self.pnl_matrix[i] {
                output.push_str(&format!("{:.2}\t", pnl));
            }
            output.push('\n');
        }
        
        output
    }
}

pub struct PriceShockAnalysis;

impl PriceShockAnalysis {
    /// Generate scenario matrix
    pub fn scenario_matrix(
        positions: &[Position],
        price_shocks: &[Decimal],
        vol_shocks: &[Decimal],
    ) -> Result<ScenarioMatrix, MetricError> {
        let mut matrix = Vec::new();
        
        for &price_shock in price_shocks {
            let mut row = Vec::new();
            
            for &vol_shock in vol_shocks {
                let pnl = Self::combined_shock_pnl(positions, price_shock, vol_shock)?;
                row.push(pnl);
            }
            
            matrix.push(row);
        }
        
        Ok(ScenarioMatrix {
            price_shocks: price_shocks.to_vec(),
            vol_shocks: vol_shocks.to_vec(),
            pnl_matrix: matrix,
        })
    }
    
    /// Calculate P&L for combined shocks using Taylor expansion
    fn combined_shock_pnl(
        positions: &[Position],
        price_shock: Decimal,
        vol_shock: Decimal,
    ) -> Result<Decimal, MetricError> {
        let mut total_pnl = Decimal::ZERO;
        
        for pos in positions {
            let spot = pos.option.underlying_price.to_decimal();
            let price_move = spot * price_shock;
            
            let delta = pos.option.delta()?;
            let gamma = pos.option.gamma()?;
            let vega = pos.option.vega()?;
            let vanna = pos.option.vanna()?;
            
            let qty = pos.option.quantity.to_decimal();
            let sign = if pos.option.is_long() { dec!(1) } else { dec!(-1) };
            
            // Full Taylor expansion with cross-term
            let pnl = (
                delta * price_move
                + dec!(0.5) * gamma * price_move * price_move
                + vega * vol_shock
                + vanna * price_move * vol_shock
            ) * qty * sign;
            
            total_pnl += pnl;
        }
        
        Ok(total_pnl)
    }
    
    /// Standard shock matrix with common values
    pub fn standard_matrix(positions: &[Position]) -> Result<ScenarioMatrix, MetricError> {
        let price_shocks = vec![
            dec!(-0.20), dec!(-0.10), dec!(-0.05), 
            dec!(0.0), 
            dec!(0.05), dec!(0.10), dec!(0.20)
        ];
        let vol_shocks = vec![
            dec!(-0.25), dec!(0.0), dec!(0.25), dec!(0.50), dec!(1.0)
        ];
        
        Self::scenario_matrix(positions, &price_shocks, &vol_shocks)
    }
}
```

---

### Issue #187: Extended Delta Adjustment

**Files to create/modify:**
- `src/strategies/delta_neutral/adjustment.rs` (NEW)
- `src/strategies/delta_neutral/portfolio.rs` (NEW)
- `src/strategies/delta_neutral/optimizer.rs` (NEW)
- `src/strategies/delta_neutral/mod.rs` (MODIFY)
- `src/strategies/delta_neutral/model.rs` (MODIFY)

```rust
// src/strategies/delta_neutral/adjustment.rs

/// Extended adjustment actions beyond quantity modification
#[derive(Debug, Clone)]
pub enum AdjustmentAction {
    /// Modify quantity of existing leg (current behavior)
    ModifyQuantity {
        leg_index: usize,
        new_quantity: Positive,
    },
    
    /// Add a new option leg to the position
    AddLeg {
        option: Options,
        side: Side,
        quantity: Positive,
    },
    
    /// Close/remove an existing leg
    CloseLeg {
        leg_index: usize,
    },
    
    /// Roll to different strike (close old, open new)
    RollStrike {
        leg_index: usize,
        new_strike: Positive,
        quantity: Positive,
    },
    
    /// Roll to different expiration
    RollExpiration {
        leg_index: usize,
        new_expiration: ExpirationDate,
        quantity: Positive,
    },
    
    /// Add underlying position for delta hedge
    AddUnderlying {
        quantity: Decimal,  // Negative for short
    },
}

/// Configuration for adjustment behavior
#[derive(Debug, Clone)]
pub struct AdjustmentConfig {
    /// Allow adding new legs
    pub allow_new_legs: bool,
    
    /// Allow using underlying for hedging
    pub allow_underlying: bool,
    
    /// Maximum number of new legs to add
    pub max_new_legs: Option<usize>,
    
    /// Allowed option styles for new legs
    pub allowed_styles: Vec<OptionStyle>,
    
    /// Strike range for new legs (relative to ATM)
    pub strike_range: Option<(Positive, Positive)>,
    
    /// Maximum cost for adjustments
    pub max_cost: Option<Positive>,
    
    /// Minimum option liquidity (open interest)
    pub min_liquidity: Option<u64>,
}

impl Default for AdjustmentConfig {
    fn default() -> Self {
        Self {
            allow_new_legs: true,
            allow_underlying: false,
            max_new_legs: Some(2),
            allowed_styles: vec![OptionStyle::Call, OptionStyle::Put],
            strike_range: None,
            max_cost: None,
            min_liquidity: None,
        }
    }
}

/// Result of adjustment calculation
#[derive(Debug, Clone)]
pub struct AdjustmentPlan {
    /// Actions to execute
    pub actions: Vec<AdjustmentAction>,
    
    /// Estimated cost of adjustments
    pub estimated_cost: Decimal,
    
    /// Greeks after adjustment
    pub resulting_greeks: PortfolioGreeks,
    
    /// Residual delta after adjustment
    pub residual_delta: Decimal,
    
    /// Quality score (lower is better)
    pub quality_score: Decimal,
}

// src/strategies/delta_neutral/portfolio.rs

/// Aggregated Greeks at portfolio level
#[derive(Debug, Clone, Default)]
pub struct PortfolioGreeks {
    pub delta: Decimal,
    pub gamma: Decimal,
    pub theta: Decimal,
    pub vega: Decimal,
    pub vanna: Decimal,
    pub vomma: Decimal,
    pub rho: Decimal,
}

impl PortfolioGreeks {
    /// Calculate from a set of positions
    pub fn from_positions(positions: &[Position]) -> Result<Self, GreekError> {
        let mut greeks = Self::default();
        
        for pos in positions {
            let qty = pos.option.quantity.to_decimal();
            let sign = if pos.option.is_long() { dec!(1) } else { dec!(-1) };
            let mult = qty * sign;
            
            greeks.delta += pos.option.delta()? * mult;
            greeks.gamma += pos.option.gamma()? * mult;
            greeks.theta += pos.option.theta()? * mult;
            greeks.vega += pos.option.vega()? * mult;
            greeks.vanna += pos.option.vanna()? * mult;
            greeks.vomma += pos.option.vomma()? * mult;
            greeks.rho += pos.option.rho()? * mult;
        }
        
        Ok(greeks)
    }
    
    /// Check if approximately delta neutral
    pub fn is_delta_neutral(&self, tolerance: Decimal) -> bool {
        self.delta.abs() <= tolerance
    }
    
    /// Check if approximately gamma neutral
    pub fn is_gamma_neutral(&self, tolerance: Decimal) -> bool {
        self.gamma.abs() <= tolerance
    }
}

/// Target Greeks for adjustment
#[derive(Debug, Clone, Default)]
pub struct AdjustmentTarget {
    pub delta: Option<Decimal>,
    pub gamma: Option<Decimal>,
    pub vega: Option<Decimal>,
    pub theta: Option<Decimal>,
}

impl AdjustmentTarget {
    pub fn delta_neutral() -> Self {
        Self {
            delta: Some(Decimal::ZERO),
            ..Default::default()
        }
    }
    
    pub fn delta_gamma_neutral() -> Self {
        Self {
            delta: Some(Decimal::ZERO),
            gamma: Some(Decimal::ZERO),
            ..Default::default()
        }
    }
}

// src/strategies/delta_neutral/optimizer.rs

use crate::chains::OptionChain;

/// Portfolio-level adjustment optimizer
pub struct AdjustmentOptimizer<'a> {
    positions: &'a [Position],
    chain: &'a OptionChain,
    config: AdjustmentConfig,
    target: AdjustmentTarget,
}

impl<'a> AdjustmentOptimizer<'a> {
    pub fn new(
        positions: &'a [Position],
        chain: &'a OptionChain,
        config: AdjustmentConfig,
        target: AdjustmentTarget,
    ) -> Self {
        Self { positions, chain, config, target }
    }
    
    /// Calculate optimal adjustment plan
    pub fn optimize(&self) -> Result<AdjustmentPlan, AdjustmentError> {
        let current_greeks = PortfolioGreeks::from_positions(self.positions)?;
        
        // Calculate gaps to target
        let delta_gap = self.target.delta
            .map(|t| t - current_greeks.delta)
            .unwrap_or(Decimal::ZERO);
        
        let gamma_gap = self.target.gamma
            .map(|t| t - current_greeks.gamma);
        
        // Try different adjustment strategies
        let mut best_plan: Option<AdjustmentPlan> = None;
        
        // Strategy 1: Adjust existing legs only
        if let Ok(plan) = self.optimize_existing_legs(delta_gap, gamma_gap) {
            best_plan = Some(plan);
        }
        
        // Strategy 2: Add new legs
        if self.config.allow_new_legs {
            if let Ok(plan) = self.optimize_with_new_legs(delta_gap, gamma_gap) {
                if best_plan.is_none() || plan.quality_score < best_plan.as_ref().unwrap().quality_score {
                    best_plan = Some(plan);
                }
            }
        }
        
        // Strategy 3: Use underlying
        if self.config.allow_underlying && gamma_gap.is_none() {
            if let Ok(plan) = self.optimize_with_underlying(delta_gap) {
                if best_plan.is_none() || plan.quality_score < best_plan.as_ref().unwrap().quality_score {
                    best_plan = Some(plan);
                }
            }
        }
        
        best_plan.ok_or(AdjustmentError::NoViablePlan)
    }
    
    /// Optimize by adjusting existing leg quantities
    fn optimize_existing_legs(
        &self,
        delta_gap: Decimal,
        gamma_gap: Option<Decimal>,
    ) -> Result<AdjustmentPlan, AdjustmentError> {
        let mut actions = Vec::new();
        let mut remaining_delta = delta_gap;
        
        // Sort legs by delta contribution
        let mut legs_with_delta: Vec<(usize, Decimal)> = self.positions
            .iter()
            .enumerate()
            .filter_map(|(i, p)| {
                p.option.delta().ok().map(|d| {
                    let sign = if p.option.is_long() { dec!(1) } else { dec!(-1) };
                    (i, d * sign)
                })
            })
            .collect();
        
        legs_with_delta.sort_by(|a, b| b.1.abs().partial_cmp(&a.1.abs()).unwrap());
        
        // Greedily adjust quantities
        for (idx, leg_delta) in legs_with_delta {
            if remaining_delta.abs() < dec!(0.01) {
                break;
            }
            
            if leg_delta.abs() < dec!(0.001) {
                continue;
            }
            
            let current_qty = self.positions[idx].option.quantity.to_decimal();
            let adjustment_qty = remaining_delta / leg_delta;
            
            // Can't adjust more than current quantity for shorts
            let new_qty = current_qty + adjustment_qty;
            if new_qty > Decimal::ZERO {
                actions.push(AdjustmentAction::ModifyQuantity {
                    leg_index: idx,
                    new_quantity: Positive::try_from(new_qty)?,
                });
                remaining_delta -= adjustment_qty * leg_delta;
            }
        }
        
        self.build_plan(actions, remaining_delta)
    }
    
    /// Optimize by adding new option legs
    fn optimize_with_new_legs(
        &self,
        delta_gap: Decimal,
        gamma_gap: Option<Decimal>,
    ) -> Result<AdjustmentPlan, AdjustmentError> {
        let mut actions = Vec::new();
        let atm_strike = self.positions
            .first()
            .map(|p| p.option.underlying_price)
            .ok_or(AdjustmentError::NoPositions)?;
        
        // Find best option to add based on delta gap
        let candidates = self.find_candidate_options(delta_gap)?;
        
        if let Some((option, quantity)) = candidates.first() {
            let side = if delta_gap > Decimal::ZERO { Side::Long } else { Side::Short };
            
            actions.push(AdjustmentAction::AddLeg {
                option: option.clone(),
                side,
                quantity: *quantity,
            });
        }
        
        let residual = self.calculate_residual(&actions, delta_gap)?;
        self.build_plan(actions, residual)
    }
    
    /// Optimize using underlying shares
    fn optimize_with_underlying(&self, delta_gap: Decimal) -> Result<AdjustmentPlan, AdjustmentError> {
        // Each share has delta = 1
        let shares_needed = delta_gap;
        
        let actions = vec![AdjustmentAction::AddUnderlying {
            quantity: shares_needed,
        }];
        
        self.build_plan(actions, Decimal::ZERO)
    }
    
    /// Find candidate options for delta adjustment
    fn find_candidate_options(&self, delta_gap: Decimal) -> Result<Vec<(Options, Positive)>, AdjustmentError> {
        let mut candidates = Vec::new();
        let target_delta_sign = delta_gap.signum();
        
        for opt_data in self.chain.options_iter() {
            // Filter by style
            if !self.config.allowed_styles.contains(&opt_data.option_style) {
                continue;
            }
            
            // Filter by liquidity
            if let Some(min_oi) = self.config.min_liquidity {
                if opt_data.open_interest.unwrap_or(0) < min_oi {
                    continue;
                }
            }
            
            let option = opt_data.to_option(self.chain.underlying_price)?;
            let option_delta = option.delta()?;
            
            // Check if this option helps reduce the gap
            if option_delta.signum() == target_delta_sign {
                let quantity = (delta_gap.abs() / option_delta.abs()).min(dec!(100));
                if let Ok(qty) = Positive::try_from(quantity) {
                    candidates.push((option, qty));
                }
            }
        }
        
        // Sort by efficiency (delta per dollar cost)
        candidates.sort_by(|a, b| {
            let eff_a = a.0.delta().unwrap_or(Decimal::ZERO) / a.0.calculate_price_black_scholes().unwrap_or(dec!(1));
            let eff_b = b.0.delta().unwrap_or(Decimal::ZERO) / b.0.calculate_price_black_scholes().unwrap_or(dec!(1));
            eff_b.abs().partial_cmp(&eff_a.abs()).unwrap()
        });
        
        Ok(candidates)
    }
    
    fn calculate_residual(&self, actions: &[AdjustmentAction], original_gap: Decimal) -> Result<Decimal, AdjustmentError> {
        let mut delta_change = Decimal::ZERO;
        
        for action in actions {
            match action {
                AdjustmentAction::AddLeg { option, side, quantity } => {
                    let d = option.delta()?;
                    let sign = if *side == Side::Long { dec!(1) } else { dec!(-1) };
                    delta_change += d * quantity.to_decimal() * sign;
                }
                AdjustmentAction::AddUnderlying { quantity } => {
                    delta_change += *quantity;
                }
                AdjustmentAction::ModifyQuantity { leg_index, new_quantity } => {
                    let old_qty = self.positions[*leg_index].option.quantity.to_decimal();
                    let d = self.positions[*leg_index].option.delta()?;
                    let sign = if self.positions[*leg_index].option.is_long() { dec!(1) } else { dec!(-1) };
                    delta_change += d * (new_quantity.to_decimal() - old_qty) * sign;
                }
                _ => {}
            }
        }
        
        Ok(original_gap - delta_change)
    }
    
    fn build_plan(&self, actions: Vec<AdjustmentAction>, residual_delta: Decimal) -> Result<AdjustmentPlan, AdjustmentError> {
        let cost = self.estimate_cost(&actions)?;
        
        // Validate cost constraint
        if let Some(max_cost) = &self.config.max_cost {
            if cost > max_cost.to_decimal() {
                return Err(AdjustmentError::CostExceeded);
            }
        }
        
        // Apply actions to get resulting Greeks
        let new_positions = self.apply_actions_preview(&actions)?;
        let resulting_greeks = PortfolioGreeks::from_positions(&new_positions)?;
        
        // Quality score: lower is better
        let quality = residual_delta.abs() + cost * dec!(0.01);
        
        Ok(AdjustmentPlan {
            actions,
            estimated_cost: cost,
            resulting_greeks,
            residual_delta,
            quality_score: quality,
        })
    }
    
    fn estimate_cost(&self, actions: &[AdjustmentAction]) -> Result<Decimal, AdjustmentError> {
        let mut cost = Decimal::ZERO;
        
        for action in actions {
            match action {
                AdjustmentAction::AddLeg { option, quantity, .. } => {
                    let price = option.calculate_price_black_scholes()?;
                    cost += price * quantity.to_decimal();
                }
                AdjustmentAction::AddUnderlying { quantity } => {
                    let spot = self.positions.first()
                        .map(|p| p.option.underlying_price.to_decimal())
                        .unwrap_or(Decimal::ZERO);
                    cost += (spot * quantity).abs();
                }
                _ => {}
            }
        }
        
        Ok(cost)
    }
    
    fn apply_actions_preview(&self, actions: &[AdjustmentAction]) -> Result<Vec<Position>, AdjustmentError> {
        let mut positions = self.positions.to_vec();
        
        for action in actions {
            match action {
                AdjustmentAction::ModifyQuantity { leg_index, new_quantity } => {
                    if let Some(pos) = positions.get_mut(*leg_index) {
                        pos.option.quantity = *new_quantity;
                    }
                }
                AdjustmentAction::AddLeg { option, quantity, .. } => {
                    let mut new_option = option.clone();
                    new_option.quantity = *quantity;
                    positions.push(Position::new(
                        new_option,
                        Positive::ZERO,  // Will be filled at execution
                        Utc::now(),
                        Positive::ZERO,
                        Positive::ZERO,
                        None,
                        None,
                    ));
                }
                AdjustmentAction::CloseLeg { leg_index } => {
                    if *leg_index < positions.len() {
                        positions.remove(*leg_index);
                    }
                }
                _ => {}
            }
        }
        
        Ok(positions)
    }
}
```

---

## Implementation Roadmap

### Phase 1: Foundation (1 week)
1. Create `src/metrics/traits.rs` with common interfaces
2. Update `src/metrics/mod.rs` to organize submodules
3. Add helper methods to `Options` and `OptionChain` if needed
4. Add new error variants to `src/error/metrics.rs`

### Phase 2: Temporal Metrics - #137 (1 week)
1. Implement `ThetaCurveGenerator` and `ThetaSurfaceGenerator`
2. Implement `CharmCalculator` with curve generation
3. Implement `ColorCalculator` with curve generation
4. Add tests and examples

### Phase 3: Risk Metrics - #139 (1 week)
1. Implement `IVSurfaceGenerator`
2. Implement `RiskReversal` with curve
3. Implement `DollarGamma` with curve
4. Add chain helper for delta-based lookups

### Phase 4: Composite Metrics - #140 (1.5 weeks)
1. Implement `DeltaGammaProfile`
2. Implement `SmileDynamics`
3. Implement `VannaVolgaHedge` (more complex)

### Phase 5: Liquidity Metrics - #141 (1 week)
1. Define `MarketDataSource` trait
2. Implement `BidAskSpread`
3. Implement `OpenInterestDistribution`
4. Implement `VolumeProfile`

### Phase 6: Stress Metrics - #142 (1 week)
1. Implement `VolatilitySensitivity`
2. Implement `PriceShockAnalysis` with scenario matrix
3. Implement `TimeDecayProfile`

### Phase 7: Delta Adjustment - #187 (1.5 weeks)
1. Create new adjustment types
2. Implement `PortfolioGreeks`
3. Implement `AdjustmentOptimizer`
4. Integrate with existing `delta_neutral` module
5. Add comprehensive tests

---

## Dependencies Between Issues

```
                    ┌─────────────────┐
                    │  #137 Temporal  │
                    └────────┬────────┘
                             │
         ┌───────────────────┼───────────────────┐
         │                   │                   │
         ▼                   ▼                   ▼
┌─────────────────┐ ┌─────────────────┐ ┌─────────────────┐
│   #139 Risk     │ │ #142 Stress    │ │   #141 Liquidity│
└────────┬────────┘ └────────┬────────┘ └─────────────────┘
         │                   │
         └─────────┬─────────┘
                   │
                   ▼
         ┌─────────────────┐
         │ #140 Composite  │
         └─────────────────┘

         ┌─────────────────┐
         │ #187 Adjustment │  (Independent track)
         └─────────────────┘
```

Issue #187 can be developed in parallel since it extends `delta_neutral/` rather than the metrics system.

---

## Testing Strategy

### Unit Tests Per Module

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::tests::create_test_chain;
    
    #[test]
    fn test_theta_curve_generation() {
        let chain = create_test_chain();
        let generator = ThetaCurveGenerator::new();
        
        let curve = generator.generate(&chain).unwrap();
        
        assert!(!curve.points.is_empty());
        // Theta should be negative for long options
        assert!(curve.points.iter().all(|(_, theta)| *theta < Decimal::ZERO));
    }
    
    #[test]
    fn test_charm_approximation_accuracy() {
        let option = create_test_option();
        let charm = CharmCalculator::charm(&option).unwrap();
        
        // Numerical verification
        let dt = dec!(0.00274);
        let delta_t0 = option.delta().unwrap();
        let delta_t1 = option.with_time_shift(-dt).unwrap().delta().unwrap();
        let expected = (delta_t1 - delta_t0) / dt;
        
        assert!((charm - expected).abs() < dec!(0.0001));
    }
}
```

### Integration Tests

```rust
// tests/integration/metrics_integration.rs

#[test]
fn test_full_metrics_pipeline() {
    let chain = load_test_chain("SPY_2024_01.json");
    
    // Generate all temporal curves
    let theta = ThetaCurveGenerator::new().generate(&chain).unwrap();
    let charm = CharmCurveGenerator::new().generate(&chain).unwrap();
    
    // Generate IV surface
    let iv_surface = IVSurfaceGenerator::new().generate(&chain).unwrap();
    
    // Verify consistency
    assert_eq!(theta.points.len(), charm.points.len());
}
```

---

## Visualization Integration

The existing `visualization/` module with plotly support can be extended:

```rust
// Example: Add to visualization/utils.rs or create visualization/metrics.rs

impl Curve {
    #[cfg(feature = "plotly")]
    pub fn to_plotly_trace(&self, name: &str) -> Box<Scatter<f64, f64>> {
        let x: Vec<f64> = self.points.iter().map(|(x, _)| x.to_f64()).collect();
        let y: Vec<f64> = self.points.iter().map(|(_, y)| y.to_f64()).collect();
        
        Scatter::new(x, y).name(name)
    }
}

impl Surface {
    #[cfg(feature = "plotly")]
    pub fn to_plotly_surface(&self, name: &str) -> Box<Surface3D<f64, f64, f64>> {
        // Transform points to plotly format
        // ...
    }
}
```

---

## Summary

This revised implementation plan integrates with your existing architecture:

1. **Metrics go in `src/metrics/`** - expanding the existing structure
2. **Reuse `Curve` and `Surface`** types from existing modules
3. **Reuse interpolation** from `src/geometrics/interpolation/`
4. **Delta adjustment extends** `src/strategies/delta_neutral/`

Estimated total: **8-10 weeks** for one developer, with #187 parallelizable with the metrics track.
