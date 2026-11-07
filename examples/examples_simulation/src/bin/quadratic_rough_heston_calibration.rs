/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 7/11/26
******************************************************************************/

//! # Calibrating the quadratic rough Heston model
//!
//! Generates an implied volatility surface from the model at known parameters, then fits the
//! model to it from a start away from them, with the initial factors fixed, and prints the
//! fitted parameters, the `V_0` they imply and the error per quote. The fit runs on a fixed
//! Monte Carlo grid and seed, so it recovers the parameters that generated the surface on that
//! grid; a second fit at another seed shows how far Monte Carlo noise moves them.
//! A last fit adds the model's VIX future and calls at two maturities and fits both legs.
//!
//! **Every parameter here is illustrative.** None of them is calibrated to any underlying,
//! and the papers' values are for SPX and VIX.

use optionstratlib::prelude::*;
use optionstratlib::simulation::{
    QuadraticRoughHestonCalibrationConfig, QuadraticRoughHestonFreeParameters,
    QuadraticRoughHestonGrid, QuadraticRoughHestonKernel, QuadraticRoughHestonMonteCarloConfig,
    QuadraticRoughHestonParams, QuadraticRoughHestonQuote, QuadraticRoughHestonSurfacePoint,
    QuadraticRoughHestonVix, QuadraticRoughHestonVixPayoff, QuadraticRoughHestonVixQuote,
    calibrate_quadratic_rough_heston, calibrate_quadratic_rough_heston_joint,
    price_surface_quadratic_rough_heston, price_vix_quadratic_rough_heston,
};
use osl_example_support::setup_logger;
use rust_decimal_macros::dec;
use std::num::NonZeroUsize;
use tracing::info;

/// The seed of the surface and of the first fit.
const SEED: u64 = 967;

/// The seed of the second fit.
const OTHER_SEED: u64 = 968;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_logger();

    // Illustrative values: `alpha = 0.6`, 2 factors, ratio 5.
    let kernel = QuadraticRoughHestonKernel::try_new(dec!(0.6), 2, dec!(5))?;
    // The parameters the surface is generated from.
    let truth = QuadraticRoughHestonParams::try_new(
        dec!(0.4),
        dec!(0.1),
        dec!(0.02),
        dec!(1.5),
        dec!(0.8),
        kernel.clone(),
        vec![dec!(-0.1), dec!(-0.2)],
        Positive::HUNDRED,
    )?;
    info!(v0 = %truth.initial_variance(), "parameters of the surface");

    // Three maturities and five strikes, priced on 4,000 paths with steps of at most 0.01 years.
    let mut points = Vec::new();
    for maturity in [dec!(0.1), dec!(0.25), dec!(0.5)] {
        for strike in [dec!(85), dec!(95), dec!(100), dec!(105), dec!(115)] {
            points.push(QuadraticRoughHestonSurfacePoint {
                maturity: Positive::new_decimal(maturity)?,
                strike: Positive::new_decimal(strike)?,
            });
        }
    }
    let monte_carlo = QuadraticRoughHestonMonteCarloConfig {
        paths: NonZeroUsize::try_from(4_000_usize)?,
        max_step: Positive::new_decimal(dec!(0.01))?,
        seed: SEED,
    };
    let surface = price_surface_quadratic_rough_heston(&truth, &points, &monte_carlo)?;
    let mut quotes = Vec::new();
    for point in &surface {
        info!(
            maturity = %point.maturity,
            strike = %point.strike,
            style = ?point.style,
            price = %point.price,
            implied_volatility = ?point.implied_volatility,
            "surface"
        );
        if let Some(implied_volatility) = point.implied_volatility {
            quotes.push(QuadraticRoughHestonQuote {
                maturity: point.maturity,
                strike: point.strike,
                implied_volatility,
                weight: Positive::ONE,
            });
        }
    }

    // A start 30% to 50% away on each model parameter, with the initial factors fixed.
    let start = QuadraticRoughHestonParams::try_new(
        dec!(0.6),
        dec!(0.05),
        dec!(0.012),
        dec!(1),
        dec!(1.1),
        kernel,
        vec![dec!(-0.1), dec!(-0.2)],
        Positive::HUNDRED,
    )?;
    for seed in [SEED, OTHER_SEED] {
        let config = QuadraticRoughHestonCalibrationConfig {
            monte_carlo: QuadraticRoughHestonMonteCarloConfig {
                seed,
                ..monte_carlo
            },
            ..Default::default()
        };
        let fit = calibrate_quadratic_rough_heston(
            &quotes,
            &start,
            QuadraticRoughHestonFreeParameters::MODEL,
            &config,
        )?;
        let params = fit.params();
        info!(
            seed,
            a = %params.a(),
            b = %params.b(),
            c = %params.c(),
            lambda = %params.lambda(),
            eta = %params.eta(),
            v0 = %params.initial_variance(),
            rmse = %fit.rmse(),
            iterations = fit.iterations(),
            evaluations = fit.evaluations(),
            paths = fit.paths(),
            grid = ?fit.grid(),
            "fit"
        );
        for quote in fit.quotes() {
            info!(
                seed,
                maturity = %quote.quote.maturity,
                strike = %quote.quote.strike,
                market_iv = %quote.quote.implied_volatility,
                model_iv = ?quote.model_implied_volatility,
                residual = %quote.residual,
                iv_error = ?quote.implied_volatility_error,
                "quote"
            );
        }
    }
    // The joint fit: the VIX future and three calls at two maturities, priced by the model on
    // single-maturity grids of the same 0.01 step, which draw the paths of the fit's grid.
    let window = QuadraticRoughHestonVix::thirty_days(&truth)?.window();
    let mut vix_quotes = Vec::new();
    for (maturity, steps) in [(dec!(0.25), 25), (dec!(0.5), 50)] {
        let grid = QuadraticRoughHestonGrid::try_new(maturity, steps)?;
        let mut payoffs = vec![QuadraticRoughHestonVixPayoff::Future];
        for strike in [dec!(12), dec!(16), dec!(20)] {
            payoffs.push(QuadraticRoughHestonVixPayoff::Vanilla {
                style: OptionStyle::Call,
                strike: Positive::new_decimal(strike)?,
            });
        }
        for payoff in payoffs {
            let estimate = price_vix_quadratic_rough_heston(
                &truth,
                &grid,
                window,
                payoff,
                monte_carlo.paths,
                SEED,
            )?;
            vix_quotes.push(QuadraticRoughHestonVixQuote {
                maturity: Positive::new_decimal(maturity)?,
                payoff,
                price: Positive::new_decimal(estimate.price())?,
                weight: Positive::ONE,
            });
        }
    }
    let config = QuadraticRoughHestonCalibrationConfig {
        monte_carlo,
        ..Default::default()
    };
    let fit = calibrate_quadratic_rough_heston_joint(
        &quotes,
        &vix_quotes,
        &start,
        QuadraticRoughHestonFreeParameters::MODEL,
        &config,
    )?;
    let params = fit.params();
    info!(
        a = %params.a(),
        b = %params.b(),
        c = %params.c(),
        lambda = %params.lambda(),
        eta = %params.eta(),
        rmse = %fit.rmse(),
        spx_rmse = ?fit.spx_rmse(),
        vix_rmse = ?fit.vix_rmse(),
        iterations = fit.iterations(),
        "joint fit"
    );
    for quote in fit.vix_quotes() {
        info!(
            maturity = %quote.quote.maturity,
            payoff = ?quote.quote.payoff,
            market = %quote.quote.price,
            model = %quote.model_price,
            residual = %quote.residual,
            "vix quote"
        );
    }
    Ok(())
}
