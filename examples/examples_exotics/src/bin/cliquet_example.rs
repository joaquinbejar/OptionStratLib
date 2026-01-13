/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 13/01/26
******************************************************************************/

use optionstratlib::model::option::ExoticParams;
use optionstratlib::model::types::{OptionStyle, OptionType, Side};
use optionstratlib::pricing::cliquet::cliquet_black_scholes;
use optionstratlib::{ExpirationDate, Options};
use positive::pos_or_panic;
use rust_decimal_macros::dec;

fn main() {
    // 1. Define Cliquet Option parameters
    // We want a 1-year Cliquet option with quarterly resets (every 90 days)
    let reset_dates = vec![90.0, 180.0, 270.0];
    let expiration_days = 365.0;

    // Local cap of 5% and floor of -2% per period
    let cliquet_local_cap = Some(dec!(0.05));
    let cliquet_local_floor = Some(dec!(-0.02));

    // 2. Create the Options struct
    let option = Options::new(
        OptionType::Cliquet { reset_dates },
        Side::Long,
        "SPX".to_string(),
        pos_or_panic!(5000.0), // Note: Cliquet strike is reset, initial strike is often spot
        ExpirationDate::Days(pos_or_panic!(expiration_days)),
        pos_or_panic!(0.15), // 15% volatility
        pos_or_panic!(1.0),
        pos_or_panic!(5000.0), // Current price
        dec!(0.04),            // 4% risk-free rate
        OptionStyle::Call,
        pos_or_panic!(0.015), // 1.5% dividend yield
        Some(ExoticParams {
            spot_prices: None,
            spot_min: None,
            spot_max: None,
            cliquet_local_cap,
            cliquet_local_floor,
            cliquet_global_cap: None,
            cliquet_global_floor: None,
            rainbow_second_asset_price: None,
            rainbow_second_asset_volatility: None,
            rainbow_second_asset_dividend: None,
            rainbow_correlation: None,
            spread_second_asset_volatility: None,
            spread_second_asset_dividend: None,
            spread_correlation: None,
            quanto_fx_volatility: None,
            quanto_fx_correlation: None,
            quanto_foreign_rate: None,
            exchange_second_asset_volatility: None,
            exchange_second_asset_dividend: None,
            exchange_correlation: None,
        }),
    );

    // 3. Price the option
    match cliquet_black_scholes(&option) {
        Ok(price) => {
            println!("Cliquet Option Price: ${:.4}", price);
            println!("Number of periods: 4 (Inception to 90, 90-180, 180-270, 270-365)");
            println!("Local Cap: 5.00%");
            println!("Local Floor: -2.00%");
        }
        Err(e) => eprintln!("Error pricing Cliquet: {:?}", e),
    }

    // 4. Show price without cap/floor (unconstrained cliquet)
    let mut unconstrained = option.clone();
    if let Some(ref mut params) = unconstrained.exotic_params {
        params.cliquet_local_cap = Some(dec!(1.0)); // effectively no cap
        params.cliquet_local_floor = Some(dec!(-1.0)); // effectively no floor
    }

    match cliquet_black_scholes(&unconstrained) {
        Ok(price) => println!("Unconstrained Cliquet Price: ${:.4}", price),
        Err(e) => eprintln!("Error pricing unconstrained Cliquet: {:?}", e),
    }
}
