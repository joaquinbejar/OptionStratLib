use crate::Positive;
use crate::model::Position;
use crate::strategies::base::StrategyType;
use serde::{Deserialize, Serialize};

pub(super) const LONG_PUT_DESCRIPTION: &str = "A Long Put is an options strategy where the trader purchases a put option, gaining the right (but not the obligation) to sell the underlying asset at the strike price until expiration. \
    This strategy requires an initial investment (the premium paid) and provides downside protection or profit potential if the underlying asset's price decreases. \
    The breakeven point is the strike price minus the premium paid. Long puts are commonly used as insurance against price declines or to express a bearish outlook.";

/// # PoorMansCoveredCall
///
/// Represents a Poor Man's Covered Call options trading strategy. This strategy is a cost-effective
/// alternative to the traditional covered call, using a deep in-the-money long-term call option
/// instead of owning the underlying stock, while selling shorter-term out-of-the-money call options.
///
/// A Poor Man's Covered Call (also known as a PMCC or Diagonal Debit Call Spread) requires less capital
/// than a standard covered call while still providing similar profit potential and risk profile.
///
/// ## Fields
/// * `name`: A descriptive name for the specific strategy instance.
/// * `kind`: The type of strategy, which is `StrategyType::PoorMansCoveredCall`.
/// * `description`: A detailed description of this specific strategy instance.
/// * `break_even_points`: The price points at which the strategy breaks even (neither profit nor loss).
/// * `long_call`: The long call position (typically a LEAP - Long-Term Equity Anticipation Security).
/// * `short_call`: The short call position (shorter-term, out-of-the-money call).
///
/// ## Risk and Reward
/// The maximum risk in this strategy is limited to the net debit paid (cost of the long call minus
/// the premium received for the short call).
///
/// The maximum profit is capped and occurs when the underlying price at expiration of the short call
/// equals or exceeds the strike price of the short call.
///
/// ## Break-Even Point
/// The break-even point at expiration of the short call is approximately the strike price of the long call
/// plus the net debit paid for the spread.
///
/// ## Strategy Usage
/// This strategy is typically used when:
/// - The trader is moderately to strongly bullish on the underlying asset
/// - The trader wants to generate income while still participating in potential upside movement
/// - The trader wants to implement a covered call strategy with less capital investment
/// - Implied volatility is relatively high for near-term options
///
/// ## Management Considerations
/// - The strategy often involves rolling the short call forward to continue generating income
/// - The long call should have sufficient time value to avoid assignment complications
/// - Ideally implemented when the underlying asset has a strong positive outlook over the long term
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LongPut {
    /// Name identifier for this specific strategy instance
    pub name: String,
    /// Identifies this as a PoorMansCoveredCall strategy type
    pub kind: StrategyType,
    /// Detailed description of this strategy instance
    pub description: String,
    /// Price points where the strategy neither makes nor loses money
    pub break_even_points: Vec<Positive>,
    /// The long-term in-the-money call option (usually a LEAP)
    pub(super) long_put: Position,
}
