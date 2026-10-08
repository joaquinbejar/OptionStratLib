/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 16/8/24
******************************************************************************/

pub use crate::pnl::PnLCalculator;
use chrono::{DateTime, Utc};
use optionstratlib_core::error::DecimalError;
use optionstratlib_core::error::TradeError;
use optionstratlib_core::model::Positive;
use optionstratlib_core::model::Trade;
use optionstratlib_core::model::decimal::d_add;
use optionstratlib_core::{impl_json_debug_pretty, impl_json_display};
use optionstratlib_pricing::error::PricingError;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Represents the Profit and Loss (PnL) of a financial instrument.
///
/// This structure captures the financial performance details of an investment or trading position,
/// including both realized and unrealized gains or losses, as well as the initial costs and income
/// associated with the position.
///
/// PnL serves as a fundamental measurement of trading performance, providing a comprehensive view
/// of the current financial status of positions. It is particularly useful for options trading,
/// portfolio management, and financial reporting.
#[derive(Clone, Serialize, Deserialize, PartialEq, Default)]
#[cfg_attr(feature = "schema", derive(utoipa::ToSchema))]
pub struct PnL {
    /// The realized profit or loss that has been crystallized through closed positions.
    /// This represents actual gains or losses that have been confirmed by completing the trade.
    pub realized: Option<Decimal>,

    /// The unrealized profit or loss representing the current market value compared to entry price.
    /// This value fluctuates with market movements and represents potential gains or losses if
    /// the position were to be closed at current market prices.
    pub unrealized: Option<Decimal>,

    /// The initial costs associated with entering the position, such as fees, commissions,
    /// or premiums paid when buying options.
    pub initial_costs: Positive,

    /// The initial income received when entering the position, such as premiums collected
    /// when selling options or other upfront payments received.
    pub initial_income: Positive,

    /// The timestamp when this PnL calculation was performed.
    /// Useful for tracking performance over time and creating historical PnL reports.
    pub date_time: DateTime<Utc>,
}

impl_json_debug_pretty!(PnL);
impl_json_display!(PnL);

impl PnL {
    /// Creates a new Profit and Loss (PnL) instance.
    ///
    /// This constructor initializes a new PnL object with information about the financial
    /// performance of a trading position, including both realized and unrealized components.
    ///
    /// # Parameters
    ///
    /// * `realized` - The confirmed profit or loss from closed positions, if available.
    ///   This represents actual gains or losses that have been crystallized through completed trades.
    ///
    /// * `unrealized` - The potential profit or loss based on current market values, if available.
    ///   This value represents the theoretical gain or loss if the position were closed at current prices.
    ///
    /// * `initial_costs` - The costs associated with entering the position, such as premiums paid,
    ///   commissions, or fees. Always represented as a positive value.
    ///
    /// * `initial_income` - The income received when entering the position, such as premiums
    ///   collected when selling options. Always represented as a positive value.
    ///
    /// * `date_time` - The timestamp when this PnL calculation was performed, useful for
    ///   tracking performance over time and creating historical reports.
    ///
    /// # Returns
    ///
    /// A new `PnL` instance containing the provided financial performance data.
    ///
    /// # Example
    ///
    /// ```rust
    /// use chrono::Utc;
    /// use rust_decimal_macros::dec;
    /// use optionstratlib_analytics::pnl::utils::PnL;
    /// use optionstratlib_core::{model::Positive, pos_or_panic};
    ///
    /// let pnl = PnL::new(
    ///     Some(dec!(500.0)),  // Realized PnL
    ///     Some(dec!(250.0)),  // Unrealized PnL
    ///     Positive::HUNDRED,        // Initial costs
    ///     pos_or_panic!(350.0),        // Initial income
    ///     Utc::now(),         // Current timestamp
    /// );
    /// ```
    #[inline]
    #[must_use]
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

    /// Calculates the total P&L by summing realized and unrealized components.
    ///
    /// # Returns
    ///
    /// The total P&L as an `Option<Decimal>`. Returns `None` if both realized
    /// and unrealized are `None`, otherwise returns the sum of available values.
    ///
    /// # Example
    ///
    /// ```rust
    /// use chrono::Utc;
    /// use rust_decimal_macros::dec;
    /// use optionstratlib_analytics::pnl::utils::PnL;
    /// use optionstratlib_core::{model::Positive, pos_or_panic};
    ///
    /// let pnl = PnL::new(
    ///     Some(dec!(500.0)),
    ///     Some(dec!(250.0)),
    ///     Positive::HUNDRED,
    ///     pos_or_panic!(350.0),
    ///     Utc::now(),
    /// );
    ///
    /// assert_eq!(pnl.total_pnl(), Some(dec!(750.0)));
    /// ```
    #[inline]
    #[must_use]
    pub fn total_pnl(&self) -> Option<Decimal> {
        match (self.realized, self.unrealized) {
            // `checked_add` keeps the `Option` honest: a total outside the
            // `Decimal` range is not a number this type can report, and `None`
            // already means "no total available" to every caller.
            (Some(r), Some(u)) => r.checked_add(u),
            (Some(r), None) => Some(r),
            (None, Some(u)) => Some(u),
            (None, None) => None,
        }
    }

    /// Adds two `PnL` values, reporting an overflow instead of aborting.
    ///
    /// Realized and unrealized totals add as `Decimal`s (a side with no
    /// value takes the other's), costs and incomes add as `Positive`s, and
    /// the later timestamp is kept. To total a sequence, fold with it:
    /// `items.iter().try_fold(PnL::default(), |acc, item| acc.try_add(item))`.
    ///
    /// # Decision (#471, #788)
    ///
    /// This used to sit beside `impl Add` / `impl Sum` for `PnL`. `std` fixes
    /// both operators to return `Self`, so they could only abort on an
    /// overflowing total; #788 removed them and this method is the one way
    /// to add `PnL` values.
    ///
    /// # Errors
    ///
    /// Returns [`PricingError::Positive`] when either running cost total
    /// leaves the `Positive` range, and [`PricingError::Decimal`] when a
    /// realized or unrealized total leaves the `Decimal` range.
    ///
    /// # Example
    ///
    /// ```rust
    /// use chrono::Utc;
    /// use rust_decimal_macros::dec;
    /// use optionstratlib_analytics::pnl::utils::PnL;
    /// use optionstratlib_core::model::Positive;
    ///
    /// let now = Utc::now();
    /// let a = PnL::new(Some(dec!(500.0)), None, Positive::HUNDRED, Positive::ZERO, now);
    /// let b = PnL::new(Some(dec!(250.0)), None, Positive::HUNDRED, Positive::ZERO, now);
    ///
    /// let total = a.try_add(&b)?;
    /// assert_eq!(total.realized, Some(dec!(750.0)));
    /// # Ok::<(), optionstratlib_pricing::error::PricingError>(())
    /// ```
    pub fn try_add(&self, other: &PnL) -> Result<PnL, PricingError> {
        fn add_leg(
            a: Option<Decimal>,
            b: Option<Decimal>,
            op: &'static str,
        ) -> Result<Option<Decimal>, DecimalError> {
            match (a, b) {
                (Some(a), Some(b)) => Ok(Some(d_add(a, b, op)?)),
                (Some(a), None) => Ok(Some(a)),
                (None, Some(b)) => Ok(Some(b)),
                (None, None) => Ok(None),
            }
        }

        Ok(PnL {
            realized: add_leg(self.realized, other.realized, "PnL::try_add/realized")?,
            unrealized: add_leg(self.unrealized, other.unrealized, "PnL::try_add/unrealized")?,
            initial_costs: self.initial_costs.checked_add(&other.initial_costs)?,
            initial_income: self.initial_income.checked_add(&other.initial_income)?,
            date_time: if self.date_time > other.date_time {
                self.date_time
            } else {
                other.date_time
            },
        })
    }
}

/// The P&L a trade realises on entry: its net cash flow, with its cost and
/// income.
///
/// This was `From<Trade>` until the trade's monetary helpers became checked
/// (#765): [`Trade::cost`], [`Trade::income`] and [`Trade::net`] multiply
/// public `Positive` fields and can overflow.
impl TryFrom<Trade> for PnL {
    type Error = TradeError;

    fn try_from(value: Trade) -> Result<Self, Self::Error> {
        PnL::try_from(&value)
    }
}

/// The borrowed form of `PnL::try_from(trade)`.
impl TryFrom<&Trade> for PnL {
    type Error = TradeError;

    fn try_from(value: &Trade) -> Result<Self, Self::Error> {
        Ok(PnL {
            realized: Some(value.net()?),
            unrealized: None,
            initial_costs: value.cost()?,
            initial_income: value.income()?,
            date_time: value.datetime(),
        })
    }
}

#[cfg(test)]
mod tests_sum {
    use super::*;
    use optionstratlib_core::pos_or_panic;

    use rust_decimal_macros::dec;

    /// Totals a sequence the way callers do now that `impl Sum` is gone (#788).
    fn total(items: &[PnL]) -> PnL {
        items
            .iter()
            .try_fold(PnL::default(), |acc, item| acc.try_add(item))
            .expect("operands are well inside the Decimal range")
    }

    #[test]
    fn test_pnl_sum() {
        let pnl1 = PnL {
            realized: Some(dec!(10.0)),
            unrealized: Some(dec!(5.0)),
            initial_costs: Positive::TWO,
            initial_income: Positive::ONE,
            date_time: Utc::now(),
        };

        let pnl2 = PnL {
            realized: Some(dec!(20.0)),
            unrealized: Some(dec!(10.0)),
            initial_costs: pos_or_panic!(3.0),
            initial_income: Positive::TWO,
            date_time: Utc::now(),
        };

        let sum = total(&[pnl1.clone(), pnl2.clone()]);

        assert_eq!(sum.realized, Some(dec!(30.0)));
        assert_eq!(sum.unrealized, Some(dec!(15.0)));
        assert_eq!(sum.initial_costs, pos_or_panic!(5.0));
        assert_eq!(sum.initial_income, pos_or_panic!(3.0));
    }

    #[test]
    fn test_pnl_sum_both_none() {
        let pnl1 = PnL {
            realized: None,
            unrealized: None,
            initial_costs: Positive::TWO,
            initial_income: Positive::ONE,
            date_time: Utc::now(),
        };

        let pnl2 = PnL {
            realized: None,
            unrealized: None,
            initial_costs: pos_or_panic!(3.0),
            initial_income: Positive::TWO,
            date_time: Utc::now(),
        };

        let sum = total(&[pnl1, pnl2]);

        assert_eq!(sum.realized, None);
        assert_eq!(sum.unrealized, None);
        assert_eq!(sum.initial_costs, pos_or_panic!(5.0));
        assert_eq!(sum.initial_income, pos_or_panic!(3.0));
    }

    #[test]
    fn test_pnl_sum_with_none() {
        let pnl1 = PnL {
            realized: None,
            unrealized: Some(dec!(5.0)),
            initial_costs: Positive::TWO,
            initial_income: Positive::ONE,
            date_time: Utc::now(),
        };

        let pnl2 = PnL {
            realized: Some(dec!(20.0)),
            unrealized: None,
            initial_costs: pos_or_panic!(3.0),
            initial_income: Positive::TWO,
            date_time: Utc::now(),
        };

        let sum = total(&[pnl1.clone(), pnl2.clone()]);

        assert_eq!(sum.realized, Some(dec!(20.0)));
        assert_eq!(sum.unrealized, Some(dec!(5.0)));
        assert_eq!(sum.initial_costs, pos_or_panic!(5.0));
        assert_eq!(sum.initial_income, pos_or_panic!(3.0));
    }

    #[test]
    fn test_pnl_sum_of_borrowed_values() {
        let pnl1 = PnL {
            realized: Some(dec!(10.0)),
            unrealized: Some(dec!(5.0)),
            initial_costs: Positive::TWO,
            initial_income: Positive::ONE,
            date_time: Utc::now(),
        };

        let pnl2 = PnL {
            realized: Some(dec!(20.0)),
            unrealized: Some(dec!(10.0)),
            initial_costs: pos_or_panic!(3.0),
            initial_income: Positive::TWO,
            date_time: Utc::now(),
        };

        let sum = total(&[pnl1, pnl2]);

        assert_eq!(sum.realized, Some(dec!(30.0)));
        assert_eq!(sum.unrealized, Some(dec!(15.0)));
        assert_eq!(sum.initial_costs, pos_or_panic!(5.0));
        assert_eq!(sum.initial_income, pos_or_panic!(3.0));
    }
}

#[cfg(test)]
mod tests_add {
    use super::*;
    use optionstratlib_core::pos_or_panic;

    use rust_decimal_macros::dec;

    #[test]
    fn test_pnl_add() {
        let pnl1 = PnL {
            realized: Some(dec!(10.0)),
            unrealized: Some(dec!(5.0)),
            initial_costs: Positive::TWO,
            initial_income: Positive::ONE,
            date_time: Utc::now(),
        };

        let pnl2 = PnL {
            realized: Some(dec!(20.0)),
            unrealized: Some(dec!(10.0)),
            initial_costs: pos_or_panic!(3.0),
            initial_income: Positive::TWO,
            date_time: Utc::now(),
        };

        let sum = pnl1.try_add(&pnl2).expect("in range");
        assert_eq!(sum.realized, Some(dec!(30.0)));
        assert_eq!(sum.unrealized, Some(dec!(15.0)));
        assert_eq!(sum.initial_costs, pos_or_panic!(5.0));
        assert_eq!(sum.initial_income, pos_or_panic!(3.0));
    }

    #[test]
    fn test_pnl_add_is_commutative_in_range() {
        let pnl1 = PnL {
            realized: Some(dec!(10.0)),
            unrealized: Some(dec!(5.0)),
            initial_costs: Positive::TWO,
            initial_income: Positive::ONE,
            date_time: Utc::now(),
        };

        let pnl2 = PnL {
            realized: Some(dec!(20.0)),
            unrealized: Some(dec!(10.0)),
            initial_costs: pos_or_panic!(3.0),
            initial_income: Positive::TWO,
            date_time: Utc::now(),
        };

        let sum = pnl2.try_add(&pnl1).expect("in range");
        assert_eq!(sum.realized, Some(dec!(30.0)));
        assert_eq!(sum.unrealized, Some(dec!(15.0)));
        assert_eq!(sum.initial_costs, pos_or_panic!(5.0));
        assert_eq!(sum.initial_income, pos_or_panic!(3.0));
    }

    /// `try_add` keeps the later timestamp and adds every leg.
    #[test]
    fn test_pnl_try_add_in_range() {
        let now = Utc::now();
        let pnl1 = PnL {
            realized: Some(dec!(10.0)),
            unrealized: Some(dec!(5.0)),
            initial_costs: Positive::TWO,
            initial_income: Positive::ONE,
            date_time: now,
        };
        let pnl2 = PnL {
            realized: Some(dec!(20.0)),
            unrealized: Some(dec!(10.0)),
            initial_costs: pos_or_panic!(3.0),
            initial_income: Positive::TWO,
            date_time: now,
        };

        let Ok(sum) = pnl1.try_add(&pnl2) else {
            unreachable!("both operands are well inside the Decimal range")
        };
        assert_eq!(
            sum,
            PnL {
                realized: Some(dec!(30.0)),
                unrealized: Some(dec!(15.0)),
                initial_costs: pos_or_panic!(5.0),
                initial_income: pos_or_panic!(3.0),
                date_time: now,
            }
        );
    }

    /// Where the removed operator aborted, `try_add` reports (#471, #788).
    #[test]
    fn test_pnl_try_add_reports_an_overflowing_total() {
        let now = Utc::now();
        let pnl1 = PnL {
            realized: None,
            unrealized: None,
            initial_costs: Positive::MAX,
            initial_income: Positive::ZERO,
            date_time: now,
        };
        let pnl2 = PnL {
            realized: None,
            unrealized: None,
            initial_costs: Positive::MAX,
            initial_income: Positive::ZERO,
            date_time: now,
        };

        assert!(pnl1.try_add(&pnl2).is_err());
    }
}

#[cfg(test)]
mod tests_total_pnl {
    use super::*;
    use optionstratlib_core::pos_or_panic;

    use rust_decimal_macros::dec;

    #[test]
    fn test_total_pnl_both_some() {
        let pnl = PnL::new(
            Some(dec!(500.0)),
            Some(dec!(250.0)),
            Positive::HUNDRED,
            pos_or_panic!(350.0),
            Utc::now(),
        );

        assert_eq!(pnl.total_pnl(), Some(dec!(750.0)));
    }

    #[test]
    fn test_total_pnl_only_realized() {
        let pnl = PnL::new(
            Some(dec!(300.0)),
            None,
            Positive::HUNDRED,
            pos_or_panic!(200.0),
            Utc::now(),
        );

        assert_eq!(pnl.total_pnl(), Some(dec!(300.0)));
    }

    #[test]
    fn test_total_pnl_only_unrealized() {
        let pnl = PnL::new(
            None,
            Some(dec!(150.0)),
            pos_or_panic!(50.0),
            Positive::HUNDRED,
            Utc::now(),
        );

        assert_eq!(pnl.total_pnl(), Some(dec!(150.0)));
    }

    #[test]
    fn test_total_pnl_both_none() {
        let pnl = PnL::new(None, None, Positive::ZERO, Positive::ZERO, Utc::now());

        assert_eq!(pnl.total_pnl(), None);
    }

    #[test]
    fn test_total_pnl_negative_values() {
        let pnl = PnL::new(
            Some(dec!(-200.0)),
            Some(dec!(-100.0)),
            pos_or_panic!(50.0),
            pos_or_panic!(25.0),
            Utc::now(),
        );

        assert_eq!(pnl.total_pnl(), Some(dec!(-300.0)));
    }

    #[test]
    fn test_total_pnl_mixed_signs() {
        let pnl = PnL::new(
            Some(dec!(500.0)),
            Some(dec!(-200.0)),
            Positive::HUNDRED,
            pos_or_panic!(300.0),
            Utc::now(),
        );

        assert_eq!(pnl.total_pnl(), Some(dec!(300.0)));
    }
}

#[cfg(test)]
mod tests_try_from_trade {
    use super::*;
    use optionstratlib_core::model::TradeStatus;
    use optionstratlib_core::model::types::{Action, OptionStyle, Side};
    use optionstratlib_core::pos_or_panic;
    use rust_decimal_macros::dec;

    fn trade(premium: Positive) -> Trade {
        Trade::new(
            Default::default(),
            Action::Buy,
            Side::Long,
            OptionStyle::Call,
            pos_or_panic!(0.15),
            None,
            pos_or_panic!(180.0),
            Utc::now(),
            pos_or_panic!(3.0),
            premium,
            pos_or_panic!(185.0),
            None,
            TradeStatus::Open,
        )
        .unwrap()
        .with_contract_size(Positive::HUNDRED)
    }

    #[test]
    fn test_pnl_try_from_trade_carries_cost_income_and_net() {
        let trade = trade(pos_or_panic!(2.5));
        let Ok(pnl) = PnL::try_from(&trade) else {
            panic!("a representable trade converts");
        };
        // (2.50 × 100 + 0.15) × 3
        assert_eq!(pnl.initial_costs, pos_or_panic!(750.45));
        assert_eq!(pnl.initial_income, Positive::ZERO);
        assert_eq!(pnl.realized, Some(dec!(-750.45)));
        assert_eq!(pnl.unrealized, None);
        assert_eq!(pnl.date_time, trade.datetime());

        let Ok(owned) = PnL::try_from(trade.clone()) else {
            panic!("a representable trade converts");
        };
        assert_eq!(owned, pnl);
    }

    #[test]
    fn test_pnl_try_from_trade_overflow_is_an_error() {
        assert!(matches!(
            PnL::try_from(trade(Positive::MAX)),
            Err(TradeError::ArithmeticOverflow { .. })
        ));
    }
}
