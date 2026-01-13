/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 12/01/26
******************************************************************************/

//! Binary option pricing module.
//!
//! Binary options (also called digital options) have a fixed payout if the
//! option expires in-the-money, regardless of how far in-the-money it is.
//!
//! # Variants
//!
//! - **Cash-or-Nothing**: Pays a fixed cash amount Q if the option is ITM
//! - **Asset-or-Nothing**: Pays the asset value S if the option is ITM
//! - **Gap**: Pays the difference between asset price and a "gap" strike
//!
//! # Formulas
//!
//! **Cash-or-Nothing Call**: `C = Q * e^(-rT) * N(d2)`
//! **Cash-or-Nothing Put**: `P = Q * e^(-rT) * N(-d2)`
//!
//! **Asset-or-Nothing Call**: `C = S * e^(-qT) * N(d1)`
//! **Asset-or-Nothing Put**: `P = S * e^(-qT) * N(-d1)`
//!
//! # Greeks Note
//!
//! Binary options have discontinuous Delta at the strike price.
//! Gamma can be extremely large near expiration when near the strike.

use crate::Options;
use crate::error::PricingError;
use crate::greeks::{big_n, d1, d2};
use crate::model::types::{BinaryType, OptionStyle, OptionType};
use positive::Positive;
use rust_decimal::Decimal;
use rust_decimal::prelude::*;
use rust_decimal_macros::dec;

/// Default cash payout for cash-or-nothing options (when not specified).
const DEFAULT_CASH_PAYOUT: Decimal = dec!(1.0);

/// Prices a Binary option using appropriate closed-form formula.
///
/// # Arguments
///
/// * `option` - The option to price. Must have `OptionType::Binary`.
///
/// # Returns
///
/// The option price as a `Decimal`, or a `PricingError` if pricing fails.
pub fn binary_black_scholes(option: &Options) -> Result<Decimal, PricingError> {
    match &option.option_type {
        OptionType::Binary { binary_type } => match binary_type {
            BinaryType::CashOrNothing => cash_or_nothing_price(option, DEFAULT_CASH_PAYOUT),
            BinaryType::AssetOrNothing => asset_or_nothing_price(option),
            BinaryType::Gap => gap_binary_price(option),
        },
        _ => Err(PricingError::other(
            "binary_black_scholes requires OptionType::Binary",
        )),
    }
}

/// Prices a cash-or-nothing binary option.
///
/// **Call**: Pays Q if S_T > K at expiration
/// **Put**: Pays Q if S_T < K at expiration
///
/// # Formula
/// - Call: `C = Q * e^(-rT) * N(d2)`
/// - Put: `P = Q * e^(-rT) * N(-d2)`
fn cash_or_nothing_price(option: &Options, payout: Decimal) -> Result<Decimal, PricingError> {
    let s = option.underlying_price;
    let k = option.strike_price;
    let r = option.risk_free_rate;
    let q = option.dividend_yield.to_dec();
    let sigma = option.implied_volatility;
    let t = option
        .expiration_date
        .get_years()
        .map_err(|e| PricingError::other(&e.to_string()))?;

    if t == Positive::ZERO {
        return Ok(intrinsic_cash_or_nothing(option, payout));
    }

    if sigma == Positive::ZERO {
        // At zero vol, price is deterministic
        let forward = s * ((r - q) * t).exp();
        let itm = match option.option_style {
            OptionStyle::Call => forward > k,
            OptionStyle::Put => k > forward,
        };
        let discount = (-r * t).exp();
        let value = if itm {
            payout * discount
        } else {
            Decimal::ZERO
        };
        return Ok(apply_side(value, option));
    }

    // Calculate d2 for cash-or-nothing
    let b = r - q;
    let d2_val = d2(s, k, b, t, sigma)
        .map_err(|e: crate::error::GreeksError| PricingError::other(&e.to_string()))?;

    let discount = (-r * t).exp();

    let price = match option.option_style {
        OptionStyle::Call => {
            let n_d2 = big_n(d2_val).unwrap_or(Decimal::ZERO);
            payout * discount * n_d2
        }
        OptionStyle::Put => {
            let n_neg_d2 = big_n(-d2_val).unwrap_or(Decimal::ZERO);
            payout * discount * n_neg_d2
        }
    };

    Ok(apply_side(price, option))
}

/// Prices an asset-or-nothing binary option.
///
/// **Call**: Pays S_T if S_T > K at expiration
/// **Put**: Pays S_T if S_T < K at expiration
///
/// # Formula
/// - Call: `C = S * e^(-qT) * N(d1)`
/// - Put: `P = S * e^(-qT) * N(-d1)`
fn asset_or_nothing_price(option: &Options) -> Result<Decimal, PricingError> {
    let s = option.underlying_price;
    let k = option.strike_price;
    let r = option.risk_free_rate;
    let q = option.dividend_yield.to_dec();
    let sigma = option.implied_volatility;
    let t = option
        .expiration_date
        .get_years()
        .map_err(|e| PricingError::other(&e.to_string()))?;

    if t == Positive::ZERO {
        return Ok(intrinsic_asset_or_nothing(option));
    }

    if sigma == Positive::ZERO {
        let forward = s * ((r - q) * t).exp();
        let itm = match option.option_style {
            OptionStyle::Call => forward > k,
            OptionStyle::Put => k > forward,
        };
        let discount = (-q * t).exp();
        let value = if itm {
            s.to_dec() * discount
        } else {
            Decimal::ZERO
        };
        return Ok(apply_side(value, option));
    }

    let b = r - q;
    let d1_val = d1(s, k, b, t, sigma)
        .map_err(|e: crate::error::GreeksError| PricingError::other(&e.to_string()))?;

    let dividend_discount = (-q * t).exp();

    let price = match option.option_style {
        OptionStyle::Call => {
            let n_d1 = big_n(d1_val).unwrap_or(Decimal::ZERO);
            s.to_dec() * dividend_discount * n_d1
        }
        OptionStyle::Put => {
            let n_neg_d1 = big_n(-d1_val).unwrap_or(Decimal::ZERO);
            s.to_dec() * dividend_discount * n_neg_d1
        }
    };

    Ok(apply_side(price, option))
}

/// Prices a gap binary option.
///
/// Gap options pay the difference between the asset price and a "gap" strike
/// if the option expires in-the-money. For simplicity, we use the standard
/// strike as the gap strike in this implementation.
fn gap_binary_price(option: &Options) -> Result<Decimal, PricingError> {
    // Gap binary is similar to asset-or-nothing minus cash-or-nothing
    // C_gap = C_asset - K * C_cash_unit
    let asset_price = asset_or_nothing_price(option)?;
    let cash_price = cash_or_nothing_price(option, option.strike_price.to_dec())?;

    // For a gap call: pays (S - K) if S > K, otherwise 0
    // This is: asset_or_nothing - K * cash_or_nothing(payout=1)
    let unit_cash = cash_or_nothing_price(option, dec!(1.0))?;

    // Apply side correction (they already have side applied, so we need to be careful)
    let side_multiplier = match option.side {
        crate::model::types::Side::Long => dec!(1),
        crate::model::types::Side::Short => dec!(-1),
    };

    // Remove side from components, compute gap, then reapply
    let asset_unsigned = asset_price * side_multiplier;
    let unit_cash_unsigned = unit_cash * side_multiplier;
    let gap_unsigned = asset_unsigned - option.strike_price.to_dec() * unit_cash_unsigned;

    // Suppress unused variable warning
    let _ = cash_price;

    Ok(gap_unsigned * side_multiplier)
}

/// Calculates intrinsic value for cash-or-nothing at expiration.
fn intrinsic_cash_or_nothing(option: &Options, payout: Decimal) -> Decimal {
    let s = option.underlying_price;
    let k = option.strike_price;
    let itm = match option.option_style {
        OptionStyle::Call => s > k,
        OptionStyle::Put => k > s,
    };
    let value = if itm { payout } else { Decimal::ZERO };
    apply_side(value, option)
}

/// Calculates intrinsic value for asset-or-nothing at expiration.
fn intrinsic_asset_or_nothing(option: &Options) -> Decimal {
    let s = option.underlying_price;
    let k = option.strike_price;
    let itm = match option.option_style {
        OptionStyle::Call => s > k,
        OptionStyle::Put => k > s,
    };
    let value = if itm { s.to_dec() } else { Decimal::ZERO };
    apply_side(value, option)
}

/// Applies the side (long/short) multiplier to the price.
fn apply_side(price: Decimal, option: &Options) -> Decimal {
    match option.side {
        crate::model::types::Side::Long => price,
        crate::model::types::Side::Short => -price,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ExpirationDate;
    use crate::assert_decimal_eq;
    use crate::model::types::{OptionStyle, OptionType, Side};
    use positive::pos_or_panic;
    use rust_decimal_macros::dec;

    fn create_binary_option(style: OptionStyle, binary_type: BinaryType) -> Options {
        Options::new(
            OptionType::Binary { binary_type },
            Side::Long,
            "TEST".to_string(),
            Positive::HUNDRED,                          // strike
            ExpirationDate::Days(pos_or_panic!(182.5)), // ~0.5 years
            pos_or_panic!(0.25),                        // volatility
            Positive::ONE,                              // quantity
            Positive::HUNDRED,                          // underlying (ATM)
            dec!(0.05),                                 // risk-free rate
            style,
            Positive::ZERO, // dividend yield
            None,
        )
    }

    #[test]
    fn test_cash_or_nothing_call() {
        let option = create_binary_option(OptionStyle::Call, BinaryType::CashOrNothing);
        let price = binary_black_scholes(&option).unwrap();
        // ATM call should have ~50% chance of payout, discounted
        assert!(
            price > Decimal::ZERO,
            "Cash-or-nothing call should be positive: {}",
            price
        );
        assert!(
            price < dec!(1.0),
            "Cash-or-nothing call payout=1 should be < 1: {}",
            price
        );
    }

    #[test]
    fn test_cash_or_nothing_put() {
        let option = create_binary_option(OptionStyle::Put, BinaryType::CashOrNothing);
        let price = binary_black_scholes(&option).unwrap();
        assert!(
            price > Decimal::ZERO,
            "Cash-or-nothing put should be positive: {}",
            price
        );
        assert!(
            price < dec!(1.0),
            "Cash-or-nothing put payout=1 should be < 1: {}",
            price
        );
    }

    #[test]
    fn test_asset_or_nothing_call() {
        let option = create_binary_option(OptionStyle::Call, BinaryType::AssetOrNothing);
        let price = binary_black_scholes(&option).unwrap();
        // ATM asset-or-nothing should have significant value
        assert!(
            price > dec!(30.0),
            "Asset-or-nothing call should have substantial value: {}",
            price
        );
        assert!(
            price < dec!(100.0),
            "Asset-or-nothing call should be < S: {}",
            price
        );
    }

    #[test]
    fn test_asset_or_nothing_put() {
        let option = create_binary_option(OptionStyle::Put, BinaryType::AssetOrNothing);
        let price = binary_black_scholes(&option).unwrap();
        assert!(
            price > dec!(30.0),
            "Asset-or-nothing put should have substantial value: {}",
            price
        );
    }

    #[test]
    fn test_gap_binary_call() {
        let option = create_binary_option(OptionStyle::Call, BinaryType::Gap);
        let price = binary_black_scholes(&option).unwrap();
        // Gap binary at ATM should have small value (pays S-K when ITM)
        assert!(
            price > dec!(-10.0),
            "Gap binary call should be reasonable: {}",
            price
        );
    }

    #[test]
    fn test_call_put_parity_cash_or_nothing() {
        // Cash-or-nothing call + put should equal discounted payout
        let call = create_binary_option(OptionStyle::Call, BinaryType::CashOrNothing);
        let put = create_binary_option(OptionStyle::Put, BinaryType::CashOrNothing);

        let call_price = binary_black_scholes(&call).unwrap();
        let put_price = binary_black_scholes(&put).unwrap();

        // Call + Put = Q * e^(-rT)
        let t = call.expiration_date.get_years().unwrap();
        let r = call.risk_free_rate;
        let discounted_payout = DEFAULT_CASH_PAYOUT * (-r * t).exp();

        assert_decimal_eq!(call_price + put_price, discounted_payout, dec!(0.01));
    }

    #[test]
    fn test_short_binary_option() {
        let mut option = create_binary_option(OptionStyle::Call, BinaryType::CashOrNothing);
        let long_price = binary_black_scholes(&option).unwrap();

        option.side = Side::Short;
        let short_price = binary_black_scholes(&option).unwrap();

        assert_decimal_eq!(long_price, -short_price, dec!(1e-10));
    }

    #[test]
    fn test_zero_time_to_expiry_itm() {
        let mut option = create_binary_option(OptionStyle::Call, BinaryType::CashOrNothing);
        option.underlying_price = pos_or_panic!(110.0); // ITM
        option.expiration_date = ExpirationDate::Days(Positive::ZERO);
        let price = binary_black_scholes(&option).unwrap();
        assert_decimal_eq!(price, DEFAULT_CASH_PAYOUT, dec!(1e-10));
    }

    #[test]
    fn test_zero_time_to_expiry_otm() {
        let mut option = create_binary_option(OptionStyle::Call, BinaryType::CashOrNothing);
        option.underlying_price = pos_or_panic!(90.0); // OTM
        option.expiration_date = ExpirationDate::Days(Positive::ZERO);
        let price = binary_black_scholes(&option).unwrap();
        assert_decimal_eq!(price, Decimal::ZERO, dec!(1e-10));
    }

    #[test]
    fn test_deep_itm_cash_or_nothing() {
        let mut option = create_binary_option(OptionStyle::Call, BinaryType::CashOrNothing);
        option.underlying_price = pos_or_panic!(150.0); // Deep ITM
        let price = binary_black_scholes(&option).unwrap();
        // Deep ITM should be close to discounted payout
        let t = option.expiration_date.get_years().unwrap();
        let r = option.risk_free_rate;
        let discounted = DEFAULT_CASH_PAYOUT * (-r * t).exp();
        assert!(
            price > discounted * dec!(0.9),
            "Deep ITM should be close to discounted payout: {}",
            price
        );
    }

    #[test]
    fn test_deep_otm_cash_or_nothing() {
        let mut option = create_binary_option(OptionStyle::Call, BinaryType::CashOrNothing);
        option.underlying_price = pos_or_panic!(50.0); // Deep OTM
        let price = binary_black_scholes(&option).unwrap();
        assert!(
            price < dec!(0.1),
            "Deep OTM should be nearly zero: {}",
            price
        );
    }
}
