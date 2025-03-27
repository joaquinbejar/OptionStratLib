use optionstratlib::chains::chain::OptionChain;
use optionstratlib::simulation::randomwalk::RandomWalk;
use optionstratlib::simulation::steps::{Step, Xstep, Ystep};
use optionstratlib::simulation::{WalkParams, WalkType, WalkTypeAble};
use optionstratlib::utils::setup_logger;
use optionstratlib::utils::time::{TimeFrame, convert_time_frame};
use optionstratlib::visualization::utils::{Graph, GraphBackend};
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

fn create_chain_from_step(
    previous_y_step: &Ystep<OptionChain>,
    new_price: &Positive,
    walk_params: &WalkParams<Positive, OptionChain>,
) -> Result<OptionChain, Box<dyn std::error::Error>> {
    let mut chain = previous_y_step.value();
    let chain_params = chain.to_build_params()?;

    print!("Price: {}", new_price);
    print!("Walk Params: {}", walk_params);
    // TODO: update chain params with new price

    let new_chain = OptionChain::build_chain(&chain_params);

    Ok(new_chain)
}

fn generator(walk_params: &WalkParams<Positive, OptionChain>) -> Vec<Step<Positive, OptionChain>> {
    info!("{}", walk_params);
    let mut y_steps = walk_params.walker.geometric_brownian(&walk_params).unwrap();
    let _ = y_steps.remove(0);
    let mut steps: Vec<Step<Positive, OptionChain>> = vec![walk_params.init_step.clone()];

    let mut previous_x_step = walk_params.init_step.x.clone();
    let mut previous_y_step = walk_params.ystep();

    for y_step in y_steps.iter() {
        previous_x_step = match previous_x_step.next() {
            Ok(x_step) => x_step,
            Err(_) => break,
        };
        // convert y_step to OptionChain
        let y_step_chain: OptionChain =
            create_chain_from_step(&previous_y_step, &y_step, &walk_params).unwrap();
        previous_y_step = previous_y_step.next(y_step_chain).clone();
        let step = Step {
            x: previous_x_step,
            y: previous_y_step.clone(),
        };
        steps.push(step)
    }

    assert!(steps.len() <= walk_params.size);

    steps
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_logger();
    // let n_steps = 43_200; // 30 days in minutes
    let n_steps = 3;
    let initial_chain =
        OptionChain::load_from_json("examples/Chains/SP500-18-oct-2024-5781.88.json")?;
    // info!("Initial Chain: {}", initial_chain);

    let std_dev = pos!(20.0);
    let walker = Box::new(Walker::new());
    let days = pos!(30.0);

    let walk_params = WalkParams {
        size: n_steps,
        init_step: Step {
            x: Xstep::new(Positive::ONE, TimeFrame::Minute, ExpirationDate::Days(days)),
            y: Ystep::new(0, initial_chain),
        },
        walk_type: WalkType::GeometricBrownian {
            dt: convert_time_frame(pos!(1.0) / days, &TimeFrame::Minute, &TimeFrame::Day), // TODO
            drift: dec!(0.0),
            volatility: std_dev,
        },
        walker: walker,
    };

    let random_walk = RandomWalk::new("Random Walk".to_string(), &walk_params, generator);
    debug!("Random Walk: {}", random_walk);
    //
    // random_walk.graph(
    //     GraphBackend::Bitmap {
    //         file_path: "Draws/Simulation/random_walk_chain.png",
    //         size: (1200, 800),
    //     },
    //     20,
    // )?;

    Ok(())
}
