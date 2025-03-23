
use optionstratlib::simulation::randomwalk::RandomWalk;
use optionstratlib::simulation::step::{Step, Xstep, Ystep};
use optionstratlib::simulation::walk::{WalkParams, WalkType};
use optionstratlib::utils::setup_logger;
use optionstratlib::utils::time::TimeFrame;
use optionstratlib::{ExpirationDate, pos, spos, Positive};
use rust_decimal_macros::dec;
use tracing::info;

fn generator(walk_params: WalkParams<Positive, Positive>) -> Vec<Step<Positive, Positive>> {
    info!("{}", walk_params);
    vec![Step {
        x: Xstep::new(pos!(0.0), TimeFrame::Day, ExpirationDate::Days(pos!(30.0))),
        y: Ystep::new(pos!(100.0)),
    }]
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_logger();
    let years = 3.0;
    let n_steps = 252 * years as usize;
    let initial_price = pos!(7000.0);
    let mean = 0.0;
    let std_dev = pos!(0.2);
    let std_dev_change = pos!(0.001);
    let risk_free_rate = Some(dec!(0.0));
    let dividend_yield = spos!(0.02);
    let volatility_window = 20;
    let initial_volatility = Some(std_dev);

    let walk_params = WalkParams {
        size: n_steps,
        init_step: Step {
            x: Xstep::new(pos!(0.0), TimeFrame::Day, ExpirationDate::Days(pos!(30.0))),
            y: Ystep::new(pos!(100.0)),
        },
        walk_type: WalkType::GeometricBrownian {
            dt: pos!(1.0 / 252.0),
            drift: dec!(0.0),
            volatility: initial_volatility.unwrap(),
        },
    };

    let mut random_walk = RandomWalk::new("Random Walk".to_string(), walk_params, generator);

    Ok(())
}
