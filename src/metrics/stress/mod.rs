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
//! ```ignore
//! use optionstratlib::chains::chain::OptionChain;
//! use optionstratlib::metrics::{VolatilitySensitivityCurve, VolatilitySensitivitySurface};
//!
//! let chain = OptionChain::load_from_json("options.json")?;
//! let vega_curve = chain.volatility_sensitivity_curve()?;
//! let vol_surface = chain.volatility_sensitivity_surface(price_range, vol_range)?;
//! ```
//!
//! ### Time Decay Profile
//!
//! ```ignore
//! use optionstratlib::chains::chain::OptionChain;
//! use optionstratlib::metrics::{TimeDecayCurve, TimeDecaySurface};
//!
//! let chain = OptionChain::load_from_json("options.json")?;
//! let theta_curve = chain.time_decay_curve()?;
//! let decay_surface = chain.time_decay_surface(price_range, days)?;
//! ```
//!
//! ### Price Shock Impact
//!
//! ```ignore
//! use optionstratlib::chains::chain::OptionChain;
//! use optionstratlib::metrics::{PriceShockCurve, PriceShockSurface};
//!
//! let chain = OptionChain::load_from_json("options.json")?;
//! let shock_curve = chain.price_shock_curve(shock_pct)?;
//! let shock_surface = chain.price_shock_surface(price_range, vol_range)?;
//! ```

pub mod price_shock;
pub mod time_decay;
pub mod volatility_sensitivity;

pub use price_shock::{PriceShockCurve, PriceShockSurface};
pub use time_decay::{TimeDecayCurve, TimeDecaySurface};
pub use volatility_sensitivity::{VolatilitySensitivityCurve, VolatilitySensitivitySurface};
