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

use crate::visualization::interface::{GraphDataType, GraphType};
use crate::visualization::styles::{PlotType, TraceMode};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use crate::curves::Curve;
use crate::surfaces::Surface;
use crate::visualization::{get_color_from_scheme, ColorScheme};

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
    pub z: Vec<Decimal>,
    pub name: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum GraphData {
    Series(Series2D),
    MultiSeries(MultiSeries2D),
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

impl From<&Curve> for GraphData {
    fn from(curve: &Curve) -> Self {
        curve.into()
    }
}

impl From<Vec<Curve>> for GraphData {
    fn from(curves: Vec<Curve>) -> Self {
        let color_scheme = ColorScheme::Viridis;

        let series: Vec<Series2D> = curves.into_iter().enumerate().map(|(idx, c)| {
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
        }).collect();

        GraphData::MultiSeries(series)
    }
}

impl From<&Vec<Curve>> for GraphData {
    fn from(curves: &Vec<Curve>) -> Self {
        curves.into()
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

impl From<&Surface> for GraphData {
    fn from(surface: &Surface) -> Self {
        surface.into()
    }
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
