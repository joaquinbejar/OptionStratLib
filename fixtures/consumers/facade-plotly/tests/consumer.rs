//! A downstream chart consumer on the `optionstratlib` facade with
//! default features plus `plotly` (#544): the interactive
//! methods of `Graph` work, HTML is written, and a PNG or SVG request is
//! reported as an error instead of resolving the export stack.

use optionstratlib::error::GraphError;
use optionstratlib::prelude::*;
use optionstratlib::visualization::{GraphConfig, OutputType};
use std::path::PathBuf;

/// A chart a downstream crate defines: it supplies `graph_data` and keeps the
/// default `graph_config`, which is all the contract asks on every surface.
struct Probe;

impl Graph for Probe {
    fn graph_data(&self) -> GraphData {
        GraphData::Series(Series2D {
            x: vec![dec!(1), dec!(2), dec!(3)],
            y: vec![dec!(2), dec!(4), dec!(8)],
            name: "probe".to_string(),
            mode: TraceMode::Lines,
            line_color: None,
            line_width: None,
        })
    }
}

/// Compiles only for a type that implements the component crate's `Graph`.
/// Passing a facade-path implementor proves the facade re-exports the trait
/// rather than defining a second one.
fn component_series<G: optionstratlib_visualization::visualization::Graph>(graph: &G) -> usize {
    match graph.graph_data() {
        optionstratlib_visualization::visualization::GraphData::Series(series) => series.x.len(),
        optionstratlib_visualization::visualization::GraphData::MultiSeries(series) => series.len(),
        optionstratlib_visualization::visualization::GraphData::GraphSurface(_) => 0,
    }
}

/// Long 100 call, 30 days, premium 5.
fn long_call() -> LongCall {
    match LongCall::new(
        "TEST".to_string(),
        Positive::HUNDRED,
        ExpirationDate::Days(pos_or_panic!(30.0)),
        pos_or_panic!(0.20),
        Positive::ONE,
        Positive::HUNDRED,
        dec!(0.05),
        Positive::ZERO,
        pos_or_panic!(5.0),
        pos_or_panic!(0.5),
        pos_or_panic!(0.5),
    ) {
        Ok(strategy) => strategy,
        Err(error) => panic!("LongCall::new: {error}"),
    }
}

#[test]
fn test_a_downstream_type_implements_the_one_graph_contract() {
    assert_eq!(component_series(&Probe), 3);
    assert_eq!(Probe.graph_config(), GraphConfig::default());
}

#[test]
fn test_a_strategy_is_charted_through_the_prelude() {
    assert!(component_series(&long_call()) > 0);
    match long_call().graph_data() {
        GraphData::MultiSeries(series) => assert!(!series.is_empty()),
        GraphData::Series(series) => assert!(!series.x.is_empty()),
        GraphData::GraphSurface(_) => panic!("a payoff chart is not a surface"),
    }
}

/// A path under cargo's per-target scratch directory for this test binary.
fn scratch(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(name)
}

#[test]
fn test_to_plot_builds_the_interactive_figure() {
    let json = Probe.to_plot().to_json();
    assert!(
        json.contains("probe"),
        "the series name is in the figure: {json}"
    );
}

#[test]
fn test_the_trace_builders_come_with_plotly() {
    // Absent without `plotly` (see the `compile_fail` doctests of the
    // `facade-visualization` and `headless-full` fixtures).
    let GraphData::Series(series) = Probe.graph_data() else {
        panic!("the probe is a single series");
    };
    let _trace = optionstratlib::visualization::make_scatter(&series);
}

#[test]
fn test_html_is_written_without_a_browser() {
    let path = scratch("facade-plotly.html");
    assert!(Probe.write_html(&path).is_ok());
    assert!(path.is_file());
    assert!(Probe.render(OutputType::Html(&path)).is_ok());
}

#[test]
fn test_png_and_svg_are_reported_not_resolved() {
    let path = scratch("facade-plotly.png");
    for output in [OutputType::Png(&path), OutputType::Svg(&path)] {
        match Probe.render(output) {
            Err(GraphError::Render(message)) => assert!(message.contains("static_export")),
            other => panic!("expected GraphError::Render, got {other:?}"),
        }
    }
    assert!(!path.exists());
}
