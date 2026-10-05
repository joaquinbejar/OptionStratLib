//! * `Option Series` - Functionality for working with collections of option chains across expirations.
//!
//! Provides tools to manage, filter, and analyze multiple option chains grouped by expiration dates.
//! Includes utilities for constructing series data, navigating expirations, and performing
//! cross-expiration analysis and visualization.

mod model;
mod params;

pub use model::OptionSeries;
pub use params::OptionSeriesBuildParams;
