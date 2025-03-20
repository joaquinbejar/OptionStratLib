/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 16/8/24
******************************************************************************/
use crate::{ExpirationDate, Positive};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::iter::Sum;
use std::ops::Add;

/// Represents the Profit and Loss (PnL) of a financial instrument.
///
/// This structure captures the financial performance details of an investment or trading position,
/// including both realized and unrealized gains or losses, as well as the initial costs and income
/// associated with the position.
///
/// PnL serves as a fundamental measurement of trading performance, providing a comprehensive view
/// of the current financial status of positions. It is particularly useful for options trading,
/// portfolio management, and financial reporting.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
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
    /// use optionstratlib::pos;
    ///
    /// let pnl = PnL::new(
    ///     Some(dec!(500.0)),  // Realized PnL
    ///     Some(dec!(250.0)),  // Unrealized PnL
    ///     pos!(100.0),        // Initial costs
    ///     pos!(350.0),        // Initial income
    ///     Utc::now(),         // Current timestamp
    /// );
    /// ```
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

/// Defines the interface for profit and loss (PnL) calculation on financial instruments.
///
/// This trait provides methods to calculate the profit and loss of financial instruments
/// (particularly options) under different scenarios: at current market conditions and
/// at expiration. Implementations of this trait can provide specific PnL calculation
/// logic for different types of financial instruments or strategies.
///
/// # Examples
///
/// ```
/// use std::error::Error;
/// use optionstratlib::pnl::utils::{PnL, PnLCalculator};
/// use optionstratlib::{ExpirationDate, Positive};
/// use chrono::Utc;
/// use optionstratlib::pos;
///
/// struct MyOption;
///
/// impl PnLCalculator for MyOption {
///     fn calculate_pnl(
///         &self,
///         underlying_price: &Positive,
///         expiration_date: ExpirationDate,
///         implied_volatility: &Positive,
///     ) -> Result<PnL, Box<dyn Error>> {
///         // Implement PnL calculation logic
///         Ok(PnL::default())
///     }
///
///     fn calculate_pnl_at_expiration(
///         &self,
///         underlying_price: &Positive,
///     ) -> Result<PnL, Box<dyn Error>> {
///         // Implement PnL calculation at expiration
///         Ok(PnL::default())
///     }
/// }
/// ```
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

#[cfg(test)]
mod tests_sum {
    use super::*;
    use crate::pos;
    use rust_decimal_macros::dec;

    #[test]
    fn test_pnl_sum() {
        let pnl1 = PnL {
            realized: Some(dec!(10.0)),
            unrealized: Some(dec!(5.0)),
            initial_costs: pos!(2.0),
            initial_income: pos!(1.0),
            date_time: Utc::now(),
        };

        let pnl2 = PnL {
            realized: Some(dec!(20.0)),
            unrealized: Some(dec!(10.0)),
            initial_costs: pos!(3.0),
            initial_income: pos!(2.0),
            date_time: Utc::now(),
        };

        let sum: PnL = vec![pnl1.clone(), pnl2.clone()].into_iter().sum();

        assert_eq!(sum.realized, Some(dec!(30.0)));
        assert_eq!(sum.unrealized, Some(dec!(15.0)));
        assert_eq!(sum.initial_costs, pos!(5.0));
        assert_eq!(sum.initial_income, pos!(3.0));
    }

    #[test]
    fn test_pnl_sum_with_none() {
        let pnl1 = PnL {
            realized: None,
            unrealized: Some(dec!(5.0)),
            initial_costs: pos!(2.0),
            initial_income: pos!(1.0),
            date_time: Utc::now(),
        };

        let pnl2 = PnL {
            realized: Some(dec!(20.0)),
            unrealized: None,
            initial_costs: pos!(3.0),
            initial_income: pos!(2.0),
            date_time: Utc::now(),
        };

        let sum: PnL = vec![pnl1.clone(), pnl2.clone()].into_iter().sum();

        assert_eq!(sum.realized, Some(dec!(20.0)));
        assert_eq!(sum.unrealized, Some(dec!(5.0)));
        assert_eq!(sum.initial_costs, pos!(5.0));
        assert_eq!(sum.initial_income, pos!(3.0));
    }

    #[test]
    fn test_pnl_sum_reference() {
        let pnl1 = PnL {
            realized: Some(dec!(10.0)),
            unrealized: Some(dec!(5.0)),
            initial_costs: pos!(2.0),
            initial_income: pos!(1.0),
            date_time: Utc::now(),
        };

        let pnl2 = PnL {
            realized: Some(dec!(20.0)),
            unrealized: Some(dec!(10.0)),
            initial_costs: pos!(3.0),
            initial_income: pos!(2.0),
            date_time: Utc::now(),
        };

        let sum: PnL = vec![&pnl1, &pnl2].into_iter().sum();

        assert_eq!(sum.realized, Some(dec!(30.0)));
        assert_eq!(sum.unrealized, Some(dec!(15.0)));
        assert_eq!(sum.initial_costs, pos!(5.0));
        assert_eq!(sum.initial_income, pos!(3.0));
    }
}

#[cfg(test)]
mod tests_add {
    use super::*;
    use crate::pos;
    use rust_decimal_macros::dec;

    #[test]
    fn test_pnl_add() {
        let pnl1 = PnL {
            realized: Some(dec!(10.0)),
            unrealized: Some(dec!(5.0)),
            initial_costs: pos!(2.0),
            initial_income: pos!(1.0),
            date_time: Utc::now(),
        };

        let pnl2 = PnL {
            realized: Some(dec!(20.0)),
            unrealized: Some(dec!(10.0)),
            initial_costs: pos!(3.0),
            initial_income: pos!(2.0),
            date_time: Utc::now(),
        };

        let sum = pnl1 + pnl2;
        assert_eq!(sum.realized, Some(dec!(30.0)));
        assert_eq!(sum.unrealized, Some(dec!(15.0)));
        assert_eq!(sum.initial_costs, pos!(5.0));
        assert_eq!(sum.initial_income, pos!(3.0));
    }

    #[test]
    fn test_pnl_add_ref() {
        let pnl1 = PnL {
            realized: Some(dec!(10.0)),
            unrealized: Some(dec!(5.0)),
            initial_costs: pos!(2.0),
            initial_income: pos!(1.0),
            date_time: Utc::now(),
        };

        let pnl2 = PnL {
            realized: Some(dec!(20.0)),
            unrealized: Some(dec!(10.0)),
            initial_costs: pos!(3.0),
            initial_income: pos!(2.0),
            date_time: Utc::now(),
        };

        let sum = &pnl1 + &pnl2;
        assert_eq!(sum.realized, Some(dec!(30.0)));
        assert_eq!(sum.unrealized, Some(dec!(15.0)));
        assert_eq!(sum.initial_costs, pos!(5.0));
        assert_eq!(sum.initial_income, pos!(3.0));
    }
}
