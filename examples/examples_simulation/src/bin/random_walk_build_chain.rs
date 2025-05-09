use optionstratlib::chains::chain::OptionChain;
use optionstratlib::chains::utils::OptionDataPriceParams;
use optionstratlib::chains::{OptionChainBuildParams, generator_optionchain};
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
    // let n_steps = 43_200; // 30 days in minutes
    let n_steps = 5; // 30 days in minutes
    let underlying_price = pos!(150.0);
    let days = pos!(30.0);
    let std_dev = pos!(20.0);
    let implied_volatility = Some(std_dev / 100.0);
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
        underlying_price,
        ExpirationDate::Days(days),
        implied_volatility,
        risk_free_rate,
        dividend_yield,
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
            dt: convert_time_frame(pos!(1.0) / days, &TimeFrame::Minute, &TimeFrame::Day),
            drift: dec!(0.0),
            volatility: implied_volatility.unwrap(),
        },
        walker: walker,
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
