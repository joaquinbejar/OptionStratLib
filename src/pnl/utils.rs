/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 16/8/24
******************************************************************************/

use crate::error::DecimalError;
use crate::error::PricingError;
use crate::model::Trade;
use crate::model::decimal::d_add;
pub use crate::pnl::PnLCalculator;
use chrono::{DateTime, Utc};
use positive::Positive;
use pretty_simple_display::{DebugPretty, DisplaySimple};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::iter::Sum;
use std::ops::Add;
use utoipa::ToSchema;

/// Represents the Profit and Loss (PnL) of a financial instrument.
///
/// This structure captures the financial performance details of an investment or trading position,
/// including both realized and unrealized gains or losses, as well as the initial costs and income
/// associated with the position.
///
/// PnL serves as a fundamental measurement of trading performance, providing a comprehensive view
/// of the current financial status of positions. It is particularly useful for options trading,
/// portfolio management, and financial reporting.
#[derive(
    DebugPretty, DisplaySimple, Clone, Serialize, Deserialize, PartialEq, Default, ToSchema,
)]
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
    /// use optionstratlib::pnl::utils::PnL;
    /// use positive::{Positive, pos_or_panic};
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
    /// use optionstratlib::pnl::utils::PnL;
    /// use positive::{pos_or_panic, Positive};
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

    /// Checked counterpart of the `Add` operator.
    ///
    /// `impl Add for PnL` returns `Self`, so it has nowhere to report an
    /// overflow and aborts instead. Every accumulation inside the library goes
    /// through this method, which reports it.
    ///
    /// # Decision (issue #471): public, and the operator stays
    ///
    /// `Add::add` and `Sum::sum` are fixed by `std` to return `Self`, so no
    /// fallible form of either exists — there is no signature change that
    /// makes the operator safe. This method was already the crate-internal
    /// accumulation path; making it `pub` gives callers the same route.
    ///
    /// The operator impls are kept rather than removed: deleting them would
    /// break every `a + b` and `.sum()` at a call site that has no overflow
    /// to worry about. Rust does not accept `#[deprecated]` on a trait `impl`
    /// block or on a trait method inside one (`error: #[deprecated]
    /// attribute cannot be used on trait impl blocks`), so the redirection
    /// is documented on the impls themselves rather than emitted by the
    /// compiler.
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
    /// use optionstratlib::pnl::utils::PnL;
    /// use positive::Positive;
    ///
    /// let now = Utc::now();
    /// let a = PnL::new(Some(dec!(500.0)), None, Positive::HUNDRED, Positive::ZERO, now);
    /// let b = PnL::new(Some(dec!(250.0)), None, Positive::HUNDRED, Positive::ZERO, now);
    ///
    /// let total = a.try_add(&b)?;
    /// assert_eq!(total.realized, Some(dec!(750.0)));
    /// # Ok::<(), optionstratlib::error::PricingError>(())
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

/// Sums a sequence of `PnL` values.
///
/// # Deprecated in favour of [`PnL::try_add`]
///
/// `initial_costs` and `initial_income` are `Positive`, and this fold adds
/// them with the raw `+` operator, which aborts on overflow. `Sum::sum` is
/// fixed by `std` to return `Self`, so there is no fallible form of it and
/// the abort cannot be removed from this signature. Fold with
/// [`PnL::try_add`] instead:
///
/// ```rust
/// use optionstratlib::pnl::utils::PnL;
/// # let items: Vec<PnL> = Vec::new();
/// let total = items
///     .iter()
///     .try_fold(PnL::default(), |acc, item| acc.try_add(item))?;
/// # Ok::<(), optionstratlib::error::PricingError>(())
/// ```
///
/// The impl is retained because removing it would break every call site that
/// sums values it already knows to be in range. Rust rejects `#[deprecated]`
/// on a trait `impl` block, so this notice is the only marker available.
impl Sum for PnL {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(PnL::default(), |acc, x| PnL {
            realized: match (acc.realized, x.realized) {
                (Some(a), Some(b)) => Some(a + b),
                (Some(a), None) => Some(a),
                (None, Some(b)) => Some(b),
                (None, None) => None,
            },
            unrealized: match (acc.unrealized, x.unrealized) {
                (Some(a), Some(b)) => Some(a + b),
                (Some(a), None) => Some(a),
                (None, Some(b)) => Some(b),
                (None, None) => None,
            },
            initial_costs: acc.initial_costs + x.initial_costs,
            initial_income: acc.initial_income + x.initial_income,
            date_time: x.date_time, // Tomamos la fecha más reciente
        })
    }
}

/// Sums a sequence of `&PnL` values.
///
/// # Deprecated in favour of [`PnL::try_add`]
///
/// Same defect and same reasoning as [`Sum::sum`] for owned `PnL`: the
/// `Positive` accumulation uses the raw `+` operator and `std` gives
/// `Sum::sum` no error channel. Fold with [`PnL::try_add`] instead.
impl<'a> Sum<&'a PnL> for PnL {
    fn sum<I: Iterator<Item = &'a PnL>>(iter: I) -> Self {
        iter.fold(PnL::default(), |acc, x| PnL {
            realized: match (acc.realized, x.realized) {
                (Some(a), Some(b)) => Some(a + b),
                (Some(a), None) => Some(a),
                (None, Some(b)) => Some(b),
                (None, None) => None,
            },
            unrealized: match (acc.unrealized, x.unrealized) {
                (Some(a), Some(b)) => Some(a + b),
                (Some(a), None) => Some(a),
                (None, Some(b)) => Some(b),
                (None, None) => None,
            },
            initial_costs: acc.initial_costs + x.initial_costs,
            initial_income: acc.initial_income + x.initial_income,
            date_time: x.date_time, // Tomamos la fecha más reciente
        })
    }
}

/// Adds two `PnL` values.
///
/// # Deprecated in favour of [`PnL::try_add`]
///
/// `initial_costs: Positive + Positive` uses the raw `+` operator, which
/// aborts on overflow. `Add::add` is fixed by `std` to return `Self`, so
/// there is no fallible form of it. Use [`PnL::try_add`], which returns
/// `Result<PnL, PricingError>`:
///
/// ```rust
/// use chrono::Utc;
/// use optionstratlib::pnl::utils::PnL;
/// use positive::Positive;
/// use rust_decimal_macros::dec;
///
/// let now = Utc::now();
/// let a = PnL::new(Some(dec!(1.0)), None, Positive::ONE, Positive::ZERO, now);
/// let b = PnL::new(Some(dec!(2.0)), None, Positive::ONE, Positive::ZERO, now);
///
/// let total = a.try_add(&b)?;      // instead of `a + b`
/// assert_eq!(total.realized, Some(dec!(3.0)));
/// # Ok::<(), optionstratlib::error::PricingError>(())
/// ```
///
/// The impl is retained because removing it would break every call site that
/// adds values it already knows to be in range. Rust rejects `#[deprecated]`
/// on a trait `impl` block, so this notice is the only marker available.
impl Add for PnL {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        PnL {
            realized: match (self.realized, other.realized) {
                (Some(a), Some(b)) => Some(a + b),
                (Some(a), None) => Some(a),
                (None, Some(b)) => Some(b),
                (None, None) => None,
            },
            unrealized: match (self.unrealized, other.unrealized) {
                (Some(a), Some(b)) => Some(a + b),
                (Some(a), None) => Some(a),
                (None, Some(b)) => Some(b),
                (None, None) => None,
            },
            initial_costs: self.initial_costs + other.initial_costs,
            initial_income: self.initial_income + other.initial_income,
            date_time: if self.date_time > other.date_time {
                self.date_time
            } else {
                other.date_time
            },
        }
    }
}

/// Adds two `&PnL` values.
///
/// # Deprecated in favour of [`PnL::try_add`]
///
/// Same defect and same reasoning as [`Add::add`] for owned `PnL`: the
/// `Positive` accumulation uses the raw `+` operator and `std` gives
/// `Add::add` no error channel. Use [`PnL::try_add`], which already takes
/// both operands by reference.
impl Add for &PnL {
    type Output = PnL;

    fn add(self, other: Self) -> PnL {
        PnL {
            realized: match (self.realized, other.realized) {
                (Some(a), Some(b)) => Some(a + b),
                (Some(a), None) => Some(a),
                (None, Some(b)) => Some(b),
                (None, None) => None,
            },
            unrealized: match (self.unrealized, other.unrealized) {
                (Some(a), Some(b)) => Some(a + b),
                (Some(a), None) => Some(a),
                (None, Some(b)) => Some(b),
                (None, None) => None,
            },
            initial_costs: self.initial_costs + other.initial_costs,
            initial_income: self.initial_income + other.initial_income,
            date_time: if self.date_time > other.date_time {
                self.date_time
            } else {
                other.date_time
            },
        }
    }
}

impl From<Trade> for PnL {
    fn from(value: Trade) -> Self {
        PnL {
            realized: Some(value.net()),
            unrealized: None,
            initial_costs: value.cost(),
            initial_income: value.income(),
            date_time: value.datetime(),
        }
    }
}

impl From<&Trade> for PnL {
    fn from(value: &Trade) -> Self {
        PnL {
            realized: Some(value.net()),
            unrealized: None,
            initial_costs: value.cost(),
            initial_income: value.income(),
            date_time: value.datetime(),
        }
    }
}

#[cfg(test)]
mod tests_sum {
    use super::*;
    use positive::pos_or_panic;

    use rust_decimal_macros::dec;

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

        let sum: PnL = vec![pnl1.clone(), pnl2.clone()].into_iter().sum();

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

        let sum: PnL = vec![pnl1, pnl2].into_iter().sum();

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

        let sum: PnL = vec![pnl1.clone(), pnl2.clone()].into_iter().sum();

        assert_eq!(sum.realized, Some(dec!(20.0)));
        assert_eq!(sum.unrealized, Some(dec!(5.0)));
        assert_eq!(sum.initial_costs, pos_or_panic!(5.0));
        assert_eq!(sum.initial_income, pos_or_panic!(3.0));
    }

    #[test]
    fn test_pnl_sum_reference() {
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

        let sum: PnL = vec![&pnl1, &pnl2].into_iter().sum();

        assert_eq!(sum.realized, Some(dec!(30.0)));
        assert_eq!(sum.unrealized, Some(dec!(15.0)));
        assert_eq!(sum.initial_costs, pos_or_panic!(5.0));
        assert_eq!(sum.initial_income, pos_or_panic!(3.0));
    }
}

#[cfg(test)]
mod tests_add {
    use super::*;
    use positive::pos_or_panic;

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

        let sum = pnl1 + pnl2;
        assert_eq!(sum.realized, Some(dec!(30.0)));
        assert_eq!(sum.unrealized, Some(dec!(15.0)));
        assert_eq!(sum.initial_costs, pos_or_panic!(5.0));
        assert_eq!(sum.initial_income, pos_or_panic!(3.0));
    }

    #[test]
    fn test_pnl_add_ref() {
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

        let sum = &pnl1 + &pnl2;
        assert_eq!(sum.realized, Some(dec!(30.0)));
        assert_eq!(sum.unrealized, Some(dec!(15.0)));
        assert_eq!(sum.initial_costs, pos_or_panic!(5.0));
        assert_eq!(sum.initial_income, pos_or_panic!(3.0));
    }

    /// `try_add` is the public replacement for the operator: on values the
    /// operator handles it gives the same answer.
    #[test]
    fn test_pnl_try_add_matches_the_operator_in_range() {
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
        assert_eq!(sum, pnl1 + pnl2);
    }

    /// Where the operator aborts, `try_add` reports. This is the whole point
    /// of making it public in #471.
    #[test]
    fn test_pnl_try_add_reports_what_the_operator_aborts_on() {
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
    use positive::pos_or_panic;

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
