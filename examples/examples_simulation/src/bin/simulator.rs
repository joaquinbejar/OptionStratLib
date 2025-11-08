use optionstratlib::chains::generator_positive;
use optionstratlib::prelude::volatility_for_dt;
use optionstratlib::simulation::simulator::Simulator;
use optionstratlib::simulation::steps::{Step, Xstep, Ystep};
use optionstratlib::simulation::{WalkParams, WalkType, WalkTypeAble};
use optionstratlib::utils::setup_logger;
use optionstratlib::utils::time::{TimeFrame, convert_time_frame};
use optionstratlib::visualization::Graph;
use optionstratlib::{ExpirationDate, Positive, pos};
use rust_decimal_macros::dec;
use tracing::{debug, info};

struct Walker {}

impl Walker {
    fn new() -> Self {
        Walker {}
    }
}

impl WalkTypeAble<Positive, Positive> for Walker {}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_logger();
    let simulator_size: usize = 35;
    // let n_steps = 43_200; // 30 days in minutes
    let n_steps = 10080;
    let initial_price = pos!(4011.0);
    let iv = pos!(0.27);
    let walker = Box::new(Walker::new());
    let days = pos!(7.0);
    let dt = convert_time_frame(pos!(1.0) / days, &TimeFrame::Minute, &TimeFrame::Day);
    let volatility_dt = volatility_for_dt(iv, dt, TimeFrame::Minute, TimeFrame::Day)?;

    let walk_params = WalkParams {
        size: n_steps,
        init_step: Step {
            x: Xstep::new(Positive::ONE, TimeFrame::Minute, ExpirationDate::Days(days)),
            y: Ystep::new(0, initial_price),
        },
        walk_type: WalkType::Brownian {
            dt,
            drift: dec!(0.0),
            volatility: volatility_dt,
        },
        walker,
    };

    let simulator = Simulator::new(
        "Simulator".to_string(),
        simulator_size,
        &walk_params,
        generator_positive,
    );
    debug!("Simulator: {}", simulator);

    // let last_steps: Vec<&Step<Positive, Positive>> = simulator
    //     .into_iter()
    //     .map(|step| step.last().unwrap())
    //     .collect();
    // info!("Last Steps: {:?}", last_steps);

    let last_values: Vec<&Positive> = simulator
        .into_iter()
        .map(|step| step.last().unwrap().get_value())
        .collect();
    info!("Last Values: {:?}", last_values);
    let path: &std::path::Path = "Draws/Simulation/simulator.png".as_ref();
    simulator.write_png(path)?;
    Ok(())
}
