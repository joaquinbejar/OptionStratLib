/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 25/10/24
******************************************************************************/
use crate::chains::chain::OptionData;
use crate::constants::ZERO;
use crate::error::chains::ChainError;
use crate::model::types::ExpirationDate;
use crate::Positive;
use rust_decimal::Decimal;
use std::collections::BTreeSet;
use std::fmt::Display;

#[derive(Debug)]
pub enum OptionDataGroup<'a> {
    One(&'a OptionData),
    Two(&'a OptionData, &'a OptionData),
    Three(&'a OptionData, &'a OptionData, &'a OptionData),
    Four(
        &'a OptionData,
        &'a OptionData,
        &'a OptionData,
        &'a OptionData,
    ),
    Any(Vec<&'a OptionData>),
}
pub struct OptionChainBuildParams {
    pub(crate) symbol: String,
    pub(crate) volume: Option<Positive>,
    pub(crate) chain_size: usize,
    pub(crate) strike_interval: Positive,
    pub(crate) skew_factor: f64,
    pub(crate) spread: Positive,
    pub(crate) decimal_places: i32,
    pub(crate) price_params: OptionDataPriceParams,
}

#[allow(clippy::too_many_arguments)]
impl OptionChainBuildParams {
    pub fn new(
        symbol: String,
        volume: Option<Positive>,
        chain_size: usize,
        strike_interval: Positive,
        skew_factor: f64,
        spread: Positive,
        decimal_places: i32,
        price_params: OptionDataPriceParams,
    ) -> Self {
        Self {
            symbol,
            volume,
            chain_size,
            strike_interval,
            skew_factor,
            spread,
            decimal_places,
            price_params,
        }
    }
}

#[derive(Debug)]
pub struct OptionDataPriceParams {
    pub(crate) underlying_price: Positive,
    pub(crate) expiration_date: ExpirationDate,
    pub(crate) implied_volatility: Option<Positive>,
    pub(crate) risk_free_rate: Decimal,
    pub(crate) dividend_yield: Positive,
}

impl OptionDataPriceParams {
    pub fn new(
        underlying_price: Positive,
        expiration_date: ExpirationDate,
        implied_volatility: Option<Positive>,
        risk_free_rate: Decimal,
        dividend_yield: Positive,
    ) -> Self {
        Self {
            underlying_price,
            expiration_date,
            implied_volatility,
            risk_free_rate,
            dividend_yield,
        }
    }

    pub fn get_underlying_price(&self) -> Positive {
        self.underlying_price
    }

    pub fn get_expiration_date(&self) -> ExpirationDate {
        self.expiration_date.clone()
    }

    pub fn get_implied_volatility(&self) -> Option<Positive> {
        self.implied_volatility
    }

    pub fn get_risk_free_rate(&self) -> Decimal {
        self.risk_free_rate
    }

    pub fn get_dividend_yield(&self) -> Positive {
        self.dividend_yield
    }
}

impl Default for OptionDataPriceParams {
    fn default() -> Self {
        Self {
            underlying_price: Positive::ZERO,
            expiration_date: ExpirationDate::Days(0.0),
            implied_volatility: None,
            risk_free_rate: Decimal::ZERO,
            dividend_yield: Positive::ZERO,
        }
    }
}

impl Display for OptionDataPriceParams {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Underlying Price: {:.3}, Expiration: {:.4} Years, Implied Volatility: {:.3}, Risk-Free Rate: {:.2}, Dividend Yield: {:.2}",
            self.underlying_price,
            self.expiration_date.get_years(),
            self.implied_volatility.unwrap_or(Positive::ZERO).value(),
            self.risk_free_rate,
            self.dividend_yield
        )
    }
}

pub trait OptionChainParams {
    fn get_params(&self, strike_price: Positive) -> Result<OptionDataPriceParams, ChainError>;
}

/// Parameters for generating random positions in an option chain
#[derive(Clone, Debug)]
pub struct RandomPositionsParams {
    /// Number of long put positions to generate
    pub qty_puts_long: Option<usize>,
    /// Number of short put positions to generate  
    pub qty_puts_short: Option<usize>,
    /// Number of long call positions to generate
    pub qty_calls_long: Option<usize>,
    /// Number of short call positions to generate
    pub qty_calls_short: Option<usize>,
    /// Expiration date for the options
    pub expiration_date: ExpirationDate,
    /// Quantity for each option position
    pub option_qty: Positive,
    /// Risk free interest rate
    pub risk_free_rate: Decimal,
    /// Dividend yield of the underlying
    pub dividend_yield: Positive,
    /// Fee for opening put positions
    pub open_put_fee: f64,
    /// Fee for opening call positions
    pub open_call_fee: f64,
    /// Fee for closing put positions
    pub close_put_fee: f64,
    /// Fee for closing call positions
    pub close_call_fee: f64,
}

impl RandomPositionsParams {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        qty_puts_long: Option<usize>,
        qty_puts_short: Option<usize>,
        qty_calls_long: Option<usize>,
        qty_calls_short: Option<usize>,
        expiration_date: ExpirationDate,
        option_qty: Positive,
        risk_free_rate: Decimal,
        dividend_yield: Positive,
        open_put_fee: f64,
        open_call_fee: f64,
        close_put_fee: f64,
        close_call_fee: f64,
    ) -> Self {
        Self {
            qty_puts_long,
            qty_puts_short,
            qty_calls_long,
            qty_calls_short,
            expiration_date,
            option_qty,
            risk_free_rate,
            dividend_yield,
            open_put_fee,
            open_call_fee,
            close_put_fee,
            close_call_fee,
        }
    }

    /// Returns the total number of positions to generate
    pub fn total_positions(&self) -> usize {
        self.qty_puts_long.unwrap_or(0)
            + self.qty_puts_short.unwrap_or(0)
            + self.qty_calls_long.unwrap_or(0)
            + self.qty_calls_short.unwrap_or(0)
    }
}

#[allow(dead_code)]
pub(crate) fn generate_list_of_strikes(
    reference_price: Positive,
    chain_size: usize,
    strike_interval: Positive,
) -> BTreeSet<Positive> {
    let mut strikes = BTreeSet::new();
    let reference_price_rounded = rounder(reference_price, strike_interval);

    for i in 0..=chain_size {
        let next_strike = i as f64 * strike_interval;
        if reference_price_rounded < next_strike {
            // panic!("Reference price is lower than the next strike: {}, {}", next_strike, reference_price_rounded);
            break;
        }
        let lower_strike = (reference_price_rounded - (i as f64 * strike_interval)).floor();
        let upper_strike = (reference_price_rounded + (i as f64 * strike_interval)).floor();

        if i == 0 {
            strikes.insert(reference_price_rounded);
        } else {
            strikes.insert(lower_strike);
            strikes.insert(upper_strike);
        }
    }
    strikes
}

pub(crate) fn adjust_volatility(
    volatility: Option<Positive>,
    skew_factor: f64,
    atm_distance: f64,
) -> Option<Positive> {
    volatility?;
    let skew = skew_factor * atm_distance.abs();
    let smile = skew_factor * atm_distance.powi(2);

    let volatility_skew = volatility.unwrap() * (1.0 + skew + smile);
    Some(volatility_skew)
}

pub(crate) fn parse<T: std::str::FromStr>(s: &str) -> Option<T> {
    let input: Result<T, String> = match s.parse::<T>() {
        Ok(value) => Ok(value),
        Err(_) => {
            return None;
        }
    };

    match input {
        Ok(value) => Some(value),
        Err(_) => None,
    }
}

pub(crate) fn default_empty_string<T: ToString>(input: Option<T>) -> String {
    input.map_or_else(|| "".to_string(), |v| v.to_string())
}

pub(crate) fn rounder(reference_price: Positive, strike_interval: Positive) -> Positive {
    if strike_interval == Positive::ZERO {
        return reference_price;
    }
    let price = reference_price.value();
    let interval = strike_interval.value();

    let remainder = price % interval;
    let base = price - remainder;

    let rounded = if remainder >= interval / Decimal::TWO {
        base + interval
    } else {
        base
    };

    rounded.into()
}

#[cfg(test)]
mod tests_rounder {
    use super::*;
    use crate::pos;

    #[test]
    fn test_rounder() {
        assert_eq!(rounder(pos!(151.0), pos!(5.0)), pos!(150.0));
        assert_eq!(rounder(pos!(154.0), pos!(5.0)), pos!(155.0));
        assert_eq!(rounder(pos!(152.5), pos!(5.0)), pos!(155.0));
        assert_eq!(rounder(pos!(152.4), pos!(5.0)), pos!(150.0));

        assert_eq!(rounder(pos!(151.0), pos!(10.0)), pos!(150.0));
        assert_eq!(rounder(pos!(156.0), pos!(10.0)), pos!(160.0));
        assert_eq!(rounder(pos!(155.0), pos!(10.0)), pos!(160.0));
        assert_eq!(rounder(pos!(154.9), pos!(10.0)), pos!(150.0));

        assert_eq!(rounder(pos!(17.0), pos!(15.0)), pos!(15.0));
        assert_eq!(rounder(pos!(43.0), pos!(15.0)), pos!(45.0));
        assert_eq!(rounder(pos!(37.5), pos!(15.0)), pos!(45.0));
        assert_eq!(rounder(pos!(37.4), pos!(15.0)), pos!(30.0));
    }
}

#[cfg(test)]
mod tests_generate_list_of_strikes {
    use super::*;
    use crate::Positive;

    #[test]
    fn test_generate_list_of_strikes_basic() {
        let reference_price = Positive::THOUSAND;
        let chain_size = 3;
        let strike_interval = Positive::TEN;

        let strikes = generate_list_of_strikes(reference_price, chain_size, strike_interval);

        assert_eq!(strikes.len(), 7);

        assert!(strikes.contains(&Positive::new(970.0).unwrap()));
        assert!(strikes.contains(&Positive::new(980.0).unwrap()));
        assert!(strikes.contains(&reference_price));
        assert!(strikes.contains(&Positive::new(1010.0).unwrap()));
        assert!(strikes.contains(&Positive::new(1030.0).unwrap()));
    }

    #[test]
    fn test_generate_list_of_strikes_zero_chain_size() {
        let reference_price = Positive::new(1000.0).unwrap();
        let chain_size = 0;
        let strike_interval = Positive::new(10.0).unwrap();

        let strikes = generate_list_of_strikes(reference_price, chain_size, strike_interval);

        assert_eq!(strikes.len(), 1);
        assert!(strikes.contains(&reference_price));
    }

    #[test]
    fn test_generate_list_of_strikes_large_interval() {
        let reference_price = Positive::new(1000.0).unwrap();
        let chain_size = 3;
        let strike_interval = Positive::new(100.0).unwrap();

        let strikes = generate_list_of_strikes(reference_price, chain_size, strike_interval);

        assert!(strikes.contains(&Positive::new(700.0).unwrap()));
        assert!(strikes.contains(&Positive::new(800.0).unwrap()));
        assert!(strikes.contains(&Positive::new(900.0).unwrap()));
        assert!(strikes.contains(&reference_price));
        assert!(strikes.contains(&Positive::new(1100.0).unwrap()));
        assert!(strikes.contains(&Positive::new(1200.0).unwrap()));
        assert!(strikes.contains(&Positive::new(1300.0).unwrap()));
    }

    #[test]
    fn test_generate_list_of_strikes_duplicate_strikes() {
        let reference_price = Positive::new(1000.0).unwrap();
        let chain_size = 1;
        let strike_interval = Positive::new(0.0).unwrap();

        let strikes = generate_list_of_strikes(reference_price, chain_size, strike_interval);

        assert_eq!(strikes.len(), 1);
        assert!(strikes.contains(&reference_price));
    }
}

#[cfg(test)]
mod tests_parse {
    use super::*;
    use crate::spos;
    use std::f64::consts::PI;

    #[test]
    fn test_parse_valid_integer() {
        let input = "42";
        let result: Option<i32> = parse(input);
        assert_eq!(result, Some(42));
    }

    #[test]
    fn test_parse_invalid_integer() {
        let input = "not_a_number";
        let result: Option<i32> = parse(input);
        assert_eq!(result, None);
    }

    #[test]
    fn test_parse_valid_float() {
        let input = &*PI.to_string();
        let result: Option<f64> = parse(input);
        assert_eq!(result, Some(PI));
    }

    #[test]
    fn test_positive_f64() {
        let input = "42.01";
        let result: Option<Positive> = parse(input);
        assert_eq!(result, spos!(42.01));
    }
}

#[cfg(test)]
mod tests_default_empty_string {
    use super::*;

    #[test]
    fn test_default_empty_string_with_some_value() {
        let input = Some(42);
        let result = default_empty_string(input);
        assert_eq!(result, "42");
    }

    #[test]
    fn test_default_empty_string_with_float() {
        let input = Some(42.01223);
        let result = default_empty_string(input);
        assert_eq!(result, "42.01223");
    }

    #[test]
    fn test_default_empty_string_with_none() {
        let input: Option<i32> = None;
        let result = default_empty_string(input);
        assert_eq!(result, "");
    }

    #[test]
    fn test_default_empty_string_with_string_value() {
        let input = Some("Hello");
        let result = default_empty_string(input);
        assert_eq!(result, "Hello");
    }
}

#[cfg(test)]
mod tests_random_positions_params {
    use num_traits::ToPrimitive;
    use rust_decimal_macros::dec;
    use super::*;
    use crate::pos;

    fn create_test_params() -> RandomPositionsParams {
        RandomPositionsParams::new(
            Some(1),
            Some(1),
            Some(1),
            Some(1),
            ExpirationDate::Days(30.0),
            pos!(1.0),
            dec!(0.05),
            pos!(0.02),
            1.0,
            1.0,
            1.0,
            1.0,
        )
    }

    #[test]
    fn test_new_params() {
        let params = create_test_params();
        assert_eq!(params.qty_puts_long, Some(1));
        assert_eq!(params.qty_puts_short, Some(1));
        assert_eq!(params.qty_calls_long, Some(1));
        assert_eq!(params.qty_calls_short, Some(1));
        assert_eq!(params.option_qty, 1.0);
        assert_eq!(params.risk_free_rate.to_f64().unwrap(), 0.05);
        assert_eq!(params.dividend_yield.to_f64(), 0.02);
        assert_eq!(params.open_put_fee, 1.0);
        assert_eq!(params.close_put_fee, 1.0);
        assert_eq!(params.open_call_fee, 1.0);
        assert_eq!(params.close_call_fee, 1.0);
    }

    #[test]
    fn test_total_positions() {
        let params = create_test_params();
        assert_eq!(params.total_positions(), 4);

        let params = RandomPositionsParams::new(
            Some(2),
            None,
            Some(3),
            None,
            ExpirationDate::Days(30.0),
            pos!(1.0),
            dec!(0.05),
            pos!(0.02),
            1.0,
            1.0,
            1.0,
            1.0,
        );
        assert_eq!(params.total_positions(), 5);

        let params = RandomPositionsParams::new(
            None,
            None,
            None,
            None,
            ExpirationDate::Days(30.0),
            pos!(1.0),
            dec!(0.05),
            pos!(0.02),
            1.0,
            1.0,
            1.0,
            1.0,
        );
        assert_eq!(params.total_positions(), 0);
    }

    #[test]
    fn test_clone() {
        let params = create_test_params();
        let cloned = params.clone();
        assert_eq!(params.total_positions(), cloned.total_positions());
    }

    #[test]
    fn test_debug() {
        let params = create_test_params();
        let debug_output = format!("{:?}", params);
        assert!(debug_output.contains("RandomPositionsParams"));
    }
}

#[cfg(test)]
mod tests_adjust_volatility {
    use super::*;
    use crate::spos;

    #[test]
    fn test_adjust_volatility_none() {
        let result = adjust_volatility(None, 0.1, 10.0);
        assert_eq!(result, None);
    }

    #[test]
    fn test_adjust_volatility_zero_skew() {
        let vol = spos!(0.2);
        let result = adjust_volatility(vol, 0.0, 10.0);
        assert_eq!(result, vol);
    }

    #[test]
    fn test_adjust_volatility_positive_distance() {
        let vol = spos!(0.2);
        let result = adjust_volatility(vol, 0.1, 10.0);
        assert!(result.is_some());
        assert!(result.unwrap() > vol.unwrap());
    }

    #[test]
    fn test_adjust_volatility_negative_distance() {
        let vol = spos!(0.2);
        let result = adjust_volatility(vol, 0.1, -10.0);
        assert!(result.is_some());
        assert!(result.unwrap() > vol.unwrap());
    }
}

#[cfg(test)]
mod tests_option_data_price_params {
    use num_traits::ToPrimitive;
    use rust_decimal_macros::dec;
    use super::*;
    use crate::{pos, spos};

    #[test]
    fn test_new_price_params() {
        let params = OptionDataPriceParams::new(
            pos!(100.0),
            ExpirationDate::Days(30.0),
            spos!(0.2),
            dec!(0.05),
            pos!(0.02),
        );

        assert_eq!(params.underlying_price, pos!(100.0));
        assert_eq!(params.risk_free_rate.to_f64().unwrap(), 0.05);
        assert_eq!(params.dividend_yield.to_f64(), 0.02);
        assert_eq!(params.implied_volatility, spos!(0.2));
    }

    #[test]
    fn test_default_price_params() {
        let params = OptionDataPriceParams::default();
        assert_eq!(params.underlying_price, Positive::ZERO);
        assert_eq!(params.risk_free_rate.to_f64().unwrap(), ZERO);
        assert_eq!(params.dividend_yield.to_f64(), ZERO);
        assert_eq!(params.implied_volatility, None);
    }

    #[test]
    fn test_display_price_params() {
        let params = OptionDataPriceParams::new(
            pos!(100.0),
            ExpirationDate::Days(30.0),
            spos!(0.2),
            dec!(0.05),
            pos!(0.02),
        );
        let display_string = format!("{}", params);
        assert!(display_string.contains("Underlying Price: 100.000"));
        assert!(display_string.contains("Implied Volatility: 0.200"));
        assert!(display_string.contains("Risk-Free Rate: 0.05"));
        assert!(display_string.contains("Dividend Yield: 0.02"));
    }

    #[test]
    fn test_display_price_params_no_volatility() {
        let params =
            OptionDataPriceParams::new(pos!(100.0), ExpirationDate::Days(30.0), None, dec!(0.05), pos!(0.02));
        let display_string = format!("{}", params);
        assert!(display_string.contains("Implied Volatility: 0.000"));
    }
}

#[cfg(test)]
mod tests_option_chain_build_params {
    use rust_decimal_macros::dec;
    use super::*;
    use crate::{pos, spos};

    #[test]
    fn test_new_chain_build_params() {
        let price_params = OptionDataPriceParams::new(
            pos!(100.0),
            ExpirationDate::Days(30.0),
            spos!(0.2),
            dec!(0.05),
            pos!(0.02),
        );

        let params = OptionChainBuildParams::new(
            "TEST".to_string(),
            spos!(1000.0),
            10,
            pos!(5.0),
            0.1,
            pos!(0.02),
            2,
            price_params,
        );

        assert_eq!(params.symbol, "TEST");
        assert_eq!(params.volume, spos!(1000.0));
        assert_eq!(params.chain_size, 10);
        assert_eq!(params.strike_interval, pos!(5.0));
        assert_eq!(params.skew_factor, 0.1);
        assert_eq!(params.spread, pos!(0.02));
        assert_eq!(params.decimal_places, 2);
    }

    #[test]
    fn test_chain_build_params_without_volume() {
        let price_params = OptionDataPriceParams::default();

        let params = OptionChainBuildParams::new(
            "TEST".to_string(),
            None,
            10,
            pos!(5.0),
            0.1,
            pos!(0.02),
            2,
            price_params,
        );

        assert_eq!(params.volume, None);
    }
}

#[cfg(test)]
mod tests_random_positions_params_extended {
    use rust_decimal_macros::dec;
    use super::*;
    use crate::pos;

    #[test]
    fn test_partial_positions() {
        let params = RandomPositionsParams::new(
            Some(2),
            None,
            Some(1),
            None,
            ExpirationDate::Days(30.0),
            pos!(1.0),
            dec!(0.05),
            pos!(0.02),
            1.0,
            1.0,
            1.0,
            1.0,
        );

        assert_eq!(params.qty_puts_long, Some(2));
        assert_eq!(params.qty_puts_short, None);
        assert_eq!(params.qty_calls_long, Some(1));
        assert_eq!(params.qty_calls_short, None);
        assert_eq!(params.total_positions(), 3);
    }

    #[test]
    fn test_no_positions() {
        let params = RandomPositionsParams::new(
            None,
            None,
            None,
            None,
            ExpirationDate::Days(30.0),
            pos!(1.0),
            dec!(0.05),
            pos!(0.02),
            1.0,
            1.0,
            1.0,
            1.0,
        );

        assert_eq!(params.total_positions(), 0);
    }

    #[test]
    fn test_expiration_date() {
        let params = RandomPositionsParams::new(
            None,
            None,
            None,
            None,
            ExpirationDate::Days(30.0),
            pos!(1.0),
            dec!(0.05),
            pos!(0.02),
            1.0,
            1.0,
            1.0,
            1.0,
        );

        match params.expiration_date {
            ExpirationDate::Days(days) => assert_eq!(days, 30.0),
            _ => panic!("Expected ExpirationDate::Days"),
        }
    }
}

#[cfg(test)]
mod tests_sample {
    use rust_decimal_macros::dec;
    use super::*;
    use crate::chains::chain::OptionChain;

    #[test]
    fn test_chain() {
        let chain = OptionDataPriceParams::new(
            Positive::new(2000.0).unwrap(),
            ExpirationDate::Days(10.0),
            Some(Positive::new(0.01).unwrap()),
            dec!(0.01),
            Positive::ZERO,
        );

        let params = OptionChainBuildParams::new(
            "SP500".to_string(),
            Some(Positive::ONE),
            5,
            Positive::ONE,
            0.0001,
            Positive::new(0.02).unwrap(),
            2,
            chain,
        );

        let built_chain = OptionChain::build_chain(&params);

        assert_eq!(built_chain.symbol, "SP500");
        assert_eq!(built_chain.underlying_price, Positive::new(2000.0).unwrap());
    }
}
