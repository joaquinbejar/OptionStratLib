/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 24/12/25
******************************************************************************/

//! # Stress Metrics Module
//!
//! Provides stress-testing metrics for options analysis, helping evaluate how
//! options behave under extreme market conditions such as large moves in
//! volatility, rapid time decay, or sudden price shocks.
//!
//! ## Core Features
//!
//! ### Volatility Sensitivity
//!
//! Analyzes portfolio sensitivity to volatility changes using Taylor expansion:
//! - First-order effect: Vega × Δσ
//! - Second-order effect: 0.5 × Vomma × Δσ²
//!
//! - **Curve representation by strike**: Shows vega exposure at each strike
//! - **Surface representation (price vs volatility)**: P&L across price-vol space
//!
//! ### Time Decay Profile
//!
//! Tracks how option value decays as time passes:
//! - Theta decay acceleration near expiration
//! - Time value erosion patterns
//!
//! - **Curve representation by strike**: Theta at each strike
//! - **Surface representation (price vs time)**: Value decay over time
//!
//! ### Price Shock Impact
//!
//! Evaluates impact of sudden price movements:
//! - Delta effect: First-order price sensitivity
//! - Gamma effect: Second-order price sensitivity
//! - Cross-effects with volatility (Vanna)
//!
//! - **Curve representation by strike**: P&L for price shock at each strike
//! - **Surface representation (price vs volatility)**: Combined shock scenarios
//!
//! ## Usage Examples
//!
//! ### Volatility Sensitivity
//!
//! ```rust
//! use std::collections::BTreeSet;
//! use rust_decimal::Decimal;
//! use optionstratlib::curves::Curve;
//! use optionstratlib::error::{CurveError, SurfaceError};
//! use optionstratlib::surfaces::Surface;
//! use optionstratlib::metrics::{VolatilitySensitivityCurve, VolatilitySensitivitySurface};
//! use positive::Positive;
//!
//! struct MyVolSensitivity;
//!
//! impl VolatilitySensitivityCurve for MyVolSensitivity {
//!     fn volatility_sensitivity_curve(&self) -> Result<Curve, CurveError> {
//!         // Custom logic to compute vega exposure by strike
//!         Ok(Curve { points: BTreeSet::new(), x_range: (Decimal::ZERO, Decimal::ZERO) })
//!     }
//! }
//!
//! impl VolatilitySensitivitySurface for MyVolSensitivity {
//!     fn volatility_sensitivity_surface(
//!         &self,
//!         _price_range: (Positive, Positive),
//!         _vol_range: (Positive, Positive),
//!         _price_steps: usize,
//!         _vol_steps: usize,
//!     ) -> Result<Surface, SurfaceError> {
//!         // Custom logic to compute P&L surface across price-vol space
//!         Ok(Surface::new(BTreeSet::new()))
//!     }
//! }
//! ```
//!
//! ### Time Decay Profile
//!
//! ```rust
//! use std::collections::BTreeSet;
//! use rust_decimal::Decimal;
//! use optionstratlib::curves::Curve;
//! use optionstratlib::error::{CurveError, SurfaceError};
//! use optionstratlib::surfaces::Surface;
//! use optionstratlib::metrics::{TimeDecayCurve, TimeDecaySurface};
//! use positive::Positive;
//!
//! struct MyTimeDecay;
//!
//! impl TimeDecayCurve for MyTimeDecay {
//!     fn time_decay_curve(&self) -> Result<Curve, CurveError> {
//!         // Custom logic to compute theta by strike
//!         Ok(Curve { points: BTreeSet::new(), x_range: (Decimal::ZERO, Decimal::ZERO) })
//!     }
//! }
//!
//! impl TimeDecaySurface for MyTimeDecay {
//!     fn time_decay_surface(
//!         &self,
//!         _price_range: (Positive, Positive),
//!         _days_to_expiry: Vec<Positive>,
//!         _price_steps: usize,
//!     ) -> Result<Surface, SurfaceError> {
//!         // Custom logic to compute value decay surface
//!         Ok(Surface::new(BTreeSet::new()))
//!     }
//! }
//! ```
//!
//! ### Price Shock Impact
//!
//! ```rust
//! use std::collections::BTreeSet;
//! use rust_decimal::Decimal;
//! use optionstratlib::curves::Curve;
//! use optionstratlib::error::{CurveError, SurfaceError};
//! use optionstratlib::surfaces::Surface;
//! use optionstratlib::metrics::{PriceShockCurve, PriceShockSurface};
//! use positive::Positive;
//!
//! struct MyPriceShock;
//!
//! impl PriceShockCurve for MyPriceShock {
//!     fn price_shock_curve(&self, _shock_pct: Decimal) -> Result<Curve, CurveError> {
//!         // Custom logic to compute P&L impact by strike
//!         Ok(Curve { points: BTreeSet::new(), x_range: (Decimal::ZERO, Decimal::ZERO) })
//!     }
//! }
//!
//! impl PriceShockSurface for MyPriceShock {
//!     fn price_shock_surface(
//!         &self,
//!         _price_range: (Positive, Positive),
//!         _vol_range: (Positive, Positive),
//!         _price_steps: usize,
//!         _vol_steps: usize,
//!     ) -> Result<Surface, SurfaceError> {
//!         // Custom logic to compute shock impact surface
//!         Ok(Surface::new(BTreeSet::new()))
//!     }
//! }
//! ```

pub mod price_shock;
pub mod time_decay;
pub mod volatility_sensitivity;

pub use price_shock::{PriceShockCurve, PriceShockSurface};
pub use time_decay::{TimeDecayCurve, TimeDecaySurface};
pub use volatility_sensitivity::{VolatilitySensitivityCurve, VolatilitySensitivitySurface};
