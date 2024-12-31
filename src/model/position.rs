/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 18/8/24
******************************************************************************/
use crate::chains::chain::OptionData;
use crate::constants::ZERO;
use crate::greeks::equations::{Greek, Greeks};
use crate::model::option::Options;
use crate::model::types::{ExpirationDate, OptionStyle, Side};
use crate::pnl::utils::{PnL, PnLCalculator};
use crate::pricing::payoff::Profit;
use crate::visualization::model::ChartVerticalLine;
use crate::visualization::utils::Graph;
use crate::{f2p, Positive};
use chrono::{DateTime, Utc};
use plotters::prelude::{ShapeStyle, BLACK};
use tracing::{debug, trace};

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
#[derive(Clone, PartialEq)]
pub struct Position {
    pub option: Options,
    pub premium: f64,
    pub date: DateTime<Utc>,
    pub open_fee: f64,
    pub close_fee: f64,
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

    pub(crate) fn update_from_option_data(&mut self, option_data: &OptionData) {
        self.date = Utc::now();
        self.option.update_from_option_data(option_data);

        match (self.option.side.clone(), self.option.option_style.clone()) {
            (Side::Long, OptionStyle::Call) => {
                self.premium = option_data.call_ask.unwrap().to_f64();
            }
            (Side::Long, OptionStyle::Put) => {
                self.premium = option_data.put_ask.unwrap().to_f64();
            }
            (Side::Short, OptionStyle::Call) => {
                self.premium = option_data.call_bid.unwrap().to_f64();
            }
            (Side::Short, OptionStyle::Put) => {
                self.premium = option_data.put_bid.unwrap().to_f64();
            }
        }
        trace!("Updated position: {:#?}", self);
    }

    /// Calculates the total cost of the position based on the option's side and fees.
    ///
    /// Depending on whether the position is long or short, different components
    /// contribute to the total cost calculation:
    ///
    /// - For a long position, the total cost includes the premium, open fee, and close fee
    ///   multiplied by the option's quantity.
    /// - For a short position, the total cost includes only the open fee and close fee
    ///   multiplied by the option's quantity.
    ///
    /// # Returns
    ///
    /// A `f64` representing the total cost of the position. THE VALUE IS ALWAYS POSITIVE
    ///
    pub fn total_cost(&self) -> Positive {
        let f64_total_cost = match self.option.side {
            Side::Long => (self.premium + self.open_fee + self.close_fee) * self.option.quantity.to_f64(),
            Side::Short => (self.open_fee + self.close_fee) * self.option.quantity.to_f64(),
        };
        f2p!(f64_total_cost)
    }

    pub fn premium_received(&self) -> f64 {
        match self.option.side {
            Side::Long => ZERO,
            Side::Short => self.premium * self.option.quantity.to_f64(),
        }
    }

    pub fn net_premium_received(&self) -> f64 {
        match self.option.side {
            Side::Long => ZERO,
            Side::Short => self.premium_received() - self.total_cost().to_f64(),
        }
    }

    pub fn pnl_at_expiration(&self, underlying_price: &Option<Positive>) -> f64 {
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

    pub fn unrealized_pnl(&self, current_option_price: Positive) -> f64 {
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

    /// Calculates the net cost of the position based on the option's side and fees.
    ///
    /// This method calculates the net cost of a position by determining whether the position
    /// is long or short and then computing the respective costs:
    ///
    /// - For a long position, the net cost is equivalent to the `total_cost()` of the position.
    /// - For a short position, the net cost is calculated by subtracting the premium from the
    ///   sum of the open and close fees, and then multiplying the result by the option's quantity.
    ///
    /// # Returns
    ///
    /// A `f64` representing the net cost of the position.
    /// The value should be positive but if the fee is higher than the premium it will be negative
    /// in short positions
    pub(crate) fn net_cost(&self) -> f64 {
        match self.option.side {
            Side::Long => self.total_cost().into(),
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

    pub fn break_even(&self) -> Option<Positive> {
        if self.option.quantity == ZERO {
            return None;
        }
        let total_cost_per_contract = self.total_cost() / self.option.quantity;
        match (&self.option.side, &self.option.option_style) {
            (Side::Long, OptionStyle::Call) => {
                Some(self.option.strike_price + total_cost_per_contract)
            }
            (Side::Short, OptionStyle::Call) => {
                Some(self.option.strike_price + self.premium - total_cost_per_contract)
            }
            (Side::Long, OptionStyle::Put) => {
                Some(self.option.strike_price - total_cost_per_contract)
            }
            (Side::Short, OptionStyle::Put) => {
                Some(self.option.strike_price- self.premium + total_cost_per_contract)
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
            Side::Long => self.total_cost().into(),
            Side::Short => f64::INFINITY,
        }
    }

    #[allow(dead_code)]
    pub(crate) fn fees(&self) -> f64 {
        (self.open_fee + self.close_fee) * self.option.quantity
    }

    pub(crate) fn validate(&self) -> bool {
        if self.option.side == Side::Short {
            if self.premium == ZERO {
                debug!("Premium must be greater than zero for short positions.");
                return false;
            }
            if self.premium < self.open_fee + self.close_fee {
                debug!("Premium must be greater than the sum of the fees.");
                return false;
            }
        }
        if self.premium < ZERO {
            debug!("Premium must be greater than zero.");
            return false;
        }
        if self.open_fee < ZERO {
            debug!("Open fee must be greater than zero.");
            return false;
        }
        if self.close_fee < ZERO {
            debug!("Close fee must be greater than zero.");
            return false;
        }
        if !self.option.validate() {
            debug!("Option is not valid.");
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
    fn calculate_pnl(&self, date_time: DateTime<Utc>, market_price: Positive) -> PnL {
        PnL::new(
            None,
            Some(self.unrealized_pnl(market_price)),
            self.total_cost().into(),
            self.premium_received(),
            date_time,
        )
    }

    fn calculate_pnl_at_expiration(&self, underlying_price: Option<Positive>) -> PnL {
        PnL::new(
            Some(self.pnl_at_expiration(&underlying_price)),
            None,
            self.total_cost().into(),
            self.premium_received(),
            self.option.expiration_date.get_date(),
        )
    }
}

impl Profit for Position {
    fn calculate_profit_at(&self, price: Positive) -> f64 {
        let price = price.into();
        self.pnl_at_expiration(&price)
    }
}

impl Graph for Position {
    fn title(&self) -> String {
        self.option.title()
    }

    fn get_values(&self, data: &[Positive]) -> Vec<f64> {
        data.iter()
            .map(|&price| self.pnl_at_expiration(&Some(price)))
            .collect()
    }

    fn get_vertical_lines(&self) -> Vec<ChartVerticalLine<f64, f64>> {
        match self.break_even() {
            Some(break_even) => {
                let vertical_lines = vec![ChartVerticalLine {
                    x_coordinate: break_even.into(),
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
            None => vec![],
        }
    }
}

#[cfg(test)]
mod tests_position {
    use super::*;
    use crate::constants::ZERO;
    use crate::model::types::{ExpirationDate, OptionStyle, OptionType, Side};
    use crate::f2p;
    use chrono::Duration;

    fn setup_option(
        side: Side,
        option_style: OptionStyle,
        strike_price: Positive,
        underlying_price: Positive,
        quantity: Positive,
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
            f2p!(100.0),
            f2p!(105.0),
            f2p!(1.0),
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
            f2p!(100.0),
            f2p!(105.0),
            f2p!(10.0),
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
            f2p!(100.0),
            f2p!(105.0),
            f2p!(1.0),
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
            f2p!(100.0),
            f2p!(105.0),
            f2p!(10.0),
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
            f2p!(100.0),
            f2p!(110.0),
            f2p!(1.0),
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
            f2p!(100.0),
            f2p!(110.0),
            f2p!(1.0),
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
            f2p!(100.0),
            f2p!(110.0),
            f2p!(10.0),
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
            f2p!(100.0),
            f2p!(110.0),
            f2p!(1.0),
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
            f2p!(100.0),
            f2p!(110.0),
            f2p!(10.0),
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
            f2p!(100.0),
            f2p!(90.0),
            f2p!(1.0),
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
            f2p!(100.0),
            f2p!(90.0),
            f2p!(10.0),
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
            f2p!(100.0),
            f2p!(90.0),
            f2p!(1.0),
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
            f2p!(100.0),
            f2p!(90.0),
            f2p!(10.0),
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
            f2p!(100.0),
            f2p!(110.0),
            f2p!(1.0),
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
            f2p!(100.0),
            f2p!(110.0),
            f2p!(10.0),
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
            f2p!(100.0),
            f2p!(105.0),
            f2p!(1.0),
            30,
        );
        let position = Position::new(option, 5.0, Utc::now(), 1.0, 1.0);
        assert_eq!(
            position.unrealized_pnl(f2p!(7.0)),
            ZERO,
            "Unrealized PNL for long call is incorrect."
        );
    }

    #[test]
    fn test_unrealized_pnl_long_call_quantity() {
        let option = setup_option(
            Side::Long,
            OptionStyle::Call,
            f2p!(100.0),
            f2p!(105.0),
            f2p!(10.0),
            30,
        );
        let position = Position::new(option, 5.0, Utc::now(), 1.0, 1.0);
        assert_eq!(
            position.unrealized_pnl(f2p!(7.0)),
            ZERO,
            "Unrealized PNL for long call is incorrect."
        );
    }

    #[test]
    fn test_unrealized_pnl_short_call() {
        let option = setup_option(
            Side::Short,
            OptionStyle::Call,
            f2p!(100.0),
            f2p!(105.0),
            f2p!(10.0),
            30,
        );
        let position = Position::new(option, 5.0, Utc::now(), 1.0, 1.0);
        assert_eq!(
            position.unrealized_pnl(f2p!(3.0)),
            ZERO,
            "Unrealized PNL for short call is incorrect."
        );
    }

    #[test]
    fn test_unrealized_pnl_short_call_bis() {
        let option = setup_option(
            Side::Short,
            OptionStyle::Call,
            f2p!(100.0),
            f2p!(105.0),
            f2p!(1.0),
            30,
        );
        let position = Position::new(option, 5.0, Utc::now(), 1.0, 1.0);
        assert_eq!(
            position.unrealized_pnl(f2p!(10.0)),
            -7.0,
            "Unrealized PNL for short call is incorrect."
        );
    }

    #[test]
    fn test_days_held() {
        let option = setup_option(
            Side::Long,
            OptionStyle::Call,
            f2p!(100.0),
            f2p!(105.0),
            f2p!(10.0),
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
            f2p!(100.0),
            f2p!(105.0),
            f2p!(10.0),
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
            f2p!(100.0),
            f2p!(105.0),
            f2p!(10.0),
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
            f2p!(100.0),
            f2p!(105.0),
            f2p!(10.0),
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
    use crate::model::types::{OptionType};
    use crate::f2p;
    use chrono::Utc;

    fn create_valid_option() -> Options {
        Options {
            option_type: OptionType::European,
            side: Side::Long,
            underlying_symbol: "AAPL".to_string(),
            strike_price: f2p!(100.0),
            expiration_date: ExpirationDate::Days(30.0),
            implied_volatility: 0.2,
            quantity: f2p!(1.0),
            underlying_price: f2p!(105.0),
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
        assert!(position.validate());
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
        position.option.strike_price = Positive::ZERO; // This makes the option invalid
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
    use crate::model::types::{ExpirationDate, OptionStyle, OptionType, Side};
    use crate::f2p;

    fn setup_option(
        side: Side,
        option_style: OptionStyle,
        strike_price: Positive,
        underlying_price: Positive,
        quantity: Positive,
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
            f2p!(100.0),
            f2p!(105.0),
            f2p!(1.0),
            30,
        );
        let position = Position::new(option, 5.0, Utc::now(), 1.0, 1.0);
        assert_eq!(position.break_even().unwrap(), 107.0);
    }

    #[test]
    fn test_unrealized_pnl_long_call_size() {
        let option = setup_option(
            Side::Long,
            OptionStyle::Call,
            f2p!(100.0),
            f2p!(105.0),
            f2p!(10.0),
            30,
        );
        let position = Position::new(option, 5.0, Utc::now(), 1.0, 1.0);
        assert_eq!(position.break_even().unwrap(), 107.0);
    }

    #[test]
    fn test_unrealized_pnl_short_call() {
        let option = setup_option(
            Side::Short,
            OptionStyle::Call,
            f2p!(100.0),
            f2p!(105.0),
            f2p!(1.0),
            30,
        );
        let position = Position::new(option, 5.0, Utc::now(), 1.0, 1.0);
        assert_eq!(position.break_even().unwrap(), 103.0);
    }

    #[test]
    fn test_unrealized_pnl_short_call_size() {
        let option = setup_option(
            Side::Short,
            OptionStyle::Call,
            f2p!(100.0),
            f2p!(105.0),
            f2p!(10.0),
            30,
        );
        let position = Position::new(option, 5.0, Utc::now(), 1.0, 1.0);
        assert_eq!(position.break_even().unwrap(), 103.0);
    }

    #[test]
    fn test_unrealized_pnl_long_put() {
        let option = setup_option(
            Side::Long,
            OptionStyle::Put,
            f2p!(100.0),
            f2p!(105.0),
            f2p!(1.0),
            30,
        );
        let position = Position::new(option, 5.0, Utc::now(), 1.0, 1.0);
        assert_eq!(position.break_even().unwrap(), 93.0);
    }

    #[test]
    fn test_unrealized_pnl_long_put_size() {
        let option = setup_option(
            Side::Long,
            OptionStyle::Put,
            f2p!(100.0),
            f2p!(105.0),
            f2p!(10.0),
            30,
        );
        let position = Position::new(option, 5.0, Utc::now(), 1.0, 1.0);
        assert_eq!(position.break_even().unwrap(), 93.0);
    }

    #[test]
    fn test_unrealized_pnl_short_put() {
        let option = setup_option(
            Side::Short,
            OptionStyle::Put,
            f2p!(100.0),
            f2p!(105.0),
            f2p!(1.0),
            30,
        );
        let position = Position::new(option, 5.0, Utc::now(), 1.0, 1.0);
        assert_eq!(position.break_even().unwrap(), 97.0);
    }

    #[test]
    fn test_unrealized_pnl_short_put_size() {
        let option = setup_option(
            Side::Short,
            OptionStyle::Put,
            f2p!(100.0),
            f2p!(105.0),
            f2p!(10.0),
            30,
        );
        let position = Position::new(option, 5.0, Utc::now(), 1.0, 1.0);
        assert_eq!(position.break_even().unwrap(), 97.0);
    }
}

#[cfg(test)]
mod tests_position_max_loss_profit {
    use super::*;
    use crate::constants::ZERO;
    use crate::model::types::{ExpirationDate, OptionStyle, OptionType, Side};
    use crate::f2p;
    use approx::assert_relative_eq;

    fn setup_option(
        side: Side,
        option_style: OptionStyle,
        strike_price: Positive,
        underlying_price: Positive,
        quantity: Positive,
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
            f2p!(100.0),
            f2p!(105.0),
            f2p!(1.0),
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
            f2p!(100.0),
            f2p!(105.0),
            f2p!(10.0),
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
            f2p!(100.0),
            f2p!(105.0),
            f2p!(1.0),
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
            f2p!(100.0),
            f2p!(105.0),
            f2p!(10.0),
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
            f2p!(100.0),
            f2p!(105.0),
            f2p!(1.0),
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
            f2p!(100.0),
            f2p!(105.0),
            f2p!(10.0),
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
            f2p!(100.0),
            f2p!(105.0),
            f2p!(1.0),
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
            f2p!(100.0),
            f2p!(105.0),
            f2p!(10.0),
            30,
        );
        let position = Position::new(option, 5.0, Utc::now(), 1.0, 1.0);
        assert_relative_eq!(position.max_loss(), f64::INFINITY, epsilon = 0.001);
        assert_relative_eq!(position.max_profit(), 30.0, epsilon = 0.001);
    }
}

#[cfg(test)]
mod tests_update_from_option_data {
    use super::*;
    use crate::{f2p, spos};

    fn create_test_option_data() -> OptionData {
        OptionData::new(
            f2p!(110.0),
            spos!(9.5),
            spos!(10.0),
            spos!(8.5),
            spos!(9.0),
            spos!(0.25),
            Some(-0.3),
            None,
            None,
        )
    }

    #[test]
    fn test_update_long_call() {
        let mut position = Position::default();
        position.option.side = Side::Long;
        position.option.option_style = OptionStyle::Call;

        let option_data = create_test_option_data();
        position.update_from_option_data(&option_data);

        assert_eq!(position.option.strike_price, f2p!(110.0));
        assert_eq!(position.option.implied_volatility, 0.25);
        assert_eq!(position.premium, 10.0); // call_ask
    }

    #[test]
    fn test_update_short_call() {
        let mut position = Position::default();
        position.option.side = Side::Short;
        position.option.option_style = OptionStyle::Call;

        let option_data = create_test_option_data();
        position.update_from_option_data(&option_data);

        assert_eq!(position.premium, 9.5); // call_bid
    }

    #[test]
    fn test_update_long_put() {
        let mut position = Position::default();
        position.option.side = Side::Long;
        position.option.option_style = OptionStyle::Put;

        let option_data = create_test_option_data();
        position.update_from_option_data(&option_data);

        assert_eq!(position.premium, 9.0); // put_ask
    }

    #[test]
    fn test_update_short_put() {
        let mut position = Position::default();
        position.option.side = Side::Short;
        position.option.option_style = OptionStyle::Put;

        let option_data = create_test_option_data();
        position.update_from_option_data(&option_data);

        assert_eq!(position.premium, 8.5); // put_bid
    }
}

#[cfg(test)]
mod tests_premium {
    use super::*;
    use crate::f2p;

    fn setup_basic_position(side: Side) -> Position {
        let option = Options {
            side,
            quantity: f2p!(1.0),
            ..Default::default()
        };

        Position::new(option, 5.0, Utc::now(), 1.0, 1.0)
    }

    #[test]
    fn test_premium_received_long() {
        let position = setup_basic_position(Side::Long);
        assert_eq!(position.premium_received(), 0.0);
    }

    #[test]
    fn test_premium_received_short() {
        let position = setup_basic_position(Side::Short);
        assert_eq!(position.premium_received(), 5.0);
    }

    #[test]
    fn test_net_premium_received_long() {
        let position = setup_basic_position(Side::Long);
        assert_eq!(position.net_premium_received(), 0.0);
    }

    #[test]
    fn test_net_premium_received_short() {
        let position = setup_basic_position(Side::Short);
        assert_eq!(position.net_premium_received(), 3.0); // 5.0 - 2.0 (fees)
    }

    #[test]
    fn test_premium_received_with_quantity() {
        let side = Side::Short;
        let option = Options {
            side,
            quantity: f2p!(10.0),
            ..Default::default()
        };

        let position = Position::new(option, 5.0, Utc::now(), 1.0, 1.0);
        assert_eq!(position.premium_received(), 50.0);
    }
}

#[cfg(test)]
mod tests_pnl_calculator {
    use super::*;
    use crate::f2p;

    fn setup_test_position(side: Side, option_style: OptionStyle) -> Position {
        let option = Options {
            side,
            option_style,
            strike_price: f2p!(100.0),
            quantity: f2p!(1.0),
            underlying_price: f2p!(100.0),
            ..Default::default()
        };

        Position::new(option, 5.0, Utc::now(), 1.0, 1.0)
    }

    #[test]
    fn test_calculate_pnl_long_call() {
        let position = setup_test_position(Side::Long, OptionStyle::Call);
        let pnl = position.calculate_pnl(Utc::now(), f2p!(7.0));

        assert_eq!(pnl.unrealized.unwrap(), -0.0); // 7.0 - 7.0 (premium + fees)
        assert_eq!(position.total_cost(), 7.0);
        assert_eq!(position.premium_received(), 0.0);
    }

    #[test]
    fn test_calculate_pnl_short_call() {
        let position = setup_test_position(Side::Short, OptionStyle::Call);
        let pnl = position.calculate_pnl(Utc::now(), f2p!(3.0));

        assert_eq!(pnl.unrealized.unwrap(), 0.0); // 5.0 - 3.0 - 2.0 (fees)
        assert_eq!(position.total_cost(), 2.0);
        assert_eq!(position.premium_received(), 5.0);
    }

    #[test]
    fn test_calculate_pnl_at_expiration_long_call() {
        let position = setup_test_position(Side::Long, OptionStyle::Call);
        let pnl = position.calculate_pnl_at_expiration(Some(f2p!(110.0)));

        assert_eq!(pnl.realized.unwrap(), 3.0); // 10.0 - 7.0 (total cost)
        assert_eq!(position.total_cost(), 7.0);
        assert_eq!(position.premium_received(), 0.0);
    }

    #[test]
    fn test_calculate_pnl_at_expiration_short_put() {
        let position = setup_test_position(Side::Short, OptionStyle::Put);
        let pnl = position.calculate_pnl_at_expiration(Some(f2p!(90.0)));

        assert_eq!(pnl.realized.unwrap(), -7.0); // -10.0 + 5.0 (premium) - 2.0 (fees)
        assert_eq!(position.total_cost(), 2.0);
        assert_eq!(position.premium_received(), 5.0);
    }

    #[test]
    fn test_calculate_pnl_at_expiration_no_underlying() {
        let position = setup_test_position(Side::Long, OptionStyle::Call);
        let pnl = position.calculate_pnl_at_expiration(None);

        assert_eq!(pnl.realized.unwrap(), -7.0);
        assert_eq!(position.total_cost(), 7.0);
        assert_eq!(position.premium_received(), 0.0);
    }

    #[test]
    #[should_panic]
    fn test_calculate_pnl_at_zero_price() {
        let position = setup_test_position(Side::Long, OptionStyle::Call);
        let _ = position.calculate_pnl(Utc::now(), f2p!(0.0));
    }
}

#[cfg(test)]
mod tests_graph {
    use super::*;
    use crate::f2p;

    #[test]
    fn test_title() {
        let position = Position::default();
        assert_eq!(position.title(), position.option.title());
    }

    #[test]
    fn test_get_values() {
        let position = Position::default();
        let prices = vec![f2p!(90.0), f2p!(100.0), f2p!(110.0)];
        let values = position.get_values(&prices);

        assert_eq!(values.len(), 3);
        assert!(!values.iter().any(|&x| x.is_nan()));
    }

    #[test]
    fn test_get_vertical_lines() {
        let position = Position::default();
        let lines = position.get_vertical_lines();
        assert_eq!(lines.len(), 0);
    }
}
