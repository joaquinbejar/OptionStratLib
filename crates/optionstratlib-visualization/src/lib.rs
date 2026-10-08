#![deny(missing_docs, rustdoc::broken_intra_doc_links)]
// Per rules/global_rules.md §Error Handling, unchecked `[]` / slicing is
// banned in production code, with no per-file escape hatch (#341, #788).
#![deny(clippy::indexing_slicing)]
// Unit tests routinely index into `Vec`s they just pushed into, so the lint
// is silenced in `#[cfg(test)]` only.
#![cfg_attr(test, allow(clippy::indexing_slicing))]

//! # optionstratlib-visualization
//!
//! Charts for OptionStratLib: backend-neutral chart models, the one
//! [`visualization::Graph`] contract, its implementations for the library's
//! types, and an optional Plotly backend. It is the leaf of the workspace:
//! it depends on `optionstratlib-core`, `optionstratlib-math`,
//! `optionstratlib-pricing`, `optionstratlib-simulation`,
//! `optionstratlib-market`, `optionstratlib-strategies` and
//! `optionstratlib-backtest`, and no OptionStratLib crate depends on it, so
//! headless consumers never build it.
//!
//! - [`visualization`]: the [`visualization::Graph`] trait, the chart data
//!   ([`visualization::GraphData`], [`visualization::Series2D`],
//!   [`visualization::Surface3D`]), [`visualization::GraphConfig`] and the
//!   styles, [`visualization::PlotBuilder`] and
//!   [`visualization::Plottable`], and the `Graph` implementations for
//!   options, positions, curves, surfaces, random walks, simulators and every
//!   concrete strategy.
//! - [`visualization::terminal`]: terminal tables for option chains and
//!   simulation statistics, the only code in the library that writes to
//!   stdout, and only when asked to ([`visualization::terminal::ChainReport`],
//!   [`visualization::terminal::SimulationReport`]).
//! - [`error`]: the [`error::GraphError`] type.
//!
//! ## One graph contract
//!
//! [`visualization::Graph`] has the same definition and the same required
//! items on every feature surface: an implementor supplies `graph_data` and
//! may override `graph_config`, whatever backend is enabled. The backend
//! features add provided rendering methods to it and never change what an
//! implementor writes:
//!
//! | Surface | Provided by `Graph` |
//! | --- | --- |
//! | no features | `graph_data`, `graph_config` |
//! | `plotly` | the above, plus `to_plot`, `write_html`, `show`, `render`, `to_interactive_html` |
//! | `static_export` | the above, plus `write_png`, `write_svg` |
//!
//! [`visualization::OutputType`] has the same variants everywhere; asking
//! `render` for a PNG or SVG without `static_export` returns
//! [`error::GraphError::Render`] rather than doing nothing.
//!
//! ## Adapters
//!
//! Every chart of a library type is a `Graph` implementation in this crate.
//! The orphan rule allows `impl Graph for T` only in the crate of `Graph` or
//! in the crate of `T`, and no lower crate may depend on this one, so every
//! adapter lives here: the lower crates name no chart type, and no domain
//! type is wrapped or copied to chart it.
//!
//! | Type | Defining crate | Adapter module | Chart |
//! | --- | --- | --- | --- |
//! | `Options`, `Position` | core | `model_impls` | payoff at expiry, split into a positive and a negative series |
//! | `Curve`, `Vec<Curve>` | math | `curves` | one series per curve |
//! | `Surface` | math | `surfaces` | one 3D surface |
//! | `RandomWalk`, `Simulator` | simulation | `simulation` | one price series per walk |
//! | the 22 concrete strategies | strategies | `strategies` ([`impl_graph_for_payoff_strategy!`]) | profit and loss over the strategy's price range, with the break-even, current-price and P&L markers |
//! | [`visualization::PlotBuilder`] | visualization | `plot_builder` | the wrapped value's data under the builder's configuration |
//!
//! `tests/graph_data_golden_test.rs` pins the data and configuration of
//! every row against the output of the same adapters before they moved
//! into this crate.
//!
//! ## Charting a strategy
//!
//! The strategy contract has no `Graph` supertrait: a strategy is charted
//! because this crate implements `Graph` for it, so code that renders names
//! the bound (`T: Strategies + Graph`) and code that does not never builds
//! this crate. The capability this changes is the boxed strategy:
//! `StrategyRequest::get_strategy` returns a `Box<dyn Strategable>`, which
//! has no chart. A consumer that renders a strategy built from positions
//! names the concrete type, through `StrategyConstructor`, or keeps its own
//! object type with both bounds:
//!
//! ```rust
//! use optionstratlib_core::model::{ExpirationDate, Position, Positive};
//! use optionstratlib_core::pos_or_panic;
//! use optionstratlib_strategies::strategies::base::{Positionable, StrategyType};
//! use optionstratlib_strategies::strategies::{
//!     BullCallSpread, Strategable, Strategies, StrategyConstructor, StrategyRequest,
//! };
//! use optionstratlib_visualization::visualization::{Graph, GraphData};
//! use rust_decimal::Decimal;
//!
//! /// A consumer that renders names the bound itself.
//! fn payoff_series<S: Strategies + Graph>(strategy: &S) -> usize {
//!     match strategy.graph_data() {
//!         GraphData::MultiSeries(series) => series.len(),
//!         GraphData::Series(_) => 1,
//!         GraphData::GraphSurface(_) => 0,
//!     }
//! }
//!
//! /// An object type for strategies that can be charted.
//! trait Chartable: Strategable + Graph {}
//! impl<T: Strategable + Graph> Chartable for T {}
//!
//! /// Builds the strategy a request names, as a chartable object.
//! fn build(request: &StrategyRequest) -> Result<Box<dyn Chartable>, Box<dyn std::error::Error>> {
//!     match &request.strategy_type {
//!         StrategyType::BullCallSpread => {
//!             Ok(Box::new(BullCallSpread::get_strategy(&request.positions)?))
//!         }
//!         other => Err(format!("{other:?} is not charted here").into()),
//!     }
//! }
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let strategy = BullCallSpread::new(
//!         "AAPL".to_string(),
//!         pos_or_panic!(100.0),
//!         pos_or_panic!(95.0),
//!         pos_or_panic!(105.0),
//!         ExpirationDate::Days(pos_or_panic!(30.0)),
//!         pos_or_panic!(0.25),
//!         Decimal::new(5, 2),   // risk-free rate, 5% per year
//!         Positive::ZERO,       // dividend yield
//!         Positive::ONE,        // quantity
//!         pos_or_panic!(2.50),  // long call premium
//!         pos_or_panic!(1.20),  // short call premium
//!         Positive::ZERO,       // open and close fees of each leg
//!         Positive::ZERO,
//!         Positive::ZERO,
//!         Positive::ZERO,
//!     )?;
//!     // The payoff chart: profit and loss over the strategy's price range.
//!     assert!(payoff_series(&strategy) > 0);
//!     // With `plotly`, the same value renders; `write_html` writes the page.
//!     #[cfg(feature = "plotly")]
//!     assert!(!strategy.to_plot().to_json().is_empty());
//!
//!     // The same strategy as a request carries it, as positions: built back
//!     // through the builder, it charts the same.
//!     let positions: Vec<Position> = strategy.get_positions()?.into_iter().cloned().collect();
//!     let request = StrategyRequest::new(StrategyType::BullCallSpread, positions);
//!     let rebuilt = build(&request)?;
//!     assert_eq!(rebuilt.graph_data(), strategy.graph_data());
//!     Ok(())
//! }
//! ```
//!
//! ## Imports
//!
//! There is no prelude, as in the other component crates (ADR-0001 D7): the
//! module roots re-export each capability, and the `optionstratlib` facade
//! prelude serves broad imports.
//!
//! ## Place in the workspace
//!
//! - **Depends on** `optionstratlib-core`, `-math`, `-pricing`, `-simulation`,
//!   `-market`, `-strategies` and `-backtest`.
//! - **Must not be a dependency** of any other OptionStratLib crate: it is the
//!   leaf of the layering, and only the `optionstratlib` facade depends on it;
//!   `make check-graph` enforces the layering (ADR-0001 D9).
//! - **In the facade:** `optionstratlib::visualization`, `GraphError` and the
//!   aggregate `Error` in `optionstratlib::error` and the root
//!   `impl_graph_for_payoff_strategy!`, under the facade feature `visualization`;
//!   the facade's `plotly` and `static_export` forward to the features below. The
//!   facade paths are the same types as the paths here; the [ownership
//!   map](https://github.com/joaquinbejar/OptionStratLib/blob/main/docs/ownership.md)
//!   lists every one with its feature.
//!
//! Moving from 0.21? The [0.22 architecture and adoption
//! guide](https://github.com/joaquinbejar/OptionStratLib/blob/main/docs/migration-0.22.md)
//! maps each redesign to its replacement workflow; 0.22 keeps no
//! compatibility with 0.21.
//!
//! ## Example
//!
//! A runnable program that depends on this crate directly, with the smallest
//! dependency set and no facade, is [`osl-example-direct-visualization`](https://github.com/joaquinbejar/OptionStratLib/tree/main/examples/direct/visualization); `make tree-example-direct-visualization`
//! asserts its resolved graph.
//!
//! ## Features
//!
//! - `plotly` (off by default): Plotly rendering and the trace builders
//!   (`visualization::make_scatter`, `visualization::make_surface`).
//!   Resolves no image-export, WebDriver or async runtime package.
//! - `static_export` (off by default, implies `plotly`): PNG and SVG export
//!   through `plotly_static`, which needs a WebDriver at run time.

/// Chart models, the `Graph` contract and its implementations.
pub mod visualization;

/// Errors raised by the visualization layer.
pub mod error;

/// Items `impl_graph_for_payoff_strategy!` names through `$crate`, so a caller
/// needs no imports of its own. Not part of the public API.
#[doc(hidden)]
pub mod __private {
    pub use optionstratlib_core::model::Positive;
    pub use optionstratlib_pricing::pricing::Profit;
    pub use optionstratlib_strategies::strategies::base::BreakEvenable;
    pub use optionstratlib_strategies::strategies::{BasicAble, Strategies};
    pub use rust_decimal::Decimal;
    pub use tracing;
}

/// Version of the `optionstratlib-visualization` crate.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Compiles the Rust examples of this crate's `README.md` as doctests, so the
/// README cannot drift from the API (#554). Exists only under `cfg(doctest)`.
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
pub struct ReadmeDoctests;
