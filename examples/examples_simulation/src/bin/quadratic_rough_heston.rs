/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 6/11/26
******************************************************************************/

//! # Quadratic rough Heston
//!
//! Builds the multi-factor kernel, simulates one path by the scheme (A.1) and prices an ATM
//! call and put by Monte Carlo, at two seeds and two step sizes. One run does not show that a
//! price has converged, even with a small standard error: compare seeds and step counts, as
//! `QuadraticRoughHestonParams` explains.
//!
//! **Every parameter here is illustrative.** None of them is calibrated to any underlying,
//! and the papers' values are for SPX and VIX. The price has no discounting, as with
//! `r = q = 0`.

use optionstratlib::prelude::*;
use optionstratlib::simulation::{
    QuadraticRoughHestonGrid, QuadraticRoughHestonKernel, QuadraticRoughHestonParams,
    price_vanilla_quadratic_rough_heston, simulate_quadratic_rough_heston_path,
};
use osl_example_support::setup_logger;
use rust_decimal_macros::dec;
use std::num::NonZeroUsize;
use tracing::info;

/// The seed of the path and of the first prices.
const SEED: u64 = 958;

/// The seed to compare the first prices with.
const OTHER_SEED: u64 = 959;

/// The horizon, in years.
const HORIZON: Decimal = dec!(0.25);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_logger();

    // Illustrative values: `alpha = 0.6`, 5 factors, ratio 5.
    let kernel = QuadraticRoughHestonKernel::try_new(dec!(0.6), 5, dec!(5))?;
    info!(weights = ?kernel.weights(), discounts = ?kernel.discounts(), "kernel");

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
    info!(z = %params.initial_z(), variance = %params.initial_variance(), "initial state");

    let grid = QuadraticRoughHestonGrid::try_new(HORIZON, 100)?;
    let path = simulate_quadratic_rough_heston_path(&params, &grid, SEED)?;
    info!(
        terminal_spot = ?path.spot().last(),
        terminal_variance = ?path.variance().last(),
        "one path"
    );

    let paths = NonZeroUsize::try_from(100_000_usize)?;
    for (seed, steps) in [
        (SEED, 100),
        (OTHER_SEED, 100),
        (SEED, 200),
        (OTHER_SEED, 200),
    ] {
        let grid = QuadraticRoughHestonGrid::try_new(HORIZON, steps)?;
        for style in [OptionStyle::Call, OptionStyle::Put] {
            let estimate = price_vanilla_quadratic_rough_heston(
                &params,
                &grid,
                style,
                Positive::HUNDRED,
                paths,
                seed,
            )?;
            info!(
                seed,
                steps,
                style = ?style,
                price = %estimate.price(),
                standard_error = %estimate.standard_error(),
                mean_terminal_spot = %estimate.mean_terminal_spot(),
                terminal_spot_standard_error = %estimate.terminal_spot_standard_error(),
                non_positive_paths = estimate.non_positive_paths(),
                "estimate"
            );
        }
    }
    Ok(())
}
