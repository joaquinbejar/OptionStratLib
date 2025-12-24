/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 24/12/24
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
//! ```ignore
//! use optionstratlib::chains::chain::OptionChain;
//! use optionstratlib::metrics::{ThetaCurve, ThetaSurface};
//!
//! let chain = OptionChain::load_from_json("options.json")?;
//! let theta_curve = chain.theta_curve()?;
//! let theta_surface = chain.theta_surface(price_range, days)?;
//! ```
//!
//! ### Charm Curve and Surface
//!
//! ```ignore
//! use optionstratlib::chains::chain::OptionChain;
//! use optionstratlib::metrics::{CharmCurve, CharmSurface};
//!
//! let chain = OptionChain::load_from_json("options.json")?;
//! let charm_curve = chain.charm_curve()?;
//! let charm_surface = chain.charm_surface(price_range, days)?;
//! ```
//!
//! ### Color Curve and Surface
//!
//! ```ignore
//! use optionstratlib::chains::chain::OptionChain;
//! use optionstratlib::metrics::{ColorCurve, ColorSurface};
//!
//! let chain = OptionChain::load_from_json("options.json")?;
//! let color_curve = chain.color_curve()?;
//! let color_surface = chain.color_surface(price_range, days)?;
//! ```

pub mod charm;
pub mod color;
pub mod theta;

pub use charm::{CharmCurve, CharmSurface};
pub use color::{ColorCurve, ColorSurface};
pub use theta::{ThetaCurve, ThetaSurface};
