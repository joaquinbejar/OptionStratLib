//! Consumer fixture for the `optionstratlib` facade built with only its
//! `visualization` capability (#544). The crate
//! has no code of its own: `tests/consumer.rs` uses the facade the way a
//! downstream service would, and `expect.toml` lists the packages its graph
//! must and must not resolve.
//!
//! A chart consumer without a rendering backend has `graph_data` and
//! `graph_config` and nothing that renders: asking the contract for a Plotly
//! figure does not compile.
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
//! fn html<G: Graph>(graph: &G, path: &std::path::Path) {
//!     let _ = graph.write_html(path);
//! }
//! ```
//!
//! The Plotly trace builders are behind `plotly` too.
//!
//! ```compile_fail,E0432
//! use optionstratlib::visualization::make_scatter;
//! ```
