/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 10/12/24
******************************************************************************/
use crate::error::position::PositionValidationErrorKind;
use crate::error::{GreeksError, PositionError, StrategyError};
use crate::greeks::calculate_delta_neutral_sizes;
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
use crate::model::types::{Action, OptionStyle};
use crate::strategies::base::Positionable;
use crate::strategies::Strategies;
use crate::{Options, Positive, Side};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;
use tracing::info;

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
#[derive(Debug, PartialEq, Serialize, Deserialize)]
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
    SameSize(Box<DeltaAdjustment>, Box<DeltaAdjustment>),
}

/// Represents the delta and associated details for a single position in an options strategy.
///
/// ## Fields
/// - `delta`: The delta value of the position, representing the sensitivity of the position's price
///   to changes in the underlying asset price.
/// - `quantity`: The quantity of the options in the position.
/// - `strike`: The strike price of the option, represented as a positive value.
/// - `option_style`: Indicates whether the option is a call or a put.
/// - `side`: Indicates whether the position is long or short.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct DeltaPositionInfo {
    pub delta: Decimal,
    pub quantity: Positive,
    pub strike: Positive,
    pub option_style: OptionStyle,
    pub side: Side,
}
impl fmt::Display for DeltaPositionInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Delta: {:.4}", self.delta)?;
        writeln!(f, "  Quantity: {}", self.quantity)?;
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
/// ## Fields
/// - `net_delta`: The net delta of the strategy, representing the overall sensitivity
///   of the strategy to changes in the underlying asset price.
/// - `individual_deltas`: A vector of `DeltaPositionInfo` structures containing
///   the delta information for each position in the strategy.
/// - `is_neutral`: Indicates whether the strategy is considered delta neutral based
///   on the configured neutrality threshold.
/// - `neutrality_threshold`: The threshold used to determine if the strategy is delta neutral.
///   If the net delta is within this range, the strategy is considered neutral.
/// - `underlying_price`: The current price of the underlying asset, represented as a positive value.
///
/// ## Purpose
/// DeltaInfo serves as a central structure to analyze and manage the delta status
/// of multi-position strategies, such as those used in options trading. It is particularly
/// useful for implementing delta-neutral strategy adjustments.
#[derive(Debug, Clone)]
pub struct DeltaInfo {
    pub net_delta: Decimal,
    pub individual_deltas: Vec<DeltaPositionInfo>,
    pub is_neutral: bool,
    pub neutrality_threshold: Decimal,
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
        let underlying_price = self.get_underlying_price();
        let individual_deltas: Vec<DeltaPositionInfo> = options
            .iter()
            .map(|option| DeltaPositionInfo {
                delta: option.delta().unwrap(),
                quantity: option.quantity,
                strike: option.strike_price,
                option_style: option.option_style,
                side: option.side,
            })
            .collect();

        Ok(DeltaInfo {
            net_delta: self.delta()?,
            individual_deltas: individual_deltas,
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

    fn get_atm_strike(&self) -> Result<Positive, StrategyError> {
        Ok(self.get_underlying_price())
    }

    /// Calculates required position adjustments to maintain delta neutrality
    ///
    /// # Arguments
    /// None - Uses internal position state
    ///
    /// # Returns
    /// * `Result<Vec<DeltaAdjustment>, GreeksError>` - Vector of suggested position adjustments
    /// or error if calculations fail
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

        // Helper to determine adjustment type based on delta sign and position
        let get_adjustment = |net_delta: Decimal, option_delta: Decimal, option: &Options| -> DeltaAdjustment {
        
            match (net_delta.is_sign_positive(), option_delta.is_sign_positive()) {
                (true, true) => DeltaAdjustment::SellOptions {
                    quantity: Positive((net_delta / option_delta).abs()),
                    strike: option.strike_price,
                    option_style: option.option_style,
                    side: option.side,
                },
                (true, false) => DeltaAdjustment::BuyOptions {
                    quantity: Positive((net_delta / option_delta).abs()),
                    strike: option.strike_price,
                    option_style: option.option_style,
                    side: option.side,
                },
                (false, true) => DeltaAdjustment::BuyOptions {
                    quantity: Positive((net_delta / option_delta).abs()),
                    strike: option.strike_price,
                    option_style: option.option_style,
                    side: option.side,
                },
                (false, false) => DeltaAdjustment::SellOptions {
                    quantity: Positive((net_delta / option_delta).abs()),
                    strike: option.strike_price,
                    option_style: option.option_style,
                    side: option.side,
                },
            }
        };

        let mut total_size: Positive = Positive::ZERO;
        // Calculate adjustments for each option
        for option in &options {
            let delta = option.delta()?;
            total_size += option.quantity;
            if delta.abs() > DELTA_THRESHOLD {
                // Avoid division by zero
                let adjustment = get_adjustment(net_delta, delta / option.quantity, option);
                adjustments.push(adjustment);
            }
        }

        if options.len() == 2 {
            // Calculate delta neutral sizes based on the current options
            let (delta_neutral_size1, delta_neutral_size2) = calculate_delta_neutral_sizes(
                options[0].delta()?,
                options[1].delta()?,
                total_size,
            )?;

            // Calculate position differences (how much to adjust each position)
            let size_diff1: Decimal = delta_neutral_size1.to_dec() - options[0].quantity.to_dec();
            let size_diff2: Decimal = delta_neutral_size2.to_dec() - options[1].quantity.to_dec();

            // Create adjustment for first option
            let adjustment1 = if size_diff1.is_sign_positive() {
                DeltaAdjustment::BuyOptions {
                    quantity: Positive(size_diff1.abs()),
                    strike: options[0].strike_price,
                    option_style: options[0].option_style.clone(),
                    side: options[0].side,
                }
            } else {
                DeltaAdjustment::SellOptions {
                    quantity: Positive(size_diff1.abs()),
                    strike: options[0].strike_price,
                    option_style: options[0].option_style.clone(),
                    side: options[0].side,
                }
            };

            // Create adjustment for second option
            let adjustment2 = if size_diff2.is_sign_positive() {
                DeltaAdjustment::BuyOptions {
                    quantity: Positive(size_diff2.abs()),
                    strike: options[1].strike_price,
                    option_style: options[1].option_style.clone(),
                    side: options[1].side,
                }
            } else {
                DeltaAdjustment::SellOptions {
                    quantity: Positive(size_diff2.abs()),
                    strike: options[1].strike_price,
                    option_style: options[1].option_style.clone(),
                    side: options[1].side,
                }
            };

            adjustments.push(DeltaAdjustment::SameSize(
                Box::new(adjustment1),
                Box::new(adjustment2),
            ));
        }

        Ok(adjustments)
    }

    fn apply_delta_adjustments(
        &mut self,
        action: Option<Action>,
    ) -> Result<(), Box<dyn Error>> {
        let delta_info = self.delta_neutrality()?;
        if delta_info.is_neutral {
            return Ok(());
        }

        for adjustment in self.delta_adjustments()? {
            match adjustment {
                DeltaAdjustment::BuyUnderlying(quantity) => {
                    if side.is_none() || side == Some(Side::Long) {
                        self.adjust_underlying_position(quantity, Side::Long)?;
                    }
                }
                DeltaAdjustment::SellUnderlying(quantity) => {
                    if side.is_none() || side == Some(Side::Short) {
                        self.adjust_underlying_position(quantity, Side::Short)?;
                    }
                }
                DeltaAdjustment::BuyOptions {
                    quantity,
                    strike,
                    option_style,
                    side,
                } => {
                    self.adjust_option_position(quantity.to_dec(), &strike, &option_style, &side)?;
                }
                DeltaAdjustment::SellOptions {
                    quantity,
                    strike,
                    option_style,
                    side,
                } => {
                    self.adjust_option_position(quantity.to_dec(), &strike, &option_style, &side)?;
                }
                DeltaAdjustment::NoAdjustmentNeeded => {
                    return Ok(());
                }
                DeltaAdjustment::SameSize(_, _) => {
                    // TODO Implement SameSize adjustment
                    return Err(Box::new(PositionError::ValidationError(
                        PositionValidationErrorKind::InvalidPosition {
                            reason: "SameSize adjustment not supported".to_string(),
                        },
                    )));
                }
            }
        }

        // Log skipped adjustments if filters were applied
        if side.is_some() || option_style.is_some() {
            info!(
                "Applied delta adjustments with filters - Side: {:?}, OptionStyle: {:?}",
                side, option_style
            );
        }

        Ok(())
    }

    fn adjust_underlying_position(
        &mut self,
        _quantity: Positive,
        _side: Side,
    ) -> Result<(), Box<dyn Error>> {
        // Implementation for adjusting underlying position
        // This would typically modify the quantity of the underlying asset
        unimplemented!("Implement underlying position adjustment")
    }

    fn adjust_option_position(
        &mut self,
        quantity: Decimal,
        strike: &Positive,
        option_type: &OptionStyle,
        side: &Side,
    ) -> Result<(), Box<dyn Error>> {
        let mut binding = self.get_position(option_type, side, strike)?;
        if let Some(current_position) = binding.first_mut() {
            let mut updated_position = (*current_position).clone();
            updated_position.option.quantity += quantity;
            self.modify_position(&updated_position)?;
        } else {
            return Err(Box::new(PositionError::ValidationError(
                PositionValidationErrorKind::InvalidPosition {
                    reason: "Position not found".to_string(),
                },
            )));
        }
        Ok(())
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::error::GreeksError;
//     use crate::greeks::Greek;
//     use crate::{pos, Options};
//     use rust_decimal::Decimal;
//     use rust_decimal_macros::dec;
//
//     // Mock struct to implement required traits for testing
//     struct MockStrategy {
//         delta: Decimal,
//         underlying_price: Positive,
//         individual_deltas: Vec<Decimal>,
//     }
//
//     // Implement Greeks trait for MockStrategy
//     impl Greeks for MockStrategy {
//         fn get_options(&self) -> Result<Vec<&Options>, GreeksError> {
//             Ok(vec![])
//         }
//
//         fn greeks(&self) -> Result<Greek, GreeksError> {
//             Ok(Greek {
//                 delta: self.delta,
//                 gamma: Decimal::ZERO,
//                 theta: Decimal::ZERO,
//                 vega: Decimal::ZERO,
//                 rho: Decimal::ZERO,
//                 rho_d: Decimal::ZERO,
//                 alpha: Decimal::ZERO,
//             })
//         }
//     }
//
//     impl Positionable for MockStrategy {}
//
//     // Implement DeltaNeutrality trait for MockStrategy
//     impl DeltaNeutrality for MockStrategy {
//         fn calculate_net_delta(&self) -> DeltaInfo {
//             DeltaInfo {
//                 net_delta: self.delta,
//                 individual_deltas: self.individual_deltas.clone(),
//                 is_neutral: self.delta.abs() <= dec!(0.01),
//                 neutrality_threshold: dec!(0.01),
//                 underlying_price: self.underlying_price,
//             }
//         }
//
//         fn get_atm_strike(&self) -> Positive {
//             pos!(100.0)
//         }
//     }
//
//     // Helper function to create a mock strategy
//     fn create_mock_strategy(delta: Decimal, price: Positive) -> MockStrategy {
//         MockStrategy {
//             delta,
//             underlying_price: price,
//             individual_deltas: vec![delta],
//         }
//     }
//
//     #[test]
//     #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
//     fn test_calculate_net_delta() {
//         let strategy = create_mock_strategy(dec!(0.5), pos!(100.0));
//         let info = strategy.calculate_net_delta();
//
//         assert_eq!(info.net_delta, dec!(0.5));
//         assert_eq!(info.individual_deltas, vec![dec!(0.5)]);
//         assert!(!info.is_neutral);
//         assert_eq!(info.underlying_price, pos!(100.0));
//     }
//
//     #[test]
//     #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
//     fn test_is_delta_neutral() {
//         let neutral_strategy = create_mock_strategy(dec!(0.00005), pos!(100.0));
//         let non_neutral_strategy = create_mock_strategy(dec!(0.5), pos!(100.0));
//
//         assert!(neutral_strategy.is_delta_neutral());
//         assert!(!non_neutral_strategy.is_delta_neutral());
//     }
//
//     #[test]
//     #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
//     fn test_suggest_delta_adjustments_neutral() {
//         let strategy = create_mock_strategy(dec!(0.005), pos!(100.0));
//         let adjustments = strategy.delta_adjustments().unwrap();
//
//         assert_eq!(adjustments, vec![DeltaAdjustment::NoAdjustmentNeeded]);
//     }
//
//     #[test]
//     #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
//     fn test_suggest_delta_adjustments_positive() {
//         let strategy = create_mock_strategy(dec!(0.5), pos!(100.0));
//         let adjustments = strategy.delta_adjustments().unwrap();
//
//         assert_eq!(
//             adjustments,
//             vec![
//                 DeltaAdjustment::SellUnderlying(pos!(0.5)),
//                 DeltaAdjustment::BuyOptions {
//                     quantity: pos!(1.0),
//                     strike: pos!(100.0),
//                     option_type: OptionStyle::Put,
//                 },
//                 DeltaAdjustment::SellOptions {
//                     quantity: pos!(1.0),
//                     strike: pos!(100.0),
//                     option_type: OptionStyle::Call,
//                 }
//             ]
//         );
//     }
//
//     #[test]
//     #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
//     fn test_suggest_delta_adjustments_negative() {
//         let strategy = create_mock_strategy(dec!(-0.5), pos!(100.0));
//         let adjustments = strategy.delta_adjustments().unwrap();
//
//         assert_eq!(
//             adjustments,
//             vec![
//                 DeltaAdjustment::BuyUnderlying(pos!(0.5)),
//                 DeltaAdjustment::BuyOptions {
//                     quantity: pos!(1.0),
//                     strike: pos!(100.0),
//                     option_type: OptionStyle::Call,
//                 },
//                 DeltaAdjustment::SellOptions {
//                     quantity: pos!(1.0),
//                     strike: pos!(100.0),
//                     option_type: OptionStyle::Put,
//                 }
//             ]
//         );
//     }
//
//     #[test]
//     #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
//     fn test_delta_info_display() {
//         let strategy = create_mock_strategy(dec!(0.5), pos!(100.0));
//         let info = strategy.calculate_net_delta();
//         let display_string = format!("{}", info);
//
//         assert!(display_string.contains("Net Delta: 0.5000"));
//         assert!(display_string.contains("Is Neutral: false"));
//         assert!(display_string.contains("Underlying Price: 100"));
//         assert!(display_string.contains("Individual Deltas:"));
//     }
//
//     #[test]
//     #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
//     fn test_generate_delta_reducing_adjustments() {
//         let strategy = create_mock_strategy(dec!(0.5), pos!(100.0));
//         let adjustments = strategy.generate_delta_reducing_adjustments();
//
//         assert_eq!(
//             adjustments,
//             vec![
//                 DeltaAdjustment::SellUnderlying(pos!(0.5)),
//                 DeltaAdjustment::BuyOptions {
//                     quantity: pos!(1.0),
//                     strike: pos!(100.0),
//                     option_type: OptionStyle::Put,
//                 },
//                 DeltaAdjustment::SellOptions {
//                     quantity: pos!(1.0),
//                     strike: pos!(100.0),
//                     option_type: OptionStyle::Call,
//                 }
//             ]
//         );
//     }
//
//     #[test]
//     #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
//     fn test_generate_delta_increasing_adjustments() {
//         let strategy = create_mock_strategy(dec!(-0.5), pos!(100.0));
//         let adjustments = strategy.generate_delta_increasing_adjustments();
//
//         assert_eq!(
//             adjustments,
//             vec![
//                 DeltaAdjustment::BuyUnderlying(pos!(0.5)),
//                 DeltaAdjustment::BuyOptions {
//                     quantity: pos!(1.0),
//                     strike: pos!(100.0),
//                     option_type: OptionStyle::Call,
//                 },
//                 DeltaAdjustment::SellOptions {
//                     quantity: pos!(1.0),
//                     strike: pos!(100.0),
//                     option_type: OptionStyle::Put,
//                 }
//             ]
//         );
//     }
// }
//
// #[cfg(test)]
// mod additional_tests {
//     use super::*;
//     use crate::error::GreeksError;
//     use crate::greeks::Greek;
//     use crate::{pos, Options};
//     use std::cell::RefCell;
//
//     // Enhanced mock to track adjustments
//     struct MockStrategyWithAdjustments {
//         delta: Decimal,
//         underlying_price: Positive,
//         individual_deltas: Vec<Decimal>,
//         underlying_adjustments: RefCell<Vec<(Positive, Side)>>,
//         option_adjustments: RefCell<Vec<(Positive, Positive, OptionStyle, Side)>>,
//     }
//
//     impl Greeks for MockStrategyWithAdjustments {
//         fn get_options(&self) -> Result<Vec<&Options>, GreeksError> {
//             Ok(vec![])
//         }
//
//         fn greeks(&self) -> Result<Greek, GreeksError> {
//             Ok(Greek {
//                 delta: self.delta,
//                 gamma: Decimal::ZERO,
//                 theta: Decimal::ZERO,
//                 vega: Decimal::ZERO,
//                 rho: Decimal::ZERO,
//                 rho_d: Decimal::ZERO,
//                 alpha: Decimal::ZERO,
//             })
//         }
//     }
//
//     impl Positionable for MockStrategyWithAdjustments {}
//
//     impl DeltaNeutrality for MockStrategyWithAdjustments {
//         fn calculate_net_delta(&self) -> DeltaInfo {
//             DeltaInfo {
//                 net_delta: self.delta,
//                 individual_deltas: self.individual_deltas.clone(),
//                 is_neutral: self.delta.abs() <= dec!(0.01),
//                 neutrality_threshold: dec!(0.01),
//                 underlying_price: self.underlying_price,
//             }
//         }
//
//         fn get_atm_strike(&self) -> Positive {
//             self.underlying_price
//         }
//
//         fn adjust_underlying_position(
//             &mut self,
//             quantity: Positive,
//             side: Side,
//         ) -> Result<(), Box<dyn Error>> {
//             self.underlying_adjustments
//                 .borrow_mut()
//                 .push((quantity, side));
//             Ok(())
//         }
//
//         fn adjust_option_position(
//             &mut self,
//             quantity: Positive,
//             strike: &Positive,
//             option_type: &OptionStyle,
//             side: &Side,
//         ) -> Result<(), Box<dyn Error>> {
//             self.option_adjustments.borrow_mut().push((
//                 quantity,
//                 *strike,
//                 option_type.clone(),
//                 side.clone(),
//             ));
//             Ok(())
//         }
//     }
//
//     impl MockStrategyWithAdjustments {
//         fn new(delta: Decimal, price: Positive) -> Self {
//             Self {
//                 delta,
//                 underlying_price: price,
//                 individual_deltas: vec![delta],
//                 underlying_adjustments: RefCell::new(vec![]),
//                 option_adjustments: RefCell::new(vec![]),
//             }
//         }
//     }
//
//     #[test]
//     fn test_get_atm_strike() {
//         let strategy = MockStrategyWithAdjustments::new(dec!(0.5), pos!(100.0));
//         assert_eq!(strategy.get_atm_strike(), pos!(100.0));
//     }
//
//     #[test]
//     fn test_no_adjustment_needed() {
//         let strategy = MockStrategyWithAdjustments::new(dec!(0.0001), pos!(100.0));
//         let adjustments = strategy.delta_adjustments().unwrap();
//         assert_eq!(adjustments, vec![DeltaAdjustment::NoAdjustmentNeeded]);
//     }
//
//     #[test]
//     fn test_apply_delta_adjustments_neutral() -> Result<(), Box<dyn Error>> {
//         let mut strategy = MockStrategyWithAdjustments::new(dec!(0.0001), pos!(100.0));
//         strategy.apply_delta_adjustments(None, None)?;
//
//         assert!(strategy.underlying_adjustments.borrow().is_empty());
//         assert!(strategy.option_adjustments.borrow().is_empty());
//         Ok(())
//     }
//
//     #[test]
//     fn test_apply_delta_adjustments_with_filters() -> Result<(), Box<dyn Error>> {
//         let mut strategy = MockStrategyWithAdjustments::new(dec!(0.5), pos!(100.0));
//
//         // Test with Side filter
//         strategy.apply_delta_adjustments(Some(Side::Long), None)?;
//
//         // Only Long adjustments should be applied
//         let option_adjustments = strategy.option_adjustments.borrow();
//         assert!(option_adjustments
//             .iter()
//             .all(|(_, _, _, side)| matches!(side, Side::Long)));
//
//         Ok(())
//     }
//
//     #[test]
//     fn test_adjust_underlying_position() -> Result<(), Box<dyn Error>> {
//         let mut strategy = MockStrategyWithAdjustments::new(dec!(0.5), pos!(100.0));
//
//         strategy.adjust_underlying_position(pos!(1.0), Side::Long)?;
//
//         let adjustments = strategy.underlying_adjustments.borrow();
//         assert_eq!(adjustments.len(), 1);
//         assert_eq!(adjustments[0], (pos!(1.0), Side::Long));
//
//         Ok(())
//     }
//
//     #[test]
//     fn test_adjust_option_position() -> Result<(), Box<dyn Error>> {
//         let mut strategy = MockStrategyWithAdjustments::new(dec!(0.5), pos!(100.0));
//
//         strategy.adjust_option_position(
//             pos!(1.0),
//             &pos!(100.0),
//             &OptionStyle::Call,
//             &Side::Long,
//         )?;
//
//         let adjustments = strategy.option_adjustments.borrow();
//         assert_eq!(adjustments.len(), 1);
//         assert_eq!(
//             adjustments[0],
//             (pos!(1.0), pos!(100.0), OptionStyle::Call, Side::Long)
//         );
//
//         Ok(())
//     }
//
//     #[test]
//     fn test_apply_delta_adjustments_with_option_style_filter() -> Result<(), Box<dyn Error>> {
//         let mut strategy = MockStrategyWithAdjustments::new(dec!(0.5), pos!(100.0));
//
//         // Test with OptionStyle filter
//         strategy.apply_delta_adjustments(None, Some(OptionStyle::Call))?;
//
//         // Only Call options should be adjusted
//         let option_adjustments = strategy.option_adjustments.borrow();
//         assert!(option_adjustments
//             .iter()
//             .all(|(_, _, style, _)| matches!(style, OptionStyle::Call)));
//
//         Ok(())
//     }
// }
//
// #[cfg(test)]
// mod delta_adjustment_tests {
//     use super::*;
//     use crate::error::GreeksError;
//     use crate::greeks::Greek;
//     use crate::{pos, Options};
//     use std::cell::RefCell;
//
//     // Enhanced mock strategy for testing specific adjustment scenarios
//     struct TestMockStrategy {
//         delta: Decimal,
//         underlying_price: Positive,
//         underlying_adjustments: RefCell<Vec<(Positive, Side)>>,
//         option_adjustments: RefCell<Vec<(Positive, Positive, OptionStyle, Side)>>,
//     }
//
//     impl Greeks for TestMockStrategy {
//         fn greeks(&self) -> Result<Greek, GreeksError> {
//             Ok(Greek {
//                 delta: self.delta,
//                 gamma: Decimal::ZERO,
//                 theta: Decimal::ZERO,
//                 vega: Decimal::ZERO,
//                 rho: Decimal::ZERO,
//                 rho_d: Decimal::ZERO,
//                 alpha: Decimal::ZERO,
//             })
//         }
//
//         fn get_options(&self) -> Result<Vec<&Options>, GreeksError> {
//             Ok(vec![])
//         }
//     }
//
//     impl Positionable for TestMockStrategy {}
//
//     impl DeltaNeutrality for TestMockStrategy {
//         fn calculate_net_delta(&self) -> DeltaInfo {
//             DeltaInfo {
//                 net_delta: self.delta,
//                 individual_deltas: vec![self.delta],
//                 is_neutral: self.delta.abs() <= DELTA_THRESHOLD,
//                 neutrality_threshold: DELTA_THRESHOLD,
//                 underlying_price: self.underlying_price,
//             }
//         }
//
//         fn get_atm_strike(&self) -> Positive {
//             self.underlying_price
//         }
//
//         fn adjust_underlying_position(
//             &mut self,
//             quantity: Positive,
//             side: Side,
//         ) -> Result<(), Box<dyn Error>> {
//             self.underlying_adjustments
//                 .borrow_mut()
//                 .push((quantity, side));
//             Ok(())
//         }
//
//         fn adjust_option_position(
//             &mut self,
//             quantity: Positive,
//             strike: &Positive,
//             option_type: &OptionStyle,
//             side: &Side,
//         ) -> Result<(), Box<dyn Error>> {
//             self.option_adjustments.borrow_mut().push((
//                 quantity,
//                 *strike,
//                 option_type.clone(),
//                 side.clone(),
//             ));
//             Ok(())
//         }
//     }
//
//     impl TestMockStrategy {
//         fn new(delta: Decimal, price: Positive) -> Self {
//             Self {
//                 delta,
//                 underlying_price: price,
//                 underlying_adjustments: RefCell::new(vec![]),
//                 option_adjustments: RefCell::new(vec![]),
//             }
//         }
//     }
//
//     #[test]
//     fn test_apply_buy_underlying_adjustment() -> Result<(), Box<dyn Error>> {
//         let mut strategy = TestMockStrategy::new(dec!(-0.5), pos!(100.0));
//
//         // Test BuyUnderlying with no side filter
//         strategy.apply_delta_adjustments(None, None)?;
//
//         let adjustments = strategy.underlying_adjustments.borrow();
//         assert!(adjustments
//             .iter()
//             .any(|(_, side)| matches!(side, Side::Long)));
//         assert!(adjustments.iter().any(|(qty, _)| *qty == pos!(0.5)));
//
//         Ok(())
//     }
//
//     #[test]
//     fn test_apply_buy_options_adjustment() -> Result<(), Box<dyn Error>> {
//         let mut strategy = TestMockStrategy::new(dec!(-0.5), pos!(100.0));
//
//         // Test BuyOptions with call option
//         strategy.apply_delta_adjustments(Some(Side::Long), Some(OptionStyle::Call))?;
//
//         let adjustments = strategy.option_adjustments.borrow();
//         let call_adjustments: Vec<_> = adjustments
//             .iter()
//             .filter(|(_, _, style, side)| {
//                 matches!(style, OptionStyle::Call) && matches!(side, Side::Long)
//             })
//             .collect();
//
//         assert!(!call_adjustments.is_empty());
//         assert_eq!(call_adjustments[0].0, pos!(1.0)); // Quantity
//         assert_eq!(call_adjustments[0].1, pos!(100.0)); // Strike
//
//         Ok(())
//     }
//
//     #[test]
//     fn test_apply_sell_options_adjustment() -> Result<(), Box<dyn Error>> {
//         let mut strategy = TestMockStrategy::new(dec!(0.5), pos!(100.0));
//
//         // Test SellOptions with call option
//         strategy.apply_delta_adjustments(Some(Side::Short), Some(OptionStyle::Call))?;
//
//         let adjustments = strategy.option_adjustments.borrow();
//         let call_adjustments: Vec<_> = adjustments
//             .iter()
//             .filter(|(_, _, style, side)| {
//                 matches!(style, OptionStyle::Call) && matches!(side, Side::Short)
//             })
//             .collect();
//
//         assert!(!call_adjustments.is_empty());
//         assert_eq!(call_adjustments[0].0, pos!(1.0)); // Quantity
//         assert_eq!(call_adjustments[0].1, pos!(100.0)); // Strike
//
//         Ok(())
//     }
//
//     #[test]
//     fn test_no_adjustment_needed_early_return() -> Result<(), Box<dyn Error>> {
//         let mut strategy = TestMockStrategy::new(dec!(0.00005), pos!(100.0));
//
//         // Execute adjustment when strategy is already neutral
//         strategy.apply_delta_adjustments(None, None)?;
//
//         // Verify no adjustments were made
//         assert!(strategy.underlying_adjustments.borrow().is_empty());
//         assert!(strategy.option_adjustments.borrow().is_empty());
//
//         Ok(())
//     }
//
//     #[test]
//     fn test_adjust_underlying_position_direct() -> Result<(), Box<dyn Error>> {
//         let mut strategy = TestMockStrategy::new(dec!(0.5), pos!(100.0));
//
//         // Test direct adjustment of underlying position
//         strategy.adjust_underlying_position(pos!(1.0), Side::Long)?;
//
//         {
//             let adjustments = strategy.underlying_adjustments.borrow();
//             assert_eq!(adjustments.len(), 1);
//             assert_eq!(adjustments[0], (pos!(1.0), Side::Long));
//         } // préstamo inmutable termina aquí
//
//         // Test with short side
//         strategy.adjust_underlying_position(pos!(2.0), Side::Short)?;
//
//         {
//             let adjustments = strategy.underlying_adjustments.borrow();
//             assert_eq!(adjustments.len(), 2);
//             assert_eq!(adjustments[1], (pos!(2.0), Side::Short));
//         }
//
//         Ok(())
//     }
//
//     #[test]
//     fn test_adjust_option_position_direct() -> Result<(), Box<dyn Error>> {
//         let mut strategy = TestMockStrategy::new(dec!(0.5), pos!(100.0));
//
//         // Test calls
//         strategy.adjust_option_position(
//             pos!(1.0),
//             &pos!(100.0),
//             &OptionStyle::Call,
//             &Side::Long,
//         )?;
//
//         {
//             let adjustments = strategy.option_adjustments.borrow();
//             assert_eq!(adjustments.len(), 1);
//             assert_eq!(
//                 adjustments[0],
//                 (pos!(1.0), pos!(100.0), OptionStyle::Call, Side::Long)
//             );
//         } // préstamo inmutable termina aquí
//
//         // Test puts
//         strategy.adjust_option_position(
//             pos!(2.0),
//             &pos!(110.0),
//             &OptionStyle::Put,
//             &Side::Short,
//         )?;
//
//         {
//             let adjustments = strategy.option_adjustments.borrow();
//             assert_eq!(adjustments.len(), 2);
//             assert_eq!(
//                 adjustments[1],
//                 (pos!(2.0), pos!(110.0), OptionStyle::Put, Side::Short)
//             );
//         }
//
//         Ok(())
//     }
// }
