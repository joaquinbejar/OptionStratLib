use optionstratlib::chains::OptionChain;
use optionstratlib::chains::utils::{OptionChainBuildParams, OptionDataPriceParams};
use optionstratlib::pnl::{PnL, PnLCalculator, PnLMetricsStep};
use optionstratlib::strategies::base::{Optimizable, Positionable};
use optionstratlib::strategies::{FindOptimalSide, ShortStrangle};
use optionstratlib::utils::{read_ohlcv_from_zip, setup_logger};
use optionstratlib::{ExpirationDate, Positive, pos, spos};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use tracing::info;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_logger();
    let ohlc = read_ohlcv_from_zip("examples/Data/gc-1m.zip", "01/05/2007", "08/05/2008")?;
    let close_prices = ohlc
        .iter()
        .map(|candle| Positive::new_decimal(candle.close).unwrap())
        .collect::<Vec<_>>();
    let days = pos!(7.0);
    let symbol = "GC".to_string();
    let underlying_price = close_prices[0];
    let expiration_date = ExpirationDate::Days(days);
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
    let positions = strategy.get_positions()?;
    let mut time_passed = Positive::ZERO;

    let mut pnl_metrics: PnLMetricsStep = PnLMetricsStep::default();
    for price in &close_prices {
        if time_passed >= days {
            break;
        } else {
            let pnl = &positions
                .iter()
                .map(|position| {
                    let expiration_date = ExpirationDate::Days(days - time_passed);
                    match position.calculate_pnl(&price, expiration_date, &iv) {
                        Ok(pnl) => pnl,
                        _ => position.calculate_pnl_at_expiration(&price).unwrap(),
                    }
                })
                .map(|pnl| {
                    let unrealized = pnl.unrealized.unwrap();
                    match unrealized.is_sign_positive() {
                        true => {
                            pnl_metrics.winning_steps += 1;
                            if unrealized > pnl_metrics.max_unrealized_pnl.to_dec() {
                                pnl_metrics.max_unrealized_pnl = Positive::new_decimal(unrealized).unwrap();
                            }
                        }
                        false => {
                            pnl_metrics.losing_steps += 1;
                            if unrealized.abs() > pnl_metrics.min_unrealized_pnl.to_dec() {
                                pnl_metrics.min_unrealized_pnl =
                                    Positive::new_decimal(unrealized.abs()).unwrap();
                            }
                        }
                    }

                    pnl
                })
                .sum::<PnL>();

            info!(
                "Days: {} Price: {} {:?}",
                (days - time_passed.to_dec()).round_to(3),
                price,
                pnl
            );
            pnl_metrics.pnl = pnl.clone();
            pnl_metrics.step_duration = days;
            if pnl_metrics.pnl.realized.unwrap().is_sign_positive() {
                pnl_metrics.win = true;
            } else {
                pnl_metrics.win = false;
            }
            time_passed += pos!(1.0) / (pos!(24.0) * pos!(60.0)); // 1 minute
        }
    }
    info!("PnL: {:?}", pnl_metrics);

    Ok(())
}
