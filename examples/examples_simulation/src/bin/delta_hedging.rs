use optionstratlib::chains::OptionChain;
use optionstratlib::chains::utils::{OptionChainBuildParams, OptionDataPriceParams};
use optionstratlib::pnl::PnLCalculator;
use optionstratlib::strategies::base::Optimizable;
use optionstratlib::strategies::{FindOptimalSide, ShortStrangle};
use optionstratlib::utils::{read_ohlcv_from_zip, setup_logger};
use optionstratlib::{ExpirationDate, Positive, pos, spos};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use tracing::info;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_logger();
    let ohlc = read_ohlcv_from_zip("examples/Data/gc-1m.zip", "01/05/2007", "08/05/2008")?;
    let close_prices = ohlc.iter().map(|candle| candle.close).collect::<Vec<_>>();

    let symbol = "GC".to_string();
    let underlying_price = Positive::from(close_prices[0]);
    let expiration_date = ExpirationDate::Days(pos!(7.0));
    let chain_params = OptionChainBuildParams::new(
        symbol.clone(),
        None,
        30,
        pos!(1.0),
        dec!(0.00003),
        pos!(0.02),
        2,
        OptionDataPriceParams::new(
            underlying_price,
            expiration_date,
            spos!(0.17),
            dec!(0.0),
            pos!(0.0),
            Some(symbol.clone()),
        ),
    );
    let option_chain = OptionChain::build_chain(&chain_params);
    info!("{}", option_chain);

    let mut strategy = ShortStrangle::new(
        symbol,
        underlying_price, // underlying_price
        Positive::ZERO,   // call_strike
        Positive::ZERO,   // put_strike
        expiration_date,
        Positive::ZERO, // implied_volatility
        Decimal::ZERO,  // risk_free_rate
        Positive::ZERO, // dividend_yield
        pos!(1.0),      // quantity
        Positive::ZERO, // premium_short_call
        Positive::ZERO, // premium_short_put
        pos!(0.82),     // open_fee_short_call
        pos!(0.82),     // close_fee_short_call
        pos!(0.82),     // open_fee_short_put
        pos!(0.82),     // close_fee_short_put
    );
    strategy.best_area(&option_chain, FindOptimalSide::Center);
    info!("Strategy:  {:#?}", strategy);
    let iv = option_chain
        .atm_implied_volatility()?
        .unwrap_or(Positive::ZERO);
    let pnl = strategy.calculate_pnl(&underlying_price, expiration_date, &iv)?;
    info!("PnL: {:#?}", pnl);
    Ok(())
}
