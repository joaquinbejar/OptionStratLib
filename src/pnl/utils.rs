/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 16/8/24
******************************************************************************/

use crate::model::types::PositiveF64;
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
    fn calculate_pnl(&self, date_time: DateTime<Utc>, market_price: PositiveF64) -> PnL;
    fn calculate_pnl_at_expiration(&self, underlying_price: Option<PositiveF64>) -> PnL;
}

#[cfg(test)]
mod tests_pnl_calculator {
    use super::*;
    use chrono::Utc;
    use crate::pos;

    #[test]
    fn test_pnl_new() {
        let now = Utc::now();
        let pnl = PnL::new(Some(100.0), Some(50.0), 25.0, 75.0, now);

        assert_eq!(pnl.realized, Some(100.0));
        assert_eq!(pnl.unrealized, Some(50.0));
        assert_eq!(pnl.initial_costs, 25.0);
        assert_eq!(pnl.initial_income, 75.0);
        assert_eq!(pnl.date_time, now);
    }

    #[test]
    fn test_pnl_with_none_values() {
        let now = Utc::now();
        let pnl = PnL::new(None, None, 10.0, 20.0, now);

        assert_eq!(pnl.realized, None);
        assert_eq!(pnl.unrealized, None);
        assert_eq!(pnl.initial_costs, 10.0);
        assert_eq!(pnl.initial_income, 20.0);
        assert_eq!(pnl.date_time, now);
    }

    // Esta estructura es solo para probar el trait PnLCalculator
    struct DummyOption;

    impl PnLCalculator for DummyOption {
        fn calculate_pnl(&self, date_time: DateTime<Utc>, market_price: PositiveF64) -> PnL {
            PnL::new(Some(market_price.value()), None, 10.0, 20.0, date_time)
        }

        fn calculate_pnl_at_expiration(&self, underlying_price: Option<PositiveF64>) -> PnL {
            let underlying_price = Some(underlying_price.unwrap().value());
            PnL::new(underlying_price, None, 10.0, 20.0, Utc::now())
        }
    }

    #[test]
    fn test_pnl_calculator() {
        let dummy = DummyOption;
        let now = Utc::now();

        let pnl = dummy.calculate_pnl(now, pos!(100.0));
        assert_eq!(pnl.realized, Some(100.0));
        assert_eq!(pnl.unrealized, None);
        assert_eq!(pnl.initial_costs, 10.0);
        assert_eq!(pnl.initial_income, 20.0);
        assert_eq!(pnl.date_time, now);

        let pnl_at_expiration = dummy.calculate_pnl_at_expiration(Some(pos!(150.0)));
        assert_eq!(pnl_at_expiration.realized, Some(150.0));
        assert_eq!(pnl_at_expiration.unrealized, None);
        assert_eq!(pnl_at_expiration.initial_costs, 10.0);
        assert_eq!(pnl_at_expiration.initial_income, 20.0);
    }
}