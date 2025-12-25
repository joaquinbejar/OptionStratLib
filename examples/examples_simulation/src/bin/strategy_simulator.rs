use positive::pos_or_panic;
use optionstratlib::prelude::*;

struct Walker {}

impl Walker {
    fn new() -> Self {
        Walker {}
    }
}

impl WalkTypeAble<Positive, Positive> for Walker {}

fn main() -> Result<(), Error> {
    setup_logger();
    let symbol = "GOLD".to_string();
    let strike_price = pos_or_panic!(3930.0);
    let simulator_size: usize = 35;
    let n_steps = 10080;
    let initial_price = pos_or_panic!(4011.95);
    let iv = pos_or_panic!(0.27);
    let open_premium = pos_or_panic!(27.05);
    let walker = Box::new(Walker::new());
    let days = pos_or_panic!(7.0);
    let dt = convert_time_frame(
        Positive::ONE / days,
        &TimeFrame::Minute,
        &TimeFrame::Day,
    );
    let volatility_dt = volatility_for_dt(iv, dt, TimeFrame::Minute, TimeFrame::Day)?;

    let short_put_strategy = ShortPut::new(
        symbol,
        strike_price,
        ExpirationDate::Days(days),
        iv,
        Positive::ONE,
        initial_price,
        Decimal::ZERO,
        Positive::ZERO,
        open_premium,
        Positive::ZERO,
        Positive::ZERO,
    );

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

    info!("Open Premium: ${:.2}", open_premium);

    for simulation in simulator.into_iter() {
        for step in simulation.get_steps() {
            let days_left = step.x.days_left()?;
            let market_price = step.get_value();
            info!(
                "Simulation days left: {} value: ${}",
                days_left,
                step.get_value()
            );
            let pnl = short_put_strategy.calculate_pnl(
                market_price,
                ExpirationDate::Days(days_left),
                &iv,
            )?;
            info!("Simulation PNL: ${:.2}", pnl);
        }
    }

    let last_values: Vec<&Positive> = simulator
        .into_iter()
        .map(|step| step.last().unwrap().get_value())
        .collect();
    info!("Last Values: {:?}", last_values);
    let path: &std::path::Path = "Draws/Simulation/position_simulator.png".as_ref();
    simulator.write_png(path)?;
    Ok(())
}
