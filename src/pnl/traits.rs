use crate::ExpirationDate;
use crate::error::{PricingError, TransactionError};
use crate::model::Position;
use crate::pnl::transaction::Transaction;
use crate::pnl::utils::PnL;
use crate::strategies::DeltaAdjustment;
use positive::Positive;

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
    /// * `Result<PnL, PricingError>` - The calculated PnL or an error
    fn calculate_pnl(
        &self,
        _underlying_price: &Positive,
        _expiration_date: ExpirationDate,
        _implied_volatility: &Positive,
    ) -> Result<PnL, PricingError>;

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
    /// * `Result<PnL, PricingError>` - The calculated PnL at expiration or an error
    fn calculate_pnl_at_expiration(
        &self,
        _underlying_price: &Positive,
    ) -> Result<PnL, PricingError>;

    /// Calculates the Profit and Loss (PnL) for a single delta adjustment
    /// applied to a trading strategy.
    ///
    /// The default implementation is a no-op: strategies that do not track
    /// delta-adjustment PnL return a typed error so callers can detect the
    /// missing override without panicking.
    ///
    /// # Arguments
    ///
    /// * `_adjustments` - A `DeltaAdjustment` value describing the adjustment
    ///   applied to maintain delta neutrality in a trading strategy.
    ///
    /// # Returns
    ///
    /// * `Result<PnL, PricingError>` - Specific strategies return the computed
    ///   `PnL`; the default implementation always returns an error.
    ///
    /// # Errors
    ///
    /// Returns `PricingError::MethodError` (method = `"adjustments_pnl"`) when
    /// the implementing type does not override this method.
    fn adjustments_pnl(&self, _adjustments: &DeltaAdjustment) -> Result<PnL, PricingError> {
        Err(PricingError::method_error(
            "adjustments_pnl",
            &format!("not implemented for {}", std::any::type_name::<Self>()),
        ))
    }

    /// Calculates the profit and loss (PnL) for a given trading position.
    ///
    /// The default implementation is a no-op: strategies that do not implement
    /// per-position PnL return a typed error instead of panicking.
    ///
    /// # Parameters
    ///
    /// * `_position` - A reference to a trading position (`Position`) for which
    ///   the PnL is to be calculated.
    ///
    /// # Returns
    ///
    /// * `Result<PnL, PricingError>` - Specific strategies return the computed
    ///   `PnL`; the default implementation always returns an error.
    ///
    /// # Errors
    ///
    /// Returns `PricingError::MethodError` (method = `"diff_position_pnl"`)
    /// when the implementing type does not override this method.
    fn diff_position_pnl(&self, _position: &Position) -> Result<PnL, PricingError> {
        Err(PricingError::method_error(
            "diff_position_pnl",
            &format!("not implemented for {}", std::any::type_name::<Self>()),
        ))
    }
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

    use chrono::Utc;
    use positive::pos_or_panic;
    use rust_decimal_macros::dec;

    #[test]
    fn test_pnl_new() {
        let now = Utc::now();
        let pnl = PnL::new(
            Some(dec!(100.0)),
            Some(dec!(50.0)),
            pos_or_panic!(25.0),
            pos_or_panic!(75.0),
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
        let pnl = PnL::new(None, None, pos_or_panic!(10.0), pos_or_panic!(20.0), now);

        assert_eq!(pnl.realized, None);
        assert_eq!(pnl.unrealized, None);
        assert_eq!(pnl.initial_costs, pos_or_panic!(10.0));
        assert_eq!(pnl.initial_income, pos_or_panic!(20.0));
        assert_eq!(pnl.date_time, now);
    }

    struct DummyOption;

    impl TransactionAble for DummyOption {
        fn add_transaction(&mut self, _transaction: Transaction) -> Result<(), TransactionError> {
            unimplemented!()
        }

        fn get_transactions(&self) -> Result<Vec<Transaction>, TransactionError> {
            unimplemented!()
        }
    }

    impl PnLCalculator for DummyOption {
        fn calculate_pnl(
            &self,
            market_price: &Positive,
            expiration_date: ExpirationDate,
            _implied_volatility: &Positive,
        ) -> Result<PnL, PricingError> {
            Ok(PnL::new(
                Some(market_price.into()),
                None,
                pos_or_panic!(10.0),
                pos_or_panic!(20.0),
                expiration_date.get_date()?,
            ))
        }

        fn calculate_pnl_at_expiration(
            &self,
            underlying_price: &Positive,
        ) -> Result<PnL, PricingError> {
            let underlying_price = underlying_price.to_dec();
            Ok(PnL::new(
                Some(underlying_price),
                None,
                pos_or_panic!(10.0),
                pos_or_panic!(20.0),
                Utc::now(),
            ))
        }
    }

    #[test]
    fn test_pnl_calculator() {
        let dummy = DummyOption;
        let now = ExpirationDate::Days(pos_or_panic!(3.0));

        let pnl = dummy
            .calculate_pnl(&Positive::HUNDRED, now, &Positive::HUNDRED)
            .unwrap();
        assert_eq!(pnl.realized, Some(dec!(100.0)));
        assert_eq!(pnl.unrealized, None);
        assert_eq!(pnl.initial_costs, pos_or_panic!(10.0));
        assert_eq!(pnl.initial_income, pos_or_panic!(20.0));
        assert_eq!(
            pnl.date_time.format("%Y-%m-%d").to_string(),
            now.get_date_string().unwrap()
        );

        let pnl_at_expiration = dummy
            .calculate_pnl_at_expiration(&pos_or_panic!(150.0))
            .unwrap();
        assert_eq!(pnl_at_expiration.realized, Some(dec!(150.0)));
        assert_eq!(pnl_at_expiration.unrealized, None);
        assert_eq!(pnl_at_expiration.initial_costs, pos_or_panic!(10.0));
        assert_eq!(pnl_at_expiration.initial_income, pos_or_panic!(20.0));
    }

    #[test]
    fn test_default_adjustments_pnl_returns_method_error() {
        let dummy = DummyOption;
        let adj = DeltaAdjustment::NoAdjustmentNeeded;
        match dummy.adjustments_pnl(&adj) {
            Err(PricingError::MethodError { method, reason }) => {
                assert_eq!(method, "adjustments_pnl");
                assert!(reason.contains("not implemented"));
                assert!(reason.contains("DummyOption"));
            }
            other => panic!("expected MethodError, got {other:?}"),
        }
    }

    #[test]
    fn test_default_diff_position_pnl_returns_method_error() {
        use crate::model::types::{OptionStyle, Side};
        use crate::model::utils::create_sample_option_simplest;

        let dummy = DummyOption;
        let option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        let position = Position::new(
            option,
            pos_or_panic!(5.25),
            Utc::now(),
            pos_or_panic!(0.65),
            pos_or_panic!(0.65),
            None,
            None,
        );
        match dummy.diff_position_pnl(&position) {
            Err(PricingError::MethodError { method, reason }) => {
                assert_eq!(method, "diff_position_pnl");
                assert!(reason.contains("not implemented"));
                assert!(reason.contains("DummyOption"));
            }
            other => panic!("expected MethodError, got {other:?}"),
        }
    }
}
