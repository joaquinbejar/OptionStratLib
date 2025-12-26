use optionstratlib::ExpirationDate;
use optionstratlib::chains::chain::OptionChain;
use optionstratlib::chains::utils::{OptionChainBuildParams, OptionDataPriceParams};
use positive::Positive;
use rust_decimal_macros::dec;

fn main() -> Result<(), optionstratlib::error::Error> {
    // Create OptionDataPriceParams first
    let price_params = OptionDataPriceParams::new(
        Some(Box::new(Positive::new(150.0)?)), // underlying_price
        Some(ExpirationDate::Days(Positive::new(30.0)?)), // expiration_date
        Some(dec!(0.05)),                      // risk_free_rate
        Some(Positive::new(0.02)?),            // dividend_yield
        Some("SPY".to_string()),               // underlying_symbol
    );

    // Create OptionChainBuildParams with strikes that include multiples of 25
    // This should create strikes around 150, including 125, 150, 175 (all multiples of 25)
    let params = OptionChainBuildParams::new(
        "SPY".to_string(),           // symbol
        Some(Positive::new(100.0)?), // volume
        10,                          // chain_size (more strikes for testing)
        Some(Positive::new(5.0)?),   // strike_interval (5-point intervals)
        dec!(0.1),                   // skew_slope
        dec!(0.05),                  // smile_curve
        Positive::new(0.02)?,        // spread
        2,                           // decimal_places
        price_params,                // price_params
        Positive::new(0.20)?,        // implied_volatility
    );

    let mut chain = OptionChain::build_chain(&params);
    chain.update_greeks();

    tracing::info!("=== Testing Yellow Highlighting for Strike Prices Multiple of 25 ===");
    tracing::info!("Expected: Rows with strike prices 125, 150, 175 should be in YELLOW");
    tracing::info!("Other strikes (130, 135, 140, 145, 155, 160, 165, 170) should be normal color");

    // Show the chain with the new yellow highlighting
    chain.show();

    Ok(())
}
