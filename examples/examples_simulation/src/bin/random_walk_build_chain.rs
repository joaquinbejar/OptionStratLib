use optionstratlib::prelude::*;
use positive::pos_or_panic;

#[warn(dead_code)]
struct Walker {}

impl Walker {
    fn new() -> Self {
        Walker {}
    }
}

impl WalkTypeAble<Positive, OptionChain> for Walker {}

fn main() -> Result<(), Error> {
    // let n_steps = 43_200; // 30 days in minutes
    let n_steps = 10080; // 30 days in minutes
    let underlying_price = pos_or_panic!(4011.95);
    let days = pos_or_panic!(7.0);
    let std_dev = Positive::TWO;
    let implied_volatility = std_dev / 100.0;
    let risk_free_rate = dec!(0.0);
    let dividend_yield = Positive::ZERO;
    let symbol = "GOLF".to_string();
    let volume = Some(Positive::ONE);
    let chain_size = 30;
    let strike_interval = spos!(10.0);
    let skew_slope = dec!(-0.3);
    let smile_curve = dec!(0.1);
    let spread = Positive::TWO;
    let decimal_places = 2;

    let price_params = OptionDataPriceParams::new(
        Some(Box::new(underlying_price)),
        Some(ExpirationDate::Days(days)),
        Some(risk_free_rate),
        Some(dividend_yield),
        Some(symbol.clone()),
    );
    let build_params = OptionChainBuildParams::new(
        symbol,
        volume,
        chain_size,
        strike_interval,
        skew_slope,
        smile_curve,
        spread,
        decimal_places,
        price_params,
        implied_volatility,
    );
    let mut initial_chain = OptionChain::build_chain(&build_params);
    initial_chain.update_expiration_date(get_x_days_formatted(2));
    let walker = Box::new(Walker::new());

    let walk_params = WalkParams {
        size: n_steps,
        init_step: Step {
            x: Xstep::new(Positive::ONE, TimeFrame::Minute, ExpirationDate::Days(days)),
            y: Ystep::new(0, initial_chain),
        },
        walk_type: WalkType::GeometricBrownian {
            dt: convert_time_frame(
                Positive::ONE / days,
                &TimeFrame::Minute,
                &TimeFrame::Day,
            ),
            drift: dec!(0.0),
            volatility: implied_volatility,
        },
        walker,
    };

    let random_walk = RandomWalk::new(
        "Random Walk".to_string(),
        &walk_params,
        generator_optionchain,
    );
    debug!("Random Walk: {}", random_walk);
    let path: &std::path::Path = "Draws/Simulation/random_walk_build_chain.png".as_ref();
    random_walk.write_png(path)?;

    info!("Last Chain: {}", random_walk.last().unwrap().y.value());

    Ok(())
}
