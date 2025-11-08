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
use crate::risk::RiskMetricsSimulation;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use pretty_simple_display::{DebugPretty, DisplaySimple};
use crate::simulation::ExitPolicy;

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


#[derive(DebugPretty, DisplaySimple, Clone, Serialize, Deserialize, Default)]
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
    pub pnl: Decimal,
    
    /// Holding period in steps
    pub holding_period: usize,
    
    /// Exit policy that triggered the exit
    pub exit_reason: ExitPolicy,
}