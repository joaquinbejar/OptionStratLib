/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 24/12/25
******************************************************************************/

//! # Composite Metrics Module
//!
//! Provides advanced composite metrics for options analysis, combining multiple
//! Greeks and market data to provide deeper insights into risk profiles and
//! market structure.
//!
//! ## Core Features
//!
//! ### Vanna-Volga Hedge
//!
//! The Vanna-Volga method is a technique for pricing and hedging options that
//! accounts for the volatility smile. It uses three vanilla options (typically
//! ATM and two OTM options) to hedge vega, vanna, and volga risks.
//!
//! - **Surface representation (price vs volatility)**: Shows how hedge costs
//!   vary across different underlying prices and volatility levels.
//!
//! ### Delta-Gamma Profile
//!
//! Combined analysis of delta and gamma exposure across a portfolio or option
//! chain. Essential for understanding directional risk and convexity.
//!
//! - **Curve representation by strike**: Shows delta/gamma exposure at each strike
//! - **Surface representation (price vs time)**: Shows how exposure evolves
//!
//! ### Smile Dynamics
//!
//! Tracks how the volatility smile evolves over time, measuring changes in:
//! - ATM volatility level (parallel shift)
//! - Skew (slope of the smile)
//! - Curvature (butterfly/convexity)
//!
//! - **Curve representation by strike**: Current smile shape
//! - **Surface representation (strike vs time)**: Smile evolution over time
//!
//! ## Usage Examples
//!
//! ### Vanna-Volga Surface
//!
//! ```rust
//! use std::collections::BTreeSet;
//! use optionstratlib_math::error::SurfaceError;
//! use optionstratlib_math::surfaces::Surface;
//! use optionstratlib_analytics::metrics::VannaVolgaSurface;
//! use optionstratlib_core::model::Positive;
//!
//! struct MyVannaVolga;
//!
//! impl VannaVolgaSurface for MyVannaVolga {
//!     fn vanna_volga_surface(
//!         &self,
//!         _price_range: (Positive, Positive),
//!         _vol_range: (Positive, Positive),
//!         _price_steps: usize,
//!         _vol_steps: usize,
//!     ) -> Result<Surface, SurfaceError> {
//!         // Custom logic to compute Vanna-Volga hedge surface
//!         Ok(Surface::new(BTreeSet::new()))
//!     }
//! }
//! ```
//!
//! ### Delta-Gamma Profile
//!
//! ```rust
//! use std::collections::BTreeSet;
//! use rust_decimal::Decimal;
//! use optionstratlib_math::curves::Curve;
//! use optionstratlib_math::error::CurveError;
//! use optionstratlib_math::error::SurfaceError;
//! use optionstratlib_math::surfaces::Surface;
//! use optionstratlib_analytics::metrics::{DeltaGammaProfileCurve, DeltaGammaProfileSurface};
//! use optionstratlib_core::model::Positive;
//!
//! struct MyDeltaGamma;
//!
//! impl DeltaGammaProfileCurve for MyDeltaGamma {
//!     fn delta_gamma_curve(&self) -> Result<Curve, CurveError> {
//!         // Custom logic to compute delta-gamma profile by strike
//!         Ok(Curve { points: BTreeSet::new(), x_range: (Decimal::ZERO, Decimal::ZERO) })
//!     }
//! }
//!
//! impl DeltaGammaProfileSurface for MyDeltaGamma {
//!     fn delta_gamma_surface(
//!         &self,
//!         _price_range: (Positive, Positive),
//!         _days_to_expiry: Vec<Positive>,
//!         _price_steps: usize,
//!     ) -> Result<Surface, SurfaceError> {
//!         // Custom logic to compute delta-gamma surface
//!         Ok(Surface::new(BTreeSet::new()))
//!     }
//! }
//! ```
//!
//! ### Smile Dynamics
//!
//! ```rust
//! use std::collections::BTreeSet;
//! use rust_decimal::Decimal;
//! use optionstratlib_math::curves::Curve;
//! use optionstratlib_math::error::CurveError;
//! use optionstratlib_math::error::SurfaceError;
//! use optionstratlib_math::surfaces::Surface;
//! use optionstratlib_analytics::metrics::{SmileDynamicsCurve, SmileDynamicsSurface};
//! use optionstratlib_core::model::Positive;
//!
//! struct MySmileDynamics;
//!
//! impl SmileDynamicsCurve for MySmileDynamics {
//!     fn smile_dynamics_curve(&self) -> Result<Curve, CurveError> {
//!         // Custom logic to compute volatility smile curve
//!         Ok(Curve { points: BTreeSet::new(), x_range: (Decimal::ZERO, Decimal::ZERO) })
//!     }
//! }
//!
//! impl SmileDynamicsSurface for MySmileDynamics {
//!     fn smile_dynamics_surface(
//!         &self,
//!         _days_to_expiry: Vec<Positive>,
//!     ) -> Result<Surface, SurfaceError> {
//!         // Custom logic to compute smile dynamics surface
//!         Ok(Surface::new(BTreeSet::new()))
//!     }
//! }
//! ```

pub mod delta_gamma_profile;
pub mod smile_dynamics;
pub mod vanna_volga;

pub use delta_gamma_profile::{DeltaGammaProfileCurve, DeltaGammaProfileSurface};
pub use smile_dynamics::{SmileDynamicsCurve, SmileDynamicsSurface};
pub use vanna_volga::VannaVolgaSurface;
