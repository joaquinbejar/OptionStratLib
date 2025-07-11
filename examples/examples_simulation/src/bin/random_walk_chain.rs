use optionstratlib::chains::chain::OptionChain;
use optionstratlib::chains::generator_optionchain;
use optionstratlib::simulation::randomwalk::RandomWalk;
use optionstratlib::simulation::steps::{Step, Xstep, Ystep};
use optionstratlib::simulation::{WalkParams, WalkType, WalkTypeAble};
use optionstratlib::utils::setup_logger;
use optionstratlib::utils::time::{TimeFrame, convert_time_frame, get_x_days_formatted};
use optionstratlib::visualization::Graph;
use optionstratlib::{ExpirationDate, Positive, pos};
use rust_decimal_macros::dec;
use tracing::{debug, info};

#[warn(dead_code)]
struct Walker {}

impl Walker {
    fn new() -> Self {
        Walker {}
    }
}

impl WalkTypeAble<Positive, OptionChain> for Walker {}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_logger();
    let n_steps = 43_200; // 30 days in minutes
    let mut initial_chain = OptionChain::load_from_json(
        "examples/Chains/Germany-40-2025-05-27-15:29:00-UTC-24209.json",
    )?;
    initial_chain.update_expiration_date(get_x_days_formatted(2));
    let iv = pos!(0.20);
    let walker = Box::new(Walker::new());
    let days = pos!(2.0);

    let walk_params = WalkParams {
        size: n_steps,
        init_step: Step {
            x: Xstep::new(Positive::ONE, TimeFrame::Minute, ExpirationDate::Days(days)),
            y: Ystep::new(0, initial_chain),
        },
        walk_type: WalkType::GeometricBrownian {
            dt: convert_time_frame(pos!(1.0) / days, &TimeFrame::Minute, &TimeFrame::Day),
            drift: dec!(0.0),
            volatility: iv,
        },
        walker: walker,
    };

    let random_walk = RandomWalk::new(
        "Random Walk".to_string(),
        &walk_params,
        generator_optionchain,
    );

    debug!("Random Walk: {}", random_walk);
    let path: &std::path::Path = "Draws/Simulation/random_walk_chain.png".as_ref();
    random_walk.write_png(path)?;
    info!("Last Chain: {}", random_walk.last().unwrap().y.value());

    Ok(())
}
