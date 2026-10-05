/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 25/12/24
******************************************************************************/

//! Extensions for the Positive type specific to OptionStratLib.
//!
//! This module provides additional conversions and trait implementations
//! for the `Positive` type that are specific to the options trading domain.

use crate::model::utils::ToRound;
use positive::Positive;
use rust_decimal::Decimal;

impl ToRound for Positive {
    fn round(&self) -> Decimal {
        Positive::round(self).to_dec()
    }

    fn round_to(&self, decimal_places: u32) -> Decimal {
        Positive::round_to(self, decimal_places).to_dec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use positive::pos_or_panic;

    #[test]
    fn test_to_round_trait() {
        let p = pos_or_panic!(1.567);
        assert_eq!(p.round(), Decimal::from(2));
        assert_eq!(p.round_to(1), Decimal::new(16, 1));
    }
}
