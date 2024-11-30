use optionstratlib::chains::chain::OptionChain;
use optionstratlib::chains::utils::RandomPositionsParams;
use optionstratlib::constants::ZERO;
use optionstratlib::model::position::Position;
use optionstratlib::model::types::{ExpirationDate, PositiveF64, PZERO};
use optionstratlib::pos;
use optionstratlib::strategies::base::Strategies;
use optionstratlib::strategies::custom::CustomStrategy;
use optionstratlib::strategies::utils::FindOptimalSide;
use optionstratlib::utils::logger::setup_logger;
use optionstratlib::visualization::utils::Graph;
use std::error::Error;
use tracing::{debug, info};

fn main() -> Result<(), Box<dyn Error>> {
    setup_logger();

    let option_chain =
        OptionChain::load_from_json("./examples/Chains/SP500-18-oct-2024-5781.88.json")?;
    let underlying_price = option_chain.underlying_price;

    let params = RandomPositionsParams::new(
        None,    // qty_puts_long
        Some(1), // qty_puts_short
        None,    // qty_calls_long
        Some(1), // qty_calls_short
        ExpirationDate::Days(30.0),
        pos!(1.0),
        0.05,
        0.02,
        1.0,
        1.0,
        1.0,
        1.0,
    );
    let positions: Vec<Position> = option_chain.get_random_positions(params)?;

    let mut strategy = CustomStrategy::new(
        "Custom Strategy".to_string(),
        "SP500".to_string(),
        "Example of a custom strategy".to_string(),
        underlying_price,
        positions,
        0.01,
        100,
        0.1,
    );
    strategy.best_ratio(&option_chain, FindOptimalSide::All);
    let price_range = strategy.best_range_to_show(pos!(1.0)).unwrap();
    let range = strategy.range_of_profit().unwrap_or(PZERO);
    info!(
        "Range of analysis: {} - {}",
        price_range.first().unwrap(),
        price_range.last().unwrap()
    );
    info!("Title: {}", strategy.title());
    info!("Break Even Points: {:?}", strategy.break_even_points);
    info!(
        "Net Premium Received: ${:.2}",
        strategy.net_premium_received()
    );
    info!("Max Profit: ${:.2}", strategy.max_profit_iter());
    info!("Max Loss: ${:0.2}", strategy.max_loss_iter());
    info!("Total Fees: ${:.2}", strategy.fees());
    info!(
        "Range of Profit: ${:.2} {:.2}%",
        range,
        (range / 2.0) / underlying_price * 100.0
    );
    info!("Profit Ratio: {:.2}%", strategy.profit_ratio());

    if strategy.profit_ratio() > ZERO {
        debug!("Strategy:  {:#?}", strategy);
        strategy.graph(
            &price_range,
            "Draws/Strategy/custom_profit_loss_chart_best_ratio.png",
            20,
            (1400, 933),
        )?;
    }

    Ok(())
}