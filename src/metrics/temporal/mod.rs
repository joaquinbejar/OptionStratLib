/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 24/12/25
******************************************************************************/

//! # Temporal Metrics Module
//!
//! Provides temporal sensitivity metrics for options analysis, measuring how
//! option prices and risk exposures change over time. These metrics are critical
//! for time decay analysis and hedging strategies.
//!
//! ## Core Features
//!
//! ### Theta (Time Decay)
//!
//! Measures the rate of change of option value with respect to time:
//! - First-order time sensitivity
//! - Typically negative for long options (value decays)
//! - Accelerates near expiration
//!
//! - **Curve representation by strike**: Shows theta at each strike
//! - **Surface representation (price vs time)**: Theta evolution over time
//!
//! ### Charm (Delta Decay)
//!
//! Measures the rate of change of delta with respect to time:
//! - Also known as DdeltaDtime
//! - Important for delta hedging over time
//! - Shows how delta drifts as time passes
//!
//! - **Curve representation by strike**: Shows charm at each strike
//! - **Surface representation (price vs time)**: Charm evolution over time
//!
//! ### Color (Gamma Decay)
//!
//! Measures the rate of change of gamma with respect to time:
//! - Also known as DgammaDtime
//! - Important for gamma hedging over time
//! - Shows how gamma changes as expiration approaches
//!
//! - **Curve representation by strike**: Shows color at each strike
//! - **Surface representation (price vs time)**: Color evolution over time
//!
//! ## Usage Examples
//!
//! ### Theta Curve and Surface
//!
//! ```rust
//! use std::collections::BTreeSet;
//! use rust_decimal::Decimal;
//! use optionstratlib::curves::Curve;
//! use optionstratlib::error::{CurveError, SurfaceError};
//! use optionstratlib::surfaces::Surface;
//! use optionstratlib::metrics::{ThetaCurve, ThetaSurface};
//! use positive::Positive;
//!
//! struct MyTheta;
//!
//! impl ThetaCurve for MyTheta {
//!     fn theta_curve(&self) -> Result<Curve, CurveError> {
//!         // Custom logic to compute theta by strike
//!         Ok(Curve { points: BTreeSet::new(), x_range: (Decimal::ZERO, Decimal::ZERO) })
//!     }
//! }
//!
//! impl ThetaSurface for MyTheta {
//!     fn theta_surface(
//!         &self,
//!         _price_range: (Positive, Positive),
//!         _days_to_expiry: Vec<Positive>,
//!         _price_steps: usize,
//!     ) -> Result<Surface, SurfaceError> {
//!         // Custom logic to compute theta surface
//!         Ok(Surface::new(BTreeSet::new()))
//!     }
//! }
//! ```
//!
//! ### Charm Curve and Surface
//!
//! ```rust
//! use std::collections::BTreeSet;
//! use rust_decimal::Decimal;
//! use optionstratlib::curves::Curve;
//! use optionstratlib::error::{CurveError, SurfaceError};
//! use optionstratlib::surfaces::Surface;
//! use optionstratlib::metrics::{CharmCurve, CharmSurface};
//! use positive::Positive;
//!
//! struct MyCharm;
//!
//! impl CharmCurve for MyCharm {
//!     fn charm_curve(&self) -> Result<Curve, CurveError> {
//!         // Custom logic to compute charm (delta decay) by strike
//!         Ok(Curve { points: BTreeSet::new(), x_range: (Decimal::ZERO, Decimal::ZERO) })
//!     }
//! }
//!
//! impl CharmSurface for MyCharm {
//!     fn charm_surface(
//!         &self,
//!         _price_range: (Positive, Positive),
//!         _days_to_expiry: Vec<Positive>,
//!         _price_steps: usize,
//!     ) -> Result<Surface, SurfaceError> {
//!         // Custom logic to compute charm surface
//!         Ok(Surface::new(BTreeSet::new()))
//!     }
//! }
//! ```
//!
//! ### Color Curve and Surface
//!
//! ```rust
//! use std::collections::BTreeSet;
//! use rust_decimal::Decimal;
//! use optionstratlib::curves::Curve;
//! use optionstratlib::error::{CurveError, SurfaceError};
//! use optionstratlib::surfaces::Surface;
//! use optionstratlib::metrics::{ColorCurve, ColorSurface};
//! use positive::Positive;
//!
//! struct MyColor;
//!
//! impl ColorCurve for MyColor {
//!     fn color_curve(&self) -> Result<Curve, CurveError> {
//!         // Custom logic to compute color (gamma decay) by strike
//!         Ok(Curve { points: BTreeSet::new(), x_range: (Decimal::ZERO, Decimal::ZERO) })
//!     }
//! }
//!
//! impl ColorSurface for MyColor {
//!     fn color_surface(
//!         &self,
//!         _price_range: (Positive, Positive),
//!         _days_to_expiry: Vec<Positive>,
//!         _price_steps: usize,
//!     ) -> Result<Surface, SurfaceError> {
//!         // Custom logic to compute color surface
//!         Ok(Surface::new(BTreeSet::new()))
//!     }
//! }
//! ```

pub mod charm;
pub mod color;
pub mod theta;

pub use charm::{CharmCurve, CharmSurface};
pub use color::{ColorCurve, ColorSurface};
pub use theta::{ThetaCurve, ThetaSurface};
