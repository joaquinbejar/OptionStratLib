/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 16/8/24
******************************************************************************/
use crate::{ExpirationDate, Positive};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use std::error::Error;

/// Represents the Profit and Loss (PnL) of a financial instrument.
#[derive(Debug, Clone)]
pub struct PnL {
    /// The realized profit or loss.
    pub realized: Option<Decimal>,
    /// The unrealized profit or loss.
    pub unrealized: Option<Decimal>,
    /// The initial costs (fees, premiums paid).
    pub initial_costs: Positive,
    /// The initial income (premiums received).
    pub initial_income: Positive,
    /// The date of the PnL calculation.
    pub date_time: DateTime<Utc>,
}

impl PnL {
    pub fn new(
        realized: Option<Decimal>,
        unrealized: Option<Decimal>,
        initial_costs: Positive,
        initial_income: Positive,
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
    fn calculate_pnl(
        &self,
        _market_price: &Positive,
        _expiration_date: ExpirationDate,
        _implied_volatility: &Positive,
    ) -> Result<PnL, Box<dyn Error>>;

    fn calculate_pnl_at_expiration(
        &self,
        _underlying_price: &Positive,
    ) -> Result<PnL, Box<dyn Error>>;
}

#[cfg(test)]
mod tests_pnl_calculator {
    use super::*;
    use crate::pos;
    use chrono::Utc;
    use rust_decimal_macros::dec;

    #[cfg(target_arch = "wasm32")]
    use wasm_bindgen_test::*;

    #[cfg(target_arch = "wasm32")]
    wasm_bindgen_test_configure!(run_in_browser);

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn test_pnl_new() {
        let now = Utc::now();
        let pnl = PnL::new(
            Some(dec!(100.0)),
            Some(dec!(50.0)),
            pos!(25.0),
            pos!(75.0),
            now,
        );

        assert_eq!(pnl.realized, Some(dec!(100.0)));
        assert_eq!(pnl.unrealized, Some(dec!(50.0)));
        assert_eq!(pnl.initial_costs, 25.0);
        assert_eq!(pnl.initial_income, 75.0);
        assert_eq!(pnl.date_time, now);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn test_pnl_with_none_values() {
        let now = Utc::now();
        let pnl = PnL::new(None, None, pos!(10.0), pos!(20.0), now);

        assert_eq!(pnl.realized, None);
        assert_eq!(pnl.unrealized, None);
        assert_eq!(pnl.initial_costs, pos!(10.0));
        assert_eq!(pnl.initial_income, pos!(20.0));
        assert_eq!(pnl.date_time, now);
    }

    struct DummyOption;

    impl PnLCalculator for DummyOption {
        fn calculate_pnl(
            &self,
            market_price: &Positive,
            expiration_date: ExpirationDate,
            _implied_volatility: &Positive,
        ) -> Result<PnL, Box<dyn Error>> {
            Ok(PnL::new(
                Some(market_price.into()),
                None,
                pos!(10.0),
                pos!(20.0),
                expiration_date.get_date()?,
            ))
        }

        fn calculate_pnl_at_expiration(
            &self,
            underlying_price: &Positive,
        ) -> Result<PnL, Box<dyn Error>> {
            let underlying_price = underlying_price.to_dec();
            Ok(PnL::new(
                Some(underlying_price),
                None,
                pos!(10.0),
                pos!(20.0),
                Utc::now(),
            ))
        }
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    fn test_pnl_calculator() {
        let dummy = DummyOption;
        let now = ExpirationDate::Days(pos!(3.0));

        let pnl = dummy
            .calculate_pnl(&pos!(100.0), now.clone(), &pos!(100.0))
            .unwrap();
        assert_eq!(pnl.realized, Some(dec!(100.0)));
        assert_eq!(pnl.unrealized, None);
        assert_eq!(pnl.initial_costs, pos!(10.0));
        assert_eq!(pnl.initial_income, pos!(20.0));
        assert_eq!(
            pnl.date_time.format("%Y-%m-%d").to_string(),
            now.get_date_string().unwrap()
        );

        let pnl_at_expiration = dummy.calculate_pnl_at_expiration(&pos!(150.0)).unwrap();
        assert_eq!(pnl_at_expiration.realized, Some(dec!(150.0)));
        assert_eq!(pnl_at_expiration.unrealized, None);
        assert_eq!(pnl_at_expiration.initial_costs, pos!(10.0));
        assert_eq!(pnl_at_expiration.initial_income, pos!(20.0));
    }
}
