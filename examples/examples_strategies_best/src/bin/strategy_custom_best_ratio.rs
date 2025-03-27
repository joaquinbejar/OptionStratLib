use optionstratlib::ExpirationDate;
use optionstratlib::chains::chain::OptionChain;
use optionstratlib::chains::utils::RandomPositionsParams;
use optionstratlib::model::position::Position;
use optionstratlib::strategies::base::{Optimizable, Strategies};
use optionstratlib::strategies::custom::CustomStrategy;
use optionstratlib::strategies::utils::FindOptimalSide;
use optionstratlib::utils::setup_logger;
use optionstratlib::visualization::utils::{Graph, GraphBackend};
use optionstratlib::{Positive, pos};
use rust_decimal_macros::dec;
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
        ExpirationDate::Days(pos!(30.0)),
        pos!(1.0),
        dec!(0.05),
        pos!(0.02),
        Positive::ONE,
        Positive::ONE,
        Positive::ONE,
        Positive::ONE,
    );
    let positions: Vec<Position> = option_chain.get_random_positions(params)?;

    let mut strategy = CustomStrategy::new(
        "Custom Strategy".to_string(),
        "SP500".to_string(),
        "Example of a custom strategy".to_string(),
        underlying_price,
        positions,
        pos!(0.01),
        100,
        pos!(0.1),
    );
    strategy.best_ratio(&option_chain, FindOptimalSide::All);
    let range = strategy.range_of_profit().unwrap_or(Positive::ZERO);

    info!("Title: {}", strategy.title());
    info!("Break Even Points: {:?}", strategy.break_even_points);
    info!(
        "Net Premium Received: ${:.2}",
        strategy.net_premium_received()?
    );
    info!("Max Profit: ${:.2}", strategy.max_profit_iter()?);
    info!("Max Loss: ${:0.2}", strategy.max_loss_iter()?);
    info!("Total Fees: ${:.2}", strategy.fees()?);
    info!(
        "Range of Profit: ${:.2} {:.2}%",
        range,
        (range / 2.0) / underlying_price * 100.0
    );
    info!("Profit Ratio: {:.2}%", strategy.profit_ratio()?);

    if strategy.profit_ratio()? > Positive::ZERO.into() {
        debug!("Strategy:  {:#?}", strategy);
        strategy.graph(
            GraphBackend::Bitmap {
                file_path: "Draws/Strategy/custom_profit_loss_chart_best_ratio.png",
                size: (1400, 933),
            },
            20,
        )?;
    }

    Ok(())
}
