:::::::::::::: width-limiter
::::::::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../index.html)
:::

# Module visualization Copy item path

[[Source](../../src/optionstratlib/visualization/mod.rs.html#1-328){.src}
]{.sub-heading}
::::

Expand description

:::::::::: docblock
- `visualization` - Tools for plotting and visual representation of
  options data.

Graphics and visualization utilities for creating charts, graphs, and
interactive plots of options data, strategies, and analytics. Supports
various plot types optimized for different aspects of options analysis.

## [§](#visualization){.doc-anchor}Visualization

## [§](#visualization-library-usage-guide){.doc-anchor}Visualization Library Usage Guide

This guide explains how to use the plotly.rs-based visualization library
to create financial charts and other types of visualizations.

### [§](#setup){.doc-anchor}Setup

First, ensure you have the correct dependencies in your `Cargo.toml`:

::: example-wrap
``` language-toml
[dependencies]
plotly = "0.12.1"
serde = { version = "1.0", features = ["derive"] }
```
:::

### [§](#core-concepts){.doc-anchor}Core Concepts

The library follows a domain modeling pattern where it separates:

1.  **What to visualize** (the data to represent)
2.  **How to visualize it** (styles and configuration)
3.  **Where to display it** (browser, HTML file, PNG image)

#### [§](#main-trait-graph){.doc-anchor}Main Trait: `Graph`

The `Graph` trait is the central component that any object wanting to be
visualized must implement:

::: {.example-wrap .ignore}
[ⓘ](# "This example is not tested"){.tooltip}

``` {.rust .rust-example-rendered}
use optionstratlib::visualization::{GraphConfig, GraphData};

pub trait Graph {
    fn graph_data(&self) -> GraphData;
     
    fn graph_config(&self) -> GraphConfig {
        GraphConfig::default()
    }
     
    // Additional methods provided by default...
}
```
:::

To create a chart, you only need to implement:

- `graph_data()`: to provide the data to visualize
- Optionally, `graph_config()`: to customize the appearance

#### [§](#data-types){.doc-anchor}Data Types

The library supports these types of visualizations through the
`GraphData` enum:

::: example-wrap
``` {.rust .rust-example-rendered}
use optionstratlib::visualization::{MultiSeries2D, Series2D, Surface3D};

pub enum GraphData {
    Series(Series2D),              // Line or scatter 2D
    MultiSeries(MultiSeries2D),    // Multiple 2D series
    Surface(Surface3D),            // 3D surface
}
```
:::

### [§](#example-simple-line-chart){.doc-anchor}Example: Simple Line Chart

::: example-wrap
``` {.rust .rust-example-rendered}
use std::fs;
use std::path::{Path, PathBuf};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use optionstratlib::visualization::{Graph, GraphData, Series2D, GraphConfig, OutputType};
use optionstratlib::visualization::{LineStyle, ColorScheme, TraceMode};

struct MyData {
    x: Vec<Decimal>,
    y: Vec<Decimal>,
}

impl Graph for MyData {
    fn graph_data(&self) -> GraphData {
        GraphData::Series(Series2D {
            x: self.x.clone(),
            y: self.y.clone(),
            name: "My series".to_string(),
            mode: TraceMode::Lines,
            line_color: Some("#1f77b4".to_string()),
            line_width: Some(2.0),
        })
    }

    fn graph_config(&self) -> GraphConfig {
        GraphConfig {
            title: "My chart".to_string(),
            width: 800,
            height: 600,
            x_label: Some("X Axis".to_string()),
            y_label: Some("Y Axis".to_string()),
            z_label: None,
            line_style: LineStyle::Solid,
            color_scheme: ColorScheme::Viridis,
            legend: None,
            show_legend: false,
        }
    }
}
#[cfg(feature = "static_export")]
{
    // Using the chart
    let data = MyData {
        x: vec![dec!(1.0), dec!(2.0), dec!(3.0), dec!(4.0), dec!(5.0)],
        y: vec![dec!(2.0), dec!(3.0), dec!(5.0), dec!(7.0), dec!(11.0)],
    };
    // Display in browser
    data.show();
    // Save as PNG
    let filename: PathBuf = PathBuf::from("my_chart.png");
    data.render(OutputType::Png(&filename)).unwrap();
    if Path::new(&filename.clone()).exists() {
            fs::remove_file(filename.clone())
                .unwrap_or_else(|_| panic!("Failed to remove {}", filename.to_str().unwrap()));
    }
}
```
:::

### [§](#example-3d-surface){.doc-anchor}Example: 3D Surface

::: example-wrap
``` {.rust .rust-example-rendered}

use rust_decimal::Decimal;
use optionstratlib::visualization::{ColorScheme, Graph, GraphConfig, GraphData, LineStyle, Surface3D};

struct SurfaceData {
    x: Vec<Decimal>,
    y: Vec<Decimal>,
    z: Vec<Decimal>,
}

impl Graph for SurfaceData {
    fn graph_data(&self) -> GraphData {
        GraphData::GraphSurface(Surface3D {
            x: self.x.clone(),
            y: self.y.clone(),
            z: self.z.clone(),
            name: "My surface".to_string(),
        })
    }

    fn graph_config(&self) -> GraphConfig {
        GraphConfig {
            title: "3D Surface".to_string(),
            width: 800,
            height: 600,
            x_label: Some("X".to_string()),
            y_label: Some("Y".to_string()),
            z_label: Some("Z".to_string()),
            line_style: LineStyle::Solid,
            color_scheme: ColorScheme::Plasma,
            legend: None,
            show_legend: false,
         }
    }
}
```
:::

### [§](#color-schemes){.doc-anchor}Color Schemes

The library provides several predefined color schemes:

- `ColorScheme::Default`: Uses plotly's default colors
- `ColorScheme::Viridis`: A color palette ranging from violet to yellow
- `ColorScheme::Plasma`: A color palette ranging from dark blue to
  yellow
- `ColorScheme::Custom(Vec<String>)`: Define your own colors as
  hexadecimal strings

### [§](#line-styles){.doc-anchor}Line Styles

You can customize lines with:

- `LineStyle::Solid`: Continuous line
- `LineStyle::Dotted`: Dotted line
- `LineStyle::Dashed`: Dashed line

### [§](#display-modes){.doc-anchor}Display Modes

For 2D series, you can choose how to display points:

- `TraceMode::Lines`: Lines only
- `TraceMode::Markers`: Markers (points) only
- `TraceMode::LinesMarkers`: Both lines and markers

### [§](#error-handling){.doc-anchor}Error Handling

The library provides a `GraphError` type that encapsulates
graph-specific errors and I/O errors. All methods that can fail return
`Result<(), GraphError>`.

### [§](#advanced-tips){.doc-anchor}Advanced Tips

#### [§](#serialization-and-deserialization){.doc-anchor}Serialization and Deserialization

All main types implement `serde::Serialize` and `serde::Deserialize`,
allowing you to save and load chart configurations in JSON or other
compatible formats.

#### [§](#multiple-series){.doc-anchor}Multiple Series

For charts with multiple series, use the `GraphData::MultiSeries` type:

::: example-wrap
``` {.rust .rust-example-rendered}
use rust_decimal_macros::dec;
use optionstratlib::visualization::{GraphData, Series2D, TraceMode};
let series1 = Series2D {
    x: vec![dec!(1.0), dec!(2.0), dec!(3.0)],
    y: vec![dec!(4.0), dec!(5.0), dec!(6.0)],
    name: "Series 1".to_string(),
    mode: TraceMode::Lines,
    line_color: Some("#1f77b4".to_string()),
    line_width: Some(2.0),
};

let series2 = Series2D {
    x: vec![dec!(1.0), dec!(2.0), dec!(3.0)],
    y: vec![dec!(7.0), dec!(8.0), dec!(9.0)],
    name: "Series 2".to_string(),
    mode: TraceMode::Markers,
    line_color: Some("#ff7f0e".to_string()),
    line_width: Some(2.0),
};

let graph_data = GraphData::MultiSeries(vec![series1, series2]);
```
:::

#### [§](#interactive-html-generation){.doc-anchor}Interactive HTML Generation

If you need interactive HTML with advanced hover and tooltip functions,
use the `to_interactive_html` method:

::: example-wrap
``` {.rust .rust-example-rendered}
use optionstratlib::visualization::{Graph, GraphData, Series2D, TraceMode, GraphConfig};
use optionstratlib::error::GraphError;
use std::path::PathBuf;
use tracing::info;
use rust_decimal_macros::dec;
struct SimpleChart {
    series: Series2D
}

impl Graph for SimpleChart {
    fn graph_data(&self) -> GraphData {
        GraphData::Series(self.series.clone())
    }
     
    fn graph_config(&self) -> GraphConfig {
        GraphConfig {
            title: "Interactive Chart Example".into(),
            width: 800,
            height: 600,
            x_label: Some("X Axis".into()),
            y_label: Some("Y Axis".into()),
            ..GraphConfig::default()
        }
    }
}

fn main() -> Result<(), GraphError> {
    use std::fs;
use std::path::Path;
let series = Series2D {
        x: vec![dec!(1.0), dec!(2.0), dec!(3.0)],
        y: vec![dec!(4.0), dec!(5.0), dec!(6.0)],
        name: "Series 1".to_string(),
        mode: TraceMode::Lines,
        line_color: Some("#1f77b4".to_string()),
        line_width: Some(2.0),
    };
     
    #[cfg(feature = "static_export")]
    {
    let chart = SimpleChart { series };
    let filename: PathBuf = PathBuf::from("interactive_chart.html");
    chart.to_interactive_html(&filename)?;
    info!("Interactive HTML chart created successfully!");
    if Path::new(&filename.clone()).exists() {
            fs::remove_file(filename.clone())
                .unwrap_or_else(|_| panic!("Failed to remove {}", filename.to_str().unwrap()));
        }
    }
    Ok(())
}
```
:::

### [§](#complete-examples){.doc-anchor}Complete Examples

Check the `examples/` directory for practical examples, including:

- Stock price charts with moving averages
- Volatility surfaces for options
- Scatter plots with multiple series

### [§](#adaptation-to-your-needs){.doc-anchor}Adaptation to Your Needs

The library is designed to be extensible. If you need additional chart
types, you can:

1.  Extend the `GraphData` enum with new types
2.  Implement conversion functions in the `utils.rs` module
3.  Update the `to_plot` method in the `Graph` trait to handle the new
    types

Enjoy visualizing your financial data!
::::::::::

## Structs[§](#structs){.anchor} {#structs .section-header}

[GraphConfig](struct.GraphConfig.html "struct optionstratlib::visualization::GraphConfig"){.struct}
:   Represents the configuration parameters for a graph or chart
    visualization.

[Label2D](struct.Label2D.html "struct optionstratlib::visualization::Label2D"){.struct}
:   A text label attached to a 2D point for displaying information on a
    2D plot.

[Label3D](struct.Label3D.html "struct optionstratlib::visualization::Label3D"){.struct}
:   A text label attached to a 3D point for displaying information on a
    3D plot.

[Series2D](struct.Series2D.html "struct optionstratlib::visualization::Series2D"){.struct}
:   Represents a two-dimensional data series for plotting.

[Surface3D](struct.Surface3D.html "struct optionstratlib::visualization::Surface3D"){.struct}
:   A struct representing a 3D surface plot.

[VisPoint2D](struct.VisPoint2D.html "struct optionstratlib::visualization::VisPoint2D"){.struct}
:   A 2D point representation for plotting in a 2D coordinate system.

[VisPoint3D](struct.VisPoint3D.html "struct optionstratlib::visualization::VisPoint3D"){.struct}
:   A 3D point representation for plotting in a 3D coordinate system.

## Enums[§](#enums){.anchor} {#enums .section-header}

[ColorScheme](enum.ColorScheme.html "enum optionstratlib::visualization::ColorScheme"){.enum}
:   Represents the color scheme used for visualizations.

[GraphData](enum.GraphData.html "enum optionstratlib::visualization::GraphData"){.enum}
:   Represents different types of graph data for visualization.

[LineStyle](enum.LineStyle.html "enum optionstratlib::visualization::LineStyle"){.enum}
:   Represents the visual style of a line in graphical elements.

[OutputType](enum.OutputType.html "enum optionstratlib::visualization::OutputType"){.enum}
:   Represents the different output types for saving or displaying
    plots.

[PlotType](enum.PlotType.html "enum optionstratlib::visualization::PlotType"){.enum}
:   Represents different types of plots that can be generated.

[TraceMode](enum.TraceMode.html "enum optionstratlib::visualization::TraceMode"){.enum}
:   Defines the visual appearance of traces in plots.

## Traits[§](#traits){.anchor} {#traits .section-header}

[Graph](trait.Graph.html "trait optionstratlib::visualization::Graph"){.trait}
:   A trait that defines the functionality for creating, configuring,
    and rendering graphical representations of data, along with support
    for various output formats.

[GraphType](trait.GraphType.html "trait optionstratlib::visualization::GraphType"){.trait}
:   A trait defining the type of graph or plot used for visualization
    purposes.

## Functions[§](#functions){.anchor} {#functions .section-header}

[get_color_from_scheme](fn.get_color_from_scheme.html "fn optionstratlib::visualization::get_color_from_scheme"){.fn}
:   Get color from a color scheme based on index

## Type Aliases[§](#types){.anchor} {#types .section-header}

[MultiSeries2D](type.MultiSeries2D.html "type optionstratlib::visualization::MultiSeries2D"){.type}
:   A type alias representing a collection of 2D series.
:::::::::::::
::::::::::::::
