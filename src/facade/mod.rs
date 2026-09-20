//! Items that compose two components and belong to neither.
//!
//! The crate's target architecture (ADR-0001) forbids a component from
//! naming a component that sits above it. A few public items of the 0.21
//! surface do exactly that, not because either component needs the other,
//! but because they are convenience dispatchers over both. They live here,
//! in the layer that may depend on everything, and keep their 0.21 paths
//! through re-exports marked `// facade-compat: facade` in the component
//! that used to own them. At extraction time this module is the facade
//! crate's own code.

/// The pricing dispatcher that names the concrete simulation engine.
pub mod pricing_engine;

/// The combination helper that reports the crate-level unified error.
pub mod combinations;

pub use combinations::process_n_times_iter;
pub use pricing_engine::{Priceable, PricingEngine, price_option};
