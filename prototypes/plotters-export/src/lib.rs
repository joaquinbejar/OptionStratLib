//! Prototype for #920: native PNG and SVG export of OptionStratLib charts
//! with Plotters, to compare with the current Plotly export.
//!
//! The Plotly path (`Graph::write_png` / `write_svg` under `static_export`)
//! drives a headless Chrome through chromedriver for every image. This
//! crate draws the same backend-neutral [`GraphData`] and [`GraphConfig`]
//! natively instead: [`render::write_png`] and [`render::write_svg`] start
//! no process and need no network.
//!
//! [`fixtures`] builds the three charts of the comparison, and the
//! `compare` binary writes all twelve artifacts (three charts, two formats,
//! two renderers) and times them. Nothing here changes a published crate;
//! the decision report is `REPORT.md`.
//!
//! [`GraphData`]: optionstratlib_visualization::visualization::GraphData
//! [`GraphConfig`]: optionstratlib_visualization::visualization::GraphConfig

pub mod error;
pub mod fixtures;
pub mod render;
pub mod svg;

pub use error::ExportError;
pub use render::{SvgText, render_svg, write_png, write_svg};
