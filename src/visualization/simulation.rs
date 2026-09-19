//! `Graph` implementations for the simulation containers.
//!
//! A [`RandomWalk`] plots as one price series against days to expiry
//! (negated so time runs left to right); a [`Simulator`] plots every walk
//! it holds as one series each. The implementations live here because
//! `Graph` is a visualization trait and simulation must not depend on
//! visualization (ADR-0001 D2, simulation row).

use crate::simulation::randomwalk::RandomWalk;
use crate::simulation::simulator::Simulator;
use crate::visualization::{ColorScheme, Graph, GraphConfig, GraphData, Series2D, TraceMode};
use positive::Positive;
use rust_decimal::Decimal;
use std::fmt::Display;
use std::ops::AddAssign;

/// Chart width in pixels shared by the simulation charts.
const SIMULATION_CHART_WIDTH: u32 = 1600;
/// Chart height in pixels shared by the simulation charts.
const SIMULATION_CHART_HEIGHT: u32 = 900;
/// Line width shared by the simulation charts.
const SIMULATION_LINE_WIDTH: f64 = 2.0;
/// Line colour of a single random walk.
const RANDOM_WALK_LINE_COLOR: &str = "#1f77b4";

impl<X, Y> Graph for RandomWalk<X, Y>
where
    X: Copy + TryInto<Positive> + AddAssign + Display,
    Y: TryInto<Positive> + Display + Clone,
{
    fn graph_data(&self) -> GraphData {
        let steps = self.get_steps();
        let y: Vec<Decimal> = steps
            .iter()
            .map(|step| step.get_graph_y_value().unwrap_or(Positive::ZERO).to_dec())
            .collect();
        let x: Vec<Decimal> = steps
            .iter()
            .map(|step| -step.get_graph_x_in_days_left().to_dec())
            .collect();

        GraphData::Series(Series2D {
            x,
            y,
            name: self.get_title().to_string(),
            mode: TraceMode::Lines,
            line_color: Some(RANDOM_WALK_LINE_COLOR.to_string()),
            line_width: Some(SIMULATION_LINE_WIDTH),
        })
    }

    fn graph_config(&self) -> GraphConfig {
        GraphConfig {
            title: self.get_title().to_string(),
            x_label: Some("Date".to_string()),
            y_label: Some("Price".to_string()),
            z_label: None,
            width: SIMULATION_CHART_WIDTH,
            height: SIMULATION_CHART_HEIGHT,
            show_legend: false,
            color_scheme: ColorScheme::Default,
            line_style: Default::default(),
            legend: None,
        }
    }
}

impl<X, Y> Graph for Simulator<X, Y>
where
    X: Copy + TryInto<Positive> + AddAssign + Display,
    Y: TryInto<Positive> + Display + Clone,
{
    fn graph_data(&self) -> GraphData {
        let random_walks = self.get_steps();
        let mut series: Vec<Series2D> = Vec::with_capacity(random_walks.len());
        for (i, steps) in random_walks.iter().enumerate() {
            let y: Vec<Decimal> = steps
                .iter()
                .map(|step| step.get_graph_y_value().unwrap_or(Positive::ZERO).to_dec())
                .collect();
            let x: Vec<Decimal> = steps
                .iter()
                .map(|step| -step.get_graph_x_in_days_left().to_dec())
                .collect();
            series.push(Series2D {
                x,
                y,
                name: format!("Sim_{i}"),
                mode: TraceMode::Lines,
                line_color: None,
                line_width: Some(SIMULATION_LINE_WIDTH),
            });
        }
        GraphData::MultiSeries(series)
    }

    fn graph_config(&self) -> GraphConfig {
        GraphConfig {
            title: self.get_title().to_string(),
            x_label: Some("Date".to_string()),
            y_label: Some("Price".to_string()),
            z_label: None,
            width: SIMULATION_CHART_WIDTH,
            height: SIMULATION_CHART_HEIGHT,
            show_legend: true,
            color_scheme: ColorScheme::HighContrast,
            line_style: Default::default(),
            legend: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ExpirationDate;
    use crate::simulation::steps::Step;
    use crate::simulation::{WalkParams, WalkType, WalkTypeAble, generator_positive};
    use crate::utils::TimeFrame;
    use positive::pos_or_panic;
    use rust_decimal_macros::dec;

    #[derive(Clone)]
    struct TestWalker;
    impl WalkTypeAble<Positive, Positive> for TestWalker {}

    fn params() -> WalkParams<Positive, Positive> {
        WalkParams {
            size: 3,
            init_step: Step::new(
                Positive::ONE,
                TimeFrame::Day,
                ExpirationDate::Days(pos_or_panic!(30.0)),
                Positive::HUNDRED,
            ),
            walker: Box::new(TestWalker),
            walk_type: WalkType::Historical {
                timeframe: TimeFrame::Day,
                prices: vec![
                    Positive::HUNDRED,
                    pos_or_panic!(101.0),
                    pos_or_panic!(102.0),
                ],
                symbol: None,
            },
        }
    }

    fn simulator(size: usize) -> Simulator<Positive, Positive> {
        Simulator::new(
            "Graph Test".to_string(),
            size,
            &params(),
            generator_positive,
        )
        .unwrap()
    }

    #[test]
    fn test_random_walk_graph_data_is_one_series_of_prices() {
        let sim = simulator(1);
        let walk = sim.first().unwrap();
        match walk.graph_data() {
            GraphData::Series(series) => {
                assert_eq!(series.name, "Graph Test_0");
                assert_eq!(series.y, vec![dec!(100), dec!(101), dec!(102)]);
                assert_eq!(series.x.len(), 3);
                assert!(series.x.windows(2).all(|w| w[0] <= w[1]));
                assert_eq!(series.line_color.as_deref(), Some(RANDOM_WALK_LINE_COLOR));
            }
            other => panic!("expected a single series, got {other:?}"),
        }
        let config = walk.graph_config();
        assert_eq!(config.title, "Graph Test_0");
        assert!(!config.show_legend);
        assert_eq!(config.width, SIMULATION_CHART_WIDTH);
    }

    #[test]
    fn test_simulator_graph_data_has_one_series_per_walk() {
        let sim = simulator(3);
        match sim.graph_data() {
            GraphData::MultiSeries(series) => {
                assert_eq!(series.len(), 3);
                let names: Vec<&str> = series.iter().map(|s| s.name.as_str()).collect();
                assert_eq!(names, vec!["Sim_0", "Sim_1", "Sim_2"]);
                assert!(series.iter().all(|s| s.y.len() == 3));
            }
            other => panic!("expected multiple series, got {other:?}"),
        }
        let config = sim.graph_config();
        assert_eq!(config.title, "Graph Test");
        assert!(config.show_legend);
        assert_eq!(config.height, SIMULATION_CHART_HEIGHT);
    }

    #[cfg(feature = "plotly")]
    #[test]
    fn test_simulator_write_html_round_trip() -> Result<(), Box<dyn std::error::Error>> {
        use std::{fs, path::Path};

        let sim = simulator(2);
        let file_name = "Draws/Simulation/test_simulator.html".as_ref();
        sim.write_html(file_name)?;
        if Path::new(file_name).exists() {
            fs::remove_file(file_name)?;
        }
        Ok(())
    }
}
