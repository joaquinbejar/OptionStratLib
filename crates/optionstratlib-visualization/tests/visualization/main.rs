//! Integration tests of the visualization layer: the `Graph` contract and its
//! implementations, the chart models, the Plotly backend (behind `plotly` and
//! `static_export`) and `GraphError`. Moved from the facade's
//! `tests/unit/visualization` and `tests/unit/error/graph_test.rs` (#542).

mod default_test;
mod file_test;
mod graph_default_test;
mod graph_error_test;
mod model_test;
mod plotly_render_test;
mod plotly_test;
mod simulation_graph_test;
mod strategy_graph_test;
