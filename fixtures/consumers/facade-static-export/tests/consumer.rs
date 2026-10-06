//! A downstream chart consumer on the `optionstratlib` facade with
//! `default-features = false, features = ["static_export"]` (#544): the PNG
//! and SVG writers are on `Graph`, and `static_export` implies `plotly`, so
//! the interactive methods are too. Rendering an image needs a WebDriver that
//! matches the installed browser, which a dependency-graph fixture cannot
//! assume; `make test-visual` runs the real export, and this fixture proves
//! the surface compiles and routes to it.

use optionstratlib::error::GraphError;
use optionstratlib::prelude::*;
use optionstratlib::visualization::{GraphConfig, OutputType};
use std::path::{Path, PathBuf};

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
fn test_the_image_writers_are_on_the_contract() {
    let png: fn(&Probe, &Path) -> Result<(), GraphError> = <Probe as Graph>::write_png;
    let svg: fn(&Probe, &Path) -> Result<(), GraphError> = <Probe as Graph>::write_svg;
    // Never called: the exporter drives a browser.
    let _ = (png, svg);
}

#[test]
fn test_static_export_implies_the_interactive_surface() {
    let json = Probe.to_plot().to_json();
    assert!(
        json.contains("probe"),
        "the series name is in the figure: {json}"
    );
    let path = scratch("facade-static-export.html");
    assert!(Probe.write_html(&path).is_ok());
    assert!(path.is_file());
    assert!(Probe.render(OutputType::Html(&path)).is_ok());
}
