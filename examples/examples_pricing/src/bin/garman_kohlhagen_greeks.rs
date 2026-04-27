/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 2026-04-27
******************************************************************************/

//! Garman–Kohlhagen Greeks example — EUR/USD scenario.
//!
//! Computes spot delta, gamma, vega, theta, domestic rho, and foreign rho
//! for ATM, ITM, and OTM calls and puts on a 6-month EUR/USD option, then
//! demonstrates spot delta-parity (`Δ_call − Δ_put = e^(-r_f·T)`) and the
//! `GarmanKohlhagenGreeks` trait.

use optionstratlib::error::greeks::GreeksError;
use optionstratlib::greeks::{
    GarmanKohlhagenGreeks, delta_gk, gamma_gk, rho_domestic_gk, rho_foreign_gk, theta_gk, vega_gk,
};
use optionstratlib::model::types::{OptionStyle, OptionType, Side};
use optionstratlib::pricing::garman_kohlhagen;
use optionstratlib::{ExpirationDate, Options};
use positive::{Positive, pos_or_panic};
use rust_decimal::MathematicalOps;
use rust_decimal_macros::dec;
use tracing::info;

fn build_option(s: f64, k: f64, style: OptionStyle) -> Options {
    Options::new(
        OptionType::European,
        Side::Long,
        "EURUSD".to_string(),
        pos_or_panic!(k),
        ExpirationDate::Days(pos_or_panic!(180.0)),
        pos_or_panic!(0.10),
        Positive::ONE,
        pos_or_panic!(s),
        dec!(0.045),
        style,
        pos_or_panic!(0.025),
        None,
    )
}

fn print_greeks(label: &str, opt: &Options) -> Result<(), Box<dyn std::error::Error>> {
    let price = garman_kohlhagen(opt)?;
    let delta = delta_gk(opt)?;
    let gamma = gamma_gk(opt)?;
    let vega = vega_gk(opt)?;
    let theta = theta_gk(opt)?;
    let rho_d = rho_domestic_gk(opt)?;
    let rho_f = rho_foreign_gk(opt)?;
    info!(
        "{label:<6} S={s:>6} K={k:>6} | price={price:>9.6} Δ={delta:>9.6} Γ={gamma:>9.4} ν={vega:>9.6} Θ={theta:>10.6} ρd={rho_d:>9.6} ρf={rho_f:>9.6}",
        label = label,
        s = opt.underlying_price.to_dec(),
        k = opt.strike_price.to_dec(),
        price = price,
        delta = delta,
        gamma = gamma,
        vega = vega,
        theta = theta,
        rho_d = rho_d,
        rho_f = rho_f,
    );
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .without_time()
        .init();

    info!("=== Garman–Kohlhagen Greeks (6-month EUR/USD, σ=10%, r_d=4.5%, r_f=2.5%) ===");
    info!("");
    info!("Calls");
    print_greeks("OTM", &build_option(1.05, 1.10, OptionStyle::Call))?;
    print_greeks("ATM", &build_option(1.10, 1.10, OptionStyle::Call))?;
    print_greeks("ITM", &build_option(1.15, 1.10, OptionStyle::Call))?;
    info!("");
    info!("Puts");
    print_greeks("ITM", &build_option(1.05, 1.10, OptionStyle::Put))?;
    print_greeks("ATM", &build_option(1.10, 1.10, OptionStyle::Put))?;
    print_greeks("OTM", &build_option(1.15, 1.10, OptionStyle::Put))?;

    // ---- Spot delta-parity --------------------------------------------
    let call = build_option(1.10, 1.10, OptionStyle::Call);
    let put = build_option(1.10, 1.10, OptionStyle::Put);
    let dc = delta_gk(&call)?;
    let dp = delta_gk(&put)?;
    let years = call.expiration_date.get_years()?.to_dec();
    let df_rf = (-call.dividend_yield.to_dec() * years).exp();
    let diff = dc - dp;
    info!("");
    info!(
        "Spot delta-parity: Δ_call − Δ_put = {diff:.10}, e^(-r_f·T) = {df:.10}, |err| = {err:.2e}",
        diff = diff,
        df = df_rf,
        err = (diff - df_rf).abs(),
    );

    // ---- Trait usage --------------------------------------------------
    struct FxQuote(Options);
    impl GarmanKohlhagenGreeks for FxQuote {
        fn get_option(&self) -> Result<&Options, GreeksError> {
            Ok(&self.0)
        }
    }
    let q = FxQuote(build_option(1.10, 1.10, OptionStyle::Call));
    info!("");
    info!(
        "Trait usage: Δ={delta} ρd={rho_d} ρf={rho_f}",
        delta = q.delta_gk()?,
        rho_d = q.rho_domestic_gk()?,
        rho_f = q.rho_foreign_gk()?,
    );

    Ok(())
}
