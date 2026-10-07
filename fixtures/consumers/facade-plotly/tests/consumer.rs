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

/// Whether two paths name one type: every path of the facade must be the item
/// `optionstratlib-visualization` defines, never a copy of it.
fn same_type<A: 'static, B: 'static>() -> bool {
    std::any::TypeId::of::<A>() == std::any::TypeId::of::<B>()
}

/// Implemented through the component crate's `Graph` and passed where the
/// facade's `Graph` is required: the reverse of `component_series`.
struct ComponentProbe;

impl optionstratlib_visualization::visualization::Graph for ComponentProbe {
    fn graph_data(&self) -> optionstratlib_visualization::visualization::GraphData {
        Probe.graph_data()
    }
}

fn facade_series<G: optionstratlib::visualization::Graph>(graph: &G) -> usize {
    match graph.graph_data() {
        GraphData::Series(series) => series.x.len(),
        GraphData::MultiSeries(series) => series.len(),
        GraphData::GraphSurface(_) => 0,
    }
}

#[test]
fn test_the_facade_and_the_component_share_one_graph_contract() {
    // One trait: an implementor of either is accepted by a bound on the other.
    assert_eq!(facade_series(&ComponentProbe), 3);
    assert_eq!(component_series(&Probe), 3);
    // One definition of the chart data, the styles and the error.
    use optionstratlib_visualization as direct;
    assert!(same_type::<
        optionstratlib::visualization::GraphData,
        direct::visualization::GraphData,
    >());
    assert!(same_type::<
        optionstratlib::visualization::Series2D,
        direct::visualization::Series2D,
    >());
    assert!(same_type::<
        optionstratlib::visualization::Surface3D,
        direct::visualization::Surface3D,
    >());
    assert!(same_type::<
        optionstratlib::visualization::GraphConfig,
        direct::visualization::GraphConfig,
    >());
    assert!(same_type::<
        optionstratlib::visualization::OutputType<'static>,
        direct::visualization::OutputType<'static>,
    >());
    assert!(same_type::<
        optionstratlib::visualization::TraceMode,
        direct::visualization::TraceMode,
    >());
    assert!(same_type::<
        optionstratlib::visualization::LineStyle,
        direct::visualization::LineStyle,
    >());
    assert!(same_type::<
        optionstratlib::visualization::ColorScheme,
        direct::visualization::ColorScheme,
    >());
    assert!(same_type::<
        optionstratlib::error::GraphError,
        direct::error::GraphError,
    >());
}

#[test]
fn test_the_prelude_names_the_canonical_items() {
    assert!(same_type::<
        GraphData,
        optionstratlib::visualization::GraphData,
    >());
    assert!(same_type::<Series2D, optionstratlib::visualization::Series2D>());
    assert!(same_type::<
        Surface3D,
        optionstratlib::visualization::Surface3D,
    >());
    assert!(same_type::<
        TraceMode,
        optionstratlib::visualization::TraceMode,
    >());
    assert!(same_type::<GraphError, optionstratlib::error::GraphError>());
    // The unified error wraps the one `GraphError`, with no conversion layer.
    let wrapped = Error::from(optionstratlib_visualization::error::GraphError::Render(
        "probe".to_string(),
    ));
    assert!(matches!(
        wrapped,
        Error::Graph(optionstratlib::error::GraphError::Render(_))
    ));
}

#[test]
fn test_the_plotly_helpers_are_the_defining_crates_items() {
    same_item(
        optionstratlib::visualization::make_scatter,
        optionstratlib_visualization::visualization::make_scatter,
    );
    // The prelude's convenience name is the canonical function.
    same_item(make_surface, optionstratlib::visualization::make_surface);
}

/// Compiles only when both arguments have the same type. Every function item
/// has its own type, so this proves a path re-exports the function.
fn same_item<T>(_: T, _: T) {}
