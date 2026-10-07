use osl_example_support::setup_logger;
use positive::pos_or_panic;
/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 25/9/24
******************************************************************************/
use optionstratlib::error::Error;
use optionstratlib::prelude::*;

fn main() -> Result<(), Error> {
    setup_logger();
    let option_chain =
        OptionChain::load_from_json("./examples/Chains/SP500-18-oct-2024-5781.88.json")?;
    let underlying_price = option_chain.underlying_price;
    let mut strategy = BullCallLadder::new(
        "SP500".to_string(),
        underlying_price, // underlying_price
        // Seed legs: `new` validates them since #706, so they must form the
        // ladder; the optimizer below replaces them with the chain's best.
        pos_or_panic!(5750.0), // long_call_strike
        pos_or_panic!(5800.0), // short_call_low_strike
        pos_or_panic!(5850.0), // short_call_high_strike
        ExpirationDate::Days(Positive::TWO),
        Positive::ZERO,      // implied_volatility
        dec!(0.05),          // risk_free_rate
        Positive::ZERO,      // dividend_yield
        Positive::TWO,       // quantity
        Positive::ONE,       // premium_long_call
        Positive::ONE,       // premium_short_call_low
        Positive::ONE,       // premium_short_call_high
        Positive::ZERO,      // open_fee_long
        pos_or_panic!(0.78), // close_fee_long
        pos_or_panic!(0.78), // open_fee_short_low
        pos_or_panic!(0.73), // close_fee_short_low
        pos_or_panic!(0.73), // open_fee_short_high
        pos_or_panic!(0.73), // close_fee_short_high
    )?;

    strategy.get_best_ratio(
        &option_chain,
        FindOptimalSide::Range(pos_or_panic!(5700.0), pos_or_panic!(6000.0)),
    );
    let range = strategy.get_range_of_profit().unwrap_or(Positive::ZERO);
    info!("Title: {}", strategy.get_title());
    info!("Break Even Points: {:?}", strategy.break_even_points);
    info!(
        "Net Premium Received: ${:.2}",
        strategy.get_net_premium_received()?
    );
    info!(
        "Max Profit: ${:.2}",
        strategy.get_max_profit().unwrap_or(Positive::ZERO)
    );
    info!(
        "Max Loss: ${:0.2}",
        strategy.get_max_loss().unwrap_or(Positive::ZERO)
    );
    info!("Total Fees: ${:.2}", strategy.get_fees()?);
    info!(
        "Range of Profit: ${:.2} {:.2}%",
        range,
        (range / 2.0) / underlying_price * 100.0
    );
    info!("Profit Ratio: {:.2}%", strategy.get_profit_ratio()?);
    debug!("Strategy:  {:#?}", strategy);
    let path: &std::path::Path =
        "Draws/Strategy/bull_call_ladder_profit_loss_chart_best_ratio.png".as_ref();
    strategy.write_png(path)?;

    Ok(())
}
