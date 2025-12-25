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
use crate::pnl::PnL;
use crate::risk::RiskMetricsSimulation;
use crate::simulation::ExitPolicy;
use chrono::{DateTime, Utc};
use pretty_simple_display::{DebugPretty, DisplaySimple};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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
#[derive(DebugPretty, DisplaySimple, Clone, Serialize, Deserialize, Default, ToSchema)]
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

    /// Whether the take profit target was hit (50% premium reduction)
    pub hit_take_profit: bool,

    /// Whether the stop loss was hit (100% premium increase)
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

/// Statistics aggregated from multiple simulation runs.
///
/// This struct contains a vector of individual simulation results along with
/// aggregate statistics computed across all simulations.
#[derive(DebugPretty, DisplaySimple, Clone, Serialize, Deserialize, Default)]
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

impl SimulationStatsResult {
    /// Prints a formatted summary of the simulation statistics.
    ///
    /// This method outputs comprehensive statistics including:
    /// - Total number of simulations
    /// - Trade outcomes (profitable, loss, expired)
    /// - P&L statistics (total, average, max, min)
    /// - Holding period information
    /// - Exit reason distribution
    pub fn print_summary(&self) {
        use prettytable::{Cell, Row, Table, color, format};
        use rust_decimal_macros::dec;
        use tracing::info;

        info!("\n========== SIMULATION SUMMARY ==========");

        // General Info Table
        let mut general_table = Table::new();
        general_table.set_format(*format::consts::FORMAT_BOX_CHARS);
        general_table.set_titles(Row::new(vec![
            Cell::new("Metric").with_style(prettytable::Attr::ForegroundColor(color::BLUE)),
            Cell::new("Value").with_style(prettytable::Attr::ForegroundColor(color::BLUE)),
        ]));
        general_table.add_row(Row::new(vec![
            Cell::new("Total Simulations"),
            Cell::new(&self.total_simulations.to_string()),
        ]));
        general_table.printstd();

        // Trade Outcomes Table
        info!("\n--- Trade Outcomes ---");
        let mut outcomes_table = Table::new();
        outcomes_table.set_format(*format::consts::FORMAT_BOX_CHARS);
        outcomes_table.set_titles(Row::new(vec![
            Cell::new("Outcome").with_style(prettytable::Attr::ForegroundColor(color::BLUE)),
            Cell::new("Count").with_style(prettytable::Attr::ForegroundColor(color::BLUE)),
            Cell::new("Percentage").with_style(prettytable::Attr::ForegroundColor(color::BLUE)),
        ]));

        let expired_count = self.results.iter().filter(|r| r.expired).count();

        outcomes_table.add_row(Row::new(vec![
            Cell::new("Profitable Trades"),
            Cell::new(&self.profitable_count.to_string()),
            Cell::new(&format!("{:.2}%", self.win_rate)),
        ]));
        outcomes_table.add_row(Row::new(vec![
            Cell::new("Loss Trades"),
            Cell::new(&self.loss_count.to_string()),
            Cell::new(&format!(
                "{:.2}%",
                (self.loss_count as f64 / self.total_simulations as f64) * 100.0
            )),
        ]));
        outcomes_table.add_row(Row::new(vec![
            Cell::new("Expired Trades"),
            Cell::new(&expired_count.to_string()),
            Cell::new(&format!(
                "{:.2}%",
                (expired_count as f64 / self.total_simulations as f64) * 100.0
            )),
        ]));
        outcomes_table.printstd();

        // P&L Statistics Table
        info!("\n--- Profit/Loss Statistics ---");
        let mut pnl_table = Table::new();
        pnl_table.set_format(*format::consts::FORMAT_BOX_CHARS);
        pnl_table.set_titles(Row::new(vec![
            Cell::new("Metric").with_style(prettytable::Attr::ForegroundColor(color::BLUE)),
            Cell::new("Amount").with_style(prettytable::Attr::ForegroundColor(color::BLUE)),
        ]));

        let total_pnl: Decimal = self.results.iter().filter_map(|r| r.pnl.total_pnl()).sum();

        // Helper function to color P&L values
        let color_pnl = |value: Decimal| -> Cell {
            let text = format!("${:.2}", value);
            if value < dec!(0.0) {
                Cell::new(&text).with_style(prettytable::Attr::ForegroundColor(color::RED))
            } else if value > dec!(0.0) {
                Cell::new(&text).with_style(prettytable::Attr::ForegroundColor(color::GREEN))
            } else {
                Cell::new(&text)
            }
        };

        pnl_table.add_row(Row::new(vec![Cell::new("Total P&L"), color_pnl(total_pnl)]));
        pnl_table.add_row(Row::new(vec![
            Cell::new("Average P&L per Trade"),
            color_pnl(self.average_pnl),
        ]));
        pnl_table.add_row(Row::new(vec![
            Cell::new("Median P&L"),
            color_pnl(self.median_pnl),
        ]));
        pnl_table.add_row(Row::new(vec![
            Cell::new("Std Dev P&L"),
            Cell::new(&format!("${:.2}", self.std_dev_pnl)),
        ]));
        pnl_table.add_row(Row::new(vec![
            Cell::new("Maximum Profit"),
            color_pnl(self.best_pnl),
        ]));
        pnl_table.add_row(Row::new(vec![
            Cell::new("Maximum Loss"),
            color_pnl(self.worst_pnl),
        ]));
        pnl_table.printstd();

        // Holding Period Table
        info!("\n--- Holding Period ---");
        let mut holding_table = Table::new();
        holding_table.set_format(*format::consts::FORMAT_BOX_CHARS);
        holding_table.set_titles(Row::new(vec![
            Cell::new("Metric").with_style(prettytable::Attr::ForegroundColor(color::BLUE)),
            Cell::new("Value").with_style(prettytable::Attr::ForegroundColor(color::BLUE)),
        ]));
        holding_table.add_row(Row::new(vec![
            Cell::new("Average Holding Period"),
            Cell::new(&format!("{:.2} steps", self.average_holding_period)),
        ]));
        holding_table.printstd();

        // Exit Reasons Table
        info!("\n--- Exit Reasons ---");
        let mut exit_reasons: HashMap<String, usize> = HashMap::new();
        for result in &self.results {
            *exit_reasons
                .entry(result.exit_reason.to_string())
                .or_insert(0) += 1;
        }

        let mut exit_table = Table::new();
        exit_table.set_format(*format::consts::FORMAT_BOX_CHARS);
        exit_table.set_titles(Row::new(vec![
            Cell::new("Exit Reason").with_style(prettytable::Attr::ForegroundColor(color::BLUE)),
            Cell::new("Count").with_style(prettytable::Attr::ForegroundColor(color::BLUE)),
            Cell::new("Percentage").with_style(prettytable::Attr::ForegroundColor(color::BLUE)),
        ]));

        for (reason, count) in exit_reasons.iter() {
            exit_table.add_row(Row::new(vec![
                Cell::new(reason),
                Cell::new(&count.to_string()),
                Cell::new(&format!(
                    "{:.2}%",
                    (*count as f64 / self.total_simulations as f64) * 100.0
                )),
            ]));
        }

        exit_table.printstd();
    }

    /// Prints individual results for each simulation.
    ///
    /// This method outputs a detailed table showing:
    /// - Simulation number
    /// - Maximum and minimum premium observed
    /// - Average premium
    /// - Final P&L
    /// - Holding period
    /// - Exit reason
    pub fn print_individual_results(&self) {
        use prettytable::{Cell, Row, Table, color, format};
        use rust_decimal_macros::dec;
        use tracing::info;

        info!("\n========== INDIVIDUAL SIMULATION RESULTS ==========");

        let mut table = Table::new();
        table.set_format(*format::consts::FORMAT_BOX_CHARS);
        table.set_titles(Row::new(vec![
            Cell::new("Sim").with_style(prettytable::Attr::ForegroundColor(color::BLUE)),
            Cell::new("Max\nPremium").with_style(prettytable::Attr::ForegroundColor(color::BLUE)),
            Cell::new("Min\nPremium").with_style(prettytable::Attr::ForegroundColor(color::BLUE)),
            Cell::new("Avg\nPremium").with_style(prettytable::Attr::ForegroundColor(color::BLUE)),
            Cell::new("Final\nP&L").with_style(prettytable::Attr::ForegroundColor(color::BLUE)),
            Cell::new("Holding\nPeriod")
                .with_style(prettytable::Attr::ForegroundColor(color::BLUE)),
            Cell::new("Exit\nReason").with_style(prettytable::Attr::ForegroundColor(color::BLUE)),
        ]));

        for result in &self.results {
            let pnl = result.pnl.total_pnl().unwrap_or_default();

            // Color the P&L cell based on value
            let pnl_cell = if pnl < dec!(0.0) {
                Cell::new(&format!("${:.2}", pnl))
                    .with_style(prettytable::Attr::ForegroundColor(color::RED))
            } else if pnl > dec!(0.0) {
                Cell::new(&format!("${:.2}", pnl))
                    .with_style(prettytable::Attr::ForegroundColor(color::GREEN))
            } else {
                Cell::new(&format!("${:.2}", pnl))
            };

            table.add_row(Row::new(vec![
                Cell::new(&result.simulation_count.to_string()),
                Cell::new(&format!("${:.2}", result.max_premium)),
                Cell::new(&format!("${:.2}", result.min_premium)),
                Cell::new(&format!("${:.2}", result.avg_premium)),
                pnl_cell,
                Cell::new(&result.holding_period.to_string()),
                Cell::new(&result.exit_reason.to_string()),
            ]));
        }

        table.printstd();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pnl::PnL;

    use crate::simulation::ExitPolicy;
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
    fn test_simulation_stats_print_summary() {
        let results = vec![
            create_test_simulation_result(1, dec!(100.0), 10, false),
            create_test_simulation_result(2, dec!(-50.0), 15, true),
        ];

        let stats = SimulationStatsResult {
            results,
            total_simulations: 2,
            profitable_count: 1,
            loss_count: 1,
            average_pnl: dec!(25.0),
            median_pnl: dec!(25.0),
            std_dev_pnl: dec!(75.0),
            best_pnl: dec!(100.0),
            worst_pnl: dec!(-50.0),
            win_rate: dec!(50.0),
            average_holding_period: dec!(12.5),
        };

        // This should not panic
        stats.print_summary();
    }

    #[test]
    fn test_simulation_stats_print_individual_results() {
        let results = vec![
            create_test_simulation_result(1, dec!(100.0), 10, false),
            create_test_simulation_result(2, dec!(-50.0), 15, false),
            create_test_simulation_result(3, dec!(75.0), 12, true),
        ];

        let stats = SimulationStatsResult {
            results,
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

        // This should not panic
        stats.print_individual_results();
    }

    #[test]
    fn test_simulation_stats_empty_results() {
        let stats = SimulationStatsResult {
            results: vec![],
            total_simulations: 0,
            profitable_count: 0,
            loss_count: 0,
            average_pnl: dec!(0.0),
            median_pnl: dec!(0.0),
            std_dev_pnl: dec!(0.0),
            best_pnl: dec!(0.0),
            worst_pnl: dec!(0.0),
            win_rate: dec!(0.0),
            average_holding_period: dec!(0.0),
        };

        // Should handle empty results gracefully
        stats.print_summary();
        stats.print_individual_results();
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

        stats.print_summary();
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

        stats.print_summary();
        assert_eq!(stats.win_rate, dec!(0.0));
    }
}
