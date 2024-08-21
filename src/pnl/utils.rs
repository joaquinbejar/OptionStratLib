/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 16/8/24
******************************************************************************/

use chrono::{DateTime, Utc};

/// Represents the Profit and Loss (PnL) of a financial instrument.
#[derive(Debug, Clone)]
pub struct PnL {
    /// The realized profit or loss.
    pub realized: Option<f64>,
    /// The unrealized profit or loss.
    pub unrealized: Option<f64>,
    /// The initial costs (fees, premiums paid).
    pub initial_costs: f64,
    /// The initial income (premiums received).
    pub initial_income: f64,
    /// The date of the PnL calculation.
    pub date_time: DateTime<Utc>,
}

impl PnL {
    pub fn new(
        realized: Option<f64>,
        unrealized: Option<f64>,
        initial_costs: f64,
        initial_income: f64,
        date_time: DateTime<Utc>,
    ) -> Self {
        PnL {
            realized,
            unrealized,
            initial_costs,
            initial_income,
            date_time,
        }
    }
}

pub trait PnLCalculator {
    fn calculate_pnl(&self, date_time: DateTime<Utc>, market_price: f64) -> PnL;
    fn calculate_pnl_at_expiration(&self, underlying_price: Option<f64>) -> PnL;
}
