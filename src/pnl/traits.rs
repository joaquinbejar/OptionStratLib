use crate::error::TransactionError;
use crate::pnl::transaction::Transaction;
use crate::pnl::utils::PnL;
use crate::{ExpirationDate, Positive};
use std::error::Error;

/// Defines the interface for profit and loss (PnL) calculation on financial instruments.
///
/// This trait provides methods to calculate the profit and loss of financial instruments
/// (particularly options) under different scenarios: at current market conditions and
/// at expiration. Implementations of this trait can provide specific PnL calculation
/// logic for different types of financial instruments or strategies.
///
pub trait PnLCalculator {
    /// Calculates the current PnL based on market conditions.
    ///
    /// This method computes the profit and loss of a financial instrument given
    /// the current underlying price, time to expiration, and implied volatility.
    /// It returns a complete PnL structure with realized and unrealized values.
    ///
    /// # Parameters
    /// * `_underlying_price` - The current market price of the underlying asset
    /// * `_expiration_date` - The expiration date of the instrument
    /// * `_implied_volatility` - The current implied volatility
    ///
    /// # Returns
    /// * `Result<PnL, Box<dyn Error>>` - The calculated PnL or an error
    fn calculate_pnl(
        &self,
        _underlying_price: &Positive,
        _expiration_date: ExpirationDate,
        _implied_volatility: &Positive,
    ) -> Result<PnL, Box<dyn Error>>;

    /// Calculates the PnL at the expiration of the instrument.
    ///
    /// This method computes the final profit and loss at the expiration date,
    /// which is typically simpler than the pre-expiration calculation since
    /// time value and volatility no longer factor into the price.
    ///
    /// # Parameters
    /// * `_underlying_price` - The price of the underlying asset at expiration
    ///
    /// # Returns
    /// * `Result<PnL, Box<dyn Error>>` - The calculated PnL at expiration or an error
    fn calculate_pnl_at_expiration(
        &self,
        _underlying_price: &Positive,
    ) -> Result<PnL, Box<dyn Error>>;
}

/// # TransactionAble
///
/// A trait that defines the ability to manage financial transactions within an entity.
///
/// This trait provides a standardized interface for adding and retrieving transaction records,
/// enabling consistent transaction management across different implementations.
///
/// ## Required Methods
///
/// - `add_transaction`: Adds a new transaction to the implementing entity
/// - `get_transactions`: Retrieves all transactions from the implementing entity
///
/// ## Error Handling
///
/// Both methods return a `Result` type that may contain a `TransactionError` if the operation fails.
/// This allows for proper error propagation and handling in transaction-related operations.
///
pub trait TransactionAble {
    /// Adds a new transaction to the implementing entity.
    fn add_transaction(&mut self, transaction: Transaction) -> Result<(), TransactionError>;

    /// Retrieves all transactions from the implementing entity.
    fn get_transactions(&self) -> Result<Vec<Transaction>, TransactionError>;
}

#[cfg(test)]
mod tests_pnl_calculator {
    use super::*;
    use crate::pos;
    use chrono::Utc;
    use rust_decimal_macros::dec;

    #[test]

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

    impl TransactionAble for DummyOption {
        fn add_transaction(&mut self, _transaction: Transaction) -> Result<(), TransactionError> {
            todo!()
        }

        fn get_transactions(&self) -> Result<Vec<Transaction>, TransactionError> {
            todo!()
        }
    }

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

    fn test_pnl_calculator() {
        let dummy = DummyOption;
        let now = ExpirationDate::Days(pos!(3.0));

        let pnl = dummy
            .calculate_pnl(&pos!(100.0), now, &pos!(100.0))
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
