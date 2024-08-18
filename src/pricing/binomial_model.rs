use crate::model::types::{OptionStyle, OptionType, Side};
use crate::pricing::payoff::{Payoff, PayoffInfo};
use crate::pricing::utils::{
    calculate_discount_factor, calculate_discounted_payoff, calculate_down_factor,
    calculate_option_price, calculate_probability, calculate_up_factor, option_node_value,
    option_node_value_wrapper,
};

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
    let mut info = PayoffInfo {
        spot: params.asset,
        strike: params.strike,
        style: params.option_style.clone(),
        side: params.side.clone(),
        spot_prices: None,
        spot_min: None,
        spot_max: None,
    };

    if params.expiry == 0.0 {
        return params.option_type.payoff(&info);
    }
    if params.volatility == 0.0 {
        return calculate_discounted_payoff(params);
    }

    let dt = params.expiry / params.no_steps as f64;
    let u = calculate_up_factor(params.volatility, dt);
    let d = calculate_down_factor(params.volatility, dt);
    let p = calculate_probability(params.int_rate, dt, d, u);
    let discount_factor = calculate_discount_factor(params.int_rate, dt);

    let mut prices: Vec<f64> = (0..=params.no_steps)
        .map(|i| calculate_option_price(params.clone(), u, d, i))
        .collect();

    for step in (0..params.no_steps).rev() {
        for i in 0..=step {
            let option_value = option_node_value(p, prices[i + 1], prices[i], discount_factor);
            match params.option_type {
                OptionType::American => {
                    let spot = params.asset * u.powi(i as i32) * d.powi((step - i) as i32);
                    info.spot = spot;
                    let intrinsic_value = params.option_type.payoff(&info);
                    prices[i] = option_value.max(intrinsic_value);
                }
                OptionType::European => {
                    prices[i] = option_value;
                }
                _ => {
                    panic!("OptionType not implemented.")
                }
            }
        }
    }

    match params.side {
        Side::Long => prices[0],
        Side::Short => -prices[0],
    }
}

/// Generates a binomial tree for option pricing.
///
/// # Parameters
///
/// * `params`: A reference to `BinomialPricingParams` which contains the parameters required for
///   generating the binomial tree including expiration time, number of steps, volatility, interest rate,
///   asset price, strike price, option type, and option style.
///
/// # Returns
///
/// A tuple containing two vectors of vectors:
/// * `asset_tree`: The tree representing the possible future values of the asset at each step.
/// * `option_tree`: The tree representing the values of the option at each step.
///
/// The `generate_binomial_tree` function calculates the possible asset prices and option prices
/// at each node in a binomial tree based on the input parameters.
///
/// 1. It calculates the time interval `dt` for each step.
/// 2. `u` and `d` are the factors by which the price increases or decreases.
/// 3. `p` is the risk-neutral probability.
/// 4. It initializes the `asset_tree` and `option_tree` with the appropriate dimensions.
/// 5. The asset prices are computed for all nodes.
/// 6. The option values are computed at maturity based on the payoff function.
/// 7. The option values are then back-propagated to compute the option value at the current time.
///
/// # Example
///
/// ```rust
/// use optionstratlib::model::types::{OptionStyle, OptionType, Side};
/// use optionstratlib::pricing::binomial_model::{BinomialPricingParams, generate_binomial_tree};
/// let params = BinomialPricingParams {
///             asset: 100.0,
///             volatility: 0.2,
///             int_rate: 0.05,
///             strike: 100.0,
///             expiry: 1.0,
///             no_steps: 1000,
///             option_type: &OptionType::European,
///             option_style: &OptionStyle::Call,
///             side: &Side::Long,
///         };
/// let (asset_tree, option_tree) = generate_binomial_tree(&params);
/// ```
pub fn generate_binomial_tree(params: &BinomialPricingParams) -> (Vec<Vec<f64>>, Vec<Vec<f64>>) {
    let mut info = PayoffInfo {
        spot: params.asset,
        strike: params.strike,
        style: params.option_style.clone(),
        side: params.side.clone(),
        spot_prices: None,
        spot_min: None,
        spot_max: None,
    };

    let dt = params.expiry / params.no_steps as f64;
    let up_factor = calculate_up_factor(params.volatility, dt);
    let down_factor = calculate_down_factor(params.volatility, dt);
    let probability = calculate_probability(params.int_rate, dt, down_factor, up_factor);
    let discount_factor = calculate_discount_factor(params.int_rate, dt);

    let mut asset_tree = vec![vec![0.0; params.no_steps + 1]; params.no_steps + 1];
    let mut option_tree = vec![vec![0.0; params.no_steps + 1]; params.no_steps + 1];

    for (step, step_vec) in asset_tree.iter_mut().enumerate() {
        for (node, node_val) in step_vec.iter_mut().enumerate().take(step + 1) {
            *node_val =
                params.asset * up_factor.powi((step - node) as i32) * down_factor.powi(node as i32);
        }
    }

    for (node, node_val) in asset_tree[params.no_steps]
        .iter()
        .enumerate()
        .take(params.no_steps + 1)
    {
        info.spot = *node_val;
        option_tree[params.no_steps][node] = params.option_type.payoff(&info);
    }

    for step in (0..params.no_steps).rev() {
        let (current_step_arr, next_step_arr) = option_tree.split_at_mut(step + 1);
        for (node_idx, node_val) in current_step_arr[step].iter_mut().enumerate().take(step + 1) {
            let node_value =
                option_node_value_wrapper(probability, next_step_arr, node_idx, discount_factor);
            match params.option_type {
                OptionType::European => {
                    *node_val = node_value;
                }
                OptionType::American => {
                    if (step == 0) & (node_idx == 0) {
                        *node_val = node_value;
                    } else {
                        info.spot = asset_tree[step][node_idx];
                        let intrinsic_value = params.option_type.payoff(&info);
                        *node_val = intrinsic_value.max(node_value);
                    }
                }
                _ => {
                    panic!("OptionType not implemented.")
                }
            }
        }
    }

    (asset_tree, option_tree)
}

#[cfg(test)]
mod tests_price_binomial {
    use super::*;
    use crate::model::types::OptionType;
    use approx::assert_relative_eq;
    use crate::constants::ZERO;

    #[test]
    fn test_european_call_option() {
        let params = BinomialPricingParams {
            asset: 100.0,
            strike: 100.0,
            int_rate: 0.05,
            volatility: 0.2,
            expiry: 1.0,
            no_steps: 3,
            option_type: &OptionType::European,
            option_style: &OptionStyle::Call,
            side: &Side::Long,
        };

        let price = price_binomial(params);
        assert_relative_eq!(price, 11.043, epsilon = 0.001);
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
    fn test_european_put_option_extended() {
        let params = BinomialPricingParams {
            asset: 50.0,
            volatility: 0.2,
            int_rate: 0.05,
            strike: 52.0,
            expiry: 1.0,
            no_steps: 1,
            option_type: &OptionType::European,
            option_style: &OptionStyle::Put,
            side: &Side::Long,
        };

        let price = price_binomial(params);
        assert_relative_eq!(price, 4.446, epsilon = 0.001);
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
        assert_relative_eq!(long_price, short_price);
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
            (asset * (int_rate * expiry).exp() - strike).max(ZERO) * (-int_rate * expiry).exp();

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

#[cfg(test)]
mod tests_generate_binomial_tree {
    use super::*;
    use crate::model::types::OptionType;
    use approx::assert_relative_eq;

    #[test]
    fn test_binomial_tree_basic() {
        let params = BinomialPricingParams {
            asset: 100.0,
            strike: 100.0,
            int_rate: 0.05,
            volatility: 0.2,
            expiry: 1.0,
            no_steps: 3,
            option_type: &OptionType::European,
            option_style: &OptionStyle::Call,
            side: &Side::Long,
        };

        let (asset_tree, option_tree) = generate_binomial_tree(&params);

        // Check if the asset tree is generated correctly
        assert_eq!(asset_tree[0][0], 100.0);
        assert_relative_eq!(asset_tree[1][0], 112.240, epsilon = 0.001);
        assert_relative_eq!(asset_tree[3][1], 112.240, epsilon = 0.001);

        assert_relative_eq!(option_tree[0][0], 11.043, epsilon = 0.001);
        assert_relative_eq!(option_tree[1][0], 17.713, epsilon = 0.001);
        assert_relative_eq!(option_tree[1][1], 3.500, epsilon = 0.001);
        assert_relative_eq!(option_tree[2][0], 27.631, epsilon = 0.001);
        assert_relative_eq!(option_tree[2][1], 6.545, epsilon = 0.001);
        assert_relative_eq!(option_tree[2][2], 0.0, epsilon = 0.001);
        assert_relative_eq!(option_tree[3][0], 41.398, epsilon = 0.001);
        assert_relative_eq!(option_tree[3][1], 12.240, epsilon = 0.001);
        assert_relative_eq!(option_tree[3][2], 0.0, epsilon = 0.001);
        assert_relative_eq!(option_tree[3][3], 0.0, epsilon = 0.001);
    }

    #[test]
    fn test_binomial_tree_put_option() {
        let params = BinomialPricingParams {
            asset: 100.0,
            strike: 100.0,
            int_rate: 0.05,
            volatility: 0.2,
            expiry: 1.0,
            no_steps: 3,
            option_type: &OptionType::European,
            option_style: &OptionStyle::Put,
            side: &Side::Long,
        };

        let (_, option_tree) = generate_binomial_tree(&params);

        assert_relative_eq!(option_tree[3][0], 0.0, epsilon = 0.001);
        assert_relative_eq!(option_tree[3][1], 0.0, epsilon = 0.001);
        assert_relative_eq!(option_tree[3][2], 10.905, epsilon = 0.001);
        assert_relative_eq!(option_tree[3][3], 29.277, epsilon = 0.001);
    }

    #[test]
    fn test_binomial_tree_call_option_check() {
        let params = BinomialPricingParams {
            asset: 30.0,
            strike: 30.0,
            expiry: 1.0,
            int_rate: 0.05,
            volatility: 0.17,
            no_steps: 1,
            option_type: &OptionType::European,
            option_style: &OptionStyle::Call,
            side: &Side::Long,
        };

        let (asset_tree, option_tree) = generate_binomial_tree(&params);

        // Test asset tree
        assert_eq!(asset_tree.len(), 2);
        assert_relative_eq!(asset_tree[0][0], 30.0, epsilon = 0.001);
        assert_relative_eq!(asset_tree[1][0], 35.559, epsilon = 0.001);
        assert_relative_eq!(asset_tree[1][1], 25.309, epsilon = 0.001);
        assert_relative_eq!(option_tree[0][0], 3.213, epsilon = 0.001);
        assert_relative_eq!(option_tree[1][0], 5.559, epsilon = 0.001);
        assert_relative_eq!(option_tree[1][1], 0.0, epsilon = 0.001);

        let params = BinomialPricingParams {
            asset: 30.0,
            strike: 30.0,
            expiry: 1.0,
            int_rate: 0.05,
            volatility: 0.17,
            no_steps: 2,
            option_type: &OptionType::European,
            option_style: &OptionStyle::Call,
            side: &Side::Long,
        };

        let (asset_tree, option_tree) = generate_binomial_tree(&params);

        // Test asset tree
        assert_eq!(asset_tree.len(), 3);
        assert_relative_eq!(asset_tree[0][0], 30.0, epsilon = 0.001);
        assert_relative_eq!(asset_tree[1][0], 33.831, epsilon = 0.001);
        assert_relative_eq!(asset_tree[1][1], 26.602, epsilon = 0.001);
        assert_relative_eq!(asset_tree[2][0], 38.153, epsilon = 0.001);
        assert_relative_eq!(asset_tree[2][1], 30.0, epsilon = 0.001);
        assert_relative_eq!(asset_tree[2][2], 23.589, epsilon = 0.001);

        assert_relative_eq!(option_tree[0][0], 2.564, epsilon = 0.001);
        assert_relative_eq!(option_tree[1][0], 4.572, epsilon = 0.001);
        assert_relative_eq!(option_tree[1][1], 0.0, epsilon = 0.001);
        assert_relative_eq!(option_tree[2][0], 8.153, epsilon = 0.001);
        assert_relative_eq!(option_tree[2][1], 0.0, epsilon = 0.001);
        assert_relative_eq!(option_tree[2][2], 0.0, epsilon = 0.001);
    }

    #[test]
    fn test_binomial_tree_put_option_check() {
        let params = BinomialPricingParams {
            asset: 100.0,
            strike: 110.0,
            expiry: 3.0, // Assuming each time step is 1 unit of time
            int_rate: 0.05,
            volatility: 0.09531018, // Calculated to match the 10% up/down movement
            no_steps: 3,
            option_type: &OptionType::European,
            option_style: &OptionStyle::Put,
            side: &Side::Long,
        };

        let (asset_tree, option_tree) = generate_binomial_tree(&params);

        // Test asset tree
        assert_eq!(asset_tree.len(), 4);
        assert_relative_eq!(asset_tree[0][0], 100.0, epsilon = 0.001);
        assert_relative_eq!(asset_tree[1][0], 110.0, epsilon = 0.0000001);
        assert_relative_eq!(asset_tree[1][1], 90.909, epsilon = 0.001);
        assert_relative_eq!(asset_tree[2][0], 121.0, epsilon = 0.001);
        assert_relative_eq!(asset_tree[2][1], 100.0, epsilon = 0.001);
        assert_relative_eq!(asset_tree[2][2], 82.644, epsilon = 0.001);
        assert_relative_eq!(asset_tree[3][0], 133.1, epsilon = 0.001);
        assert_relative_eq!(asset_tree[3][1], 110.0, epsilon = 0.001);
        assert_relative_eq!(asset_tree[3][2], 90.909, epsilon = 0.001);
        assert_relative_eq!(asset_tree[3][3], 75.131, epsilon = 0.001);

        assert_relative_eq!(option_tree[0][0], 2.890, epsilon = 0.001);
        assert_relative_eq!(option_tree[1][0], 1.125, epsilon = 0.001);
        assert_relative_eq!(option_tree[1][1], 8.623, epsilon = 0.001);
        assert_relative_eq!(option_tree[2][0], 0.0, epsilon = 0.001);
        assert_relative_eq!(option_tree[2][1], 4.635, epsilon = 0.001);
        assert_relative_eq!(option_tree[2][2], 21.990, epsilon = 0.001);
        assert_relative_eq!(option_tree[3][0], 0.0, epsilon = 0.001);
        assert_relative_eq!(option_tree[3][1], 0.0, epsilon = 0.001);
        assert_relative_eq!(option_tree[3][2], 19.090, epsilon = 0.001);
        assert_relative_eq!(option_tree[3][3], 34.868, epsilon = 0.001);
    }

    #[test]
    fn test_binomial_tree_european_put_option() {
        // Define parameters for an American option test case
        let params = BinomialPricingParams {
            asset: 50.0,
            volatility: 0.2,
            int_rate: 0.05,
            strike: 52.0,
            expiry: 2.0,
            no_steps: 2,
            option_type: &OptionType::European,
            option_style: &OptionStyle::Put,
            side: &Side::Long,
        };

        let (asset_tree, option_tree) = generate_binomial_tree(&params);

        assert_relative_eq!(asset_tree[0][0], 50.0, epsilon = 0.001);
        assert_relative_eq!(asset_tree[1][0], 61.070, epsilon = 0.001);
        assert_relative_eq!(asset_tree[1][1], 40.936, epsilon = 0.001);
        assert_relative_eq!(asset_tree[2][0], 74.591, epsilon = 0.001);
        assert_relative_eq!(asset_tree[2][1], 50.0, epsilon = 0.001);
        assert_relative_eq!(asset_tree[2][2], 33.516, epsilon = 0.001);

        assert_relative_eq!(option_tree[0][0], 3.868, epsilon = 0.001);
        assert_relative_eq!(option_tree[1][0], 0.803, epsilon = 0.001);
        assert_relative_eq!(option_tree[1][1], 8.527, epsilon = 0.001);
        assert_relative_eq!(option_tree[2][0], 0.0, epsilon = 0.001);
        assert_relative_eq!(option_tree[2][1], 2.0, epsilon = 0.001);
        assert_relative_eq!(option_tree[2][2], 18.483, epsilon = 0.001);
    }

    #[test]
    fn test_binomial_tree_american_put_option() {
        // Define parameters for an American option test case
        let params = BinomialPricingParams {
            asset: 50.0,
            volatility: 0.2,
            int_rate: 0.05,
            strike: 52.0,
            expiry: 2.0,
            no_steps: 2,
            option_type: &OptionType::American,
            option_style: &OptionStyle::Put,
            side: &Side::Long,
        };
        let (asset_tree, option_tree) = generate_binomial_tree(&params);

        assert_relative_eq!(asset_tree[0][0], 50.0, epsilon = 0.001);
        assert_relative_eq!(asset_tree[1][0], 61.070, epsilon = 0.001);
        assert_relative_eq!(asset_tree[1][1], 40.936, epsilon = 0.001);
        assert_relative_eq!(asset_tree[2][0], 74.591, epsilon = 0.001);
        assert_relative_eq!(asset_tree[2][1], 50.0, epsilon = 0.001);
        assert_relative_eq!(asset_tree[2][2], 33.516, epsilon = 0.001);

        assert_relative_eq!(option_tree[2][0], 0.0, epsilon = 0.001);
        assert_relative_eq!(option_tree[2][1], 2.0, epsilon = 0.001);
        assert_relative_eq!(option_tree[2][2], 18.483, epsilon = 0.001);
        assert_relative_eq!(option_tree[1][0], 0.803, epsilon = 0.001);

        assert_relative_eq!(
            option_tree[1][1],
            params.strike - asset_tree[1][1],
            epsilon = 0.001
        );
        assert_relative_eq!(option_tree[0][0], 4.887, epsilon = 0.001);
    }
}
