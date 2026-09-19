/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 19/9/25
******************************************************************************/

//! Pricing capability for the core [`crate::Options`] contract.
//!
//! [`OptionPricing`] is the pricing-owned extension trait that carries every
//! model-based valuation of an [`crate::Options`] contract: binomial lattice,
//! Black-Scholes closed form, Monte Carlo over supplied paths, the telegraph
//! finite-difference kernel, the time-value decomposition and the implied
//! volatility bisection. The core type keeps only contract data and payoff
//! arithmetic; this trait is where the numerics attach to it.
//!
//! The inherent methods with the same names on [`crate::Options`] forward here and
//! are the 0.21 compatibility surface. Importing this trait (directly or
//! through the prelude) is the canonical 0.22 form:
//!
//! ```rust
//! use optionstratlib::pricing::OptionPricing;
//! use optionstratlib::{ExpirationDate, OptionStyle, OptionType, Options, Side};
//! use positive::{Positive, pos_or_panic};
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
//! # Ok::<(), optionstratlib::error::OptionsError>(())
//! ```

use crate::constants::{IV_TOLERANCE, MAX_ITERATIONS_IV};
use crate::error::{OptionsError, OptionsResult, PricingError, VolatilityError};
use crate::model::Options;
use crate::model::decimal::d_sub;
use crate::model::types::Side;
use crate::pricing::monte_carlo::price_option_monte_carlo;
use crate::pricing::{
    BinomialPricingParams, black_scholes, generate_binomial_tree, price_binomial, telegraph,
};
use positive::Positive;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::num::NonZeroUsize;

/// Result type for binomial tree pricing models, containing:
/// - The option price
/// - Price tree (asset price evolution)
/// - Option value tree (option value at each node)
pub type PriceBinomialTree = OptionsResult<(Decimal, Vec<Vec<Decimal>>, Vec<Vec<Decimal>>)>;

/// Model-based valuation of an [`crate::Options`] contract.
///
/// Every method is a pure function of the contract data held by the
/// implementor plus the method's own parameters; none of them mutates the
/// contract. Long positions report positive prices, short positions report
/// the negated price of the equivalent long contract.
///
/// The trait is implemented for [`crate::Options`] by the pricing layer. The
/// inherent methods of the same names on [`crate::Options`] forward to these
/// implementations and exist only so that callers that never imported this
/// trait keep compiling.
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
    ///   computational cost. See [`crate::constants::DEFAULT_BINOMIAL_STEPS`]
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

    /// Calculates option price using the Telegraph equation approach.
    ///
    /// This method implements a finite-difference method based on the Telegraph equation
    /// to price options. This approach can handle a variety of option styles and types,
    /// including path-dependent options.
    ///
    /// # Parameters
    ///
    /// * `no_steps` - The number of discrete time steps to use in the model,
    ///   as a [`NonZeroUsize`] so zero is structurally invalid at the type
    ///   level. Higher values increase precision but also computational cost.
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
    /// `PricingError::ExpirationDate` or `PricingError::MethodError`
    /// when the finite-difference kernel fails to converge.
    fn calculate_price_telegraph(&self, no_steps: NonZeroUsize) -> OptionsResult<Decimal>;

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
    /// # Errors
    ///
    /// Returns [`VolatilityError::PositiveError`] when the midpoint
    /// volatility breaches the `Positive` invariant,
    /// [`VolatilityError::NoConvergence`] when the bisection exhausts
    /// `MAX_ITERATIONS_IV` without matching the target price, or propagates
    /// [`VolatilityError::Options`] from the underlying Black–Scholes
    /// evaluation (wrapped as [`OptionsError::ImpliedVolatilityInvariant`]
    /// when the invariant check fails).
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
        let root = option_tree
            .first()
            .and_then(|row| row.first())
            .copied()
            .ok_or(PricingError::BinomialNodeMissing {
                node: "option[0][0]",
            })?;
        let price = match self.side {
            Side::Long => root,
            Side::Short => -root,
        };
        Ok((price, asset_tree, option_tree))
    }

    fn calculate_price_black_scholes(&self) -> OptionsResult<Decimal> {
        Ok(black_scholes(self)?)
    }

    fn calculate_price_montecarlo(&self, prices: &[Positive]) -> OptionsResult<Positive> {
        Ok(price_option_monte_carlo(self, prices)?)
    }

    fn calculate_price_telegraph(&self, no_steps: NonZeroUsize) -> OptionsResult<Decimal> {
        Ok(telegraph(self, no_steps, None, None)?)
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

        // Initialize high and low bounds for volatility (500% max).
        let mut high = Positive::new(5.0)?;
        let mut low = Positive::ZERO;

        // Binary search through volatilities until we find one that gives us our target price
        // or until we reach maximum iterations
        for _ in 0..MAX_ITERATIONS_IV {
            // Calculate midpoint volatility
            let mid_vol = (high.to_dec() + low.to_dec()) / Decimal::TWO;
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
            if (actual_price - target_price).abs() < IV_TOLERANCE {
                return Ok(volatility);
            }

            // Update bounds based on whether this price was too high or too low
            if actual_price > target_price {
                high = volatility;
            } else {
                low = volatility;
            }

            // Check if our range is too small (meaning we've converged)
            if (high - low).to_dec() < dec!(0.0001) {
                return Ok(volatility);
            }
        }

        // If we haven't found a solution after max iterations
        Err(VolatilityError::NoConvergence {
            iterations: MAX_ITERATIONS_IV,
            last_volatility: (high + low) / Positive::TWO,
        })
    }
}

#[cfg(test)]
mod tests_option_pricing_trait {
    use super::*;
    use crate::model::types::OptionStyle;
    use crate::model::utils::create_sample_option_simplest;
    use crate::{ExpirationDate, OptionType};
    use positive::pos_or_panic;

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
