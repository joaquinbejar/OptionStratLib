/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 18/12/25
******************************************************************************/

//! # Metrics Module
//!
//! This module provides comprehensive performance metrics tools for financial applications:
//! - Price Metrics (Volatility Skew, Put/Call Ratio, Strike Concentration)
//! - Risk Metrics (Implied Volatility, Risk Reversal, Dollar Gamma)
//!
//! ## Core Features
//!
//! ### Price Metrics
//!
//! - Volatility Skew: indicates variations in moneyness across options, revealing
//!   insights into market sentiment and expectations. Skew patterns serve as valuable tools
//!   for developing effective trading strategies. Volatility skew reflects differences in
//!   moneyness among options with the same expiration but different strike prices, highlighting
//!   market sentiment and expectations.
//! - Put Call Ratio: indicates if the investors are placing more bets on prices falling or rising.
//!   It has been conceived to be a measure of market sentiment.
//! - Strike Concentration: identifies specific strike prices with unusually high open
//!   interest or trading volume. It looks for clusters of activity where a large number of
//!   contracts are concentrated.
//!
//! ### Risk Metrics
//!
//! - **Implied Volatility**: IV curves by strike and surfaces (strike vs time)
//! - **Risk Reversal**: Difference between call and put IV at same strike
//! - **Dollar Gamma**: Gamma exposure in dollar terms (Gamma × Spot² × 0.01)
//!
//! ## Usage Examples
//!
//! ### Volatility Skew
//!
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
//! ### Put Call Ratio
//!
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
//! ### Strike Concentration
//!
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
//! ### Implied Volatility Curve
//!
//! ```rust
//! use std::collections::BTreeSet;
//! use rust_decimal::Decimal;
//! use optionstratlib::curves::Curve;
//! use optionstratlib::error::CurveError;
//! use optionstratlib::metrics::ImpliedVolatilityCurve;
//!
//! struct MyIVCurve;
//!
//! impl ImpliedVolatilityCurve for MyIVCurve {
//!     fn iv_curve(&self) -> Result<Curve, CurveError> {
//!         // Custom logic to build and return a Curve representing IV by strike
//!         Ok(Curve { points: BTreeSet::new(), x_range: (Decimal::ZERO, Decimal::ZERO) })
//!     }
//! }
//! ```
//!
//! ### Risk Reversal Curve
//!
//! ```rust
//! use std::collections::BTreeSet;
//! use rust_decimal::Decimal;
//! use optionstratlib::curves::Curve;
//! use optionstratlib::error::CurveError;
//! use optionstratlib::metrics::RiskReversalCurve;
//!
//! struct MyRRCurve;
//!
//! impl RiskReversalCurve for MyRRCurve {
//!     fn risk_reversal_curve(&self) -> Result<Curve, CurveError> {
//!         // Custom logic to build and return a Curve representing risk reversal
//!         Ok(Curve { points: BTreeSet::new(), x_range: (Decimal::ZERO, Decimal::ZERO) })
//!     }
//! }
//! ```
//!
//! ### Dollar Gamma Curve
//!
//! ```rust
//! use std::collections::BTreeSet;
//! use rust_decimal::Decimal;
//! use optionstratlib::curves::Curve;
//! use optionstratlib::error::CurveError;
//! use optionstratlib::metrics::DollarGammaCurve;
//! use optionstratlib::model::OptionStyle;
//!
//! struct MyDGCurve;
//!
//! impl DollarGammaCurve for MyDGCurve {
//!     fn dollar_gamma_curve(&self, _option_style: &OptionStyle) -> Result<Curve, CurveError> {
//!         // Custom logic to build and return a Curve representing dollar gamma
//!         Ok(Curve { points: BTreeSet::new(), x_range: (Decimal::ZERO, Decimal::ZERO) })
//!     }
//! }
//! ```
pub mod composite;
pub mod liquidity;
pub mod price;
pub mod risk;
pub mod stress;
pub mod temporal;

pub use composite::{
    DeltaGammaProfileCurve, DeltaGammaProfileSurface, SmileDynamicsCurve, SmileDynamicsSurface,
    VannaVolgaSurface,
};
pub use liquidity::{
    BidAskSpreadCurve, OpenInterestCurve, VolumeProfileCurve, VolumeProfileSurface,
};
pub use price::{PutCallRatioCurve, StrikeConcentrationCurve, VolatilitySkewCurve};
pub use risk::{
    DollarGammaCurve, ImpliedVolatilityCurve, ImpliedVolatilitySurface, RiskReversalCurve,
};
pub use stress::{
    PriceShockCurve, PriceShockSurface, TimeDecayCurve, TimeDecaySurface,
    VolatilitySensitivityCurve, VolatilitySensitivitySurface,
};
pub use temporal::{CharmCurve, CharmSurface, ColorCurve, ColorSurface, ThetaCurve, ThetaSurface};
