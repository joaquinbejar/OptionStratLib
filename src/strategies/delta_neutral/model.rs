/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 10/12/24
******************************************************************************/
use crate::error::position::PositionValidationErrorKind;
use crate::error::{GreeksError, PositionError, StrategyError};
use crate::prelude::OperationErrorKind;

use super::adjustment::{AdjustmentConfig, AdjustmentPlan};
use super::optimizer::AdjustmentOptimizer;
use super::portfolio::{AdjustmentTarget, PortfolioGreeks};
/// # Delta Neutrality Management Module
///
/// This module provides tools and structures to manage and maintain delta neutrality
/// in trading strategies. It includes enumerations, structures, and a trait to calculate
/// net delta, check neutrality status, and suggest adjustments to achieve or maintain
/// delta neutrality.
///
/// ## Overview
/// - **`DeltaAdjustment`**: Enum defining the types of actions that can be taken to adjust delta.
/// - **`DeltaInfo`**: Structure containing detailed information about the delta status of a strategy.
/// - **`DeltaNeutrality`**: Trait implementing methods for managing delta neutrality, such as
///   calculating net delta, checking neutrality, and generating suggestions for adjustments.
///
/// ## Components
/// 1. **DeltaAdjustment Enum**
///    - Represents possible adjustments needed to achieve delta neutrality.
///    - Covers buying/selling options, underlying assets, or no adjustment if strategy is neutral.
///
/// 2. **DeltaInfo Structure**
///    - Provides detailed information about the delta of a trading strategy.
///    - Includes the net delta, individual deltas, neutrality status, and more.
///
/// 3. **DeltaNeutrality Trait**
///    - A trait for trading strategies that provides the ability to calculate net delta,
///      determine neutrality, and suggest actions for neutrality adjustments.
///
/// ## Code Highlights
/// - **`DELTA_THRESHOLD`** defines the maximum allowed deviation from neutrality.
/// - The module introduces two levels of adjustment:
///   - `generate_delta_reducing_adjustments`: Suggests adjustments to reduce a positive delta.
///
/// ## Usage
/// This module is designed to help maintain a delta-neutral portfolio by suggesting
/// appropriate hedging actions (e.g., buying or selling options or underlying assets)
/// based on the delta exposure of the strategy.
use crate::greeks::Greeks;
use crate::greeks::calculate_delta_neutral_sizes;
use crate::model::types::{Action, OptionStyle};
use crate::model::{Trade, TradeStatusAble};
use crate::strategies::Strategies;
use crate::strategies::base::Positionable;
use crate::{Options, Positive, Side};
use pretty_simple_display::{DebugPretty, DisplaySimple};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use std::fmt;
use tracing::{debug, warn};
use utoipa::ToSchema;

/// # Delta Neutrality Threshold
///
/// The default threshold value used to determine if an options strategy is considered delta neutral.
///
/// When evaluating delta neutrality, a strategy's net delta is compared against this threshold value.
/// If the absolute value of the net delta is less than or equal to this threshold, the strategy
/// is considered delta neutral.
///
/// ## Value Significance
/// The small value (0.0001) represents a very tight threshold, meaning the strategy must have
/// extremely minimal directional exposure to be considered neutral. This conservative threshold
/// helps ensure strategies maintain strict delta neutrality for effective risk management.
///
/// ## Usage Context
/// This constant is primarily used within delta neutrality calculations and serves as a default
/// when a custom threshold is not specified. Functions that analyze or adjust strategies for
/// delta neutrality may use this value when determining if additional position adjustments
/// are necessary.
///
/// ## Related Components
/// Works in conjunction with the `DeltaInfo` and `DeltaNeutralResponse` structures to provide
/// consistent evaluation of delta neutrality across the delta neutral strategies module.
pub const DELTA_THRESHOLD: Decimal = dec!(0.0001);

/// The `DeltaAdjustment` enum is used to define how a trading strategy can
/// be modified to achieve or maintain a delta-neutral state. Delta neutrality
/// refers to a situation where the combined delta of all positions is close
/// to zero, minimizing directional market risk.
///
/// Variants:
/// - `BuyOptions`: Represents buying option contracts with specific parameters.
/// - `SellOptions`: Represents selling option contracts with specific parameters.
/// - `BuyUnderlying`: Represents buying units of the underlying asset.
/// - `SellUnderlying`: Represents selling units of the underlying asset.
/// - `NoAdjustmentNeeded`: Indicates that the strategy is already delta neutral
///   within a specified threshold.
/// - `SameSize`: Represents combining two `DeltaAdjustment` actions of to keep the same
///   size for balancing or maintaining neutrality.
///
/// Each variant provides detailed parameters needed for the associated adjustment,
/// such as the quantity of options or underlying asset and other relevant details.
#[derive(Debug, PartialEq, Serialize, Deserialize, ToSchema)]
pub enum DeltaAdjustment {
    /// Represents buying a number of option contracts to adjust delta.
    ///
    /// Fields:
    /// - `quantity`: The number of contracts to buy. This value is represented
    ///   using the `Positive` type, ensuring it is non-negative.
    /// - `strike`: The strike price of the options being purchased.
    /// - `option_type`: Defines the type of option being purchased, either
    ///   `Call` or `Put`, indicated by the `OptionStyle` enum.
    BuyOptions {
        /// Number of contracts to buy.
        quantity: Positive,
        /// Strike price of the options.
        strike: Positive,
        /// Type of the option (Call or Put).
        option_style: OptionStyle,
        /// Side of the option (Long or Short).
        side: Side,
    },

    /// Represents selling a number of option contracts to adjust delta.
    ///
    /// Fields:
    /// - `quantity`: The number of contracts to sell. This value is represented
    ///   using the `Positive` type, ensuring it is non-negative.
    /// - `strike`: The strike price of the options being sold.
    /// - `option_type`: Defines the type of option being sold, either `Call`
    ///   or `Put`, indicated by the `OptionStyle` enum.
    SellOptions {
        /// Number of contracts to sell.
        quantity: Positive,
        /// Strike price of the options.
        strike: Positive,
        /// Type of the option (Call or Put).
        option_style: OptionStyle,
        /// side of the option (Long or Short).
        side: Side,
    },

    /// Represents buying a quantity of the underlying asset to adjust delta.
    ///
    /// This variant is used when buying shares, units, or equivalent of the
    /// underlying asset is necessary to adjust the delta of a strategy.
    ///
    /// - The `Positive` argument specifies the quantity being purchased.
    BuyUnderlying(Positive),

    /// Represents selling a quantity of the underlying asset to adjust delta.
    ///
    /// This variant is used when selling shares, units, or equivalent of the
    /// underlying asset is necessary to adjust the delta of a strategy.
    ///
    /// - The `Positive` argument specifies the quantity being sold.
    SellUnderlying(Positive),

    /// Indicates that no adjustment is needed to achieve delta neutrality.
    ///
    /// This is used when the strategy's current delta falls within the
    /// acceptable neutrality threshold, and no action is required.
    NoAdjustmentNeeded,

    /// Combines two adjustments of the same size to maintain or balance
    /// delta neutrality.
    ///
    /// Fields:
    /// - `DeltaAdjustment, DeltaAdjustment`: The two adjustments to keep the same
    ///   size being combined.
    SameSize(DeltaAdjustmentSameSize),
}

#[derive(DebugPretty, DisplaySimple, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct DeltaAdjustmentSameSize {
    pub first: Box<DeltaAdjustment>,
    pub second: Box<DeltaAdjustment>,
}

impl fmt::Display for DeltaAdjustment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeltaAdjustment::BuyOptions {
                quantity,
                strike,
                option_style,
                side,
            } => {
                write!(
                    f,
                    "Buy {quantity} {side} {option_style} options at strike {strike}"
                )
            }
            DeltaAdjustment::SellOptions {
                quantity,
                strike,
                option_style,
                side,
            } => {
                write!(
                    f,
                    "Sell {quantity} {side} {option_style} options at strike {strike}"
                )
            }
            DeltaAdjustment::BuyUnderlying(quantity) => {
                write!(f, "Buy {quantity} units of the underlying asset")
            }
            DeltaAdjustment::SellUnderlying(quantity) => {
                write!(f, "Sell {quantity} units of the underlying asset")
            }
            DeltaAdjustment::NoAdjustmentNeeded => {
                write!(f, "No adjustment needed")
            }
            DeltaAdjustment::SameSize(adj) => {
                write!(
                    f,
                    "Same size adjustments: [{}] and [{}]",
                    adj.first, adj.second
                )
            }
        }
    }
}

/// Represents the delta and associated details for a single position in an options strategy.
///
/// This structure contains the key information about an individual option position's delta
/// characteristics, including its quantity, strike price, and whether it's a call/put and
/// long/short. It serves as a fundamental building block for analyzing and managing the
/// delta exposure of complex options strategies.
///
/// ## Fields
/// - `delta`: The delta value of the position, representing the sensitivity of the position's price
///   to changes in the underlying asset price.
/// - `quantity`: The quantity of the options in the position.
/// - `strike`: The strike price of the option, represented as a positive value.
/// - `option_style`: Indicates whether the option is a call or a put.
/// - `side`: Indicates whether the position is long or short.
///
/// ## Usage
///
/// `DeltaPositionInfo` is typically used within a collection to represent all positions in a
/// strategy when calculating net delta exposure or determining adjustments needed for
/// delta neutrality.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema)]
pub struct DeltaPositionInfo {
    /// The delta value of the position, representing the sensitivity of the position's price
    /// to changes in the underlying asset price.
    pub delta: Decimal,
    /// Represents the change in value (delta) for a single unit of a financial contract.
    ///
    /// The `delta_per_contract` field indicates the sensitivity of the contract's price to
    /// changes in the price of the underlying asset. It is expressed as a `Decimal` and is a
    /// crucial measure in derivatives trading and risk management.
    pub delta_per_contract: Decimal,
    /// The quantity of the options in the position.
    pub quantity: Positive,
    /// The strike price of the option, represented as a positive value.
    pub strike: Positive,
    /// Indicates whether the option is a call or a put.
    pub option_style: OptionStyle,
    /// Indicates whether the position is long or short.
    pub side: Side,
}

impl fmt::Display for DeltaPositionInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Delta: {:.4}", self.delta)?;
        writeln!(f, "  Delta per Contract: {:.4}", self.delta_per_contract)?;
        writeln!(f, "  Quantity: {:.4}", self.quantity)?;
        writeln!(f, "  Strike: {}", self.strike)?;
        writeln!(f, "  Option Style: {:?}", self.option_style)?;
        writeln!(f, "  Side: {:?}", self.side)?;
        Ok(())
    }
}

/// Contains detailed information about an options strategy's delta status.
///
/// This structure provides both an analysis of the cumulative delta of the strategy
/// and details about its individual positions. It can be used to evaluate whether
/// a strategy is delta neutral and to inform adjustments.
///
/// ## Purpose
/// DeltaInfo serves as a central structure to analyze and manage the delta status
/// of multi-position strategies, such as those used in options trading. It is particularly
/// useful for implementing delta-neutral strategy adjustments.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DeltaInfo {
    /// The net delta of the strategy, representing the overall sensitivity of the
    /// strategy to changes in the underlying asset price.
    pub net_delta: Decimal,
    /// A vector of `DeltaPositionInfo` structures containing
    ///   the delta information for each position in the strategy.
    pub individual_deltas: Vec<DeltaPositionInfo>,
    /// Indicates whether the strategy is considered delta neutral based
    ///   on the configured neutrality threshold.
    pub is_neutral: bool,
    /// The threshold used to determine if the strategy is delta neutral.
    ///   If the net delta is within this range, the strategy is considered neutral.
    pub neutrality_threshold: Decimal,
    /// The current price of the underlying asset, represented as a positive value.
    pub underlying_price: Positive,
}

impl fmt::Display for DeltaInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Delta Analysis:")?;
        writeln!(f, "  Net Delta: {:.4}", self.net_delta)?;
        writeln!(f, "  Is Neutral: {}", self.is_neutral)?;
        writeln!(
            f,
            "  Neutrality Threshold: {:.4}",
            self.neutrality_threshold
        )?;
        writeln!(f, "  Underlying Price: {}", self.underlying_price)?;
        writeln!(f, "  Individual Deltas:")?;
        for (i, delta) in self.individual_deltas.iter().enumerate() {
            writeln!(f, "    Position {}: {:.4}", i + 1, delta)?;
        }
        Ok(())
    }
}

/// A trait that provides functionality for managing and maintaining delta neutrality in trading strategies.
///
/// This trait extends the `Greeks` trait and introduces methods to calculate net delta,
/// check neutrality status, suggest adjustments, and generate delta adjustments for a trading strategy.
/// It implements key concepts needed to manage the delta exposure efficiently.
///
/// # Methods
///
/// * `calculate_net_delta`: Calculates the net delta of the trading strategy and provides detailed delta-related information.
/// * `is_delta_neutral`: Determines if the strategy is delta-neutral within a specified threshold.
/// * `suggest_delta_adjustments`: Suggests potential actions to achieve delta neutrality.
/// * `generate_delta_reducing_adjustments`: Produces adjustments required to reduce a positive delta.
/// * `get_atm_strike`: Retrieves the ATM (At-The-Money) strike price closest to the current underlying asset price.
pub trait DeltaNeutrality: Greeks + Positionable + Strategies {
    /// Calculates the net delta of the strategy and provides detailed information.
    ///
    /// # Returns
    /// A `DeltaInfo` struct containing:
    /// * The net delta of the strategy.
    /// * Individual deltas of all components in the strategy.
    /// * Whether the strategy is considered delta neutral.
    /// * Threshold for neutrality determination.
    /// * The current price of the underlying asset.
    ///
    /// This provides an overview of the delta position and helps in determining adjustments.
    fn delta_neutrality(&self) -> Result<DeltaInfo, GreeksError> {
        let options = self.get_options()?;
        if options.is_empty() {
            return Err(GreeksError::StdError("No options found".to_string()));
        }
        let underlying_price = *self.get_underlying_price();
        let individual_deltas: Vec<DeltaPositionInfo> = options
            .iter()
            .map(|option| DeltaPositionInfo {
                delta: option.delta().unwrap(),
                delta_per_contract: option.delta().unwrap() / option.quantity,
                quantity: option.quantity,
                strike: option.strike_price,
                option_style: option.option_style,
                side: option.side,
            })
            .collect();

        Ok(DeltaInfo {
            net_delta: self.delta()?,
            individual_deltas,
            is_neutral: self.is_delta_neutral(),
            underlying_price,
            neutrality_threshold: DELTA_THRESHOLD,
        })
    }

    /// Checks if the strategy is delta neutral within the specified threshold.
    ///
    /// # Arguments
    /// * `threshold` - A `Decimal` value representing the maximum allowed deviation from ideal delta neutrality.
    ///
    /// # Returns
    /// A boolean (`true` or `false`):
    /// * `true` if the absolute value of the net delta is within the threshold.
    /// * `false` otherwise.
    fn is_delta_neutral(&self) -> bool {
        match self.delta() {
            Ok(delta) => delta.abs() <= DELTA_THRESHOLD,
            Err(_) => false,
        }
    }

    /// # get_atm_strike
    ///
    /// Returns the at-the-money (ATM) strike price for a strategy by obtaining the underlying asset's price.
    ///
    /// An at-the-money strike is a strike price that is equal (or very close) to the current market price
    /// of the underlying asset. This is often used as a reference point for constructing option strategies.
    ///
    /// ## Returns
    ///
    /// * `Result<Positive, StrategyError>` - The underlying price as a `Positive` value wrapped in a `Result`.
    ///   Returns an error if retrieving the underlying price fails.
    ///
    /// ## Errors
    ///
    /// This function may return a `StrategyError` if the call to `get_underlying_price()` fails.
    ///
    /// ## Notes
    ///
    /// This implementation assumes that the ATM strike is exactly equal to the current price of the
    /// underlying asset. In practice, the actual ATM strike might be the nearest available strike price
    /// offered by the exchange.
    ///
    fn get_atm_strike(&self) -> Result<Positive, StrategyError> {
        Ok(*self.get_underlying_price())
    }

    /// Generates delta adjustments based on the given net delta and option delta per contract.
    ///
    /// This function calculates the necessary adjustments (buying or selling options) to bring the
    /// net delta of a position closer to neutral based on the delta of the options and current positions.
    ///
    /// # Parameters
    /// - `net_delta`: The net delta of the current portfolio or position. A positive value indicates
    ///   excess positive delta, while a negative value indicates excess negative delta.
    /// - `option_delta_per_contract`: The delta value of an individual option contract. A positive
    ///   value represents a positive delta option (e.g., calls for long positions), while a negative
    ///   value represents a negative delta option (e.g., puts for long positions).
    /// - `option`: A reference to an instance of the `Options` struct, representing the specific option
    ///   for which adjustments are to be made. This includes attributes such as the option strike price,
    ///   style, side, and current quantity of contracts held.
    ///
    /// # Returns
    /// - `Ok(DeltaAdjustment)`: An adjustment object indicating the number of contracts to buy or sell,
    ///   along with relevant option details (e.g., strike price, style, side). If no adjustment is
    ///   needed, the function returns `DeltaAdjustment::NoAdjustmentNeeded`.
    /// - `Err(GreeksError)`: Returns an error if an adjustment cannot be made due to invalid input
    ///   (e.g., delta per contract is zero) or if the required adjustment would violate contract limits
    ///   (e.g., attempting to sell more contracts than currently held).
    ///
    /// # Behavior
    /// - If `net_delta` is zero, no adjustment is needed, and the function immediately returns
    ///   `DeltaAdjustment::NoAdjustmentNeeded`.
    /// - If `option_delta_per_contract` is zero, the function returns a `GreeksError` as it is invalid
    ///   to use an option with no delta.
    /// - The number of contracts required to neutralize the delta is calculated as the absolute value
    ///   of `net_delta / option_delta_per_contract`.
    /// - Based on whether the net delta and option delta are positive or negative, the function
    ///   determines whether to buy or sell options. It also checks whether sufficient contracts are
    ///   available for sale or if additional contracts need to be acquired.
    /// - If the required contracts match the current quantity held in the portfolio, no adjustment is
    ///   needed.
    ///
    /// # Adjustment Logic
    /// 1. **Sell Options**:
    ///     - If the net delta and option delta per contract are both positive, selling options reduces
    ///       the delta.
    ///     - If the net delta and option delta per contract are both negative, selling options reduces
    ///       the negative delta.
    ///     - If the required number of contracts to sell exceeds the quantity currently held, an error
    ///       is returned because selling more than available is not possible.
    /// 2. **Buy Options**:
    ///     - If the net delta is positive and the option delta per contract is negative, buying options
    ///       adds negative delta (neutralizing the positive net delta).
    ///     - If the net delta is negative and the option delta per contract is positive, buying options
    ///       adds positive delta (neutralizing the negative net delta).
    ///
    /// # Errors
    /// - `GreeksError::StdError`:
    ///   - If `option_delta_per_contract` is zero because adjustments with zero delta options are
    ///     invalid.
    ///   - If insufficient contracts are available for an adjustment when trying to sell options.
    ///
    fn generate_delta_adjustments(
        &self,
        net_delta: Decimal,
        option_delta_per_contract: Decimal,
        option: &Options,
    ) -> Result<DeltaAdjustment, GreeksError> {
        if net_delta.is_zero() {
            return Ok(DeltaAdjustment::NoAdjustmentNeeded);
        }
        if option_delta_per_contract.is_zero() {
            return Err(GreeksError::StdError(
                "Option delta per contract cannot be zero".to_string(),
            ));
        }

        // Calculate how many contracts are needed to neutralize the net delta
        let total_contracts_needed = (net_delta / option_delta_per_contract).abs();
        if total_contracts_needed == option.quantity.to_dec() {
            return Ok(DeltaAdjustment::NoAdjustmentNeeded);
        }
        // The adjustment quantity is the difference between what we need and what we already have
        // For selling: if we already have more than needed, we sell the excess
        // For buying: if we have less than needed, we buy the difference
        let adjustment = match (
            net_delta.is_sign_positive(),
            option_delta_per_contract.is_sign_positive(),
            option.quantity.to_dec() > total_contracts_needed,
        ) {
            // If net_delta is positive and option_delta is positive, we need to sell options
            // This means we have too much positive delta and need to reduce it
            (true, true, true) => {
                // We already have more contracts than needed, sell the excess
                DeltaAdjustment::SellOptions {
                    quantity: Positive::from(total_contracts_needed),
                    strike: option.strike_price,
                    option_style: option.option_style,
                    side: option.side,
                }
            }
            // We have fewer contracts than needed, but since we need to sell,
            // we can't sell what we don't have, so no adjustment is possible
            (true, true, false) => {
                return Err(GreeksError::StdError(
                    "we can't sell what we don't have, so no adjustment is possible".to_string(),
                ));
            }
            // If net_delta is positive and option_delta is negative, we need to buy options
            // This means we have too much positive delta and need to add negative delta
            (true, false, true) => {
                // Calculate how many additional contracts we need to buy
                // discounting those we already have
                DeltaAdjustment::BuyOptions {
                    quantity: Positive::from(total_contracts_needed.abs()),
                    strike: option.strike_price,
                    option_style: option.option_style,
                    side: option.side,
                }
            }
            (true, false, false) => DeltaAdjustment::BuyOptions {
                quantity: Positive::from(total_contracts_needed.abs()),
                strike: option.strike_price,
                option_style: option.option_style,
                side: option.side,
            },
            // If net_delta is negative and option_delta is positive, we need to buy options
            // This means we have too much negative delta and need to add positive delta
            (false, true, true) => {
                // Calculate how many additional contracts we need to buy
                // discounting those we already have
                DeltaAdjustment::BuyOptions {
                    quantity: Positive::from(total_contracts_needed.abs()),
                    strike: option.strike_price,
                    option_style: option.option_style,
                    side: option.side,
                }
            }
            // We don't have enough contracts
            (false, true, false) => DeltaAdjustment::BuyOptions {
                quantity: Positive::from(total_contracts_needed.abs()),
                strike: option.strike_price,
                option_style: option.option_style,
                side: option.side,
            },
            // If net_delta is negative and option_delta is negative, we need to sell options
            // This means we have too much negative delta and need to reduce it
            (false, false, true) => {
                // We already have more contracts than needed, sell the excess
                DeltaAdjustment::SellOptions {
                    quantity: Positive::from(total_contracts_needed.abs()),
                    strike: option.strike_price,
                    option_style: option.option_style,
                    side: option.side,
                }
            }
            // We have fewer contracts than needed, but since we need to sell,
            // we can't sell what we don't have, so no adjustment is possible
            (false, false, false) => {
                return Err(GreeksError::StdError(
                    "we can't sell what we don't have, so no adjustment is possible".to_string(),
                ));
            }
        };

        Ok(adjustment)
    }

    /// Calculates required position adjustments to maintain delta neutrality
    ///
    /// # Arguments
    /// None - Uses internal position state
    ///
    /// # Returns
    /// * `Result<Vec<DeltaAdjustment>, GreeksError>` - Vector of suggested position adjustments
    ///   or error if calculations fail
    ///
    /// # Notes
    /// - Uses DELTA_THRESHOLD to determine if adjustments are needed
    /// - Suggests opposite positions to neutralize current delta exposure
    /// - Accounts for both option style (Put/Call) and position side (Long/Short)
    fn delta_adjustments(&self) -> Result<Vec<DeltaAdjustment>, GreeksError> {
        let net_delta = self.delta()?;

        // Check if adjustments are needed
        if net_delta.abs() <= DELTA_THRESHOLD {
            return Ok(vec![DeltaAdjustment::NoAdjustmentNeeded]);
        }

        let options = self.get_options()?;
        let mut adjustments = Vec::with_capacity(options.len());

        let mut total_size: Positive = Positive::ZERO;
        // Calculate adjustments for each option
        for option in &options {
            let option_delta_per_contract = option.delta()? / option.quantity.to_dec();
            total_size += option.quantity;
            if option_delta_per_contract.abs() > DELTA_THRESHOLD / dec!(10.0) {
                // Try to generate delta adjustments, but skip if not possible
                match self.generate_delta_adjustments(net_delta, option_delta_per_contract, option)
                {
                    Ok(adjustment) => adjustments.push(adjustment),
                    Err(_) => {
                        warn!("We might not be able to sell options if we don't have enough");
                        // Skip this adjustment if it's not possible
                        // We might not be able to sell options if we don't have enough
                        // This is acceptable as we'll try to adjust with other options
                    }
                }
            }
        }

        if options.len() == 2 {
            // Calculate delta neutral sizes based on the current options
            let (delta_neutral_size1, delta_neutral_size2) = calculate_delta_neutral_sizes(
                options[0].delta()?,
                options[1].delta()?,
                total_size,
            )?;

            // Calculate size differences (how much to adjust each position)
            let size_diff1: Decimal = delta_neutral_size1.to_dec() - options[0].quantity.to_dec();
            let size_diff2: Decimal = delta_neutral_size2.to_dec() - options[1].quantity.to_dec();

            // Create adjustment for the first option
            let adjustment1 = if size_diff1.is_sign_positive() {
                DeltaAdjustment::BuyOptions {
                    quantity: Positive(size_diff1.abs()),
                    strike: options[0].strike_price,
                    option_style: options[0].option_style,
                    side: options[0].side,
                }
            } else if !size_diff1.is_zero() {
                DeltaAdjustment::SellOptions {
                    quantity: Positive(size_diff1.abs()),
                    strike: options[0].strike_price,
                    option_style: options[0].option_style,
                    side: options[0].side,
                }
            } else {
                DeltaAdjustment::NoAdjustmentNeeded
            };

            // Create adjustment for the second option
            let adjustment2 = if size_diff2.is_sign_positive() {
                DeltaAdjustment::BuyOptions {
                    quantity: Positive(size_diff2.abs()),
                    strike: options[1].strike_price,
                    option_style: options[1].option_style,
                    side: options[1].side,
                }
            } else if !size_diff2.is_zero() {
                DeltaAdjustment::SellOptions {
                    quantity: Positive(size_diff2.abs()),
                    strike: options[1].strike_price,
                    option_style: options[1].option_style,
                    side: options[1].side,
                }
            } else {
                DeltaAdjustment::NoAdjustmentNeeded
            };

            // Only add the SameSize adjustment if both adjustments are significant
            match (&adjustment1, &adjustment2) {
                (DeltaAdjustment::NoAdjustmentNeeded, DeltaAdjustment::NoAdjustmentNeeded) => {
                    // Do nothing, both adjustments are insignificant
                }
                _ => {
                    adjustments.push(DeltaAdjustment::SameSize(DeltaAdjustmentSameSize {
                        first: Box::new(adjustment1),
                        second: Box::new(adjustment2),
                    }));
                }
            }
        }

        Ok(adjustments)
    }

    /// # Apply Delta Adjustments
    ///
    /// Applies delta-neutralizing adjustments to the current strategy based on calculated delta imbalances.
    /// This function ensures that the strategy remains delta-neutral by executing the appropriate
    /// position adjustments.
    ///
    /// ## Parameters
    /// * `action`: Optional filtering parameter that specifies which type of adjustments to apply:
    ///   - `Some(Action::Buy)`: Only apply options buying adjustments
    ///   - `Some(Action::Sell)`: Only apply options selling adjustments
    ///   - `None`: Apply all adjustment types, including paired adjustments
    ///
    /// ## Process
    /// 1. Calculates the current delta neutrality status of the strategy
    /// 2. If the strategy is already delta-neutral (within the configured threshold), returns early
    /// 3. Determines necessary adjustments to achieve delta neutrality
    /// 4. Applies appropriate adjustments based on the specified action filter:
    ///    - BuyOptions adjustments when Action::Buy is specified
    ///    - SellOptions adjustments when Action::Sell is specified
    ///    - All adjustments including paired SameSize adjustments when no action is specified
    ///
    /// ## Returns
    /// * `Result<(), StrategyError>` - Success if adjustments were applied successfully, or an error
    ///   if any adjustment operations failed
    ///
    /// ## Notes
    /// - The function uses the strategy's internal delta_neutrality() and delta_adjustments() methods
    ///   to determine the current state and required actions
    /// - SameSize adjustments are only applied when no specific action filter is provided
    /// - Incompatible adjustments for the specified action are skipped with a debug message
    fn apply_delta_adjustments(&mut self, action: Option<Action>) -> Result<(), StrategyError> {
        let delta_info = self.delta_neutrality()?;
        if delta_info.is_neutral {
            return Ok(());
        }

        for adjustment in self.delta_adjustments()? {
            match (action, adjustment) {
                // When action is Buy, only apply BuyOptions adjustments
                (
                    Some(Action::Buy),
                    DeltaAdjustment::BuyOptions {
                        quantity,
                        strike,
                        option_style,
                        side,
                    },
                ) => {
                    self.adjust_option_position(quantity.to_dec(), &strike, &option_style, &side)?;
                }

                // When action is Sell, only apply SellOptions adjustments
                (
                    Some(Action::Sell),
                    DeltaAdjustment::SellOptions {
                        quantity,
                        strike,
                        option_style,
                        side,
                    },
                ) => {
                    self.adjust_option_position(-quantity.to_dec(), &strike, &option_style, &side)?;
                }

                // When no action specified, apply all adjustments including SameSize
                (None, DeltaAdjustment::SameSize(adjustment)) => {
                    self.apply_single_adjustment(&adjustment.first)?;
                    self.apply_single_adjustment(&adjustment.second)?;
                }

                // Skip other combinations
                _ => {
                    debug!("Skipping adjustment - incompatible with requested action");
                }
            }
        }

        Ok(())
    }

    /// # Apply Single Adjustment
    ///
    /// Applies a single delta adjustment to the current position or strategy.
    ///
    /// This method processes a single `DeltaAdjustment` and modifies the object's state
    /// accordingly. It handles different types of adjustments that can be made to maintain
    /// or achieve delta neutrality in an options strategy.
    ///
    /// ## Parameters
    ///
    /// * `adjustment` - A reference to the `DeltaAdjustment` to apply, which can be one of several
    ///   variants specifying different types of position adjustments.
    ///
    /// ## Returns
    ///
    /// * `Result<(), StrategyError>` - Returns `Ok(())` if the adjustment was applied successfully,
    ///   or an `Error` if something went wrong during the process.
    ///
    /// ## Supported Adjustments
    ///
    /// * `BuyOptions` - Adds option contracts to the position with the specified parameters.
    /// * `SellOptions` - Removes option contracts from the position with the specified parameters.
    /// * `SameSize` - Currently not supported at the nested level (logs a debug message).
    /// * Other variants - Currently not implemented (logs a debug message).
    ///
    /// ## Notes
    ///
    /// The actual position adjustment is performed by the `adjust_option_position` method, which is
    /// called with positive quantities for buying options and negative quantities for selling options.
    ///
    fn apply_single_adjustment(
        &mut self,
        adjustment: &DeltaAdjustment,
    ) -> Result<(), StrategyError> {
        match adjustment {
            DeltaAdjustment::BuyOptions {
                quantity,
                strike,
                option_style,
                side,
            } => {
                debug!("Applying BuyOptions adjustment");
                self.adjust_option_position(quantity.to_dec(), strike, option_style, side)
            }
            DeltaAdjustment::SellOptions {
                quantity,
                strike,
                option_style,
                side,
            } => {
                debug!("Applying SellOptions adjustment");
                self.adjust_option_position(-quantity.to_dec(), strike, option_style, side)
            }
            DeltaAdjustment::SameSize(_) => {
                debug!("Nested SameSize adjustment not supported");
                Ok(())
            }
            _ => {
                debug!("Unknown adjustment type");
                Ok(())
            }
        }
    }

    /// # Adjust Option Position
    ///
    /// Modifies the quantity of an existing option position in a trading strategy.
    ///
    /// This method adjusts the quantity of an existing option position that matches the provided
    /// option type, side, and strike price. If the position is found, its quantity is increased
    /// or decreased by the specified amount. If the position is not found, an error is returned.
    ///
    /// ## Parameters
    ///
    /// * `quantity`: The decimal amount by which to adjust the position. Positive values increase
    ///   the position size, while negative values decrease it.
    /// * `strike`: The strike price of the option position to adjust.
    /// * `option_type`: The option style (Call or Put) of the position to adjust.
    /// * `side`: The side (Long or Short) of the position to adjust.
    ///
    /// ## Returns
    ///
    /// * `Ok(())` if the position was successfully adjusted.
    /// * `Err(PositionError::ValidationError)` if the position was not found.
    ///
    /// ## Errors
    ///
    /// Returns a boxed error if:
    /// - The specified position does not exist in the strategy
    /// - The `get_position` or `modify_position` methods fail
    ///
    fn adjust_option_position(
        &mut self,
        quantity: Decimal,
        strike: &Positive,
        option_type: &OptionStyle,
        side: &Side,
    ) -> Result<(), StrategyError> {
        let mut binding = self.get_position(option_type, side, strike)?;
        if let Some(current_position) = binding.first_mut() {
            let mut updated_position = (*current_position).clone();
            updated_position.option.quantity += quantity;
            self.modify_position(&updated_position)?;
        } else {
            return Err(PositionError::ValidationError(
                PositionValidationErrorKind::InvalidPosition {
                    reason: "Position not found".to_string(),
                },
            )
            .into());
        }
        Ok(())
    }

    /// Generates a `Trade` object based on the given delta adjustment action.
    ///
    /// # Parameters
    /// - `_action`: An `Action` representing the delta adjustment based on which the trade will be formulated.
    ///
    /// # Returns
    /// A `Trade` object derived from the delta adjustment logic.
    ///
    fn trade_from_delta_adjustment(&mut self, action: Action) -> Result<Vec<Trade>, StrategyError> {
        let adjustments = self.delta_adjustments()?;
        let mut trades = Vec::new();

        // Process a single BuyOptions or SellOptions adjustment
        let mut process_single_adjustment =
            |adj: &DeltaAdjustment| -> Result<Option<Trade>, StrategyError> {
                match adj {
                    DeltaAdjustment::BuyOptions {
                        quantity,
                        strike,
                        option_style,
                        side,
                    } => {
                        if quantity.is_zero() {
                            return Ok(None);
                        }
                        let positions = self.get_position(option_style, side, strike)?;
                        if let Some(position) = positions.first() {
                            let mut position_clone = (*position).clone();
                            position_clone.option.quantity = *quantity;
                            Ok(Some(position_clone.open()))
                        } else {
                            Ok(None)
                        }
                    }
                    DeltaAdjustment::SellOptions {
                        quantity,
                        strike,
                        option_style,
                        side,
                    } => {
                        if quantity.is_zero() {
                            return Ok(None);
                        }
                        let positions = self.get_position(option_style, side, strike)?;
                        if let Some(position) = positions.first() {
                            let mut position_clone = (*position).clone();
                            position_clone.option.quantity = *quantity;
                            Ok(Some(position_clone.close()))
                        } else {
                            Ok(None)
                        }
                    }
                    _ => Ok(None),
                }
            };

        for adjustment in adjustments {
            match (&action, adjustment) {
                // For Buy action, only process BuyOptions adjustments
                (Action::Buy, adj @ DeltaAdjustment::BuyOptions { .. }) => {
                    if let Some(trade) = process_single_adjustment(&adj)? {
                        trades.push(trade);
                    }
                }

                // For Sell action, only process SellOptions adjustments
                (Action::Sell, adj @ DeltaAdjustment::SellOptions { .. }) => {
                    if let Some(trade) = process_single_adjustment(&adj)? {
                        trades.push(trade);
                    }
                }

                // For Other action, process both adjustments in SameSize
                (Action::Other, DeltaAdjustment::SameSize(adjustment)) => {
                    if let Some(trade) = process_single_adjustment(&adjustment.first)? {
                        trades.push(trade);
                    }
                    if let Some(trade) = process_single_adjustment(&adjustment.second)? {
                        trades.push(trade);
                    }
                }

                // Ignore other combinations
                _ => {}
            }
        }

        Ok(trades)
    }

    /// Calculates portfolio-level Greeks for the strategy.
    ///
    /// This method aggregates all Greeks (delta, gamma, theta, vega, rho) across
    /// all positions in the strategy, providing a comprehensive view of the
    /// strategy's risk exposure.
    ///
    /// # Returns
    ///
    /// * `Ok(PortfolioGreeks)` - Aggregated Greeks for all positions
    /// * `Err(GreeksError)` - If any Greek calculation fails
    ///
    /// # Example
    ///
    /// ```ignore
    /// let greeks = strategy.portfolio_greeks()?;
    /// println!("Portfolio delta: {}", greeks.delta);
    /// println!("Portfolio gamma: {}", greeks.gamma);
    /// ```
    fn portfolio_greeks(&self) -> Result<PortfolioGreeks, GreeksError> {
        let positions: Vec<_> = self
            .get_positions()
            .map_err(|e| GreeksError::StdError(e.to_string()))?
            .into_iter()
            .cloned()
            .collect();
        PortfolioGreeks::from_positions(&positions)
            .map_err(|e| GreeksError::StdError(e.to_string()))
    }

    /// Generates an optimized adjustment plan to achieve target Greeks.
    ///
    /// This method uses the `AdjustmentOptimizer` to find the best set of
    /// adjustments to achieve the specified target Greeks, considering the
    /// provided configuration constraints.
    ///
    /// # Arguments
    ///
    /// * `config` - Configuration for adjustment behavior (what's allowed, constraints)
    /// * `target` - Target Greeks to achieve (e.g., delta neutral, delta-gamma neutral)
    ///
    /// # Returns
    ///
    /// * `Ok(AdjustmentPlan)` - The optimal adjustment plan
    /// * `Err(StrategyError)` - If no viable plan can be found
    ///
    /// # Example
    ///
    /// ```ignore
    /// let config = AdjustmentConfig::default();
    /// let target = AdjustmentTarget::delta_neutral();
    /// let plan = strategy.optimized_adjustment_plan(config, target)?;
    /// println!("Actions needed: {}", plan.actions.len());
    /// ```
    fn optimized_adjustment_plan(
        &self,
        config: AdjustmentConfig,
        target: AdjustmentTarget,
    ) -> Result<AdjustmentPlan, StrategyError> {
        let positions: Vec<_> = self.get_positions()?.into_iter().cloned().collect();
        let optimizer = AdjustmentOptimizer::new(&positions, config, target);
        optimizer.optimize().map_err(|e| {
            StrategyError::OperationError(OperationErrorKind::InvalidParameters {
                operation: "optimized_adjustment_plan".to_string(),
                reason: e.to_string(),
            })
        })
    }

    /// Generates an optimized adjustment plan using an option chain for new legs.
    ///
    /// This method extends `optimized_adjustment_plan` by allowing the optimizer
    /// to consider adding new option legs from the provided chain.
    ///
    /// # Arguments
    ///
    /// * `chain` - Option chain to source new legs from
    /// * `config` - Configuration for adjustment behavior
    /// * `target` - Target Greeks to achieve
    ///
    /// # Returns
    ///
    /// * `Ok(AdjustmentPlan)` - The optimal adjustment plan
    /// * `Err(StrategyError)` - If no viable plan can be found
    fn optimized_adjustment_plan_with_chain(
        &self,
        chain: &crate::chains::chain::OptionChain,
        config: AdjustmentConfig,
        target: AdjustmentTarget,
    ) -> Result<AdjustmentPlan, StrategyError> {
        let positions: Vec<_> = self.get_positions()?.into_iter().cloned().collect();
        let optimizer = AdjustmentOptimizer::with_chain(&positions, chain, config, target);
        optimizer.optimize().map_err(|e| {
            StrategyError::OperationError(OperationErrorKind::InvalidParameters {
                operation: "optimized_adjustment_plan_with_chain".to_string(),
                reason: e.to_string(),
            })
        })
    }

    /// Checks if the strategy meets the specified Greek targets.
    ///
    /// # Arguments
    ///
    /// * `target` - Target Greeks to check against
    /// * `tolerance` - Maximum deviation from target values
    ///
    /// # Returns
    ///
    /// `true` if all specified targets are met within tolerance
    fn meets_greek_targets(&self, target: &AdjustmentTarget, tolerance: Decimal) -> bool {
        match self.portfolio_greeks() {
            Ok(greeks) => target.is_satisfied(&greeks, tolerance),
            Err(_) => false,
        }
    }

    /// Returns the delta gap from a target value.
    ///
    /// # Arguments
    ///
    /// * `target_delta` - Target delta value (typically 0 for neutral)
    ///
    /// # Returns
    ///
    /// * `Ok(Decimal)` - The gap between current and target delta
    /// * `Err(GreeksError)` - If delta calculation fails
    fn delta_gap(&self, target_delta: Decimal) -> Result<Decimal, GreeksError> {
        let current_delta = self.delta()?;
        Ok(target_delta - current_delta)
    }
}

/// # DeltaNeutralResponse
///
/// Represents the analysis result of an options strategy's delta neutrality status and recommended adjustments.
///
/// This structure provides a comprehensive overview of a strategy's current delta status and suggests specific
/// adjustments that can be made to achieve or maintain delta neutrality. It serves as the primary output of
/// delta neutrality analysis operations.
///
/// ## Fields
/// * `delta_info`: Contains detailed information about the current delta status of the strategy, including the net delta,
///   individual position deltas, whether the strategy is currently considered neutral, and the neutrality threshold.
/// * `adjustments`: A list of recommended actions (such as buying/selling options or the underlying asset) that can be
///   taken to achieve or maintain delta neutrality for the strategy.
///
/// ## Purpose
/// This response structure helps traders and automated systems understand the current delta exposure of a strategy
/// and provides actionable recommendations to manage that exposure. It's particularly useful in risk management
/// contexts where maintaining delta neutrality is essential for minimizing directional market risk.
///
/// ## Usage Example
/// The `DeltaNeutralResponse` is typically generated by calling a delta neutrality assessment function on an
/// options strategy. Clients can then use the information to make trading decisions or execute the suggested
/// adjustments programmatically.
///
/// ## Related Concepts
/// Delta neutrality is achieved when the combined delta of all positions in a strategy is close to zero,
/// meaning the strategy's overall value is not immediately sensitive to small movements in the underlying asset's price.
///
#[derive(Serialize, Deserialize, ToSchema)]
pub struct DeltaNeutralResponse {
    /// Detailed information about the delta status of the strategy.
    pub delta_info: DeltaInfo,
    /// A list of recommended adjustments to achieve delta neutrality.
    pub adjustments: Vec<DeltaAdjustment>,
}

#[cfg(test)]
mod tests_display_implementations {
    use super::*;

    use rust_decimal_macros::dec;

    #[test]
    fn test_delta_adjustment_display() {
        // Test BuyOptions display
        let buy_options = DeltaAdjustment::BuyOptions {
            quantity: pos_or_panic!(3.0),
            strike: pos_or_panic!(105.0),
            option_style: OptionStyle::Call,
            side: Side::Long,
        };
        assert_eq!(
            buy_options.to_string(),
            "Buy 3 Long Call options at strike 105"
        );

        // Test SellOptions display
        let sell_options = DeltaAdjustment::SellOptions {
            quantity: pos_or_panic!(2.5),
            strike: pos_or_panic!(95.0),
            option_style: OptionStyle::Put,
            side: Side::Short,
        };
        assert_eq!(
            sell_options.to_string(),
            "Sell 2.5 Short Put options at strike 95"
        );

        // Test BuyUnderlying display
        let buy_underlying = DeltaAdjustment::BuyUnderlying(pos_or_panic!(10.0));
        assert_eq!(
            buy_underlying.to_string(),
            "Buy 10 units of the underlying asset"
        );

        // Test SellUnderlying display
        let sell_underlying = DeltaAdjustment::SellUnderlying(pos_or_panic!(5.0));
        assert_eq!(
            sell_underlying.to_string(),
            "Sell 5 units of the underlying asset"
        );

        // Test NoAdjustmentNeeded display
        let no_adjustment = DeltaAdjustment::NoAdjustmentNeeded;
        assert_eq!(no_adjustment.to_string(), "No adjustment needed");

        let same_size = DeltaAdjustmentSameSize {
            first: Box::new(DeltaAdjustment::BuyOptions {
                quantity: Positive::ONE,
                strike: Positive::HUNDRED,
                option_style: OptionStyle::Call,
                side: Side::Long,
            }),
            second: Box::new(DeltaAdjustment::SellOptions {
                quantity: Positive::ONE,
                strike: pos_or_panic!(110.0),
                option_style: OptionStyle::Call,
                side: Side::Short,
            }),
        };
        // Test SameSize display
        let same_size = DeltaAdjustment::SameSize(same_size);
        assert_eq!(
            same_size.to_string(),
            "Same size adjustments: [Buy 1 Long Call options at strike 100] and [Sell 1 Short Call options at strike 110]"
        );
    }

    #[test]
    fn test_delta_position_info_display() {
        let position_info = DeltaPositionInfo {
            delta: dec!(0.5),
            delta_per_contract: dec!(0.25),
            quantity: Positive::TWO,
            strike: Positive::HUNDRED,
            option_style: OptionStyle::Call,
            side: Side::Long,
        };

        // Since the Display implementation uses multiple writeln! calls,
        // we'll check for the presence of each line rather than the exact string
        let display_str = position_info.to_string();

        assert!(display_str.contains("Delta: 0.5000"));
        assert!(display_str.contains("Quantity: 2"));
        assert!(display_str.contains("Strike: 100"));
        assert!(display_str.contains("OptionStyle::Call"));
        assert!(display_str.contains("Side::Long"));
    }

    #[test]
    fn test_delta_info_display() {
        let delta_info = DeltaInfo {
            net_delta: dec!(-0.25),
            individual_deltas: vec![
                DeltaPositionInfo {
                    delta: dec!(0.5),
                    delta_per_contract: dec!(0.25),
                    quantity: Positive::ONE,
                    strike: Positive::HUNDRED,
                    option_style: OptionStyle::Call,
                    side: Side::Long,
                },
                DeltaPositionInfo {
                    delta: dec!(-0.75),
                    delta_per_contract: dec!(-0.375),
                    quantity: Positive::TWO,
                    strike: pos_or_panic!(95.0),
                    option_style: OptionStyle::Put,
                    side: Side::Short,
                },
            ],
            is_neutral: false,
            neutrality_threshold: dec!(0.1),
            underlying_price: pos_or_panic!(102.5),
        };

        // Similarly, we'll check for the presence of key components in the display string
        let display_str = delta_info.to_string();

        assert!(display_str.contains("Delta Analysis:"));
        assert!(display_str.contains("Net Delta: -0.2500"));
        assert!(display_str.contains("Is Neutral: false"));
        assert!(display_str.contains("Neutrality Threshold: 0.1000"));
        assert!(display_str.contains("Underlying Price: 102.5"));
        assert!(display_str.contains("Individual Deltas:"));
        assert!(display_str.contains("Position 1:"));
        assert!(display_str.contains("Delta: 0.5000"));
        assert!(display_str.contains("Position 2:"));
        assert!(display_str.contains("Delta: -0.7500"));
    }
}

#[cfg(test)]
mod tests_serialization {
    use super::*;
    use crate::ExpirationDate;
    use crate::strategies::ShortStrangle;
    use rust_decimal_macros::dec;
    use serde_json;
    use tracing::info;

    #[test]
    fn test_delta_adjustment_serialization() {
        // Test BuyOptions serialization/deserialization
        let buy_options = DeltaAdjustment::BuyOptions {
            quantity: pos_or_panic!(3.0),
            strike: pos_or_panic!(105.0),
            option_style: OptionStyle::Call,
            side: Side::Long,
        };
        let serialized = serde_json::to_string(&buy_options).unwrap();
        let deserialized: DeltaAdjustment = serde_json::from_str(&serialized).unwrap();
        assert_eq!(buy_options, deserialized);

        // Test SellOptions serialization/deserialization
        let sell_options = DeltaAdjustment::SellOptions {
            quantity: pos_or_panic!(2.5),
            strike: pos_or_panic!(95.0),
            option_style: OptionStyle::Put,
            side: Side::Short,
        };
        let serialized = serde_json::to_string(&sell_options).unwrap();
        let deserialized: DeltaAdjustment = serde_json::from_str(&serialized).unwrap();
        assert_eq!(sell_options, deserialized);

        // Test BuyUnderlying serialization/deserialization
        let buy_underlying = DeltaAdjustment::BuyUnderlying(pos_or_panic!(10.0));
        let serialized = serde_json::to_string(&buy_underlying).unwrap();
        let deserialized: DeltaAdjustment = serde_json::from_str(&serialized).unwrap();
        assert_eq!(buy_underlying, deserialized);

        // Test SellUnderlying serialization/deserialization
        let sell_underlying = DeltaAdjustment::SellUnderlying(pos_or_panic!(5.0));
        let serialized = serde_json::to_string(&sell_underlying).unwrap();
        let deserialized: DeltaAdjustment = serde_json::from_str(&serialized).unwrap();
        assert_eq!(sell_underlying, deserialized);

        // Test NoAdjustmentNeeded serialization/deserialization
        let no_adjustment = DeltaAdjustment::NoAdjustmentNeeded;
        let serialized = serde_json::to_string(&no_adjustment).unwrap();
        let deserialized: DeltaAdjustment = serde_json::from_str(&serialized).unwrap();
        assert_eq!(no_adjustment, deserialized);

        let same_size = DeltaAdjustmentSameSize {
            first: Box::new(DeltaAdjustment::BuyOptions {
                quantity: Positive::ONE,
                strike: Positive::HUNDRED,
                option_style: OptionStyle::Call,
                side: Side::Long,
            }),
            second: Box::new(DeltaAdjustment::SellOptions {
                quantity: Positive::ONE,
                strike: pos_or_panic!(110.0),
                option_style: OptionStyle::Call,
                side: Side::Short,
            }),
        };
        // Test SameSize serialization/deserialization
        let same_size = DeltaAdjustment::SameSize(same_size);
        let serialized = serde_json::to_string(&same_size).unwrap();
        let deserialized: DeltaAdjustment = serde_json::from_str(&serialized).unwrap();
        assert_eq!(same_size, deserialized);
    }

    #[test]
    fn test_delta_position_info_serialization() {
        let position_info = DeltaPositionInfo {
            delta: dec!(0.5),
            delta_per_contract: dec!(0.25),
            quantity: Positive::TWO,
            strike: Positive::HUNDRED,
            option_style: OptionStyle::Call,
            side: Side::Long,
        };

        let serialized = serde_json::to_string(&position_info).unwrap();
        let deserialized: DeltaPositionInfo = serde_json::from_str(&serialized).unwrap();

        assert_eq!(position_info.delta, deserialized.delta);
        assert_eq!(position_info.quantity, deserialized.quantity);
        assert_eq!(position_info.strike, deserialized.strike);
        assert_eq!(position_info.option_style, deserialized.option_style);
        assert_eq!(position_info.side, deserialized.side);
    }

    #[test]
    fn test_delta_info_serialization() {
        // Need to add #[derive(Serialize, Deserialize)] to DeltaInfo first
        // if it's not already there

        let delta_info = DeltaInfo {
            net_delta: dec!(-0.25),
            individual_deltas: vec![
                DeltaPositionInfo {
                    delta: dec!(0.5),
                    delta_per_contract: dec!(0.25),
                    quantity: Positive::ONE,
                    strike: Positive::HUNDRED,
                    option_style: OptionStyle::Call,
                    side: Side::Long,
                },
                DeltaPositionInfo {
                    delta: dec!(-0.75),
                    delta_per_contract: dec!(-0.375),
                    quantity: Positive::TWO,
                    strike: pos_or_panic!(95.0),
                    option_style: OptionStyle::Put,
                    side: Side::Short,
                },
            ],
            is_neutral: false,
            neutrality_threshold: dec!(0.1),
            underlying_price: pos_or_panic!(102.5),
        };

        let serialized = serde_json::to_string(&delta_info).unwrap();
        let deserialized: DeltaInfo = serde_json::from_str(&serialized).unwrap();

        assert_eq!(delta_info.net_delta, deserialized.net_delta);
        assert_eq!(delta_info.is_neutral, deserialized.is_neutral);
        assert_eq!(
            delta_info.neutrality_threshold,
            deserialized.neutrality_threshold
        );
        assert_eq!(delta_info.underlying_price, deserialized.underlying_price);
        assert_eq!(
            delta_info.individual_deltas.len(),
            deserialized.individual_deltas.len()
        );

        // Check each individual delta
        for (original, deserialized_delta) in delta_info
            .individual_deltas
            .iter()
            .zip(deserialized.individual_deltas.iter())
        {
            assert_eq!(original.delta, deserialized_delta.delta);
            assert_eq!(original.quantity, deserialized_delta.quantity);
            assert_eq!(original.strike, deserialized_delta.strike);
            assert_eq!(original.option_style, deserialized_delta.option_style);
            assert_eq!(original.side, deserialized_delta.side);
        }
    }

    #[test]
    fn test_specific_json_formats() {
        // Test that the JSON format is as expected
        let buy_options = DeltaAdjustment::BuyOptions {
            quantity: pos_or_panic!(3.0),
            strike: pos_or_panic!(105.0),
            option_style: OptionStyle::Call,
            side: Side::Long,
        };

        let serialized = serde_json::to_string(&buy_options).unwrap();
        // This test is fragile but can help catch unexpected changes in the JSON format
        assert!(serialized.contains("\"BuyOptions\""));
        assert!(serialized.contains("\"quantity\""));
        assert!(serialized.contains("\"strike\""));
        assert!(serialized.contains("\"option_style\""));
        assert!(serialized.contains("\"side\""));

        // Parse a manually constructed JSON and verify it deserializes correctly
        let json_str =
            r#"{"BuyOptions":{"quantity":3.0,"strike":105.0,"option_style":"Call","side":"Long"}}"#;
        let deserialized: DeltaAdjustment = serde_json::from_str(json_str).unwrap();

        match deserialized {
            DeltaAdjustment::BuyOptions {
                quantity,
                strike,
                option_style,
                side,
            } => {
                assert_eq!(quantity, pos_or_panic!(3.0));
                assert_eq!(strike, pos_or_panic!(105.0));
                assert_eq!(option_style, OptionStyle::Call);
                assert_eq!(side, Side::Long);
            }
            _ => panic!("Deserialized to wrong variant"),
        }
    }

    #[test]
    fn test_delta_response_serialization() {
        let strategy = ShortStrangle::new(
            "CL".to_string(),
            pos_or_panic!(7250.0), // underlying_price
            pos_or_panic!(7450.0), // call_strike
            pos_or_panic!(7050.0), // put_strike
            ExpirationDate::Days(pos_or_panic!(45.0)),
            pos_or_panic!(0.3745), // implied_volatility
            pos_or_panic!(0.3745), // implied_volatility
            dec!(0.05),            // risk_free_rate
            Positive::ZERO,        // dividend_yield
            Positive::TWO,    // quantity
            pos_or_panic!(84.2),   // premium_short_call
            pos_or_panic!(353.2),  // premium_short_put
            pos_or_panic!(7.01),   // open_fee_short_call
            pos_or_panic!(7.01),   // close_fee_short_call
            pos_or_panic!(7.01),   // open_fee_short_put
            pos_or_panic!(7.01),   // close_fee_short_put
        );
        let delta_info = strategy.delta_neutrality().unwrap();
        let adjustments = strategy.delta_adjustments().unwrap();
        let response = DeltaNeutralResponse {
            delta_info,
            adjustments,
        };

        // serialize and pretty print
        let serialized = serde_json::to_string_pretty(&response).unwrap();
        info!("{}", serialized);
    }
}

#[cfg(test)]
mod tests_generate_delta_adjustments {
    use super::*;
    use crate::ExpirationDate;
    use crate::strategies::base::BreakEvenable;
    use crate::strategies::{BasicAble, Validable};

    struct MockDeltaNeutral;
    impl Greeks for MockDeltaNeutral {
        fn get_options(&self) -> Result<Vec<&Options>, GreeksError> {
            Ok(Vec::new())
        }
    }
    impl Positionable for MockDeltaNeutral {}
    impl Strategies for MockDeltaNeutral {}
    impl Validable for MockDeltaNeutral {}
    impl BreakEvenable for MockDeltaNeutral {}
    impl BasicAble for MockDeltaNeutral {}
    impl DeltaNeutrality for MockDeltaNeutral {}

    // Helper function to create a test option
    fn create_test_option(option_style: OptionStyle, side: Side, size: Positive) -> Options {
        Options {
            option_type: crate::model::types::OptionType::European,
            side,
            underlying_symbol: "TEST".to_string(),
            strike_price: Positive::HUNDRED,
            expiration_date: ExpirationDate::Days(pos_or_panic!(30.0)),
            implied_volatility: pos_or_panic!(0.2),
            quantity: size,
            underlying_price: Positive::HUNDRED,
            risk_free_rate: dec!(0.05),
            option_style,
            dividend_yield: pos_or_panic!(0.01),
            exotic_params: None,
        }
    }

    #[test]
    fn test_generate_delta_adjustments_positive_net_delta_positive_option_delta() {
        // Arrange
        let delta_neutral = MockDeltaNeutral;
        let net_delta = dec!(0.5); // Positive net delta
        let option_delta_per_contract = dec!(0.25); // Positive option delta
        let option = create_test_option(OptionStyle::Call, Side::Long, pos_or_panic!(2.7));

        // Act
        let adjustment = delta_neutral
            .generate_delta_adjustments(net_delta, option_delta_per_contract, &option)
            .unwrap();

        // Assert
        match adjustment {
            DeltaAdjustment::SellOptions {
                quantity,
                strike,
                option_style,
                side,
            } => {
                assert_eq!(quantity, Positive::new(2.0).unwrap()); // 0.5 / 0.25 = 2.0 diff to 2.7 = 0.7 to sell
                assert_eq!(strike, option.strike_price);
                assert_eq!(option_style, option.option_style);
                assert_eq!(side, option.side);
            }
            _ => panic!("Expected SellOptions adjustment"),
        }
    }

    #[test]
    fn test_generate_delta_adjustments_positive_net_delta_negative_option_delta() {
        // Arrange
        let delta_neutral = MockDeltaNeutral;
        let net_delta = dec!(0.5); // Positive net delta
        let option_delta_per_contract = dec!(-0.25); // Negative option delta
        let option = create_test_option(OptionStyle::Put, Side::Long, pos_or_panic!(3.7));

        // Act
        let adjustment = delta_neutral
            .generate_delta_adjustments(net_delta, option_delta_per_contract, &option)
            .unwrap();

        // Assert
        match adjustment {
            DeltaAdjustment::BuyOptions {
                quantity,
                strike,
                option_style,
                side,
            } => {
                // For positive net delta and negative option delta, we need to buy options
                // The quantity is the total contracts needed to neutralize (2.0)
                assert_eq!(quantity, Positive::new(2.0).unwrap()); // 0.5 / -0.25 = -2.0, but we take abs()
                assert_eq!(strike, option.strike_price);
                assert_eq!(option_style, option.option_style);
                assert_eq!(side, option.side);
            }
            _ => panic!("Expected BuyOptions adjustment"),
        }
    }

    #[test]
    fn test_generate_delta_adjustments_positive_net_delta_negative_option_delta_bis() {
        // Arrange
        let delta_neutral = MockDeltaNeutral;
        let net_delta = dec!(0.5); // Positive net delta
        let option_delta_per_contract = dec!(-0.25); // Negative option delta
        let option = create_test_option(OptionStyle::Put, Side::Long, pos_or_panic!(1.7));

        // Act
        let adjustment = delta_neutral
            .generate_delta_adjustments(net_delta, option_delta_per_contract, &option)
            .unwrap();

        // Assert
        match adjustment {
            DeltaAdjustment::BuyOptions {
                quantity,
                strike,
                option_style,
                side,
            } => {
                // For positive net delta and negative option delta, we need to buy options
                // The quantity is the total contracts needed to neutralize (2.0)
                assert_eq!(quantity, Positive::new(2.0).unwrap());
                assert_eq!(strike, option.strike_price);
                assert_eq!(option_style, option.option_style);
                assert_eq!(side, option.side);
            }
            _ => panic!("Expected BuyOptions adjustment"),
        }
    }

    #[test]
    fn test_generate_delta_adjustments_negative_net_delta_positive_option_delta() {
        // Arrange
        let delta_neutral = MockDeltaNeutral;
        let net_delta = dec!(-0.5); // Negative net delta
        let option_delta_per_contract = dec!(0.25); // Positive option delta
        let option = create_test_option(OptionStyle::Call, Side::Long, pos_or_panic!(21.7));

        // Act
        let adjustment = delta_neutral
            .generate_delta_adjustments(net_delta, option_delta_per_contract, &option)
            .unwrap();

        // Assert
        match adjustment {
            DeltaAdjustment::BuyOptions {
                quantity,
                strike,
                option_style,
                side,
            } => {
                // For negative net delta and positive option delta, we need to buy options
                // The quantity is the total contracts needed to neutralize (2.0)
                assert_eq!(quantity, Positive::new(2.0).unwrap()); // -0.5 / 0.25 = -2.0, pero tomamos abs()
                assert_eq!(strike, option.strike_price);
                assert_eq!(option_style, option.option_style);
                assert_eq!(side, option.side);
            }
            _ => panic!("Expected BuyOptions adjustment"),
        }
    }

    #[test]
    fn test_generate_delta_adjustments_negative_net_delta_positive_option_delta_bis() {
        // Arrange
        let delta_neutral = MockDeltaNeutral;
        let net_delta = dec!(-0.5); // Negative net delta
        let option_delta_per_contract = dec!(0.25); // Positive option delta
        let option = create_test_option(OptionStyle::Call, Side::Long, pos_or_panic!(1.7));

        // Act
        let adjustment = delta_neutral
            .generate_delta_adjustments(net_delta, option_delta_per_contract, &option)
            .unwrap();

        // Assert
        match adjustment {
            DeltaAdjustment::BuyOptions {
                quantity,
                strike,
                option_style,
                side,
            } => {
                // For negative net delta and positive option delta, we need to buy options
                // The quantity is the total contracts needed to neutralize (2.0)
                assert_eq!(quantity, Positive::new(2.0).unwrap()); // -0.5 / 0.25 = -2.0, pero tomamos abs()
                assert_eq!(strike, option.strike_price);
                assert_eq!(option_style, option.option_style);
                assert_eq!(side, option.side);
            }
            _ => panic!("Expected BuyOptions adjustment"),
        }
    }

    #[test]
    fn test_generate_delta_adjustments_negative_net_delta_negative_option_delta_error() {
        // Arrange
        let delta_neutral = MockDeltaNeutral;
        let net_delta = dec!(-0.5); // Negative net delta
        let option_delta_per_contract = dec!(-0.25); // Negative option delta
        let option = create_test_option(OptionStyle::Put, Side::Long, pos_or_panic!(1.5));

        // Act
        let adjustment =
            delta_neutral.generate_delta_adjustments(net_delta, option_delta_per_contract, &option);

        assert!(adjustment.is_err());
    }

    #[test]
    fn test_generate_delta_adjustments_negative_net_delta_negative_option_delta() {
        // Arrange
        let delta_neutral = MockDeltaNeutral;
        let net_delta = dec!(-0.5); // Negative net delta
        let option_delta_per_contract = dec!(-0.25); // Negative option delta
        let option = create_test_option(OptionStyle::Put, Side::Long, pos_or_panic!(10.5));

        // Act
        let adjustment = delta_neutral
            .generate_delta_adjustments(net_delta, option_delta_per_contract, &option)
            .unwrap();

        // Assert
        match adjustment {
            DeltaAdjustment::SellOptions {
                quantity,
                strike,
                option_style,
                side,
            } => {
                assert_eq!(quantity, Positive::TWO);
                assert_eq!(strike, option.strike_price);
                assert_eq!(option_style, option.option_style);
                assert_eq!(side, option.side);
            }
            _ => panic!("Expected SellOptions adjustment"),
        }
    }

    #[test]
    fn test_generate_delta_adjustments_zero_option_delta() {
        // Arrange
        let delta_neutral = MockDeltaNeutral;
        let net_delta = dec!(0.5); // Any non-zero value
        let option_delta_per_contract = dec!(0.0); // Zero option delta
        let option = create_test_option(OptionStyle::Call, Side::Long, pos_or_panic!(7.0));

        // Division by zero should be handled gracefully
        let result =
            delta_neutral.generate_delta_adjustments(net_delta, option_delta_per_contract, &option);

        assert!(result.is_err());
    }

    #[test]
    fn test_generate_delta_adjustments_zero_net_delta() {
        // Arrange
        let delta_neutral = MockDeltaNeutral;
        let net_delta = dec!(0.0); // Zero net delta
        let option_delta_per_contract = dec!(0.25); // Non-zero option delta
        let option = create_test_option(OptionStyle::Call, Side::Long, pos_or_panic!(2.7223423));

        // Act
        let adjustment = delta_neutral
            .generate_delta_adjustments(net_delta, option_delta_per_contract, &option)
            .unwrap();

        // Assert
        match adjustment {
            DeltaAdjustment::NoAdjustmentNeeded => {
                // Expect no adjustment needed when net delta is zero
            }
            _ => panic!("Expected Buy/SellOptions adjustment with zero quantity"),
        }
    }

    #[test]
    fn test_generate_delta_adjustments_with_different_option_styles() {
        // Test with Put option
        let delta_neutral = MockDeltaNeutral;
        let net_delta = dec!(0.5);
        let option_delta_per_contract = dec!(0.25);
        let put_option =
            create_test_option(OptionStyle::Put, Side::Long, pos_or_panic!(5.7534523452435));

        let adjustment = delta_neutral
            .generate_delta_adjustments(net_delta, option_delta_per_contract, &put_option)
            .unwrap();

        match adjustment {
            DeltaAdjustment::SellOptions {
                option_style,
                quantity,
                ..
            } => {
                assert_eq!(option_style, OptionStyle::Put);
                let expected = Positive::TWO;
                assert_eq!(quantity, expected);
            }
            _ => panic!("Expected SellOptions adjustment"),
        }
    }

    #[test]
    fn test_generate_delta_adjustments_with_different_sides() {
        // Test with Short side
        let delta_neutral = MockDeltaNeutral;
        let net_delta = dec!(0.5);
        let option_delta_per_contract = dec!(0.25);
        let short_option = create_test_option(OptionStyle::Call, Side::Short, Positive::ONE);

        let adjustment = delta_neutral.generate_delta_adjustments(
            net_delta,
            option_delta_per_contract,
            &short_option,
        );

        assert!(adjustment.is_err());
    }

    // Add an additional test for the case of selling with a position larger than needed
    #[test]
    fn test_generate_delta_adjustments_negative_net_delta_negative_option_delta_with_excess() {
        // Arrange
        let delta_neutral = MockDeltaNeutral;
        let net_delta = dec!(-0.5); // Negative net delta
        let option_delta_per_contract = dec!(-0.25); // Negative option delta
        let option = create_test_option(OptionStyle::Put, Side::Long, pos_or_panic!(3.0)); // Más de lo necesario

        // Act
        let adjustment = delta_neutral
            .generate_delta_adjustments(net_delta, option_delta_per_contract, &option)
            .unwrap();

        // Assert
        match adjustment {
            DeltaAdjustment::SellOptions {
                quantity,
                strike,
                option_style,
                side,
            } => {
                // For negative net delta and negative option delta, we need to sell options
                // The total quantity needed to neutralize is 2.0
                // As the current position is 3.0, we need to sell 3.0 - 2.0 = 1.0
                assert_eq!(quantity, Positive::new(2.0).unwrap());
                assert_eq!(strike, option.strike_price);
                assert_eq!(option_style, option.option_style);
                assert_eq!(side, option.side);
            }
            _ => panic!("Expected SellOptions adjustment"),
        }
    }

    #[test]
    fn test_generate_delta_adjustments_large_values() {
        // Test with large values to ensure no overflow
        let delta_neutral = MockDeltaNeutral;
        let net_delta = dec!(1000.0);
        let option_delta_per_contract = dec!(0.01);
        let option = create_test_option(OptionStyle::Call, Side::Long, pos_or_panic!(1.7));

        let adjustment =
            delta_neutral.generate_delta_adjustments(net_delta, option_delta_per_contract, &option);

        assert!(adjustment.is_err());
    }
}
