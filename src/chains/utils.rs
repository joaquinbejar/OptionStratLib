/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 25/10/24
******************************************************************************/
use crate::model::types::PositiveF64;
use std::collections::BTreeSet;

#[allow(dead_code)]
pub(crate) fn generate_list_of_strikes(
    reference_price: PositiveF64,
    chain_size: usize,
    strike_interval: PositiveF64,
) -> BTreeSet<PositiveF64> {
    let mut strikes = BTreeSet::new();

    for i in 0..=chain_size {
        let lower_strike = reference_price - (i as f64 * strike_interval);
        let upper_strike = reference_price + (i as f64 * strike_interval);

        if i == 0 {
            strikes.insert(reference_price);
        } else {
            strikes.insert(lower_strike);
            strikes.insert(upper_strike);
        }
    }
    strikes
}

pub(crate) fn adjust_volatility(
    volatility: PositiveF64,
    skew_factor: f64,
    atm_distance: f64,
) -> f64 {
    let skew = skew_factor * atm_distance.abs();
    let smile = skew_factor * atm_distance.powi(2);

    volatility.value() * (1.0 + skew + smile)
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
