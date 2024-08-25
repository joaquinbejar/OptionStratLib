/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 18/8/24
******************************************************************************/
use crate::constants::ZERO;
use crate::greeks::equations::{Greek, Greeks};
use crate::model::option::Options;
use crate::model::types::{ExpirationDate, OptionStyle, Side};
use crate::pnl::utils::{PnL, PnLCalculator};
use crate::visualization::utils::Graph;
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
        match self.option.side {
            Side::Long => {
                (self.premium + self.open_fee + self.close_fee) * self.option.quantity as f64
            }
            Side::Short => (self.open_fee + self.close_fee) * self.option.quantity as f64,
        }
    }

    pub fn premium_received(&self) -> f64 {
        match self.option.side {
            Side::Long => ZERO,
            Side::Short => self.premium * self.option.quantity as f64,
        }
    }

    pub fn net_premium_received(&self) -> f64 {
        match self.option.side {
            Side::Long => ZERO,
            Side::Short => self.premium_received() - self.total_cost(),
        }
    }

    pub fn pnl_at_expiration(&self, underlying_price: Option<f64>) -> f64 {
        match underlying_price {
            None => {
                self.option.intrinsic_value(self.option.underlying_price) - self.total_cost()
                    + self.premium_received()
            }
            Some(price) => {
                self.option.intrinsic_value(price) - self.total_cost() + self.premium_received()
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

    pub(crate) fn net_cost(&self) -> f64 {
        match self.option.side {
            Side::Long => self.total_cost(),
            Side::Short => {
                (self.open_fee + self.close_fee - self.premium).abs() * self.option.quantity as f64
            }
        }
    }

    fn check_premium(mut premium: f64) -> f64 {
        if premium < ZERO {
            premium *= -1.0;
        }
        premium
    }

    pub fn break_even(&self) -> f64 {
        let total_cost_per_contract = self.total_cost() / self.option.quantity as f64;
        match (&self.option.side, &self.option.option_style) {
            (Side::Long, OptionStyle::Call) => self.option.strike_price + total_cost_per_contract,
            (Side::Short, OptionStyle::Call) => {
                self.option.strike_price + self.premium - total_cost_per_contract
            }
            (Side::Long, OptionStyle::Put) => self.option.strike_price - total_cost_per_contract,
            (Side::Short, OptionStyle::Put) => {
                self.option.strike_price - self.premium + total_cost_per_contract
            }
        }
    }

    #[allow(dead_code)]
    pub(crate) fn max_profit(&self) -> f64 {
        match self.option.side {
            Side::Long => f64::INFINITY,
            Side::Short => self.premium * self.option.quantity as f64 - self.total_cost(),
        }
    }
    #[allow(dead_code)]
    pub(crate) fn max_loss(&self) -> f64 {
        match self.option.side {
            Side::Long => self.total_cost(),
            Side::Short => f64::INFINITY,
        }
    }
}

impl Default for Position {
    fn default() -> Self {
        Position {
            option: Options::default(),
            premium: ZERO,
            date: Utc::now(),
            open_fee: ZERO,
            close_fee: ZERO,
        }
    }
}

impl Greeks for Position {
    fn greeks(&self) -> Greek {
        self.option.greeks()
    }
}

impl PnLCalculator for Position {
    fn calculate_pnl(&self, date_time: DateTime<Utc>, market_price: f64) -> PnL {
        PnL::new(
            None,
            Some(self.unrealized_pnl(market_price)),
            self.total_cost(),
            self.premium_received(),
            date_time,
        )
    }

    fn calculate_pnl_at_expiration(&self, underlying_price: Option<f64>) -> PnL {
        PnL::new(
            Some(self.pnl_at_expiration(underlying_price)),
            None,
            self.total_cost(),
            self.premium_received(),
            self.option.expiration_date.get_date(),
        )
    }
}

impl Graph for Position {

    fn title(&self) -> String {
        self.option.title()
    }

    fn get_values(&self, data: &[f64]) -> Vec<f64> {
        data.iter()
            .map(|&price| self.pnl_at_expiration(Some(price)))
            .collect()
    }


    fn get_vertical_lines(&self) -> Vec<(String, f64)> {
        [("Break Even".to_string(), self.break_even())].to_vec()
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
        let option = setup_option(Side::Long, OptionStyle::Call, 100.0, 105.0, 1, 30);
        let position = Position::new(option, 5.0, Utc::now(), 1.0, 1.0);
        assert_eq!(
            position.total_cost(),
            7.0,
            "Total cost calculation is incorrect."
        );
    }

    #[test]
    fn test_position_total_cost_size() {
        let option = setup_option(Side::Long, OptionStyle::Call, 100.0, 105.0, 10, 30);
        let position = Position::new(option, 5.0, Utc::now(), 1.0, 1.0);
        assert_eq!(
            position.total_cost(),
            70.0,
            "Total cost calculation is incorrect."
        );
    }

    #[test]
    fn test_position_total_cost_short() {
        let option = setup_option(Side::Short, OptionStyle::Call, 100.0, 105.0, 1, 30);
        let position = Position::new(option, 5.0, Utc::now(), 1.0, 1.0);
        assert_eq!(
            position.total_cost(),
            2.0,
            "Total cost calculation is incorrect."
        );
    }

    #[test]
    fn test_position_total_cost_short_size() {
        let option = setup_option(Side::Short, OptionStyle::Call, 100.0, 105.0, 10, 30);
        let position = Position::new(option, 5.0, Utc::now(), 1.0, 1.0);
        assert_eq!(
            position.total_cost(),
            20.0,
            "Total cost calculation is incorrect."
        );
    }

    #[test]
    fn test_position_check_negative_premium() {
        let option = setup_option(Side::Long, OptionStyle::Call, 100.0, 110.0, 1, 0);
        let position = Position::new(option, 5.0, Utc::now(), 1.0, 1.0);
        assert_eq!(
            position.pnl_at_expiration(None),
            3.0,
            "PNL at expiration for long call ITM is incorrect."
        );
    }

    #[test]
    fn test_position_pnl_at_expiration_long_call_itm() {
        let option = setup_option(Side::Long, OptionStyle::Call, 100.0, 110.0, 1, 0);
        let position = Position::new(option, 5.0, Utc::now(), 1.0, 1.0);
        assert_eq!(
            position.pnl_at_expiration(None),
            3.0,
            "PNL at expiration for long call ITM is incorrect."
        );
    }

    #[test]
    fn test_position_pnl_at_expiration_long_call_itm_quantity() {
        let option = setup_option(Side::Long, OptionStyle::Call, 100.0, 110.0, 10, 0);
        let position = Position::new(option, 5.0, Utc::now(), 1.0, 1.0);
        assert_eq!(
            position.pnl_at_expiration(None),
            30.0,
            "PNL at expiration for long call ITM is incorrect."
        );
    }

    #[test]
    fn test_position_pnl_at_expiration_short_call_itm() {
        let option = setup_option(Side::Short, OptionStyle::Call, 100.0, 110.0, 1, 0);
        let position = Position::new(option, 5.0, Utc::now(), 1.0, 1.0);
        assert_eq!(
            position.pnl_at_expiration(None),
            -7.0,
            "PNL at expiration for short call ITM is incorrect."
        );
    }

    #[test]
    fn test_position_pnl_at_expiration_short_call_itm_quantity() {
        let option = setup_option(Side::Short, OptionStyle::Call, 100.0, 110.0, 10, 0);
        let position = Position::new(option, 5.0, Utc::now(), 1.0, 1.0);
        assert_eq!(
            position.pnl_at_expiration(None),
            -70.0,
            "PNL at expiration for short call ITM is incorrect."
        );
    }

    #[test]
    fn test_position_pnl_at_expiration_long_put_itm() {
        let option = setup_option(Side::Long, OptionStyle::Put, 100.0, 90.0, 1, 0);
        let position = Position::new(option, 5.0, Utc::now(), 1.0, 1.0);
        assert_eq!(
            position.pnl_at_expiration(None),
            3.0,
            "PNL at expiration for long put ITM is incorrect."
        );
    }

    #[test]
    fn test_position_pnl_at_expiration_long_put_itm_quantity() {
        let option = setup_option(Side::Long, OptionStyle::Put, 100.0, 90.0, 10, 0);
        let position = Position::new(option, 5.0, Utc::now(), 1.0, 1.0);
        assert_eq!(
            position.pnl_at_expiration(None),
            30.0,
            "PNL at expiration for long put ITM is incorrect."
        );
    }

    #[test]
    fn test_position_pnl_at_expiration_short_put_itm() {
        let option = setup_option(Side::Short, OptionStyle::Put, 100.0, 90.0, 1, 0);
        let position = Position::new(option, 5.0, Utc::now(), 1.0, 1.0);
        assert_eq!(
            position.pnl_at_expiration(None),
            -7.0,
            "PNL at expiration for short put ITM is incorrect."
        );
    }

    #[test]
    fn test_position_pnl_at_expiration_short_put_itm_quantity() {
        let option = setup_option(Side::Short, OptionStyle::Put, 100.0, 90.0, 10, 0);
        let position = Position::new(option, 5.0, Utc::now(), 1.0, 1.0);
        assert_eq!(
            position.pnl_at_expiration(None),
            -70.0,
            "PNL at expiration for short put ITM is incorrect."
        );
    }

    #[test]
    fn test_position_pnl_at_expiration_short_put_itm_winning() {
        let option = setup_option(Side::Short, OptionStyle::Put, 100.0, 110.0, 1, 0);
        let position = Position::new(option, 5.0, Utc::now(), 1.0, 1.0);
        assert_eq!(
            position.pnl_at_expiration(None),
            3.0,
            "PNL at expiration for short put ITM is incorrect."
        );
    }

    #[test]
    fn test_position_pnl_at_expiration_short_put_itm_quantity_winning() {
        let option = setup_option(Side::Short, OptionStyle::Put, 100.0, 110.0, 10, 0);
        let position = Position::new(option, 5.0, Utc::now(), 1.0, 1.0);
        assert_eq!(
            position.pnl_at_expiration(None),
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

#[cfg(test)]
mod tests_position_break_even {
    use super::*;
    use crate::constants::ZERO;
    use crate::model::types::{ExpirationDate, OptionStyle, OptionType, Side};

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
    fn test_unrealized_pnl_long_call() {
        let option = setup_option(Side::Long, OptionStyle::Call, 100.0, 105.0, 1, 30);
        let position = Position::new(option, 5.0, Utc::now(), 1.0, 1.0);
        assert_eq!(position.break_even(), 107.0);
    }

    #[test]
    fn test_unrealized_pnl_long_call_size() {
        let option = setup_option(Side::Long, OptionStyle::Call, 100.0, 105.0, 10, 30);
        let position = Position::new(option, 5.0, Utc::now(), 1.0, 1.0);
        assert_eq!(position.break_even(), 107.0);
    }

    #[test]
    fn test_unrealized_pnl_short_call() {
        let option = setup_option(Side::Short, OptionStyle::Call, 100.0, 105.0, 1, 30);
        let position = Position::new(option, 5.0, Utc::now(), 1.0, 1.0);
        assert_eq!(position.break_even(), 103.0);
    }

    #[test]
    fn test_unrealized_pnl_short_call_size() {
        let option = setup_option(Side::Short, OptionStyle::Call, 100.0, 105.0, 10, 30);
        let position = Position::new(option, 5.0, Utc::now(), 1.0, 1.0);
        assert_eq!(position.break_even(), 103.0);
    }

    #[test]
    fn test_unrealized_pnl_long_put() {
        let option = setup_option(Side::Long, OptionStyle::Put, 100.0, 105.0, 1, 30);
        let position = Position::new(option, 5.0, Utc::now(), 1.0, 1.0);
        assert_eq!(position.break_even(), 93.0);
    }

    #[test]
    fn test_unrealized_pnl_long_put_size() {
        let option = setup_option(Side::Long, OptionStyle::Put, 100.0, 105.0, 10, 30);
        let position = Position::new(option, 5.0, Utc::now(), 1.0, 1.0);
        assert_eq!(position.break_even(), 93.0);
    }

    #[test]
    fn test_unrealized_pnl_short_put() {
        let option = setup_option(Side::Short, OptionStyle::Put, 100.0, 105.0, 1, 30);
        let position = Position::new(option, 5.0, Utc::now(), 1.0, 1.0);
        assert_eq!(position.break_even(), 97.0);
    }

    #[test]
    fn test_unrealized_pnl_short_put_size() {
        let option = setup_option(Side::Short, OptionStyle::Put, 100.0, 105.0, 10, 30);
        let position = Position::new(option, 5.0, Utc::now(), 1.0, 1.0);
        assert_eq!(position.break_even(), 97.0);
    }
}

#[cfg(test)]
mod tests_position_max_loss_profit {
    use super::*;
    use crate::constants::ZERO;
    use crate::model::types::{ExpirationDate, OptionStyle, OptionType, Side};
    use approx::assert_relative_eq;

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
    fn test_unrealized_pnl_long_call() {
        let option = setup_option(Side::Long, OptionStyle::Call, 100.0, 105.0, 1, 30);
        let position = Position::new(option, 5.0, Utc::now(), 1.0, 1.0);
        assert_relative_eq!(position.max_loss(), 7.0, epsilon = 0.001);
        assert_relative_eq!(position.max_profit(), f64::INFINITY, epsilon = 0.001);
    }

    #[test]
    fn test_unrealized_pnl_long_call_size() {
        let option = setup_option(Side::Long, OptionStyle::Call, 100.0, 105.0, 10, 30);
        let position = Position::new(option, 5.0, Utc::now(), 1.0, 1.0);
        assert_relative_eq!(position.max_loss(), 70.0, epsilon = 0.001);
        assert_relative_eq!(position.max_profit(), f64::INFINITY, epsilon = 0.001);
    }

    #[test]
    fn test_unrealized_pnl_short_call() {
        let option = setup_option(Side::Short, OptionStyle::Call, 100.0, 105.0, 1, 30);
        let position = Position::new(option, 5.0, Utc::now(), 1.0, 1.0);
        assert_relative_eq!(position.max_loss(), f64::INFINITY, epsilon = 0.001);
        assert_relative_eq!(position.max_profit(), 3.0, epsilon = 0.001);
    }

    #[test]
    fn test_unrealized_pnl_short_call_size() {
        let option = setup_option(Side::Short, OptionStyle::Call, 100.0, 105.0, 10, 30);
        let position = Position::new(option, 5.0, Utc::now(), 1.0, 1.0);
        assert_relative_eq!(position.max_loss(), f64::INFINITY, epsilon = 0.001);
        assert_relative_eq!(position.max_profit(), 30.0, epsilon = 0.001);
    }

    #[test]
    fn test_unrealized_pnl_long_put() {
        let option = setup_option(Side::Long, OptionStyle::Put, 100.0, 105.0, 1, 30);
        let position = Position::new(option, 5.0, Utc::now(), 1.0, 1.0);
        assert_relative_eq!(position.max_loss(), 7.0, epsilon = 0.001);
        assert_relative_eq!(position.max_profit(), f64::INFINITY, epsilon = 0.001);
    }

    #[test]
    fn test_unrealized_pnl_long_put_size() {
        let option = setup_option(Side::Long, OptionStyle::Put, 100.0, 105.0, 10, 30);
        let position = Position::new(option, 5.0, Utc::now(), 1.0, 1.0);
        assert_relative_eq!(position.max_loss(), 70.0, epsilon = 0.001);
        assert_relative_eq!(position.max_profit(), f64::INFINITY, epsilon = 0.001);
    }

    #[test]
    fn test_unrealized_pnl_short_put() {
        let option = setup_option(Side::Short, OptionStyle::Put, 100.0, 105.0, 1, 30);
        let position = Position::new(option, 5.0, Utc::now(), 1.0, 1.0);
        assert_relative_eq!(position.max_loss(), f64::INFINITY, epsilon = 0.001);
        assert_relative_eq!(position.max_profit(), 3.0, epsilon = 0.001);
    }

    #[test]
    fn test_unrealized_pnl_short_put_size() {
        let option = setup_option(Side::Short, OptionStyle::Put, 100.0, 105.0, 10, 30);
        let position = Position::new(option, 5.0, Utc::now(), 1.0, 1.0);
        assert_relative_eq!(position.max_loss(), f64::INFINITY, epsilon = 0.001);
        assert_relative_eq!(position.max_profit(), 30.0, epsilon = 0.001);
    }
}
