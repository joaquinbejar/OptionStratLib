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

/// Represents the possible adjustments needed to achieve delta neutrality
#[allow(dead_code)]
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
#[allow(dead_code)]
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

/// Trait for evaluating and managing delta neutrality in options strategies
#[allow(dead_code)]
pub trait DeltaNeutrality : Greeks{
    /// Calculates the net delta of the strategy and provides detailed information
    ///
    /// # Returns
    /// A DeltaInfo struct containing the net delta, individual deltas, and neutrality status
    fn calculate_net_delta(&self) -> DeltaInfo;

    /// Checks if the strategy is delta neutral within the specified threshold
    ///
    /// # Arguments
    /// * `threshold` - The maximum allowed deviation from perfect neutrality
    ///
    /// # Returns
    /// true if the strategy is neutral within the threshold, false otherwise
    fn is_delta_neutral(&self, threshold: f64) -> bool {
        self.calculate_net_delta().net_delta.abs() <= threshold
    }

    /// Suggests adjustments to achieve delta neutrality
    ///
    /// # Arguments
    /// * `threshold` - The maximum allowed deviation from perfect neutrality
    ///
    /// # Returns
    /// A vector of DeltaAdjustment suggestions to achieve neutrality
    fn suggest_delta_adjustments(&self, threshold: f64) -> Vec<DeltaAdjustment> {
        let delta_info = self.calculate_net_delta();

        if delta_info.is_neutral {
            return vec![DeltaAdjustment::NoAdjustmentNeeded];
        }

        let net_delta = delta_info.net_delta;

        // For positive delta, suggest delta-reducing adjustments
        if net_delta > threshold {
            self.generate_delta_reducing_adjustments(net_delta)
        }
        // For negative delta, suggest delta-increasing adjustments
        else if net_delta < -threshold {
            self.generate_delta_increasing_adjustments(net_delta)
        } else {
            vec![DeltaAdjustment::NoAdjustmentNeeded]
        }
    }

    /// Gets the ATM strike price closest to current underlying price
    fn get_atm_strike(&self) -> PositiveF64;

    /// Generates adjustments to reduce positive delta
    fn generate_delta_reducing_adjustments(&self, net_delta: f64) -> Vec<DeltaAdjustment> {
        vec![
            DeltaAdjustment::SellUnderlying(pos!(net_delta.abs())),
            DeltaAdjustment::BuyOptions {
                quantity: pos!(net_delta.abs() / 2.0),
                strike: self.get_atm_strike(),
                option_type: OptionStyle::Put,
            },
        ]
    }

    /// Generates adjustments to increase negative delta
    fn generate_delta_increasing_adjustments(&self, net_delta: f64) -> Vec<DeltaAdjustment> {
        vec![
            DeltaAdjustment::BuyUnderlying(pos!(net_delta.abs())),
            DeltaAdjustment::BuyOptions {
                quantity: pos!(net_delta.abs() / 2.0),
                strike: self.get_atm_strike(),
                option_type: OptionStyle::Call,
            },
        ]
    }
}