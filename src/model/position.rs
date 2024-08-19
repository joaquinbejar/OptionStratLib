/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 18/8/24
******************************************************************************/
use crate::constants::ZERO;
use crate::greeks::equations::{Greek, Greeks};
use crate::model::option::Options;
use crate::model::types::{ExpirationDate, Side};
use chrono::{DateTime, Utc};

/// The `Position` struct represents a financial position in an options market.
/// It includes various attributes related to the option, such as its cost,
/// date, and associated fees.
///
/// # Attributes
/// - `option`: Represents the options details.
/// - `premium`: The premium paid for the option per contract.
/// - `date`: The date when the position was opened.
/// - `open_fee`: The fee paid to open the position per contract.
/// - `close_fee`: The fee paid to close the position per contract.
///
/// # Methods
/// - `new(option: Options, premium: f64, date: DateTime<Utc>, open_fee: f64, close_fee: f64) -> Self`
///   Creates a new `Position` instance.
/// - `total_cost(&self) -> f64`
///   Calculates the total cost including the premium and open fee.
/// - `unrealized_pnl(&self, current_price: f64) -> f64`
///   Calculates the unrealized profit or loss at the current price.
/// - `realized_pnl(&self, close_price: f64) -> f64`
///   Calculates the realized profit or loss at the closing price.
/// - `days_held(&self) -> i64`
///   Returns the number of days the position has been held.
/// - `days_to_expiration(&self) -> f64`
///   Returns the number of days until the option expires.
/// - `is_long(&self) -> bool`
///   Checks if the position is a long position.
/// - `is_short(&self) -> bool`
///   Checks if the position is a short position.
///
/// The `Greeks` trait is also implemented for the `Position` struct, allowing
/// calculations related to options' sensitivities (e.g., Delta, Gamma).
#[derive(Clone)]
pub struct Position {
    pub option: Options,
    pub premium: f64, // per contract
    pub date: DateTime<Utc>,
    pub open_fee: f64,  // per contract
    pub close_fee: f64, // per contract
}

impl Position {
    pub fn new(
        option: Options,
        premium: f64,
        date: DateTime<Utc>,
        open_fee: f64,
        close_fee: f64,
    ) -> Self {
        let premium = Self::check_premium(premium);

        Position {
            option,
            premium,
            date,
            open_fee,
            close_fee,
        }
    }

    pub fn total_cost(&self) -> f64 {
        self.premium * self.option.quantity as f64 + self.open_fee
    }

    pub fn pnl_at_expiration(&self) -> f64 {
        let intrinsic_value = self.option.intrinsic_value(self.option.underlying_price);
        match self.option.side {
            Side::Long => {
                intrinsic_value
                    - (self.premium + self.open_fee + self.close_fee) * self.option.quantity as f64
            }
            Side::Short => {
                intrinsic_value - (self.open_fee + self.close_fee) * self.option.quantity as f64
                    + self.premium * self.option.quantity as f64
            }
        }
    }

    pub fn unrealized_pnl(&self, current_option_price: f64) -> f64 {
        match self.option.side {
            Side::Long => {
                (current_option_price - self.premium - self.open_fee - self.close_fee)
                    * self.option.quantity as f64
            }
            Side::Short => {
                (self.premium - current_option_price - self.open_fee - self.close_fee)
                    * self.option.quantity as f64
            }
        }
    }

    pub fn days_held(&self) -> i64 {
        (Utc::now() - self.date).num_days()
    }

    pub fn days_to_expiration(&self) -> f64 {
        match self.option.expiration_date {
            ExpirationDate::Days(days) => days,
            ExpirationDate::DateTime(datetime) => {
                datetime.signed_duration_since(Utc::now()).num_days() as f64
            }
        }
    }

    pub fn is_long(&self) -> bool {
        match self.option.side {
            Side::Long => true,
            Side::Short => false,
        }
    }

    pub fn is_short(&self) -> bool {
        match self.option.side {
            Side::Long => false,
            Side::Short => true,
        }
    }

    fn check_premium(mut premium: f64) -> f64 {
        if premium < ZERO {
            premium *= -1.0;
        }
        premium
    }
}

impl Greeks for Position {
    fn greeks(&self) -> Greek {
        self.option.greeks()
    }
}

#[cfg(test)]
mod tests_position {
    use super::*;
    use crate::constants::ZERO;
    use crate::model::types::{ExpirationDate, OptionStyle, OptionType, Side};
    use chrono::Duration;

    fn setup_option(
        side: Side,
        option_style: OptionStyle,
        strike_price: f64,
        underlying_price: f64,
        quantity: u32,
        expiration_days: i64,
    ) -> Options {
        Options {
            option_type: OptionType::European,
            side,
            underlying_symbol: "".to_string(),
            strike_price,
            expiration_date: ExpirationDate::Days(expiration_days as f64),
            implied_volatility: 0.2,
            quantity,
            underlying_price,
            risk_free_rate: 0.01,
            option_style,
            dividend_yield: ZERO,
            exotic_params: None,
        }
    }

    #[test]
    fn test_position_total_cost() {
        let option = setup_option(Side::Long, OptionStyle::Call, 100.0, 105.0, 10, 30);
        let position = Position::new(option, 5.0, Utc::now(), 1.0, 1.0);
        assert_eq!(
            position.total_cost(),
            51.0,
            "Total cost calculation is incorrect."
        );
    }

    #[test]
    fn test_position_check_negative_premium() {
        let option = setup_option(Side::Long, OptionStyle::Call, 100.0, 110.0, 1, 0);
        let position = Position::new(option, -5.0, Utc::now(), 1.0, 1.0);
        assert_eq!(
            position.pnl_at_expiration(),
            3.0,
            "PNL at expiration for long call ITM is incorrect."
        );
    }

    #[test]
    fn test_position_pnl_at_expiration_long_call_itm() {
        let option = setup_option(Side::Long, OptionStyle::Call, 100.0, 110.0, 1, 0);
        let position = Position::new(option, 5.0, Utc::now(), 1.0, 1.0);
        assert_eq!(
            position.pnl_at_expiration(),
            3.0,
            "PNL at expiration for long call ITM is incorrect."
        );
    }

    #[test]
    fn test_position_pnl_at_expiration_long_call_itm_quantity() {
        let option = setup_option(Side::Long, OptionStyle::Call, 100.0, 110.0, 10, 0);
        let position = Position::new(option, 5.0, Utc::now(), 1.0, 1.0);
        assert_eq!(
            position.pnl_at_expiration(),
            30.0,
            "PNL at expiration for long call ITM is incorrect."
        );
    }

    #[test]
    fn test_position_pnl_at_expiration_short_call_itm() {
        let option = setup_option(Side::Short, OptionStyle::Call, 100.0, 110.0, 1, 0);
        let position = Position::new(option, 5.0, Utc::now(), 1.0, 1.0);
        assert_eq!(
            position.pnl_at_expiration(),
            -7.0,
            "PNL at expiration for short call ITM is incorrect."
        );
    }

    #[test]
    fn test_position_pnl_at_expiration_short_call_itm_quantity() {
        let option = setup_option(Side::Short, OptionStyle::Call, 100.0, 110.0, 10, 0);
        let position = Position::new(option, 5.0, Utc::now(), 1.0, 1.0);
        assert_eq!(
            position.pnl_at_expiration(),
            -70.0,
            "PNL at expiration for short call ITM is incorrect."
        );
    }

    #[test]
    fn test_position_pnl_at_expiration_long_put_itm() {
        let option = setup_option(Side::Long, OptionStyle::Put, 100.0, 90.0, 1, 0);
        let position = Position::new(option, 5.0, Utc::now(), 1.0, 1.0);
        assert_eq!(
            position.pnl_at_expiration(),
            3.0,
            "PNL at expiration for long put ITM is incorrect."
        );
    }

    #[test]
    fn test_position_pnl_at_expiration_long_put_itm_quantity() {
        let option = setup_option(Side::Long, OptionStyle::Put, 100.0, 90.0, 10, 0);
        let position = Position::new(option, 5.0, Utc::now(), 1.0, 1.0);
        assert_eq!(
            position.pnl_at_expiration(),
            30.0,
            "PNL at expiration for long put ITM is incorrect."
        );
    }

    #[test]
    fn test_position_pnl_at_expiration_short_put_itm() {
        let option = setup_option(Side::Short, OptionStyle::Put, 100.0, 90.0, 1, 0);
        let position = Position::new(option, 5.0, Utc::now(), 1.0, 1.0);
        assert_eq!(
            position.pnl_at_expiration(),
            -7.0,
            "PNL at expiration for short put ITM is incorrect."
        );
    }

    #[test]
    fn test_position_pnl_at_expiration_short_put_itm_quantity() {
        let option = setup_option(Side::Short, OptionStyle::Put, 100.0, 90.0, 10, 0);
        let position = Position::new(option, 5.0, Utc::now(), 1.0, 1.0);
        assert_eq!(
            position.pnl_at_expiration(),
            -70.0,
            "PNL at expiration for short put ITM is incorrect."
        );
    }

    #[test]
    fn test_position_pnl_at_expiration_short_put_itm_winning() {
        let option = setup_option(Side::Short, OptionStyle::Put, 100.0, 110.0, 1, 0);
        let position = Position::new(option, 5.0, Utc::now(), 1.0, 1.0);
        assert_eq!(
            position.pnl_at_expiration(),
            3.0,
            "PNL at expiration for short put ITM is incorrect."
        );
    }

    #[test]
    fn test_position_pnl_at_expiration_short_put_itm_quantity_winning() {
        let option = setup_option(Side::Short, OptionStyle::Put, 100.0, 110.0, 10, 0);
        let position = Position::new(option, 5.0, Utc::now(), 1.0, 1.0);
        assert_eq!(
            position.pnl_at_expiration(),
            30.0,
            "PNL at expiration for short put ITM is incorrect."
        );
    }

    #[test]
    fn test_unrealized_pnl_long_call() {
        let option = setup_option(Side::Long, OptionStyle::Call, 100.0, 105.0, 1, 30);
        let position = Position::new(option, 5.0, Utc::now(), 1.0, 1.0);
        assert_eq!(
            position.unrealized_pnl(7.0),
            ZERO,
            "Unrealized PNL for long call is incorrect."
        );
    }

    #[test]
    fn test_unrealized_pnl_long_call_quantity() {
        let option = setup_option(Side::Long, OptionStyle::Call, 100.0, 105.0, 10, 30);
        let position = Position::new(option, 5.0, Utc::now(), 1.0, 1.0);
        assert_eq!(
            position.unrealized_pnl(7.0),
            ZERO,
            "Unrealized PNL for long call is incorrect."
        );
    }

    #[test]
    fn test_unrealized_pnl_short_call() {
        let option = setup_option(Side::Short, OptionStyle::Call, 100.0, 105.0, 10, 30);
        let position = Position::new(option, 5.0, Utc::now(), 1.0, 1.0);
        assert_eq!(
            position.unrealized_pnl(3.0),
            ZERO,
            "Unrealized PNL for short call is incorrect."
        );
    }

    #[test]
    fn test_unrealized_pnl_short_call_bis() {
        let option = setup_option(Side::Short, OptionStyle::Call, 100.0, 105.0, 1, 30);
        let position = Position::new(option, 5.0, Utc::now(), 1.0, 1.0);
        assert_eq!(
            position.unrealized_pnl(10.0),
            -7.0,
            "Unrealized PNL for short call is incorrect."
        );
    }

    #[test]
    fn test_days_held() {
        let option = setup_option(Side::Long, OptionStyle::Call, 100.0, 105.0, 10, 30);
        let date = Utc::now() - Duration::days(10);
        let position = Position::new(option, 5.0, date, 1.0, 1.0);
        assert_eq!(
            position.days_held(),
            10,
            "Days held calculation is incorrect."
        );
    }

    #[test]
    fn test_days_to_expiration() {
        let option = setup_option(Side::Long, OptionStyle::Call, 100.0, 105.0, 10, 30);
        let position = Position::new(option, 5.0, Utc::now(), 1.0, 1.0);
        assert_eq!(
            position.days_to_expiration(),
            30.0,
            "Days to expiration calculation is incorrect."
        );
    }

    #[test]
    fn test_is_long_position() {
        let option = setup_option(Side::Long, OptionStyle::Call, 100.0, 105.0, 10, 30);
        let position = Position::new(option, 5.0, Utc::now(), 1.0, 1.0);
        assert!(
            position.is_long(),
            "is_long should return true for long positions."
        );
        assert!(
            !position.is_short(),
            "is_short should return false for long positions."
        );
    }

    #[test]
    fn test_is_short_position() {
        let option = setup_option(Side::Short, OptionStyle::Call, 100.0, 105.0, 10, 30);
        let position = Position::new(option, 5.0, Utc::now(), 1.0, 1.0);
        assert!(
            position.is_short(),
            "is_short should return true for short positions."
        );
        assert!(
            !position.is_long(),
            "is_long should return false for short positions."
        );
    }
}

// #[cfg(test)]
// mod tests_pnl_basic {
//     use super::*;
//
//     #[test]
//     fn test_pnl() {
//         let option = Options::new(
//             OptionType::European,
//             Side::Long,
//             "AAPL".to_string(),
//             100.0,
//             ExpirationDate::Days(30.0),
//             0.2,
//             1,
//             100.0,
//             0.05,
//             OptionStyle::Call,
//             0.01,
//             None,
//         );
//
//         // Test PnL when the asset price is above the strike price
//         assert_eq!(option.pnl(110.0), 5.0); // (110 - 100) - 5 = 5
//
//         // Test PnL when the asset price is below the strike price
//         assert_eq!(option.pnl(90.0), -5.0); // 0 - 5 = -5
//
//         // Test PnL when the asset price is equal to the strike price
//         assert_eq!(option.pnl(100.0), -5.0); // 0 - 5 = -5
//     }
//
//     #[test]
//     fn test_pnl_at_expiration() {
//         let option = Options::new(
//             OptionType::European,
//             Side::Long,
//             "AAPL".to_string(),
//             100.0,
//             ExpirationDate::Days(ZERO), // Expired
//             0.2,
//             1,
//             110.0, // Underlying price at expiration
//             0.05,
//             OptionStyle::Call,
//             0.01,
//             5.0, // premium
//             None,
//         );
//
//         assert_eq!(option.pnl_at_expiration(), 5.0); // (110 - 100) - 5 = 5
//     }
//
//     #[test]
//     fn test_pnl_short_option() {
//         let option = Options::new(
//             OptionType::European,
//             Side::Short,
//             "AAPL".to_string(),
//             100.0,
//             ExpirationDate::Days(30.0),
//             0.2,
//             1,
//             100.0,
//             0.05,
//             OptionStyle::Put,
//             0.01,
//             -5.0, // premium (received)
//             None,
//         );
//
//         // Test PnL when the asset price is above the strike price
//         assert_eq!(option.pnl(110.0), 5.0); // 0 + 5 = 5
//
//         // Test PnL when the asset price is below the strike price
//         assert_eq!(option.pnl(90.0), -5.0); // (100 - 90) - 5 = 5
//     }
//
//     #[test]
//     fn test_pnl_at_expiration_short_option() {
//         let option = Options::new(
//             OptionType::European,
//             Side::Short,
//             "AAPL".to_string(),
//             100.0,
//             ExpirationDate::Days(ZERO), // Expired
//             0.2,
//             1,
//             90.0, // Underlying price at expiration
//             0.05,
//             OptionStyle::Put,
//             0.01,
//             -5.0, // premium (received)
//             None,
//         );
//
//         assert_eq!(option.pnl_at_expiration(), -5.0); // (100 - 90) - 5 = 5
//     }
// }
//
// #[cfg(test)]
// mod tests_pnl_extended {
//     use super::*;
//
//     fn create_option(option_style: OptionStyle, side: Side, premium: f64) -> Options {
//         Options::new(
//             OptionType::European,
//             side,
//             "AAPL".to_string(),
//             100.0,
//             ExpirationDate::Days(30.0),
//             0.2,
//             1,
//             100.0,
//             0.05,
//             option_style,
//             0.01,
//             premium,
//             None,
//         )
//     }
//
//     #[test]
//     fn test_pnl_call_long() {
//         let option = create_option(OptionStyle::Call, Side::Long, 5.0);
//         assert_eq!(option.pnl(110.0), 5.0); // (110 - 100) - 5 = 5
//         assert_eq!(option.pnl(90.0), -5.0); // 0 - 5 = -5
//     }
//
//     #[test]
//     fn test_pnl_at_expiration_call_long() {
//         let mut option = create_option(OptionStyle::Call, Side::Long, 5.0);
//         option.underlying_price = 110.0;
//         assert_eq!(option.pnl_at_expiration(), 5.0); // (110 - 100) - 5 = 5
//     }
//
//     #[test]
//     fn test_pnl_call_short() {
//         let option = create_option(OptionStyle::Call, Side::Short, -5.0);
//         assert_eq!(option.pnl(110.0), -5.0); // -(110 - 100) + 5 = -5
//         assert_eq!(option.pnl(90.0), 5.0); // 0 + 5 = 5
//     }
//
//     #[test]
//     fn test_pnl_at_expiration_call_short() {
//         let mut option = create_option(OptionStyle::Call, Side::Short, -5.0);
//         option.underlying_price = 110.0;
//         assert_eq!(option.pnl_at_expiration(), -5.0); // -(110 - 100) + 5 = -5
//     }
//
//     #[test]
//     fn test_pnl_put_long() {
//         let option = create_option(OptionStyle::Put, Side::Long, 5.0);
//         assert_eq!(option.pnl(90.0), 5.0); // (100 - 90) - 5 = 5
//         assert_eq!(option.pnl(110.0), -5.0); // 0 - 5 = -5
//     }
//
//     #[test]
//     fn test_pnl_at_expiration_put_long() {
//         let mut option = create_option(OptionStyle::Put, Side::Long, 5.0);
//         option.underlying_price = 90.0;
//         assert_eq!(option.pnl_at_expiration(), 5.0); // (100 - 90) - 5 = 5
//     }
//
//     #[test]
//     fn test_pnl_put_short() {
//         let option = create_option(OptionStyle::Put, Side::Short, -5.0);
//         assert_eq!(option.pnl(90.0), -5.0); // -(100 - 90) + 5 = -5
//         assert_eq!(option.pnl(110.0), 5.0); // 0 + 5 = 5
//     }
//
//     #[test]
//     fn test_pnl_at_expiration_put_short() {
//         let mut option = create_option(OptionStyle::Put, Side::Short, -5.0);
//         option.underlying_price = 90.0;
//         assert_eq!(option.pnl_at_expiration(), -5.0); // -(100 - 90) + 5 = -5
//     }
// }
