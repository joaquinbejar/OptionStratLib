//! Centralised Graph trait powered by `plotly.rs`
//!
//! This supersedes the old `plotters` spaghetti and now offers:
//!
//! * **Single‑series 2‑D** (`Series2D`)
//! * **Multi‑series 2‑D** (`MultiSeries2D`)
//! * **3‑D surfaces**      (`Surface3D`)
//!
//! Out of the box you get PNG export, HTML export, inline browser view,
//! sensible error handling and a growing bag of style knobs (titles,
//! axis labels, colour schemes, line styles, …).
//!
//! Domain objects only describe *what* to plot; *how* lives here.

use crate::curves::Curve;
use crate::surfaces::Surface;
use crate::visualization::interface::GraphType;
use crate::visualization::styles::{PlotType, TraceMode};
use crate::visualization::{ColorScheme, get_color_from_scheme};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Represents a two-dimensional data series for plotting.
///
/// This struct contains the X and Y coordinates of data points, along with
/// styling information to control how the series appears in a plot.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Series2D {
    /// X-coordinates of the data points.
    pub x: Vec<Decimal>,

    /// Y-coordinates of the data points.
    pub y: Vec<Decimal>,

    /// Name of the data series, used for legends and identification.
    pub name: String,

    /// Visual mode of the trace (lines, markers, or both).
    pub mode: TraceMode,

    /// Optional color for the line in hex format (e.g., "#FF0000" for red).
    /// If None, a default color will be used.
    pub line_color: Option<String>,

    /// Optional width of the line in pixels.
    /// If None, a default width will be used.
    pub line_width: Option<f64>,
}

impl Default for Series2D {
    fn default() -> Self {
        Self {
            x: vec![],
            y: vec![],
            name: "Series".into(),
            mode: TraceMode::Lines,
            line_color: None,
            line_width: None,
        }
    }
}

/// A type alias representing a collection of 2D series.
///
/// `MultiSeries2D` is a type alias for a `Vec<Series2D>`, where each `Series2D`
/// represents a single two-dimensional data series. This alias is used to 
/// simplify the representation of multiple 2D series in a single collection.
///
pub type MultiSeries2D = Vec<Series2D>;

/// A struct representing a 3D surface plot.
///
/// This structure holds the 3D coordinates data (x, y, z) as vectors of Decimal values,
/// along with a name to identify the surface in plots.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct Surface3D {
    /// The x-coordinates of the points in the 3D surface
    pub x: Vec<Decimal>,

    /// The y-coordinates of the points in the 3D surface
    pub y: Vec<Decimal>,

    /// The z-coordinates of the points in the 3D surface
    pub z: Vec<Decimal>,

    /// A descriptive name for the 3D surface
    pub name: String,
}


/// Represents different types of graph data for visualization.
///
/// This enum encapsulates various data structures that can be used for 
/// rendering different types of charts and plots.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum GraphData {
    /// A single 2D data series containing x and y coordinates.
    /// Typically used for line, scatter, or bar charts with a single dataset.
    Series(Series2D),

    /// Multiple 2D data series for comparison in the same visualization.
    /// Useful for charts that need to display multiple datasets simultaneously.
    MultiSeries(MultiSeries2D),

    /// A 3D surface representation with x, y, and z coordinates.
    /// Used for creating 3D surface plots and visualizations.
    Surface(Surface3D),
}

impl From<Curve> for GraphData {
    fn from(curve: Curve) -> Self {
        GraphData::Series(Series2D {
            x: curve.points.iter().map(|p| p.x).collect(),
            y: curve.points.iter().map(|p| p.y).collect(),
            name: "Curve".to_string(),
            mode: TraceMode::Lines,
            line_color: Some("#1f77b4".to_string()),
            line_width: Some(2.0),
        })
    }
}

impl From<Vec<Curve>> for GraphData {
    fn from(curves: Vec<Curve>) -> Self {
        let color_scheme = ColorScheme::Plasma;

        let series: Vec<Series2D> = curves
            .into_iter()
            .enumerate()
            .map(|(idx, c)| {
                let color = get_color_from_scheme(&color_scheme, idx)
                    .unwrap_or_else(|| "#1f77b4".to_string());

                Series2D {
                    x: c.points.iter().map(|p| p.x).collect(),
                    y: c.points.iter().map(|p| p.y).collect(),
                    name: format!("Curve {}", idx + 1),
                    mode: TraceMode::Lines,
                    line_color: Some(color),
                    line_width: Some(2.0),
                }
            })
            .collect();

        GraphData::MultiSeries(series)
    }
}

impl From<Surface> for GraphData {
    fn from(surface: Surface) -> Self {
        GraphData::Surface(Surface3D {
            x: surface.points.iter().map(|p| p.x).collect(),
            y: surface.points.iter().map(|p| p.y).collect(),
            z: surface.points.iter().map(|p| p.z).collect(),
            name: "Surface".to_string(),
        })
    }
}

/// Represents the different output types for saving or displaying plots.
///
/// This enum allows specifying whether the output should be saved to a file
/// in a specific format (PNG, HTML, SVG) or displayed directly in a browser.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub enum OutputType<'a> {
    /// PNG image output with a reference to the file path where it should be saved.
    Png(&'a PathBuf),

    /// HTML document output with a reference to the file path where it should be saved.
    Html(&'a PathBuf),

    /// SVG image output with a reference to the file path where it should be saved.
    Svg(&'a PathBuf),

    /// Output directly to the default web browser without saving to a file.
    Browser,
}

impl GraphType for Series2D {
    fn plot_type() -> PlotType {
        PlotType::Line2D
    }
}

impl GraphType for Surface3D {
    fn plot_type() -> PlotType {
        PlotType::Surface3D
    }
}
