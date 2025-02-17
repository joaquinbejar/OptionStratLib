/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 16/8/24
******************************************************************************/
use crate::{ExpirationDate, Positive};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use std::error::Error;
use std::iter::Sum;
use std::ops::Add;
use serde::{Deserialize, Serialize};

/// Represents the Profit and Loss (PnL) of a financial instrument.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
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

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
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
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
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
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
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
    use rust_decimal_macros::dec;
    use crate::pos;

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
    use rust_decimal_macros::dec;
    use crate::pos;

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