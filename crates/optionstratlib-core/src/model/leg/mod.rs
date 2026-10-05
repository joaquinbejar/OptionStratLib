/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 24/12/25
******************************************************************************/

//! # Leg Module
//!
//! This module provides types for representing different kinds of trading legs
//! (positions) in multi-instrument strategies. It enables strategies that combine
//! options with underlying assets, futures, or perpetual contracts.
//!
//! ## Overview
//!
//! The module introduces a `Leg` enum that unifies different position types:
//!
//! - **Option** - Standard option positions (Call/Put) via existing `Position`
//! - **Spot** - Direct ownership of underlying assets
//! - **Future** - Exchange-traded futures contracts
//! - **Perpetual** - Crypto perpetual swap contracts
//!
//! ## Key Types
//!
//! - `Leg` - Unified enum for all leg types
//! - `SpotPosition` - Spot/underlying asset position
//! - `FuturePosition` - Futures contract position
//! - `PerpetualPosition` - Perpetual swap position
//! - `MarginType` - Cross vs Isolated margin mode
//!
//! ## Traits
//!
//! - `LegAble` - Common interface for all leg types
//! - `Marginable` - For positions with margin requirements
//! - `Fundable` - For positions with funding rate payments
//! - `Expirable` - For positions with expiration dates
//!
//! ## Example: Covered Call Strategy
//!
//! ```rust
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! use optionstratlib_core::model::leg::{Leg, SpotPosition};
//! use optionstratlib_core::model::Position;
//! use optionstratlib_core::model::types::Side;
//! use positive::{Positive, pos_or_panic};
//!
//! // Long 100 shares of stock
//! let spot = SpotPosition::long("AAPL".to_string(), Positive::HUNDRED, pos_or_panic!(150.0));
//! let spot_leg = Leg::Spot(spot);
//!
//! // The option leg would be created from a Position
//! // let call_leg = Leg::option(short_call_position);
//!
//! // Both legs can be handled uniformly via the LegAble trait
//! use optionstratlib_core::model::leg::LegAble;
//! tracing::info!("Spot P&L at 160: {}", spot_leg.pnl_at_price(pos_or_panic!(160.0))?);
//! # Ok(())
//! # }
//! ```
//!
//! ## Example: Cash & Carry Arbitrage (Crypto)
//!
//! ```rust
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! use optionstratlib_core::model::leg::{Leg, SpotPosition, PerpetualPosition, MarginType};
//! use optionstratlib_core::model::types::Side;
//! use positive::{Positive, pos_or_panic};
//! use rust_decimal_macros::dec;
//! use chrono::Utc;
//!
//! // Long 1 BTC spot
//! let spot = SpotPosition::long("BTC".to_string(), Positive::ONE, pos_or_panic!(50000.0));
//!
//! // Short 1 BTC perpetual (delta neutral)
//! let perp = PerpetualPosition::short(
//!     "BTC-USDT-PERP".to_string(),
//!     Positive::ONE,
//!     pos_or_panic!(50000.0),
//!     Positive::ONE,  // 1x leverage for delta neutral
//!     pos_or_panic!(50000.0),
//! );
//!
//! let spot_leg = Leg::Spot(spot);
//! let perp_leg = Leg::Perpetual(perp);
//!
//! // A move in the underlying leaves the hedged pair flat
//! use optionstratlib_core::model::leg::LegAble;
//! let price = pos_or_panic!(51000.0);
//! let net_pnl = spot_leg.pnl_at_price(price)? + perp_leg.pnl_at_price(price)?;
//! assert_eq!(net_pnl, rust_decimal::Decimal::ZERO);
//! # Ok(())
//! # }
//! ```
//!
//! ## Strategies Enabled
//!
//! ### Traditional Markets
//! - Covered Call (Spot + Short Call)
//! - Protective Put (Spot + Long Put)
//! - Collar (Spot + Long Put + Short Call)
//! - Synthetic positions
//!
//! ### Crypto Markets
//! - Cash & Carry (Spot + Short Perp)
//! - Basis Trade (Spot + Short Future)
//! - Delta Neutral Funding (Spot + Short Perp equal size)
//! - Hedged Perpetual (Perp + Options)

mod future;
mod leg_enum;
mod perpetual;
mod spot;
pub mod traits;

pub use future::FuturePosition;
pub use leg_enum::Leg;
pub use perpetual::{MarginType, PerpetualPosition};
pub use spot::SpotPosition;
pub use traits::{Expirable, Fundable, LegAble, Marginable};
