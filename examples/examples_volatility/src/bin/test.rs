/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 5/5/25
******************************************************************************/
use optionstratlib::prelude::*;
use rand::{Rng, rng};
use rust_decimal::MathematicalOps;
use rust_decimal::prelude::FromPrimitive;

#[allow(clippy::too_many_arguments)]
fn calculate_error(
    days: Positive,
    risk_free_rate: Decimal,
    dividend_yield: Positive,
    ask: Positive,
    bid: Positive,
    option_style: OptionStyle,
    strike_price: Positive,
    implied_volatility: Positive,
    underlying_price: Positive,
) -> Result<Decimal, optionstratlib::error::Error> {
    let mid_price = (bid + ask) / pos!(2.0);
    let option = Options {
        option_type: OptionType::European,
        side: Side::Long,
        underlying_symbol: "".to_string(),
        strike_price,
        expiration_date: ExpirationDate::Days(days),
        implied_volatility,
        quantity: Positive::ONE,
        underlying_price,
        risk_free_rate,
        option_style,
        dividend_yield,
        exotic_params: None,
    };
    let price = black_scholes(&option)?;

    // info!("{} mid: {}", price.round_dp(1), mid_price);
    Ok((price - mid_price).powu(2))
}

#[derive(Debug)]
pub struct BestParams {
    pub risk_free_rate: Decimal,
    pub dividend_yield: Positive,
    pub days: Positive,
}

fn main() -> Result<(), optionstratlib::error::Error> {
    // let risk_free_rate = dec!(0.0214);
    // let dividend_yield = pos!(0.0225);
    // let days = pos!(75.0);
    let mut thread_rng = rng();
    let mut best_params = BestParams {
        risk_free_rate: Default::default(),
        dividend_yield: Default::default(),
        days: Default::default(),
    };
    let mut best_error = Decimal::MAX;
    let mut best_error1 = Decimal::MAX;
    let mut best_error2 = Decimal::MAX;
    let mut best_error3 = Decimal::MAX;
    let mut best_error4 = Decimal::MAX;

    for _ in 0..1000000 {
        let days_f64: f64 = thread_rng.random_range(74.34..=74.46);
        let days = pos!(days_f64);

        let risk_free_rate_f64: f64 = thread_rng.random_range(0.0211..=0.0218);
        let risk_free_rate = Decimal::from_f64(risk_free_rate_f64).unwrap();

        let dividend_yield_f64: f64 = thread_rng.random_range(0.021..=0.0218);
        let dividend_yield = pos!(dividend_yield_f64);

        let error1 = calculate_error(
            days,
            risk_free_rate,
            dividend_yield,
            pos!(1590.6),
            pos!(1602.6),
            OptionStyle::Call,
            pos!(22000.0),
            pos!(0.21657),
            pos!(23196.0),
        )?;

        let error2 = calculate_error(
            days,
            risk_free_rate,
            dividend_yield,
            pos!(1005.4),
            pos!(1017.4),
            OptionStyle::Call,
            pos!(22800.0),
            pos!(0.19404),
            pos!(23196.0),
        )?;

        let error3 = calculate_error(
            days,
            risk_free_rate,
            dividend_yield,
            pos!(679.6),
            pos!(691.6),
            OptionStyle::Put,
            pos!(23000.0),
            pos!(0.18844),
            pos!(23196.0),
        )?;

        let error4 = calculate_error(
            days,
            risk_free_rate,
            dividend_yield,
            pos!(400.4),
            pos!(412.4),
            OptionStyle::Put,
            pos!(22000.0),
            pos!(0.21657),
            pos!(23196.0),
        )?;

        let total_error = error1 + error2 + error3 + error4;
        if total_error < best_error
            && error1 <= best_error1
            && error2 <= best_error2
            && error3 <= best_error3
            && error4 <= best_error4
        {
            best_params = BestParams {
                risk_free_rate,
                dividend_yield,
                days,
            };
            print!(
                "\rFound new best params: {:?} with error: {:.2}",
                best_params, best_error
            );
            best_error = total_error;
            best_error1 = error1;
            best_error2 = error2;
            best_error3 = error3;
            best_error4 = error4;
        }

        // info!("Total error: {}", total_error.round_dp(1));
    }
    info!("");
    info!(
        "Best params: {:?} with best error: {:.2}",
        best_params, best_error
    );
    info!(
        "Best Error 1: {:.2} Best Error 2: {:.2} Best Error 3: {:.2} Best Error 4: {:.2}",
        best_error1, best_error2, best_error3, best_error4
    );
    Ok(())
}
