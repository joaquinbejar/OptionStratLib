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
//! #923 adds [`SvgText::Outlines`] ([`outline`]): SVG text drawn as glyph
//! outlines, so the SVG needs no font in any viewer, with a text fixture
//! and the viewer evidence of `REPORT-outlines.md`.
//!
//! [`GraphData`]: optionstratlib_visualization::visualization::GraphData
//! [`GraphConfig`]: optionstratlib_visualization::visualization::GraphConfig

mod backend;
mod color;
pub mod error;
pub mod fixtures;
pub mod outline;
pub mod render;
pub mod svg;
mod text;

pub use error::ExportError;
pub use outline::{Label, Outlined};
pub use render::{SvgText, render_svg, render_svg_outlined, write_png, write_svg};
