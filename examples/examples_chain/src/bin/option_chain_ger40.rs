use positive::pos_or_panic;
/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 29/1/25
******************************************************************************/
use optionstratlib::prelude::*;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use tracing::{debug, info};

fn main() -> Result<(), optionstratlib::error::Error> {
    setup_logger();
    let mut option_chain = OptionChain::load_from_json(
        "examples/Chains/Germany-40-2025-05-27-15-29-00-UTC-24209.json",
    )?;

    info!("Chain loaded");
    option_chain.update_greeks();
    info!("{}", &option_chain);

    let mut strategy = ShortStrangle::new(
        option_chain.get_title(),
        option_chain.underlying_price,
        Positive::ZERO, // call_strike
        Positive::ZERO, // put_strike
        ExpirationDate::Days(pos_or_panic!(0.2)),
        Positive::ZERO,      // implied_volatility
        Positive::ZERO,      // implied_volatility
        Decimal::ZERO,       // risk_free_rate
        Positive::ZERO,      // dividend_yield
        Positive::ONE,       // quantity
        Positive::ZERO,      // premium_short_call
        Positive::ZERO,      // premium_short_put
        pos_or_panic!(0.10), // open_fee_short_call
        pos_or_panic!(0.10), // close_fee_short_call
        pos_or_panic!(0.10), // open_fee_short_put
        pos_or_panic!(0.10), // close_fee_short_put
    );
    strategy.get_best_ratio(
        &option_chain,
        FindOptimalSide::DeltaRange(dec!(-0.3), dec!(0.3)),
    );
    strategy.apply_delta_adjustments(Some(Action::Buy))?;
    info!("Strategy:  {}", strategy);
    info!("Break Even Points: {:?}", strategy.break_even_points);
    info!(
        "Net Premium Received: ${:.2}",
        strategy.get_net_premium_received()?
    );
    info!(
        "Max Profit: ${:.2}",
        strategy.get_max_profit().unwrap_or(Positive::ZERO)
    );
    info!("Profit Area: {:.2}%", strategy.get_profit_area()?);
    info!("Delta:  {:#?}", strategy.delta_neutrality()?);
    if strategy.get_profit_ratio()? > Positive::ZERO.into() {
        let path: &std::path::Path = "Draws/Chains/short_strangle_ger40_area.html".as_ref();
        strategy.write_html(path)?;
    }
    info!("Greeks:  {:#?}", strategy.greeks());

    if strategy.get_profit_ratio()? > Positive::ZERO.into() {
        debug!("Strategy:  {:#?}", strategy);
        let file_path = "Draws/Chains/short_strangle_ger40_area.png";
        let path: &std::path::Path = file_path.as_ref();
        strategy.write_png(path)?;
    }

    Ok(())
}
