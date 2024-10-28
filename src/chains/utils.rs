/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 25/10/24
******************************************************************************/
use crate::constants::ZERO;
use crate::model::types::{ExpirationDate, PositiveF64, PZERO};
use std::collections::BTreeSet;
use std::fmt::Display;
use crate::pos;

pub struct OptionChainBuildParams {
    pub(crate) symbol: String,
    pub(crate) volume: Option<PositiveF64>,
    pub(crate) chain_size: usize,
    pub(crate) strike_interval: PositiveF64,
    pub(crate) skew_factor: f64,
    pub(crate) spread: PositiveF64,
    pub(crate) decimal_places: i32,
    pub(crate) price_params: OptionDataPriceParams,
}

#[allow(clippy::too_many_arguments)]
impl OptionChainBuildParams {
    pub fn new(
        symbol: String,
        volume: Option<PositiveF64>,
        chain_size: usize,
        strike_interval: PositiveF64,
        skew_factor: f64,
        spread: PositiveF64,
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

pub struct OptionDataPriceParams {
    pub(crate) underlying_price: PositiveF64,
    pub(crate) expiration_date: ExpirationDate,
    pub(crate) implied_volatility: Option<PositiveF64>,
    pub(crate) risk_free_rate: f64,
    pub(crate) dividend_yield: f64,
}

impl OptionDataPriceParams {
    pub fn new(
        underlying_price: PositiveF64,
        expiration_date: ExpirationDate,
        implied_volatility: Option<PositiveF64>,
        risk_free_rate: f64,
        dividend_yield: f64,
    ) -> Self {
        Self {
            underlying_price,
            expiration_date,
            implied_volatility,
            risk_free_rate,
            dividend_yield,
        }
    }
}

impl Default for OptionDataPriceParams {
    fn default() -> Self {
        Self {
            underlying_price: PZERO,
            expiration_date: ExpirationDate::Days(0.0),
            implied_volatility: None,
            risk_free_rate: ZERO,
            dividend_yield: ZERO,
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
            self.implied_volatility.unwrap_or(PZERO).value(),
            self.risk_free_rate,
            self.dividend_yield
        )
    }
}

#[allow(dead_code)]
pub(crate) fn generate_list_of_strikes(
    reference_price: PositiveF64,
    chain_size: usize,
    strike_interval: PositiveF64,
) -> BTreeSet<PositiveF64> {
    let mut strikes = BTreeSet::new();
    let reference_price_rounded = rounder(reference_price, strike_interval);

    for i in 0..=chain_size {
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
    volatility: Option<PositiveF64>,
    skew_factor: f64,
    atm_distance: f64,
) -> Option<PositiveF64> {
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


pub(crate) fn rounder(reference_price: PositiveF64, strike_interval: PositiveF64) -> PositiveF64 {
    let price = reference_price.value();
    let interval = strike_interval.value();

    let remainder = price % interval;
    let base = price - remainder;

    // Si el remainder es mayor que la mitad del intervalo, redondea hacia arriba
    let rounded = if remainder >= interval / 2.0 {
        base + interval
    } else {
        base
    };

    pos!(rounded)
}

#[cfg(test)]
mod tests_rounder {
    use super::*;
    use crate::pos;

    #[test]
    fn test_rounder() {
        // Pruebas con intervalo de 5
        assert_eq!(rounder(pos!(151.0), pos!(5.0)), pos!(150.0));
        assert_eq!(rounder(pos!(154.0), pos!(5.0)), pos!(155.0));
        assert_eq!(rounder(pos!(152.5), pos!(5.0)), pos!(155.0));
        assert_eq!(rounder(pos!(152.4), pos!(5.0)), pos!(150.0));

        // Pruebas con intervalo de 10
        assert_eq!(rounder(pos!(151.0), pos!(10.0)), pos!(150.0));
        assert_eq!(rounder(pos!(156.0), pos!(10.0)), pos!(160.0));
        assert_eq!(rounder(pos!(155.0), pos!(10.0)), pos!(160.0));
        assert_eq!(rounder(pos!(154.9), pos!(10.0)), pos!(150.0));

        // Pruebas con intervalo de 15
        assert_eq!(rounder(pos!(17.0), pos!(15.0)), pos!(15.0));
        assert_eq!(rounder(pos!(43.0), pos!(15.0)), pos!(45.0));
        assert_eq!(rounder(pos!(37.5), pos!(15.0)), pos!(45.0));
        assert_eq!(rounder(pos!(37.4), pos!(15.0)), pos!(30.0));
    }
}

#[cfg(test)]
mod tests_generate_list_of_strikes {
    use super::*;
    use crate::model::types::PositiveF64;

    #[test]
    fn test_generate_list_of_strikes_basic() {
        let reference_price = PositiveF64::new(1000.0).unwrap();
        let chain_size = 3;
        let strike_interval = PositiveF64::new(10.0).unwrap();

        let strikes = generate_list_of_strikes(reference_price, chain_size, strike_interval);

        // Verificamos el número total de strikes generados (2 * chain_size + 1)
        assert_eq!(strikes.len(), 7);

        // Verificamos que los valores específicos están presentes
        assert!(strikes.contains(&PositiveF64::new(970.0).unwrap()));
        assert!(strikes.contains(&PositiveF64::new(980.0).unwrap()));
        assert!(strikes.contains(&reference_price));
        assert!(strikes.contains(&PositiveF64::new(1010.0).unwrap()));
        assert!(strikes.contains(&PositiveF64::new(1030.0).unwrap()));
    }

    #[test]
    fn test_generate_list_of_strikes_zero_chain_size() {
        let reference_price = PositiveF64::new(1000.0).unwrap();
        let chain_size = 0;
        let strike_interval = PositiveF64::new(10.0).unwrap();

        let strikes = generate_list_of_strikes(reference_price, chain_size, strike_interval);

        // Solo debería contener el precio de referencia
        assert_eq!(strikes.len(), 1);
        assert!(strikes.contains(&reference_price));
    }

    #[test]
    fn test_generate_list_of_strikes_large_interval() {
        let reference_price = PositiveF64::new(1000.0).unwrap();
        let chain_size = 3;
        let strike_interval = PositiveF64::new(100.0).unwrap();

        let strikes = generate_list_of_strikes(reference_price, chain_size, strike_interval);

        assert!(strikes.contains(&PositiveF64::new(700.0).unwrap()));
        assert!(strikes.contains(&PositiveF64::new(800.0).unwrap()));
        assert!(strikes.contains(&PositiveF64::new(900.0).unwrap()));
        assert!(strikes.contains(&reference_price));
        assert!(strikes.contains(&PositiveF64::new(1100.0).unwrap()));
        assert!(strikes.contains(&PositiveF64::new(1200.0).unwrap()));
        assert!(strikes.contains(&PositiveF64::new(1300.0).unwrap()));
    }

    #[test]
    fn test_generate_list_of_strikes_duplicate_strikes() {
        let reference_price = PositiveF64::new(1000.0).unwrap();
        let chain_size = 1;
        let strike_interval = PositiveF64::new(0.0).unwrap();

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
        let result: Option<PositiveF64> = parse(input);
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
