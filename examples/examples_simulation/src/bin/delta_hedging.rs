use std::error::Error;
use optionstratlib::chains::OptionChain;
use optionstratlib::chains::utils::{OptionChainBuildParams, OptionDataPriceParams};
use optionstratlib::pnl::{PnL, PnLCalculator, PnLMetricsStep};
use optionstratlib::strategies::base::{Optimizable, Positionable};
use optionstratlib::strategies::{FindOptimalSide, ShortStrangle};
use optionstratlib::utils::{read_ohlcv_from_zip, setup_logger, OhlcvCandle, TimeFrame};
use optionstratlib::{ExpirationDate, Positive, pos};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use tracing::info;
use optionstratlib::utils::others::calculate_log_returns;
use optionstratlib::volatility::{annualized_volatility, constant_volatility, historical_volatility};

/// Extracts volatility metrics from a series of OHLCV candles.
///
/// This function processes a vector of OHLCV candles and calculates three important volatility measures:
/// 1. A constant volatility value calculated from the entire dataset
/// 2. A vector of historical volatilities using a 60-period moving window
/// 3. The original close prices converted to Positive values
///
/// The function performs the following steps:
/// - Extracts close prices from candles and converts them to Positive values
/// - Calculates logarithmic returns from those prices
/// - Computes annualized volatility for each log return using a minute timeframe
/// - Calculates constant volatility across all returns
/// - Calculates historical volatility using a 60-period moving window
/// - Pads the historical volatility vector to match the original data length
///
/// # Arguments
///
/// * `ohlc` - A reference to a vector of OhlcvCandle structs containing price data
///
/// # Returns
///
/// A tuple containing:
/// - A constant volatility value (Positive)
/// - A vector of historical volatilities (Vec<Positive>)
/// - A vector of close prices (Vec<Positive>)
///
/// # Errors
///
/// Returns an error if any of the intermediate calculations fail, such as:
/// - Failed to calculate log returns
/// - Failed to calculate constant or historical volatility
///
fn get_volatilities_from_ohlcv(
    ohlc: &Vec<OhlcvCandle>,
) -> Result<(Positive, Vec<Positive>, Vec<Positive>), Box<dyn Error>> {
    let close_prices = ohlc
        .iter()
        .map(|candle| Positive::new_decimal(candle.close).unwrap())
        .collect::<Vec<_>>();

    let log_returns = calculate_log_returns(&close_prices)?;
    let binding = log_returns
        .iter()
        .map(|&log| annualized_volatility(log, TimeFrame::Minute).unwrap())
        .map(|log| log.to_dec())
        .collect::<Vec<Decimal>>();
    let log_returns_dec = binding.as_slice();
    let constant_volatility = constant_volatility(log_returns_dec)?;
    let mut historical_volatility = historical_volatility(log_returns_dec, 60)?;
    if let Some(&last) = historical_volatility.last() {
        historical_volatility.extend_from_slice(&[last].repeat(60));
    }
    
    Ok( (
        constant_volatility,
        historical_volatility,
        close_prices
    ) )
}

fn main() -> Result<(), Box<dyn Error>> {
    setup_logger();
    // let ohlc = read_ohlcv_from_zip("examples/Data/cl-1m-sample.zip", Some("01/05/2007"), Some("08/05/2008"))?;
    let ohlc = read_ohlcv_from_zip("examples/Data/cl-1m-sample.zip", None, None)?;
    let (volatility, historical_volatility, close_prices) = get_volatilities_from_ohlcv(&ohlc)?;
    info!("Annualized volatility {:?}", volatility);
    info!("Historical Volatility Length{:?}", historical_volatility.len());
    info!("Prices Length{:?}", close_prices.len());
    

    let days = pos!(7.0);
    let symbol = "CL".to_string();
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
            Some(volatility),
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
        volatility, // implied_volatility
        Decimal::ZERO,  // risk_free_rate
        Positive::ZERO, // dividend_yield
        Positive::ONE,      // quantity
        Positive::ZERO, // premium_short_call
        Positive::ZERO, // premium_short_put
        pos!(0.05),     // open_fee_short_call
        pos!(0.05),     // close_fee_short_call
        pos!(0.05),     // open_fee_short_put
        pos!(0.05),     // close_fee_short_put
    );
    strategy.best_ratio(&option_chain, FindOptimalSide::Center);
    info!("Strategy:  {:#?}", strategy);


    let positions = strategy.get_positions()?;
    let mut time_passed = Positive::ZERO;

    let mut pnl_metrics: PnLMetricsStep = PnLMetricsStep::default();
    for (i, price) in close_prices.into_iter().enumerate() {
        let iv = historical_volatility[i].round_to(3);
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
                "Days: {} Price: {} IV: {} {:?}",
                (days - time_passed.to_dec()).round_to(3),
                price,
                iv,
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
