/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 11/8/24
******************************************************************************/

use crate::greeks::utils::{big_n, d1, d2, n};
use crate::model::option::Options;
use crate::model::types::OptionStyle;

#[allow(dead_code)]
pub fn delta(option: &Options, time_to_expiry: f64) -> f64 {
    let d1 = d1(
        option.underlying_price,
        option.risk_free_rate,
        time_to_expiry,
        option.implied_volatility,
    );

    match option.option_style {
        OptionStyle::Call => (-option.dividend_yield * time_to_expiry).exp() * big_n(d1),
        OptionStyle::Put => (-option.dividend_yield * time_to_expiry).exp() * (big_n(d1) - 1.0),
    }
}

#[allow(dead_code)]
pub fn gamma(option: &Options, time_to_expiry: f64) -> f64 {
    let d1 = d1(
        option.underlying_price,
        option.risk_free_rate,
        time_to_expiry,
        option.implied_volatility,
    );

    (-option.dividend_yield * time_to_expiry).exp() * n(d1)
        / (option.underlying_price * option.implied_volatility * time_to_expiry.sqrt())
}

#[allow(dead_code)]
pub fn theta(option: &Options, time_to_expiry: f64) -> f64 {
    let d1 = d1(
        option.underlying_price,
        option.risk_free_rate,
        time_to_expiry,
        option.implied_volatility,
    );
    let d2 = d2(
        option.underlying_price,
        option.risk_free_rate,
        time_to_expiry,
        option.implied_volatility,
    );

    let common_term = -option.underlying_price
        * option.implied_volatility
        * (-option.dividend_yield * time_to_expiry).exp()
        * n(d1)
        / (2.0 * time_to_expiry.sqrt());

    match option.option_style {
        OptionStyle::Call => {
            common_term
                - option.risk_free_rate
                    * option.strike_price
                    * (-option.risk_free_rate * time_to_expiry).exp()
                    * big_n(d2)
                + option.dividend_yield
                    * option.underlying_price
                    * (-option.dividend_yield * time_to_expiry).exp()
                    * big_n(d1)
        }
        OptionStyle::Put => {
            common_term
                + option.risk_free_rate
                    * option.strike_price
                    * (-option.risk_free_rate * time_to_expiry).exp()
                    * big_n(-d2)
                - option.dividend_yield
                    * option.underlying_price
                    * (-option.dividend_yield * time_to_expiry).exp()
                    * big_n(-d1)
        }
    }
}

#[allow(dead_code)]
pub fn vega(option: &Options, time_to_expiry: f64) -> f64 {
    let d1 = d1(
        option.underlying_price,
        option.risk_free_rate,
        time_to_expiry,
        option.implied_volatility,
    );

    option.underlying_price
        * (-option.dividend_yield * time_to_expiry).exp()
        * n(d1)
        * time_to_expiry.sqrt()
}

#[allow(dead_code)]
pub fn rho(option: &Options, time_to_expiry: f64) -> f64 {
    let d2 = d2(
        option.underlying_price,
        option.risk_free_rate,
        time_to_expiry,
        option.implied_volatility,
    );

    match option.option_style {
        OptionStyle::Call => {
            option.strike_price
                * time_to_expiry
                * (-option.risk_free_rate * time_to_expiry).exp()
                * big_n(d2)
        }
        OptionStyle::Put => {
            -option.strike_price
                * time_to_expiry
                * (-option.risk_free_rate * time_to_expiry).exp()
                * big_n(-d2)
        }
    }
}

#[allow(dead_code)]
pub fn rho_d(option: &Options, time_to_expiry: f64) -> f64 {
    let d1 = d1(
        option.underlying_price,
        option.risk_free_rate,
        time_to_expiry,
        option.implied_volatility,
    );

    match option.option_style {
        OptionStyle::Call => {
            -time_to_expiry
                * option.underlying_price
                * (-option.dividend_yield * time_to_expiry).exp()
                * big_n(d1)
        }
        OptionStyle::Put => {
            time_to_expiry
                * option.underlying_price
                * (-option.dividend_yield * time_to_expiry).exp()
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
        }
    }

    #[test]
    fn test_delta() {
        let call_option = create_test_option(OptionStyle::Call);
        let put_option = create_test_option(OptionStyle::Put);
        let time_to_expiry = 1.0;

        assert_relative_eq!(delta(&call_option, time_to_expiry), 0.9801, epsilon = 1e-4);
        assert_relative_eq!(
            delta(&put_option, time_to_expiry),
            -1.0639e-5,
            epsilon = 1e-6
        );
    }

    #[test]
    fn test_gamma() {
        let option = create_test_option(OptionStyle::Call); // Gamma is the same for calls and puts
        let time_to_expiry = 1.0;

        assert_relative_eq!(
            gamma(&option, time_to_expiry),
            6.59222905750737e-8,
            epsilon = 1e-8
        );
    }

    #[test]
    fn test_theta() {
        let call_option = create_test_option(OptionStyle::Call);
        let put_option = create_test_option(OptionStyle::Put);
        let time_to_expiry = 1.0;

        assert_relative_eq!(theta(&call_option, time_to_expiry), 1.9358, epsilon = 1e-4);
        assert_relative_eq!(theta(&put_option, time_to_expiry), 4.7315, epsilon = 1e-4);
    }

    #[test]
    fn test_vega() {
        let option = create_test_option(OptionStyle::Call); // Vega is the same for calls and puts
        let time_to_expiry = 1.0;

        assert_relative_eq!(vega(&option, time_to_expiry), 0.0047, epsilon = 1e-4);
    }

    #[test]
    fn test_rho() {
        let call_option = create_test_option(OptionStyle::Call);
        let put_option = create_test_option(OptionStyle::Put);
        let time_to_expiry = 1.0;

        assert_relative_eq!(rho(&call_option, time_to_expiry), 0.14945, epsilon = 1e-4);
        assert_relative_eq!(rho(&put_option, time_to_expiry), -94.9734, epsilon = 1e-4);
    }

    #[test]
    fn test_rho_d() {
        let call_option = create_test_option(OptionStyle::Call);
        let put_option = create_test_option(OptionStyle::Put);
        let time_to_expiry = 1.0;

        assert_relative_eq!(
            rho_d(&call_option, time_to_expiry),
            -98.01880,
            epsilon = 1e-4
        );
        assert_relative_eq!(rho_d(&put_option, time_to_expiry), 0.0010, epsilon = 1e-4);
    }
}
