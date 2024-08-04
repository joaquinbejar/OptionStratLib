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

/// Calculates the price of an option using the binomial model.
///
/// This function implements the binomial model for option pricing,
/// which is a numerical method for estimating the price of both European and American options.
/// The model constructs a binomial tree of possible future underlying asset prices
/// and then recursively calculates the option value from the leaves to the root of the tree.
///
/// # Arguments
///
/// * `params` - A `BinomialPricingParams` struct containing all necessary pricing parameters:
///     - `asset`: Current price of the underlying asset.
///     - `volatility`: Annualized volatility of the underlying asset.
///     - `int_rate`: Annualized risk-free interest rate.
///     - `strike`: Strike price of the option.
///     - `expiry`: Time to expiration in years.
///     - `no_steps`: Number of steps in the binomial tree.
///     - `option_type`: Type of option (e.g., European, American).
///     - `option_style`: Style of the option (Call or Put).
///     - `side`: Side of the trade (Long or Short).
///
/// # Returns
///
/// Returns the calculated price of the option as an `f64`.
///
/// # Special cases
///
/// - If `expiry` is 0, the function returns the intrinsic value of the option.
/// - If `volatility` is 0, the function calculates the option price deterministically.
///
/// # Example
///
/// ```
/// use optionstratlib::model::types::{OptionType, OptionStyle, Side};
/// use optionstratlib::pricing::binomial_model::{price_binomial, BinomialPricingParams};
///
/// let params = BinomialPricingParams {
///     asset: 100.0,
///     volatility: 0.2,
///     int_rate: 0.05,
///     strike: 100.0,
///     expiry: 1.0,
///     no_steps: 100,
///     option_type: &OptionType::European,
///     option_style: &OptionStyle::Call,
///     side: &Side::Long,
/// };
///
/// let price = price_binomial(params);
/// println!("The option price is: {}", price);
/// ```
///
/// # Notes
///
/// - The model's accuracy increases with the number of steps, but so does the computation time.
/// - This model assumes that the underlying asset follows a multiplicative binomial process.
/// - For American options, this model accounts for the possibility of early exercise.
pub fn price_binomial(params: BinomialPricingParams) -> f64 {
    if params.expiry == 0.0 {
        let payoff = params
            .option_type
            .payoff(params.asset, params.strike, params.option_style);
        return match params.side {
            Side::Long => payoff,
            Side::Short => -payoff,
        };
    }

    if params.volatility == 0.0 {
        let future_asset_price = params.asset * (params.int_rate * params.expiry).exp();
        let discounted_payoff = (-params.int_rate * params.expiry).exp()
            * params
                .option_type
                .payoff(future_asset_price, params.strike, params.option_style);
        return match params.side {
            Side::Long => discounted_payoff,
            Side::Short => -discounted_payoff,
        };
    }

    let dt = params.expiry / params.no_steps as f64;
    let u = (params.volatility * dt.sqrt()).exp();
    let d = 1.0 / u;
    let p = (((params.int_rate * dt).exp() - d) / (u - d)).clamp(0.0, 1.0);

    let mut prices: Vec<f64> = (0..=params.no_steps)
        .map(|i| {
            let price = params.asset * u.powi(i as i32) * d.powi((params.no_steps - i) as i32);
            params
                .option_type
                .payoff(price, params.strike, params.option_style)
        })
        .collect();

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

        let exact_price =
            (asset * (int_rate * expiry).exp() - strike).max(0.0) * (-int_rate * expiry).exp();

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
