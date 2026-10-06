//! The chart of a random walk (#633): revived from the never-compiled
//! `tests/unit/simulation/model_and_randomwalk_tests.rs`, whose `Graph` part
//! belongs with the visualization layer.

use optionstratlib_core::model::{ExpirationDate, Positive};
use optionstratlib_core::pos_or_panic;
use optionstratlib_core::utils::TimeFrame;
use optionstratlib_simulation::simulation::randomwalk::RandomWalk;
use optionstratlib_simulation::simulation::steps::Step;
use optionstratlib_simulation::simulation::{
    WalkParams, WalkType, WalkTypeAble, generator_positive,
};
use optionstratlib_visualization::visualization::{Graph, GraphData};

/// A walker that keeps every default: a `Historical` walk replays its prices.
#[derive(Clone)]
struct Replay;

impl WalkTypeAble<Positive, Positive> for Replay {}

#[test]
fn test_random_walk_graph_data_and_config() {
    let params = WalkParams {
        size: 3,
        init_step: Step::new(
            Positive::ONE,
            TimeFrame::Day,
            ExpirationDate::Days(pos_or_panic!(30.0)),
            Positive::HUNDRED,
        ),
        walker: Box::new(Replay),
        walk_type: WalkType::Historical {
            timeframe: TimeFrame::Day,
            prices: vec![
                Positive::HUNDRED,
                pos_or_panic!(101.0),
                pos_or_panic!(103.0),
            ],
            symbol: None,
        },
        seed: None,
    };
    let walk = match RandomWalk::new("RW_Title".to_string(), &params, generator_positive) {
        Ok(walk) => walk,
        Err(error) => panic!("RandomWalk::new: {error}"),
    };
    match walk.graph_data() {
        GraphData::Series(series) => {
            assert_eq!(series.name, "RW_Title");
            assert_eq!(series.x.len(), 3);
            assert_eq!(series.y.len(), 3);
        }
        _ => panic!("expected a 2D series"),
    }
    let config = walk.graph_config();
    assert_eq!(config.title, "RW_Title");
    assert_eq!(config.width, 1600);
    assert_eq!(config.height, 900);
}
