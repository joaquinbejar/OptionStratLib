use optionstratlib::simulation::WalkTypeAble;
use optionstratlib::simulation::randomwalk::RandomWalk;
use optionstratlib::simulation::step::{Step, Xstep, Ystep};
use optionstratlib::simulation::walk::{WalkParams, WalkType};
use optionstratlib::utils::setup_logger;
use optionstratlib::utils::time::TimeFrame;
use optionstratlib::{ExpirationDate, Positive, pos, spos};
use rust_decimal_macros::dec;
use tracing::info;

struct Walker {
    x: Positive,
    y: Positive,
}

impl Walker {
    fn new(x: Positive, y: Positive) -> Self {
        Walker { x, y }
    }
}

impl WalkTypeAble<Positive, Positive> for Walker {}

fn generator(walk_params: WalkParams<Positive, Positive>) -> Vec<Step<Positive, Positive>> {
    info!("{}", walk_params);
    let mut y_steps = walk_params.walker.geometric_brownian(&walk_params).unwrap();
    let _ = y_steps.remove(0);
    let mut steps: Vec<Step<Positive, Positive>> = vec![walk_params.init_step];

    let mut previous_x_step = walk_params.init_step.x.clone();
    let mut previous_y_step = walk_params.init_step.y.clone();

    for y_step in y_steps.iter() {
        previous_x_step = match previous_x_step.next() {
            Ok(x_step) => x_step,
            Err(e) => break,
        };
        previous_y_step = previous_y_step.next(y_step.clone());
        let step = Step {
            x: previous_x_step,
            y: previous_y_step,
        };
        steps.push(step)
    }

    assert!(steps.len() <= walk_params.size);

    steps
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_logger();
    let years = 3.0;
    // let n_steps = 252 * years as usize;
    let n_steps = 35;
    let initial_price = pos!(7000.0);
    let mean = 0.0;
    let std_dev = pos!(0.2);
    let std_dev_change = pos!(0.001);
    let risk_free_rate = Some(dec!(0.0));
    let dividend_yield = spos!(0.02);
    let volatility_window = 20;
    let initial_volatility = Some(std_dev);

    let walker = Box::new(Walker::new(pos!(0.0), initial_price));

    let walk_params = WalkParams {
        size: n_steps,
        init_step: Step {
            x: Xstep::new(pos!(1.0), TimeFrame::Day, ExpirationDate::Days(pos!(30.0))),
            y: Ystep::new(0, pos!(100.0)),
        },
        walk_type: WalkType::GeometricBrownian {
            dt: pos!(1.0 / 252.0),
            drift: dec!(0.0),
            volatility: initial_volatility.unwrap(),
        },
        walker: walker,
    };

    let mut random_walk = RandomWalk::new("Random Walk".to_string(), walk_params, generator);
    info!("Random Walk: {}", random_walk);

    Ok(())
}
