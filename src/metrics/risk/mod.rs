/******************************************************************************
use positive::pos_or_panic;
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 24/12/25
******************************************************************************/

//! # Risk Metrics Module
//!
//! Provides comprehensive risk metrics tools for financial applications,
//! including implied volatility curves and surfaces, risk reversal curves,
//! and dollar gamma curves.
//!
//! ## Core Features
//!
//! ### Implied Volatility
//!
//! - **Curve representation by strike price**: Shows how IV varies across
//!   different strike prices for options with the same expiration.
//! - **Surface representation (strike vs time)**: Provides a complete view
//!   of the volatility structure across both strikes and time horizons.
//!
//! ### Risk Reversal
//!
//! - **Curve representation by strike price**: Measures the difference between
//!   OTM call and put implied volatilities, indicating market sentiment.
//! - Positive values suggest bullish sentiment (calls more expensive)
//! - Negative values suggest bearish sentiment (puts more expensive)
//!
//! ### Dollar Gamma
//!
//! - **Curve representation by strike price**: Shows gamma exposure in dollar
//!   terms, crucial for understanding monetary risk from gamma.
//! - Formula: Dollar Gamma = Gamma × Spot² × 0.01
//!
//! ## Usage Examples
//!
//! ### Implied Volatility Curve
//!
//! ```ignore
//! use optionstratlib::chains::chain::OptionChain;
//! use optionstratlib::metrics::ImpliedVolatilityCurve;
//!
//! let chain = OptionChain::new("SPY", pos_or_panic!(450.0), "2024-03-15".to_string(), None, None);
//! let iv_curve = chain.iv_curve()?;
//! ```
//!
//! ### Risk Reversal Curve
//!
//! ```ignore
//! use optionstratlib::chains::chain::OptionChain;
//! use optionstratlib::metrics::RiskReversalCurve;
//!
//! let chain = OptionChain::new("SPY", pos_or_panic!(450.0), "2024-03-15".to_string(), None, None);
//! let rr_curve = chain.risk_reversal_curve()?;
//! ```
//!
//! ### Dollar Gamma Curve
//!
//! ```ignore
//! use optionstratlib::chains::chain::OptionChain;
//! use optionstratlib::metrics::DollarGammaCurve;
//! use optionstratlib::model::OptionStyle;
//!
//! let chain = OptionChain::new("SPY", pos_or_panic!(450.0), "2024-03-15".to_string(), None, None);
//! let dg_curve = chain.dollar_gamma_curve(&OptionStyle::Call)?;
//! ```

pub mod dollar_gamma;
pub mod implied_volatility;
pub mod risk_reversal;

pub use dollar_gamma::DollarGammaCurve;
pub use implied_volatility::{ImpliedVolatilityCurve, ImpliedVolatilitySurface};
pub use risk_reversal::RiskReversalCurve;
