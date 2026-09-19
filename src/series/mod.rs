//! * `Option Series` - Functionality for working with collections of option chains across expirations.
//!
//! Provides tools to manage, filter, and analyze multiple option chains grouped by expiration dates.
//! Includes utilities for constructing series data, navigating expirations, and performing
//! cross-expiration analysis and visualization.

/// Simulation-backed series generation, gated like `chains::generators` by
/// the `synthetic` feature (on by default).
#[cfg(feature = "synthetic")]
mod generators;
mod model;
mod params;

#[cfg(feature = "synthetic")]
pub use generators::generator_optionseries;
pub use model::OptionSeries;
pub use params::OptionSeriesBuildParams;
