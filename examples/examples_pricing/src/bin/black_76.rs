/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 2026-04-26
******************************************************************************/

//! Black-76 pricing example.
//!
//! Demonstrates pricing options on futures and forwards using the Black-76
//! model, including the Hull canonical example, an in-the-money commodity
//! futures call, dispatch through the unified `PricingEngine`, and the
//! short-side sign convention.

use optionstratlib::model::types::{OptionStyle, OptionType, Side};
use optionstratlib::pricing::{PricingEngine, black_76, price_option};
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

    info!("=== Black-76 Pricing Examples ===");

    // ---- Example 1: Hull canonical reference (ATM, 4-month future) -------
    // F = K = 20, r = 0.09, T = 4/12, sigma = 0.25 -> call ≈ 1.1166.
    info!("");
    info!("Example 1: Hull canonical example (ATM future)");
    let hull_t_days = 365.0 / 3.0;
    let option1 = Options::new(
        OptionType::European,
        Side::Long,
        "HULL".to_string(),
        pos_or_panic!(20.0),
        ExpirationDate::Days(pos_or_panic!(hull_t_days)),
        pos_or_panic!(0.25),
        pos_or_panic!(1.0),
        pos_or_panic!(20.0),
        dec!(0.09),
        OptionStyle::Call,
        pos_or_panic!(0.0),
        None,
    );

    let call_price = black_76(&option1)?;
    let mut put_option1 = option1.clone();
    put_option1.option_style = OptionStyle::Put;
    let put_price = black_76(&put_option1)?;
    let years1 = option1.expiration_date.get_years()?.to_dec();
    let df1 = (-option1.risk_free_rate * years1).exp();
    let parity_diff1 = call_price - put_price;
    let expected_parity1 = df1 * (dec!(20) - dec!(20));

    info!("  F = 20, K = 20, r = 0.09, T = 1/3 yr, sigma = 0.25");
    info!(
        "  Call price       = {} (Hull reference ≈ 1.1166)",
        call_price
    );
    info!("  Put price        = {}", put_price);
    info!("  C - P            = {}", parity_diff1);
    info!("  e^(-rT)(F - K)   = {}", expected_parity1);

    // ---- Example 2: In-the-money commodity futures call ------------------
    info!("");
    info!("Example 2: In-the-money commodity futures call");
    let option2 = Options::new(
        OptionType::European,
        Side::Long,
        "ES".to_string(),
        pos_or_panic!(4000.0),
        ExpirationDate::Days(pos_or_panic!(90.0)),
        pos_or_panic!(0.15),
        pos_or_panic!(1.0),
        pos_or_panic!(4100.0),
        dec!(0.05),
        OptionStyle::Call,
        pos_or_panic!(0.0),
        None,
    );

    let call2 = black_76(&option2)?;
    let mut put_option2 = option2.clone();
    put_option2.option_style = OptionStyle::Put;
    let put2 = black_76(&put_option2)?;
    let years2 = option2.expiration_date.get_years()?.to_dec();
    let df2 = (-option2.risk_free_rate * years2).exp();
    let parity_diff2 = call2 - put2;
    let expected_parity2 = df2 * (dec!(4100) - dec!(4000));

    info!("  F = 4100, K = 4000, r = 0.05, T = 90d, sigma = 0.15");
    info!("  Call price       = {}", call2);
    info!("  Put price        = {}", put2);
    info!("  C - P            = {}", parity_diff2);
    info!("  e^(-rT)(F - K)   = {}", expected_parity2);

    // ---- Example 3: Unified PricingEngine dispatch -----------------------
    info!("");
    info!("Example 3: Unified API via PricingEngine::ClosedFormBlack76");
    let option3 = Options::new(
        OptionType::European,
        Side::Long,
        "CL".to_string(),
        pos_or_panic!(80.0),
        ExpirationDate::Days(pos_or_panic!(60.0)),
        pos_or_panic!(0.2),
        pos_or_panic!(1.0),
        pos_or_panic!(85.0),
        dec!(0.04),
        OptionStyle::Call,
        pos_or_panic!(0.0),
        None,
    );
    let price_direct = black_76(&option3)?;
    let price_via_engine = price_option(&option3, &PricingEngine::ClosedFormBlack76)?;
    info!("  Direct black_76()         = {}", price_direct);
    info!("  Via PricingEngine dispatch = {}", price_via_engine);

    // ---- Example 4: Short side sign convention ---------------------------
    info!("");
    info!("Example 4: Short position negates the long price");
    let mut option4 = option2.clone();
    option4.side = Side::Short;
    let short_call = black_76(&option4)?;
    info!("  Long call         = {}", call2);
    info!("  Short call        = {}", short_call);
    info!("  Short == -Long    = {}", short_call == -call2);

    Ok(())
}
