/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 11/8/24
******************************************************************************/

use crate::greeks::utils::{big_n, d1, d2, n};
use crate::model::option::Options;
use crate::model::types::OptionStyle;

#[allow(dead_code)]
pub fn delta(option: &Options) -> f64 {
    let d1 = d1(
        option.underlying_price,
        option.strike_price,
        option.risk_free_rate,
        option.expiration_date.get_years(),
        option.implied_volatility,
    );

    match option.option_style {
        OptionStyle::Call => (-option.dividend_yield * option.expiration_date.get_years()).exp() * big_n(d1),
        OptionStyle::Put => (-option.dividend_yield * option.expiration_date.get_years()).exp() * (big_n(d1) - 1.0),
    }
}

#[allow(dead_code)]
pub fn gamma(option: &Options) -> f64 {
    let d1 = d1(
        option.underlying_price,
        option.strike_price,
        option.risk_free_rate,
        option.expiration_date.get_years(),
        option.implied_volatility,
    );

    (-option.dividend_yield * option.expiration_date.get_years()).exp() * n(d1)
        / (option.underlying_price * option.implied_volatility * option.expiration_date.get_years().sqrt())
}

#[allow(dead_code)]
pub fn theta(option: &Options) -> f64 {
    let d1 = d1(
        option.underlying_price,
        option.strike_price,
        option.risk_free_rate,
        option.expiration_date.get_years(),
        option.implied_volatility,
    );
    let d2 = d2(
        option.underlying_price,
        option.strike_price,
        option.risk_free_rate,
        option.expiration_date.get_years(),
        option.implied_volatility,
    );

    let common_term = -option.underlying_price
        * option.implied_volatility
        * (-option.dividend_yield * option.expiration_date.get_years()).exp()
        * n(d1)
        / (2.0 * option.expiration_date.get_years().sqrt());

    match option.option_style {
        OptionStyle::Call => {
            common_term
                - option.risk_free_rate
                    * option.strike_price
                    * (-option.risk_free_rate * option.expiration_date.get_years()).exp()
                    * big_n(d2)
                + option.dividend_yield
                    * option.underlying_price
                    * (-option.dividend_yield * option.expiration_date.get_years()).exp()
                    * big_n(d1)
        }
        OptionStyle::Put => {
            common_term
                + option.risk_free_rate
                    * option.strike_price
                    * (-option.risk_free_rate * option.expiration_date.get_years()).exp()
                    * big_n(-d2)
                - option.dividend_yield
                    * option.underlying_price
                    * (-option.dividend_yield * option.expiration_date.get_years()).exp()
                    * big_n(-d1)
        }
    }
}

#[allow(dead_code)]
pub fn vega(option: &Options) -> f64 {
    let d1 = d1(
        option.underlying_price,
        option.strike_price,
        option.risk_free_rate,
        option.expiration_date.get_years(),
        option.implied_volatility,
    );

    option.underlying_price
        * (-option.dividend_yield * option.expiration_date.get_years()).exp()
        * n(d1)
        * option.expiration_date.get_years().sqrt()
}

#[allow(dead_code)]
pub fn rho(option: &Options) -> f64 {
    let d2 = d2(
        option.underlying_price,
        option.strike_price,
        option.risk_free_rate,
        option.expiration_date.get_years(),
        option.implied_volatility,
    );

    match option.option_style {
        OptionStyle::Call => {
            option.strike_price
                * option.expiration_date.get_years()
                * (-option.risk_free_rate * option.expiration_date.get_years()).exp()
                * big_n(d2)
        }
        OptionStyle::Put => {
            -option.strike_price
                * option.expiration_date.get_years()
                * (-option.risk_free_rate * option.expiration_date.get_years()).exp()
                * big_n(-d2)
        }
    }
}

#[allow(dead_code)]
pub fn rho_d(option: &Options) -> f64 {
    let d1 = d1(
        option.underlying_price,
        option.strike_price,
        option.risk_free_rate,
        option.expiration_date.get_years(),
        option.implied_volatility,
    );

    match option.option_style {
        OptionStyle::Call => {
            -option.expiration_date.get_years()
                * option.underlying_price
                * (-option.dividend_yield * option.expiration_date.get_years()).exp()
                * big_n(d1)
        }
        OptionStyle::Put => {
            option.expiration_date.get_years()
                * option.underlying_price
                * (-option.dividend_yield * option.expiration_date.get_years()).exp()
                * big_n(-d1)
        }
    }
}

#[cfg(test)]
mod tests_greeks_equations {
    use super::*;
    use crate::model::option::Options;
    use crate::model::types::{OptionStyle, OptionType, Side};
    use approx::assert_relative_eq;

    fn create_test_option(style: OptionStyle) -> Options {
        Options {
            option_type: OptionType::European,
            side: Side::Long,
            underlying_price: 100.0,
            strike_price: 100.0,
            risk_free_rate: 0.05,
            dividend_yield: 0.02,
            implied_volatility: 7.2,
            option_style: style,
            underlying_symbol: "".to_string(),
            expiration_date: Default::default(),
            quantity: 0,
            spot_prices: None,
            spot_min: None,
            spot_max: None,
        }
    }

    #[test]
    fn test_delta() {
        let call_option = create_test_option(OptionStyle::Call);
        let put_option = create_test_option(OptionStyle::Put);


        assert_relative_eq!(delta(&call_option), 0.9801, epsilon = 1e-4);
        assert_relative_eq!(
            delta(&put_option),
            -0.000151,
            epsilon = 1e-6
        );
    }

    #[test]
    fn test_gamma() {
        let option = create_test_option(OptionStyle::Call); // Gamma is the same for calls and puts


        assert_relative_eq!(
            gamma(&option),
            8.124480543702491e-7,
            epsilon = 1e-6
        );
    }

    #[test]
    fn test_theta() {
        let call_option = create_test_option(OptionStyle::Call);
        let put_option = create_test_option(OptionStyle::Put);


        assert_relative_eq!(theta(&call_option), 1.7487299, epsilon = 1e-4);
        assert_relative_eq!(theta(&put_option), 4.5444, epsilon = 1e-4);
    }

    #[test]
    fn test_vega() {
        let option = create_test_option(OptionStyle::Call); // Vega is the same for calls and puts
        assert_relative_eq!(vega(&option), 0.05849, epsilon = 1e-4);
    }

    #[test]
    fn test_rho() {
        let call_option = create_test_option(OptionStyle::Call);
        let put_option = create_test_option(OptionStyle::Put);

        assert_relative_eq!(rho(&call_option), 0.015544, epsilon = 1e-4);
        assert_relative_eq!(rho(&put_option), -95.1073, epsilon = 1e-4);
    }

    #[test]
    fn test_rho_d() {
        let call_option = create_test_option(OptionStyle::Call);
        let put_option = create_test_option(OptionStyle::Put);


        assert_relative_eq!(
            rho_d(&call_option),
            -98.00468,
            epsilon = 1e-4
        );
        assert_relative_eq!(rho_d(&put_option), 0.01518, epsilon = 1e-4);
    }
}
