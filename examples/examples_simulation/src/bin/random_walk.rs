use optionstratlib::prelude::*;

#[warn(dead_code)]
struct Walker {}

impl Walker {
    fn new() -> Self {
        Walker {}
    }
}

impl WalkTypeAble<Positive, Positive> for Walker {}

fn main() -> Result<(), Error> {
    setup_logger();
    let n_steps = 43_200; // 30 days in minutes
    let initial_price = Positive::HUNDRED;
    let std_dev = pos_or_panic!(20.0);
    let walker = Box::new(Walker::new());
    let days = pos_or_panic!(30.0);

    let walk_params = WalkParams {
        size: n_steps,
        init_step: Step {
            x: Xstep::new(Positive::ONE, TimeFrame::Minute, ExpirationDate::Days(days)),
            y: Ystep::new(0, initial_price),
        },
        walk_type: WalkType::GeometricBrownian {
            dt: convert_time_frame(
                Positive::ONE / days,
                &TimeFrame::Minute,
                &TimeFrame::Day,
            ),
            drift: dec!(0.0),
            volatility: std_dev,
        },
        walker,
    };

    let random_walk = RandomWalk::new("Random Walk".to_string(), &walk_params, generator_positive);
    debug!("Random Walk: {}", random_walk);
    let path: &std::path::Path = "Draws/Simulation/random_walk.png".as_ref();
    random_walk.write_png(path)?;

    Ok(())
}
