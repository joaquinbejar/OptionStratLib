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
//! ```ignore
//! use optionstratlib::chains::chain::OptionChain;
//! use optionstratlib::metrics::BidAskSpreadCurve;
//!
//! let chain = OptionChain::load_from_json("options.json")?;
//! let spread_curve = chain.bid_ask_spread_curve()?;
//! ```
//!
//! ### Volume Profile
//!
//! ```ignore
//! use optionstratlib::chains::chain::OptionChain;
//! use optionstratlib::metrics::{VolumeProfileCurve, VolumeProfileSurface};
//!
//! let chain = OptionChain::load_from_json("options.json")?;
//! let volume_curve = chain.volume_profile_curve()?;
//! let volume_surface = chain.volume_profile_surface(days_to_expiry)?;
//! ```
//!
//! ### Open Interest Distribution
//!
//! ```ignore
//! use optionstratlib::chains::chain::OptionChain;
//! use optionstratlib::metrics::OpenInterestCurve;
//!
//! let chain = OptionChain::load_from_json("options.json")?;
//! let oi_curve = chain.open_interest_curve()?;
//! ```

pub mod bid_ask_spread;
pub mod open_interest;
pub mod volume_profile;

pub use bid_ask_spread::BidAskSpreadCurve;
pub use open_interest::OpenInterestCurve;
pub use volume_profile::{VolumeProfileCurve, VolumeProfileSurface};
