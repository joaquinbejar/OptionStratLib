/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 18/12/25
******************************************************************************/

//! # Price Metrics Module
//!
//! Provides comprehensive price performance metrics tools for financial applications
//!
//! ## Core Components
//! - `VolatilitySkewCurve` trait: Enables plotting functionality for volatility skew curves.
//! - `PutCallRatioCurve` trait: Provides plotting functionalities for premium-weighted Put/Call
//!   ratio curves.
//! - `StrikeConcentrationCurve` trait: Provides plotting functionalities for premium-weighted
//!   Strike Concentration curves.
//!
//! ## Usage Example Volatility Skew
//! ```rust
//! use std::collections::BTreeSet;
//! use rust_decimal::Decimal;
//! use optionstratlib::curves::Curve;
//! use optionstratlib::error::CurveError;
//! use optionstratlib::metrics::VolatilitySkewCurve;
//!
//! struct MySkew;
//!
//! impl VolatilitySkewCurve for MySkew {
//!     fn volatility_skew(&self) -> Result<Curve, CurveError> {
//!         // Custom logic to build and return a Curve representing the skew
//!         let curve = Curve { points: BTreeSet::new(), x_range: (Decimal::ZERO, Decimal::ZERO) };
//!         Ok(curve)
//!     }
//! }
//! ```
//!
//! ## Usage Example Put Call Ratio
//! ```rust
//! use std::collections::BTreeSet;
//! use rust_decimal::Decimal;
//! use optionstratlib::curves::Curve;
//! use optionstratlib::error::CurveError;
//! use optionstratlib::metrics::PutCallRatioCurve;
//!
//! struct MyPcr;
//!
//! impl PutCallRatioCurve for MyPcr {
//!     fn premium_weighted_pcr(&self) -> Result<Curve, CurveError> {
//!         // Custom logic to build and return a Curve representing the premium
//!         // weighted put/call ratio
//!         let curve = Curve { points: BTreeSet::new(), x_range: (Decimal::ZERO, Decimal::ZERO) };
//!         Ok(curve)
//!     }
//! }
//! ```
//!
//! ## Usage Example Strike Concentration
//! ```rust
//! use std::collections::BTreeSet;
//! use rust_decimal::Decimal;
//! use optionstratlib::curves::Curve;
//! use optionstratlib::error::CurveError;
//! use optionstratlib::metrics::StrikeConcentrationCurve;
//!
//! struct MyStrikeConcentration;
//!
//! impl StrikeConcentrationCurve for MyStrikeConcentration {
//!     fn premium_concentration(&self) -> Result<Curve, CurveError> {
//!         // Custom logic to build and return a Curve representing the premium
//!         // weighted strike concentration
//!         let curve = Curve { points: BTreeSet::new(), x_range: (Decimal::ZERO, Decimal::ZERO) };
//!         Ok(curve)
//!     }
//! }
//! ```

pub mod put_call_ratio;
pub mod strike_concentration;
pub mod volatility_skew;

pub use put_call_ratio::PutCallRatioCurve;
pub use strike_concentration::StrikeConcentrationCurve;
pub use volatility_skew::VolatilitySkewCurve;
