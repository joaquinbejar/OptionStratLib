//! The PNG/SVG export through the public `Graph` API (#927): drawn natively
//! from `graph_data()`/`graph_config()`, with typed errors that reach every
//! entry point unchanged, no browser and no process-wide lock.
#![cfg(feature = "static_export")]

use super::artifacts::{assert_png_artifact, assert_svg_artifact};
use optionstratlib_visualization::error::GraphError;
use optionstratlib_visualization::visualization::{
    Graph, GraphConfig, GraphData, OutputType, Series2D, TraceMode,
};
use rust_decimal_macros::dec;
use std::path::PathBuf;

struct Chart {
    data: GraphData,
    title: &'static str,
}

impl Graph for Chart {
    fn graph_data(&self) -> GraphData {
        self.data.clone()
    }

    fn graph_config(&self) -> GraphConfig {
        GraphConfig {
            title: self.title.to_string(),
            width: 640,
            height: 400,
            ..GraphConfig::default()
        }
    }
}

fn line(y: Vec<rust_decimal::Decimal>) -> GraphData {
    GraphData::Series(Series2D {
        x: vec![dec!(1), dec!(2), dec!(3)],
        y,
        name: "line".to_string(),
        mode: TraceMode::Lines,
        line_color: None,
        line_width: None,
    })
}

#[test]
fn test_typed_errors_reach_every_entry_point_unchanged() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempfile::tempdir()?;
    // Two y values for three x values: Plotly drew two points, the static
    // export refuses rather than drop one.
    let chart = Chart {
        data: line(vec![dec!(1), dec!(2)]),
        title: "short",
    };
    let png = dir.path().join("short.png");
    let svg = dir.path().join("short.svg");
    let mismatch = |result: Result<(), GraphError>| {
        matches!(result, Err(GraphError::LengthMismatch { x: 3, y: 2, .. }))
    };
    assert!(mismatch(chart.write_png(&png)));
    assert!(mismatch(chart.write_svg(&svg)));
    assert!(mismatch(chart.render(OutputType::Png(&png))));
    assert!(mismatch(chart.render(OutputType::Svg(&svg))));
    assert!(!png.exists() && !svg.exists());
    Ok(())
}

/// A chart whose `to_plot` is overridden: since 0.23 the images come from
/// `graph_data()`/`graph_config()`, and the override reaches HTML only.
struct Customised;

impl Graph for Customised {
    fn graph_data(&self) -> GraphData {
        line(vec![dec!(1), dec!(3), dec!(2)])
    }

    fn graph_config(&self) -> GraphConfig {
        GraphConfig {
            title: "From graph_config".to_string(),
            width: 640,
            height: 400,
            ..GraphConfig::default()
        }
    }

    fn to_plot(&self) -> plotly::Plot {
        let mut plot = plotly::Plot::new();
        plot.set_layout(plotly::Layout::new().title("From to_plot"));
        plot
    }
}

#[test]
fn test_images_come_from_graph_data_and_html_from_to_plot() -> Result<(), Box<dyn std::error::Error>>
{
    let dir = tempfile::tempdir()?;
    let (svg, html, png) = (
        dir.path().join("chart.svg"),
        dir.path().join("chart.html"),
        dir.path().join("chart.png"),
    );
    Customised.write_svg(&svg)?;
    Customised.write_html(&html)?;
    Customised.write_png(&png)?;
    let svg = std::fs::read_to_string(&svg)?;
    assert!(svg.contains("<title>From graph_config</title>"));
    assert!(!svg.contains("From to_plot"));
    assert!(std::fs::read_to_string(&html)?.contains("From to_plot"));
    assert_png_artifact(&png);
    Ok(())
}

#[test]
fn test_concurrent_exports_are_independent() -> Result<(), Box<dyn std::error::Error>> {
    // No browser, driver or process-wide lock: exports in parallel threads
    // each write their own complete file.
    let dir = tempfile::tempdir()?;
    let handles: Vec<_> = (0..8)
        .map(|index| {
            let base = dir.path().to_path_buf();
            std::thread::spawn(move || -> Result<(PathBuf, PathBuf), GraphError> {
                let chart = Chart {
                    data: line(vec![dec!(1), rust_decimal::Decimal::from(index), dec!(2)]),
                    title: "parallel",
                };
                let (png, svg) = (
                    base.join(format!("{index}.png")),
                    base.join(format!("{index}.svg")),
                );
                chart.write_png(&png)?;
                chart.write_svg(&svg)?;
                Ok((png, svg))
            })
        })
        .collect();
    for handle in handles {
        let (png, svg) = handle.join().map_err(|_| "export thread panicked")??;
        assert_png_artifact(&png);
        assert_svg_artifact(&svg);
    }
    Ok(())
}
