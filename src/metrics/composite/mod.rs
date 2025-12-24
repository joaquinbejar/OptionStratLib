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
//! ```ignore
//! use optionstratlib::chains::chain::OptionChain;
//! use optionstratlib::metrics::VannaVolgaSurface;
//!
//! let chain = OptionChain::load_from_json("options.json")?;
//! let surface = chain.vanna_volga_surface(price_range, vol_range)?;
//! ```
//!
//! ### Delta-Gamma Profile
//!
//! ```ignore
//! use optionstratlib::chains::chain::OptionChain;
//! use optionstratlib::metrics::{DeltaGammaProfileCurve, DeltaGammaProfileSurface};
//!
//! let chain = OptionChain::load_from_json("options.json")?;
//! let curve = chain.delta_gamma_curve()?;
//! let surface = chain.delta_gamma_surface(price_range, time_range)?;
//! ```
//!
//! ### Smile Dynamics
//!
//! ```ignore
//! use optionstratlib::chains::chain::OptionChain;
//! use optionstratlib::metrics::{SmileDynamicsCurve, SmileDynamicsSurface};
//!
//! let chain = OptionChain::load_from_json("options.json")?;
//! let curve = chain.smile_dynamics_curve()?;
//! let surface = chain.smile_dynamics_surface(days_to_expiry)?;
//! ```

pub mod delta_gamma_profile;
pub mod smile_dynamics;
pub mod vanna_volga;

pub use delta_gamma_profile::{DeltaGammaProfileCurve, DeltaGammaProfileSurface};
pub use smile_dynamics::{SmileDynamicsCurve, SmileDynamicsSurface};
pub use vanna_volga::VannaVolgaSurface;
