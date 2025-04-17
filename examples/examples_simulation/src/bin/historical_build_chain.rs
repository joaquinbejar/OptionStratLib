use optionstratlib::chains::chain::OptionChain;
use optionstratlib::chains::utils::OptionDataPriceParams;
use optionstratlib::chains::{OptionChainBuildParams, generator_optionchain};
use optionstratlib::simulation::randomwalk::RandomWalk;
use optionstratlib::simulation::steps::{Step, Xstep, Ystep};
use optionstratlib::simulation::{WalkParams, WalkType, WalkTypeAble};
use optionstratlib::utils::others::calculate_log_returns;
use optionstratlib::utils::time::{TimeFrame, get_x_days_formatted};
use optionstratlib::utils::{read_ohlcv_from_zip, setup_logger};
use optionstratlib::visualization::utils::{Graph, GraphBackend};
use optionstratlib::volatility::{adjust_volatility, constant_volatility};
use optionstratlib::{ExpirationDate, Positive, pos};
use rust_decimal::Decimal;
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

    let ohlc = read_ohlcv_from_zip("examples/Data/cl-1m-sample.zip", None, None)?;
    let ohlc = ohlc.iter().take(1000).collect::<Vec<_>>(); // Take only 1000 minutes
    let prices: Vec<Positive> = ohlc.iter().map(|x| Positive::from(x.close)).collect();
    let log_returns: Vec<Decimal> = calculate_log_returns(&prices)?
        .iter()
        .map(|p| p.to_dec())
        .collect();
    let implied_volatility = adjust_volatility(
        constant_volatility(&log_returns)?,
        TimeFrame::Minute,
        TimeFrame::Year,
    )?;

    info!("Volatility {}", implied_volatility);
    info!("Prices Lenght: {}", prices.len());

    let n_steps = prices.len();
    let underlying_price = prices[0];
    let days = pos!(30.0);
    // let std_dev = implied_volatility * 100.0;
    let risk_free_rate = dec!(0.02);
    let dividend_yield = pos!(0.01);
    let symbol = "AAPL".to_string();
    let volume = Some(Positive::ONE);
    let chain_size = 30;
    let strike_interval = pos!(1.0);
    let skew_factor = dec!(0.0005);
    let spread = pos!(0.01);
    let decimal_places = 2;

    let price_params = OptionDataPriceParams::new(
        underlying_price,
        ExpirationDate::Days(days),
        Some(implied_volatility),
        risk_free_rate,
        dividend_yield,
        Some(symbol.clone()),
    );
    let build_params = OptionChainBuildParams::new(
        symbol,
        volume,
        chain_size,
        strike_interval,
        skew_factor,
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
        walk_type: WalkType::Historical {
            timeframe: TimeFrame::Minute,
            prices,
        },
        walker: walker,
    };

    let random_walk = RandomWalk::new(
        "Random Walk".to_string(),
        &walk_params,
        generator_optionchain,
    );
    debug!("Random Walk: {}", random_walk);

    random_walk.graph(
        GraphBackend::Bitmap {
            file_path: "Draws/Simulation/historical_build_chain.png",
            size: (1200, 800),
        },
        20,
    )?;
    info!("First Chain: {}", random_walk.first().unwrap().y.value());
    info!("Last Chain: {}", random_walk.last().unwrap().y.value());

    Ok(())
}
