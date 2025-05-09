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
//! use optionstratlib::visualization::{Graph, GraphData, Series2D, GraphConfig, OutputType};
//! use optionstratlib::visualization::{LineStyle, ColorScheme, TraceMode};
//!
//! struct MyData {
//!     x: Vec<f64>,
//!     y: Vec<f64>,
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
//!         }
//!     }
//! }
//!
//! // Using the chart
//! let data = MyData {
//!     x: vec![1.0, 2.0, 3.0, 4.0, 5.0],
//!     y: vec![2.0, 3.0, 5.0, 7.0, 11.0],
//! };
//!
//! // Display in browser
//! data.show();
//!
//! // Save as HTML
//! data.render(OutputType::Html("my_chart.html".into()))?;
//!
//! // Save as PNG
//! data.render(OutputType::Png("my_chart.png".into()))?;
//! ```
//! 
//! ## Example: 3D Surface
//! 
//! ```rust
//!
//! use optionstratlib::visualization::{ColorScheme, Graph, GraphConfig, GraphData, LineStyle, Surface3D};
//!
//! struct SurfaceData {
//!     x: Vec<f64>,
//!     y: Vec<f64>,
//!     z: Vec<Vec<f64>>,
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
//!         }
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
//! use optionstratlib::visualization::{GraphData, Series2D, TraceMode};
//! let series1 = Series2D { 
//!     x: vec![1.0, 2.0, 3.0], 
//!     y: vec![4.0, 5.0, 6.0],
//!     name: "Series 1".to_string(),
//!     mode: TraceMode::Lines,
//!     line_color: Some("#1f77b4".to_string()),
//!     line_width: Some(2.0),
//! };
//!
//! let series2 = Series2D { 
//!     x: vec![1.0, 2.0, 3.0], 
//!     y: vec![7.0, 8.0, 9.0],
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
//!
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
//!     let series = Series2D { 
//!         x: vec![1.0, 2.0, 3.0], 
//!         y: vec![4.0, 5.0, 6.0],
//!         name: "Series 1".to_string(),
//!         mode: TraceMode::Lines,
//!         line_color: Some("#1f77b4".to_string()),
//!         line_width: Some(2.0),
//!     };
//!     
//!     let chart = SimpleChart { series };
//!     chart.to_interactive_html("interactive_chart.html")?;
//!     info!("Interactive HTML chart created successfully!");
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

mod model;
mod utils;
mod interface;
mod config;
mod styles;

pub use model::{Series2D, Surface3D, GraphData, OutputType, MultiSeries2D};
pub use interface::{Graph, GraphType, GraphDataType};
pub use config::GraphConfig;
pub use styles::{LineStyle, ColorScheme, TraceMode, PlotType};
pub use utils::{pick_color, get_color_from_scheme, to_plotly_mode};