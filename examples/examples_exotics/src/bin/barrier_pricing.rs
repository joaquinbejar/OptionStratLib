/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 12/01/26
******************************************************************************/

use optionstratlib::greeks::Greeks;
use optionstratlib::model::types::{BarrierType, OptionStyle, OptionType, Side};
use optionstratlib::pricing::unified::{Priceable, PricingEngine};
use optionstratlib::{ExpirationDate, Options};
use positive::pos_or_panic;
use prettytable::{Table, row};
use rust_decimal_macros::dec;

fn main() {
    tracing_subscriber::fmt::init();

    let underlying_price = 100.0;
    let strike_price = 100.0;
    let volatility = 0.25;
    let _risk_free_rate = 0.08;
    let dividend_yield = 0.04;
    let time_to_expiration = 0.5;
    let barrier_level = 95.0;

    let mut table = Table::new();
    table.add_row(row![
        "Barrier Type",
        "Style",
        "Price",
        "Delta",
        "Gamma",
        "Vega",
        "Rho"
    ]);

    let barrier_types = vec![
        BarrierType::DownAndIn,
        BarrierType::DownAndOut,
        BarrierType::UpAndIn,
        BarrierType::UpAndOut,
    ];

    let styles = vec![OptionStyle::Call, OptionStyle::Put];

    for barrier_type in barrier_types {
        for style in &styles {
            let option = Options {
                option_type: OptionType::Barrier {
                    barrier_type,
                    barrier_level,
                    rebate: None,
                },
                side: Side::Long,
                underlying_symbol: "XYZ".to_string(),
                strike_price: pos_or_panic!(strike_price),
                expiration_date: ExpirationDate::Days(pos_or_panic!(time_to_expiration * 365.0)),
                implied_volatility: pos_or_panic!(volatility),
                quantity: pos_or_panic!(1.0),
                underlying_price: pos_or_panic!(underlying_price),
                risk_free_rate: dec!(0.08),
                option_style: *style,
                dividend_yield: pos_or_panic!(dividend_yield),
                exotic_params: None,
            };

            let price = option.price(&PricingEngine::ClosedFormBS).unwrap();
            let delta = option.delta().unwrap();
            let gamma = option.gamma().unwrap();
            let vega = option.vega().unwrap();
            let rho = option.rho().unwrap();

            table.add_row(row![
                format!("{:?}", barrier_type),
                format!("{:?}", style),
                format!("{:.4}", price),
                format!("{:.4}", delta),
                format!("{:.4}", gamma),
                format!("{:.4}", vega),
                format!("{:.4}", rho),
            ]);
        }
    }

    table.printstd();
}
