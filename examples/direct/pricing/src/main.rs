//! Pricing with the component crates and no facade (#555).
//!
//! `optionstratlib-core` defines the contract (`Options`) and
//! `optionstratlib-pricing` prices it: closed-form Black-Scholes, the Greeks
//! as methods on the same `Options`, and the implied volatility that
//! reproduces a price. Nothing else is compiled: no option chain, strategy,
//! simulation, chart or I/O code. The figures are Hull's example: S = 42,
//! K = 40, r = 10%, sigma = 20%, T = 0.5 years.

use optionstratlib_core::model::{
    ExpirationDate, OptionStyle, OptionType, Options, Positive, Side,
};
use optionstratlib_pricing::greeks::Greeks;
use optionstratlib_pricing::pricing::{OptionPricing, black_scholes};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::error::Error;
use tracing::info;

/// What the workflow reads off one option.
#[derive(Debug, Clone, PartialEq)]
struct PricingReport {
    price: Decimal,
    delta: Decimal,
    gamma: Decimal,
    implied_volatility: Positive,
}

/// Hull's European option on a non-dividend-paying stock.
fn hull_option(style: OptionStyle) -> Result<Options, Box<dyn Error>> {
    Ok(Options::new(
        OptionType::European,
        Side::Long,
        "XYZ".to_string(),
        Positive::new(40.0)?,
        ExpirationDate::Days(Positive::new(182.5)?),
        Positive::new(0.2)?,
        Positive::ONE,
        Positive::new(42.0)?,
        dec!(0.10),
        style,
        Positive::ZERO,
        None,
    ))
}

fn report(style: OptionStyle) -> Result<PricingReport, Box<dyn Error>> {
    let option = hull_option(style)?;
    let price = black_scholes(&option)?;
    Ok(PricingReport {
        price,
        delta: option.delta()?,
        gamma: option.gamma()?,
        implied_volatility: option.calculate_implied_volatility(price)?,
    })
}

fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt().init();
    for style in [OptionStyle::Call, OptionStyle::Put] {
        let report = report(style)?;
        info!(
            "{style:?}: price {:.4}, delta {:.4}, gamma {:.4}, implied volatility {}",
            report.price, report.delta, report.gamma, report.implied_volatility
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hull_prices_and_the_volatility_round_trip() -> Result<(), Box<dyn Error>> {
        let call = report(OptionStyle::Call)?;
        let put = report(OptionStyle::Put)?;
        assert!((call.price - dec!(4.76)).abs() <= dec!(0.005));
        assert!((put.price - dec!(0.81)).abs() <= dec!(0.005));
        // Put-call parity on the deltas: delta_call - delta_put = 1.
        assert!((call.delta - put.delta - Decimal::ONE).abs() <= dec!(0.000001));
        assert!((call.implied_volatility.to_dec() - dec!(0.2)).abs() <= dec!(0.001));
        Ok(())
    }
}
