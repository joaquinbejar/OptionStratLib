/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 24/12/25
******************************************************************************/

//! # Liquidity Metrics Module
//!
//! Provides comprehensive liquidity metrics for options analysis, helping assess
//! market depth, trading activity, and positioning across different strikes.
//!
//! ## Core Features
//!
//! ### Bid-Ask Spread
//!
//! Measures the difference between bid and ask prices, indicating:
//! - Market liquidity at each strike
//! - Transaction costs for entering/exiting positions
//! - Market maker activity and competition
//!
//! - **Curve representation by strike**: Shows spread across all strikes
//!
//! ### Volume Profile
//!
//! Tracks trading activity across strikes and time:
//! - Identifies active trading zones
//! - Reveals market interest at specific price levels
//! - Helps identify support/resistance levels
//!
//! - **Curve representation by strike**: Current volume distribution
//! - **Surface representation (price vs time)**: Volume evolution over time
//!
//! ### Open Interest Distribution
//!
//! Shows outstanding contracts at each strike:
//! - Indicates where positions are concentrated
//! - Helps identify potential "max pain" levels
//! - Reveals market positioning and sentiment
//!
//! - **Curve representation by strike**: OI distribution across strikes
//!
//! ## Usage Examples
//!
//! ### Bid-Ask Spread Curve
//!
//! ```rust
//! use std::collections::BTreeSet;
//! use rust_decimal::Decimal;
//! use optionstratlib::curves::Curve;
//! use optionstratlib::error::CurveError;
//! use optionstratlib::metrics::BidAskSpreadCurve;
//!
//! struct MyBidAskSpread;
//!
//! impl BidAskSpreadCurve for MyBidAskSpread {
//!     fn bid_ask_spread_curve(&self) -> Result<Curve, CurveError> {
//!         // Custom logic to compute bid-ask spread by strike
//!         Ok(Curve { points: BTreeSet::new(), x_range: (Decimal::ZERO, Decimal::ZERO) })
//!     }
//! }
//! ```
//!
//! ### Volume Profile
//!
//! ```rust
//! use std::collections::BTreeSet;
//! use rust_decimal::Decimal;
//! use optionstratlib::curves::Curve;
//! use optionstratlib::error::{CurveError, SurfaceError};
//! use optionstratlib::surfaces::Surface;
//! use optionstratlib::metrics::{VolumeProfileCurve, VolumeProfileSurface};
//! use positive::Positive;
//!
//! struct MyVolumeProfile;
//!
//! impl VolumeProfileCurve for MyVolumeProfile {
//!     fn volume_profile_curve(&self) -> Result<Curve, CurveError> {
//!         // Custom logic to compute volume distribution by strike
//!         Ok(Curve { points: BTreeSet::new(), x_range: (Decimal::ZERO, Decimal::ZERO) })
//!     }
//! }
//!
//! impl VolumeProfileSurface for MyVolumeProfile {
//!     fn volume_profile_surface(&self, _days: Vec<Positive>) -> Result<Surface, SurfaceError> {
//!         // Custom logic to compute volume surface over time
//!         Ok(Surface::new(BTreeSet::new()))
//!     }
//! }
//! ```
//!
//! ### Open Interest Distribution
//!
//! ```rust
//! use std::collections::BTreeSet;
//! use rust_decimal::Decimal;
//! use optionstratlib::curves::Curve;
//! use optionstratlib::error::CurveError;
//! use optionstratlib::metrics::OpenInterestCurve;
//!
//! struct MyOpenInterest;
//!
//! impl OpenInterestCurve for MyOpenInterest {
//!     fn open_interest_curve(&self) -> Result<Curve, CurveError> {
//!         // Custom logic to compute open interest distribution by strike
//!         Ok(Curve { points: BTreeSet::new(), x_range: (Decimal::ZERO, Decimal::ZERO) })
//!     }
//! }
//! ```

pub mod bid_ask_spread;
pub mod open_interest;
pub mod volume_profile;

pub use bid_ask_spread::BidAskSpreadCurve;
pub use open_interest::OpenInterestCurve;
pub use volume_profile::{VolumeProfileCurve, VolumeProfileSurface};
