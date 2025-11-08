/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 8/11/25
******************************************************************************/
use crate::backtesting::results::SimulationResult;
use crate::simulation::ExitPolicy;
use prettytable::{Cell, Row, Table, format};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::collections::HashMap;
use tracing::info;

/// Statistics for tracking Short Put strategy performance across multiple simulations.
#[derive(Debug, Clone)]
pub struct SimulationStats {
    /// Total number of simulations run
    total_simulations: usize,
    /// Number of trades that closed with profit (50% premium reduction)
    profitable_closes: usize,
    /// Number of trades that closed with loss (100% premium increase)
    loss_closes: usize,
    /// Number of trades that expired without hitting exit conditions
    expired_trades: usize,
    /// Total profit/loss across all simulations in dollars
    total_pnl: Decimal,
    /// Maximum profit achieved in a single simulation in dollars
    max_profit: Decimal,
    /// Maximum loss incurred in a single simulation in dollars
    max_loss: Decimal,
    /// Average holding period in steps for closed trades
    avg_holding_period: f64,
    /// Distribution of exit policies that triggered exits
    exit_reasons: HashMap<ExitPolicy, usize>,
    /// Individual simulation results
    results: Vec<SimulationResult>,
}

impl Default for SimulationStats {
    fn default() -> Self {
        Self::new()
    }
}

impl SimulationStats {
    /// Creates a new instance of `SimulationStats` with default values.
    ///
    /// # Returns
    ///
    /// A new `SimulationStats` instance with all counters set to zero.
    pub fn new() -> Self {
        Self {
            total_simulations: 0,
            profitable_closes: 0,
            loss_closes: 0,
            expired_trades: 0,
            total_pnl: dec!(0.0),
            max_profit: Decimal::MIN,
            max_loss: Decimal::MAX,
            avg_holding_period: 0.0,
            exit_reasons: HashMap::new(),
            results: Vec::new(),
        }
    }

    /// Updates statistics with results from a single simulation run.
    ///
    /// # Parameters
    ///
    /// * `result` - The simulation result containing all metrics
    pub fn update(&mut self, result: SimulationResult) {
        self.total_simulations += 1;
        self.total_pnl += result.pnl.realized.unwrap_or(dec!(0.0));

        if result.hit_take_profit {
            self.profitable_closes += 1;
        } else if result.hit_stop_loss {
            self.loss_closes += 1;
        } else if result.expired {
            self.expired_trades += 1;
        }

        // Track exit reason
        *self
            .exit_reasons
            .entry(result.exit_reason.clone())
            .or_insert(0) += 1;

        if let Some(realized) = result.pnl.realized {
            if realized > self.max_profit {
                self.max_profit = realized;
            }
            if realized < self.max_loss {
                self.max_loss = realized;
            }
        }

        // Update average holding period
        let total_holding = self.avg_holding_period * (self.total_simulations - 1) as f64;
        self.avg_holding_period =
            (total_holding + result.holding_period as f64) / self.total_simulations as f64;

        self.results.push(result);
    }

    /// Prints a formatted summary of the simulation statistics.
    pub fn print_summary(&self) {
        info!("========== SHORT PUT SIMULATION SUMMARY ==========");

        // General Info Table
        let mut info_table = Table::new();
        info_table.set_format(*format::consts::FORMAT_BOX_CHARS);
        info_table.add_row(Row::new(vec![
            Cell::new("Metric").style_spec("Fb"),
            Cell::new("Value").style_spec("Fb"),
        ]));
        info_table.add_row(Row::new(vec![
            Cell::new("Total Simulations"),
            Cell::new(&self.total_simulations.to_string()),
        ]));
        info_table.printstd();

        // Trade Outcomes Table
        info!("--- Trade Outcomes ---");
        let mut outcomes_table = Table::new();
        outcomes_table.set_format(*format::consts::FORMAT_BOX_CHARS);
        outcomes_table.add_row(Row::new(vec![
            Cell::new("Outcome").style_spec("Fb"),
            Cell::new("Count").style_spec("Fb"),
            Cell::new("Percentage").style_spec("Fb"),
        ]));

        if self.total_simulations > 0 {
            let win_rate = (self.profitable_closes as f64 / self.total_simulations as f64) * 100.0;
            let loss_rate = (self.loss_closes as f64 / self.total_simulations as f64) * 100.0;
            let expired_rate = (self.expired_trades as f64 / self.total_simulations as f64) * 100.0;

            outcomes_table.add_row(Row::new(vec![
                Cell::new("Profitable Closes (50% reduction)"),
                Cell::new(&self.profitable_closes.to_string()).style_spec("Fg"),
                Cell::new(&format!("{:.2}%", win_rate)).style_spec("Fg"),
            ]));
            outcomes_table.add_row(Row::new(vec![
                Cell::new("Loss Closes (100% increase)"),
                Cell::new(&self.loss_closes.to_string()).style_spec("Fr"),
                Cell::new(&format!("{:.2}%", loss_rate)).style_spec("Fr"),
            ]));
            outcomes_table.add_row(Row::new(vec![
                Cell::new("Expired Trades"),
                Cell::new(&self.expired_trades.to_string()),
                Cell::new(&format!("{:.2}%", expired_rate)),
            ]));
        }
        outcomes_table.printstd();

        // P&L Statistics Table
        info!("--- Profit/Loss Statistics ---");
        let mut pnl_table = Table::new();
        pnl_table.set_format(*format::consts::FORMAT_BOX_CHARS);
        pnl_table.add_row(Row::new(vec![
            Cell::new("Metric").style_spec("Fb"),
            Cell::new("Amount").style_spec("Fb"),
        ]));

        pnl_table.add_row(Row::new(vec![
            Cell::new("Total P&L"),
            Cell::new(&format!("${:.2}", self.total_pnl)),
        ]));

        if self.total_simulations > 0 {
            let avg_pnl = self.total_pnl / Decimal::from(self.total_simulations);
            pnl_table.add_row(Row::new(vec![
                Cell::new("Average P&L per Trade"),
                Cell::new(&format!("${:.2}", avg_pnl)),
            ]));
        }

        pnl_table.add_row(Row::new(vec![
            Cell::new("Maximum Profit"),
            Cell::new(&format!("${:.2}", self.max_profit)).style_spec("Fg"),
        ]));
        pnl_table.add_row(Row::new(vec![
            Cell::new("Maximum Loss"),
            Cell::new(&format!("${:.2}", self.max_loss)).style_spec("Fr"),
        ]));
        pnl_table.printstd();

        // Holding Period Table
        info!("--- Holding Period ---");
        let mut holding_table = Table::new();
        holding_table.set_format(*format::consts::FORMAT_BOX_CHARS);
        holding_table.add_row(Row::new(vec![
            Cell::new("Metric").style_spec("Fb"),
            Cell::new("Value").style_spec("Fb"),
        ]));
        holding_table.add_row(Row::new(vec![
            Cell::new("Average Holding Period"),
            Cell::new(&format!("{:.2} steps", self.avg_holding_period)),
        ]));
        holding_table.printstd();

        // Exit Reasons Table
        info!("--- Exit Reasons ---");
        let mut exit_table = Table::new();
        exit_table.set_format(*format::consts::FORMAT_BOX_CHARS);
        exit_table.add_row(Row::new(vec![
            Cell::new("Exit Reason").style_spec("Fb"),
            Cell::new("Count").style_spec("Fb"),
            Cell::new("Percentage").style_spec("Fb"),
        ]));

        for (reason, count) in &self.exit_reasons {
            let percentage = (*count as f64 / self.total_simulations as f64) * 100.0;
            exit_table.add_row(Row::new(vec![
                Cell::new(&reason.to_string()),
                Cell::new(&count.to_string()),
                Cell::new(&format!("{:.2}%", percentage)),
            ]));
        }
        exit_table.printstd();

        info!("==================================================");
    }

    /// Prints detailed results for each individual simulation in a table format.
    pub fn print_individual_results(&self) {
        info!("========== INDIVIDUAL SIMULATION RESULTS ==========");

        let mut table = Table::new();
        table.set_format(*format::consts::FORMAT_BOX_CHARS);

        // Add header
        table.add_row(Row::new(vec![
            Cell::new("Sim"),
            Cell::new("Max\nPremium"),
            Cell::new("Min\nPremium"),
            Cell::new("Avg\nPremium"),
            Cell::new("Final\nP&L"),
            Cell::new("Holding\nPeriod"),
            Cell::new("Exit\nReason"),
        ]));

        // Add data rows
        for result in &self.results {
            table.add_row(Row::new(vec![
                Cell::new(&result.simulation_count.to_string()),
                Cell::new(&format!("${:.2}", result.max_premium)),
                Cell::new(&format!("${:.2}", result.min_premium)),
                Cell::new(&format!("${:.2}", result.avg_premium)),
                Cell::new(&format!("${:.2}", result.pnl.realized.unwrap_or(dec!(0.0)))),
                Cell::new(&result.holding_period.to_string()),
                Cell::new(&result.exit_reason.to_string()),
            ]));
        }

        table.printstd();
    }
}
