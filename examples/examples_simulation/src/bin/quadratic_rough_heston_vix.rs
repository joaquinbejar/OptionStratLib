/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 7/11/26
******************************************************************************/

//! # VIX under the quadratic rough Heston model
//!
//! Reads the index at the initial factor state over the standard 30-day window, follows it
//! along one simulated path, and prices a VIX future and a small VIX smile by Monte Carlo at
//! two seeds. As with the vanilla prices, one run does not show that a price has converged:
//! compare seeds and step counts.
//!
//! **Every parameter here is illustrative.** None of them is calibrated to any underlying,
//! and the papers' values are for SPX and VIX. The prices have no discounting, as with
//! `r = q = 0`, and the index is in volatility points.

use optionstratlib::prelude::*;
use optionstratlib::simulation::{
    QuadraticRoughHestonGrid, QuadraticRoughHestonKernel, QuadraticRoughHestonParams,
    QuadraticRoughHestonVix, QuadraticRoughHestonVixPayoff, price_vix_quadratic_rough_heston,
    simulate_quadratic_rough_heston_path,
};
use osl_example_support::setup_logger;
use rust_decimal_macros::dec;
use std::num::NonZeroUsize;
use tracing::info;

/// The seed of the path and of the first prices.
const SEED: u64 = 968;

/// The seed to compare the first prices with.
const OTHER_SEED: u64 = 969;

/// The maturity of the future and the options, in years.
const HORIZON: Decimal = dec!(0.25);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_logger();

    // Illustrative values: `alpha = 0.6`, 5 factors, ratio 5.
    let kernel = QuadraticRoughHestonKernel::try_new(dec!(0.6), 5, dec!(5))?;
    // Illustrative values: `a`, `b`, `c`, `lambda`, `eta`, the initial factors and the spot.
    let params = QuadraticRoughHestonParams::try_new(
        dec!(0.5),
        dec!(0.1),
        dec!(0.01),
        dec!(1.5),
        dec!(1),
        kernel,
        vec![dec!(-0.05); 5],
        Positive::HUNDRED,
    )?;

    // The index as a quadratic form in the factors, over `30 / 365` years.
    let vix = QuadraticRoughHestonVix::thirty_days(&params)?;
    info!(
        window_years = %vix.window(),
        initial_variance = %params.initial_variance(),
        vix = %vix.at(params.initial_factors())?,
        "vix at the initial state"
    );

    // The index along one path, every 10 steps.
    let grid = QuadraticRoughHestonGrid::try_new(HORIZON, 100)?;
    let path = simulate_quadratic_rough_heston_path(&params, &grid, SEED)?;
    for step in (0..=grid.steps()).step_by(10) {
        if let Some(factors) = path.factors_at(step) {
            info!(
                step,
                spot = ?path.spot().get(step),
                variance = ?path.variance().get(step),
                vix = %vix.at(factors)?,
                "along one path"
            );
        }
    }

    // The future and a small smile, at two seeds.
    let paths = NonZeroUsize::try_from(50_000_usize)?;
    for seed in [SEED, OTHER_SEED] {
        let future = price_vix_quadratic_rough_heston(
            &params,
            &grid,
            vix.window(),
            QuadraticRoughHestonVixPayoff::Future,
            paths,
            seed,
        )?;
        info!(
            seed,
            future = %future.price(),
            standard_error = %future.standard_error(),
            "vix future"
        );
        for strike in [dec!(10), dec!(15), dec!(20), dec!(25), dec!(30)] {
            let strike = Positive::new_decimal(strike)?;
            for style in [OptionStyle::Call, OptionStyle::Put] {
                let estimate = price_vix_quadratic_rough_heston(
                    &params,
                    &grid,
                    vix.window(),
                    QuadraticRoughHestonVixPayoff::Vanilla { style, strike },
                    paths,
                    seed,
                )?;
                info!(
                    seed,
                    style = ?style,
                    strike = %strike,
                    price = %estimate.price(),
                    standard_error = %estimate.standard_error(),
                    "vix option"
                );
            }
        }
    }
    Ok(())
}
