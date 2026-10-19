//! A downstream chart consumer of `optionstratlib-visualization` alone, with
//! `default-features = false, features = ["static_export"]` (#928): the PNG
//! and SVG writers are drawn natively (#927), with no browser or WebDriver,
//! and `static_export` implies `plotly`, so the interactive API is there
//! without naming `plotly`.

use optionstratlib_visualization::error::GraphError;
use optionstratlib_visualization::visualization::{
    Graph, GraphConfig, GraphData, OutputType, Series2D, TraceMode, make_scatter,
};
use rust_decimal_macros::dec;
use std::path::PathBuf;

/// A chart a downstream crate defines: it supplies `graph_data` and a title.
struct Probe;

fn probe_series() -> Series2D {
    Series2D {
        x: vec![dec!(1), dec!(2), dec!(3)],
        y: vec![dec!(2), dec!(4), dec!(8)],
        name: "probe".to_string(),
        mode: TraceMode::Lines,
        line_color: None,
        line_width: None,
    }
}

impl Graph for Probe {
    fn graph_data(&self) -> GraphData {
        GraphData::Series(probe_series())
    }

    fn graph_config(&self) -> GraphConfig {
        GraphConfig {
            title: "Direct probe".to_string(),
            ..GraphConfig::default()
        }
    }
}

/// A path under cargo's per-target scratch directory for this test binary.
fn scratch(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(name)
}

#[test]
fn test_png_and_svg_are_drawn_natively() -> Result<(), Box<dyn std::error::Error>> {
    let (png, svg) = (scratch("direct.png"), scratch("direct.svg"));
    Probe.write_png(&png)?;
    Probe.write_svg(&svg)?;
    assert!(std::fs::read(&png)?.starts_with(b"\x89PNG\r\n\x1a\n"));
    let document = std::fs::read_to_string(&svg)?;
    // Text is drawn as outlines: no `<text>`, no font reference, and the
    // title kept as the document's accessible name.
    assert!(document.starts_with("<svg"));
    assert!(!document.contains("<text") && !document.contains("font-family"));
    assert!(document.contains("<title>Direct probe</title>"));
    Probe.render(OutputType::Png(&png))?;
    Probe.render(OutputType::Svg(&svg))?;
    Ok(())
}

#[test]
fn test_static_export_brings_the_interactive_api() -> Result<(), Box<dyn std::error::Error>> {
    // `to_plot`, HTML and the public Plotly helpers without naming `plotly`.
    let json = Probe.to_plot().to_json();
    assert!(
        json.contains("probe"),
        "the series is in the figure: {json}"
    );
    let html = scratch("direct.html");
    Probe.write_html(&html)?;
    Probe.render(OutputType::Html(&html))?;
    assert!(std::fs::read_to_string(&html)?.contains("probe"));
    // The figure type comes from the component; the consumer never names it.
    let mut plot = Probe.to_plot();
    plot.add_trace(make_scatter(&probe_series()));
    assert!(plot.to_json().contains("probe"));
    Ok(())
}

/// A chart whose series has more x than y values.
struct Lopsided;

impl Graph for Lopsided {
    fn graph_data(&self) -> GraphData {
        GraphData::Series(Series2D {
            x: vec![dec!(1), dec!(2)],
            y: vec![dec!(1)],
            ..probe_series()
        })
    }
}

#[test]
fn test_bad_data_is_a_typed_error_a_consumer_can_match() {
    // `GraphError` is `#[non_exhaustive]`: a consumer matches the variants it
    // handles and keeps a wildcard arm.
    let mismatch = |result: Result<(), GraphError>| match result {
        Err(GraphError::LengthMismatch { x, y, .. }) => Some((x, y)),
        Err(_) | Ok(()) => None,
    };
    let path = scratch("lopsided.png");
    assert_eq!(mismatch(Lopsided.write_png(&path)), Some((2, 1)));
    assert_eq!(
        mismatch(Lopsided.render(OutputType::Png(&path))),
        Some((2, 1))
    );
    assert!(!path.exists());
}
