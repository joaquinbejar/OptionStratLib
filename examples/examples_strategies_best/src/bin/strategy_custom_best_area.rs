use optionstratlib::chains::chain::OptionChain;
use optionstratlib::chains::utils::RandomPositionsParams;
use optionstratlib::constants::ZERO;
use optionstratlib::f2p;
use optionstratlib::model::position::Position;
use optionstratlib::strategies::base::{Optimizable, Strategies};
use optionstratlib::strategies::custom::CustomStrategy;
use optionstratlib::strategies::utils::FindOptimalSide;
use optionstratlib::utils::setup_logger;
use optionstratlib::visualization::utils::Graph;
use optionstratlib::ExpirationDate;
use optionstratlib::Positive;
use std::error::Error;
use tracing::{debug, info};

fn main() -> Result<(), Box<dyn Error>> {
    setup_logger();

    let option_chain =
        OptionChain::load_from_json("./examples/Chains/SP500-18-oct-2024-5781.88.json")?;
    let underlying_price = option_chain.underlying_price;

    let params = RandomPositionsParams::new(
        None,    // qty_puts_long
        None,    // qty_puts_short
        Some(1), // qty_calls_long
        Some(1), // qty_calls_short
        ExpirationDate::Days(30.0),
        f2p!(1.0),
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
    strategy.best_area(&option_chain, FindOptimalSide::All);
    debug!("Strategy:  {:#?}", strategy);
    let price_range = strategy.best_range_to_show(f2p!(1.0)).unwrap();
    info!(
        "Price Range from: {} to: {}",
        price_range.first().unwrap(),
        price_range.last().unwrap()
    );
    let range = strategy.range_of_profit().unwrap_or(Positive::ZERO);
    info!("Title: {}", strategy.title());
    info!("Break Even Points: {:?}", strategy.break_even_points);
    info!(
        "Net Premium Received: ${:.2}",
        strategy.net_premium_received()?
    );
    info!("Max Profit: ${:.2}", strategy.max_profit_iter());
    info!("Max Loss: ${:0.2}", strategy.max_loss_iter());
    info!("Total Fees: ${:.2}", strategy.fees()?);
    info!(
        "Range of Profit: ${:.2} {:.2}%",
        range,
        (range / 2.0) / underlying_price * 100.0
    );
    info!("Profit Area: {:.2}%", strategy.profit_area()?);

    if strategy.profit_ratio()? > Positive::ZERO.into() {
        debug!("Strategy:  {:#?}", strategy);
        strategy.graph(
            &price_range,
            "Draws/Strategy/custom_profit_loss_chart_best_area.png",
            20,
            (1400, 933),
        )?;
    }

    Ok(())
}
