/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 27/10/24
******************************************************************************/

use optionstratlib::chains::chain::OptionChain;
use optionstratlib::chains::utils::OptionChainBuildParams;
use optionstratlib::f2p;
use optionstratlib::simulation::walk::{RandomWalkGraph, Walkable};
use optionstratlib::utils::setup_logger;
use optionstratlib::utils::time::TimeFrame;
use optionstratlib::visualization::utils::Graph;
use optionstratlib::Positive;
use tracing::info;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_logger();
    let years = 3.0;
    let n_steps = 252 * years as usize;
    let initial_price = f2p!(100.0);
    let mean = 0.02;
    let std_dev = f2p!(1.0);
    let std_dev_change = f2p!(0.1);
    let risk_free_rate = Some(0.05);
    let dividend_yield = Some(0.02);
    let volatility_window = 20;
    let initial_volatility = Some(std_dev);

    let chain_symbol = "SP500".to_string();
    let chain_volume = None;
    let chain_size = 10;
    let chain_strike_interval = f2p!(5.0);
    let chain_skew_factor = 0.0001;
    let chain_spread = f2p!(0.02);
    let chain_decimal_places = 2;

    let mut random_walk = RandomWalkGraph::new(
        "Random Walk".to_string(),
        risk_free_rate,
        dividend_yield,
        TimeFrame::Day,
        volatility_window,
        initial_volatility,
    );
    random_walk.generate_random_walk(n_steps, initial_price, mean, std_dev, std_dev_change);
    let _ = random_walk.graph(
        &[],
        "Draws/Simulation/option_chain_from_random_walk.png",
        20,
        (1200, 800),
    );

    for (i, price_params) in random_walk.enumerate() {
        let option_chain_build_params = OptionChainBuildParams::new(
            chain_symbol.clone(),
            chain_volume,
            chain_size,
            chain_strike_interval,
            chain_skew_factor,
            chain_spread,
            chain_decimal_places,
            price_params,
        );
        let chain = OptionChain::build_chain(&option_chain_build_params);

        info!("Step {}: Chain: {}", i, chain,);
    }
    Ok(())
}
