//! A downstream chart consumer on the `optionstratlib` facade with
//! `default-features = false, features = ["static_export"]` (#544): the PNG
//! and SVG writers are on `Graph`, and `static_export` implies `plotly`, so
//! the interactive methods are too. Since #927 the images are drawn natively,
//! with no browser or WebDriver, so this fixture runs the real export and
//! checks the typed error a downstream crate can match on.

use optionstratlib::error::Error;
use optionstratlib::error::GraphError;
use optionstratlib::prelude::*;
use optionstratlib::visualization::make_surface;
use optionstratlib::visualization::{GraphConfig, OutputType};
use optionstratlib::visualization::{GraphData, Series2D, Surface3D, TraceMode};
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
    let (png_path, svg_path) = (
        scratch("facade-static-export.png"),
        scratch("facade-static-export.svg"),
    );
    assert!(png(&Probe, &png_path).is_ok());
    assert!(svg(&Probe, &svg_path).is_ok());
    let bytes = std::fs::read(&png_path).unwrap_or_default();
    assert!(bytes.starts_with(b"\x89PNG\r\n\x1a\n"));
    let document = std::fs::read_to_string(&svg_path).unwrap_or_default();
    assert!(document.contains("<svg") && !document.contains("<text"));
    assert!(Probe.render(OutputType::Png(&png_path)).is_ok());
    assert!(long_call().write_svg(&svg_path).is_ok());
}

/// A chart whose series has more x than y values.
struct Lopsided;

impl Graph for Lopsided {
    fn graph_data(&self) -> GraphData {
        GraphData::Series(Series2D {
            x: vec![dec!(1), dec!(2)],
            y: vec![dec!(1)],
            name: "lopsided".to_string(),
            mode: TraceMode::Lines,
            line_color: None,
            line_width: None,
        })
    }
}

#[test]
fn test_bad_data_is_a_typed_error_a_consumer_can_match() {
    let result = Lopsided.write_png(&scratch("lopsided.png"));
    // `GraphError` is `#[non_exhaustive]` since 0.23: a consumer matches the
    // variants it handles and keeps a wildcard arm.
    let mismatch = match result {
        Err(GraphError::LengthMismatch { x, y, .. }) => Some((x, y)),
        Err(_) | Ok(()) => None,
    };
    assert_eq!(mismatch, Some((2, 1)));
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
fn test_the_chart_models_are_the_canonical_items() {
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
