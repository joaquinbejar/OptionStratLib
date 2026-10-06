//! Consumer fixture for the `optionstratlib` facade built with its default
//! features (#544). The crate
//! has no code of its own: `tests/consumer.rs` uses the facade the way a
//! downstream service would, and `expect.toml` lists the packages its graph
//! must and must not resolve.
//!
//! The default surface is the whole library without a rendering backend, so
//! the Plotly figure is not there to call.
//!
//! ```compile_fail,E0599
//! use optionstratlib::prelude::*;
//!
//! fn figure<G: Graph>(graph: &G) {
//!     let _ = graph.to_plot();
//! }
//! ```
//!
//! ```compile_fail,E0599
//! use optionstratlib::prelude::*;
//!
//! fn export<G: Graph>(graph: &G, path: &std::path::Path) {
//!     let _ = graph.write_png(path);
//! }
//! ```
//!
//! ```compile_fail,E0432
//! use optionstratlib::visualization::make_scatter;
//! ```
