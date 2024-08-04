use crate::model::types::{OptionType, Side};
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
    pub side: &'a Side,
}

pub fn price_binomial(params: BinomialPricingParams) -> f64 {
    if params.expiry == 0.0 {
        return params.option_type.payoff(params.asset, params.strike);
    }

    let dt = params.expiry / params.no_steps as f64;
    let u = (params.volatility * dt.sqrt()).exp();
    let d = 1.0 / u;
    let p = (((params.int_rate * dt).exp() - d) / (u - d)).clamp(0.0, 1.0);

    let mut prices = vec![0.0; params.no_steps + 1];
    for i in 0..=params.no_steps {
        let price = params.asset * u.powi(i as i32) * d.powi((params.no_steps - i) as i32);
        prices[i] = params.option_type.payoff(price, params.strike);
    }

    for step in (0..params.no_steps).rev() {
        for i in 0..=step {
            prices[i] = (p * prices[i + 1] + (1.0 - p) * prices[i]) * (-params.int_rate * dt).exp();
        }
    }

    let price = prices[0];
    match params.side {
        Side::Long => price,
        Side::Short => -price,
    }
}

#[cfg(test)]
mod tests_price_binomial {
    use super::*;
    use crate::model::types::OptionType;
    use approx::assert_relative_eq;

    // #[test]
    // fn test_european_call_option() {
    //     let params = BinomialPricingParams {
    //         asset: 100.0,
    //         volatility: 0.2,
    //         int_rate: 0.05,
    //         strike: 100.0,
    //         expiry: 1.0,
    //         no_steps: 1000,
    //         option_type: &OptionType::European,
    //         side: &Side::Long,
    //     };
    //
    //     let price = price_binomial(params);
    //     assert_relative_eq!(price, 10.45, epsilon = 0.1);
    // }

    // #[test]
    // fn test_european_put_option() {
    //     let params = BinomialPricingParams {
    //         asset: 100.0,
    //         volatility: 0.2,
    //         int_rate: 0.05,
    //         strike: 100.0,
    //         expiry: 1.0,
    //         no_steps: 1000,
    //         option_type: &OptionType::European,
    //         side: &Side::Long,
    //     };
    //
    //     let price = price_binomial(params);
    //     assert_relative_eq!(price, 5.57, epsilon = 0.1);
    // }

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
            side: &Side::Long,
        };

        let long_price = price_binomial(params.clone());
        let short_price = price_binomial(BinomialPricingParams {
            side: &Side::Short,
            ..params
        });
        assert_relative_eq!(long_price, -short_price);
    }

    // #[test]
    // fn test_zero_volatility() {
    //     let params = BinomialPricingParams {
    //         asset: 100.0,
    //         volatility: 0.0,
    //         int_rate: 0.05,
    //         strike: 100.0,
    //         expiry: 1.0,
    //         no_steps: 1000,
    //         option_type: &OptionType::European,
    //         side: &Side::Long,
    //     };
    //
    //     let price = price_binomial(params);
    //     assert_relative_eq!(price, 5.0, epsilon = 0.01);
    // }

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
            side: &Side::Long,
        };

        let price = price_binomial(params);
        assert_relative_eq!(price, 0.0, epsilon = 0.01);
    }
}
