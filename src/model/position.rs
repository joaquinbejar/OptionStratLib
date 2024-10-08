/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 18/8/24
******************************************************************************/
use crate::constants::ZERO;
use crate::greeks::equations::{Greek, Greeks};
use crate::model::option::Options;
use crate::model::types::{ExpirationDate, OptionStyle, PositiveF64, Side};
use crate::pnl::utils::{PnL, PnLCalculator};
use crate::pos;
use crate::pricing::payoff::Profit;
use crate::visualization::model::ChartVerticalLine;
use crate::visualization::utils::Graph;
use chrono::{DateTime, Utc};
use plotters::prelude::{ShapeStyle, BLACK};

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
            Side::Long => (self.premium + self.open_fee + self.close_fee) * self.option.quantity,
            Side::Short => (self.open_fee + self.close_fee) * self.option.quantity,
        }
    }

    pub fn premium_received(&self) -> f64 {
        match self.option.side {
            Side::Long => ZERO,
            Side::Short => self.premium * self.option.quantity,
        }
    }

    pub fn net_premium_received(&self) -> f64 {
        match self.option.side {
            Side::Long => ZERO,
            Side::Short => self.premium_received() - self.total_cost(),
        }
    }

    pub fn pnl_at_expiration(&self, underlying_price: &Option<PositiveF64>) -> f64 {
        match underlying_price {
            None => {
                self.option.intrinsic_value(self.option.underlying_price) - self.total_cost()
                    + self.premium_received()
            }
            Some(price) => {
                self.option.intrinsic_value(*price) - self.total_cost() + self.premium_received()
            }
        }
    }

    pub fn unrealized_pnl(&self, current_option_price: PositiveF64) -> f64 {
        match self.option.side {
            Side::Long => ((current_option_price - self.premium - self.open_fee - self.close_fee)
                * self.option.quantity)
                .into(),
            Side::Short => {
                (self.premium - current_option_price - self.open_fee - self.close_fee)
                    * self.option.quantity
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
                (self.open_fee + self.close_fee - self.premium).abs() * self.option.quantity
            }
        }
    }

    fn check_premium(mut premium: f64) -> f64 {
        if premium < ZERO {
            premium *= -1.0;
        }
        premium
    }

    pub fn break_even(&self) -> PositiveF64 {
        let total_cost_per_contract = self.total_cost() / self.option.quantity;
        match (&self.option.side, &self.option.option_style) {
            (Side::Long, OptionStyle::Call) => {
                pos!(self.option.strike_price.value() + total_cost_per_contract)
            }
            (Side::Short, OptionStyle::Call) => {
                pos!(self.option.strike_price.value() + self.premium - total_cost_per_contract)
            }
            (Side::Long, OptionStyle::Put) => {
                pos!(self.option.strike_price.value() - total_cost_per_contract)
            }
            (Side::Short, OptionStyle::Put) => {
                pos!(self.option.strike_price.value() - self.premium + total_cost_per_contract)
            }
        }
    }

    #[allow(dead_code)]
    pub(crate) fn max_profit(&self) -> f64 {
        match self.option.side {
            Side::Long => f64::INFINITY,
            Side::Short => self.premium * self.option.quantity - self.total_cost(),
        }
    }
    #[allow(dead_code)]
    pub(crate) fn max_loss(&self) -> f64 {
        match self.option.side {
            Side::Long => self.total_cost(),
            Side::Short => f64::INFINITY,
        }
    }

    #[allow(dead_code)]
    pub(crate) fn fees(&self) -> f64 {
        (self.open_fee + self.close_fee) * self.option.quantity
    }

    pub(crate) fn validate(&self) -> bool {
        if self.premium <= ZERO {
            return false;
        }
        if self.open_fee < ZERO {
            return false;
        }
        if self.close_fee < ZERO {
            return false;
        }
        if !self.option.validate() {
            return false;
        }
        true
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
    fn calculate_pnl(&self, date_time: DateTime<Utc>, market_price: PositiveF64) -> PnL {
        PnL::new(
            None,
            Some(self.unrealized_pnl(market_price)),
            self.total_cost(),
            self.premium_received(),
            date_time,
        )
    }

    fn calculate_pnl_at_expiration(&self, underlying_price: Option<PositiveF64>) -> PnL {
        PnL::new(
            Some(self.pnl_at_expiration(&underlying_price)),
            None,
            self.total_cost(),
            self.premium_received(),
            self.option.expiration_date.get_date(),
        )
    }
}

impl Profit for Position {
    fn calculate_profit_at(&self, price: PositiveF64) -> f64 {
        let price = price.into();
        self.pnl_at_expiration(&price)
    }
}

impl Graph for Position {
    fn title(&self) -> String {
        self.option.title()
    }

    fn get_values(&self, data: &[PositiveF64]) -> Vec<f64> {
        data.iter()
            .map(|&price| self.pnl_at_expiration(&Some(price)))
            .collect()
    }

    fn get_vertical_lines(&self) -> Vec<ChartVerticalLine<f64, f64>> {
        let vertical_lines = vec![ChartVerticalLine {
            x_coordinate: self.break_even().value(),
            y_range: (-50000.0, 50000.0),
            label: "Break Even".to_string(),
            label_offset: (5.0, 5.0),
            line_color: BLACK,
            label_color: BLACK,
            line_style: ShapeStyle::from(&BLACK).stroke_width(1),
            font_size: 18,
        }];

        vertical_lines
    }
}

#[cfg(test)]
mod tests_position {
    use super::*;
    use crate::constants::ZERO;
    use crate::model::types::{ExpirationDate, OptionStyle, OptionType, PositiveF64, Side};
    use crate::pos;
    use chrono::Duration;

    fn setup_option(
        side: Side,
        option_style: OptionStyle,
        strike_price: PositiveF64,
        underlying_price: PositiveF64,
        quantity: PositiveF64,
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
        let option = setup_option(
            Side::Long,
            OptionStyle::Call,
            pos!(100.0),
            pos!(105.0),
            pos!(1.0),
            30,
        );
        let position = Position::new(option, 5.0, Utc::now(), 1.0, 1.0);
        assert_eq!(
            position.total_cost(),
            7.0,
            "Total cost calculation is incorrect."
        );
    }

    #[test]
    fn test_position_total_cost_size() {
        let option = setup_option(
            Side::Long,
            OptionStyle::Call,
            pos!(100.0),
            pos!(105.0),
            pos!(10.0),
            30,
        );
        let position = Position::new(option, 5.0, Utc::now(), 1.0, 1.0);
        assert_eq!(
            position.total_cost(),
            70.0,
            "Total cost calculation is incorrect."
        );
    }

    #[test]
    fn test_position_total_cost_short() {
        let option = setup_option(
            Side::Short,
            OptionStyle::Call,
            pos!(100.0),
            pos!(105.0),
            pos!(1.0),
            30,
        );
        let position = Position::new(option, 5.0, Utc::now(), 1.0, 1.0);
        assert_eq!(
            position.total_cost(),
            2.0,
            "Total cost calculation is incorrect."
        );
    }

    #[test]
    fn test_position_total_cost_short_size() {
        let option = setup_option(
            Side::Short,
            OptionStyle::Call,
            pos!(100.0),
            pos!(105.0),
            pos!(10.0),
            30,
        );
        let position = Position::new(option, 5.0, Utc::now(), 1.0, 1.0);
        assert_eq!(
            position.total_cost(),
            20.0,
            "Total cost calculation is incorrect."
        );
    }

    #[test]
    fn test_position_check_negative_premium() {
        let option = setup_option(
            Side::Long,
            OptionStyle::Call,
            pos!(100.0),
            pos!(110.0),
            pos!(1.0),
            0,
        );
        let position = Position::new(option, 5.0, Utc::now(), 1.0, 1.0);
        assert_eq!(
            position.pnl_at_expiration(&None),
            3.0,
            "PNL at expiration for long call ITM is incorrect."
        );
    }

    #[test]
    fn test_position_pnl_at_expiration_long_call_itm() {
        let option = setup_option(
            Side::Long,
            OptionStyle::Call,
            pos!(100.0),
            pos!(110.0),
            pos!(1.0),
            0,
        );
        let position = Position::new(option, 5.0, Utc::now(), 1.0, 1.0);
        assert_eq!(
            position.pnl_at_expiration(&None),
            3.0,
            "PNL at expiration for long call ITM is incorrect."
        );
    }

    #[test]
    fn test_position_pnl_at_expiration_long_call_itm_quantity() {
        let option = setup_option(
            Side::Long,
            OptionStyle::Call,
            pos!(100.0),
            pos!(110.0),
            pos!(10.0),
            0,
        );
        let position = Position::new(option, 5.0, Utc::now(), 1.0, 1.0);
        assert_eq!(
            position.pnl_at_expiration(&None),
            30.0,
            "PNL at expiration for long call ITM is incorrect."
        );
    }

    #[test]
    fn test_position_pnl_at_expiration_short_call_itm() {
        let option = setup_option(
            Side::Short,
            OptionStyle::Call,
            pos!(100.0),
            pos!(110.0),
            pos!(1.0),
            0,
        );
        let position = Position::new(option, 5.0, Utc::now(), 1.0, 1.0);
        assert_eq!(
            position.pnl_at_expiration(&None),
            -7.0,
            "PNL at expiration for short call ITM is incorrect."
        );
    }

    #[test]
    fn test_position_pnl_at_expiration_short_call_itm_quantity() {
        let option = setup_option(
            Side::Short,
            OptionStyle::Call,
            pos!(100.0),
            pos!(110.0),
            pos!(10.0),
            0,
        );
        let position = Position::new(option, 5.0, Utc::now(), 1.0, 1.0);
        assert_eq!(
            position.pnl_at_expiration(&None),
            -70.0,
            "PNL at expiration for short call ITM is incorrect."
        );
    }

    #[test]
    fn test_position_pnl_at_expiration_long_put_itm() {
        let option = setup_option(
            Side::Long,
            OptionStyle::Put,
            pos!(100.0),
            pos!(90.0),
            pos!(1.0),
            0,
        );
        let position = Position::new(option, 5.0, Utc::now(), 1.0, 1.0);
        assert_eq!(
            position.pnl_at_expiration(&None),
            3.0,
            "PNL at expiration for long put ITM is incorrect."
        );
    }

    #[test]
    fn test_position_pnl_at_expiration_long_put_itm_quantity() {
        let option = setup_option(
            Side::Long,
            OptionStyle::Put,
            pos!(100.0),
            pos!(90.0),
            pos!(10.0),
            0,
        );
        let position = Position::new(option, 5.0, Utc::now(), 1.0, 1.0);
        assert_eq!(
            position.pnl_at_expiration(&None),
            30.0,
            "PNL at expiration for long put ITM is incorrect."
        );
    }

    #[test]
    fn test_position_pnl_at_expiration_short_put_itm() {
        let option = setup_option(
            Side::Short,
            OptionStyle::Put,
            pos!(100.0),
            pos!(90.0),
            pos!(1.0),
            0,
        );
        let position = Position::new(option, 5.0, Utc::now(), 1.0, 1.0);
        assert_eq!(
            position.pnl_at_expiration(&None),
            -7.0,
            "PNL at expiration for short put ITM is incorrect."
        );
    }

    #[test]
    fn test_position_pnl_at_expiration_short_put_itm_quantity() {
        let option = setup_option(
            Side::Short,
            OptionStyle::Put,
            pos!(100.0),
            pos!(90.0),
            pos!(10.0),
            0,
        );
        let position = Position::new(option, 5.0, Utc::now(), 1.0, 1.0);
        assert_eq!(
            position.pnl_at_expiration(&None),
            -70.0,
            "PNL at expiration for short put ITM is incorrect."
        );
    }

    #[test]
    fn test_position_pnl_at_expiration_short_put_itm_winning() {
        let option = setup_option(
            Side::Short,
            OptionStyle::Put,
            pos!(100.0),
            pos!(110.0),
            pos!(1.0),
            0,
        );
        let position = Position::new(option, 5.0, Utc::now(), 1.0, 1.0);
        assert_eq!(
            position.pnl_at_expiration(&None),
            3.0,
            "PNL at expiration for short put ITM is incorrect."
        );
    }

    #[test]
    fn test_position_pnl_at_expiration_short_put_itm_quantity_winning() {
        let option = setup_option(
            Side::Short,
            OptionStyle::Put,
            pos!(100.0),
            pos!(110.0),
            pos!(10.0),
            0,
        );
        let position = Position::new(option, 5.0, Utc::now(), 1.0, 1.0);
        assert_eq!(
            position.pnl_at_expiration(&None),
            30.0,
            "PNL at expiration for short put ITM is incorrect."
        );
    }

    #[test]
    fn test_unrealized_pnl_long_call() {
        let option = setup_option(
            Side::Long,
            OptionStyle::Call,
            pos!(100.0),
            pos!(105.0),
            pos!(1.0),
            30,
        );
        let position = Position::new(option, 5.0, Utc::now(), 1.0, 1.0);
        assert_eq!(
            position.unrealized_pnl(pos!(7.0)),
            ZERO,
            "Unrealized PNL for long call is incorrect."
        );
    }

    #[test]
    fn test_unrealized_pnl_long_call_quantity() {
        let option = setup_option(
            Side::Long,
            OptionStyle::Call,
            pos!(100.0),
            pos!(105.0),
            pos!(10.0),
            30,
        );
        let position = Position::new(option, 5.0, Utc::now(), 1.0, 1.0);
        assert_eq!(
            position.unrealized_pnl(pos!(7.0)),
            ZERO,
            "Unrealized PNL for long call is incorrect."
        );
    }

    #[test]
    fn test_unrealized_pnl_short_call() {
        let option = setup_option(
            Side::Short,
            OptionStyle::Call,
            pos!(100.0),
            pos!(105.0),
            pos!(10.0),
            30,
        );
        let position = Position::new(option, 5.0, Utc::now(), 1.0, 1.0);
        assert_eq!(
            position.unrealized_pnl(pos!(3.0)),
            ZERO,
            "Unrealized PNL for short call is incorrect."
        );
    }

    #[test]
    fn test_unrealized_pnl_short_call_bis() {
        let option = setup_option(
            Side::Short,
            OptionStyle::Call,
            pos!(100.0),
            pos!(105.0),
            pos!(1.0),
            30,
        );
        let position = Position::new(option, 5.0, Utc::now(), 1.0, 1.0);
        assert_eq!(
            position.unrealized_pnl(pos!(10.0)),
            -7.0,
            "Unrealized PNL for short call is incorrect."
        );
    }

    #[test]
    fn test_days_held() {
        let option = setup_option(
            Side::Long,
            OptionStyle::Call,
            pos!(100.0),
            pos!(105.0),
            pos!(10.0),
            30,
        );
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
        let option = setup_option(
            Side::Long,
            OptionStyle::Call,
            pos!(100.0),
            pos!(105.0),
            pos!(10.0),
            30,
        );
        let position = Position::new(option, 5.0, Utc::now(), 1.0, 1.0);
        assert_eq!(
            position.days_to_expiration(),
            30.0,
            "Days to expiration calculation is incorrect."
        );
    }

    #[test]
    fn test_is_long_position() {
        let option = setup_option(
            Side::Long,
            OptionStyle::Call,
            pos!(100.0),
            pos!(105.0),
            pos!(10.0),
            30,
        );
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
        let option = setup_option(
            Side::Short,
            OptionStyle::Call,
            pos!(100.0),
            pos!(105.0),
            pos!(10.0),
            30,
        );
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
mod tests_valid_position {
    use super::*;
    use crate::constants::ZERO;
    use crate::model::types::PositiveF64;
    use crate::model::types::{OptionType, PZERO};
    use crate::pos;
    use chrono::Utc;

    fn create_valid_option() -> Options {
        Options {
            option_type: OptionType::European,
            side: Side::Long,
            underlying_symbol: "AAPL".to_string(),
            strike_price: pos!(100.0),
            expiration_date: ExpirationDate::Days(30.0),
            implied_volatility: 0.2,
            quantity: pos!(1.0),
            underlying_price: pos!(105.0),
            risk_free_rate: 0.05,
            option_style: OptionStyle::Call,
            dividend_yield: 0.01,
            exotic_params: None,
        }
    }

    fn create_valid_position() -> Position {
        Position {
            option: create_valid_option(),
            premium: 5.0,
            date: Utc::now(),
            open_fee: 0.5,
            close_fee: 0.5,
        }
    }

    #[test]
    fn test_valid_position() {
        let position = create_valid_position();
        assert!(position.validate());
    }

    #[test]
    fn test_zero_premium() {
        let mut position = create_valid_position();
        position.premium = ZERO;
        assert!(!position.validate());
    }

    #[test]
    fn test_negative_premium() {
        let mut position = create_valid_position();
        position.premium = -1.0;
        assert!(!position.validate());
    }

    #[test]
    fn test_negative_open_fee() {
        let mut position = create_valid_position();
        position.open_fee = -0.1;
        assert!(!position.validate());
    }

    #[test]
    fn test_negative_close_fee() {
        let mut position = create_valid_position();
        position.close_fee = -0.1;
        assert!(!position.validate());
    }

    #[test]
    fn test_invalid_option() {
        let mut position = create_valid_position();
        position.option.strike_price = PZERO; // This makes the option invalid
        assert!(!position.validate());
    }

    #[test]
    fn test_zero_fees() {
        let mut position = create_valid_position();
        position.open_fee = ZERO;
        position.close_fee = ZERO;
        assert!(position.validate());
    }
}

#[cfg(test)]
mod tests_position_break_even {
    use super::*;
    use crate::constants::ZERO;
    use crate::model::types::{ExpirationDate, OptionStyle, OptionType, PositiveF64, Side};
    use crate::pos;

    fn setup_option(
        side: Side,
        option_style: OptionStyle,
        strike_price: PositiveF64,
        underlying_price: PositiveF64,
        quantity: PositiveF64,
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
        let option = setup_option(
            Side::Long,
            OptionStyle::Call,
            pos!(100.0),
            pos!(105.0),
            pos!(1.0),
            30,
        );
        let position = Position::new(option, 5.0, Utc::now(), 1.0, 1.0);
        assert_eq!(position.break_even(), 107.0);
    }

    #[test]
    fn test_unrealized_pnl_long_call_size() {
        let option = setup_option(
            Side::Long,
            OptionStyle::Call,
            pos!(100.0),
            pos!(105.0),
            pos!(10.0),
            30,
        );
        let position = Position::new(option, 5.0, Utc::now(), 1.0, 1.0);
        assert_eq!(position.break_even(), 107.0);
    }

    #[test]
    fn test_unrealized_pnl_short_call() {
        let option = setup_option(
            Side::Short,
            OptionStyle::Call,
            pos!(100.0),
            pos!(105.0),
            pos!(1.0),
            30,
        );
        let position = Position::new(option, 5.0, Utc::now(), 1.0, 1.0);
        assert_eq!(position.break_even(), 103.0);
    }

    #[test]
    fn test_unrealized_pnl_short_call_size() {
        let option = setup_option(
            Side::Short,
            OptionStyle::Call,
            pos!(100.0),
            pos!(105.0),
            pos!(10.0),
            30,
        );
        let position = Position::new(option, 5.0, Utc::now(), 1.0, 1.0);
        assert_eq!(position.break_even(), 103.0);
    }

    #[test]
    fn test_unrealized_pnl_long_put() {
        let option = setup_option(
            Side::Long,
            OptionStyle::Put,
            pos!(100.0),
            pos!(105.0),
            pos!(1.0),
            30,
        );
        let position = Position::new(option, 5.0, Utc::now(), 1.0, 1.0);
        assert_eq!(position.break_even(), 93.0);
    }

    #[test]
    fn test_unrealized_pnl_long_put_size() {
        let option = setup_option(
            Side::Long,
            OptionStyle::Put,
            pos!(100.0),
            pos!(105.0),
            pos!(10.0),
            30,
        );
        let position = Position::new(option, 5.0, Utc::now(), 1.0, 1.0);
        assert_eq!(position.break_even(), 93.0);
    }

    #[test]
    fn test_unrealized_pnl_short_put() {
        let option = setup_option(
            Side::Short,
            OptionStyle::Put,
            pos!(100.0),
            pos!(105.0),
            pos!(1.0),
            30,
        );
        let position = Position::new(option, 5.0, Utc::now(), 1.0, 1.0);
        assert_eq!(position.break_even(), 97.0);
    }

    #[test]
    fn test_unrealized_pnl_short_put_size() {
        let option = setup_option(
            Side::Short,
            OptionStyle::Put,
            pos!(100.0),
            pos!(105.0),
            pos!(10.0),
            30,
        );
        let position = Position::new(option, 5.0, Utc::now(), 1.0, 1.0);
        assert_eq!(position.break_even(), 97.0);
    }
}

#[cfg(test)]
mod tests_position_max_loss_profit {
    use super::*;
    use crate::constants::ZERO;
    use crate::model::types::{ExpirationDate, OptionStyle, OptionType, PositiveF64, Side};
    use crate::pos;
    use approx::assert_relative_eq;

    fn setup_option(
        side: Side,
        option_style: OptionStyle,
        strike_price: PositiveF64,
        underlying_price: PositiveF64,
        quantity: PositiveF64,
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
        let option = setup_option(
            Side::Long,
            OptionStyle::Call,
            pos!(100.0),
            pos!(105.0),
            pos!(1.0),
            30,
        );
        let position = Position::new(option, 5.0, Utc::now(), 1.0, 1.0);
        assert_relative_eq!(position.max_loss(), 7.0, epsilon = 0.001);
        assert_relative_eq!(position.max_profit(), f64::INFINITY, epsilon = 0.001);
    }

    #[test]
    fn test_unrealized_pnl_long_call_size() {
        let option = setup_option(
            Side::Long,
            OptionStyle::Call,
            pos!(100.0),
            pos!(105.0),
            pos!(10.0),
            30,
        );
        let position = Position::new(option, 5.0, Utc::now(), 1.0, 1.0);
        assert_relative_eq!(position.max_loss(), 70.0, epsilon = 0.001);
        assert_relative_eq!(position.max_profit(), f64::INFINITY, epsilon = 0.001);
    }

    #[test]
    fn test_unrealized_pnl_short_call() {
        let option = setup_option(
            Side::Short,
            OptionStyle::Call,
            pos!(100.0),
            pos!(105.0),
            pos!(1.0),
            30,
        );
        let position = Position::new(option, 5.0, Utc::now(), 1.0, 1.0);
        assert_relative_eq!(position.max_loss(), f64::INFINITY, epsilon = 0.001);
        assert_relative_eq!(position.max_profit(), 3.0, epsilon = 0.001);
    }

    #[test]
    fn test_unrealized_pnl_short_call_size() {
        let option = setup_option(
            Side::Short,
            OptionStyle::Call,
            pos!(100.0),
            pos!(105.0),
            pos!(10.0),
            30,
        );
        let position = Position::new(option, 5.0, Utc::now(), 1.0, 1.0);
        assert_relative_eq!(position.max_loss(), f64::INFINITY, epsilon = 0.001);
        assert_relative_eq!(position.max_profit(), 30.0, epsilon = 0.001);
    }

    #[test]
    fn test_unrealized_pnl_long_put() {
        let option = setup_option(
            Side::Long,
            OptionStyle::Put,
            pos!(100.0),
            pos!(105.0),
            pos!(1.0),
            30,
        );
        let position = Position::new(option, 5.0, Utc::now(), 1.0, 1.0);
        assert_relative_eq!(position.max_loss(), 7.0, epsilon = 0.001);
        assert_relative_eq!(position.max_profit(), f64::INFINITY, epsilon = 0.001);
    }

    #[test]
    fn test_unrealized_pnl_long_put_size() {
        let option = setup_option(
            Side::Long,
            OptionStyle::Put,
            pos!(100.0),
            pos!(105.0),
            pos!(10.0),
            30,
        );
        let position = Position::new(option, 5.0, Utc::now(), 1.0, 1.0);
        assert_relative_eq!(position.max_loss(), 70.0, epsilon = 0.001);
        assert_relative_eq!(position.max_profit(), f64::INFINITY, epsilon = 0.001);
    }

    #[test]
    fn test_unrealized_pnl_short_put() {
        let option = setup_option(
            Side::Short,
            OptionStyle::Put,
            pos!(100.0),
            pos!(105.0),
            pos!(1.0),
            30,
        );
        let position = Position::new(option, 5.0, Utc::now(), 1.0, 1.0);
        assert_relative_eq!(position.max_loss(), f64::INFINITY, epsilon = 0.001);
        assert_relative_eq!(position.max_profit(), 3.0, epsilon = 0.001);
    }

    #[test]
    fn test_unrealized_pnl_short_put_size() {
        let option = setup_option(
            Side::Short,
            OptionStyle::Put,
            pos!(100.0),
            pos!(105.0),
            pos!(10.0),
            30,
        );
        let position = Position::new(option, 5.0, Utc::now(), 1.0, 1.0);
        assert_relative_eq!(position.max_loss(), f64::INFINITY, epsilon = 0.001);
        assert_relative_eq!(position.max_profit(), 30.0, epsilon = 0.001);
    }
}
