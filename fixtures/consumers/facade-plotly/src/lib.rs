//! Consumer fixture for the `optionstratlib` facade built with only its
//! default features and `plotly` (#544). The crate has no code of its own:
//! `tests/consumer.rs` uses the facade the way a downstream service would, and
//! `expect.toml` lists the packages its graph must and must not resolve.
//!
//! `plotly` adds the interactive methods to `Graph` and nothing for image
//! export: PNG and SVG writers are not there to call.
//!
//! ```compile_fail,E0599
//! use optionstratlib::prelude::*;
//!
//! fn export<G: Graph>(graph: &G, path: &std::path::Path) {
//!     let _ = graph.write_png(path);
//! }
//! ```
//!
//! ```compile_fail,E0599
//! use optionstratlib::prelude::*;
//!
//! fn export<G: Graph>(graph: &G, path: &std::path::Path) {
//!     let _ = graph.write_svg(path);
//! }
//! ```
