use crate::model::types::{OptionStyle, OptionType, Side};
use crate::pricing::payoff::Payoff;

#[derive(Clone)]
pub struct BinomialPricingParams<'a> {
    pub asset: f64,
    pub volatility: f64,
    pub int_rate: f64,
    pub strike: f64,
    pub expiry: f64,
    pub no_steps: usize,
    pub option_type: &'a OptionType,
    pub option_style: &'a OptionStyle,
    pub side: &'a Side,
}

// pub fn price_binomial(params: BinomialPricingParams) -> f64 {
//     if params.expiry == 0.0 {
//         let payoff = params.option_type.payoff(params.asset, params.strike, &params.option_style);
//         return match params.side {
//             Side::Long => payoff,
//             Side::Short => -payoff,
//         };
//     }
//
//     let dt = params.expiry / params.no_steps as f64;
//     let u = (params.volatility * dt.sqrt()).exp();
//     let d = 1.0 / u;
//     let p = (((params.int_rate * dt).exp() - d) / (u - d)).max(0.0).min(1.0);
//
//     let mut prices = vec![0.0; params.no_steps + 1];
//     for i in 0..=params.no_steps {
//         let price = params.asset * u.powi(i as i32) * d.powi((params.no_steps - i) as i32);
//         prices[i] = params.option_type.payoff(price, params.strike, &params.option_style);
//     }
//
//     for step in (0..params.no_steps).rev() {
//         for i in 0..=step {
//             prices[i] = (p * prices[i + 1] + (1.0 - p) * prices[i]) * (-params.int_rate * dt).exp();
//         }
//     }
//
//     match params.side {
//         Side::Long => prices[0],
//         Side::Short => -prices[0],
//     }
// }

pub fn price_binomial(params: BinomialPricingParams) -> f64 {
    if params.expiry == 0.0 {
        let payoff = params.option_type.payoff(params.asset, params.strike, &params.option_style);
        return match params.side {
            Side::Long => payoff,
            Side::Short => -payoff,
        };
    }

    if params.volatility == 0.0 {
        let future_asset_price = params.asset * (params.int_rate * params.expiry).exp();
        let discounted_payoff = (-params.int_rate * params.expiry).exp() *
            params.option_type.payoff(future_asset_price, params.strike, &params.option_style);
        return match params.side {
            Side::Long => discounted_payoff,
            Side::Short => -discounted_payoff,
        };
    }

    let dt = params.expiry / params.no_steps as f64;
    let u = (params.volatility * dt.sqrt()).exp();
    let d = 1.0 / u;
    let p = (((params.int_rate * dt).exp() - d) / (u - d)).max(0.0).min(1.0);

    let mut prices = vec![0.0; params.no_steps + 1];
    for i in 0..=params.no_steps {
        let price = params.asset * u.powi(i as i32) * d.powi((params.no_steps - i) as i32);
        prices[i] = params.option_type.payoff(price, params.strike, &params.option_style);
    }

    for step in (0..params.no_steps).rev() {
        for i in 0..=step {
            prices[i] = (p * prices[i + 1] + (1.0 - p) * prices[i]) * (-params.int_rate * dt).exp();
        }
    }

    match params.side {
        Side::Long => prices[0],
        Side::Short => -prices[0],
    }
}

#[cfg(test)]
mod tests_price_binomial {
    use super::*;
    use crate::model::types::OptionType;
    use approx::assert_relative_eq;

    #[test]
    fn test_european_call_option() {
        let params = BinomialPricingParams {
            asset: 100.0,
            volatility: 0.2,
            int_rate: 0.05,
            strike: 100.0,
            expiry: 1.0,
            no_steps: 1000,
            option_type: &OptionType::European,
            option_style: &OptionStyle::Call,
            side: &Side::Long,
        };

        let price = price_binomial(params);
        assert_relative_eq!(price, 10.45, epsilon = 0.1);
    }

    #[test]
    fn test_european_put_option() {
        let params = BinomialPricingParams {
            asset: 100.0,
            volatility: 0.2,
            int_rate: 0.05,
            strike: 100.0,
            expiry: 1.0,
            no_steps: 1000,
            option_type: &OptionType::European,
            option_style: &OptionStyle::Put,
            side: &Side::Long,
        };

        let price = price_binomial(params);
        assert_relative_eq!(price, 5.57, epsilon = 0.1);
    }

    #[test]
    fn test_short_option() {
        let params = BinomialPricingParams {
            asset: 100.0,
            volatility: 0.2,
            int_rate: 0.05,
            strike: 100.0,
            expiry: 1.0,
            no_steps: 1000,
            option_type: &OptionType::European,
            option_style: &OptionStyle::Call,
            side: &Side::Long,
        };

        let long_price = price_binomial(params.clone());
        let short_price = price_binomial(BinomialPricingParams {
            side: &Side::Short,
            ..params
        });
        assert_relative_eq!(long_price, -short_price);
    }

    #[test]
    fn test_zero_volatility() {
        let asset = 100.0;
        let strike = 100.0;
        let int_rate = 0.05;
        let expiry = 1.0;

        let params = BinomialPricingParams {
            asset,
            volatility: 0.0,
            int_rate,
            strike,
            expiry,
            no_steps: 1000,
            option_type: &OptionType::European,
            option_style: &OptionStyle::Call,
            side: &Side::Long,
        };

        let price = price_binomial(params);

        let exact_price = (asset * (int_rate * expiry).exp() - strike).max(0.0) * (-int_rate * expiry).exp();

        assert_relative_eq!(price, exact_price, epsilon = 1e-10);
    }

    #[test]
    fn test_deep_in_the_money() {
        let params = BinomialPricingParams {
            asset: 150.0,
            volatility: 0.2,
            int_rate: 0.05,
            strike: 100.0,
            expiry: 1.0,
            no_steps: 1000,
            option_type: &OptionType::European,
            option_style: &OptionStyle::Call,
            side: &Side::Long,
        };

        let price = price_binomial(params);
        assert!(price > 50.0);
    }

    #[test]
    fn test_deep_out_of_the_money() {
        let params = BinomialPricingParams {
            asset: 50.0,
            volatility: 0.2,
            int_rate: 0.05,
            strike: 100.0,
            expiry: 1.0,
            no_steps: 1000,
            option_type: &OptionType::European,
            option_style: &OptionStyle::Call,
            side: &Side::Long,
        };

        let price = price_binomial(params);
        assert!(price < 1.0);
    }

    #[test]
    fn test_zero_time_to_expiry() {
        let params = BinomialPricingParams {
            asset: 100.0,
            volatility: 0.2,
            int_rate: 0.05,
            strike: 100.0,
            expiry: 0.0,
            no_steps: 1000,
            option_type: &OptionType::European,
            option_style: &OptionStyle::Call,
            side: &Side::Long,
        };

        let price = price_binomial(params);
        assert_relative_eq!(price, 0.0, epsilon = 0.01);
    }
}
