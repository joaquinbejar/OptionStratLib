use crate::Positive;
use crate::greeks::GreeksSnapshot;
use crate::model::Position;
use chrono::{DateTime, NaiveDateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Represents detailed statistical information aggregated from a collection of individual trades.
///
/// This struct provides insights into trading performance by summarizing various aspects
/// such as the number of trades, win/loss ratios, average returns, trade durations,
/// and breakdowns by trade type (e.g., long, short, call, put).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TradeStatistics {
    /// The total number of trades included in the statistics.
    pub number_of_trades: usize,
    /// The number of trades that resulted in a profit.
    pub winners: usize,
    /// The number of trades that resulted in a loss.
    pub losers: usize,
    /// The number of trades that resulted in neither profit nor loss (return is zero).
    pub break_even: usize,
    /// The average return per trade, expressed as a decimal value.
    pub average_trade_return: Decimal,
    /// The median return across all trades. Half the trades had a return less than or equal to this value,
    /// and half had a return greater than or equal to this value.
    pub median_trade_return: Decimal,
    /// The largest profit recorded among all winning trades. `None` if there were no winning trades.
    pub largest_win: Option<Decimal>,
    /// The largest loss recorded among all losing trades (represented as a negative value). `None` if there were no losing trades.
    pub largest_loss: Option<Decimal>,
    /// The average duration for which trades were held open. Represented as a non-negative `Decimal`.
    pub average_holding_period: Positive,
    /// The median duration for which trades were held open. Represented as a non-negative `Decimal`.
    pub median_holding_period: Positive,
    /// The minimum duration any trade was held open. Represented as a non-negative `Decimal`.
    pub min_holding_period: Positive,
    /// The maximum duration any trade was held open. Represented as a non-negative `Decimal`.
    pub max_holding_period: Positive,
    /// The number of trades that were long positions.
    pub long_trades: usize,
    /// The number of trades that were short positions.
    pub short_trades: usize,
    /// The number of trades involving call options.
    pub call_trades: usize,
    /// The number of trades involving put options.
    pub put_trades: usize,
    /// The number of trades involving spreads (multi-leg option strategies).
    pub spread_trades: usize,
}

/// Represents a comprehensive analysis of drawdown periods within a financial time series.
///
/// Drawdown refers to the decline from a historical peak in some variable, typically the
/// cumulative profit or total value of a financial portfolio or instrument. This struct
/// aggregates various metrics related to drawdowns observed over the analyzed period.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DrawdownAnalysis {
    /// The magnitude of the largest single drawdown observed during the analysis period.
    /// This is typically expressed as a positive decimal representing the percentage loss
    /// from the peak to the subsequent trough (e.g., 0.1 for a 10% loss).
    pub max_drawdown: Decimal,
    /// The duration from the peak to the trough of the maximum drawdown event.
    /// Represented as a positive value (e.g., number of days).
    pub max_drawdown_duration: Positive,
    /// The time it took for the value to recover from the trough of the maximum drawdown
    /// back to the level of its preceding peak.
    /// This is `None` if recovery did not occur by the end of the analysis period.
    pub recovery_duration: Option<Positive>,
    /// The duration from the beginning of the analysis period until the peak
    /// that preceded the maximum drawdown event.
    pub time_to_max_drawdown: Positive,
    /// A vector containing details of every individual drawdown event identified
    /// during the analysis period. See [`DrawdownEvent`] for details.
    pub drawdowns: Vec<DrawdownEvent>,
    /// The average magnitude of all drawdown events recorded in `drawdowns`.
    pub avg_drawdown: Decimal,
    /// The average time it took to recover from the trough back to the preceding peak
    /// across all drawdown events that achieved recovery within the analysis period.
    /// This is `None` if no drawdowns recovered.
    pub avg_recovery_time: Option<Positive>,
    /// The total duration (e.g., number of days) the analyzed value spent below
    /// a previously established peak ("underwater").
    pub total_underwater_days: Positive,
    /// The percentage of the total analysis duration that the value spent underwater.
    /// Calculated as `total_underwater_days` divided by the total duration of the analysis period.
    pub underwater_percentage: Decimal,
}

/// Represents a single drawdown event in a time series, typically of asset prices or portfolio values.
///
/// A drawdown is a peak-to-trough decline during a specific period for an investment,
/// trading account, or fund. It is usually quoted as the percentage between the peak and
/// the subsequent trough.
///
/// This struct captures the key dates, the magnitude (severity) of the drawdown,
/// and its duration, including the time it takes to recover back to the initial peak value.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrawdownEvent {
    /// The date and time when the drawdown period started (the peak before the decline).
    pub start_date: NaiveDateTime,
    /// The date and time when the value reached its lowest point (the trough) during the drawdown.
    pub bottom_date: NaiveDateTime,
    /// The date and time when the value recovered back to the level of `start_date`.
    /// This is `None` if the value has not yet recovered by the end of the analyzed period.
    pub recovery_date: Option<NaiveDateTime>,
    /// The magnitude of the drawdown, typically expressed as a positive decimal representing
    /// the percentage loss from the peak (`start_date`) to the trough (`bottom_date`).
    /// For example, a value of `0.1` represents a 10% drawdown.
    pub magnitude: Decimal,
    /// The duration of the drawdown from the `start_date` to the `bottom_date`,
    /// represented as a positive value (e.g., number of days or other time units).
    pub duration: Positive,
    /// The duration it took for the value to recover from the `bottom_date` back to
    /// the `start_date` level. This is `None` if the recovery has not occurred
    /// (i.e., `recovery_date` is `None`). Represented as a positive value.
    pub recovery_duration: Option<Positive>,
}

/// Represents the allocation and utilization of capital within a trading strategy.
///
/// This struct tracks various metrics related to how capital is deployed and managed,
/// including overall capital usage, specific metrics for options trading (margin and premium),
/// and risk allocation measures like position sizing.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CapitalUtilization {
    /// The maximum amount of capital utilized at any point during the trading period.
    pub max_capital_used: Decimal,
    /// The average amount of capital utilized throughout the trading period.
    pub avg_capital_used: Decimal,
    /// A measure of return generated relative to the average capital used (Return / Average Capital Used).
    pub capital_efficiency: Decimal, // return / average capital used
    /// The total margin required and used for options positions throughout the period.
    pub total_margin_used: Decimal,
    /// The peak margin utilized at any single point in time.
    pub max_margin_used: Decimal,
    /// The average margin utilized throughout the trading period.
    pub avg_margin_used: Decimal,
    /// The total amount of premium paid for purchasing options.
    pub total_premium_paid: Decimal,
    /// The total amount of premium received for selling options.
    pub total_premium_received: Decimal,
    /// The net difference between premium received and premium paid (Received - Paid).
    pub net_premium: Decimal,
    /// The largest single position size taken, expressed as a percentage of total available capital.
    pub max_position_size: Decimal, // as % of total capital
    /// The average size of positions taken, expressed as a percentage of total available capital.
    pub avg_position_size: Decimal,
}

/// Represents time-series data for tracking various portfolio metrics over time.
///
/// This struct holds vectors of data points, each corresponding to a specific timestamp.
/// It's used to store the evolution of the equity curve, drawdown, margin usage,
/// position count, and potentially option greeks exposures and custom metrics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TimeSeriesData {
    /// Timestamps corresponding to each data point in the series.
    pub timestamps: Vec<DateTime<Utc>>,
    /// The value of the portfolio equity at each timestamp.
    pub equity_curve: Vec<Decimal>,
    /// The drawdown percentage or value from the peak equity at each timestamp.
    pub drawdown_curve: Vec<Decimal>,
    /// The amount or percentage of margin used at each timestamp.
    pub margin_usage: Vec<Decimal>,
    /// The number of open positions at each timestamp.
    pub position_count: Vec<usize>,
    /// Optional time series of the portfolio's delta exposure.
    pub delta_exposure: Option<Vec<Decimal>>,
    /// Optional time series of the portfolio's gamma exposure.
    pub gamma_exposure: Option<Vec<Decimal>>,
    /// Optional time series of the portfolio's theta exposure.
    pub theta_exposure: Option<Vec<Decimal>>,
    /// Optional time series of the portfolio's vega exposure.
    pub vega_exposure: Option<Vec<Decimal>>,
    /// A map holding custom time series metrics, identified by a string key.
    /// Each key maps to a vector of decimal values corresponding to the timestamps.
    pub custom_series: HashMap<String, Vec<Decimal>>,
}

/// Represents a detailed record of an individual trade, capturing all relevant information
/// from entry to exit.
///
/// This struct is used to log and analyze the performance and characteristics of specific trades,
/// particularly in the context of options trading. It includes timestamps, financial details,
/// associated strategy information, and reasons for exit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeRecord {
    /// A unique identifier (`Uuid`) for this specific trade record.
    pub id: Uuid,
    /// The exact date and time (UTC) when the trade position was initiated or entered.
    pub entry_date: DateTime<Utc>,
    /// The exact date and time (UTC) when the trade position was closed or exited, if it has been closed.
    /// `None` if the position is still open.
    pub exit_date: Option<DateTime<Utc>>,
    /// The duration for which the trade was held, typically measured in days or fractions thereof.
    /// This value is guaranteed to be positive if present. `None` if the position is still open.
    pub duration: Option<Positive>,
    /// An optional identifier (`Uuid`) linking this trade to a specific trading strategy.
    pub strategy: Option<Uuid>,
    /// Contains detailed information about the option position itself, including the underlying option,
    /// premium, fees, and entry date. See [`Position`] for more details.
    pub position: Position,
    /// The price per unit/contract at which the position was closed. `None` if the position is still open.
    pub exit_price: Option<Decimal>,
    /// Any additional cost incurred during the trade execution beyond the explicit fees,
    /// such as price slippage. Usually represented as a cost per unit/contract or total cost.
    pub slippage: Option<Decimal>,
    /// The realized profit or loss from this trade after accounting for entry/exit prices and fees.
    /// `None` if the position is still open or P/L hasn't been calculated yet.
    pub profit_loss: Option<Decimal>,
    /// The percentage return generated by the trade, typically calculated based on the initial investment or margin.
    /// `None` if the position is still open or the return hasn't been calculated.
    pub return_percentage: Option<Decimal>,
    /// The amount of capital or margin required to open and maintain this position.
    pub margin_required: Option<Decimal>,
    /// The specific reason why the position was closed (e.g., target reached, stop loss hit, expiration).
    /// See [`ExitReason`] for possible values. `None` if the position is still open.
    pub exit_reason: Option<ExitReason>,
    /// Optional field for any textual notes, observations, or comments related to this specific trade.
    pub notes: Option<String>,
    /// A snapshot of the calculated option Greeks (Delta, Gamma, Theta, Vega, Rho) at the moment of trade entry.
    /// See [`GreeksSnapshot`] for details. `None` if Greeks were not calculated or recorded at entry.
    pub entry_greeks: Option<GreeksSnapshot>,
    /// A snapshot of the calculated option Greeks at the moment of trade exit.
    /// See [`GreeksSnapshot`] for details. `None` if the position is still open or Greeks were not calculated/recorded at exit.
    pub exit_greeks: Option<GreeksSnapshot>,
}

/// Represents the reason why a trading position was closed or exited.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ExitReason {
    /// The position was closed because the profit target price was reached.
    TargetReached,
    /// The position was closed because the stop-loss price was triggered.
    StopLoss,
    /// The position was closed due to the expiration of the underlying contract or option.
    Expiration,
    /// The position was rolled over to a new contract period.
    RollOver,
    /// The position was closed manually by the trader or user.
    ManualClose,
    /// The position was liquidated due to a margin call (insufficient funds).
    MarginCall,
    /// Represents any other reason for closing the position, with a descriptive string.
    Other(String),
}

/// Represents volatility surface information captured during a backtest.
///
/// This struct stores data related to the implied volatility metrics
/// observed at the time trades occurred within the simulation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VolatilityData {
    /// Flag indicating whether implied volatility data was used in the backtest calculations for this trade.
    pub implied_volatility_used: bool,
    /// The average implied volatility observed around the time the trade was executed, if available.
    pub avg_iv_traded: Option<Decimal>,
    /// The percentile rank of the implied volatility at the time of the trade compared to its historical distribution, if available.
    pub iv_percentile_traded: Option<Decimal>,
    /// The rank of the implied volatility at the time of the trade compared to its historical range (e.g., IV Rank), if available.
    pub iv_rank_traded: Option<Decimal>,
}
