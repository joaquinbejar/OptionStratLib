/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 12/6/25
******************************************************************************/
use optionstratlib::prelude::*;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use tracing::info;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let symbol = "GER400";
    setup_logger();
    let option_chain = OptionChain::load_from_json(
        "examples/Chains/Germany-40-2025-06-13-16:00:00-UTC-23794.5.json",
    )?;
    info!("Successfully retrieved option chain for {}", symbol);
    info!("{}", option_chain);

    let underlying_price = option_chain.underlying_price;
    let expiration_date = option_chain
        .get_expiration()
        .ok_or("No expiration date found")?;
    let symbol = option_chain.symbol.clone();

    info!("Underlying Price: {}", underlying_price);
    info!("Expiration Date: {}", expiration_date);
    info!("Symbol: {}", symbol);

    let mut strategy = ShortStrangle::new(
        "Germany 40".to_string(),
        underlying_price, // underlying_price
        Positive::ZERO,   // call_strike
        Positive::ZERO,   // put_strike
        expiration_date,
        Positive::ZERO, // implied_volatility
        Positive::ZERO, // implied_volatility
        Decimal::ZERO,  // risk_free_rate
        Positive::ZERO, // dividend_yield
        pos!(1.0),      // quantity
        Positive::ZERO, // premium_short_call
        Positive::ZERO, // premium_short_put
        pos!(0.1),      // open_fee_short_call
        pos!(0.1),      // close_fee_short_call
        pos!(0.1),      // open_fee_short_put
        pos!(0.1),      // close_fee_short_put
    );
    let max_delta = dec!(0.3);
    let min_delta = dec!(0.15);
    strategy.get_best_area(
        &option_chain,
        FindOptimalSide::DeltaRange(min_delta, max_delta),
    );
    info!("Strategy:  {:#?}", strategy);
    info!("Delta:  {:#?}", strategy.delta_neutrality()?);
    strategy.apply_delta_adjustments(Some(Action::Buy))?;
    info!("Delta:  {:#?}", strategy.delta_neutrality()?);
    Ok(())
}
