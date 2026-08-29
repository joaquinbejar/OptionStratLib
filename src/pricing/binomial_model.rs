// Scoped allow: bulk migration of unchecked `[]` indexing to
// `.get().ok_or_else(..)` tracked as follow-ups to #341. The existing
// call sites are internal to this file and audited for invariant-bound
// indices (fixed-length buffers, just-pushed slices, etc.).
#![allow(clippy::indexing_slicing)]

use crate::error::PricingError;
use crate::model::decimal::{d_div, d_mul, d_powd, d_sub};
use crate::model::types::{OptionStyle, OptionType, Side};
use crate::pricing::payoff::{Payoff, PayoffInfo};
use crate::pricing::utils::*;
use crate::{d2f, f2d};
use positive::Positive;
use rust_decimal::Decimal;
use std::num::NonZeroUsize;
use tracing::instrument;

#[cfg(test)]
use positive::pos_or_panic;

type BinomialTreeResult = Result<(Vec<Vec<Decimal>>, Vec<Vec<Decimal>>), PricingError>;

/// Parameters for pricing options using the Binomial Tree model.
///
/// This structure encapsulates all the necessary parameters required to calculate
/// the price of an option using the binomial pricing model. The binomial model is
/// a discrete-time, lattice-based approach to option pricing that can handle various
/// option types and styles.
///
/// The model builds a tree of possible future asset prices to determine the option's
/// value at each node, working backwards from expiration to the present value.
/// This approach is particularly valuable for American options or other early-exercise
/// scenarios.
#[derive(Debug, Clone)]
pub struct BinomialPricingParams<'a> {
    /// The current price of the underlying asset, represented as a positive value.
    pub asset: Positive,

    /// The volatility of the underlying asset, expressed as a positive value.
    /// This represents the standard deviation of the asset's returns.
    pub volatility: Positive,

    /// The risk-free interest rate used in the pricing model.
    pub int_rate: Decimal,

    /// The strike price of the option, represented as a positive value.
    pub strike: Positive,

    /// The time to expiration of the option in years, represented as a positive value.
    pub expiry: Positive,

    /// The number of steps to use in the binomial tree calculation,
    /// as a [`NonZeroUsize`] so zero is structurally invalid at the
    /// type level. Higher values increase accuracy but also
    /// computational cost. See
    /// [`crate::constants::DEFAULT_BINOMIAL_STEPS`] for a sensible
    /// default.
    pub no_steps: NonZeroUsize,

    /// The type of option (European, American, etc.) which determines
    /// when the option can be exercised.
    pub option_type: &'a OptionType,

    /// The style of the option (Call or Put) which determines whether the option
    /// gives the right to buy or sell the underlying asset.
    pub option_style: &'a OptionStyle,

    /// Indicates whether the option position is long (buying the option) or
    /// short (selling/writing the option).
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
/// - If `volatility` is 0, the underlying is deterministic and the price is
///   computed in closed form, which still honours early exercise: an American
///   is worth the better of exercising now and holding to expiry, a Bermuda
///   the best of its schedule, a European its discounted payoff.
///
/// # Notes
///
/// - The model's accuracy increases with the number of steps, but so does the computation time.
/// - This model assumes that the underlying asset follows a multiplicative binomial process.
/// - For American options, this model accounts for the possibility of early exercise.
///
/// # Errors
///
/// Returns [`PricingError::SqrtFailure`] when the up-factor exponent
/// produces an invalid `Decimal`, [`PricingError::BinomialNodeMissing`]
/// when the induction step cannot read an intermediate node, and
/// [`PricingError::Positive`] when any `Positive` construction
/// downstream (e.g. strike × discount factor) underflows below zero.
#[instrument(skip(params), fields(
    strike = %params.strike,
    asset = %params.asset,
    steps = params.no_steps.get(),
    style = ?*params.option_style,
    side = ?*params.side,
))]
pub fn price_binomial(params: BinomialPricingParams) -> Result<Decimal, PricingError> {
    let mut info = PayoffInfo {
        spot: params.asset,
        strike: params.strike,
        style: *params.option_style,
        side: *params.side,
        spot_prices: None,
        spot_min: None,
        spot_max: None,
    };

    if params.expiry == Decimal::ZERO {
        let intrinsic_value = f2d!(params.option_type.payoff(&info));
        return Ok(intrinsic_value);
    }
    if params.volatility == Decimal::ZERO {
        return price_deterministic(&params);
    }

    let no_steps_raw = params.no_steps.get();
    let dt = (params.expiry / Positive::new(no_steps_raw as f64)?).to_dec();
    let u = calculate_up_factor(params.volatility, dt)?;
    let d = calculate_down_factor(params.volatility, dt)?;
    if u == d {
        // `σ√dt` underflowed below the representable scale, so the lattice has
        // collapsed onto a single deterministic path: same answer as the
        // zero-volatility branch above.
        return price_deterministic(&params);
    }
    let p = calculate_probability(params.int_rate, dt, d, u)?;
    let discount_factor = calculate_discount_factor(params.int_rate, dt)?;

    let mut prices: Vec<Decimal> = (0..=no_steps_raw)
        .map(|i| calculate_option_price(params.clone(), u, d, i))
        .collect::<Result<Vec<_>, _>>()?;

    let half_dt = d_div(dt, Decimal::TWO, "pricing::binomial::half_dt")?;
    for step in (0..no_steps_raw).rev() {
        for i in 0..=step {
            let price_up = *prices
                .get(i + 1)
                .ok_or(PricingError::BinomialNodeMissing { node: "price_up" })?;
            let price_down = *prices
                .get(i)
                .ok_or(PricingError::BinomialNodeMissing { node: "price_down" })?;
            let option_value = option_node_value(p, price_up, price_down, discount_factor)?;
            let slot = prices
                .get_mut(i)
                .ok_or(PricingError::BinomialNodeMissing { node: "price_slot" })?;
            match params.option_type {
                OptionType::American => {
                    info.spot = lattice_spot(params.asset, u, d, i, step)?;
                    let intrinsic_value = f2d!(params.option_type.payoff(&info));
                    *slot = option_value.max(intrinsic_value);
                }
                OptionType::Bermuda { exercise_dates } => {
                    // Calculate time at this step
                    let time_at_step = d_mul(
                        dt,
                        Decimal::from(step as u64),
                        "pricing::binomial::time_at_step",
                    )?;
                    // Check if this step is an exercise date
                    let mut is_exercise_date = false;
                    for exercise in exercise_dates {
                        let gap = d_sub(
                            time_at_step,
                            exercise.to_dec(),
                            "pricing::binomial::exercise_gap",
                        )?;
                        if gap.abs() < half_dt {
                            is_exercise_date = true;
                            break;
                        }
                    }
                    if is_exercise_date {
                        let spot = lattice_spot(params.asset, u, d, i, step)?;
                        let slot_value = option_value;
                        info.spot = spot;
                        let intrinsic_value = f2d!(params.option_type.payoff(&info));
                        let slot = prices
                            .get_mut(i)
                            .ok_or(PricingError::BinomialNodeMissing { node: "price_slot" })?;
                        *slot = slot_value.max(intrinsic_value);
                    } else {
                        *slot = option_value;
                    }
                }
                OptionType::European => {
                    *slot = option_value;
                }
                _ => {
                    return Err(PricingError::other(
                        "OptionType not supported for binomial pricing",
                    ));
                }
            }
        }
    }
    prices
        .first()
        .copied()
        .ok_or(PricingError::BinomialNodeMissing { node: "root" })
}

/// Price of a contract whose underlying is deterministic.
///
/// With no volatility the underlying follows its forward, `S(t) = S · e^{r·t}`,
/// so exercising at `t` is worth `e^{-r·t} · payoff(S · e^{r·t})` today. Each
/// candidate exercise time is valued by [`calculate_discounted_payoff`] on a
/// copy of the parameters whose expiry is that time, which keeps the European
/// answer bit-identical to the one the direct call used to produce.
///
/// The exercise opportunities depend on the contract:
///
/// - **European** — expiry only.
/// - **American** — any `t ∈ [0, T]`. For a standard call or put the value
///   above is monotone in `t` (a call is worth `(S − K·e^{-r·t})⁺`, a put
///   `(K·e^{-r·t} − S)⁺`, and only the discount factor moves), so the maximum
///   always sits at one of the two endpoints and `max(immediate, expiry)` is
///   the exact optimum rather than an approximation of it.
/// - **Bermuda** — expiry plus every scheduled date that falls on or before
///   it. An empty schedule leaves expiry alone, which is a European.
///
/// Any other contract keeps the previous behaviour and is valued at expiry:
/// the lattice rejects those types, but this branch never did and widening
/// the error surface is a separate decision.
///
/// # Errors
///
/// Propagates whatever [`calculate_discounted_payoff`] reports for a single
/// exercise time: [`PricingError::NonFinite`] for a non-finite payoff and
/// [`PricingError::Decimal`] when the growth or discount factor leaves the
/// representable range.
fn price_deterministic(params: &BinomialPricingParams) -> Result<Decimal, PricingError> {
    let exercise_value = |time: Positive| -> Result<Decimal, PricingError> {
        calculate_discounted_payoff(BinomialPricingParams {
            expiry: time,
            ..params.clone()
        })
    };

    let expiry_value = exercise_value(params.expiry)?;
    match params.option_type {
        OptionType::American => {
            let immediate = exercise_value(Positive::ZERO)?;
            Ok(expiry_value.max(immediate))
        }
        OptionType::Bermuda { exercise_dates } => {
            let mut best = expiry_value;
            for date in exercise_dates {
                if *date > params.expiry {
                    // Not an exercise opportunity: the contract is already gone.
                    continue;
                }
                best = best.max(exercise_value(*date)?);
            }
            Ok(best)
        }
        _ => Ok(expiry_value),
    }
}

/// Spot price at lattice node `(step, i)`: `S · u^i · d^(step - i)`.
///
/// # Errors
///
/// Returns [`PricingError::Decimal`] when either power or the product leaves
/// the representable `Decimal` range, [`PricingError::Positive`] when the
/// result is not a valid `Positive`, and [`PricingError::BinomialNodeMissing`]
/// when `i` walks past `step`.
fn lattice_spot(
    asset: Positive,
    u: Decimal,
    d: Decimal,
    i: usize,
    step: usize,
) -> Result<Positive, PricingError> {
    let down_steps = step
        .checked_sub(i)
        .ok_or(PricingError::BinomialNodeMissing { node: "down_steps" })?;
    let up_power = d_powd(
        u,
        Decimal::from(i as u64),
        "pricing::binomial::lattice_spot::up",
    )?;
    let down_power = d_powd(
        d,
        Decimal::from(down_steps as u64),
        "pricing::binomial::lattice_spot::down",
    )?;
    let spot = d_mul(
        d_mul(
            asset.to_dec(),
            up_power,
            "pricing::binomial::lattice_spot::spot_up",
        )?,
        down_power,
        "pricing::binomial::lattice_spot::spot",
    )?;
    Ok(Positive::new_decimal(spot)?)
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
/// use optionstratlib::nz;
/// use positive::pos_or_panic;
/// use optionstratlib::pricing::binomial_model::{BinomialPricingParams, generate_binomial_tree};
/// use positive::Positive;
/// # fn run() -> Result<(), optionstratlib::error::Error> {
/// let params = BinomialPricingParams {
///             asset: Positive::HUNDRED,
///             volatility: pos_or_panic!(0.2),
///             int_rate: dec!(0.05),
///             strike: Positive::HUNDRED,
///             expiry: Positive::ONE,
///             no_steps: nz!(1000),
///             option_type: &OptionType::European,
///             option_style: &OptionStyle::Call,
///             side: &Side::Long,
///         };
/// let (asset_tree, option_tree) = generate_binomial_tree(&params)?;
/// # Ok(())
/// # }
/// ```
///
/// # Errors
///
/// Same failure surface as [`price_binomial`]:
/// [`PricingError::SqrtFailure`] when the up-factor exponent cannot
/// be represented, [`PricingError::BinomialNodeMissing`] when an
/// intermediate node of the lattice is unexpectedly absent, and
/// [`PricingError::Positive`] when a `Positive` construction
/// downstream underflows.
pub fn generate_binomial_tree(params: &BinomialPricingParams) -> BinomialTreeResult {
    let mut info = PayoffInfo {
        spot: params.asset,
        strike: params.strike,
        style: *params.option_style,
        side: *params.side,
        spot_prices: None,
        spot_min: None,
        spot_max: None,
    };

    let no_steps_raw = params.no_steps.get();
    let dt = (params.expiry / f2d!(no_steps_raw as f64)).to_dec();
    let up_factor = calculate_up_factor(params.volatility, dt)?;
    let down_factor = calculate_down_factor(params.volatility, dt)?;
    let probability = calculate_probability(params.int_rate, dt, down_factor, up_factor)?;
    let discount_factor = calculate_discount_factor(params.int_rate, dt)?;

    let mut asset_tree = vec![vec![Decimal::ZERO; no_steps_raw + 1]; no_steps_raw + 1];
    let mut option_tree = vec![vec![Decimal::ZERO; no_steps_raw + 1]; no_steps_raw + 1];

    for (step, step_vec) in asset_tree.iter_mut().enumerate() {
        for (node, node_val) in step_vec.iter_mut().enumerate().take(step + 1) {
            let up_steps = step
                .checked_sub(node)
                .ok_or(PricingError::BinomialNodeMissing { node: "up_steps" })?;
            let up_power = d_powd(
                up_factor,
                Decimal::from(up_steps as u64),
                "pricing::binomial::tree::up_power",
            )?;
            let down_power = d_powd(
                down_factor,
                Decimal::from(node as u64),
                "pricing::binomial::tree::down_power",
            )?;
            *node_val = d_mul(
                d_mul(up_power, down_power, "pricing::binomial::tree::factor")?,
                params.asset.to_dec(),
                "pricing::binomial::tree::asset_price",
            )?;
        }
    }

    let terminal_assets = asset_tree
        .get(no_steps_raw)
        .ok_or(PricingError::BinomialNodeMissing {
            node: "terminal_step",
        })?
        .clone();
    let terminal_options =
        option_tree
            .get_mut(no_steps_raw)
            .ok_or(PricingError::BinomialNodeMissing {
                node: "terminal_step",
            })?;
    for (node, node_val) in terminal_assets.iter().enumerate().take(no_steps_raw + 1) {
        info.spot = Positive::new_decimal(*node_val)?;
        let slot = terminal_options
            .get_mut(node)
            .ok_or(PricingError::BinomialNodeMissing {
                node: "terminal_node",
            })?;
        *slot = f2d!(params.option_type.payoff(&info));
    }

    let half_dt = d_div(dt, Decimal::TWO, "pricing::binomial::tree::half_dt")?;
    for step in (0..no_steps_raw).rev() {
        let step_assets = asset_tree
            .get(step)
            .ok_or(PricingError::BinomialNodeMissing { node: "asset_step" })?
            .clone();
        let (current_step_arr, next_step_arr) = option_tree.split_at_mut(step + 1);
        let current = current_step_arr
            .get_mut(step)
            .ok_or(PricingError::BinomialNodeMissing {
                node: "option_step",
            })?;
        for (node_idx, node_val) in current.iter_mut().enumerate().take(step + 1) {
            let node_value =
                option_node_value_wrapper(probability, next_step_arr, node_idx, discount_factor)?;
            let node_asset = || -> Result<Positive, PricingError> {
                let raw = step_assets
                    .get(node_idx)
                    .ok_or(PricingError::BinomialNodeMissing { node: "asset_node" })?;
                Ok(Positive::new_decimal(*raw)?)
            };
            match params.option_type {
                OptionType::European => {
                    *node_val = node_value;
                }
                OptionType::American => {
                    if (step == 0) & (node_idx == 0) {
                        *node_val = node_value;
                    } else {
                        info.spot = node_asset()?;
                        let intrinsic_value = params.option_type.payoff(&info);
                        let dec_node_val = d2f!(node_value);
                        *node_val = f2d!(intrinsic_value.max(dec_node_val));
                    }
                }
                OptionType::Bermuda { exercise_dates } => {
                    // Calculate time at this step
                    let time_at_step = d_mul(
                        dt,
                        Decimal::from(step as u64),
                        "pricing::binomial::tree::time_at_step",
                    )?;
                    // Check if this step is an exercise date
                    let mut is_exercise_date = false;
                    for exercise in exercise_dates {
                        let gap = d_sub(
                            time_at_step,
                            exercise.to_dec(),
                            "pricing::binomial::tree::exercise_gap",
                        )?;
                        if gap.abs() < half_dt {
                            is_exercise_date = true;
                            break;
                        }
                    }
                    if is_exercise_date && !((step == 0) & (node_idx == 0)) {
                        info.spot = node_asset()?;
                        let intrinsic_value = params.option_type.payoff(&info);
                        let dec_node_val = d2f!(node_value);
                        *node_val = f2d!(intrinsic_value.max(dec_node_val));
                    } else {
                        *node_val = node_value;
                    }
                }
                _ => {
                    return Err(PricingError::other(
                        "OptionType not supported for binomial tree generation",
                    ));
                }
            }
        }
    }

    Ok((asset_tree, option_tree))
}

#[cfg(test)]
mod tests_price_binomial {
    use super::*;
    use crate::assert_decimal_eq;
    use crate::model::types::OptionType;
    use rust_decimal::MathematicalOps;
    use rust_decimal_macros::dec;

    const EPSILON: Decimal = dec!(1e-6);

    #[test]
    fn test_european_call_option() {
        let params = BinomialPricingParams {
            asset: Positive::HUNDRED,
            strike: Positive::HUNDRED,
            int_rate: dec!(0.05),
            volatility: pos_or_panic!(0.2),
            expiry: Positive::ONE,
            no_steps: crate::nz!(3),
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
            asset: Positive::HUNDRED,
            volatility: pos_or_panic!(0.2),
            int_rate: dec!(0.05),
            strike: Positive::HUNDRED,
            expiry: Positive::ONE,
            no_steps: crate::nz!(1000),
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
            asset: pos_or_panic!(50.0),
            volatility: pos_or_panic!(0.2),
            int_rate: dec!(0.05),
            strike: pos_or_panic!(52.0),
            expiry: Positive::ONE,
            no_steps: crate::nz!(1),
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
            asset: Positive::HUNDRED,
            volatility: pos_or_panic!(0.2),
            int_rate: dec!(0.05),
            strike: Positive::HUNDRED,
            expiry: Positive::ONE,
            no_steps: crate::nz!(1000),
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
        assert_decimal_eq!(long_price, -short_price, EPSILON);
    }

    #[test]
    fn test_zero_volatility() {
        let asset = Positive::HUNDRED;
        let strike = Positive::HUNDRED;
        let int_rate = dec!(0.05);
        let expiry = Positive::ONE;

        let params = BinomialPricingParams {
            asset,
            volatility: Positive::ZERO,
            int_rate,
            strike,
            expiry,
            no_steps: crate::nz!(1000),
            option_type: &OptionType::European,
            option_style: &OptionStyle::Call,
            side: &Side::Long,
        };

        let price = price_binomial(params).unwrap();

        let exact_price = (asset * (int_rate * expiry).exp() - strike).max(Positive::ZERO)
            * (-int_rate * expiry).exp();

        assert_decimal_eq!(price, exact_price, EPSILON);
    }

    #[test]
    fn test_deep_in_the_money() {
        let params = BinomialPricingParams {
            asset: pos_or_panic!(150.0),
            volatility: pos_or_panic!(0.2),
            int_rate: dec!(0.05),
            strike: Positive::HUNDRED,
            expiry: Positive::ONE,
            no_steps: crate::nz!(1000),
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
            asset: pos_or_panic!(50.0),
            volatility: pos_or_panic!(0.2),
            int_rate: dec!(0.05),
            strike: Positive::HUNDRED,
            expiry: Positive::ONE,
            no_steps: crate::nz!(1000),
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
            asset: Positive::HUNDRED,
            volatility: pos_or_panic!(0.2),
            int_rate: dec!(0.05),
            strike: Positive::HUNDRED,
            expiry: Positive::ZERO,
            no_steps: crate::nz!(1000),
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
    use crate::assert_decimal_eq;
    use crate::model::types::OptionType;
    use rust_decimal_macros::dec;

    const EPSILON: Decimal = dec!(1e-5);

    #[test]
    fn test_binomial_tree_basic() {
        let params = BinomialPricingParams {
            asset: Positive::HUNDRED,
            strike: Positive::HUNDRED,
            int_rate: dec!(0.05),
            volatility: pos_or_panic!(0.2),
            expiry: Positive::ONE,
            no_steps: crate::nz!(3),
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
            asset: Positive::HUNDRED,
            strike: Positive::HUNDRED,
            int_rate: dec!(0.05),
            volatility: pos_or_panic!(0.2),
            expiry: Positive::ONE,
            no_steps: crate::nz!(3),
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
            asset: pos_or_panic!(30.0),
            strike: pos_or_panic!(30.0),
            expiry: Positive::ONE,
            int_rate: dec!(0.05),
            volatility: pos_or_panic!(0.17),
            no_steps: crate::nz!(1),
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
            asset: pos_or_panic!(30.0),
            strike: pos_or_panic!(30.0),
            expiry: Positive::ONE,
            int_rate: dec!(0.05),
            volatility: pos_or_panic!(0.17),
            no_steps: crate::nz!(2),
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
            asset: Positive::HUNDRED,
            strike: pos_or_panic!(110.0),
            expiry: pos_or_panic!(3.0), // Assuming each time step is 1 unit of time
            int_rate: dec!(0.05),
            volatility: pos_or_panic!(0.09531018), // Calculated to match the 10% up/down movement
            no_steps: crate::nz!(3),
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
            asset: pos_or_panic!(50.0),
            volatility: pos_or_panic!(0.2),
            int_rate: dec!(0.05),
            strike: pos_or_panic!(52.0),
            expiry: Positive::TWO,
            no_steps: crate::nz!(2),
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
            asset: pos_or_panic!(50.0),
            volatility: pos_or_panic!(0.2),
            int_rate: dec!(0.05),
            strike: pos_or_panic!(52.0),
            expiry: Positive::TWO,
            no_steps: crate::nz!(2),
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

        assert_decimal_eq!(option_tree[1][1], params.strike - asset_tree[1][1], EPSILON);
        assert_decimal_eq!(option_tree[0][0], dec!(4.887966), EPSILON);
    }
}

#[cfg(test)]
mod tests_bermuda_option {
    use super::*;
    use crate::assert_decimal_eq;
    use crate::model::types::OptionType;
    use rust_decimal_macros::dec;

    const EPSILON: Decimal = dec!(1e-4);

    #[test]
    fn test_bermuda_price_between_european_and_american() {
        // Bermuda price should be: European <= Bermuda <= American
        let european_params = BinomialPricingParams {
            asset: pos_or_panic!(50.0),
            volatility: pos_or_panic!(0.2),
            int_rate: dec!(0.05),
            strike: pos_or_panic!(52.0),
            expiry: Positive::ONE,
            no_steps: crate::nz!(100),
            option_type: &OptionType::European,
            option_style: &OptionStyle::Put,
            side: &Side::Long,
        };

        let american_params = BinomialPricingParams {
            option_type: &OptionType::American,
            ..european_params.clone()
        };

        // Exercise at 3 months, 6 months, 9 months
        let bermuda_type = OptionType::Bermuda {
            exercise_dates: vec![pos_or_panic!(0.25), pos_or_panic!(0.5), pos_or_panic!(0.75)],
        };
        let bermuda_params = BinomialPricingParams {
            option_type: &bermuda_type,
            ..european_params.clone()
        };

        let european_price = price_binomial(european_params).unwrap();
        let american_price = price_binomial(american_params).unwrap();
        let bermuda_price = price_binomial(bermuda_params).unwrap();

        assert!(
            european_price <= bermuda_price,
            "European {} should be <= Bermuda {}",
            european_price,
            bermuda_price
        );
        assert!(
            bermuda_price <= american_price,
            "Bermuda {} should be <= American {}",
            bermuda_price,
            american_price
        );
    }

    #[test]
    fn test_bermuda_single_exercise_date() {
        // Single exercise date should give price between European and American
        let bermuda_type = OptionType::Bermuda {
            exercise_dates: vec![pos_or_panic!(0.5)],
        };
        let params = BinomialPricingParams {
            asset: Positive::HUNDRED,
            volatility: pos_or_panic!(0.3),
            int_rate: dec!(0.05),
            strike: pos_or_panic!(105.0),
            expiry: Positive::ONE,
            no_steps: crate::nz!(50),
            option_type: &bermuda_type,
            option_style: &OptionStyle::Put,
            side: &Side::Long,
        };

        let price = price_binomial(params).unwrap();
        assert!(
            price > Decimal::ZERO,
            "Bermuda put price should be positive"
        );
    }

    #[test]
    fn test_bermuda_many_exercise_dates_approaches_american() {
        // With many exercise dates, Bermuda should approach American price
        let european_params = BinomialPricingParams {
            asset: pos_or_panic!(50.0),
            volatility: pos_or_panic!(0.2),
            int_rate: dec!(0.05),
            strike: pos_or_panic!(52.0),
            expiry: Positive::ONE,
            no_steps: crate::nz!(52),
            option_type: &OptionType::European,
            option_style: &OptionStyle::Put,
            side: &Side::Long,
        };

        let american_params = BinomialPricingParams {
            option_type: &OptionType::American,
            ..european_params.clone()
        };

        // Weekly exercise dates (52 dates for 1 year)
        let exercise_dates: Vec<Positive> = (1..=52)
            .map(|i| pos_or_panic!(f64::from(i) / 52.0))
            .collect();
        let bermuda_type = OptionType::Bermuda { exercise_dates };
        let bermuda_params = BinomialPricingParams {
            option_type: &bermuda_type,
            ..european_params.clone()
        };

        let american_price = price_binomial(american_params).unwrap();
        let bermuda_price = price_binomial(bermuda_params).unwrap();

        // Bermuda with weekly exercise should be close to American
        let diff = (american_price - bermuda_price).abs();
        assert!(
            diff < dec!(0.5),
            "Bermuda with 52 exercise dates should be close to American: diff = {}",
            diff
        );
    }

    #[test]
    fn test_bermuda_no_exercise_dates_equals_european() {
        // Empty exercise dates results in European-like behavior
        let european_params = BinomialPricingParams {
            asset: Positive::HUNDRED,
            volatility: pos_or_panic!(0.2),
            int_rate: dec!(0.05),
            strike: Positive::HUNDRED,
            expiry: Positive::ONE,
            no_steps: crate::nz!(50),
            option_type: &OptionType::European,
            option_style: &OptionStyle::Put,
            side: &Side::Long,
        };

        let bermuda_type = OptionType::Bermuda {
            exercise_dates: vec![],
        };
        let bermuda_params = BinomialPricingParams {
            option_type: &bermuda_type,
            ..european_params.clone()
        };

        let european_price = price_binomial(european_params).unwrap();
        let bermuda_price = price_binomial(bermuda_params).unwrap();

        assert_decimal_eq!(european_price, bermuda_price, EPSILON);
    }

    #[test]
    fn test_bermuda_call_option() {
        let bermuda_type = OptionType::Bermuda {
            exercise_dates: vec![pos_or_panic!(0.25), pos_or_panic!(0.5), pos_or_panic!(0.75)],
        };
        let params = BinomialPricingParams {
            asset: Positive::HUNDRED,
            volatility: pos_or_panic!(0.25),
            int_rate: dec!(0.05),
            strike: pos_or_panic!(95.0),
            expiry: Positive::ONE,
            no_steps: crate::nz!(100),
            option_type: &bermuda_type,
            option_style: &OptionStyle::Call,
            side: &Side::Long,
        };

        let price = price_binomial(params).unwrap();
        assert!(
            price > dec!(5.0),
            "ITM Bermuda call should have value > intrinsic"
        );
    }
}

#[cfg(test)]
mod tests_zero_volatility_early_exercise {
    use super::*;
    use crate::assert_decimal_eq;
    use crate::model::types::OptionType;
    use rust_decimal_macros::dec;

    /// With no volatility the underlying is its own forward, so every price
    /// below has a closed form. The only error between the two is
    /// `Decimal::checked_exp`'s series truncation; the largest gap observed
    /// across these cases is `7.1e-14`, so `1e-12` pins roughly thirteen
    /// significant digits with room to spare.
    const ANALYTIC_EPSILON: Decimal = dec!(1e-12);

    /// One year, one hundred steps, no volatility. The step count is
    /// irrelevant on this path — it never builds a lattice — and is only
    /// here so the parameters stay comparable with the small-volatility
    /// runs further down.
    fn zero_vol_params<'a>(
        asset: Positive,
        strike: Positive,
        int_rate: Decimal,
        option_type: &'a OptionType,
        option_style: &'a OptionStyle,
    ) -> BinomialPricingParams<'a> {
        BinomialPricingParams {
            asset,
            volatility: Positive::ZERO,
            int_rate,
            strike,
            expiry: Positive::ONE,
            no_steps: crate::nz!(100),
            option_type,
            option_style,
            side: &Side::Long,
        }
    }

    #[test]
    fn test_price_binomial_zero_volatility_american_put_itm_exercises_immediately() {
        // S = 90, K = 100, r = 5%, T = 1, σ = 0. The forward is 94.61, so
        // holding to expiry is worth e^{-0.05}(100 − 90·e^{0.05}) = 5.1229.
        // Exercising now is worth K − S = 10, and that is the price.
        let params = zero_vol_params(
            pos_or_panic!(90.0),
            Positive::HUNDRED,
            dec!(0.05),
            &OptionType::American,
            &OptionStyle::Put,
        );
        let european = price_binomial(BinomialPricingParams {
            option_type: &OptionType::European,
            ..params.clone()
        })
        .unwrap();
        let american = price_binomial(params).unwrap();

        // Immediate exercise is exact: no discount factor is involved.
        assert_eq!(american, dec!(10));
        assert_decimal_eq!(european, dec!(5.122942450071406), ANALYTIC_EPSILON);
        assert!(
            american > european,
            "American {american} must carry an early-exercise premium over European {european}"
        );
    }

    #[test]
    fn test_price_binomial_zero_volatility_american_call_itm_holds_to_expiry() {
        // S = 110, K = 100, r = 5%. A call on a non-dividend-paying forward
        // is never exercised early while r > 0: holding is worth
        // S − K·e^{-rT} = 14.877, above the intrinsic 10.
        let params = zero_vol_params(
            Positive::HUNDRED + pos_or_panic!(10.0),
            Positive::HUNDRED,
            dec!(0.05),
            &OptionType::American,
            &OptionStyle::Call,
        );
        let european = price_binomial(BinomialPricingParams {
            option_type: &OptionType::European,
            ..params.clone()
        })
        .unwrap();
        let american = price_binomial(params).unwrap();

        assert_eq!(american, european);
        assert_decimal_eq!(american, dec!(14.877057549928594), ANALYTIC_EPSILON);
        assert!(american > dec!(10), "holding must beat the intrinsic 10");
    }

    #[test]
    fn test_price_binomial_zero_volatility_american_call_negative_rate_exercises_immediately() {
        // Flip the sign of the rate and the call is the one with an
        // early-exercise premium: holding is worth
        // e^{0.05}(110·e^{-0.05} − 100) = 4.873, exercising now is worth 10.
        let params = zero_vol_params(
            Positive::HUNDRED + pos_or_panic!(10.0),
            Positive::HUNDRED,
            dec!(-0.05),
            &OptionType::American,
            &OptionStyle::Call,
        );
        let european = price_binomial(BinomialPricingParams {
            option_type: &OptionType::European,
            ..params.clone()
        })
        .unwrap();
        let american = price_binomial(params).unwrap();

        assert_eq!(american, dec!(10));
        assert_decimal_eq!(european, dec!(4.872890362397602), ANALYTIC_EPSILON);
    }

    #[test]
    fn test_price_binomial_zero_volatility_american_put_negative_rate_holds_to_expiry() {
        // The mirror image: with r < 0 the put is never exercised early,
        // and holding is worth 100·e^{0.02} − 90 = 12.020 against an
        // intrinsic of 10.
        let params = zero_vol_params(
            pos_or_panic!(90.0),
            Positive::HUNDRED,
            dec!(-0.02),
            &OptionType::American,
            &OptionStyle::Put,
        );
        let european = price_binomial(BinomialPricingParams {
            option_type: &OptionType::European,
            ..params.clone()
        })
        .unwrap();
        let american = price_binomial(params).unwrap();

        assert_eq!(american, european);
        assert_decimal_eq!(american, dec!(12.020134002675576), ANALYTIC_EPSILON);
        assert!(american > dec!(10), "holding must beat the intrinsic 10");
    }

    #[test]
    fn test_price_binomial_zero_volatility_european_put_keeps_the_discounted_forward_payoff() {
        // The regression bar: early exercise must not leak into a European.
        let params = zero_vol_params(
            pos_or_panic!(90.0),
            Positive::HUNDRED,
            dec!(0.05),
            &OptionType::European,
            &OptionStyle::Put,
        );

        let price = price_binomial(params).unwrap();
        assert_decimal_eq!(price, dec!(5.122942450071406), ANALYTIC_EPSILON);
    }

    #[test]
    fn test_price_binomial_zero_volatility_bermuda_takes_the_best_scheduled_date() {
        // Exercising at t is worth K·e^{-rt} − S, which decays in t, so the
        // earliest scheduled date wins: 100·e^{-0.0125} − 90 = 8.7578. The
        // price sits strictly between the European (no early exercise) and
        // the American (exercise at t = 0), which is what proves the
        // schedule is being read rather than ignored in either direction.
        let bermuda_type = OptionType::Bermuda {
            exercise_dates: vec![pos_or_panic!(0.25), pos_or_panic!(0.5), pos_or_panic!(0.75)],
        };
        let params = zero_vol_params(
            pos_or_panic!(90.0),
            Positive::HUNDRED,
            dec!(0.05),
            &bermuda_type,
            &OptionStyle::Put,
        );
        let european = price_binomial(BinomialPricingParams {
            option_type: &OptionType::European,
            ..params.clone()
        })
        .unwrap();
        let american = price_binomial(BinomialPricingParams {
            option_type: &OptionType::American,
            ..params.clone()
        })
        .unwrap();
        let bermuda = price_binomial(params).unwrap();

        assert_decimal_eq!(bermuda, dec!(8.757780049388145), ANALYTIC_EPSILON);
        assert!(
            european < bermuda,
            "European {european} < Bermuda {bermuda}"
        );
        assert!(
            bermuda < american,
            "Bermuda {bermuda} < American {american}"
        );
    }

    #[test]
    fn test_price_binomial_zero_volatility_bermuda_later_only_schedule_is_worth_less() {
        // Same contract, the 0.25 date removed: the best remaining date is
        // 0.5 and the price drops to 100·e^{-0.025} − 90 = 7.5310. A branch
        // that exercised continuously would return the same number for both
        // schedules.
        let early_type = OptionType::Bermuda {
            exercise_dates: vec![pos_or_panic!(0.25), pos_or_panic!(0.5)],
        };
        let late_type = OptionType::Bermuda {
            exercise_dates: vec![pos_or_panic!(0.5)],
        };
        let params = zero_vol_params(
            pos_or_panic!(90.0),
            Positive::HUNDRED,
            dec!(0.05),
            &late_type,
            &OptionStyle::Put,
        );
        let early = price_binomial(BinomialPricingParams {
            option_type: &early_type,
            ..params.clone()
        })
        .unwrap();
        let late = price_binomial(params).unwrap();

        assert_decimal_eq!(late, dec!(7.530991202833263), ANALYTIC_EPSILON);
        assert_decimal_eq!(early, dec!(8.757780049388145), ANALYTIC_EPSILON);
        assert!(late < early, "later-only schedule {late} < {early}");
    }

    #[test]
    fn test_price_binomial_zero_volatility_bermuda_empty_schedule_equals_european() {
        let bermuda_type = OptionType::Bermuda {
            exercise_dates: vec![],
        };
        let params = zero_vol_params(
            pos_or_panic!(90.0),
            Positive::HUNDRED,
            dec!(0.05),
            &bermuda_type,
            &OptionStyle::Put,
        );
        let european = price_binomial(BinomialPricingParams {
            option_type: &OptionType::European,
            ..params.clone()
        })
        .unwrap();
        let bermuda = price_binomial(params).unwrap();

        assert_eq!(bermuda, european);
    }

    #[test]
    fn test_price_binomial_zero_volatility_bermuda_schedule_after_expiry_equals_european() {
        // A date the contract never reaches is not an exercise opportunity.
        let bermuda_type = OptionType::Bermuda {
            exercise_dates: vec![Positive::TWO],
        };
        let params = zero_vol_params(
            pos_or_panic!(90.0),
            Positive::HUNDRED,
            dec!(0.05),
            &bermuda_type,
            &OptionStyle::Put,
        );
        let european = price_binomial(BinomialPricingParams {
            option_type: &OptionType::European,
            ..params.clone()
        })
        .unwrap();
        let bermuda = price_binomial(params).unwrap();

        assert_eq!(bermuda, european);
    }

    #[test]
    fn test_price_binomial_zero_volatility_bermuda_exercisable_now_equals_american() {
        // A schedule containing t = 0 gives the holder the only date the
        // American would have used here, so the two must agree.
        let bermuda_type = OptionType::Bermuda {
            exercise_dates: vec![Positive::ZERO],
        };
        let params = zero_vol_params(
            pos_or_panic!(90.0),
            Positive::HUNDRED,
            dec!(0.05),
            &bermuda_type,
            &OptionStyle::Put,
        );
        let american = price_binomial(BinomialPricingParams {
            option_type: &OptionType::American,
            ..params.clone()
        })
        .unwrap();
        let bermuda = price_binomial(params).unwrap();

        assert_eq!(bermuda, american);
        assert_eq!(bermuda, dec!(10));
    }

    #[test]
    fn test_price_binomial_collapsed_lattice_american_put_exercises_immediately() {
        // `1e-28` is the smallest non-zero `Decimal`, so `σ√dt` underflows
        // and `u == d`: the lattice collapses and the deterministic branch
        // takes over. It must honour early exercise like the σ = 0 branch
        // it stands in for. (`1e-29` and below round to zero and go through
        // the σ = 0 branch instead.)
        let params = BinomialPricingParams {
            asset: pos_or_panic!(90.0),
            volatility: Positive::new_decimal(dec!(1e-28)).unwrap(),
            int_rate: dec!(0.05),
            strike: Positive::HUNDRED,
            expiry: Positive::ONE,
            no_steps: crate::nz!(10),
            option_type: &OptionType::American,
            option_style: &OptionStyle::Put,
            side: &Side::Long,
        };

        let price = price_binomial(params).unwrap();
        assert_eq!(price, dec!(10));
    }
}

#[cfg(test)]
mod tests_zero_volatility_continuity {
    use super::*;
    use crate::assert_decimal_eq;
    use crate::model::types::OptionType;
    use rust_decimal_macros::dec;

    /// Largest gap measured across the five cases below is `4.9e-15`, which
    /// is `Decimal` round-off accumulated through two hundred induction
    /// steps rather than model error. `1e-12` leaves two orders of
    /// magnitude of headroom without hiding a real discrepancy: the defect
    /// this guards against moved the American put by `4.88`.
    const CONTINUITY_EPSILON: Decimal = dec!(1e-12);

    /// The volatility used to approach the limit, and the step count that
    /// goes with it.
    ///
    /// The two cannot be chosen independently. A CRR lattice is only
    /// arbitrage-free while `d < e^{r·dt} < u`, i.e. while `σ > |r|·√dt`;
    /// below that the risk-neutral probability leaves `[0, 1]`, is clamped,
    /// and the lattice stops approximating anything. With `|r| = 0.05` and
    /// `dt = 1/200`, the bound is `0.0035`, so `σ = 0.005` clears it by a
    /// factor of 1.4. Pushing σ lower without raising the step count moves
    /// *away* from the limit — measured at `σ = 0.002, n = 400` the
    /// American call is off by `−1.09`.
    const LIMIT_VOLATILITY: Decimal = dec!(0.005);

    /// Prices the same contract twice, at σ = 0 and at `LIMIT_VOLATILITY`,
    /// and asserts the two agree.
    fn assert_zero_volatility_is_the_limit(
        asset: Positive,
        strike: Positive,
        int_rate: Decimal,
        option_type: &OptionType,
        option_style: &OptionStyle,
    ) {
        let base = BinomialPricingParams {
            asset,
            volatility: Positive::ZERO,
            int_rate,
            strike,
            expiry: Positive::ONE,
            no_steps: crate::nz!(200),
            option_type,
            option_style,
            side: &Side::Long,
        };

        let at_zero = price_binomial(base.clone()).unwrap();
        let near_zero = price_binomial(BinomialPricingParams {
            volatility: Positive::new_decimal(LIMIT_VOLATILITY).unwrap(),
            ..base
        })
        .unwrap();

        assert_decimal_eq!(at_zero, near_zero, CONTINUITY_EPSILON);
    }

    #[test]
    fn test_price_binomial_zero_volatility_american_put_matches_the_lattice_limit() {
        assert_zero_volatility_is_the_limit(
            pos_or_panic!(90.0),
            Positive::HUNDRED,
            dec!(0.05),
            &OptionType::American,
            &OptionStyle::Put,
        );
    }

    #[test]
    fn test_price_binomial_zero_volatility_american_call_matches_the_lattice_limit() {
        // r > 0: the limit is the European value, so this catches a branch
        // that exercised early when it should not.
        assert_zero_volatility_is_the_limit(
            Positive::HUNDRED + pos_or_panic!(10.0),
            Positive::HUNDRED,
            dec!(0.05),
            &OptionType::American,
            &OptionStyle::Call,
        );
    }

    #[test]
    fn test_price_binomial_zero_volatility_american_call_negative_rate_matches_the_lattice_limit() {
        // r < 0: the limit is the intrinsic, so this catches the opposite
        // mistake on the same contract.
        assert_zero_volatility_is_the_limit(
            Positive::HUNDRED + pos_or_panic!(10.0),
            Positive::HUNDRED,
            dec!(-0.05),
            &OptionType::American,
            &OptionStyle::Call,
        );
    }

    #[test]
    fn test_price_binomial_zero_volatility_bermuda_matches_the_lattice_limit() {
        let bermuda_type = OptionType::Bermuda {
            exercise_dates: vec![pos_or_panic!(0.25), pos_or_panic!(0.5), pos_or_panic!(0.75)],
        };
        assert_zero_volatility_is_the_limit(
            pos_or_panic!(90.0),
            Positive::HUNDRED,
            dec!(0.05),
            &bermuda_type,
            &OptionStyle::Put,
        );
    }

    #[test]
    fn test_price_binomial_zero_volatility_european_matches_the_lattice_limit() {
        assert_zero_volatility_is_the_limit(
            pos_or_panic!(90.0),
            Positive::HUNDRED,
            dec!(0.05),
            &OptionType::European,
            &OptionStyle::Put,
        );
    }
}
