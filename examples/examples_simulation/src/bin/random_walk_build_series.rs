use optionstratlib::prelude::*;
use rust_decimal_macros::dec;
use tracing::{debug, info};

#[warn(dead_code)]
struct Walker {}

impl Walker {
    fn new() -> Self {
        Walker {}
    }
}

impl WalkTypeAble<Positive, OptionSeries> for Walker {}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_logger();
    // let n_steps = 43_200; // 30 days in minutes
    let n_steps = 5; // 30 days in minutes
    let underlying_price = pos!(150.0);
    let days = pos!(30.0);
    let std_dev = pos!(20.0);
    let implied_volatility = std_dev / 100.0;
    let risk_free_rate = dec!(0.02);
    let dividend_yield = pos!(0.01);
    let symbol = "AAPL".to_string();
    let volume = Some(Positive::ONE);
    let chain_size = 30;
    let strike_interval = None;
    let skew_slope = dec!(-0.3);
    let smile_curve = dec!(0.1);
    let spread = pos!(0.01);
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

    let series_params = OptionSeriesBuildParams::new(
        build_params.clone(),
        vec![
            pos!(1.0),
            pos!(2.0),
            pos!(5.0),
            pos!(15.0),
            pos!(30.0),
            pos!(45.0),
            pos!(60.0),
            pos!(120.0),
        ],
    );
    let initial_series = OptionSeries::build_series(&series_params);
    let walker = Box::new(Walker::new());

    let walk_params = WalkParams {
        size: n_steps,
        init_step: Step {
            x: Xstep::new(Positive::ONE, TimeFrame::Minute, ExpirationDate::Days(days)),
            y: Ystep::new(0, initial_series),
        },
        walk_type: WalkType::GeometricBrownian {
            dt: convert_time_frame(pos!(1.0) / days, &TimeFrame::Minute, &TimeFrame::Day),
            drift: dec!(0.0),
            volatility: implied_volatility,
        },
        walker,
    };
    let random_walk = RandomWalk::new(
        "Random Walk".to_string(),
        &walk_params,
        generator_optionseries,
    );
    debug!("Random Walk: {}", random_walk);
    let path: &std::path::Path = "Draws/Simulation/random_walk_build_series.png".as_ref();
    random_walk.write_png(path)?;
    let last = random_walk.last().unwrap().y.value();
    info!("Last Series: {}", last);

    Ok(())
}
