/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 26/4/25
******************************************************************************/
use crate::backtesting::metrics::{
    AdvancedRiskMetrics, GeneralPerformanceMetrics, MarketConditionMetrics, OptionsSpecificMetrics,
};
use crate::backtesting::types::{
    CapitalUtilization, DrawdownAnalysis, TimeSeriesData, TradeRecord, TradeStatistics,
    VolatilityData,
};
use chrono::{DateTime, Utc};
use optionstratlib_analytics::pnl::PnL;
use optionstratlib_analytics::risk::RiskMetricsSimulation;
use optionstratlib_core::{impl_json_debug_pretty, impl_json_display};
use optionstratlib_simulation::simulation::ExitPolicy;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
#[cfg(feature = "schema")]
use utoipa::ToSchema;

/// Comprehensive container for all results generated during a backtest simulation.
///
/// This struct aggregates various performance metrics, time-series data, trade details,
/// risk analysis, and context related to a specific trading strategy simulation.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BacktestResult {
    /// Core performance metrics applicable to most trading strategies.
    /// Includes returns, risk measures (like volatility), risk-adjusted returns (like Sharpe),
    /// and win/loss statistics.
    pub general_performance: GeneralPerformanceMetrics,
    /// Metrics specifically relevant to options trading strategies.
    /// Includes metrics like return on margin, premium capture, and average Greek exposures.
    pub options_metrics: OptionsSpecificMetrics,
    /// Statistical analysis of the trades executed during the backtest.
    /// Contains counts, averages, and distributions related to trade outcomes.
    pub trade_statistics: TradeStatistics,
    /// Analysis of the portfolio's value declines (drawdowns) from peak equity.
    /// Includes metrics like maximum drawdown and longest drawdown period.
    pub drawdown_analysis: DrawdownAnalysis,
    /// Metrics detailing how capital was deployed and managed throughout the backtest.
    /// Includes average/max capital usage, margin metrics, and premium flows.
    pub capital_utilization: CapitalUtilization,
    /// Time-series data tracking the evolution of key portfolio metrics over the backtest period.
    /// Includes equity curve, drawdown curve, margin usage, position counts, and potentially Greek exposures.
    pub time_series: TimeSeriesData,
    /// A detailed record of each individual trade executed during the backtest simulation.
    pub trades: Vec<TradeRecord>,
    /// Optional metrics describing the market environment during the backtest period.
    /// Can include classifications like bull/bear market days or high/low volatility periods.
    pub market_conditions: Option<MarketConditionMetrics>,
    /// Optional data related to implied volatility observed during the backtest, often at trade execution times.
    /// Includes metrics like average IV traded or IV rank/percentile.
    pub volatility_data: Option<VolatilityData>,
    /// Optional advanced risk analytics beyond standard deviation.
    /// May include metrics like Value at Risk (VaR), Expected Shortfall (ES), or Ulcer Index.
    pub risk_metrics: Option<AdvancedRiskMetrics>,
    /// Optional results from Monte Carlo simulations performed on the strategy's outcomes.
    /// Provides probabilistic insights into future performance potential and risk.
    pub monte_carlo_simulation: Option<SimulationResult>,
    /// The name identifier of the trading strategy being backtested.
    pub strategy_name: String,
    /// The starting date and time of the backtest period.
    pub test_period_start: DateTime<Utc>,
    /// The ending date and time of the backtest period.
    pub test_period_end: DateTime<Utc>,
    /// The initial amount of capital allocated at the beginning of the backtest.
    pub initial_capital: Decimal,
    /// The final amount of capital remaining at the end of the backtest.
    pub final_capital: Decimal,
    /// A flexible map allowing storage of custom, strategy-specific metrics.
    /// Keys are metric names (strings), and values are the calculated metric values (Decimals).
    pub custom_metrics: HashMap<String, Decimal>,
}

/// `SimulationResult` represents the outcome of a financial or trading simulation,
/// capturing the details of the simulation's performance and metrics.
///
/// # Fields
///
/// * `simulation_count` - The total number of simulation runs performed.
/// * `risk_metrics` - Optional risk metrics derived from the simulations.
/// * `final_equity_percentiles` - A map containing the percentiles of the final equity
///   distribution. The keys represent the percentile (e.g., 5, 50, 95), and the values
///   are the corresponding equity values.
/// * `max_premium` - The maximum premium value observed during the simulation.
/// * `min_premium` - The minimum premium value observed during the simulation.
/// * `avg_premium` - The average premium value observed during the simulation.
/// * `hit_take_profit` - A boolean indicating if the take profit target was achieved,
///   defined as a 50% reduction in premium.
/// * `hit_stop_loss` - A boolean indicating if the stop loss condition was triggered,
///   defined as a 100% increase in premium.
/// * `expired` - A boolean indicating if the option expired without hitting the take
///   profit or stop loss conditions.
/// * `expiration_premium` - Optional final premium value at expiration. Only available
///   if the option expired.
/// * `pnl` - The final profit or loss (P&L) resulting from the simulation.
/// * `holding_period` - The number of simulation steps during which the asset was held.
/// * `exit_reason` - The reason or policy that triggered the exit from the position
///   (e.g., take profit, stop loss, expiration).
///
/// # Notes
///
/// `SimulationResult` is a serializable and cloneable structure, making it convenient
/// for storing, displaying, and transmitting simulation outcomes. It also provides
/// a user-friendly debug and display interface through derived traits.
#[derive(Clone, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "schema", derive(ToSchema))]
pub struct SimulationResult {
    /// Number of simulation runs
    pub simulation_count: usize,

    /// Risk metrics calculated from simulations
    pub risk_metrics: Option<RiskMetricsSimulation>,

    /// Percentiles of final equity distribution
    pub final_equity_percentiles: HashMap<u8, Decimal>,

    /// Maximum premium value during the simulation
    pub max_premium: Decimal,

    /// Minimum premium value during the simulation
    pub min_premium: Decimal,

    /// Average premium value during the simulation
    pub avg_premium: Decimal,

    /// Whether the exit policy's take-profit condition closed the run
    pub hit_take_profit: bool,

    /// Whether the exit policy's stop-loss condition closed the run
    pub hit_stop_loss: bool,

    /// Whether the option expired without hitting take profit or stop loss
    pub expired: bool,

    /// Final premium value at expiration (only if expired)
    pub expiration_premium: Option<Decimal>,

    /// Final P&L
    pub pnl: PnL,

    /// Holding period in steps
    pub holding_period: usize,

    /// Exit policy that triggered the exit
    pub exit_reason: ExitPolicy,
}

impl_json_debug_pretty!(SimulationResult);
impl_json_display!(SimulationResult);

/// Statistics aggregated from multiple simulation runs.
///
/// This struct contains a vector of individual simulation results along with
/// aggregate statistics computed across all simulations.
#[derive(Clone, Serialize, Deserialize, Default)]
pub struct SimulationStatsResult {
    /// Individual results from each simulation run
    pub results: Vec<SimulationResult>,

    /// Total number of simulations performed
    pub total_simulations: usize,

    /// Number of profitable simulations
    pub profitable_count: usize,

    /// Number of loss-making simulations
    pub loss_count: usize,

    /// Average P&L across all simulations
    pub average_pnl: Decimal,

    /// Median P&L across all simulations
    pub median_pnl: Decimal,

    /// Standard deviation of P&L
    pub std_dev_pnl: Decimal,

    /// Best (maximum) P&L achieved
    pub best_pnl: Decimal,

    /// Worst (minimum) P&L achieved
    pub worst_pnl: Decimal,

    /// Win rate (percentage of profitable simulations)
    pub win_rate: Decimal,

    /// Average holding period across all simulations
    pub average_holding_period: Decimal,
}

impl_json_debug_pretty!(SimulationStatsResult);
impl_json_display!(SimulationStatsResult);

#[cfg(test)]
mod tests {
    use super::*;
    use optionstratlib_analytics::pnl::PnL;
    use optionstratlib_core::pos_or_panic;

    use optionstratlib_simulation::simulation::ExitPolicy;
    use rust_decimal_macros::dec;

    fn create_test_simulation_result(
        sim_count: usize,
        pnl_value: Decimal,
        holding_period: usize,
        expired: bool,
    ) -> SimulationResult {
        SimulationResult {
            simulation_count: sim_count,
            risk_metrics: None,
            final_equity_percentiles: HashMap::new(),
            max_premium: dec!(100.0),
            min_premium: dec!(50.0),
            avg_premium: dec!(75.0),
            hit_take_profit: pnl_value > dec!(0.0),
            hit_stop_loss: pnl_value < dec!(0.0),
            expired,
            expiration_premium: if expired { Some(dec!(50.0)) } else { None },
            pnl: PnL::new(
                Some(pnl_value),
                None,
                pos_or_panic!(10.0),
                pos_or_panic!(5.0),
                Utc::now(),
            ),
            holding_period,
            exit_reason: ExitPolicy::Expiration,
        }
    }

    #[test]
    fn test_simulation_stats_creation() {
        let results = vec![
            create_test_simulation_result(1, dec!(100.0), 10, false),
            create_test_simulation_result(2, dec!(-50.0), 15, false),
            create_test_simulation_result(3, dec!(75.0), 12, true),
        ];

        let stats = SimulationStatsResult {
            results: results.clone(),
            total_simulations: 3,
            profitable_count: 2,
            loss_count: 1,
            average_pnl: dec!(41.67),
            median_pnl: dec!(75.0),
            std_dev_pnl: dec!(62.92),
            best_pnl: dec!(100.0),
            worst_pnl: dec!(-50.0),
            win_rate: dec!(66.67),
            average_holding_period: dec!(12.33),
        };

        assert_eq!(stats.total_simulations, 3);
        assert_eq!(stats.profitable_count, 2);
        assert_eq!(stats.loss_count, 1);
        assert_eq!(stats.results.len(), 3);
    }

    #[test]
    fn test_simulation_stats_all_profitable() {
        let results = vec![
            create_test_simulation_result(1, dec!(100.0), 10, false),
            create_test_simulation_result(2, dec!(50.0), 15, false),
            create_test_simulation_result(3, dec!(75.0), 12, false),
        ];

        let stats = SimulationStatsResult {
            results,
            total_simulations: 3,
            profitable_count: 3,
            loss_count: 0,
            average_pnl: dec!(75.0),
            median_pnl: dec!(75.0),
            std_dev_pnl: dec!(20.41),
            best_pnl: dec!(100.0),
            worst_pnl: dec!(50.0),
            win_rate: dec!(100.0),
            average_holding_period: dec!(12.33),
        };

        assert_eq!(stats.win_rate, dec!(100.0));
    }

    #[test]
    fn test_simulation_stats_all_losses() {
        let results = vec![
            create_test_simulation_result(1, dec!(-100.0), 10, false),
            create_test_simulation_result(2, dec!(-50.0), 15, false),
        ];

        let stats = SimulationStatsResult {
            results,
            total_simulations: 2,
            profitable_count: 0,
            loss_count: 2,
            average_pnl: dec!(-75.0),
            median_pnl: dec!(-75.0),
            std_dev_pnl: dec!(25.0),
            best_pnl: dec!(-50.0),
            worst_pnl: dec!(-100.0),
            win_rate: dec!(0.0),
            average_holding_period: dec!(12.5),
        };

        assert_eq!(stats.win_rate, dec!(0.0));
    }
}
