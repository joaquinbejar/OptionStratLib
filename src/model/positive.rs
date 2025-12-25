/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 30/12/24
******************************************************************************/

//! Re-exports the Positive type from the positive crate.
//!
//! This module provides the `Positive` type for representing guaranteed
//! non-negative decimal values. The core implementation is provided by
//! the external `positive` crate, while OptionStratLib-specific extensions
//! are provided in the `positive_ext` module.
//!
//! ## Example
//!
//! ```rust
//! use optionstratlib::pos_or_panic;
//! let strike_price = pos_or_panic!(100.0);
//! ```

// Re-export everything from the positive crate
pub use positive::error::{PositiveError, PositiveResult};
pub use positive::{EPSILON as POSITIVE_EPSILON, Positive, is_positive};
