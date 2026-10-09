//! # Probability Analysis Module
//!
//! This module provides functionality for analyzing the probability metrics of option trading strategies.
//! It includes tools for calculating expected values, profit probabilities, and risk-reward ratios based
//! on various market conditions like volatility and price trends.
//!
//! The `ProbabilityAnalysis` trait extends the `Strategies` and `Profit` traits to provide
//! comprehensive probability analysis capabilities for option strategies.

use optionstratlib_core::model::Positive;
use optionstratlib_core::model::decimal::{d_add, d_mul, d_sub};
#[cfg(test)]
use optionstratlib_core::pos_or_panic;

use crate::error::strategies::StrategyError;
use crate::strategies::base::Strategies;
use crate::strategies::probabilities::analysis::StrategyProbabilityAnalysis;
use optionstratlib_analytics::analytics::ProfitLossRange;
use optionstratlib_analytics::analytics::probability::{
    PriceTrend, VolatilityAdjustment, calculate_single_point_probability,
};
use optionstratlib_analytics::error::probability::{
    ProbabilityCalculationErrorKind, ProbabilityError,
};
use optionstratlib_core::model::decimal::decimal_to_f64;
use optionstratlib_pricing::pricing::Profit;
use rust_decimal::Decimal;
use tracing::warn;

/// Trait for analyzing probabilities and risk metrics of option strategies
///
/// This trait provides methods to analyze the probability characteristics of options strategies,
/// including probability of profit/loss, expected value, and risk-reward metrics.
///
/// # Type Requirements
///
/// Implementors must also implement:
/// - The `Strategies` trait, which provides access to strategy configuration
/// - The `Profit` trait, which provides profit calculation capabilities
///
/// # Key Features
///
/// - Calculate probability of profit for option strategies
/// - Compute expected values with adjustments for volatility and price trends
/// - Determine break-even points and risk-reward ratios
/// - Analyze extreme outcome probabilities (max profit and max loss scenarios)
///
pub trait ProbabilityAnalysis: Strategies + Profit {
    /// Implied volatility the strategy is priced at when a caller passes no
    /// [`VolatilityAdjustment`].
    ///
    /// The probability model is log-normal with a single volatility, so a
    /// strategy whose legs carry different implied volatilities (skew) needs
    /// one number for it. This is the implied volatility of the leg whose
    /// strike is closest to the underlying price, the at-the-money leg; when
    /// two legs are equally close, the lower strike wins, so the answer does
    /// not depend on the order the legs were added in.
    ///
    /// Every method in this trait that takes `volatility_adj: None` uses this
    /// value with no widening (`std_dev_adjustment = 0`).
    ///
    /// # Errors
    ///
    /// Returns [`ProbabilityError::CalculationError`] with
    /// [`ProbabilityCalculationErrorKind::VolatilityAdjustmentError`] when the
    /// strategy has no option position to read a volatility from, and
    /// propagates the strategy's own error when its positions cannot be
    /// enumerated.
    fn reference_volatility(&self) -> Result<Positive, ProbabilityError> {
        let spot = self.get_underlying_price()?.to_dec();
        let positions = self.get_positions().map_err(StrategyError::from)?;
        let mut closest: Option<(Decimal, Positive, Positive)> = None;
        for position in positions {
            let strike = position.option.strike_price;
            let distance = strike
                .to_dec()
                .checked_sub(spot)
                .ok_or_else(|| {
                    ProbabilityError::CalculationError(
                        ProbabilityCalculationErrorKind::VolatilityAdjustmentError {
                            reason: format!(
                                "distance from strike {strike} to spot {spot} overflows"
                            ),
                        },
                    )
                })?
                .abs();
            let better = match closest {
                None => true,
                Some((best_distance, best_strike, _)) => {
                    distance < best_distance || (distance == best_distance && strike < best_strike)
                }
            };
            if better {
                closest = Some((distance, strike, position.option.implied_volatility));
            }
        }
        closest.map(|(_, _, volatility)| volatility).ok_or_else(|| {
            ProbabilityError::CalculationError(
                ProbabilityCalculationErrorKind::VolatilityAdjustmentError {
                    reason: "strategy has no option position to take a reference volatility from"
                        .to_string(),
                },
            )
        })
    }

    /// Calculate probability analysis for a strategy
    ///
    /// Performs a comprehensive probability analysis for an option strategy, taking into
    /// account optional volatility adjustments and price trend parameters.
    ///
    /// # Parameters
    ///
    /// - `volatility_adj`: Volatility to price at; `None` uses
    ///   [`ProbabilityAnalysis::reference_volatility`] with no widening.
    /// - `trend`: Optional price trend parameters indicating market direction bias
    ///
    /// # Returns
    ///
    /// - `Result<StrategyProbabilityAnalysis, ProbabilityError>`: Structured analysis results or an error
    ///
    /// # Analysis Components
    ///
    /// The returned analysis includes:
    /// - Probability of profit
    /// - Probability of reaching maximum profit
    /// - Probability of suffering maximum loss
    /// - Expected value
    /// - Break-even points
    /// - Risk-reward ratio
    ///
    /// # Errors
    ///
    /// Propagates any [`ProbabilityError`] returned by
    /// `probability_of_profit`, `probability_of_loss`,
    /// `calculate_extreme_probabilities` or `expected_value`;
    /// typically [`ProbabilityError::CalculationError`] when an
    /// underlying break-even or payoff evaluation fails.
    fn analyze_probabilities(
        &self,
        volatility_adj: Option<VolatilityAdjustment>,
        trend: Option<PriceTrend>,
    ) -> Result<StrategyProbabilityAnalysis, ProbabilityError> {
        let break_even_points = self.get_break_even_points()?;
        // If both parameters are None, return default probabilities based on profit ranges
        if volatility_adj.is_none() && trend.is_none() {
            let probability_of_profit = self.probability_of_profit(None, None)?;
            let expected_value = self.expected_value(None, None)?;

            return Ok(StrategyProbabilityAnalysis {
                probability_of_profit,
                probability_of_max_profit: Positive::ZERO, // Default value when no volatility adjustment
                probability_of_max_loss: Positive::ZERO, // Default value when no volatility adjustment
                expected_value,
                break_even_points: break_even_points.to_vec(),
                risk_reward_ratio: Positive::new_decimal(self.get_profit_ratio()?)?,
            });
        }

        // If we have adjustments, calculate with them
        let probability_of_profit = self.probability_of_profit(volatility_adj, trend.clone())?;
        let expected_value = self.expected_value(volatility_adj, trend.clone())?;
        let (prob_max_profit, prob_max_loss) =
            self.calculate_extreme_probabilities(volatility_adj, trend)?;
        let risk_reward_ratio = Positive::new_decimal(self.get_profit_ratio()?)?;

        Ok(StrategyProbabilityAnalysis {
            probability_of_profit,
            probability_of_max_profit: prob_max_profit,
            probability_of_max_loss: prob_max_loss,
            expected_value,
            break_even_points: break_even_points.to_vec(),
            risk_reward_ratio,
        })
    }

    /// This function calculates the expected value of an option strategy
    /// based on an underlying price, volatility adjustments, and price trends.
    ///
    /// # Parameters
    /// - `volatility_adj`: Volatility to price at; `None` uses
    ///   [`ProbabilityAnalysis::reference_volatility`] with no widening.
    /// - `trend`: An optional `PriceTrend` parameter, which indicates the
    ///   annual drift rate and the confidence level for the trend.
    ///
    /// # Returns
    /// - `Result<Decimal, ProbabilityError>`: On success, the probability-weighted
    ///   profit at expiration. It is signed: a negative value means the strategy
    ///   loses money on average under the given volatility and trend, and is
    ///   reported as such rather than floored to zero (#623). With a zero
    ///   volatility and no widening it is the profit at the current underlying
    ///   price, of either sign.
    ///
    /// The `trend` drift enters through the distribution the probabilities are
    /// taken from; the sum is the expectation under that distribution and is
    /// not scaled again afterwards.
    ///
    /// The function performs the following operations:
    /// - Determines the pricing range using the underlying asset's price and steps based
    ///   on 1% increments of the current price.
    /// - Calculates the single-point probability for each price within the range using the
    ///   provided volatility adjustments and price trends.
    /// - Computes the expected value by summing up the product of calculated probabilities
    ///   and the strategy's profit at each price point.
    /// - Logs the calculated range with probabilities for diagnostic purposes.
    ///
    /// This function relies on several auxiliary methods and traits, such as
    /// `get_underlying_price`, `best_range_to_show`, and `calculate_profit_at`,
    /// which are defined in the module's traits and utilities.
    ///
    /// # Errors
    ///
    /// Returns [`ProbabilityError::CalculationError`] when the display
    /// range cannot be computed or when profit evaluation fails at a
    /// sample point, and propagates any [`ProbabilityError`] surfaced by
    /// `probability_at` (typically
    /// `ProbabilityCalculationErrorKind::InvalidProbabilityRange`
    /// for malformed volatility adjustments).
    ///
    /// Also returns
    /// [`ProbabilityCalculationErrorKind::InvalidProbability`] when the
    /// cumulative probability at a price is smaller than at the previous one.
    /// A distribution function is monotone over an ascending range, so this
    /// only happens when the inputs sit past the precision of the price model;
    /// it is reported rather than floored to zero, which would weight the
    /// expected value by a distribution that does not sum to one.
    ///
    /// Returns [`ProbabilityCalculationErrorKind::TrendError`] when the trend
    /// drift has no `f64` form, and
    /// [`ProbabilityCalculationErrorKind::ExpectedValueError`] when the
    /// probability-weighted sum overflows `Decimal`.
    fn expected_value(
        &self,
        volatility_adj: Option<VolatilityAdjustment>,
        trend: Option<PriceTrend>,
    ) -> Result<Decimal, ProbabilityError> {
        // Special case: when volatility is zero, return the current value
        if let Some(ref vol_adj) = volatility_adj
            && vol_adj.base_volatility == Positive::ZERO
            && vol_adj.std_dev_adjustment == Positive::ZERO
        {
            return Ok(self.calculate_profit_at(self.get_underlying_price()?)?);
        }

        let volatility = resolve_volatility(self, volatility_adj)?;
        let step = self.get_underlying_price()?.checked_div_f64(100.0)?;
        let range = self.get_best_range_to_show(step)?;
        let expiration = *self.get_expiration().values().next().ok_or_else(|| {
            StrategyError::empty_collection("expected_value: no expiration on strategy")
        })?;

        let mut probabilities = Vec::with_capacity(range.len());
        let mut last_prob = Decimal::ZERO;
        let underlying_price = self.get_underlying_price()?;

        for price in range.iter() {
            let prob = calculate_single_point_probability(
                underlying_price,
                price,
                volatility,
                trend.clone(),
                expiration,
                None,
            )?;

            // A distribution function is monotone over an ascending price
            // range, so each step's marginal mass is non-negative; equality is
            // a step carrying zero mass, which `sub_or_none` returns. The
            // prices stay distinct: two adjacent grid points far into a tail
            // can round to the same `Decimal` probability, and zero is still
            // the right mass for that step. A
            // smaller probability at the higher price is a result outside the
            // model's precision, not a property of the range: near
            // `Positive::MAX` with a volatility around `1e-28`, a ratio that
            // rounds to `0.9999999999999999999999999999` makes
            // `Decimal::checked_ln` return `+9e-28` where the true value is
            // `-1e-28`, which puts the lower price above the spot. Flooring
            // that to zero dropped the step silently and left the expected
            // value weighted by a distribution that does not sum to one, so it
            // is reported (#570, same rule as
            // `ProfitLossRange::calculate_probability`, #569).
            let marginal_prob = match prob.0.sub_or_none(&last_prob) {
                Some(marginal) => marginal,
                None => {
                    // The reported gap is formed from the nearest `f64` of
                    // each probability (#828); a failed conversion is its own
                    // error rather than a NaN payload.
                    let value = decimal_to_f64(prob.0.to_dec())? - decimal_to_f64(last_prob)?;
                    return Err(ProbabilityError::CalculationError(
                        ProbabilityCalculationErrorKind::InvalidProbability {
                            value,
                            reason: format!(
                                "cumulative probability at {price} ({}) is smaller than at \
                                 the previous price ({last_prob}); the inputs are outside \
                                 the precision of the price model",
                                prob.0
                            ),
                        },
                    ));
                }
            };
            probabilities.push(marginal_prob);
            last_prob = prob.0.to_dec();
        }

        // Summed in `Decimal` with checked arithmetic: the expected value is a
        // signed monetary quantity, and a negative one is a valid answer
        // ("this strategy loses money on average"), not a floor to zero
        // (#623). The drift of `trend` already shapes the distribution the
        // probabilities come from, so the sum is the expectation under that
        // trend and takes no further adjustment.
        let mut expected_value = Decimal::ZERO;
        let mut total_prob = Decimal::ZERO;
        for (price, prob) in range.iter().zip(probabilities.iter()) {
            let profit = self.calculate_profit_at(price)?;
            let weighted = d_mul(
                profit,
                prob.to_dec(),
                "expected_value: profit * probability",
            )?;
            expected_value = d_add(expected_value, weighted, "expected_value: sum")?;
            total_prob = d_add(total_prob, prob.to_dec(), "expected_value: probability sum")?;
        }

        let deviation = d_sub(
            total_prob,
            Decimal::ONE,
            "expected_value: probability deviation",
        )?;
        if deviation.abs() > Decimal::new(5, 2) {
            warn!(
                "Sum of probabilities ({}) deviates significantly from 1.0",
                total_prob
            );
        }
        Ok(expected_value)
    }

    /// Calculate probability of profit
    ///
    /// Calculates the probability that the option strategy will result in a profit at expiration.
    /// This method aggregates probabilities across all price ranges that would result in a profit.
    ///
    /// # Parameters
    ///
    /// - `volatility_adj`: Volatility to price at; `None` uses
    ///   [`ProbabilityAnalysis::reference_volatility`] with no widening.
    /// - `trend`: Optional price trend parameters
    ///
    /// # Returns
    ///
    /// - `Result<Positive, ProbabilityError>`: The probability of profit (between 0 and 1) or an error
    ///
    /// # Errors
    ///
    /// Returns [`ProbabilityError::RangeError`] when the profit ranges
    /// cannot be constructed, or propagates
    /// [`ProbabilityError::CalculationError`] from the probability
    /// integration over the computed ranges.
    fn probability_of_profit(
        &self,
        volatility_adj: Option<VolatilityAdjustment>,
        trend: Option<PriceTrend>,
    ) -> Result<Positive, ProbabilityError> {
        let volatility = resolve_volatility(self, volatility_adj)?;
        let mut sum_of_probabilities = Positive::ZERO;
        let ranges = self.get_profit_ranges()?;
        let option = self.one_option()?;
        let expiration = option.expiration_date;
        let risk_free_rate = option.risk_free_rate;
        let underlying_price = option.underlying_price;
        for mut range in ranges {
            range.calculate_probability(
                &underlying_price,
                volatility,
                trend.clone(),
                &expiration,
                Some(risk_free_rate),
            )?;
            sum_of_probabilities = sum_of_probabilities.checked_add(&range.probability)?;
        }
        Ok(sum_of_probabilities)
    }

    /// Calculate probability of loss
    ///
    /// Calculates the probability that the option strategy will result in a loss at expiration.
    /// This method aggregates probabilities across all price ranges that would result in a loss.
    ///
    /// # Parameters
    ///
    /// - `volatility_adj`: Volatility to price at; `None` uses
    ///   [`ProbabilityAnalysis::reference_volatility`] with no widening.
    /// - `trend`: Optional price trend parameters
    ///
    /// # Returns
    ///
    /// - `Result<Positive, ProbabilityError>`: The probability of loss (between 0 and 1) or an error
    ///
    /// # Errors
    ///
    /// Returns [`ProbabilityError::RangeError`] when the loss ranges
    /// cannot be constructed, or propagates
    /// [`ProbabilityError::CalculationError`] from the probability
    /// integration over the computed ranges.
    fn probability_of_loss(
        &self,
        volatility_adj: Option<VolatilityAdjustment>,
        trend: Option<PriceTrend>,
    ) -> Result<Positive, ProbabilityError> {
        let volatility = resolve_volatility(self, volatility_adj)?;
        let mut sum_of_probabilities = Positive::ZERO;
        let ranges = self.get_loss_ranges()?;
        let option = self.one_option()?;
        let expiration = option.expiration_date;
        let risk_free_rate = option.risk_free_rate;
        let underlying_price = option.underlying_price;
        for mut range in ranges {
            range.calculate_probability(
                &underlying_price,
                volatility,
                trend.clone(),
                &expiration,
                Some(risk_free_rate),
            )?;
            sum_of_probabilities = sum_of_probabilities.checked_add(&range.probability)?;
        }
        Ok(sum_of_probabilities)
    }

    /// Calculate extreme probabilities (max profit and max loss)
    ///
    /// Calculates the probabilities of reaching the maximum possible profit and
    /// suffering the maximum possible loss for the strategy.
    ///
    /// # Parameters
    ///
    /// - `volatility_adj`: Volatility to price at; `None` uses
    ///   [`ProbabilityAnalysis::reference_volatility`] with no widening.
    /// - `trend`: Optional price trend parameters
    ///
    /// # Returns
    ///
    /// - `Result<(Positive, Positive), ProbabilityError>`: A tuple containing (probability_of_max_profit,
    ///   probability_of_max_loss) or an error
    ///
    /// # Errors
    ///
    /// Propagates any [`ProbabilityError`] returned by
    /// `probability_of_profit` and `probability_of_loss`; typically
    /// [`ProbabilityError::CalculationError`] when a tail integration
    /// beyond the break-even points fails.
    fn calculate_extreme_probabilities(
        &self,
        volatility_adj: Option<VolatilityAdjustment>,
        trend: Option<PriceTrend>,
    ) -> Result<(Positive, Positive), ProbabilityError> {
        let volatility = resolve_volatility(self, volatility_adj)?;
        let profit_ranges = self.get_profit_ranges()?;
        let loss_ranges = self.get_loss_ranges()?;

        let max_profit_range = profit_ranges
            .iter()
            .find(|range| range.upper_bound.is_none());

        let max_loss_range = loss_ranges.iter().find(|range| range.lower_bound.is_none());
        let expiration = *self.get_expiration().values().next().ok_or_else(|| {
            StrategyError::empty_collection(
                "calculate_extreme_probabilities: no expiration on strategy",
            )
        })?;
        let risk_free_rate = *self.get_risk_free_rate()?.values().next().ok_or_else(|| {
            StrategyError::empty_collection(
                "calculate_extreme_probabilities: no risk_free_rate on strategy",
            )
        })?;
        let underlying_price = self.get_underlying_price()?;

        let mut max_profit_prob = Positive::ZERO;
        if let Some(range) = max_profit_range {
            let mut range_clone = range.clone();
            range_clone.calculate_probability(
                underlying_price,
                volatility,
                trend.clone(),
                expiration,
                Some(*risk_free_rate),
            )?;
            max_profit_prob = range_clone.probability;
        }

        let mut max_loss_prob = Positive::ZERO;
        if let Some(range) = max_loss_range {
            let mut range_clone = range.clone();
            range_clone.calculate_probability(
                underlying_price,
                volatility,
                trend,
                expiration,
                Some(*risk_free_rate),
            )?;
            max_loss_prob = range_clone.probability;
        }

        Ok((max_profit_prob, max_loss_prob))
    }

    /// Get the price ranges that would result in a profit
    ///
    /// # Returns
    /// - `Result<Vec<ProfitLossRange>, ProbabilityError>`: A vector of price ranges
    ///   that result in profit, or an error
    ///
    /// # Errors
    ///
    /// Returns [`ProbabilityError::RangeError`] when no break-even
    /// points can be derived from the strategy, or
    /// [`ProbabilityError::CalculationError`] when the profit
    /// evaluation fails at a sampled price.
    fn get_profit_ranges(&self) -> Result<Vec<ProfitLossRange>, ProbabilityError>;

    /// # Get Profit/Loss Ranges
    ///
    /// Returns a collection of price ranges with associated probabilities for profit and loss scenarios.
    ///
    /// This function analyzes the strategy to identify distinct price ranges where the strategy
    /// would result in either profit or loss at expiration. Each range includes probability
    /// information based on the statistical model for the underlying asset.
    ///
    /// ## Returns
    ///
    /// * `Result<Vec<ProfitLossRange>, ProbabilityError>` - On success, returns a vector of
    ///   profit/loss ranges sorted by their price boundaries. On failure, returns a
    ///   `ProbabilityError` indicating what went wrong during the analysis.
    ///
    /// # Errors
    ///
    /// Returns [`ProbabilityError::RangeError`] when no break-even
    /// points can be derived from the strategy, or
    /// [`ProbabilityError::CalculationError`] when the loss evaluation
    /// fails at a sampled price.
    fn get_loss_ranges(&self) -> Result<Vec<ProfitLossRange>, ProbabilityError>;
}

/// The volatility a [`ProbabilityAnalysis`] method prices at: the caller's
/// adjustment when given, otherwise the strategy's
/// [`ProbabilityAnalysis::reference_volatility`] with no widening.
fn resolve_volatility<S: ProbabilityAnalysis + ?Sized>(
    strategy: &S,
    volatility_adj: Option<VolatilityAdjustment>,
) -> Result<VolatilityAdjustment, ProbabilityError> {
    match volatility_adj {
        Some(adjustment) => Ok(adjustment),
        None => Ok(VolatilityAdjustment {
            base_volatility: strategy.reference_volatility()?,
            std_dev_adjustment: Positive::ZERO,
        }),
    }
}

#[cfg(test)]
mod tests_probability_analysis {
    use super::*;
    use crate::strategies::BullCallSpread;
    use optionstratlib_core::model::ExpirationDate;
    use rust_decimal_macros::dec;

    fn price_trend(
        drift_rate: rust_decimal::Decimal,
        confidence: rust_decimal::Decimal,
    ) -> PriceTrend {
        match PriceTrend::new(drift_rate, confidence) {
            Ok(trend) => trend,
            Err(e) => panic!("valid trend: {e}"),
        }
    }

    fn test_strategy() -> BullCallSpread {
        BullCallSpread::new(
            "GOLD".to_string(),
            pos_or_panic!(2505.8), // underlying_price
            pos_or_panic!(2460.0), // long_strike_itm
            pos_or_panic!(2515.0), // short_strike
            ExpirationDate::Days(pos_or_panic!(30.0)),
            pos_or_panic!(0.2),   // implied_volatility
            dec!(0.05),           // risk_free_rate
            Positive::ZERO,       // dividend_yield
            Positive::ONE,        // quantity
            pos_or_panic!(27.26), // premium_long
            pos_or_panic!(5.33),  // premium_short
            pos_or_panic!(0.58),  // open_fee_long
            pos_or_panic!(0.58),  // close_fee_long
            pos_or_panic!(0.55),  // close_fee_short
            pos_or_panic!(0.54),  // open_fee_short
        )
        .unwrap()
    }

    #[test]
    fn test_analyze_probabilities_without_adjustments() {
        let strategy = test_strategy();
        let result = strategy.analyze_probabilities(None, None);

        assert!(result.is_ok());
        let analysis = result.unwrap();
        assert!(analysis.probability_of_profit > Positive::ZERO);
        assert_eq!(analysis.probability_of_max_profit, Positive::ZERO);
        assert_eq!(analysis.probability_of_max_loss, Positive::ZERO);
        assert!(analysis.risk_reward_ratio > Positive::ZERO);
    }

    #[test]
    fn test_analyze_probabilities_with_adjustments() {
        let strategy = test_strategy();
        let vol_adj = Some(VolatilityAdjustment {
            base_volatility: pos_or_panic!(0.2),
            std_dev_adjustment: pos_or_panic!(0.05),
        });
        let trend = Some(price_trend(dec!(0.1), dec!(0.95)));

        let result = strategy.analyze_probabilities(vol_adj, trend);

        assert!(result.is_ok());
        let analysis = result.unwrap();
        assert!(analysis.probability_of_profit > Positive::ZERO);
        assert!(analysis.probability_of_max_profit >= Positive::ZERO);
        assert!(analysis.probability_of_max_loss >= Positive::ZERO);
    }

    #[test]
    fn test_expected_value_calculation() {
        let strategy = test_strategy();
        let result = strategy.expected_value(None, None);

        assert!(result.is_ok());
        assert!(result.unwrap() > Decimal::ZERO);
    }

    #[test]
    fn test_expected_value_with_trend() {
        let strategy = test_strategy();
        let trend = Some(price_trend(dec!(0.1), dec!(0.95)));

        let result = strategy.expected_value(None, trend);

        assert!(result.is_ok());
        assert!(result.unwrap() > Decimal::ZERO);
    }

    #[test]
    fn test_probability_of_profit() {
        let strategy = test_strategy();
        let result = strategy.probability_of_profit(None, None);

        assert!(result.is_ok());
        let prob = result.unwrap();
        assert!(prob > Positive::ZERO);
        assert!(prob <= Positive::ONE);
    }

    #[test]
    fn test_probability_of_loss() {
        let strategy = test_strategy();
        let result = strategy.probability_of_loss(None, None);

        assert!(result.is_ok());
        let prob = result.unwrap();
        assert!(prob > Positive::ZERO);
        assert!(prob <= Positive::ONE);
    }

    #[test]
    fn test_calculate_extreme_probabilities() {
        let strategy = test_strategy();
        let result = strategy.calculate_extreme_probabilities(None, None);

        assert!(result.is_ok());
        let (max_profit_prob, max_loss_prob) = result.unwrap();
        assert!(max_profit_prob >= Positive::ZERO);
        assert!(max_loss_prob >= Positive::ZERO);
        assert!(max_profit_prob + max_loss_prob <= Positive::ONE);
    }

    #[test]
    fn test_extreme_probabilities_with_adjustments() {
        let strategy = test_strategy();
        let vol_adj = Some(VolatilityAdjustment {
            base_volatility: pos_or_panic!(0.2),
            std_dev_adjustment: pos_or_panic!(0.05),
        });
        let trend = Some(price_trend(dec!(0.1), dec!(0.95)));

        let result = strategy.calculate_extreme_probabilities(vol_adj, trend);

        assert!(result.is_ok());
        let (max_profit_prob, max_loss_prob) = result.unwrap();
        assert!(max_profit_prob >= Positive::ZERO);
        assert!(max_loss_prob >= Positive::ZERO);
        assert!(max_profit_prob + max_loss_prob <= Positive::ONE);
    }

    #[test]
    fn test_expected_value_with_volatility() {
        let strategy = test_strategy();
        let vol_adj = Some(VolatilityAdjustment {
            base_volatility: pos_or_panic!(0.3),
            std_dev_adjustment: pos_or_panic!(0.05),
        });

        let result = strategy.expected_value(vol_adj, None);
        assert!(result.is_ok());
    }
}

#[cfg(test)]
mod tests_expected_value {
    use super::*;
    use crate::strategies::BullCallSpread;
    use optionstratlib_core::model::ExpirationDate;
    use rust_decimal_macros::dec;

    fn price_trend(
        drift_rate: rust_decimal::Decimal,
        confidence: rust_decimal::Decimal,
    ) -> PriceTrend {
        match PriceTrend::new(drift_rate, confidence) {
            Ok(trend) => trend,
            Err(e) => panic!("valid trend: {e}"),
        }
    }

    // Helper function to create a test strategy
    fn create_test_strategy() -> BullCallSpread {
        BullCallSpread::new(
            "GOLD".to_string(),
            pos_or_panic!(2505.8), // underlying_price
            pos_or_panic!(2460.0), // long_strike_itm
            pos_or_panic!(2515.0), // short_strike
            ExpirationDate::Days(pos_or_panic!(30.0)),
            pos_or_panic!(0.2),   // implied_volatility
            dec!(0.05),           // risk_free_rate
            Positive::ZERO,       // dividend_yield
            Positive::ONE,        // quantity
            pos_or_panic!(27.26), // premium_long
            pos_or_panic!(5.33),  // premium_short
            pos_or_panic!(0.58),  // open_fee_long
            pos_or_panic!(0.58),  // close_fee_long
            pos_or_panic!(0.55),  // close_fee_short
            pos_or_panic!(0.54),  // open_fee_short
        )
        .unwrap()
    }

    /// A drift with 15 decimal places, where `Decimal::to_f64` lands away
    /// from the nearest `f64` (on `2.999789999999903`) and would move the
    /// last digits (to `0.003104587227155627`). Pinned from the `f64`-field
    /// code fed `drift_rate: 2.999789999999902, confidence: 0.95` (#656), then
    /// re-baselined when the probability threshold gained the lognormal
    /// `-sigma^2 / 2` term (#664: `0.002802439458791824` ->
    /// `0.003104587227155649`). Re-baselined again by #623, which sums in
    /// `Decimal` and drops the extra `1 / (1 + |drift|)` scaling applied
    /// after the sum: the drift already shapes the distribution, so the old
    /// value was the expectation divided by `3.999789999999902`
    /// (`0.003104587227155649` -> `0.012417696945304589008056`).
    #[test]
    fn test_expected_value_many_decimal_places_drift_matches_f64_field() {
        let strategy = create_test_strategy();
        let trend = Some(price_trend(dec!(2.999789999999902), dec!(0.95)));
        match strategy.expected_value(None, trend) {
            Ok(ev) => assert_eq!(ev, dec!(0.012417696945304589008056)),
            Err(e) => panic!("expected value evaluates: {e}"),
        }
    }

    #[test]
    fn test_expected_value_basic() {
        let strategy = create_test_strategy();
        let result = strategy.expected_value(None, None);

        assert!(result.is_ok(), "Expected value calculation should succeed");
        let ev = result.unwrap();
        assert!(ev > Decimal::ZERO, "expected value {ev}");
    }

    #[test]
    fn test_expected_value_with_volatility() {
        let strategy = create_test_strategy();
        let vol_adj = Some(VolatilityAdjustment {
            base_volatility: pos_or_panic!(0.25),
            std_dev_adjustment: pos_or_panic!(0.1),
        });

        let result = strategy.expected_value(vol_adj, None);
        assert!(result.is_ok());
        // Floored to zero before #623: the spread loses money on average
        // here, and the signed expected value reports it.
        let ev = result.unwrap();
        assert!(ev < Decimal::ZERO, "expected value {ev}");
    }

    #[test]
    fn test_expected_value_with_trend() {
        let strategy = create_test_strategy();
        let trend = Some(price_trend(dec!(0.1), dec!(0.95)));

        let result = strategy.expected_value(None, trend);
        assert!(result.is_ok());
        assert!(result.unwrap() > Decimal::ZERO);
    }

    /// A trend held with zero confidence is ignored by the distribution, so
    /// it leaves the expected value unchanged. Before #623 the sum was also
    /// divided by `1 + |drift|` whatever the confidence, which made this
    /// expected value a fifth of the untrended one.
    #[test]
    fn test_expected_value_zero_confidence_trend_matches_no_trend() {
        let strategy = create_test_strategy();
        let trend = Some(price_trend(dec!(4.0), Decimal::ZERO));
        let with_trend = match strategy.expected_value(None, trend) {
            Ok(ev) => ev,
            Err(e) => panic!("expected value with trend: {e}"),
        };
        let without = match strategy.expected_value(None, None) {
            Ok(ev) => ev,
            Err(e) => panic!("expected value without trend: {e}"),
        };
        assert_eq!(with_trend, without);
    }

    /// With zero volatility the expected value is the profit at the current
    /// price, of either sign. With the spot below both strikes the debit
    /// spread is worthless, so the answer is minus the net debit, not zero.
    #[test]
    fn test_expected_value_zero_volatility_reports_current_loss() {
        let strategy = match BullCallSpread::new(
            "GOLD".to_string(),
            pos_or_panic!(2400.0), // underlying_price, below both strikes
            pos_or_panic!(2460.0),
            pos_or_panic!(2515.0),
            ExpirationDate::Days(pos_or_panic!(30.0)),
            pos_or_panic!(0.2),
            dec!(0.05),
            Positive::ZERO,
            Positive::ONE,
            pos_or_panic!(27.26),
            pos_or_panic!(5.33),
            pos_or_panic!(0.58),
            pos_or_panic!(0.58),
            pos_or_panic!(0.55),
            pos_or_panic!(0.54),
        ) {
            Ok(strategy) => strategy,
            Err(e) => panic!("spread constructs: {e}"),
        };
        let vol_adj = Some(VolatilityAdjustment {
            base_volatility: Positive::ZERO,
            std_dev_adjustment: Positive::ZERO,
        });
        let current = match strategy.calculate_profit_at(&pos_or_panic!(2400.0)) {
            Ok(profit) => profit,
            Err(e) => panic!("profit at the spot: {e}"),
        };
        assert!(current < Decimal::ZERO, "profit at the spot {current}");
        match strategy.expected_value(vol_adj, None) {
            Ok(ev) => assert_eq!(ev, current),
            Err(e) => panic!("expected value at zero volatility: {e}"),
        }
    }

    #[test]
    fn test_expected_value_with_both_adjustments() {
        let strategy = create_test_strategy();
        let vol_adj = Some(VolatilityAdjustment {
            base_volatility: pos_or_panic!(0.25),
            std_dev_adjustment: pos_or_panic!(0.1),
        });
        let trend = Some(price_trend(dec!(0.1), dec!(0.95)));

        let result = strategy.expected_value(vol_adj, trend);
        assert!(result.is_ok());
        // Floored to zero before #623: the spread loses money on average
        // here, and the signed expected value reports it.
        let ev = result.unwrap();
        assert!(ev < Decimal::ZERO, "expected value {ev}");
    }

    #[test]
    fn test_expected_value_with_high_volatility() {
        let strategy = create_test_strategy();
        let vol_adj = Some(VolatilityAdjustment {
            base_volatility: Positive::ONE,
            std_dev_adjustment: pos_or_panic!(0.5),
        });

        let result = strategy.expected_value(vol_adj, None);
        assert!(result.is_ok());
        // Floored to zero before #623: the spread loses money on average
        // here, and the signed expected value reports it.
        let ev = result.unwrap();
        assert!(ev < Decimal::ZERO, "expected value {ev}");
    }

    #[test]
    fn test_expected_value_with_negative_trend() {
        let strategy = create_test_strategy();
        let trend = Some(price_trend(dec!(-0.2), dec!(0.90)));

        let result = strategy.expected_value(None, trend);
        assert!(result.is_ok());
        // Floored to zero before #623: the spread loses money on average
        // here, and the signed expected value reports it.
        let ev = result.unwrap();
        assert!(ev < Decimal::ZERO, "expected value {ev}");
    }

    #[test]
    fn test_expected_value_probabilities_sum() {
        let strategy = create_test_strategy();
        let result = strategy.expected_value(None, None);
        assert!(result.is_ok());

        // Test passes implicitly if no warning is logged about probability sum deviation
        // The actual check is done inside the method using warn!
    }

    #[test]
    fn test_expected_value_with_minimal_volatility() {
        let strategy = create_test_strategy();
        // Use a very small but positive volatility value
        let vol_adj = Some(VolatilityAdjustment {
            base_volatility: pos_or_panic!(0.0001), // Very small but non-zero volatility
            std_dev_adjustment: Positive::ZERO,
        });

        let result = strategy.expected_value(vol_adj, None);
        assert!(
            result.is_ok(),
            "Expected value calculation should succeed with minimal volatility"
        );
        let ev = result.unwrap();
        assert!(ev > Decimal::ZERO, "expected value {ev}");
    }
}

#[cfg(test)]
mod tests_marginal_probability_inversion {
    use super::*;
    use crate::strategies::BullCallSpread;
    use crate::strategies::base::{BasicAble, Strategies};
    use optionstratlib_core::model::ExpirationDate;
    use rust_decimal_macros::dec;

    fn spread(spot: f64, volatility: f64) -> Result<BullCallSpread, StrategyError> {
        BullCallSpread::new(
            "PROBE".to_string(),
            Positive::new(spot).expect("the probe spot is positive and finite"),
            Positive::new(spot).expect("the probe spot is positive and finite"),
            Positive::new(spot * 1.000_000_1).expect("the probe strike is positive and finite"),
            ExpirationDate::Days(pos_or_panic!(30.0)),
            Positive::new(volatility).expect("the probe volatility is positive and finite"),
            dec!(0.05),
            Positive::ZERO,
            Positive::ONE,
            pos_or_panic!(27.26),
            pos_or_panic!(5.33),
            pos_or_panic!(0.58),
            pos_or_panic!(0.58),
            pos_or_panic!(0.55),
            pos_or_panic!(0.54),
        )
    }

    /// `expected_value` weights each profit by the marginal mass between two
    /// consecutive prices, and #570 replaced the floor on that subtraction
    /// with the same report `ProfitLossRange::calculate_probability` uses.
    ///
    /// The inversion itself is not reachable here through the public API, and
    /// two independent limits stop it before the subtraction:
    ///
    /// * **The display range.** The grid steps by `spot / 100`, so a ratio
    ///   close enough to one to invert `checked_ln` needs a spot near
    ///   `Positive::MAX`, and `get_best_range_to_show` scales the highest
    ///   point by `STRIKE_PRICE_UPPER_BOUND_MULTIPLIER` (1.02) before a single
    ///   probability is computed. Measured at `7.9e28`, `get_range_to_show`
    ///   reports `mul_f64: overflow` while `calculate_profit_at` on the same
    ///   strategy still returns `Ok(-24.18)`, so the range, not the profit, is
    ///   what stops it.
    /// * **The volatility.** The inversion needs a volatility around `1e-28`,
    ///   and at that value the z-score leaves the finite range: the kernel
    ///   reports a conversion failure at every spot from `1e3` to `1e28`
    ///   rather than producing two CDF values to subtract. `1e-20` and above
    ///   succeed everywhere in that span.
    ///
    /// The report is therefore a guard, and these tests pin the other half of
    /// the contract, that it does not misfire on the extreme inputs that *are*
    /// reachable. If either limit is lifted so the inversion becomes
    /// reachable, it errors rather than silently under-weighting a step.
    fn adjustment(volatility: f64) -> Option<VolatilityAdjustment> {
        Some(VolatilityAdjustment {
            base_volatility: Positive::new(volatility)
                .expect("the probe volatility is positive and finite"),
            std_dev_adjustment: Positive::ZERO,
        })
    }

    /// The volatility is passed explicitly so each test states the value it
    /// probes, independent of which leg `reference_volatility` would pick.
    ///
    /// At the smallest volatility the model can hold, the guard is never
    /// reached: the z-score leaves the finite range and the kernel reports a
    /// conversion failure instead of producing two CDF values to subtract.
    /// That is a typed error, not a silent wrong answer, which is the property
    /// worth pinning.
    #[test]
    fn test_degenerate_volatility_is_reported_before_the_subtraction() {
        for spot in [1e3f64, 1e15, 1e28] {
            let strategy = match spread(spot, 1e-28) {
                Ok(strategy) => strategy,
                Err(error) => panic!("the probe spread must construct at {spot:e}: {error:?}"),
            };
            let result = strategy.expected_value(adjustment(1e-28), None);
            assert!(
                result.is_err(),
                "a non-finite z-score must be reported at {spot:e}, got {result:?}"
            );
        }
    }

    /// One order of magnitude up from that limit the kernel is well defined
    /// again, and the guard must not fire: these are the most extreme inputs
    /// that actually reach the subtraction, and every step's mass is
    /// non-negative on all of them.
    #[test]
    fn test_extreme_but_workable_volatility_does_not_trip_the_guard() {
        for spot in [1e3f64, 1e6, 1e15, 1e20, 1e28] {
            let strategy = match spread(spot, 1e-20) {
                Ok(strategy) => strategy,
                Err(error) => panic!("the probe spread must construct at {spot:e}: {error:?}"),
            };
            let result = strategy.expected_value(adjustment(1e-20), None);
            assert!(
                result.is_ok(),
                "a monotone grid must not report an inversion at {spot:e}, got {result:?}"
            );
        }
    }

    /// Pins the reachability claim above rather than leaving it a comment: at
    /// `7.9e28` the display range is what fails, not the profit evaluation, so
    /// `expected_value` never reaches the subtraction the guard protects. If a
    /// later change lifts the range limit, this test starts failing and the
    /// guard's comment has to be revisited with it.
    #[test]
    fn test_the_display_range_is_what_stops_an_extreme_spot() {
        let strategy = match spread(7.9e28, 1e-28) {
            Ok(strategy) => strategy,
            Err(error) => panic!("the probe spread must construct at 7.9e28: {error:?}"),
        };

        assert!(
            strategy.get_range_to_show().is_err(),
            "the 1.02 upper-bound scaling must overflow at 7.9e28"
        );
        assert!(
            strategy
                .calculate_profit_at(strategy.get_underlying_price().unwrap())
                .is_ok(),
            "the profit evaluation still succeeds, so the range is the limit"
        );
        assert!(
            strategy.expected_value(adjustment(1e-20), None).is_err(),
            "expected_value fails on the range, before any probability"
        );
    }

    /// The ordinary path keeps working, including with a volatility
    /// adjustment, which is what exercises the two independent round trips
    /// through `big_n`.
    #[test]
    fn test_ordinary_spread_expected_value_is_unaffected() {
        let strategy = match spread(2505.8, 0.2) {
            Ok(strategy) => strategy,
            Err(error) => panic!("the ordinary spread must construct: {error:?}"),
        };
        let adjustment = Some(VolatilityAdjustment {
            base_volatility: pos_or_panic!(0.25),
            std_dev_adjustment: pos_or_panic!(0.1),
        });

        let result = strategy.expected_value(adjustment, None);
        assert!(
            result.is_ok(),
            "an ordinary spread must not report an inversion, got {result:?}"
        );
    }
}

#[cfg(test)]
mod tests_reference_volatility {
    use super::*;
    use crate::strategies::BullCallSpread;
    use optionstratlib_core::model::ExpirationDate;
    use optionstratlib_core::pos_or_panic;
    use rust_decimal_macros::dec;

    /// A bull call spread with each leg at its own implied volatility.
    fn spread(
        spot: Positive,
        long_strike: Positive,
        short_strike: Positive,
        long_iv: Positive,
        short_iv: Positive,
    ) -> BullCallSpread {
        let mut strategy = match BullCallSpread::new(
            "REFVOL".to_string(),
            spot,
            long_strike,
            short_strike,
            ExpirationDate::Days(pos_or_panic!(30.0)),
            long_iv,
            dec!(0.05),
            Positive::ZERO,
            Positive::ONE,
            pos_or_panic!(27.26),
            pos_or_panic!(5.33),
            pos_or_panic!(0.58),
            pos_or_panic!(0.58),
            pos_or_panic!(0.55),
            pos_or_panic!(0.54),
        ) {
            Ok(strategy) => strategy,
            Err(error) => panic!("the probe spread must construct: {error}"),
        };
        strategy.short_call.option.implied_volatility = short_iv;
        strategy
    }

    fn flat(volatility: Positive) -> Option<VolatilityAdjustment> {
        Some(VolatilityAdjustment {
            base_volatility: volatility,
            std_dev_adjustment: Positive::ZERO,
        })
    }

    /// The issue's own test: with no adjustment the strategy prices at its own
    /// implied volatility, not at a value the kernel picked. Before #619 the
    /// first assertion failed, because `None` meant a flat 0.2.
    #[test]
    fn test_probability_of_profit_none_uses_strategy_volatility() {
        let iv = pos_or_panic!(0.8);
        let strategy = spread(
            pos_or_panic!(2505.8),
            pos_or_panic!(2460.0),
            pos_or_panic!(2515.0),
            iv,
            iv,
        );

        let by_default = strategy.probability_of_profit(None, None);
        let at_own_iv = strategy.probability_of_profit(flat(iv), None);
        let at_0_2 = strategy.probability_of_profit(flat(pos_or_panic!(0.2)), None);

        match (by_default, at_own_iv, at_0_2) {
            (Ok(by_default), Ok(at_own_iv), Ok(at_0_2)) => {
                assert_eq!(by_default, at_own_iv);
                assert_ne!(by_default, at_0_2, "None must not price at 0.2 any more");
            }
            other => panic!("all three must succeed, got {other:?}"),
        }
    }

    /// With skew, the at-the-money leg's volatility is the reference: spot
    /// 2505.8 sits 9.2 below the 2515 strike and 45.8 above the 2460 one.
    #[test]
    fn test_reference_volatility_picks_leg_closest_to_spot() {
        let strategy = spread(
            pos_or_panic!(2505.8),
            pos_or_panic!(2460.0),
            pos_or_panic!(2515.0),
            pos_or_panic!(0.30),
            pos_or_panic!(0.24),
        );
        match strategy.reference_volatility() {
            Ok(volatility) => assert_eq!(volatility, pos_or_panic!(0.24)),
            Err(error) => panic!("reference volatility: {error}"),
        }
    }

    /// Two legs equally far from spot: the lower strike wins, so the answer
    /// does not depend on the order the legs were built in.
    #[test]
    fn test_reference_volatility_tie_takes_lower_strike() {
        let strategy = spread(
            pos_or_panic!(105.0),
            pos_or_panic!(100.0),
            pos_or_panic!(110.0),
            pos_or_panic!(0.31),
            pos_or_panic!(0.27),
        );
        match strategy.reference_volatility() {
            Ok(volatility) => assert_eq!(volatility, pos_or_panic!(0.31)),
            Err(error) => panic!("reference volatility: {error}"),
        }
    }

    /// An explicit adjustment always wins over the strategy's own volatility.
    #[test]
    fn test_explicit_adjustment_overrides_reference_volatility() {
        let strategy = spread(
            pos_or_panic!(2505.8),
            pos_or_panic!(2460.0),
            pos_or_panic!(2515.0),
            pos_or_panic!(0.8),
            pos_or_panic!(0.8),
        );
        let explicit = strategy.probability_of_profit(flat(pos_or_panic!(0.2)), None);
        let by_default = strategy.probability_of_profit(None, None);
        match (explicit, by_default) {
            (Ok(explicit), Ok(by_default)) => assert_ne!(explicit, by_default),
            other => panic!("both must succeed, got {other:?}"),
        }
    }
}
