//! # Visualization
//!
//! # Visualization Library Usage Guide
//!
//! This guide explains how to use the plotly.rs-based visualization library to create financial charts and other types of visualizations.
//!
//! ## Setup
//!
//! First, ensure you have the correct dependencies in your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! plotly = "0.12.1"
//! serde = { version = "1.0", features = ["derive"] }
//! ```
//!
//! ## Core Concepts
//!
//! The library follows a domain modeling pattern where it separates:
//!
//! 1. **What to visualize** (the data to represent)
//! 2. **How to visualize it** (styles and configuration)
//! 3. **Where to display it** (browser, HTML file, PNG image)
//!
//! ### Main Trait: `Graph`
//!
//! The `Graph` trait is the central component that any object wanting to be visualized must implement:
//!
//! ```rust,ignore,no_run
//! use optionstratlib::visualization::{GraphConfig, GraphData};
//!
//! pub trait Graph {
//!     fn graph_data(&self) -> GraphData;
//!     
//!     fn graph_config(&self) -> GraphConfig {
//!         GraphConfig::default()
//!     }
//!     
//!     // Additional methods provided by default...
//! }
//! ```
//!
//! To create a chart, you only need to implement:
//! - `graph_data()`: to provide the data to visualize
//! - Optionally, `graph_config()`: to customize the appearance
//!
//! ### Data Types
//!
//! The library supports these types of visualizations through the `GraphData` enum:
//!
//! ```rust
//! use optionstratlib::visualization::{MultiSeries2D, Series2D, Surface3D};
//!
//! pub enum GraphData {
//!     Series(Series2D),              // Line or scatter 2D
//!     MultiSeries(MultiSeries2D),    // Multiple 2D series
//!     Surface(Surface3D),            // 3D surface
//! }
//! ```
//!
//! ## Example: Simple Line Chart
//!
//! ```rust
//! use std::fs;
//! use std::path::{Path, PathBuf};
//! use rust_decimal::Decimal;
//! use rust_decimal_macros::dec;
//! use optionstratlib::visualization::{Graph, GraphData, Series2D, GraphConfig, OutputType};
//! use optionstratlib::visualization::{LineStyle, ColorScheme, TraceMode};
//!
//! struct MyData {
//!     x: Vec<Decimal>,
//!     y: Vec<Decimal>,
//! }
//!
//! impl Graph for MyData {
//!     fn graph_data(&self) -> GraphData {
//!         GraphData::Series(Series2D {
//!             x: self.x.clone(),
//!             y: self.y.clone(),
//!             name: "My series".to_string(),
//!             mode: TraceMode::Lines,
//!             line_color: Some("#1f77b4".to_string()),
//!             line_width: Some(2.0),
//!         })
//!     }
//!
//!     fn graph_config(&self) -> GraphConfig {
//!         GraphConfig {
//!             title: "My chart".to_string(),
//!             width: 800,
//!             height: 600,
//!             x_label: Some("X Axis".to_string()),
//!             y_label: Some("Y Axis".to_string()),
//!             z_label: None,
//!             line_style: LineStyle::Solid,
//!             color_scheme: ColorScheme::Viridis,
//!             legend: None,
//!             show_legend: false,
//!         }
//!     }
//! }
//!
//! // Using the chart
//! let data = MyData {
//!     x: vec![dec!(1.0), dec!(2.0), dec!(3.0), dec!(4.0), dec!(5.0)],
//!     y: vec![dec!(2.0), dec!(3.0), dec!(5.0), dec!(7.0), dec!(11.0)],
//! };
//!
//! // Display in browser
//! data.show();
//! // Save as PNG
//! let filename: PathBuf = PathBuf::from("my_chart.png");
//! data.render(OutputType::Png(&filename)).unwrap();
//! if Path::new(&filename.clone()).exists() {
//!         fs::remove_file(filename.clone())
//!             .unwrap_or_else(|_| panic!("Failed to remove {}", filename.to_str().unwrap()));
//! }
//! ```
//!
//! ## Example: 3D Surface
//!
//! ```rust
//!
//! use rust_decimal::Decimal;
//! use optionstratlib::visualization::{ColorScheme, Graph, GraphConfig, GraphData, LineStyle, Surface3D};
//!
//! struct SurfaceData {
//!     x: Vec<Decimal>,
//!     y: Vec<Decimal>,
//!     z: Vec<Decimal>,
//! }
//!
//! impl Graph for SurfaceData {
//!     fn graph_data(&self) -> GraphData {
//!         GraphData::Surface(Surface3D {
//!             x: self.x.clone(),
//!             y: self.y.clone(),
//!             z: self.z.clone(),
//!             name: "My surface".to_string(),
//!         })
//!     }
//!
//!     fn graph_config(&self) -> GraphConfig {
//!         GraphConfig {
//!             title: "3D Surface".to_string(),
//!             width: 800,
//!             height: 600,
//!             x_label: Some("X".to_string()),
//!             y_label: Some("Y".to_string()),
//!             z_label: Some("Z".to_string()),
//!             line_style: LineStyle::Solid,
//!             color_scheme: ColorScheme::Plasma,
//!             legend: None,
//!             show_legend: false,
//!          }
//!     }
//! }
//! ```
//!
//! ## Color Schemes
//!
//! The library provides several predefined color schemes:
//!
//! - `ColorScheme::Default`: Uses plotly's default colors
//! - `ColorScheme::Viridis`: A color palette ranging from violet to yellow
//! - `ColorScheme::Plasma`: A color palette ranging from dark blue to yellow
//! - `ColorScheme::Custom(Vec<String>)`: Define your own colors as hexadecimal strings
//!
//! ## Line Styles
//!
//! You can customize lines with:
//!
//! - `LineStyle::Solid`: Continuous line
//! - `LineStyle::Dotted`: Dotted line
//! - `LineStyle::Dashed`: Dashed line
//!
//! ## Display Modes
//!
//! For 2D series, you can choose how to display points:
//!
//! - `TraceMode::Lines`: Lines only
//! - `TraceMode::Markers`: Markers (points) only
//! - `TraceMode::LinesMarkers`: Both lines and markers
//!
//! ## Error Handling
//!
//! The library provides a `GraphError` type that encapsulates graph-specific errors and I/O errors. All methods that can fail return `Result<(), GraphError>`.
//!
//! ## Advanced Tips
//!
//! ### Serialization and Deserialization
//!
//! All main types implement `serde::Serialize` and `serde::Deserialize`, allowing you to save and load chart configurations in JSON or other compatible formats.
//!
//! ### Multiple Series
//!
//! For charts with multiple series, use the `GraphData::MultiSeries` type:
//!
//! ```rust
//! use rust_decimal_macros::dec;
//! use optionstratlib::visualization::{GraphData, Series2D, TraceMode};
//! let series1 = Series2D {
//!     x: vec![dec!(1.0), dec!(2.0), dec!(3.0)],
//!     y: vec![dec!(4.0), dec!(5.0), dec!(6.0)],
//!     name: "Series 1".to_string(),
//!     mode: TraceMode::Lines,
//!     line_color: Some("#1f77b4".to_string()),
//!     line_width: Some(2.0),
//! };
//!
//! let series2 = Series2D {
//!     x: vec![dec!(1.0), dec!(2.0), dec!(3.0)],
//!     y: vec![dec!(7.0), dec!(8.0), dec!(9.0)],
//!     name: "Series 2".to_string(),
//!     mode: TraceMode::Markers,
//!     line_color: Some("#ff7f0e".to_string()),
//!     line_width: Some(2.0),
//! };
//!
//! let graph_data = GraphData::MultiSeries(vec![series1, series2]);
//! ```
//!
//! ### Interactive HTML Generation
//!
//! If you need interactive HTML with advanced hover and tooltip functions, use the `to_interactive_html` method:
//!
//! ```rust
//! use optionstratlib::visualization::{Graph, GraphData, Series2D, TraceMode, GraphConfig};
//! use optionstratlib::error::GraphError;
//! use std::path::PathBuf;
//! use tracing::info;
//! use rust_decimal_macros::dec;
//! struct SimpleChart {
//!     series: Series2D
//! }
//!
//! impl Graph for SimpleChart {
//!     fn graph_data(&self) -> GraphData {
//!         GraphData::Series(self.series.clone())
//!     }
//!     
//!     fn graph_config(&self) -> GraphConfig {
//!         GraphConfig {
//!             title: "Interactive Chart Example".into(),
//!             width: 800,
//!             height: 600,
//!             x_label: Some("X Axis".into()),
//!             y_label: Some("Y Axis".into()),
//!             ..GraphConfig::default()
//!         }
//!     }
//! }
//!
//! fn main() -> Result<(), GraphError> {
//!     use std::fs;
//! use std::path::Path;
//! let series = Series2D {
//!         x: vec![dec!(1.0), dec!(2.0), dec!(3.0)],
//!         y: vec![dec!(4.0), dec!(5.0), dec!(6.0)],
//!         name: "Series 1".to_string(),
//!         mode: TraceMode::Lines,
//!         line_color: Some("#1f77b4".to_string()),
//!         line_width: Some(2.0),
//!     };
//!     
//!     let chart = SimpleChart { series };
//!     let filename: PathBuf = PathBuf::from("interactive_chart.html");
//!     chart.to_interactive_html(&filename)?;
//!     info!("Interactive HTML chart created successfully!");
//!     if Path::new(&filename.clone()).exists() {
//!             fs::remove_file(filename.clone())
//!                 .unwrap_or_else(|_| panic!("Failed to remove {}", filename.to_str().unwrap()));
//!         }
//!     Ok(())
//! }
//! ```
//!
//! ## Complete Examples
//!
//! Check the `examples/` directory for practical examples, including:
//! - Stock price charts with moving averages
//! - Volatility surfaces for options
//! - Scatter plots with multiple series
//!
//! ## Adaptation to Your Needs
//!
//! The library is designed to be extensible. If you need additional chart types, you can:
//!
//! 1. Extend the `GraphData` enum with new types
//! 2. Implement conversion functions in the `utils.rs` module
//! 3. Update the `to_plot` method in the `Graph` trait to handle the new types
//!
//! Enjoy visualizing your financial data!

mod config;
mod interface;
mod model;
mod styles;
mod tests;
mod utils;

#[cfg(not(feature = "plotly"))]
mod default;
#[cfg(feature = "plotly")]
mod plotly;

#[cfg(feature = "plotly")]
mod test_plotly;

#[cfg(feature = "plotly")]
pub use {
    plotly::Graph,
    utils::{make_scatter, make_surface, pick_color, to_plotly_mode},
};

#[cfg(not(feature = "plotly"))]
pub use default::Graph;

pub use config::GraphConfig;
pub use interface::GraphType;
pub use model::{GraphData, MultiSeries2D, OutputType, Series2D, Surface3D};
pub use styles::{ColorScheme, LineStyle, PlotType, TraceMode};
pub use utils::get_color_from_scheme;
