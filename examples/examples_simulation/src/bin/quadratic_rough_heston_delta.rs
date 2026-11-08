/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 8/11/26
******************************************************************************/

//! # Delta along the Brownian under the quadratic rough Heston model
//!
//! Computes the delta of a vanilla, a VIX future and a VIX call at three bump sizes. Compare
//! the bumps, the seeds and the step counts before trusting a value.
//!
//! **Every parameter here is illustrative.** None of them is calibrated to any underlying.

use optionstratlib::prelude::*;
use optionstratlib::simulation::{
    QuadraticRoughHestonGrid, QuadraticRoughHestonInstrument, QuadraticRoughHestonKernel,
    QuadraticRoughHestonParams, QuadraticRoughHestonVix, QuadraticRoughHestonVixPayoff,
    brownian_delta_quadratic_rough_heston,
};
use osl_example_support::setup_logger;
use rust_decimal_macros::dec;
use std::num::NonZeroUsize;
use tracing::info;

const SEED: u64 = 974;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_logger();

    let kernel = QuadraticRoughHestonKernel::try_new(dec!(0.6), 5, dec!(5))?;
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
    let window = QuadraticRoughHestonVix::thirty_days(&params)?.window();
    let grid = QuadraticRoughHestonGrid::try_new(dec!(0.25), 100)?;
    let paths = NonZeroUsize::try_from(50_000_usize)?;
    let strike = Positive::new_decimal(dec!(15))?;

    let instruments = [
        (
            "spot call",
            QuadraticRoughHestonInstrument::Vanilla {
                style: OptionStyle::Call,
                strike: Positive::HUNDRED,
            },
        ),
        (
            "vix future",
            QuadraticRoughHestonInstrument::Vix {
                window,
                payoff: QuadraticRoughHestonVixPayoff::Future,
            },
        ),
        (
            "vix call",
            QuadraticRoughHestonInstrument::Vix {
                window,
                payoff: QuadraticRoughHestonVixPayoff::Vanilla {
                    style: OptionStyle::Call,
                    strike,
                },
            },
        ),
    ];
    for (name, instrument) in instruments {
        for bump in [dec!(0.04), dec!(0.02), dec!(0.01)] {
            let estimate = brownian_delta_quadratic_rough_heston(
                &params,
                &grid,
                instrument,
                Positive::new_decimal(bump)?,
                paths,
                SEED,
            )?;
            info!(
                name,
                bump = %bump,
                delta = %estimate.delta(),
                standard_error = %estimate.standard_error(),
                "delta along the brownian"
            );
        }
    }
    Ok(())
}
