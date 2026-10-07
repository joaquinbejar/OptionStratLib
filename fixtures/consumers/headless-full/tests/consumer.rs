//! A downstream consumer on the `optionstratlib` facade with its default
//! features (#544): the whole library, charts included, without a rendering
//! backend. The chart data of a strategy comes through the `prelude`.

use optionstratlib::prelude::*;
use optionstratlib::visualization::GraphConfig;

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

/// Compiles only for a type that derives `utoipa::ToSchema`.
fn derives_schema<T: utoipa::ToSchema>() {}

#[test]
fn test_the_default_derives_a_schema_for_every_component() {
    // `schema` is a default feature, so each component that has it is on:
    // one type from each (`facade-schema-off` shows the same types without).
    derives_schema::<optionstratlib::model::option::ExoticParams>();
    derives_schema::<optionstratlib::geometrics::MergeOperation>();
    derives_schema::<optionstratlib::greeks::Greek>();
    derives_schema::<optionstratlib::simulation::ExitPolicy>();
    derives_schema::<optionstratlib::chains::OptionsInStrike>();
    derives_schema::<optionstratlib::pnl::DeltaAdjustment>();
    derives_schema::<optionstratlib::strategies::LongCall>();
    derives_schema::<optionstratlib::backtesting::results::SimulationResult>();
}
