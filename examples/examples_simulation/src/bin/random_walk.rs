use optionstratlib::chains::generator_positive;
use optionstratlib::simulation::randomwalk::RandomWalk;
use optionstratlib::simulation::steps::{Step, Xstep, Ystep};
use optionstratlib::simulation::{WalkParams, WalkType, WalkTypeAble};
use optionstratlib::utils::setup_logger;
use optionstratlib::utils::time::{TimeFrame, convert_time_frame};
use optionstratlib::visualization::utils::{Graph, GraphBackend};
use optionstratlib::{ExpirationDate, Positive, pos};
use rust_decimal_macros::dec;
use tracing::debug;

#[warn(dead_code)]
struct Walker {}

impl Walker {
    fn new() -> Self {
        Walker {}
    }
}

impl WalkTypeAble<Positive, Positive> for Walker {}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_logger();
    let n_steps = 43_200; // 30 days in minutes
    let initial_price = pos!(100.0);
    let std_dev = pos!(20.0);
    let walker = Box::new(Walker::new());
    let days = pos!(30.0);

    let walk_params = WalkParams {
        size: n_steps,
        init_step: Step {
            x: Xstep::new(Positive::ONE, TimeFrame::Minute, ExpirationDate::Days(days)),
            y: Ystep::new(0, initial_price),
        },
        walk_type: WalkType::GeometricBrownian {
            dt: convert_time_frame(pos!(1.0) / days, &TimeFrame::Minute, &TimeFrame::Day), // TODO
            drift: dec!(0.0),
            volatility: std_dev,
        },
        walker: walker,
    };

    let random_walk = RandomWalk::new("Random Walk".to_string(), &walk_params, generator_positive);
    debug!("Random Walk: {}", random_walk);

    random_walk.graph(
        GraphBackend::Bitmap {
            file_path: "Draws/Simulation/random_walk.png",
            size: (1200, 800),
        },
        20,
    )?;

    Ok(())
}
