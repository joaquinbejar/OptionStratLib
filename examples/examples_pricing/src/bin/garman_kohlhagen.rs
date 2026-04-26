/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 2026-04-26
******************************************************************************/

//! Garman–Kohlhagen pricing example.
//!
//! Demonstrates pricing European FX options using the Garman–Kohlhagen
//! (1983) model: the Hull canonical example (USD/GBP, 4-month ATM), an
//! ITM EUR/USD scenario with FX put-call-parity check, dispatch through
//! the unified `PricingEngine`, and the symmetric-rate degenerate case.

use optionstratlib::model::types::{OptionStyle, OptionType, Side};
use optionstratlib::pricing::{PricingEngine, garman_kohlhagen, price_option};
use optionstratlib::{ExpirationDate, Options};
use positive::pos_or_panic;
use rust_decimal::MathematicalOps;
use rust_decimal_macros::dec;
use tracing::info;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .without_time()
        .init();

    info!("=== Garman-Kohlhagen FX Pricing Examples ===");

    // ---- Example 1: Hull canonical USD/GBP example ----------------------
    // S = K = 1.6 USD/GBP, r_d = 0.08, r_f = 0.11, sigma = 0.2, T = 4/12
    // -> call ~ 0.0639 (Hull, "Options, Futures and Other Derivatives").
    info!("");
    info!("Example 1: Hull canonical USD/GBP (ATM, 4-month)");
    let hull_t_days = 365.0 / 3.0;
    let option1 = Options::new(
        OptionType::European,
        Side::Long,
        "GBPUSD".to_string(),
        pos_or_panic!(1.6),
        ExpirationDate::Days(pos_or_panic!(hull_t_days)),
        pos_or_panic!(0.2),
        pos_or_panic!(1.0),
        pos_or_panic!(1.6),
        dec!(0.08),
        OptionStyle::Call,
        pos_or_panic!(0.11),
        None,
    );
    let call_price = garman_kohlhagen(&option1)?;
    let mut put1 = option1.clone();
    put1.option_style = OptionStyle::Put;
    let put_price = garman_kohlhagen(&put1)?;

    let years1 = option1.expiration_date.get_years()?.to_dec();
    let df_d = (-option1.risk_free_rate * years1).exp();
    let df_f = (-option1.dividend_yield.to_dec() * years1).exp();
    let parity_lhs = call_price - put_price;
    let parity_rhs =
        option1.underlying_price.to_dec() * df_f - option1.strike_price.to_dec() * df_d;

    info!("  S = 1.6, K = 1.6, r_d = 0.08, r_f = 0.11, T = 1/3 yr, sigma = 0.2");
    info!(
        "  Call price       = {} (Hull reference ~ 0.0639)",
        call_price
    );
    info!("  Put price        = {}", put_price);
    info!("  C - P            = {}", parity_lhs);
    info!("  S e^(-r_f T) - K e^(-r_d T) = {}", parity_rhs);

    // ---- Example 2: ITM EUR/USD call ------------------------------------
    info!("");
    info!("Example 2: ITM EUR/USD call (FX parity check)");
    let option2 = Options::new(
        OptionType::European,
        Side::Long,
        "EURUSD".to_string(),
        pos_or_panic!(1.20),
        ExpirationDate::Days(pos_or_panic!(180.0)),
        pos_or_panic!(0.10),
        pos_or_panic!(1.0),
        pos_or_panic!(1.25),
        dec!(0.045),
        OptionStyle::Call,
        pos_or_panic!(0.025),
        None,
    );
    let call2 = garman_kohlhagen(&option2)?;
    let mut put2 = option2.clone();
    put2.option_style = OptionStyle::Put;
    let put2_price = garman_kohlhagen(&put2)?;

    let years2 = option2.expiration_date.get_years()?.to_dec();
    let df_d2 = (-option2.risk_free_rate * years2).exp();
    let df_f2 = (-option2.dividend_yield.to_dec() * years2).exp();
    let parity2 = option2.underlying_price.to_dec() * df_f2 - option2.strike_price.to_dec() * df_d2;

    info!("  S = 1.25, K = 1.20, r_d = 4.5%, r_f = 2.5%, T = 180d, sigma = 0.10");
    info!("  Call price       = {}", call2);
    info!("  Put price        = {}", put2_price);
    info!("  C - P            = {}", call2 - put2_price);
    info!("  Expected parity  = {}", parity2);

    // ---- Example 3: Unified API dispatch --------------------------------
    info!("");
    info!("Example 3: Unified API via PricingEngine::ClosedFormGK");
    let direct = garman_kohlhagen(&option2)?;
    let via_engine = price_option(&option2, &PricingEngine::ClosedFormGK)?;
    info!("  Direct garman_kohlhagen()   = {}", direct);
    info!("  Via PricingEngine dispatch  = {}", via_engine);

    // ---- Example 4: Symmetric rates collapse to forward parity ----------
    info!("");
    info!("Example 4: Symmetric rates (r_d = r_f) collapse to forward parity");
    let option4 = Options::new(
        OptionType::European,
        Side::Long,
        "USDUSD".to_string(),
        pos_or_panic!(1.20),
        ExpirationDate::Days(pos_or_panic!(180.0)),
        pos_or_panic!(0.10),
        pos_or_panic!(1.0),
        pos_or_panic!(1.25),
        dec!(0.04),
        OptionStyle::Call,
        pos_or_panic!(0.04),
        None,
    );
    let call4 = garman_kohlhagen(&option4)?;
    let mut put4 = option4.clone();
    put4.option_style = OptionStyle::Put;
    let put4_price = garman_kohlhagen(&put4)?;
    let years4 = option4.expiration_date.get_years()?.to_dec();
    let df = (-option4.risk_free_rate * years4).exp();
    let collapsed = df * (option4.underlying_price.to_dec() - option4.strike_price.to_dec());
    info!("  C - P                = {}", call4 - put4_price);
    info!("  e^(-r T) (S - K)     = {}", collapsed);

    Ok(())
}
