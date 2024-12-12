/******************************************************************************
    Author: Joaquín Béjar García
    Email: jb@taunais.com 
    Date: 10/12/24
 ******************************************************************************/

//! # Delta Neutral Strategies Module
use crate::pos;
use std::fmt;
use crate::greeks::equations::Greeks;
use crate::model::types::{OptionStyle, PositiveF64};

pub const DELTA_THRESHOLD: f64 = 0.005;

/// Represents the possible adjustments needed to achieve delta neutrality
#[derive(Debug, PartialEq)]
pub enum DeltaAdjustment {
    /// Buy options with specified parameters
    BuyOptions {
        /// Number of contracts to buy
        quantity: PositiveF64,
        /// Strike price of the options
        strike: PositiveF64,
        /// Type of option (Call or Put)
        option_type: OptionStyle,
    },
    /// Sell options with specified parameters
    SellOptions {
        /// Number of contracts to sell
        quantity: PositiveF64,
        /// Strike price of the options
        strike: PositiveF64,
        /// Type of option (Call or Put)
        option_type: OptionStyle,
    },
    /// Buy underlying asset with specified quantity
    BuyUnderlying(PositiveF64),
    /// Sell underlying asset with specified quantity
    SellUnderlying(PositiveF64),
    /// No adjustment needed, strategy is already neutral within threshold
    NoAdjustmentNeeded,
}

/// Contains detailed information about the delta status of a strategy
#[derive(Debug)]
pub struct DeltaInfo  {
    /// Net delta of the entire strategy
    pub net_delta: f64,
    /// Individual deltas of each component
    pub individual_deltas: Vec<f64>,
    /// Whether the strategy is considered delta neutral
    pub is_neutral: bool,
    /// The threshold used to determine neutrality
    pub neutrality_threshold: f64,
    /// The current underlying price
    pub underlying_price: PositiveF64,
}

impl fmt::Display for DeltaInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Delta Analysis:")?;
        writeln!(f, "  Net Delta: {:.4}", self.net_delta)?;
        writeln!(f, "  Is Neutral: {}", self.is_neutral)?;
        writeln!(f, "  Neutrality Threshold: {:.4}", self.neutrality_threshold)?;
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
/// * `generate_delta_increasing_adjustments`: Produces adjustments required to increase a negative delta.
/// * `get_atm_strike`: Retrieves the ATM (At-The-Money) strike price closest to the current underlying asset price.
pub trait DeltaNeutrality : Greeks {
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
    fn calculate_net_delta(&self) -> DeltaInfo;

    /// Checks if the strategy is delta neutral within the specified threshold.
    ///
    /// # Arguments
    /// * `threshold` - A `f64` value representing the maximum allowed deviation from ideal delta neutrality.
    ///
    /// # Returns
    /// A boolean (`true` or `false`):
    /// * `true` if the absolute value of the net delta is within the threshold.
    /// * `false` otherwise.
    fn is_delta_neutral(&self) -> bool {
        self.calculate_net_delta().net_delta.abs() <= DELTA_THRESHOLD
    }
    
    fn get_atm_strike(&self) -> PositiveF64 {
        panic!("get_atm_strike Not implemented");
    }

    /// Suggests adjustments to achieve delta neutrality.
    ///
    /// # Arguments
    /// * `threshold` - A `f64` value defining the maximum allowable deviation for neutrality.
    ///
    /// # Returns
    /// * A `Vec<DeltaAdjustment>` containing potential adjustments (if needed) to bring the strategy closer to neutrality.
    /// * If the strategy is already neutral, a `DeltaAdjustment::NoAdjustmentNeeded` is suggested.
    ///
    /// The adjustments suggested may include buying or selling options or the underlying asset, depending on the net delta.
    fn suggest_delta_adjustments(&self) -> Vec<DeltaAdjustment> {
        let delta_info = self.calculate_net_delta();
        if delta_info.is_neutral {
            return vec![DeltaAdjustment::NoAdjustmentNeeded];
        }
        let net_delta = delta_info.net_delta;
        // For positive delta, suggest delta-reducing adjustments
        if net_delta > DELTA_THRESHOLD {
            self.generate_delta_reducing_adjustments()
        }
        // For negative delta, suggest delta-increasing adjustments
        else if net_delta < -DELTA_THRESHOLD {
            self.generate_delta_increasing_adjustments()
        } else {
            vec![DeltaAdjustment::NoAdjustmentNeeded]
        }
    }
    

    /// Generates adjustments to reduce a positive delta.
    ///
    /// # Arguments
    /// * `net_delta` - A `f64` representing the current positive net delta requiring adjustment.
    ///
    /// # Returns
    /// * A `Vec<DeltaAdjustment>` suggesting actions to reduce delta (e.g., selling the underlying asset or buying put options).
    ///
    /// Adjustments may include:
    /// * Selling the underlying asset.
    /// * Buying puts at ATM strikes.
    fn generate_delta_reducing_adjustments(&self) -> Vec<DeltaAdjustment> {
        let net_delta = self.calculate_net_delta().net_delta;
        vec![
            DeltaAdjustment::SellUnderlying(pos!(net_delta.abs())),
            DeltaAdjustment::BuyOptions {
                quantity: pos!(net_delta.abs() / 0.5),
                strike: self.get_atm_strike(),
                option_type: OptionStyle::Put,
            },
            DeltaAdjustment::SellOptions {
                quantity: pos!(net_delta.abs()/ 0.5),
                strike: self.get_atm_strike(),
                option_type: OptionStyle::Call,
            },
        ]
    }

    /// Generates adjustments to increase a negative delta.
    ///
    /// # Arguments
    /// * `net_delta` - A `f64` representing the current negative net delta requiring adjustment.
    ///
    /// # Returns
    /// * A `Vec<DeltaAdjustment>` suggesting actions to increase delta (e.g., buying the underlying asset or buying call options).
    ///
    /// Adjustments may include:
    /// * Buying the underlying asset.
    /// * Buying calls at ATM strikes.
    fn generate_delta_increasing_adjustments(&self) -> Vec<DeltaAdjustment> {
        let net_delta = self.calculate_net_delta().net_delta;
        vec![
            DeltaAdjustment::BuyUnderlying(pos!(net_delta.abs())),
            DeltaAdjustment::BuyOptions {
                quantity: pos!(net_delta.abs() / 0.5),
                strike: self.get_atm_strike(),
                option_type: OptionStyle::Call,
            },
            DeltaAdjustment::SellOptions {
                quantity: pos!(net_delta.abs() / 0.5),
                strike: self.get_atm_strike(),
                option_type: OptionStyle::Put,
            },
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::greeks::equations::Greek;

    // Mock struct to implement required traits for testing
    struct MockStrategy {
        delta: f64,
        underlying_price: PositiveF64,
        individual_deltas: Vec<f64>,
    }

    // Implement Greeks trait for MockStrategy
    impl Greeks for MockStrategy {
        fn greeks(&self) -> Greek {
            Greek {
                delta: self.delta,
                gamma: 0.0,
                theta: 0.0,
                vega: 0.0,
                rho: 0.0,
                rho_d: 0.0,
            }
        }
    }

    // Implement DeltaNeutrality trait for MockStrategy
    impl DeltaNeutrality for MockStrategy {
        fn calculate_net_delta(&self) -> DeltaInfo {
            DeltaInfo {
                net_delta: self.delta,
                individual_deltas: self.individual_deltas.clone(),
                is_neutral: self.delta.abs() <= 0.01,
                neutrality_threshold: 0.01,
                underlying_price: self.underlying_price,
            }
        }

        fn get_atm_strike(&self) -> PositiveF64 {
            pos!(100.0)
        }
        
    }

    // Helper function to create a mock strategy
    fn create_mock_strategy(delta: f64, price: f64) -> MockStrategy {
        MockStrategy {
            delta,
            underlying_price: pos!(price),
            individual_deltas: vec![delta],
        }
    }

    #[test]
    fn test_calculate_net_delta() {
        let strategy = create_mock_strategy(0.5, 100.0);
        let info = strategy.calculate_net_delta();

        assert_eq!(info.net_delta, 0.5);
        assert_eq!(info.individual_deltas, vec![0.5]);
        assert!(!info.is_neutral);
        assert_eq!(info.underlying_price, pos!(100.0));
    }

    #[test]
    fn test_is_delta_neutral() {
        let neutral_strategy = create_mock_strategy(0.005, 100.0);
        let non_neutral_strategy = create_mock_strategy(0.5, 100.0);

        assert!(neutral_strategy.is_delta_neutral());
        assert!(!non_neutral_strategy.is_delta_neutral());
    }

    #[test]
    fn test_suggest_delta_adjustments_neutral() {
        let strategy = create_mock_strategy(0.005, 100.0);
        let adjustments = strategy.suggest_delta_adjustments();

        assert_eq!(adjustments, vec![DeltaAdjustment::NoAdjustmentNeeded]);
    }

    #[test]
    fn test_suggest_delta_adjustments_positive() {
        let strategy = create_mock_strategy(0.5, 100.0);
        let adjustments = strategy.suggest_delta_adjustments();

        assert_eq!(
            adjustments,
            vec![
                DeltaAdjustment::SellUnderlying(pos!(0.5)),
                DeltaAdjustment::BuyOptions {
                    quantity: pos!(1.0),
                    strike: pos!(100.0),
                    option_type: OptionStyle::Put,
                },
                DeltaAdjustment::SellOptions {
                    quantity: pos!(1.0),
                    strike: pos!(100.0),
                    option_type: OptionStyle::Call,
                }
            ]
        );
    }

    #[test]
    fn test_suggest_delta_adjustments_negative() {
        let strategy = create_mock_strategy(-0.5, 100.0);
        let adjustments = strategy.suggest_delta_adjustments();

        assert_eq!(
            adjustments,
            vec![
                DeltaAdjustment::BuyUnderlying(pos!(0.5)),
                DeltaAdjustment::BuyOptions {
                    quantity: pos!(1.0),
                    strike: pos!(100.0),
                    option_type: OptionStyle::Call,
                },
                DeltaAdjustment::SellOptions {
                    quantity: pos!(1.0),
                    strike: pos!(100.0),
                    option_type: OptionStyle::Put,
                }
            ]
        );
    }

    #[test]
    fn test_delta_info_display() {
        let strategy = create_mock_strategy(0.5, 100.0);
        let info = strategy.calculate_net_delta();
        let display_string = format!("{}", info);

        assert!(display_string.contains("Net Delta: 0.5000"));
        assert!(display_string.contains("Is Neutral: false"));
        assert!(display_string.contains("Underlying Price: 100"));
        assert!(display_string.contains("Individual Deltas:"));
    }

    #[test]
    fn test_generate_delta_reducing_adjustments() {
        let strategy = create_mock_strategy(0.5, 100.0);
        let adjustments = strategy.generate_delta_reducing_adjustments();

        assert_eq!(
            adjustments,
            vec![
                DeltaAdjustment::SellUnderlying(pos!(0.5)),
                DeltaAdjustment::BuyOptions {
                    quantity: pos!(1.0),
                    strike: pos!(100.0),
                    option_type: OptionStyle::Put,
                },
                DeltaAdjustment::SellOptions {
                    quantity: pos!(1.0),
                    strike: pos!(100.0),
                    option_type: OptionStyle::Call,
                }
            ]
        );
    }

    #[test]
    fn test_generate_delta_increasing_adjustments() {
        let strategy = create_mock_strategy(-0.5, 100.0);
        let adjustments = strategy.generate_delta_increasing_adjustments();

        assert_eq!(
            adjustments,
            vec![
                DeltaAdjustment::BuyUnderlying(pos!(0.5)),
                DeltaAdjustment::BuyOptions {
                    quantity: pos!(1.0),
                    strike: pos!(100.0),
                    option_type: OptionStyle::Call,
                },
                DeltaAdjustment::SellOptions {
                    quantity: pos!(1.0),
                    strike: pos!(100.0),
                    option_type: OptionStyle::Put,
                }
            ]
        );
    }
    
}