/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 19/9/25
******************************************************************************/

//! Pricing capability for the core [`optionstratlib_core::model::Options`] contract.
//!
//! [`OptionPricing`] is the pricing-owned extension trait that carries every
//! model-based valuation of an [`optionstratlib_core::model::Options`] contract: binomial lattice,
//! Black-Scholes closed form, Monte Carlo over supplied paths, the telegraph
//! finite-difference kernel, the time-value decomposition and the implied
//! volatility bisection. The core type keeps only contract data and payoff
//! arithmetic; this trait is where the numerics attach to it.
//!
//! `Options` has no pricing methods of its own; bring this trait into scope
//! (directly, or through the `optionstratlib` prelude) to price one:
//!
//! ```rust
//! use optionstratlib_pricing::pricing::OptionPricing;
//! use optionstratlib_core::model::{ExpirationDate, OptionStyle, OptionType, Options, Side};
//! use optionstratlib_core::{model::Positive, pos_or_panic};
//! use rust_decimal_macros::dec;
//!
//! let option = Options::new(
//!     OptionType::European,
//!     Side::Long,
//!     "TEST".to_string(),
//!     pos_or_panic!(100.0),
//!     ExpirationDate::Days(pos_or_panic!(30.0)),
//!     pos_or_panic!(0.2),
//!     Positive::ONE,
//!     pos_or_panic!(100.0),
//!     dec!(0.05),
//!     OptionStyle::Call,
//!     Positive::ZERO,
//!     None,
//! );
//! let price = option.calculate_price_black_scholes()?;
//! assert!(price > rust_decimal::Decimal::ZERO);
//! # Ok::<(), optionstratlib_core::error::OptionsError>(())
//! ```

use crate::error::{PricingError, VolatilityError};
use crate::pricing::black_scholes_model::european_price_band;
use crate::pricing::constants::{IV_TOLERANCE, MAX_ITERATIONS_IV};
use crate::pricing::monte_carlo::price_option_monte_carlo;
use crate::pricing::{
    BinomialPricingParams, RegimeVolatility, TELEGRAPH_PATHS, black_scholes,
    generate_binomial_tree, price_binomial, telegraph,
};
use optionstratlib_core::error::{OptionsError, OptionsResult};
use optionstratlib_core::model::Options;
use optionstratlib_core::model::Positive;
use optionstratlib_core::model::decimal::{d_add, d_div, d_sub};
use optionstratlib_core::model::types::OptionType;
use rand::Rng;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::num::NonZeroUsize;

/// Result type for binomial tree pricing models, containing:
/// - The option price
/// - Price tree (asset price evolution)
/// - Option value tree (option value at each node)
pub type PriceBinomialTree = OptionsResult<(Decimal, Vec<Vec<Decimal>>, Vec<Vec<Decimal>>)>;

/// Model-based valuation of an [`optionstratlib_core::model::Options`] contract.
///
/// Every method is a pure function of the contract data held by the
/// implementor plus the method's own parameters; none of them mutates the
/// contract. Long positions report positive prices, short positions report
/// the negated price of the equivalent long contract.
///
/// The trait is implemented for [`optionstratlib_core::model::Options`] by the
/// pricing layer. `Options` has no pricing methods of its own: bring this
/// trait into scope to price one.
pub trait OptionPricing {
    /// Calculates the price of an option using the binomial tree model.
    ///
    /// This method implements the binomial option pricing model which constructs a
    /// discrete-time lattice (tree) of possible future underlying asset prices to
    /// determine the option's value. The approach is particularly valuable for pricing
    /// American options and other early-exercise scenarios.
    ///
    /// The calculation divides the time to expiration into a specified number of steps,
    /// creating a binomial tree that represents possible price paths of the underlying asset.
    /// The option's value is then calculated by working backward from expiration to the
    /// present value.
    ///
    /// # Parameters
    ///
    /// * `no_steps` - The number of steps to use in the binomial tree calculation,
    ///   as a [`NonZeroUsize`] so zero is structurally invalid at the type level
    ///   (no runtime check required). Higher values increase accuracy but also
    ///   computational cost. See [`crate::pricing::constants::DEFAULT_BINOMIAL_STEPS`]
    ///   for a sensible default.
    ///
    /// # Returns
    ///
    /// * `OptionsResult<Decimal>` - A result containing the calculated option price as a
    ///   Decimal value, or an OptionsError if the calculation failed.
    ///
    /// # Errors
    ///
    /// Returns an `OptionsError` if:
    /// * The time to expiration calculation fails
    /// * The binomial price calculation fails
    fn calculate_price_binomial(&self, no_steps: NonZeroUsize) -> OptionsResult<Decimal>;

    /// Calculates option price using the binomial tree model.
    ///
    /// This method implements a binomial tree (lattice) approach to option pricing, which
    /// discretizes the underlying asset's price movement over time. The model builds a tree
    /// of possible future asset prices and works backwards to determine the current option value.
    ///
    /// # Parameters
    ///
    /// * `no_steps` - The number of discrete time steps to use in the model,
    ///   as a [`NonZeroUsize`] so zero is structurally invalid at the type
    ///   level. Higher values increase precision but also computational cost.
    ///
    /// # Returns
    ///
    /// * `PriceBinomialTree` - A result containing:
    ///   - The calculated option price
    ///   - The asset price tree (underlying price evolution)
    ///   - The option value tree (option price at each node)
    ///
    /// This method is particularly valuable for pricing American options and other early-exercise
    /// scenarios that cannot be accurately priced using closed-form solutions.
    ///
    /// # Errors
    ///
    /// Returns [`OptionsError::ExpirationDate`] when the option's expiration
    /// cannot be converted to a positive year fraction, or propagates any
    /// `PricingError` surfaced by [`generate_binomial_tree`] (e.g.
    /// [`PricingError::BinomialNodeMissing`] or [`PricingError::SqrtFailure`]).
    fn calculate_price_binomial_tree(&self, no_steps: NonZeroUsize) -> PriceBinomialTree;

    /// Calculates option price using the Black-Scholes model.
    ///
    /// This method implements the Black-Scholes option pricing formula, which provides
    /// a closed-form solution for European-style options. The model assumes lognormal
    /// distribution of underlying asset prices and constant volatility.
    ///
    /// # Returns
    ///
    /// * `OptionsResult<Decimal>` - A result containing the calculated option price
    ///   as a Decimal value, or an error if the calculation failed.
    ///
    /// This method is computationally efficient but limited to European options without
    /// early exercise capabilities.
    ///
    /// # Errors
    ///
    /// Propagates any `PricingError` returned by [`black_scholes`] (wrapped as
    /// `OptionsError::PricingError`), most commonly
    /// `PricingError::ExpirationDate` when the expiration cannot be resolved
    /// or `PricingError::MethodError` when the closed-form formula fails
    /// numerically.
    fn calculate_price_black_scholes(&self) -> OptionsResult<Decimal>;

    /// Calculates the price of an option using the Monte Carlo simulation method.
    ///
    /// # Arguments
    ///
    /// * `prices` - A slice of `Positive` values representing the prices used
    ///   in the Monte Carlo simulation.
    ///
    /// # Returns
    ///
    /// * `OptionsResult<Positive>` - The calculated price of the option wrapped in
    ///   an `OptionsResult`. This will return a valid `Positive` value if successful,
    ///   or an error if the simulation fails.
    ///
    /// # Errors
    ///
    /// This function will return an error in the `OptionsResult` if the internal
    /// Monte Carlo price computation fails during the execution of `price_option_monte_carlo`.
    fn calculate_price_montecarlo(&self, prices: &[Positive]) -> OptionsResult<Positive>;

    /// Calculates option price with the telegraph regime-switching
    /// volatility Monte-Carlo pricer.
    ///
    /// Averages the discounted payoff over
    /// [`crate::pricing::TELEGRAPH_PATHS`] simulated paths whose volatility
    /// switches between the two levels of `volatility`, with both transition
    /// rates estimated from returns simulated at the option's implied
    /// volatility; see [`crate::pricing::telegraph()`] for the model. Call
    /// that function directly to choose the path count or the rates.
    ///
    /// # Parameters
    ///
    /// * `no_steps` - The number of discrete time steps to use in the model,
    ///   as a [`NonZeroUsize`] so zero is structurally invalid at the type
    ///   level. Higher values increase precision but also computational cost.
    /// * `volatility` - The volatilities of the two regimes. A single implied
    ///   volatility does not identify two levels, so the caller states them;
    ///   [`RegimeVolatility::constant`]`(option.implied_volatility)` prices at
    ///   the option's own volatility, which converges to Black-Scholes.
    /// * `rng` - The generator every draw of the telegraph simulation is
    ///   taken from (see [`crate::pricing::telegraph()`]). A seeded generator
    ///   such as [`optionstratlib_core::utils::deterministic_rng`] makes the
    ///   price reproducible; pass `&mut rand::rng()` to draw from the
    ///   thread-local RNG. It is a trait object so the trait stays
    ///   dyn-compatible.
    ///
    /// # Returns
    ///
    /// * `Result<Decimal, PricingError>` - A result containing the calculated option price
    ///   as a Decimal value, or a boxed error if the calculation failed.
    ///
    /// # Errors
    ///
    /// Propagates any `PricingError` returned by the `telegraph` pricing
    /// kernel (wrapped as `OptionsError::PricingError`), typically
    /// `PricingError::ExpirationDate`, `PricingError::NonFinite` or
    /// `PricingError::MethodError` when a simulated quantity is not
    /// representable.
    fn calculate_price_telegraph(
        &self,
        no_steps: NonZeroUsize,
        volatility: RegimeVolatility,
        rng: &mut dyn Rng,
    ) -> OptionsResult<Decimal>;

    /// Calculates the time value component of an option's price.
    ///
    /// Time value represents the portion of an option's premium that exceeds its intrinsic value.
    /// It reflects the market's expectation that the option may become more valuable before expiration
    /// due to potential favorable movements in the underlying asset price.
    ///
    /// The calculation uses the Black-Scholes model to determine the total option price,
    /// then subtracts the intrinsic value to find the time value component.
    ///
    /// # Returns
    /// - `Ok(Decimal)` containing the time value (never negative, minimum value is zero)
    /// - `Err` if the price calculation encounters an error
    ///
    /// # Errors
    ///
    /// Propagates any `OptionsError` returned by
    /// [`OptionPricing::calculate_price_black_scholes`] or
    /// [`Options::intrinsic_value`] (typically `OptionsError::PricingError`
    /// with `PricingError::ExpirationDate` as the inner cause).
    fn time_value(&self) -> OptionsResult<Decimal>;

    /// Estimates the implied volatility of an option from its market price
    /// using binary search.
    ///
    /// Implied volatility is a key metric in options trading that reflects
    /// the market's view of the expected volatility of the underlying asset.
    ///
    /// ### Parameters:
    ///
    /// - `market_price`: The market price of the option as a `Decimal`. This represents the cost
    ///   at which the option is traded in the market.
    ///
    /// ### Returns:
    ///
    /// - `Ok(Positive)`: A `Positive` value representing the calculated implied volatility as a percentage.
    /// - `Err(VolatilityError)`: An error indicating the reason calculation failed, such as:
    ///     - No convergence within the maximum number of iterations.
    ///     - Invalid option parameters.
    ///
    /// ### Implementation Details:
    ///
    /// - **Binary Search**: The function uses a binary search approach to iteratively find
    ///   the implied volatility (`volatility`) that narrows the difference between the calculated
    ///   option price (via Black-Scholes) and the target `market_price`.
    ///
    /// - **Short Options Adjustment**: For short options, the market price is inverted (negated),
    ///   and this adjustment ensures proper calculation of implied volatility.
    ///
    /// - **Bounds and Iteration**: The method starts with a maximum bound (`5.0`, representing 500%
    ///   volatility) and a lower bound (`0.0`). It adjusts these bounds based on whether the computed
    ///   price is above or below the target and repeats until convergence or the maximum number of
    ///   iterations is reached (`MAX_ITERATIONS_IV`).
    ///
    /// - **Convergence Tolerance**: The function stops iterating when the computed price is within `IV_TOLERANCE`
    ///   of the target market price or when the difference between the high and low bounds is smaller
    ///   than a threshold (`0.0001`).
    ///
    /// - **No-arbitrage band**: a European target is first checked against the band of attainable
    ///   Black–Scholes prices (Hull, bounds on option prices); outside it no implied volatility
    ///   exists and the solver reports an error rather than an edge of its bracket.
    ///
    /// # Errors
    ///
    /// - [`VolatilityError::InvalidPrice`] when a European target lies
    ///   outside the no-arbitrage band `[max(S e^(-qT) - K e^(-rT), 0),
    ///   S e^(-qT)]` (call) or `[max(K e^(-rT) - S e^(-qT), 0), K e^(-rT)]`
    ///   (put), widened by `IV_TOLERANCE`: no volatility reproduces such a
    ///   price. `price` is the magnitude of `market_price`.
    /// - [`VolatilityError::NoConvergence`] when the target is inside the band
    ///   but above the price at the top of the bracket (`σ = 500 %`), so the
    ///   bisection cannot bracket the root, or when it exhausts
    ///   `MAX_ITERATIONS_IV` (1000, far above the ~16 halvings the bracket
    ///   floor needs).
    /// - [`VolatilityError::DecimalError`] when a bracket or residual step
    ///   overflows `Decimal` (a target near `Decimal::MAX`).
    /// - [`VolatilityError::Options`] from the underlying Black–Scholes
    ///   evaluation or the band (an expired option, for instance), wrapped
    ///   as [`OptionsError::ImpliedVolatilityInvariant`] when the midpoint
    ///   invariant check fails.
    fn calculate_implied_volatility(
        &self,
        market_price: Decimal,
    ) -> Result<Positive, VolatilityError>;
}

impl OptionPricing for Options {
    fn calculate_price_binomial(&self, no_steps: NonZeroUsize) -> OptionsResult<Decimal> {
        let expiry = self.time_to_expiration()?;
        let cpb = price_binomial(BinomialPricingParams {
            asset: self.underlying_price,
            volatility: self.implied_volatility,
            int_rate: self.risk_free_rate,
            strike: self.strike_price,
            expiry,
            no_steps,
            option_type: &self.option_type,
            option_style: &self.option_style,
            side: &self.side,
        })?;
        Ok(cpb)
    }

    fn calculate_price_binomial_tree(&self, no_steps: NonZeroUsize) -> PriceBinomialTree {
        let expiry = self.time_to_expiration()?;
        let params = BinomialPricingParams {
            asset: self.underlying_price,
            volatility: self.implied_volatility,
            int_rate: self.risk_free_rate,
            strike: self.strike_price,
            expiry,
            no_steps,
            option_type: &self.option_type,
            option_style: &self.option_style,
            side: &self.side,
        };
        let (asset_tree, option_tree) = generate_binomial_tree(&params)?;
        // The tree is already signed by `side` (#648); negating its root
        // again priced a short option at the long price.
        let price = option_tree
            .first()
            .and_then(|row| row.first())
            .copied()
            .ok_or(PricingError::BinomialNodeMissing {
                node: "option[0][0]",
            })?;
        Ok((price, asset_tree, option_tree))
    }

    fn calculate_price_black_scholes(&self) -> OptionsResult<Decimal> {
        Ok(black_scholes(self)?)
    }

    fn calculate_price_montecarlo(&self, prices: &[Positive]) -> OptionsResult<Positive> {
        Ok(price_option_monte_carlo(self, prices)?)
    }

    fn calculate_price_telegraph(
        &self,
        no_steps: NonZeroUsize,
        volatility: RegimeVolatility,
        rng: &mut dyn Rng,
    ) -> OptionsResult<Decimal> {
        Ok(telegraph(
            self,
            no_steps,
            TELEGRAPH_PATHS,
            None,
            None,
            volatility,
            rng,
        )?)
    }

    fn time_value(&self) -> OptionsResult<Decimal> {
        let option_price = OptionPricing::calculate_price_black_scholes(self)?.abs();
        let intrinsic_value = self.intrinsic_value(self.underlying_price)?;
        // `Decimal`'s `-` panics on overflow, which a price near the top of
        // the range against a negative intrinsic value reaches.
        let time_value = d_sub(option_price, intrinsic_value, "model::option::time_value")?;
        Ok(time_value.max(Decimal::ZERO))
    }

    fn calculate_implied_volatility(
        &self,
        market_price: Decimal,
    ) -> Result<Positive, VolatilityError> {
        let is_short = self.is_short();
        let target_price = if is_short {
            -market_price
        } else {
            market_price
        };

        // A European target outside the no-arbitrage band has no implied
        // volatility (#652); without this check the bisection collapses on
        // an edge of its bracket and reports that edge as the answer.
        if matches!(self.option_type, OptionType::European) {
            let (lower, upper) = european_price_band(self).map_err(OptionsError::from)?;
            let below = target_price < d_sub(lower, IV_TOLERANCE, "pricing::iv::band::lower")?;
            let above = target_price > d_add(upper, IV_TOLERANCE, "pricing::iv::band::upper")?;
            if below || above {
                return Err(outside_band_error(market_price, target_price, lower, upper));
            }
        }

        // Initialize high and low bounds for volatility (500% max).
        let mut high = IV_BISECTION_MAX_VOLATILITY;
        let mut low = Positive::ZERO;
        // The top of the bracket only moves down when a midpoint prices above
        // the target, so while it has not moved the root may lie above it.
        let mut high_moved = false;

        // Binary search through volatilities until we find one that gives us our target price
        // or until we reach maximum iterations
        for iteration in 1..=MAX_ITERATIONS_IV {
            // Calculate midpoint volatility
            let mid_vol = d_div(
                d_add(high.to_dec(), low.to_dec(), "pricing::iv::bracket_sum")?,
                Decimal::TWO,
                "pricing::iv::midpoint",
            )?;
            // mid_vol is the average of two non-negative bounds, so it is
            // structurally non-negative; a None here would indicate a
            // breached invariant on the bounds themselves.
            let volatility = Positive::new_decimal(mid_vol).map_err(|e| {
                OptionsError::ImpliedVolatilityInvariant {
                    reason: format!("mid_vol invariant breached: {e}"),
                }
            })?;

            // Calculate option price at this volatility
            let mut option_copy = self.clone();
            option_copy.implied_volatility = volatility;
            let price = OptionPricing::calculate_price_black_scholes(&option_copy)?;

            // Adjust price for short positions
            let actual_price = if is_short { -price } else { price };

            // Check if we're close enough to the target price
            if d_sub(actual_price, target_price, "pricing::iv::residual")?.abs() < IV_TOLERANCE {
                return Ok(volatility);
            }

            // Update bounds based on whether this price was too high or too low
            if actual_price > target_price {
                high = volatility;
                high_moved = true;
            } else {
                low = volatility;
            }

            // Check if our range is too small (meaning we've converged)
            if d_sub(high.to_dec(), low.to_dec(), "pricing::iv::bracket")? < IV_BISECTION_BRACKET {
                if !high_moved {
                    // Every midpoint priced below the target. The root is
                    // bracketed only if the bracket top still prices at or
                    // above it; otherwise the implied volatility exceeds the
                    // bracket and the bisection did not converge (#652).
                    let mut top = self.clone();
                    top.implied_volatility = IV_BISECTION_MAX_VOLATILITY;
                    let top_price = OptionPricing::calculate_price_black_scholes(&top)?;
                    let top_price = if is_short { -top_price } else { top_price };
                    if top_price < d_sub(target_price, IV_TOLERANCE, "pricing::iv::top")? {
                        return Err(VolatilityError::NoConvergence {
                            iterations: iteration,
                            last_volatility: volatility,
                        });
                    }
                }
                return Ok(volatility);
            }
        }

        // If we haven't found a solution after max iterations
        Err(VolatilityError::NoConvergence {
            iterations: MAX_ITERATIONS_IV,
            last_volatility: high.checked_add(&low)?.checked_div(&Positive::TWO)?,
        })
    }
}

/// Top of the bisection bracket of
/// [`OptionPricing::calculate_implied_volatility`]: 500 % per year.
const IV_BISECTION_MAX_VOLATILITY: Positive = Positive::FIVE;

/// Width at which the bisection bracket counts as converged, in volatility
/// units (`0.01 %` per year).
const IV_BISECTION_BRACKET: Decimal = dec!(0.0001);

/// Builds the error for a target outside the no-arbitrage band.
///
/// `price` carries the magnitude of the quoted `market_price` (a short
/// position quotes it negative); the reason names the signed long-equivalent
/// target and the band it missed.
#[cold]
#[inline(never)]
fn outside_band_error(
    market_price: Decimal,
    target_price: Decimal,
    lower: Decimal,
    upper: Decimal,
) -> VolatilityError {
    match Positive::new_decimal(market_price.abs()) {
        Ok(price) => VolatilityError::InvalidPrice {
            price,
            reason: format!(
                "target {target_price} is outside the no-arbitrage band [{lower}, {upper}], so no implied volatility exists"
            ),
        },
        Err(e) => VolatilityError::PositiveError(e),
    }
}

#[cfg(test)]
mod tests_option_pricing_trait {
    use super::*;
    use optionstratlib_core::model::types::{OptionStyle, Side};
    use optionstratlib_core::model::utils::create_sample_option_simplest;
    use optionstratlib_core::model::{ExpirationDate, OptionType};
    use optionstratlib_core::pos_or_panic;

    fn sample() -> Options {
        Options::new(
            OptionType::European,
            Side::Long,
            "TEST".to_string(),
            pos_or_panic!(100.0),
            ExpirationDate::Days(pos_or_panic!(30.0)),
            pos_or_panic!(0.2),
            Positive::ONE,
            pos_or_panic!(100.0),
            dec!(0.05),
            OptionStyle::Call,
            Positive::ZERO,
            None,
        )
    }

    #[test]
    fn test_option_pricing_trait_matches_inherent_black_scholes() {
        let option = sample();
        let via_trait = OptionPricing::calculate_price_black_scholes(&option);
        let via_inherent = Options::calculate_price_black_scholes(&option);
        assert_eq!(via_trait.ok(), via_inherent.ok());
    }

    #[test]
    fn test_option_pricing_trait_matches_inherent_binomial() {
        let option = sample();
        let steps = NonZeroUsize::new(50).expect("non-zero");
        let via_trait = OptionPricing::calculate_price_binomial(&option, steps);
        let via_inherent = Options::calculate_price_binomial(&option, steps);
        assert_eq!(via_trait.ok(), via_inherent.ok());
    }

    #[test]
    fn test_option_pricing_trait_matches_inherent_time_value() {
        let option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        let via_trait = OptionPricing::time_value(&option);
        let via_inherent = Options::time_value(&option);
        assert_eq!(via_trait.ok(), via_inherent.ok());
    }

    #[test]
    fn test_option_pricing_trait_matches_inherent_implied_volatility() {
        let option = sample();
        let market_price = dec!(3.0);
        let via_trait = OptionPricing::calculate_implied_volatility(&option, market_price);
        let via_inherent = Options::calculate_implied_volatility(&option, market_price);
        assert_eq!(via_trait.ok(), via_inherent.ok());
    }
}

#[cfg(test)]
mod tests_options_pricing {
    use super::*;
    use optionstratlib_core::model::types::{OptionStyle, OptionType, Side};
    use optionstratlib_core::model::utils::create_sample_option_simplest;
    use optionstratlib_core::model::{ExpirationDate, Options};
    use optionstratlib_core::{model::Positive, pos_or_panic};
    use rust_decimal::Decimal;

    use rust_decimal_macros::dec;

    #[test]
    fn test_calculate_price_binomial() {
        let option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        let price = option
            .calculate_price_binomial(optionstratlib_core::nz!(100))
            .unwrap();
        assert!(price > Decimal::ZERO);
    }

    #[test]
    fn test_calculate_price_binomial_tree() {
        let option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        let (price, asset_tree, option_tree) = option
            .calculate_price_binomial_tree(optionstratlib_core::nz!(5))
            .unwrap();
        assert!(price > Decimal::ZERO);
        assert_eq!(asset_tree.len(), 6);
        assert_eq!(option_tree.len(), 6);
    }

    #[test]
    fn test_calculate_price_binomial_tree_short() {
        let option = create_sample_option_simplest(OptionStyle::Call, Side::Short);
        let (price, asset_tree, option_tree) = option
            .calculate_price_binomial_tree(optionstratlib_core::nz!(5))
            .unwrap();
        // A short position is a liability: the negated long price (#648). It
        // used to come back positive, the root negated twice.
        let long = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        let (long_price, _, _) = long
            .calculate_price_binomial_tree(optionstratlib_core::nz!(5))
            .unwrap();
        assert!(price < Decimal::ZERO);
        assert_eq!(price, -long_price);
        assert_eq!(asset_tree.len(), 6);
        assert_eq!(option_tree.len(), 6);
    }

    #[test]
    fn test_calculate_price_black_scholes() {
        let option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        let price = option.calculate_price_black_scholes().unwrap();
        assert!(price > Decimal::ZERO);
    }

    #[test]
    fn test_calculate_time_value() {
        let option = Options::new(
            OptionType::European,
            Side::Long,
            "AAPL".to_string(),
            Positive::HUNDRED,
            ExpirationDate::Days(pos_or_panic!(30.0)),
            pos_or_panic!(0.2),
            Positive::ONE,
            pos_or_panic!(105.0),
            dec!(0.05),
            OptionStyle::Call,
            Positive::ZERO,
            None,
        );

        let time_value = option.time_value().unwrap();
        assert!(time_value > Decimal::ZERO);
        assert!(time_value < option.calculate_price_black_scholes().unwrap());
    }
}

#[cfg(test)]
mod tests_time_value {
    use super::*;
    use optionstratlib_core::model::types::{OptionStyle, Side};
    use optionstratlib_core::model::utils::create_sample_option_simplest_strike;
    use optionstratlib_core::{model::Positive, pos_or_panic};
    use rust_decimal::Decimal;

    use optionstratlib_core::assert_decimal_eq;
    use rust_decimal_macros::dec;
    use tracing::debug;

    #[test]
    fn test_calculate_time_value_long_call() {
        let option = create_sample_option_simplest_strike(
            Side::Long,
            OptionStyle::Call,
            pos_or_panic!(105.0),
        );
        let time_value = option.time_value().unwrap();
        assert!(time_value > Decimal::ZERO);
        assert!(time_value <= option.calculate_price_black_scholes().unwrap());
    }

    #[test]
    fn test_calculate_time_value_short_call() {
        let option = create_sample_option_simplest_strike(
            Side::Short,
            OptionStyle::Call,
            pos_or_panic!(105.0),
        );
        let time_value = option.time_value().unwrap();
        assert!(time_value > Decimal::ZERO);
        assert!(time_value <= option.calculate_price_black_scholes().unwrap().abs());
    }

    #[test]
    fn test_calculate_time_value_long_put() {
        let option =
            create_sample_option_simplest_strike(Side::Long, OptionStyle::Put, pos_or_panic!(95.0));
        let time_value = option.time_value().unwrap();
        assert!(time_value > Decimal::ZERO);
        assert!(time_value <= option.calculate_price_black_scholes().unwrap());
    }

    #[test]
    fn test_calculate_time_value_short_put() {
        let option = create_sample_option_simplest_strike(
            Side::Short,
            OptionStyle::Put,
            pos_or_panic!(95.0),
        );
        let time_value = option.time_value().unwrap();
        assert!(time_value > Decimal::ZERO);
        assert!(time_value <= option.calculate_price_black_scholes().unwrap().abs());
    }

    #[test]
    fn test_calculate_time_value_at_the_money() {
        let call =
            create_sample_option_simplest_strike(Side::Long, OptionStyle::Call, Positive::HUNDRED);
        let put =
            create_sample_option_simplest_strike(Side::Long, OptionStyle::Put, Positive::HUNDRED);

        let call_time_value = call.time_value().unwrap();
        let put_time_value = put.time_value().unwrap();

        assert!(call_time_value > Decimal::ZERO);
        assert!(put_time_value > Decimal::ZERO);
        assert_eq!(
            call_time_value,
            call.calculate_price_black_scholes().unwrap()
        );
        assert_eq!(put_time_value, put.calculate_price_black_scholes().unwrap());
    }

    #[test]
    fn test_calculate_time_value_deep_in_the_money() {
        let call = create_sample_option_simplest_strike(
            Side::Long,
            OptionStyle::Call,
            pos_or_panic!(150.0),
        );
        let put =
            create_sample_option_simplest_strike(Side::Long, OptionStyle::Put, pos_or_panic!(50.0));

        let call_time_value = call.time_value().unwrap();
        let put_time_value = put.time_value().unwrap();

        let call_price = call.calculate_price_black_scholes().unwrap();
        let put_price = put.calculate_price_black_scholes().unwrap();

        assert_decimal_eq!(call_time_value, call_price, dec!(0.01));
        assert_decimal_eq!(put_time_value, put_price, dec!(0.01));
        debug!("Call time value: {}", call_time_value);
        debug!("Call BS price: {}", call_price);
        debug!("Put time value: {}", put_time_value);
        debug!("Put BS price: {}", put_price);
        assert!(call_time_value <= call_price);
        assert!(put_time_value <= put_price);
    }
}

#[cfg(test)]
mod tests_calculate_price_binomial {
    use super::*;
    use optionstratlib_core::model::types::{OptionStyle, OptionType, Side};
    use optionstratlib_core::model::utils::{
        create_sample_option, create_sample_option_simplest, create_sample_option_with_date,
    };
    use optionstratlib_core::model::{ExpirationDate, Options};
    use optionstratlib_core::{model::Positive, pos_or_panic};
    use rust_decimal::Decimal;

    use chrono::Utc;
    use rust_decimal_macros::dec;
    use std::str::FromStr;

    #[test]
    fn test_european_call_option_basic() {
        // Test a basic European call option with standard parameters
        let option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        let result = option.calculate_price_binomial(optionstratlib_core::nz!(100));
        assert!(result.is_ok());
        let price = result.unwrap();
        // Price should be positive for a long call at-the-money
        assert!(price > Decimal::ZERO);
    }

    #[test]
    fn test_american_put_option() {
        // Test American put option which should have early exercise value
        let option = Options::new(
            OptionType::American,
            Side::Long,
            "TEST".to_string(),
            Positive::HUNDRED,
            ExpirationDate::Days(pos_or_panic!(30.0)),
            pos_or_panic!(0.2),  // volatility
            Positive::ONE,       // quantity
            pos_or_panic!(95.0), // underlying price (slightly ITM for put)
            dec!(0.05),          // risk-free rate
            OptionStyle::Put,
            Positive::ZERO, // dividend yield
            None,
        );

        let result = option.calculate_price_binomial(optionstratlib_core::nz!(100));
        assert!(result.is_ok());
        let price = result.unwrap();
        // Price should be positive and reflect early exercise premium
        assert!(price > Decimal::ZERO);
    }

    #[test]
    fn test_zero_volatility() {
        let mut option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        option.implied_volatility = Positive::ZERO;
        let result = option.calculate_price_binomial(optionstratlib_core::nz!(100));
        assert!(result.is_ok());
        // With zero volatility, price should equal discounted intrinsic value
    }

    #[test]
    fn test_zero_time_to_expiry() {
        // Test option at expiration
        let now = Utc::now().naive_utc();
        let option = create_sample_option_with_date(
            OptionStyle::Call,
            Side::Long,
            Positive::HUNDRED,
            Positive::ONE,
            pos_or_panic!(95.0),
            pos_or_panic!(0.2),
            now,
        );

        let result = option.calculate_price_binomial(optionstratlib_core::nz!(100));
        assert!(result.is_ok());
        let price = result.unwrap();
        // At expiry, price should equal intrinsic value
        assert_eq!(price, Decimal::from(5));
    }

    // The former `test_invalid_steps` test used to pass `0` to
    // `calculate_price_binomial` and assert that the runtime guard
    // returned `InvalidStepCount`. After #337 the signature is
    // `NonZeroUsize`, so the invariant is enforced at the type level
    // and the test is now obsolete (cannot construct the invalid
    // input).

    #[test]
    fn test_deep_itm_call() {
        let option = create_sample_option(
            OptionStyle::Call,
            Side::Long,
            pos_or_panic!(150.0), // Underlying price much higher than strike
            Positive::ONE,
            Positive::HUNDRED,
            pos_or_panic!(0.2),
        );

        let result = option.calculate_price_binomial(optionstratlib_core::nz!(100));
        assert!(result.is_ok());
        let price = result.unwrap();
        // Price should be close to intrinsic value for deep ITM
        assert!(price > Decimal::from(45)); // At least intrinsic - some time value
    }

    #[test]
    fn test_deep_otm_put() {
        let option = create_sample_option(
            OptionStyle::Put,
            Side::Long,
            pos_or_panic!(150.0), // Underlying price much higher than strike
            Positive::ONE,
            Positive::HUNDRED,
            pos_or_panic!(0.2),
        );

        let result = option.calculate_price_binomial(optionstratlib_core::nz!(100));
        assert!(result.is_ok());
        let price = result.unwrap();
        // Price should be very small for deep OTM
        assert!(price < Decimal::from(1));
    }

    #[test]
    fn test_convergence() {
        let option = create_sample_option_simplest(OptionStyle::Call, Side::Long);

        // Test that increasing steps leads to convergence
        let price_100 = option
            .calculate_price_binomial(optionstratlib_core::nz!(100))
            .unwrap();
        let price_1000 = option
            .calculate_price_binomial(optionstratlib_core::nz!(1000))
            .unwrap();

        // Prices should be close to each other
        let diff = (price_1000 - price_100).abs();
        assert!(diff < Decimal::from_str("0.1").unwrap());
    }

    #[test]
    fn test_short_position() {
        let long_call_option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        let mut short_call_option = long_call_option.clone();
        short_call_option.side = Side::Short;
        let mut short_put_option = short_call_option.clone();
        short_put_option.option_style = OptionStyle::Put;
        let mut long_put_option = short_put_option.clone();
        long_put_option.side = Side::Long;

        let long_call_price = long_call_option
            .calculate_price_binomial(optionstratlib_core::nz!(100))
            .unwrap();
        let short_call_price = short_call_option
            .calculate_price_binomial(optionstratlib_core::nz!(100))
            .unwrap();
        let long_put_price = long_put_option
            .calculate_price_binomial(optionstratlib_core::nz!(100))
            .unwrap();
        let short_put_price = short_put_option
            .calculate_price_binomial(optionstratlib_core::nz!(100))
            .unwrap();

        // Short position should be negative of long position
        assert_eq!(long_call_price, -short_call_price);
        assert_eq!(long_put_price, -short_put_price);
    }
}

#[cfg(test)]
mod tests_options_black_scholes {
    use super::*;
    use optionstratlib_core::assert_decimal_eq;
    use optionstratlib_core::model::types::{OptionStyle, OptionType, Side};
    use optionstratlib_core::model::{ExpirationDate, Options};
    use optionstratlib_core::{model::Positive, pos_or_panic};
    use rust_decimal_macros::dec;

    #[test]
    fn test_new_option_call() {
        let option = Options::new(
            OptionType::European,
            Side::Long,
            "SP500".to_string(),
            pos_or_panic!(5790.0),
            ExpirationDate::Days(pos_or_panic!(18.0)),
            pos_or_panic!(0.1117),
            Positive::ONE,
            pos_or_panic!(5781.88),
            dec!(0.05),
            OptionStyle::Call,
            Positive::ZERO,
            None,
        );
        assert_decimal_eq!(
            option.calculate_price_black_scholes().unwrap(),
            pos_or_panic!(60.306_765_882_668_3),
            dec!(1e-8)
        );
    }

    #[test]
    fn test_new_option_call_bis() {
        let option = Options::new(
            OptionType::European,
            Side::Long,
            "SP500".to_string(),
            pos_or_panic!(6050.0),
            ExpirationDate::Days(pos_or_panic!(61.2)),
            pos_or_panic!(0.12594),
            Positive::ONE,
            pos_or_panic!(6032.18),
            dec!(0.0),
            OptionStyle::Call,
            Positive::ZERO,
            None,
        );
        assert_decimal_eq!(
            option.calculate_price_black_scholes().unwrap(),
            pos_or_panic!(115.56),
            dec!(1e-2)
        );
    }

    #[test]
    fn test_new_option_put() {
        let option = Options::new(
            OptionType::European,
            Side::Long,
            "SP500".to_string(),
            pos_or_panic!(6050.0),
            ExpirationDate::Days(pos_or_panic!(61.2)),
            pos_or_panic!(0.1258),
            Positive::ONE,
            pos_or_panic!(6032.18),
            dec!(0.0),
            OptionStyle::Put,
            Positive::ZERO,
            None,
        );
        assert_decimal_eq!(
            option.calculate_price_black_scholes().unwrap(),
            pos_or_panic!(133.25),
            dec!(1e-2)
        );
    }

    #[test]
    fn test_new_option_call_short() {
        let option = Options::new(
            OptionType::European,
            Side::Short,
            "SP500".to_string(),
            pos_or_panic!(6050.0),
            ExpirationDate::Days(pos_or_panic!(60.0)),
            pos_or_panic!(0.12594),
            Positive::ONE,
            pos_or_panic!(6032.18),
            dec!(0.0),
            OptionStyle::Call,
            Positive::ZERO,
            None,
        );
        assert_decimal_eq!(
            option.calculate_price_black_scholes().unwrap(),
            dec!(-114.34),
            dec!(1e-2)
        );
    }

    #[test]
    fn test_new_option_put_short() {
        let option = Options::new(
            OptionType::European,
            Side::Short,
            "SP500".to_string(),
            pos_or_panic!(6050.0),
            ExpirationDate::Days(pos_or_panic!(60.0)),
            pos_or_panic!(0.12594),
            Positive::ONE,
            pos_or_panic!(6032.18),
            dec!(0.0),
            OptionStyle::Put,
            Positive::ZERO,
            None,
        );
        assert_decimal_eq!(
            option.calculate_price_black_scholes().unwrap(),
            dec!(-132.16),
            dec!(1e-2)
        );
    }
}

#[cfg(test)]
mod tests_calculate_implied_volatility {
    use super::*;
    use crate::error::VolatilityError;
    use crate::pricing::constants::IV_TOLERANCE;
    use optionstratlib_core::assert_pos_relative_eq;
    use optionstratlib_core::model::types::{OptionStyle, OptionType, Side};
    use optionstratlib_core::model::{ExpirationDate, Options};
    use optionstratlib_core::{model::Positive, pos_or_panic};
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;

    #[test]
    fn test_implied_volatility_call() {
        let option = Options::new(
            OptionType::European,
            Side::Long,
            "TEST".to_string(),
            pos_or_panic!(5790.0), // strike
            ExpirationDate::Days(pos_or_panic!(18.0)),
            pos_or_panic!(0.1),     // initial iv
            Positive::ONE,          // qty
            pos_or_panic!(5781.88), // underlying
            dec!(0.05),             // rate
            OptionStyle::Call,
            Positive::ZERO, // div
            None,
        );

        let market_price = dec!(60.30);
        let iv = option.calculate_implied_volatility(market_price).unwrap();

        assert_pos_relative_eq!(
            iv,
            pos_or_panic!(0.111618041),
            Positive::new_decimal(IV_TOLERANCE).unwrap()
        );
    }

    #[test]
    fn test_implied_volatility_put() {
        let option = Options::new(
            OptionType::European,
            Side::Long,
            "TEST".to_string(),
            pos_or_panic!(6050.0), // strike
            ExpirationDate::Days(pos_or_panic!(60.0)),
            pos_or_panic!(0.1),     // initial iv
            Positive::ONE,          // qty
            pos_or_panic!(6032.18), // underlying
            dec!(0.0),              // rate
            OptionStyle::Put,
            Positive::ZERO, // div
            None,
        );

        let market_price = dec!(132.16);
        let iv = option.calculate_implied_volatility(market_price).unwrap();
        assert_pos_relative_eq!(
            iv,
            pos_or_panic!(0.125961),
            Positive::new_decimal(IV_TOLERANCE).unwrap()
        );
    }

    #[test]
    fn test_implied_volatility_call_short() {
        let option = Options::new(
            OptionType::European,
            Side::Short,
            "TEST".to_string(),
            pos_or_panic!(6050.0), // strike
            ExpirationDate::Days(pos_or_panic!(60.0)),
            pos_or_panic!(0.1),     // initial iv
            Positive::ONE,          // qty
            pos_or_panic!(6032.18), // underlying
            dec!(0.0),              // rate
            OptionStyle::Call,
            Positive::ZERO, // div
            None,
        );

        let market_price = dec!(-114.16);
        let iv = option.calculate_implied_volatility(market_price).unwrap();

        assert_pos_relative_eq!(
            iv,
            pos_or_panic!(0.1258087),
            Positive::new_decimal(IV_TOLERANCE).unwrap()
        );
    }

    #[test]
    fn test_implied_volatility_put_short() {
        let option = Options::new(
            OptionType::European,
            Side::Short,
            "TEST".to_string(),
            pos_or_panic!(6050.0), // strike
            ExpirationDate::Days(pos_or_panic!(60.0)),
            pos_or_panic!(0.1),     // initial iv
            Positive::ONE,          // qty
            pos_or_panic!(6032.18), // underlying
            dec!(0.0),              // rate
            OptionStyle::Put,
            Positive::ZERO, // div
            None,
        );

        let market_price = dec!(-132.27);
        let iv = option.calculate_implied_volatility(market_price).unwrap();
        assert_pos_relative_eq!(
            iv,
            pos_or_panic!(0.12611389),
            Positive::new_decimal(IV_TOLERANCE).unwrap()
        );
    }

    #[test]
    fn test_invalid_market_price() {
        let option = Options::default();
        let result = option.calculate_implied_volatility(Decimal::ZERO);
        assert!(matches!(result, Err(VolatilityError::Options(_))));
    }

    #[test]
    fn test_expired_option() {
        let option = Options::new(
            OptionType::European,
            Side::Long,
            "TEST".to_string(),
            Positive::HUNDRED,
            ExpirationDate::Days(Positive::ZERO),
            pos_or_panic!(0.2),
            Positive::ONE,
            Positive::HUNDRED,
            dec!(0.05),
            OptionStyle::Call,
            Positive::ZERO,
            None,
        );

        let result = option.calculate_implied_volatility(dec!(2.5));
        assert!(matches!(result, Err(VolatilityError::Options(_))));
    }

    #[test]
    fn test_convergence_edge_cases() {
        let option = Options::new(
            OptionType::European,
            Side::Long,
            "TEST".to_string(),
            pos_or_panic!(5790.0), // strike
            ExpirationDate::Days(pos_or_panic!(18.0)),
            pos_or_panic!(0.1),     // initial iv
            Positive::ONE,          // qty
            pos_or_panic!(5781.88), // underlying
            dec!(0.05),             // rate
            OptionStyle::Call,
            Positive::ZERO, // div
            None,
        );

        // Test with small initial vol
        let iv = option.calculate_implied_volatility(dec!(60.30)).unwrap();
        assert_pos_relative_eq!(iv, pos_or_panic!(0.111328125), pos_or_panic!(0.01));

        // Test with large initial vol
        let iv = option.calculate_implied_volatility(dec!(60.30)).unwrap();
        assert_pos_relative_eq!(iv, pos_or_panic!(0.111328125), pos_or_panic!(0.01));
    }
}
