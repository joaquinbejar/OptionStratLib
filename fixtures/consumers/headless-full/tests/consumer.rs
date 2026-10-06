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
