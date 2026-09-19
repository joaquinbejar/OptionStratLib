/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 19/9/25
******************************************************************************/

//! P&L capability implementations for the core model types.
//!
//! [`PnLCalculator`] for `Options` and `Position`, and [`TransactionAble`]
//! for `Position`, live next to their traits (local trait, core-owned type)
//! so the core model never references the P&L layer. The valuation inside
//! `calculate_pnl` goes through the pricing-owned [`OptionPricing`] trait.

use crate::error::{PositionError, PricingError, TransactionError};
use crate::model::decimal::{d_add, d_sub};
use crate::model::types::Side;
use crate::model::{ExpirationDate, Options, Position, resolve_expiration_date};
use crate::pnl::utils::PnL;
use crate::pnl::{PnLCalculator, Transaction, TransactionAble};
use crate::pricing::OptionPricing;
use num_traits::ToPrimitive;
use positive::Positive;
use rust_decimal::Decimal;

impl PnLCalculator for Options {
    fn calculate_pnl(
        &self,
        market_price: &Positive,
        expiration_date: ExpirationDate,
        implied_volatility: &Positive,
    ) -> Result<PnL, PricingError> {
        // Create a copy of the current option with updated parameters
        let mut current_option = self.clone();
        current_option.underlying_price = *market_price;
        current_option.expiration_date = expiration_date;
        current_option.implied_volatility = *implied_volatility;

        // Calculate theoretical price at current market conditions
        let current_price = OptionPricing::calculate_price_black_scholes(&current_option)?;

        // Calculate initial price (when option was created)
        let initial_price = OptionPricing::calculate_price_black_scholes(self)?;

        // Calculate initial costs (premium paid/received)
        let (initial_costs, initial_income) = match self.side {
            Side::Long => (initial_price * self.quantity, Decimal::ZERO),
            Side::Short => (Decimal::ZERO, -initial_price * self.quantity),
        };

        // Calculate unrealized PnL adjusted for position side
        let unrealized = Some((current_price - initial_price) * self.quantity);

        Ok(PnL::new(
            None, // No realized PnL yet
            unrealized,
            Positive::new_decimal(initial_costs)?,
            Positive::new_decimal(initial_income)?,
            resolve_expiration_date(&current_option.expiration_date)?,
        ))
    }

    fn calculate_pnl_at_expiration(
        &self,
        underlying_price: &Positive,
    ) -> Result<PnL, PricingError> {
        let realized = Some(self.payoff_at_price(underlying_price)?);
        let initial_price = OptionPricing::calculate_price_black_scholes(self)?;

        let (initial_costs, initial_income) = match self.side {
            Side::Long => (initial_price * self.quantity, Decimal::ZERO),
            Side::Short => (Decimal::ZERO, initial_price * self.quantity),
        };

        Ok(PnL::new(
            realized, // No realized PnL yet
            None,
            Positive::new_decimal(initial_costs)?,
            Positive::new_decimal(initial_income)?,
            resolve_expiration_date(&self.expiration_date)?,
        ))
    }
}

/// # Position Profit and Loss (PnL) Calculator
///
/// This trait implementation provides methods to calculate the profit and loss (PnL)
/// for option positions under different market scenarios.
///
/// The implementation offers two main calculations:
/// 1. Current PnL based on updated market conditions
/// 2. PnL at expiration based on a projected underlying price
///
/// These calculations are essential for risk management, position monitoring, and
/// strategy planning in options trading.
impl PnLCalculator for Position {
    /// Calculates the current unrealized profit and loss for an option position
    /// based on updated market conditions.
    ///
    /// This method computes the difference between the option's price at entry and its
    /// current theoretical price using the Black-Scholes model. It factors in changes to:
    /// - The underlying asset price
    /// - Time to expiration
    /// - Implied volatility
    ///
    /// # Arguments
    ///
    /// * `underlying_price` - The current price of the underlying asset
    /// * `expiration_date` - The updated expiration date for the calculation
    /// * `implied_volatility` - The current implied volatility of the option
    ///
    /// # Returns
    ///
    /// * `Result<PnL, PricingError>` - A PnL object containing unrealized profit/loss and position cost details,
    ///   or an error if the calculation fails
    fn calculate_pnl(
        &self,
        underlying_price: &Positive,
        expiration_date: ExpirationDate,
        implied_volatility: &Positive,
    ) -> Result<PnL, PricingError> {
        let price_at_buy = OptionPricing::calculate_price_black_scholes(&self.option)?;
        let mut current_option = self.option.clone();
        current_option.expiration_date = expiration_date;
        current_option.underlying_price = *underlying_price;
        current_option.implied_volatility = *implied_volatility;
        let price_at_sell = OptionPricing::calculate_price_black_scholes(&current_option)?;
        let unrealized = price_at_sell - price_at_buy;
        let initial_cost = self.total_cost()?;
        let initial_income = self.premium_received()?;

        let realized = initial_income.to_dec() - initial_cost.to_dec();
        Ok(PnL::new(
            Some(realized),
            Some(unrealized),
            initial_cost,
            initial_income,
            self.date,
        ))
    }

    /// Calculates the expected profit and loss at option expiration for a given
    /// underlying price.
    ///
    /// This method determines the realized profit or loss that would occur if the option
    /// expires with the underlying at the specified price. It uses intrinsic value calculation
    /// at expiration rather than Black-Scholes pricing.
    ///
    /// # Arguments
    ///
    /// * `underlying_price` - The projected price of the underlying asset at expiration
    ///
    /// # Returns
    ///
    /// * `Result<PnL, PricingError>` - A PnL object containing realized profit/loss and position cost details,
    ///   or an error if the calculation fails
    fn calculate_pnl_at_expiration(
        &self,
        underlying_price: &Positive,
    ) -> Result<PnL, PricingError> {
        let initial_cost = self.total_cost()?;
        let initial_income = self.premium_received()?;
        // `ExpirationDate::get_date` resolves a relative expiration with the
        // `+` operator on `DateTime<Utc>`, which aborts past the calendar
        // range instead of reporting it.
        let date_time = resolve_expiration_date(&self.option.expiration_date)?;

        // A cost or an income at the top of the `Positive` range overflows the
        // raw `Decimal` operators, so the running total is taken through
        // `d_sub`/`d_add` exactly as `pnl_at_expiration` does.
        let intrinsic = self.option.intrinsic_value(*underlying_price)?;
        let net_after_cost = d_sub(
            intrinsic,
            initial_cost.to_dec(),
            "position::calculate_pnl_at_expiration::net",
        )?;
        let realized = d_add(
            net_after_cost,
            initial_income.to_dec(),
            "position::calculate_pnl_at_expiration::total",
        )?;
        Ok(PnL::new(
            Some(realized),
            Some(Decimal::ZERO),
            initial_cost,
            initial_income,
            date_time,
        ))
    }

    fn diff_position_pnl(&self, position: &Position) -> Result<PnL, PricingError> {
        // Validate that positions belong to the same options

        // Check option_type
        if self.option.option_type != position.option.option_type {
            return Err(PositionError::invalid_position(&format!(
                "Option types do not match: {:?} vs {:?}",
                self.option.option_type, position.option.option_type
            ))
            .into());
        }

        // Check side
        if self.option.side != position.option.side {
            return Err(PositionError::invalid_position(&format!(
                "Sides do not match: {:?} vs {:?}",
                self.option.side, position.option.side
            ))
            .into());
        }

        // Check underlying_symbol
        if self.option.underlying_symbol != position.option.underlying_symbol {
            return Err(PositionError::invalid_position(&format!(
                "Underlying symbols do not match: {} vs {}",
                self.option.underlying_symbol, position.option.underlying_symbol
            ))
            .into());
        }

        // Check strike_price
        if self.option.strike_price != position.option.strike_price {
            return Err(PositionError::invalid_position(&format!(
                "Strike prices do not match: {} vs {}",
                self.option.strike_price, position.option.strike_price
            ))
            .into());
        }

        // Check expiration_date
        if self.option.expiration_date != position.option.expiration_date {
            return Err(PositionError::invalid_position(&format!(
                "Expiration dates do not match: {:?} vs {:?}",
                self.option.expiration_date, position.option.expiration_date
            ))
            .into());
        }

        // Check quantity
        if self.option.quantity != position.option.quantity {
            return Err(PositionError::invalid_position(&format!(
                "Quantities do not match: {} vs {}",
                self.option.quantity, position.option.quantity
            ))
            .into());
        }

        // Check epic
        if self.epic != position.epic {
            return Err(PositionError::invalid_position(&format!(
                "Epics do not match: {:?} vs {:?}",
                self.epic, position.epic
            ))
            .into());
        }

        // Calculate PnL as the difference between positions
        let self_pnl = self.calculate_pnl(
            &self.option.underlying_price,
            self.option.expiration_date,
            &self.option.implied_volatility,
        )?;
        let other_pnl = position.calculate_pnl(
            &position.option.underlying_price,
            position.option.expiration_date,
            &position.option.implied_volatility,
        )?;

        let realized_diff = match (self_pnl.realized, other_pnl.realized) {
            (Some(self_realized), Some(other_realized)) => Some(self_realized - other_realized),
            _ => None,
        };

        let unrealized_diff = match (self_pnl.unrealized, other_pnl.unrealized) {
            (Some(self_unrealized), Some(other_unrealized)) => {
                Some(self_unrealized - other_unrealized)
            }
            _ => None,
        };

        let cost_diff = self_pnl.initial_costs.to_dec() - other_pnl.initial_costs.to_dec();
        let income_diff = self_pnl.initial_income.to_dec() - other_pnl.initial_income.to_dec();

        let initial_costs = Positive::new(cost_diff.abs().to_f64().ok_or_else(|| {
            PricingError::method_error(
                "pnl_diff",
                "initial_costs: cost_diff Decimal cannot be represented as f64",
            )
        })?)
        .map_err(|_| {
            PricingError::method_error("pnl_diff", "initial_costs value is not strictly positive")
        })?;

        let initial_income = Positive::new(income_diff.abs().to_f64().ok_or_else(|| {
            PricingError::method_error(
                "pnl_diff",
                "initial_income: income_diff Decimal cannot be represented as f64",
            )
        })?)
        .map_err(|_| {
            PricingError::method_error("pnl_diff", "initial_income value is not strictly positive")
        })?;

        Ok(PnL {
            realized: realized_diff,
            unrealized: unrealized_diff,
            initial_costs,
            initial_income,
            date_time: self.date,
        })
    }
}

impl TransactionAble for Position {
    /// `Position` does not track an internal transaction history.
    /// This impl is intentionally unsupported; callers that need
    /// transaction tracking should store transactions in a
    /// higher-level container.
    ///
    /// # Errors
    ///
    /// Always returns a `TransactionError` — this operation is
    /// intentionally unsupported on `Position`.
    fn add_transaction(&mut self, _transaction: Transaction) -> Result<(), TransactionError> {
        Err(TransactionError::not_implemented(
            "add_transaction",
            "Position",
        ))
    }

    /// See [`Self::add_transaction`] — this method is intentionally
    /// unsupported on `Position` and always errors.
    ///
    /// # Errors
    ///
    /// Always returns a `TransactionError::NotImplemented` —
    /// this operation is intentionally unsupported on `Position`.
    fn get_transactions(&self) -> Result<Vec<Transaction>, TransactionError> {
        Err(TransactionError::not_implemented(
            "get_transactions",
            "Position",
        ))
    }
}

#[cfg(test)]
mod tests_options_pnl_calculator {
    use super::*;
    use crate::model::types::OptionStyle;
    use crate::model::utils::create_sample_option_simplest;
    use positive::pos_or_panic;

    /// A day count no calendar instant can hold is a value a caller can build
    /// and hand to any public method. Both P&L entry points resolved it
    /// through `ExpirationDate::get_date`, which aborts on the overflow
    /// rather than reporting it, so the `Result` these return could never be
    /// reached for that input.
    #[test]
    fn test_pnl_reports_an_unrepresentable_expiration_instead_of_aborting() {
        let mut option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        option.expiration_date = ExpirationDate::Days(pos_or_panic!(1_000_000_000.0));

        assert!(
            option
                .calculate_pnl_at_expiration(&Positive::HUNDRED)
                .is_err()
        );
        assert!(
            option
                .calculate_pnl(
                    &Positive::HUNDRED,
                    ExpirationDate::Days(pos_or_panic!(1_000_000_000.0)),
                    &pos_or_panic!(0.2),
                )
                .is_err()
        );
    }
}

#[cfg(test)]
mod tests_transaction_able_default {
    use super::*;
    use crate::model::TradeStatus;
    use crate::model::types::{OptionStyle, OptionType, Side};
    use crate::model::utils::create_sample_option_simplest;
    use chrono::Utc;
    use positive::pos_or_panic;

    fn make_position() -> Position {
        let option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        Position::new(
            option,
            pos_or_panic!(5.25),
            Utc::now(),
            pos_or_panic!(0.65),
            pos_or_panic!(0.65),
            None,
            None,
        )
    }

    fn make_transaction() -> Transaction {
        Transaction::new(
            TradeStatus::Open,
            None,
            OptionType::European,
            Side::Long,
            OptionStyle::Call,
            pos_or_panic!(1.0),
            pos_or_panic!(5.25),
            pos_or_panic!(0.65),
            None,
            None,
            None,
        )
    }

    #[test]
    fn test_add_transaction_returns_not_implemented() {
        let mut position = make_position();
        let result = position.add_transaction(make_transaction());
        match result {
            Err(TransactionError::NotImplemented { method, type_name }) => {
                assert_eq!(method, "add_transaction");
                assert_eq!(type_name, "Position");
            }
            other => panic!("expected NotImplemented, got {other:?}"),
        }
    }

    #[test]
    fn test_get_transactions_returns_not_implemented() {
        let position = make_position();
        let result = position.get_transactions();
        match result {
            Err(TransactionError::NotImplemented { method, type_name }) => {
                assert_eq!(method, "get_transactions");
                assert_eq!(type_name, "Position");
            }
            other => panic!("expected NotImplemented, got {other:?}"),
        }
    }
}

#[cfg(test)]
mod tests_pnl_calculator {
    use super::*;
    use crate::model::types::OptionStyle;
    use crate::{OptionType, assert_decimal_eq};
    use chrono::Utc;
    use positive::pos_or_panic;
    use rust_decimal_macros::dec;

    fn setup_test_position(side: Side, option_style: OptionStyle) -> Position {
        let option = Options::new(
            OptionType::European,
            side,
            "AAPL".to_string(),
            Positive::HUNDRED,
            ExpirationDate::Days(pos_or_panic!(30.0)),
            pos_or_panic!(0.2),
            Positive::ONE,
            Positive::HUNDRED,
            dec!(0.05),
            option_style,
            Positive::ZERO,
            None,
        );

        Position::new(
            option,
            pos_or_panic!(5.0),
            Utc::now(),
            Positive::ONE,
            Positive::ONE,
            None,
            None,
        )
    }

    #[test]
    fn test_calculate_pnl_long_call_no_changes() {
        let position = setup_test_position(Side::Long, OptionStyle::Call);
        let pnl = position
            .calculate_pnl(
                &Positive::HUNDRED,
                ExpirationDate::Days(pos_or_panic!(30.0)),
                &pos_or_panic!(0.2),
            )
            .unwrap();

        assert_eq!(pnl.unrealized.unwrap(), Decimal::ZERO); // 5.0 - 2.4933 - 2.0 (fees)
        assert_eq!(position.total_cost().unwrap(), 7.0);
        assert_eq!(position.premium_received().unwrap(), 0.0);
    }

    #[test]
    fn test_calculate_pnl_long_call_price_up() {
        let position = setup_test_position(Side::Long, OptionStyle::Call);
        let pnl = position
            .calculate_pnl(
                &pos_or_panic!(107.0),
                ExpirationDate::Days(pos_or_panic!(30.0)),
                &pos_or_panic!(0.2),
            )
            .unwrap();

        assert_decimal_eq!(pnl.unrealized.unwrap(), dec!(5.2150), dec!(0.0001));
    }

    #[test]
    fn test_calculate_pnl_long_call_vol_down() {
        let position = setup_test_position(Side::Long, OptionStyle::Call);
        let pnl = position
            .calculate_pnl(
                &Positive::HUNDRED,
                ExpirationDate::Days(pos_or_panic!(30.0)),
                &pos_or_panic!(0.1),
            )
            .unwrap();

        assert_decimal_eq!(pnl.unrealized.unwrap(), dec!(-1.1352), dec!(0.0001));
    }

    #[test]
    fn test_calculate_pnl_long_call_date_closer() {
        let position = setup_test_position(Side::Long, OptionStyle::Call);
        let pnl = position
            .calculate_pnl(
                &Positive::HUNDRED,
                ExpirationDate::Days(pos_or_panic!(3.0)),
                &pos_or_panic!(0.2),
            )
            .unwrap();

        assert_decimal_eq!(pnl.unrealized.unwrap(), dec!(-1.7494), dec!(0.0001));
    }

    #[test]
    fn test_calculate_pnl_short_call_no_changes() {
        let position = setup_test_position(Side::Short, OptionStyle::Call);
        let pnl = position
            .calculate_pnl(
                &Positive::HUNDRED,
                ExpirationDate::Days(pos_or_panic!(30.0)),
                &pos_or_panic!(0.2),
            )
            .unwrap();

        assert_eq!(pnl.unrealized.unwrap(), Decimal::ZERO); // 5.0 - 2.4933 - 2.0 (fees)
        assert_eq!(position.total_cost().unwrap(), 2.0);
        assert_eq!(position.premium_received().unwrap(), 5.0);
    }

    #[test]
    fn test_calculate_pnl_short_call_price_up() {
        let position = setup_test_position(Side::Short, OptionStyle::Call);
        let pnl = position
            .calculate_pnl(
                &pos_or_panic!(107.0),
                ExpirationDate::Days(pos_or_panic!(30.0)),
                &pos_or_panic!(0.2),
            )
            .unwrap();

        assert_decimal_eq!(pnl.unrealized.unwrap(), dec!(-5.2150), dec!(0.0001));
    }

    #[test]
    fn test_calculate_pnl_short_call_price_down() {
        let position = setup_test_position(Side::Short, OptionStyle::Call);
        let pnl = position
            .calculate_pnl(
                &pos_or_panic!(97.0),
                ExpirationDate::Days(pos_or_panic!(30.0)),
                &pos_or_panic!(0.2),
            )
            .unwrap();

        assert_decimal_eq!(pnl.unrealized.unwrap(), dec!(1.3069), dec!(0.0001));
    }

    #[test]
    fn test_calculate_pnl_short_call_vol_down() {
        let position = setup_test_position(Side::Short, OptionStyle::Call);
        let pnl = position
            .calculate_pnl(
                &Positive::HUNDRED,
                ExpirationDate::Days(pos_or_panic!(30.0)),
                &pos_or_panic!(0.1),
            )
            .unwrap();

        assert_decimal_eq!(pnl.unrealized.unwrap(), dec!(1.1352), dec!(0.0001));
    }

    #[test]
    fn test_calculate_pnl_short_call_vol_up() {
        let position = setup_test_position(Side::Short, OptionStyle::Call);
        let pnl = position
            .calculate_pnl(
                &Positive::HUNDRED,
                ExpirationDate::Days(pos_or_panic!(30.0)),
                &pos_or_panic!(0.3),
            )
            .unwrap();

        assert_decimal_eq!(pnl.unrealized.unwrap(), dec!(-1.1386), dec!(0.0001));
    }

    #[test]
    fn test_calculate_pnl_short_call_date_closer() {
        let position = setup_test_position(Side::Short, OptionStyle::Call);
        let pnl = position
            .calculate_pnl(
                &Positive::HUNDRED,
                ExpirationDate::Days(pos_or_panic!(3.0)),
                &pos_or_panic!(0.2),
            )
            .unwrap();

        assert_decimal_eq!(pnl.unrealized.unwrap(), dec!(1.7494), dec!(0.0001));
    }

    #[test]
    fn test_calculate_pnl_short_call_date_further() {
        let position = setup_test_position(Side::Short, OptionStyle::Call);
        let pnl = position
            .calculate_pnl(
                &Positive::HUNDRED,
                ExpirationDate::Days(pos_or_panic!(40.0)),
                &pos_or_panic!(0.2),
            )
            .unwrap();

        assert_decimal_eq!(pnl.unrealized.unwrap(), dec!(-0.4224), dec!(0.0001));
    }

    #[test]
    fn test_calculate_pnl_at_expiration_long_call() {
        let position = setup_test_position(Side::Long, OptionStyle::Call);
        let pnl = position
            .calculate_pnl_at_expiration(&pos_or_panic!(110.0))
            .unwrap();

        assert_eq!(pnl.realized.unwrap(), dec!(3.0)); // 10.0 - 7.0 (total cost)
        assert_eq!(position.total_cost().unwrap(), 7.0);
        assert_eq!(position.premium_received().unwrap(), 0.0);
    }

    #[test]
    fn test_calculate_pnl_at_expiration_short_put() {
        let position = setup_test_position(Side::Short, OptionStyle::Put);
        let pnl = position
            .calculate_pnl_at_expiration(&pos_or_panic!(90.0))
            .unwrap();

        assert_eq!(pnl.realized.unwrap(), dec!(-7.0)); // -10.0 + 5.0 (premium) - 2.0 (fees)
        assert_eq!(position.total_cost().unwrap(), 2.0);
        assert_eq!(position.premium_received().unwrap(), 5.0);
    }

    fn setup_test_position_with_epic(
        side: Side,
        option_style: OptionStyle,
        epic: &str,
    ) -> Position {
        let option = Options::new(
            OptionType::European,
            side,
            "AAPL".to_string(),
            Positive::HUNDRED,
            ExpirationDate::Days(pos_or_panic!(30.0)),
            pos_or_panic!(0.2),
            Positive::ONE,
            Positive::HUNDRED,
            dec!(0.05),
            option_style,
            Positive::ZERO,
            None,
        );

        Position::new(
            option,
            pos_or_panic!(5.0),
            Utc::now(),
            Positive::ONE,
            Positive::ONE,
            Some(epic.to_string()),
            None,
        )
    }

    #[test]
    fn test_from_position_pnl_compatible_positions() {
        let position1 = setup_test_position_with_epic(Side::Long, OptionStyle::Call, "EPIC123");
        let position2 = setup_test_position_with_epic(Side::Long, OptionStyle::Call, "EPIC123");

        let result = position1.diff_position_pnl(&position2);
        assert!(result.is_ok());
    }

    #[test]
    fn test_from_position_pnl_different_option_type() {
        let mut position1 = setup_test_position_with_epic(Side::Long, OptionStyle::Call, "EPIC123");
        let mut position2 = setup_test_position_with_epic(Side::Long, OptionStyle::Call, "EPIC123");

        position1.option.option_type = OptionType::European;
        position2.option.option_type = OptionType::American;

        let result = position1.diff_position_pnl(&position2);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Option types do not match")
        );
    }

    #[test]
    fn test_from_position_pnl_different_side() {
        let position1 = setup_test_position_with_epic(Side::Long, OptionStyle::Call, "EPIC123");
        let position2 = setup_test_position_with_epic(Side::Short, OptionStyle::Call, "EPIC123");

        let result = position1.diff_position_pnl(&position2);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Sides do not match")
        );
    }

    #[test]
    fn test_from_position_pnl_different_underlying_symbol() {
        let mut position1 = setup_test_position_with_epic(Side::Long, OptionStyle::Call, "EPIC123");
        let mut position2 = setup_test_position_with_epic(Side::Long, OptionStyle::Call, "EPIC123");

        position1.option.underlying_symbol = "AAPL".to_string();
        position2.option.underlying_symbol = "MSFT".to_string();

        let result = position1.diff_position_pnl(&position2);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Underlying symbols do not match")
        );
    }

    #[test]
    fn test_from_position_pnl_different_strike_price() {
        let mut position1 = setup_test_position_with_epic(Side::Long, OptionStyle::Call, "EPIC123");
        let mut position2 = setup_test_position_with_epic(Side::Long, OptionStyle::Call, "EPIC123");

        position1.option.strike_price = Positive::HUNDRED;
        position2.option.strike_price = pos_or_panic!(105.0);

        let result = position1.diff_position_pnl(&position2);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Strike prices do not match")
        );
    }

    #[test]
    fn test_from_position_pnl_different_expiration_date() {
        let mut position1 = setup_test_position_with_epic(Side::Long, OptionStyle::Call, "EPIC123");
        let mut position2 = setup_test_position_with_epic(Side::Long, OptionStyle::Call, "EPIC123");

        position1.option.expiration_date = ExpirationDate::Days(pos_or_panic!(30.0));
        position2.option.expiration_date = ExpirationDate::Days(pos_or_panic!(60.0));

        let result = position1.diff_position_pnl(&position2);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Expiration dates do not match")
        );
    }

    #[test]
    fn test_from_position_pnl_different_quantity() {
        let mut position1 = setup_test_position_with_epic(Side::Long, OptionStyle::Call, "EPIC123");
        let mut position2 = setup_test_position_with_epic(Side::Long, OptionStyle::Call, "EPIC123");

        position1.option.quantity = Positive::ONE;
        position2.option.quantity = Positive::TWO;

        let result = position1.diff_position_pnl(&position2);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Quantities do not match")
        );
    }

    #[test]
    fn test_from_position_pnl_different_epic() {
        let position1 = setup_test_position_with_epic(Side::Long, OptionStyle::Call, "EPIC123");
        let position2 = setup_test_position_with_epic(Side::Long, OptionStyle::Call, "EPIC456");

        let result = position1.diff_position_pnl(&position2);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Epics do not match")
        );
    }

    #[test]
    fn test_from_position_pnl_calculation() {
        let position1 = setup_test_position_with_epic(Side::Long, OptionStyle::Call, "EPIC123");
        let position2 = setup_test_position_with_epic(Side::Long, OptionStyle::Call, "EPIC123");

        let result = position1.diff_position_pnl(&position2);
        assert!(result.is_ok());

        let pnl = result.unwrap();
        // Since both positions are identical, the PnL difference should be zero
        assert_eq!(pnl.realized, Some(Decimal::ZERO));
        assert_eq!(pnl.unrealized, Some(Decimal::ZERO));
        assert_eq!(pnl.initial_costs, Positive::ZERO);
        assert_eq!(pnl.initial_income, Positive::ZERO);
    }

    fn setup_test_position_with_premium(
        side: Side,
        option_style: OptionStyle,
        epic: &str,
        premium: Positive,
    ) -> Position {
        let option = Options::new(
            OptionType::European,
            side,
            "AAPL".to_string(),
            Positive::HUNDRED,
            ExpirationDate::Days(pos_or_panic!(30.0)),
            pos_or_panic!(0.2),
            Positive::ONE,
            Positive::HUNDRED,
            dec!(0.05),
            option_style,
            Positive::ZERO,
            None,
        );

        Position::new(
            option,
            premium,
            Utc::now(),
            Positive::ONE,
            Positive::ONE,
            Some(epic.to_string()),
            None,
        )
    }

    #[test]
    fn test_from_position_pnl_short_call() {
        let position1 = setup_test_position_with_premium(
            Side::Short,
            OptionStyle::Call,
            "EPIC123",
            pos_or_panic!(5.0),
        );
        let position2 = setup_test_position_with_premium(
            Side::Short,
            OptionStyle::Call,
            "EPIC123",
            pos_or_panic!(3.0),
        );

        let result = position1.diff_position_pnl(&position2);
        assert!(result.is_ok());

        let pnl = result.unwrap();
        // For short positions: premium difference should be positive (received more premium initially)
        // PnL = (premium1 - premium2) = (5.0 - 3.0) = 2.0
        assert_eq!(pnl.realized, Some(dec!(2.0)));
    }

    #[test]
    fn test_from_position_pnl_long_call() {
        let position1 = setup_test_position_with_premium(
            Side::Long,
            OptionStyle::Call,
            "EPIC123",
            pos_or_panic!(5.0),
        );
        let position2 = setup_test_position_with_premium(
            Side::Long,
            OptionStyle::Call,
            "EPIC123",
            pos_or_panic!(3.0),
        );

        let result = position1.diff_position_pnl(&position2);
        assert!(result.is_ok());

        let pnl = result.unwrap();
        // For long positions: premium difference should be negative (paid more premium initially)
        // PnL = -(premium1 - premium2) = -(5.0 - 3.0) = -2.0
        assert_eq!(pnl.realized, Some(dec!(-2.0)));
    }

    #[test]
    fn test_from_position_pnl_short_put() {
        let position1 = setup_test_position_with_premium(
            Side::Short,
            OptionStyle::Put,
            "EPIC123",
            pos_or_panic!(4.0),
        );
        let position2 = setup_test_position_with_premium(
            Side::Short,
            OptionStyle::Put,
            "EPIC123",
            pos_or_panic!(2.5),
        );

        let result = position1.diff_position_pnl(&position2);
        assert!(result.is_ok());

        let pnl = result.unwrap();
        // For short positions: premium difference should be positive (received more premium initially)
        // PnL = (premium1 - premium2) = (4.0 - 2.5) = 1.5
        assert_eq!(pnl.realized, Some(dec!(1.5)));
    }

    #[test]
    fn test_from_position_pnl_long_put() {
        let position1 = setup_test_position_with_premium(
            Side::Long,
            OptionStyle::Put,
            "EPIC123",
            pos_or_panic!(4.0),
        );
        let position2 = setup_test_position_with_premium(
            Side::Long,
            OptionStyle::Put,
            "EPIC123",
            pos_or_panic!(2.5),
        );

        let result = position1.diff_position_pnl(&position2);
        assert!(result.is_ok());

        let pnl = result.unwrap();
        // For long positions: premium difference should be negative (paid more premium initially)
        // PnL = -(premium1 - premium2) = -(4.0 - 2.5) = -1.5
        assert_eq!(pnl.realized, Some(dec!(-1.5)));
    }
}
