/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 2026-04-27
******************************************************************************/

//! Black-76 Greeks example — commodity futures scenario.
//!
//! Computes delta, gamma, vega, theta, and rho for ATM, ITM, and OTM calls and
//! puts on a 6-month crude-oil futures contract. Prints the values together
//! with the Hull-style sanity identity `Δ_call − Δ_put = e^(-rT)` and shows
//! both free-function and trait-based usage.

use optionstratlib::error::greeks::GreeksError;
use optionstratlib::greeks::{Black76Greeks, delta_b76, gamma_b76, rho_b76, theta_b76, vega_b76};
use optionstratlib::model::types::{OptionStyle, OptionType, Side};
use optionstratlib::pricing::black_76;
use optionstratlib::{ExpirationDate, Options};
use positive::{Positive, pos_or_panic};
use rust_decimal::{Decimal, MathematicalOps};
use rust_decimal_macros::dec;
use tracing::info;

fn build_option(f: f64, k: f64, style: OptionStyle) -> Options {
    Options::new(
        OptionType::European,
        Side::Long,
        "CL".to_string(),
        pos_or_panic!(k),
        ExpirationDate::Days(pos_or_panic!(180.0)),
        pos_or_panic!(0.30),
        Positive::ONE,
        pos_or_panic!(f),
        dec!(0.045),
        style,
        pos_or_panic!(0.0),
        None,
    )
}

fn print_greeks(label: &str, opt: &Options) -> Result<(), Box<dyn std::error::Error>> {
    let price = black_76(opt)?;
    let delta = delta_b76(opt)?;
    let gamma = gamma_b76(opt)?;
    let vega = vega_b76(opt)?;
    let theta = theta_b76(opt)?;
    let rho = rho_b76(opt)?;
    info!(
        "{label:<6} F={F:>6} K={K:>6} | price={price:>9.4} Δ={delta:>8.4} Γ={gamma:>9.6} ν={vega:>8.4} Θ={theta:>9.4} ρ={rho:>9.4}",
        label = label,
        F = opt.underlying_price.to_dec(),
        K = opt.strike_price.to_dec(),
        price = price,
        delta = delta,
        gamma = gamma,
        vega = vega,
        theta = theta,
        rho = rho,
    );
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .without_time()
        .init();

    info!("=== Black-76 Greeks (6-month CL futures, σ=30%, r=4.5%) ===");
    info!("");
    info!("Calls");
    print_greeks("OTM", &build_option(70.0, 80.0, OptionStyle::Call))?;
    print_greeks("ATM", &build_option(75.0, 75.0, OptionStyle::Call))?;
    print_greeks("ITM", &build_option(80.0, 70.0, OptionStyle::Call))?;
    info!("");
    info!("Puts");
    print_greeks("ITM", &build_option(70.0, 80.0, OptionStyle::Put))?;
    print_greeks("ATM", &build_option(75.0, 75.0, OptionStyle::Put))?;
    print_greeks("OTM", &build_option(80.0, 70.0, OptionStyle::Put))?;

    // ---- Identity: Δ_call − Δ_put = e^(-rT) at any strike --------------
    info!("");
    let call = build_option(75.0, 75.0, OptionStyle::Call);
    let put = build_option(75.0, 75.0, OptionStyle::Put);
    let dc = delta_b76(&call)?;
    let dp = delta_b76(&put)?;
    let years = call.expiration_date.get_years()?.to_dec();
    let df = (-call.risk_free_rate * years).exp();
    let diff = dc - dp;
    info!(
        "Identity check: Δ_call − Δ_put = {diff:.10}, e^(-rT) = {df:.10}, |err| = {err:.2e}",
        diff = diff,
        df = df,
        err = (diff - df).abs(),
    );

    // ---- Trait usage ----------------------------------------------------
    struct FutureContract(Options);
    impl Black76Greeks for FutureContract {
        fn get_option(&self) -> Result<&Options, GreeksError> {
            Ok(&self.0)
        }
    }
    let contract = FutureContract(build_option(75.0, 75.0, OptionStyle::Call));
    info!("");
    info!(
        "Trait usage on wrapper type: Δ={delta} Γ={gamma}",
        delta = contract.delta_b76()?,
        gamma = contract.gamma_b76()?,
    );

    // Suppress unused-import warning on Decimal in case formatting changes.
    let _: Decimal = df;
    Ok(())
}
