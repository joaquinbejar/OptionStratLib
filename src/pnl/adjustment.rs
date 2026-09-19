//! Delta adjustment actions.
//!
//! [`DeltaAdjustment`] is the neutral description of a hedge: buy or sell
//! options or underlying, or do nothing. It carries only core types, so the
//! analytics layer (P&L) can consume it through
//! [`crate::pnl::PnLCalculator::adjustments_pnl`] without depending on the
//! strategies that produce it. The strategies module re-exports it under its
//! historical `strategies::delta_neutral::DeltaAdjustment` path.

use crate::model::types::{OptionStyle, Side};
use positive::Positive;
use pretty_simple_display::{DebugPretty, DisplaySimple};
use serde::{Deserialize, Serialize};
use std::fmt;
use utoipa::ToSchema;

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

/// Two adjustments that must be executed together with the same size, so
/// that neither leg is left unhedged if only one of them were applied.
#[derive(DebugPretty, DisplaySimple, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct DeltaAdjustmentSameSize {
    /// The first adjustment of the pair.
    pub first: Box<DeltaAdjustment>,
    /// The second adjustment of the pair, sized like the first.
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
