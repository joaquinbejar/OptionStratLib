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

use std::path::PathBuf;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use crate::visualization::interface::GraphType;
use crate::visualization::styles::{PlotType, TraceMode};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Series2D {
    pub x: Vec<Decimal>,
    pub y: Vec<Decimal>,
    pub name: String,
    pub mode: TraceMode,
    pub line_color: Option<String>,
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

pub type MultiSeries2D = Vec<Series2D>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct Surface3D {
    pub x: Vec<Decimal>,
    pub y: Vec<Decimal>,
    pub z: Vec<Vec<Decimal>>, 
    pub name: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum GraphData {
    Series(Series2D),
    MultiSeries(MultiSeries2D),
    Surface(Surface3D),
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub enum OutputType<'a> {
    Png(&'a PathBuf),
    Html(&'a PathBuf),
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

