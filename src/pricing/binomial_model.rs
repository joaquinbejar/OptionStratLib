use crate::model::types::{OptionStyle, OptionType, PositiveF64, Side};
use crate::pricing::payoff::{Payoff, PayoffInfo};
use crate::pricing::utils::{
    calculate_discount_factor, calculate_discounted_payoff, calculate_down_factor,
    calculate_option_price, calculate_probability, calculate_up_factor, option_node_value,
    option_node_value_wrapper,
};
use crate::{d2f, d2p, f2d};
use rust_decimal::{Decimal, MathematicalOps};
use std::error::Error;

type BinomialTreeResult = Result<(Vec<Vec<Decimal>>, Vec<Vec<Decimal>>), Box<dyn Error>>;

#[derive(Clone)]
pub struct BinomialPricingParams<'a> {
    pub asset: PositiveF64,
    pub volatility: Decimal,
    pub int_rate: Decimal,
    pub strike: PositiveF64,
    pub expiry: Decimal,
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
/// # Notes
///
/// - The model's accuracy increases with the number of steps, but so does the computation time.
/// - This model assumes that the underlying asset follows a multiplicative binomial process.
/// - For American options, this model accounts for the possibility of early exercise.
pub fn price_binomial(params: BinomialPricingParams) -> Result<Decimal, Box<dyn Error>> {
    let mut info = PayoffInfo {
        spot: params.asset,
        strike: params.strike,
        style: params.option_style.clone(),
        side: params.side.clone(),
        spot_prices: None,
        spot_min: None,
        spot_max: None,
    };

    if params.expiry == Decimal::ZERO {
        let intrinsic_value = f2d!(params.option_type.payoff(&info));
        return Ok(intrinsic_value);
    }
    if params.volatility == Decimal::ZERO {
        return Ok(calculate_discounted_payoff(params)?);
    }

    let dt = params.expiry / f2d!(params.no_steps as f64);
    let u = calculate_up_factor(params.volatility, dt)?;
    let d = calculate_down_factor(params.volatility, dt)?;
    let p = calculate_probability(params.int_rate, dt, d, u)?;
    let discount_factor = calculate_discount_factor(params.int_rate, dt)?;

    let mut prices: Vec<Decimal> = (0..=params.no_steps)
        .map(|i| calculate_option_price(params.clone(), u, d, i).unwrap())
        .collect();

    for step in (0..params.no_steps).rev() {
        for i in 0..=step {
            let option_value = option_node_value(p, prices[i + 1], prices[i], discount_factor)?;
            match params.option_type {
                OptionType::American => {
                    let spot = params.asset * u.powi(i as i64) * d.powi((step - i) as i64);
                    info.spot = spot;
                    let intrinsic_value = f2d!(params.option_type.payoff(&info));
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
        Side::Long => Ok(prices[0]),
        Side::Short => Ok(-prices[0]),
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
/// use rust_decimal::Decimal;
/// use rust_decimal_macros::dec;
/// use optionstratlib::model::types::{OptionStyle, OptionType, Side};
/// use optionstratlib::pos;
/// use optionstratlib::pricing::binomial_model::{BinomialPricingParams, generate_binomial_tree};
/// use optionstratlib::model::types::PositiveF64;
/// let params = BinomialPricingParams {
///             asset: pos!(100.0),
///             volatility: dec!(0.2),
///             int_rate: dec!(0.05),
///             strike: pos!(100.0),
///             expiry: Decimal::ONE,
///             no_steps: 1000,
///             option_type: &OptionType::European,
///             option_style: &OptionStyle::Call,
///             side: &Side::Long,
///         };
/// let (asset_tree, option_tree) = generate_binomial_tree(&params).unwrap();
/// ```
pub fn generate_binomial_tree(params: &BinomialPricingParams) -> BinomialTreeResult {
    let mut info = PayoffInfo {
        spot: params.asset,
        strike: params.strike,
        style: params.option_style.clone(),
        side: params.side.clone(),
        spot_prices: None,
        spot_min: None,
        spot_max: None,
    };

    let dt = params.expiry / f2d!(params.no_steps as f64);
    let up_factor = calculate_up_factor(params.volatility, dt)?;
    let down_factor = calculate_down_factor(params.volatility, dt)?;
    let probability = calculate_probability(params.int_rate, dt, down_factor, up_factor)?;
    let discount_factor = calculate_discount_factor(params.int_rate, dt)?;

    let mut asset_tree = vec![vec![Decimal::ZERO; params.no_steps + 1]; params.no_steps + 1];
    let mut option_tree = vec![vec![Decimal::ZERO; params.no_steps + 1]; params.no_steps + 1];

    for (step, step_vec) in asset_tree.iter_mut().enumerate() {
        for (node, node_val) in step_vec.iter_mut().enumerate().take(step + 1) {
            *node_val =
                up_factor.powi((step - node) as i64) * down_factor.powi(node as i64) * params.asset;
        }
    }

    for (node, node_val) in asset_tree[params.no_steps]
        .iter()
        .enumerate()
        .take(params.no_steps + 1)
    {
        info.spot = (*node_val).into();
        option_tree[params.no_steps][node] = f2d!(params.option_type.payoff(&info));
    }

    for step in (0..params.no_steps).rev() {
        let (current_step_arr, next_step_arr) = option_tree.split_at_mut(step + 1);
        for (node_idx, node_val) in current_step_arr[step].iter_mut().enumerate().take(step + 1) {
            let node_value =
                option_node_value_wrapper(probability, next_step_arr, node_idx, discount_factor)?;
            match params.option_type {
                OptionType::European => {
                    *node_val = node_value;
                }
                OptionType::American => {
                    if (step == 0) & (node_idx == 0) {
                        *node_val = node_value;
                    } else {
                        info.spot = d2p!(asset_tree[step][node_idx])?;
                        let intrinsic_value = params.option_type.payoff(&info);
                        let dec_node_val = d2f!(node_value);
                        *node_val = f2d!(intrinsic_value.max(dec_node_val));
                    }
                }
                _ => {
                    panic!("OptionType not implemented.")
                }
            }
        }
    }

    Ok((asset_tree, option_tree))
}

#[cfg(test)]
mod tests_price_binomial {
    use super::*;
    use crate::model::types::{OptionType, PZERO};
    use crate::{assert_decimal_eq, p2du, pos};
    use rust_decimal_macros::dec;

    const EPSILON: Decimal = dec!(1e-6);

    #[test]
    fn test_european_call_option() {
        let params = BinomialPricingParams {
            asset: pos!(100.0),
            strike: pos!(100.0),
            int_rate: dec!(0.05),
            volatility: dec!(0.2),
            expiry: Decimal::ONE,
            no_steps: 3,
            option_type: &OptionType::European,
            option_style: &OptionStyle::Call,
            side: &Side::Long,
        };

        let price = price_binomial(params).unwrap();
        assert_decimal_eq!(price, dec!(11.0438708), EPSILON);
    }

    #[test]
    fn test_european_put_option() {
        let params = BinomialPricingParams {
            asset: pos!(100.0),
            volatility: dec!(0.2),
            int_rate: dec!(0.05),
            strike: pos!(100.0),
            expiry: Decimal::ONE,
            no_steps: 1000,
            option_type: &OptionType::European,
            option_style: &OptionStyle::Put,
            side: &Side::Long,
        };

        let price = price_binomial(params).unwrap();
        assert_decimal_eq!(price, dec!(5.571526), EPSILON);
    }

    #[test]
    fn test_european_put_option_extended() {
        let params = BinomialPricingParams {
            asset: pos!(50.0),
            volatility: dec!(0.2),
            int_rate: dec!(0.05),
            strike: pos!(52.0),
            expiry: Decimal::ONE,
            no_steps: 1,
            option_type: &OptionType::European,
            option_style: &OptionStyle::Put,
            side: &Side::Long,
        };

        let price = price_binomial(params).unwrap();
        assert_decimal_eq!(price, dec!(4.446415), EPSILON);
    }

    #[test]
    fn test_short_option() {
        let params = BinomialPricingParams {
            asset: pos!(100.0),
            volatility: dec!(0.2),
            int_rate: dec!(0.05),
            strike: pos!(100.0),
            expiry: Decimal::ONE,
            no_steps: 1000,
            option_type: &OptionType::European,
            option_style: &OptionStyle::Call,
            side: &Side::Long,
        };

        let long_price = price_binomial(params.clone()).unwrap();
        let short_price = price_binomial(BinomialPricingParams {
            side: &Side::Short,
            ..params
        })
        .unwrap();
        assert_decimal_eq!(long_price, short_price, EPSILON);
    }

    #[test]
    fn test_zero_volatility() {
        let asset = pos!(100.0);
        let strike = pos!(100.0);
        let int_rate = dec!(0.05);
        let expiry = Decimal::ONE;

        let params = BinomialPricingParams {
            asset,
            volatility: Decimal::ZERO,
            int_rate,
            strike,
            expiry,
            no_steps: 1000,
            option_type: &OptionType::European,
            option_style: &OptionStyle::Call,
            side: &Side::Long,
        };

        let price = price_binomial(params).unwrap();

        let exact_price =
            (asset * (int_rate * expiry).exp() - strike).max(PZERO) * (-int_rate * expiry).exp();

        assert_decimal_eq!(price, p2du!(exact_price).unwrap(), EPSILON);
    }

    #[test]
    fn test_deep_in_the_money() {
        let params = BinomialPricingParams {
            asset: pos!(150.0),
            volatility: dec!(0.2),
            int_rate: dec!(0.05),
            strike: pos!(100.0),
            expiry: Decimal::ONE,
            no_steps: 1000,
            option_type: &OptionType::European,
            option_style: &OptionStyle::Call,
            side: &Side::Long,
        };

        let price = price_binomial(params).unwrap();
        assert!(price > dec!(50.0));
    }

    #[test]
    fn test_deep_out_of_the_money() {
        let params = BinomialPricingParams {
            asset: pos!(50.0),
            volatility: dec!(0.2),
            int_rate: dec!(0.05),
            strike: pos!(100.0),
            expiry: Decimal::ONE,
            no_steps: 1000,
            option_type: &OptionType::European,
            option_style: &OptionStyle::Call,
            side: &Side::Long,
        };

        let price = price_binomial(params).unwrap();
        assert!(price < Decimal::ONE);
    }

    #[test]
    fn test_zero_time_to_expiry() {
        let params = BinomialPricingParams {
            asset: pos!(100.0),
            volatility: dec!(0.2),
            int_rate: dec!(0.05),
            strike: pos!(100.0),
            expiry: Decimal::ZERO,
            no_steps: 1000,
            option_type: &OptionType::European,
            option_style: &OptionStyle::Call,
            side: &Side::Long,
        };

        let price = price_binomial(params).unwrap();
        assert_decimal_eq!(price, Decimal::ZERO, EPSILON);
    }
}

#[cfg(test)]
mod tests_generate_binomial_tree {
    use super::*;
    use crate::model::types::OptionType;
    use crate::{assert_decimal_eq, p2du, pos};
    use rust_decimal_macros::dec;

    const EPSILON: Decimal = dec!(1e-6);

    #[test]
    fn test_binomial_tree_basic() {
        let params = BinomialPricingParams {
            asset: pos!(100.0),
            strike: pos!(100.0),
            int_rate: dec!(0.05),
            volatility: dec!(0.2),
            expiry: Decimal::ONE,
            no_steps: 3,
            option_type: &OptionType::European,
            option_style: &OptionStyle::Call,
            side: &Side::Long,
        };

        let (asset_tree, option_tree) = generate_binomial_tree(&params).unwrap();

        // Check if the asset tree is generated correctly
        assert_eq!(asset_tree[0][0], dec!(100.0));
        assert_decimal_eq!(asset_tree[1][0], dec!(112.2400899), EPSILON);
        assert_decimal_eq!(asset_tree[3][1], dec!(112.2400899), EPSILON);
        assert_decimal_eq!(option_tree[0][0], dec!(11.0438708), EPSILON);
        assert_decimal_eq!(option_tree[1][0], dec!(17.713887), EPSILON);
        assert_decimal_eq!(option_tree[1][1], dec!(3.500653), EPSILON);
        assert_decimal_eq!(option_tree[2][0], dec!(27.631232), EPSILON);
        assert_decimal_eq!(option_tree[2][1], dec!(6.5458625), EPSILON);
        assert_decimal_eq!(option_tree[2][2], Decimal::ZERO, EPSILON);
        assert_decimal_eq!(option_tree[3][0], dec!(41.398244), EPSILON);
        assert_decimal_eq!(option_tree[3][1], dec!(12.240089), EPSILON);
        assert_decimal_eq!(option_tree[3][2], Decimal::ZERO, EPSILON);
        assert_decimal_eq!(option_tree[3][3], Decimal::ZERO, EPSILON);
    }

    #[test]
    fn test_binomial_tree_put_option() {
        let params = BinomialPricingParams {
            asset: pos!(100.0),
            strike: pos!(100.0),
            int_rate: dec!(0.05),
            volatility: dec!(0.2),
            expiry: Decimal::ONE,
            no_steps: 3,
            option_type: &OptionType::European,
            option_style: &OptionStyle::Put,
            side: &Side::Long,
        };

        let (_, option_tree) = generate_binomial_tree(&params).unwrap();

        assert_decimal_eq!(option_tree[3][0], Decimal::ZERO, EPSILON);
        assert_decimal_eq!(option_tree[3][1], Decimal::ZERO, EPSILON);
        assert_decimal_eq!(option_tree[3][2], dec!(10.905274), EPSILON);
        assert_decimal_eq!(option_tree[3][3], dec!(29.277764), EPSILON);
    }

    #[test]
    fn test_binomial_tree_call_option_check() {
        let params = BinomialPricingParams {
            asset: pos!(30.0),
            strike: pos!(30.0),
            expiry: Decimal::ONE,
            int_rate: dec!(0.05),
            volatility: dec!(0.17),
            no_steps: 1,
            option_type: &OptionType::European,
            option_style: &OptionStyle::Call,
            side: &Side::Long,
        };

        let (asset_tree, option_tree) = generate_binomial_tree(&params).unwrap();

        // Test asset tree
        assert_eq!(asset_tree.len(), 2);
        assert_decimal_eq!(asset_tree[0][0], dec!(30.0), EPSILON);
        assert_decimal_eq!(asset_tree[1][0], dec!(35.559145), EPSILON);
        assert_decimal_eq!(asset_tree[1][1], dec!(25.309944), EPSILON);
        assert_decimal_eq!(option_tree[0][0], dec!(3.213401), EPSILON);
        assert_decimal_eq!(option_tree[1][0], dec!(5.559145), EPSILON);
        assert_decimal_eq!(option_tree[1][1], Decimal::ZERO, EPSILON);

        let params = BinomialPricingParams {
            asset: pos!(30.0),
            strike: pos!(30.0),
            expiry: Decimal::ONE,
            int_rate: dec!(0.05),
            volatility: dec!(0.17),
            no_steps: 2,
            option_type: &OptionType::European,
            option_style: &OptionStyle::Call,
            side: &Side::Long,
        };

        let (asset_tree, option_tree) = generate_binomial_tree(&params).unwrap();

        // Test asset tree
        assert_eq!(asset_tree.len(), 3);
        assert_decimal_eq!(asset_tree[0][0], dec!(30.0), EPSILON);
        assert_decimal_eq!(asset_tree[1][0], dec!(33.831947), EPSILON);
        assert_decimal_eq!(asset_tree[1][1], dec!(26.602075), EPSILON);
        assert_decimal_eq!(asset_tree[2][0], dec!(38.153354), EPSILON);
        assert_decimal_eq!(asset_tree[2][1], dec!(30.0), EPSILON);
        assert_decimal_eq!(asset_tree[2][2], dec!(23.589013), EPSILON);

        assert_decimal_eq!(option_tree[0][0], dec!(2.564481), EPSILON);
        assert_decimal_eq!(option_tree[1][0], dec!(4.572649), EPSILON);
        assert_decimal_eq!(option_tree[1][1], Decimal::ZERO, EPSILON);
        assert_decimal_eq!(option_tree[2][0], dec!(8.153354), EPSILON);
        assert_decimal_eq!(option_tree[2][1], Decimal::ZERO, EPSILON);
        assert_decimal_eq!(option_tree[2][2], Decimal::ZERO, EPSILON);
    }

    #[test]
    fn test_binomial_tree_put_option_check() {
        let params = BinomialPricingParams {
            asset: pos!(100.0),
            strike: pos!(110.0),
            expiry: dec!(3.0), // Assuming each time step is 1 unit of time
            int_rate: dec!(0.05),
            volatility: dec!(0.09531018), // Calculated to match the 10% up/down movement
            no_steps: 3,
            option_type: &OptionType::European,
            option_style: &OptionStyle::Put,
            side: &Side::Long,
        };

        let (asset_tree, option_tree) = generate_binomial_tree(&params).unwrap();

        // Test asset tree
        assert_eq!(asset_tree.len(), 4);
        assert_decimal_eq!(asset_tree[0][0], dec!(100.0), EPSILON);
        assert_decimal_eq!(asset_tree[1][0], dec!(110.0), EPSILON);
        assert_decimal_eq!(asset_tree[1][1], dec!(90.909090), EPSILON);
        assert_decimal_eq!(asset_tree[2][0], dec!(121.0), EPSILON);
        assert_decimal_eq!(asset_tree[2][1], dec!(100.0), EPSILON);
        assert_decimal_eq!(asset_tree[2][2], dec!(82.644628), EPSILON);
        assert_decimal_eq!(asset_tree[3][0], dec!(133.1), EPSILON);
        assert_decimal_eq!(asset_tree[3][1], dec!(110.0), EPSILON);
        assert_decimal_eq!(asset_tree[3][2], dec!(90.909090), EPSILON);
        assert_decimal_eq!(asset_tree[3][3], dec!(75.131480), EPSILON);
        assert_decimal_eq!(option_tree[0][0], dec!(2.890941), EPSILON);
        assert_decimal_eq!(option_tree[1][0], dec!(1.125426), EPSILON);
        assert_decimal_eq!(option_tree[1][1], dec!(8.623025), EPSILON);
        assert_decimal_eq!(option_tree[2][0], Decimal::ZERO, EPSILON);
        assert_decimal_eq!(option_tree[2][1], dec!(4.635236), EPSILON);
        assert_decimal_eq!(option_tree[2][2], dec!(21.990608), EPSILON);
        assert_decimal_eq!(option_tree[3][0], Decimal::ZERO, EPSILON);
        assert_decimal_eq!(option_tree[3][1], Decimal::ZERO, EPSILON);
        assert_decimal_eq!(option_tree[3][2], dec!(19.090909), EPSILON);
        assert_decimal_eq!(option_tree[3][3], dec!(34.868519), EPSILON);
    }

    #[test]
    fn test_binomial_tree_european_put_option() {
        // Define parameters for an American option test case
        let params = BinomialPricingParams {
            asset: pos!(50.0),
            volatility: dec!(0.2),
            int_rate: dec!(0.05),
            strike: pos!(52.0),
            expiry: dec!(2.0),
            no_steps: 2,
            option_type: &OptionType::European,
            option_style: &OptionStyle::Put,
            side: &Side::Long,
        };

        let (asset_tree, option_tree) = generate_binomial_tree(&params).unwrap();

        assert_decimal_eq!(asset_tree[0][0], dec!(50.0), EPSILON);
        assert_decimal_eq!(asset_tree[1][0], dec!(61.070137), EPSILON);
        assert_decimal_eq!(asset_tree[1][1], dec!(40.936537), EPSILON);
        assert_decimal_eq!(asset_tree[2][0], dec!(74.591234), EPSILON);
        assert_decimal_eq!(asset_tree[2][1], dec!(50.0), EPSILON);
        assert_decimal_eq!(asset_tree[2][2], dec!(33.516002), EPSILON);
        assert_decimal_eq!(option_tree[0][0], dec!(3.8687179), EPSILON);
        assert_decimal_eq!(option_tree[1][0], dec!(0.8038018), EPSILON);
        assert_decimal_eq!(option_tree[1][1], dec!(8.5273923), EPSILON);
        assert_decimal_eq!(option_tree[2][0], Decimal::ZERO, EPSILON);
        assert_decimal_eq!(option_tree[2][1], dec!(2.0), EPSILON);
        assert_decimal_eq!(option_tree[2][2], dec!(18.483997), EPSILON);
    }

    #[test]
    fn test_binomial_tree_american_put_option() {
        // Define parameters for an American option test case
        let params = BinomialPricingParams {
            asset: pos!(50.0),
            volatility: dec!(0.2),
            int_rate: dec!(0.05),
            strike: pos!(52.0),
            expiry: dec!(2.0),
            no_steps: 2,
            option_type: &OptionType::American,
            option_style: &OptionStyle::Put,
            side: &Side::Long,
        };
        let (asset_tree, option_tree) = generate_binomial_tree(&params).unwrap();

        assert_decimal_eq!(asset_tree[0][0], dec!(50.0), EPSILON);
        assert_decimal_eq!(asset_tree[1][0], dec!(61.070137), EPSILON);
        assert_decimal_eq!(asset_tree[1][1], dec!(40.936537), EPSILON);
        assert_decimal_eq!(asset_tree[2][0], dec!(74.591234), EPSILON);
        assert_decimal_eq!(asset_tree[2][1], dec!(50.0), EPSILON);
        assert_decimal_eq!(asset_tree[2][2], dec!(33.516002), EPSILON);
        assert_decimal_eq!(option_tree[2][0], Decimal::ZERO, EPSILON);
        assert_decimal_eq!(option_tree[2][1], dec!(2.0), EPSILON);
        assert_decimal_eq!(option_tree[2][2], dec!(18.483997), EPSILON);
        assert_decimal_eq!(option_tree[1][0], dec!(0.803801), EPSILON);

        assert_decimal_eq!(
            option_tree[1][1],
            p2du!(params.strike).unwrap() - asset_tree[1][1],
            EPSILON
        );
        assert_decimal_eq!(option_tree[0][0], dec!(4.887966), EPSILON);
    }
}
