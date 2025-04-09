use itertools::Itertools;
use optionstratlib::chains::OptionChain;
use optionstratlib::chains::utils::{OptionChainBuildParams, OptionDataPriceParams};
use optionstratlib::model::types::Action;
use optionstratlib::pnl::{
    PnL, PnLCalculator, PnLMetricsStep, create_pnl_metrics_document, save_pnl_metrics_with_document,
};
use optionstratlib::strategies::base::{Optimizable, Positionable};
use optionstratlib::strategies::{
    DeltaAdjustment, DeltaNeutrality, FindOptimalSide, ShortStrangle, Strategies,
};
use optionstratlib::utils::others::calculate_log_returns;
use optionstratlib::utils::{OhlcvCandle, TimeFrame, read_ohlcv_from_zip, setup_logger};
use optionstratlib::volatility::{
    annualized_volatility, constant_volatility, historical_volatility,
};
use optionstratlib::{ExpirationDate, Positive, pos};
use rayon::ThreadPoolBuilder;
use rayon::iter::IndexedParallelIterator;
use rayon::iter::ParallelIterator;
use rayon::prelude::IntoParallelRefIterator;
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal_macros::dec;
use std::error::Error;
use std::sync::{Arc, Mutex};
use tracing::{debug, error, info, trace};

const MISPLACEMENT: Decimal = dec!(0.02);

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

    Ok((constant_volatility, historical_volatility, close_prices))
}

fn core(
    ohlc: &Vec<OhlcvCandle>,
    days: Positive,
    symbol: String,
    fee: Positive,
    step: u32,
    delta: Decimal,
    delta_adjustment_at: Decimal,
) -> Result<PnLMetricsStep, Box<dyn Error>> {
    let mut pnl_metrics: PnLMetricsStep = PnLMetricsStep::default();
    pnl_metrics.step_duration = days;

    let mut ohlc_plus = ohlc.clone();
    if let Some(last) = ohlc_plus.last().cloned() {
        ohlc_plus.push(last);
    }

    let (volatility, historical_volatility, close_prices) =
        get_volatilities_from_ohlcv(&ohlc_plus)?;
    info!(
        "Step: {} Annualized volatility {}",
        step,
        volatility.round_to(3)
    );

    let underlying_price = close_prices[0];
    pnl_metrics.initial_price = underlying_price;
    let expiration_date = ExpirationDate::Days(days);
    let chain_params = OptionChainBuildParams::new(
        symbol.clone(),
        None,
        30,
        Positive::ONE,
        dec!(0.00003),
        pos!(0.02),
        2,
        OptionDataPriceParams::new(
            underlying_price,
            expiration_date,
            Some(volatility),
            Decimal::ZERO,
            Positive::ZERO,
            Some(symbol.clone()),
        ),
    );
    let option_chain = OptionChain::build_chain(&chain_params);
    debug!("{}", option_chain);

    let mut strategy = ShortStrangle::new(
        symbol,
        underlying_price, // underlying_price
        Positive::ZERO,   // call_strike
        Positive::ZERO,   // put_strike
        expiration_date,
        volatility,     // implied_volatility
        Decimal::ZERO,  // risk_free_rate
        Positive::ZERO, // dividend_yield
        Positive::ONE,  // quantity
        Positive::ZERO, // premium_short_call
        Positive::ZERO, // premium_short_put
        fee,            // open_fee_short_call
        fee,            // close_fee_short_call
        fee,            // open_fee_short_put
        fee,            // close_fee_short_put
    );
    strategy.best_ratio(&option_chain, FindOptimalSide::DeltaRange(-delta, delta));
    debug!("Strategy:  {:#?}", strategy);

    strategy.apply_delta_adjustments(Some(Action::Buy))?;
    debug!("Delta: {:?}", strategy.delta_neutrality());

    {
        let positions = strategy.get_positions()?;
        positions.iter().for_each(|position| {
            pnl_metrics.strikes.push(position.option.strike_price);
            pnl_metrics.initial_volumes.push(position.option.quantity);
        });
    }
    let mut time_passed = Positive::ZERO;

    for (i, price) in close_prices.into_iter().enumerate() {
        let iv = historical_volatility[i].round_to(3) + MISPLACEMENT;
        // print!("set_underlying_price: ",);
        strategy.set_underlying_price(&price)?; // fail modify_position
        // print!(" {}", strategy.get_underlying_price());

        let mut pnl_init: PnL = PnL::default();
        let delta_info = strategy.delta_neutrality()?;
        if delta_info.net_delta.abs() >= delta_adjustment_at {
            pnl_metrics.delta_adjustments += Positive::ONE;
            trace!("buy net_delta: {}", delta_info.net_delta);

            // get the delta adjustments and calculate the pnl of the transaction
            let adjustments = strategy.delta_adjustments()?;

            if strategy.volume()? <= Positive::TEN {
                pnl_init.realized = if let Some(adjustment) = adjustments
                    .iter()
                    .find(|adj| matches!(adj, DeltaAdjustment::BuyOptions { .. }))
                {
                    strategy.adjustments_pnl(&adjustment)?
                } else {
                    PnL::default()
                }
                .realized;

                match strategy.apply_delta_adjustments(Some(Action::Buy)) {
                    Ok(_) => {}
                    Err(_) => {
                        debug!("Not enough premium to buy. try to apply Sell delta adjustment");
                        strategy.apply_delta_adjustments(Some(Action::Sell))?;
                    }
                };
            } else {
                pnl_init.realized = if let Some(adjustment) = adjustments
                    .iter()
                    .find(|adj| matches!(adj, DeltaAdjustment::SellOptions { .. }))
                {
                    strategy.adjustments_pnl(&adjustment)?
                } else {
                    PnL::default()
                }
                .realized;

                match strategy.apply_delta_adjustments(Some(Action::Sell)) {
                    Ok(_) => {}
                    Err(_) => {
                        debug!("Not enough qty to sell");
                        strategy.apply_delta_adjustments(None)?;
                    }
                };
            }
        }

        let positions = strategy.get_positions()?;

        if time_passed >= days {
            break;
        } else {
            let pnl = &positions
                .iter()
                .map(|position| {
                    let expiration_date = ExpirationDate::Days(days - time_passed);
                    match position.calculate_pnl(&price, expiration_date, &iv) {
                        Ok(pnl) => pnl,
                        _ => {
                            pnl_metrics.final_volumes.push(position.option.quantity);
                            position.calculate_pnl_at_expiration(&price).unwrap()
                        }
                    }
                })
                .map(|pnl| {
                    let unrealized = pnl.unrealized.unwrap();
                    match unrealized.is_sign_positive() {
                        true => {
                            pnl_metrics.winning_steps += 1;
                            if unrealized > pnl_metrics.max_unrealized_pnl.to_dec() {
                                pnl_metrics.max_unrealized_pnl =
                                    Positive::new_decimal(unrealized).unwrap();
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

            debug!(
                "Days: {} Price: {} IV: {} {}",
                (days - time_passed.to_dec()).round_to(3),
                price,
                iv,
                pnl
            );
            pnl_metrics.pnl = pnl.clone();
            pnl_metrics.pnl.realized =
                Some(pnl_metrics.pnl.realized.unwrap() - &pnl_init.initial_costs);
            pnl_metrics.pnl.realized =
                Some(pnl_metrics.pnl.realized.unwrap() + &pnl_init.initial_income);
            pnl_metrics.final_price = price;
            if pnl_metrics.pnl.realized.unwrap().is_sign_positive() {
                pnl_metrics.win = true;
            } else {
                pnl_metrics.win = false;
            }
            time_passed += pos!(1.0) / (pos!(24.0) * pos!(60.0)); // 1 minute
        }
    }
    pnl_metrics.step_number = step;
    Ok(pnl_metrics)
}

// fn main() -> Result<(), Box<dyn Error>> {
//     setup_logger();
//     // let ohlc = read_ohlcv_from_zip("examples/Data/cl-1m-sample.zip", None, None)?;
//     let ohlc = read_ohlcv_from_zip(
//         "examples/Data/gc-1m.zip",
//         Some("01/05/2007"),
//         Some("01/10/2007"),
//     )?;
//     // let ohlc = read_ohlcv_from_zip("examples/Data/gc-1m.zip", None, None)?;
//
//
//     let symbol = "GC".to_string();
//     let fee= pos!(0.10);
//     let deltas = vec![dec!(0.25)];
//     let dayss = vec![pos!(15.0)];
//     let delta_adjustment_at = vec![dec!(0.5)];
//     let file_path = format!(
//         "examples/Data/{}_short_strangle_metrics_vol{}plus.json",
//         symbol,
//         (MISPLACEMENT * Decimal::ONE_HUNDRED).to_i32().unwrap(),
//     );
//
//     for delta in deltas {
//         for days in &dayss {
//             for daa in &delta_adjustment_at {
//                 let chunk_size = (*days * 24.0 * 60.0).to_i64() as usize;
//                 info!(
//                     "Chunk size: {} # Steps: {}",
//                     chunk_size,
//                     ohlc.len() / chunk_size
//                 );
//                 let mut pnl_results: Vec<PnLMetricsStep> = Vec::new();
//
//                 for (step, chunk) in ohlc.chunks_exact(chunk_size).enumerate() {
//                     let ohlc = chunk.to_vec();
//                     let pnl_metrics = core(
//                         &ohlc,
//                         *days,
//                         symbol.clone(),
//                         fee,
//                         step as u32,
//                         delta,
//                         *daa,
//                     )?;
//                     info!("{}", pnl_metrics);
//                     pnl_results.push(pnl_metrics);
//                     // break
//                 }
//
//
//                 let pnl_metrics_document = create_pnl_metrics_document(
//                     pnl_results,
//                     *days,
//                     symbol.clone(),
//                     fee,
//                     delta,
//                     *daa,
//                 );
//                 save_pnl_metrics_with_document(&pnl_metrics_document, &*file_path)?;
//             }
//         }
//     }
//     Ok(())
// }

fn main() -> Result<(), Box<dyn Error>> {
    setup_logger();
    // let ohlc = read_ohlcv_from_zip("examples/Data/gc-1m.zip", Some("01/05/2007"), Some("08/05/2008"))?;
    let ohlc = read_ohlcv_from_zip("examples/Data/gc-1m.zip", None, None)?;

    let symbol = "GC".to_string();
    let file_path = format!(
        "examples/Data/{}_short_strangle_metrics_vol{}plus.json",
        symbol,
        (MISPLACEMENT * Decimal::ONE_HUNDRED).to_i32().unwrap(),
    );
    let fee = pos!(0.10);
    let deltas = vec![
        dec!(0.15),
        dec!(0.20),
        dec!(0.25),
        dec!(0.30),
        dec!(0.35),
        dec!(0.40),
    ];
    let dayss = vec![
        pos!(5.0),
        pos!(15.0),
        pos!(25.0),
        pos!(35.0),
        pos!(45.0),
        pos!(60.0),
        pos!(90.0),
    ];
    let delta_adjustment_at = vec![dec!(0.5), dec!(1.0), dec!(2.0)];
    for delta in deltas {
        for days in &dayss {
            for daa in &delta_adjustment_at {
                let chunk_size = (*days * 24.0 * 60.0).to_i64() as usize;
                info!(
                    "Chunk size: {} # Steps: {}",
                    chunk_size,
                    ohlc.len() / chunk_size
                );

                // Create chunks vector
                let chunks: Vec<_> = ohlc.chunks_exact(chunk_size).collect();
                let num_chunks = chunks.len();

                // Create a thread-safe container for results with step info
                // Using a tuple to store the step and the result
                let pnl_results = Arc::new(Mutex::new(
                    Vec::<(u32, PnLMetricsStep)>::with_capacity(num_chunks),
                ));

                // Configure the thread pool
                let num_threads = num_cpus::get() - 1;
                info!("Using {} threads for parallel processing", num_threads);

                // Create and execute the thread pool
                ThreadPoolBuilder::new()
                    .num_threads(num_threads)
                    .build()?
                    .install(|| {
                        chunks.par_iter().enumerate().for_each(|(step, chunk)| {
                            let step_u32 = step as u32;
                            let ohlc = chunk.to_vec();
                            match core(&ohlc, *days, "GC".to_string(), fee, step_u32, delta, *daa) {
                                Ok(pnl_metrics) => {
                                    info!("{}", pnl_metrics);
                                    // Store both the step and the metrics
                                    let mut results = pnl_results.lock().unwrap();
                                    results.push((step_u32, pnl_metrics));
                                }
                                Err(e) => {
                                    error!("Error processing chunk {}: {:?}", step, e);
                                }
                            }
                        });
                    });

                // Get the final results
                let final_results = Arc::try_unwrap(pnl_results)
                    .expect("Thread pool should be done with the results")
                    .into_inner()?;

                // Sort results by step and extract just the PnLMetricsStep
                let sorted_results: Vec<PnLMetricsStep> = final_results
                    .into_iter()
                    .sorted_by_key(|(step, _)| *step)
                    .map(|(_, metrics)| metrics)
                    .collect();

                let pnl_metrics_document = create_pnl_metrics_document(
                    sorted_results,
                    *days,
                    symbol.clone(),
                    fee,
                    delta,
                    *daa,
                );
                save_pnl_metrics_with_document(&pnl_metrics_document, &*file_path)?;
            }
        }
    }
    Ok(())
}
