/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 18/8/24
******************************************************************************/
use crate::Options;
use crate::chains::chain::OptionData;
use crate::error::position::PositionValidationErrorKind;
use crate::error::{GreeksError, PositionError};
use crate::greeks::Greeks;
use crate::model::types::{ExpirationDate, OptionStyle, Side};
use crate::pnl::utils::{PnL, PnLCalculator};
use crate::pricing::payoff::Profit;
use crate::visualization::model::ChartVerticalLine;
use crate::visualization::utils::Graph;
use crate::{Positive, pos};
use chrono::{DateTime, Utc};
use num_traits::ToPrimitive;
use plotters::prelude::{BLACK, ShapeStyle};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::error::Error;
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
/// - `new(option: Options, premium: Positive, date: DateTime<Utc>, open_fee: Positive, close_fee: Positive) -> Self`
///   Creates a new `Position` instance.
/// - `total_cost(&self) -> Result<Positive, PositionError>`
///   Calculates the total cost including the premium and open fee.
/// - `unrealized_pnl(&self, Positive: Positive) -> Result<Decimal, PositionError>`
///   Calculates the unrealized profit or loss at the current price.
/// - `pnl_at_expiration(&self, price: &Option<Positive>) -> Result<Decimal, Box<dyn Error>>`
///   Calculates the realized profit or loss at the closing price.
/// - `days_held(&self) -> Result<Positive, PositionError>`
///   Returns the number of days the position has been held.
/// - `days_to_expiration(&self) -> Result<Positive, PositionError>`
///   Returns the number of days until the option expires.
/// - `is_long(&self) -> bool`
///   Checks if the position is a long position.
/// - `is_short(&self) -> bool`
///   Checks if the position is a short position.
///
/// The `Greeks` trait is also implemented for the `Position` struct, allowing
/// calculations related to options' sensitivities (e.g., Delta, Gamma).
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Position {
    pub option: Options,
    pub premium: Positive,
    pub date: DateTime<Utc>,
    pub open_fee: Positive,
    pub close_fee: Positive,
}

impl Position {
    pub fn new(
        option: Options,
        premium: Positive,
        date: DateTime<Utc>,
        open_fee: Positive,
        close_fee: Positive,
    ) -> Self {
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

        match (self.option.side, self.option.option_style) {
            (Side::Long, OptionStyle::Call) => {
                self.premium = option_data.call_ask.unwrap();
            }
            (Side::Long, OptionStyle::Put) => {
                self.premium = option_data.put_ask.unwrap();
            }
            (Side::Short, OptionStyle::Call) => {
                self.premium = option_data.call_bid.unwrap();
            }
            (Side::Short, OptionStyle::Put) => {
                self.premium = option_data.put_bid.unwrap();
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
    pub fn total_cost(&self) -> Result<Positive, PositionError> {
        let total_cost = match self.option.side {
            Side::Long => (self.premium + self.open_fee + self.close_fee) * self.option.quantity,
            Side::Short => self.fees()?,
        };

        Ok(total_cost)
    }

    pub fn premium_received(&self) -> Result<Positive, PositionError> {
        match self.option.side {
            Side::Long => Ok(Positive::ZERO),
            Side::Short => Ok(self.premium * self.option.quantity),
        }
    }

    pub fn net_premium_received(&self) -> Result<Positive, PositionError> {
        match self.option.side {
            Side::Long => Ok(Positive::ZERO),
            Side::Short => {
                // max profit is premium received - fees (cost)
                let premium = self.premium * self.option.quantity;
                let cost = -self.total_cost()?.to_dec();
                match premium > cost {
                    true => Ok(premium + cost),
                    false => Err(PositionError::ValidationError(
                        PositionValidationErrorKind::InvalidPosition {
                            reason: "Max profit is negative.".to_string(),
                        },
                    )),
                }
            }
        }
    }

    pub fn pnl_at_expiration(
        // payoff
        &self,
        price: &Option<&Positive>,
    ) -> Result<Decimal, Box<dyn Error>> {
        match price {
            None => Ok(self.option.intrinsic_value(self.option.underlying_price)?
                - self.total_cost()?
                + self.premium_received()?),
            Some(price) => Ok(self.option.intrinsic_value(**price)? - self.total_cost()?
                + self.premium_received()?),
        }
    }

    pub fn unrealized_pnl(&self, price: Positive) -> Result<Decimal, PositionError> {
        match self.option.side {
            Side::Long => Ok((price.to_dec()
                - self.premium.to_dec()
                - self.open_fee.to_dec()
                - self.close_fee.to_dec())
                * self.option.quantity),
            Side::Short => Ok((self.premium.to_dec()
                - price.to_dec()
                - self.open_fee.to_dec()
                - self.close_fee.to_dec())
                * self.option.quantity),
        }
    }

    pub fn days_held(&self) -> Result<Positive, PositionError> {
        Ok(pos!((Utc::now() - self.date).num_days() as f64))
    }

    pub fn days_to_expiration(&self) -> Result<Positive, PositionError> {
        match self.option.expiration_date {
            ExpirationDate::Days(days) => Ok(days),
            ExpirationDate::DateTime(datetime) => Ok(pos!(
                datetime.signed_duration_since(Utc::now()).num_days() as f64
            )),
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
    /// A `Decimal` representing the net cost of the position.
    /// The value should be positive but if the fee is higher than the premium it will be negative
    /// in short positions
    pub fn net_cost(&self) -> Result<Decimal, PositionError> {
        match self.option.side {
            Side::Long => Ok(self.total_cost()?.to_dec()),
            Side::Short => {
                let fees = self.fees()?.to_dec();
                let premium = self.premium_received()?.to_dec();
                Ok(fees - premium)
            }
        }
    }

    pub fn break_even(&self) -> Option<Positive> {
        if self.option.quantity == Positive::ZERO {
            return None;
        }
        let total_cost_per_contract = self.total_cost().unwrap() / self.option.quantity;
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
                Some(self.option.strike_price - self.premium + total_cost_per_contract)
            }
        }
    }

    #[allow(dead_code)]
    pub(crate) fn max_profit(&self) -> Result<Positive, PositionError> {
        match self.option.side {
            Side::Long => Ok(Positive::INFINITY),
            Side::Short => self.net_premium_received(),
        }
    }

    #[allow(dead_code)]
    pub(crate) fn max_loss(&self) -> Result<Positive, PositionError> {
        match self.option.side {
            Side::Long => self.total_cost(),
            Side::Short => Ok(Positive::INFINITY),
        }
    }

    pub fn fees(&self) -> Result<Positive, PositionError> {
        Ok((self.open_fee + self.close_fee) * self.option.quantity)
    }

    pub fn validate(&self) -> bool {
        if self.option.side == Side::Short {
            if self.premium == Positive::ZERO {
                debug!("Premium must be greater than zero for short positions.");
                return false;
            }
            if self.premium < self.open_fee + self.close_fee {
                debug!("Premium must be greater than the sum of the fees.");
                return false;
            }
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
            premium: Positive::ZERO,
            date: Utc::now(),
            open_fee: Positive::ZERO,
            close_fee: Positive::ZERO,
        }
    }
}

impl Greeks for Position {
    fn get_options(&self) -> Result<Vec<&Options>, GreeksError> {
        Ok(vec![&self.option])
    }
}

impl PnLCalculator for Position {
    fn calculate_pnl(
        &self,
        underlying_price: &Positive,
        expiration_date: ExpirationDate,
        implied_volatility: &Positive,
    ) -> Result<PnL, Box<dyn Error>> {
        let price_at_buy = self.option.calculate_price_black_scholes()?;
        let mut current_option = self.option.clone();
        current_option.expiration_date = expiration_date;
        current_option.underlying_price = *underlying_price;
        current_option.implied_volatility = *implied_volatility;
        let price_at_sell = current_option.calculate_price_black_scholes()?;
        let unrealized = price_at_sell - price_at_buy;
        let initial_cost = self.total_cost()?;
        let initial_income = self.premium_received()?;
        Ok(PnL::new(
            None,
            Some(unrealized),
            initial_cost,
            initial_income,
            self.date,
        ))
    }

    fn calculate_pnl_at_expiration(
        &self,
        underlying_price: &Positive,
    ) -> Result<PnL, Box<dyn Error>> {
        let realized = self.pnl_at_expiration(&Some(underlying_price))?;
        let initial_cost = self.total_cost()?;
        let initial_income = self.premium_received()?;
        let date_time = self.option.expiration_date.get_date()?;

        Ok(PnL::new(
            Some(realized),
            None,
            initial_cost,
            initial_income,
            date_time,
        ))
    }
}

impl Profit for Position {
    fn calculate_profit_at(&self, price: Positive) -> Result<Decimal, Box<dyn Error>> {
        self.pnl_at_expiration(&Some(&price))
    }
}

impl Graph for Position {
    fn title(&self) -> String {
        self.option.title()
    }

    fn get_values(&self, data: &[Positive]) -> Vec<f64> {
        data.iter()
            .map(|&price| {
                self.pnl_at_expiration(&Some(&price))
                    .unwrap()
                    .to_f64()
                    .unwrap()
            })
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
    use crate::pos;
    use chrono::Duration;
    use rust_decimal_macros::dec;

    fn setup_option(
        side: Side,
        option_style: OptionStyle,
        strike_price: Positive,
        underlying_price: Positive,
        quantity: Positive,
        expiration_days: Positive,
    ) -> Options {
        Options {
            option_type: OptionType::European,
            side,
            underlying_symbol: "".to_string(),
            strike_price,
            expiration_date: ExpirationDate::Days(expiration_days),
            implied_volatility: pos!(0.2),
            quantity,
            underlying_price,
            risk_free_rate: dec!(0.01),
            option_style,
            dividend_yield: Positive::ZERO,
            exotic_params: None,
        }
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_position_total_cost() {
        let option = setup_option(
            Side::Long,
            OptionStyle::Call,
            pos!(100.0),
            pos!(105.0),
            pos!(1.0),
            pos!(30.0),
        );
        let position = Position::new(option, pos!(5.0), Utc::now(), Positive::ONE, Positive::ONE);
        assert_eq!(
            position.total_cost().unwrap(),
            7.0,
            "Total cost calculation is incorrect."
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_position_total_cost_size() {
        let option = setup_option(
            Side::Long,
            OptionStyle::Call,
            pos!(100.0),
            pos!(105.0),
            pos!(10.0),
            pos!(30.0),
        );
        let position = Position::new(option, pos!(5.0), Utc::now(), Positive::ONE, Positive::ONE);
        assert_eq!(
            position.total_cost().unwrap(),
            70.0,
            "Total cost calculation is incorrect."
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_position_total_cost_short() {
        let option = setup_option(
            Side::Short,
            OptionStyle::Call,
            pos!(100.0),
            pos!(105.0),
            pos!(1.0),
            pos!(30.0),
        );
        let position = Position::new(option, pos!(5.0), Utc::now(), Positive::ONE, Positive::ONE);
        assert_eq!(
            position.total_cost().unwrap(),
            2.0,
            "Total cost calculation is incorrect."
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_position_total_cost_short_size() {
        let option = setup_option(
            Side::Short,
            OptionStyle::Call,
            pos!(100.0),
            pos!(105.0),
            pos!(10.0),
            pos!(30.0),
        );
        let position = Position::new(option, pos!(5.0), Utc::now(), Positive::ONE, Positive::ONE);
        assert_eq!(
            position.total_cost().unwrap(),
            20.0,
            "Total cost calculation is incorrect."
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_position_check_negative_premium() {
        let option = setup_option(
            Side::Long,
            OptionStyle::Call,
            pos!(100.0),
            pos!(110.0),
            pos!(1.0),
            Positive::ZERO,
        );
        let position = Position::new(option, pos!(5.0), Utc::now(), Positive::ONE, Positive::ONE);
        assert_eq!(
            position.pnl_at_expiration(&None).unwrap(),
            dec!(3.0),
            "PNL at expiration for long call ITM is incorrect."
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_position_pnl_at_expiration_long_call_itm() {
        let option = setup_option(
            Side::Long,
            OptionStyle::Call,
            pos!(100.0),
            pos!(110.0),
            pos!(1.0),
            Positive::ZERO,
        );
        let position = Position::new(option, pos!(5.0), Utc::now(), Positive::ONE, Positive::ONE);
        assert_eq!(
            position.pnl_at_expiration(&None).unwrap(),
            dec!(3.0),
            "PNL at expiration for long call ITM is incorrect."
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_position_pnl_at_expiration_long_call_itm_quantity() {
        let option = setup_option(
            Side::Long,
            OptionStyle::Call,
            pos!(100.0),
            pos!(110.0),
            pos!(10.0),
            Positive::ZERO,
        );
        let position = Position::new(option, pos!(5.0), Utc::now(), Positive::ONE, Positive::ONE);
        assert_eq!(
            position.pnl_at_expiration(&None).unwrap(),
            dec!(30.0),
            "PNL at expiration for long call ITM is incorrect."
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_position_pnl_at_expiration_short_call_itm() {
        let option = setup_option(
            Side::Short,
            OptionStyle::Call,
            pos!(100.0),
            pos!(110.0),
            pos!(1.0),
            Positive::ZERO,
        );
        let position = Position::new(option, pos!(5.0), Utc::now(), Positive::ONE, Positive::ONE);
        assert_eq!(
            position.pnl_at_expiration(&None).unwrap().to_f64().unwrap(),
            -7.0,
            "PNL at expiration for short call ITM is incorrect."
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_position_pnl_at_expiration_short_call_itm_quantity() {
        let option = setup_option(
            Side::Short,
            OptionStyle::Call,
            pos!(100.0),
            pos!(110.0),
            pos!(10.0),
            Positive::ZERO,
        );
        let position = Position::new(option, pos!(5.0), Utc::now(), Positive::ONE, Positive::ONE);
        assert_eq!(
            position.pnl_at_expiration(&None).unwrap().to_f64().unwrap(),
            -70.0,
            "PNL at expiration for short call ITM is incorrect."
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_position_pnl_at_expiration_long_put_itm() {
        let option = setup_option(
            Side::Long,
            OptionStyle::Put,
            pos!(100.0),
            pos!(90.0),
            pos!(1.0),
            Positive::ZERO,
        );
        let position = Position::new(option, pos!(5.0), Utc::now(), Positive::ONE, Positive::ONE);
        assert_eq!(
            position.pnl_at_expiration(&None).unwrap().to_f64().unwrap(),
            3.0,
            "PNL at expiration for long put ITM is incorrect."
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_position_pnl_at_expiration_long_put_itm_quantity() {
        let option = setup_option(
            Side::Long,
            OptionStyle::Put,
            pos!(100.0),
            pos!(90.0),
            pos!(10.0),
            Positive::ZERO,
        );
        let position = Position::new(option, pos!(5.0), Utc::now(), Positive::ONE, Positive::ONE);
        assert_eq!(
            position.pnl_at_expiration(&None).unwrap().to_f64().unwrap(),
            30.0,
            "PNL at expiration for long put ITM is incorrect."
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_position_pnl_at_expiration_short_put_itm() {
        let option = setup_option(
            Side::Short,
            OptionStyle::Put,
            pos!(100.0),
            pos!(90.0),
            pos!(1.0),
            Positive::ZERO,
        );
        let position = Position::new(option, pos!(5.0), Utc::now(), Positive::ONE, Positive::ONE);
        assert_eq!(
            position.pnl_at_expiration(&None).unwrap().to_f64().unwrap(),
            -7.0,
            "PNL at expiration for short put ITM is incorrect."
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_position_pnl_at_expiration_short_put_itm_quantity() {
        let option = setup_option(
            Side::Short,
            OptionStyle::Put,
            pos!(100.0),
            pos!(90.0),
            pos!(10.0),
            Positive::ZERO,
        );
        let position = Position::new(option, pos!(5.0), Utc::now(), Positive::ONE, Positive::ONE);
        assert_eq!(
            position.pnl_at_expiration(&None).unwrap().to_f64().unwrap(),
            -70.0,
            "PNL at expiration for short put ITM is incorrect."
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_position_pnl_at_expiration_short_put_itm_winning() {
        let option = setup_option(
            Side::Short,
            OptionStyle::Put,
            pos!(100.0),
            pos!(110.0),
            pos!(1.0),
            Positive::ZERO,
        );
        let position = Position::new(option, pos!(5.0), Utc::now(), Positive::ONE, Positive::ONE);
        assert_eq!(
            position.pnl_at_expiration(&None).unwrap().to_f64().unwrap(),
            3.0,
            "PNL at expiration for short put ITM is incorrect."
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_position_pnl_at_expiration_short_put_itm_quantity_winning() {
        let option = setup_option(
            Side::Short,
            OptionStyle::Put,
            pos!(100.0),
            pos!(110.0),
            pos!(10.0),
            Positive::ZERO,
        );
        let position = Position::new(option, pos!(5.0), Utc::now(), Positive::ONE, Positive::ONE);
        assert_eq!(
            position.pnl_at_expiration(&None).unwrap().to_f64().unwrap(),
            30.0,
            "PNL at expiration for short put ITM is incorrect."
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_unrealized_pnl_long_call() {
        let option = setup_option(
            Side::Long,
            OptionStyle::Call,
            pos!(100.0),
            pos!(105.0),
            pos!(1.0),
            pos!(30.0),
        );
        let position = Position::new(option, pos!(5.0), Utc::now(), Positive::ONE, Positive::ONE);
        assert_eq!(
            position.pnl_at_expiration(&Some(&pos!(107.0))).unwrap(),
            Positive::ZERO,
            "Unrealized PNL for long call is incorrect."
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_unrealized_pnl_long_call_quantity() {
        let option = setup_option(
            Side::Long,
            OptionStyle::Call,
            pos!(100.0),
            pos!(105.0),
            pos!(10.0),
            pos!(30.0),
        );
        let position = Position::new(option, pos!(5.0), Utc::now(), Positive::ONE, Positive::ONE);
        assert_eq!(
            position
                .pnl_at_expiration(&Some(&pos!(107.0)))
                .unwrap()
                .to_f64()
                .unwrap(),
            ZERO,
            "Unrealized PNL for long call is incorrect."
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_unrealized_pnl_short_call() {
        let option = setup_option(
            Side::Short,
            OptionStyle::Call,
            pos!(100.0),
            pos!(105.0),
            pos!(10.0),
            pos!(30.0),
        );
        let position = Position::new(option, pos!(5.0), Utc::now(), Positive::ONE, Positive::ONE);
        assert_eq!(
            position
                .unrealized_pnl(pos!(3.0))
                .unwrap()
                .to_f64()
                .unwrap(),
            ZERO,
            "Unrealized PNL for short call is incorrect."
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_unrealized_pnl_short_call_bis() {
        let option = setup_option(
            Side::Short,
            OptionStyle::Call,
            pos!(100.0),
            pos!(105.0),
            pos!(1.0),
            pos!(30.0),
        );
        let position = Position::new(option, pos!(5.0), Utc::now(), Positive::ONE, Positive::ONE);
        assert_eq!(
            position
                .unrealized_pnl(pos!(10.0))
                .unwrap()
                .to_f64()
                .unwrap(),
            -7.0,
            "Unrealized PNL for short call is incorrect."
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_days_held() {
        let option = setup_option(
            Side::Long,
            OptionStyle::Call,
            pos!(100.0),
            pos!(105.0),
            pos!(10.0),
            pos!(30.0),
        );
        let date = Utc::now() - Duration::days(10);
        let position = Position::new(option, pos!(5.0), date, Positive::ONE, Positive::ONE);
        assert_eq!(
            position.days_held().unwrap().to_f64(),
            10.0,
            "Days held calculation is incorrect."
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_days_to_expiration() {
        let option = setup_option(
            Side::Long,
            OptionStyle::Call,
            pos!(100.0),
            pos!(105.0),
            pos!(10.0),
            pos!(30.0),
        );
        let position = Position::new(option, pos!(5.0), Utc::now(), Positive::ONE, Positive::ONE);
        assert_eq!(
            position.days_to_expiration().unwrap().to_f64(),
            30.0,
            "Days to expiration calculation is incorrect."
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_is_long_position() {
        let option = setup_option(
            Side::Long,
            OptionStyle::Call,
            pos!(100.0),
            pos!(105.0),
            pos!(10.0),
            pos!(30.0),
        );
        let position = Position::new(option, pos!(5.0), Utc::now(), Positive::ONE, Positive::ONE);
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
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_is_short_position() {
        let option = setup_option(
            Side::Short,
            OptionStyle::Call,
            pos!(100.0),
            pos!(105.0),
            pos!(10.0),
            pos!(30.0),
        );
        let position = Position::new(option, pos!(5.0), Utc::now(), Positive::ONE, Positive::ONE);
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
    use crate::model::utils::create_sample_position;
    use crate::pos;

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_valid_position() {
        let position = create_sample_position(
            OptionStyle::Call,
            Side::Short,
            pos!(90.0),
            pos!(1.0),
            pos!(95.0),
            pos!(0.2),
        );
        assert!(position.validate());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_zero_premium() {
        let mut position = create_sample_position(
            OptionStyle::Call,
            Side::Short,
            pos!(90.0),
            pos!(1.0),
            pos!(95.0),
            pos!(0.2),
        );
        position.premium = Positive::ZERO;
        assert!(!position.validate());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_invalid_option() {
        let mut position = create_sample_position(
            OptionStyle::Call,
            Side::Short,
            pos!(90.0),
            pos!(1.0),
            pos!(95.0),
            pos!(0.2),
        );
        position.option.strike_price = Positive::ZERO; // This makes the option invalid
        assert!(!position.validate());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_zero_fees() {
        let mut position = create_sample_position(
            OptionStyle::Call,
            Side::Short,
            pos!(90.0),
            pos!(1.0),
            pos!(95.0),
            pos!(0.2),
        );
        position.open_fee = Positive::ZERO;
        position.close_fee = Positive::ZERO;
        assert!(position.validate());
    }
}

#[cfg(test)]
mod tests_position_break_even {
    use super::*;
    use crate::model::types::{ExpirationDate, OptionStyle, OptionType, Side};
    use crate::pos;
    use rust_decimal_macros::dec;

    fn setup_option(
        side: Side,
        option_style: OptionStyle,
        strike_price: Positive,
        underlying_price: Positive,
        quantity: Positive,
        expiration_days: Positive,
    ) -> Options {
        Options {
            option_type: OptionType::European,
            side,
            underlying_symbol: "".to_string(),
            strike_price,
            expiration_date: ExpirationDate::Days(expiration_days),
            implied_volatility: pos!(0.2),
            quantity,
            underlying_price,
            risk_free_rate: dec!(0.01),
            option_style,
            dividend_yield: Positive::ZERO,
            exotic_params: None,
        }
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_unrealized_pnl_long_call() {
        let option = setup_option(
            Side::Long,
            OptionStyle::Call,
            pos!(100.0),
            pos!(105.0),
            pos!(1.0),
            pos!(30.0),
        );
        let position = Position::new(option, pos!(5.0), Utc::now(), Positive::ONE, Positive::ONE);
        assert_eq!(position.break_even().unwrap(), 107.0);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_unrealized_pnl_long_call_size() {
        let option = setup_option(
            Side::Long,
            OptionStyle::Call,
            pos!(100.0),
            pos!(105.0),
            pos!(10.0),
            pos!(30.0),
        );
        let position = Position::new(option, pos!(5.0), Utc::now(), Positive::ONE, Positive::ONE);
        assert_eq!(position.break_even().unwrap(), 107.0);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_unrealized_pnl_short_call() {
        let option = setup_option(
            Side::Short,
            OptionStyle::Call,
            pos!(100.0),
            pos!(105.0),
            pos!(1.0),
            pos!(30.0),
        );
        let position = Position::new(option, pos!(5.0), Utc::now(), Positive::ONE, Positive::ONE);
        assert_eq!(position.break_even().unwrap(), 103.0);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_unrealized_pnl_short_call_size() {
        let option = setup_option(
            Side::Short,
            OptionStyle::Call,
            pos!(100.0),
            pos!(105.0),
            pos!(10.0),
            pos!(30.0),
        );
        let position = Position::new(option, pos!(5.0), Utc::now(), Positive::ONE, Positive::ONE);
        assert_eq!(position.break_even().unwrap(), 103.0);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_unrealized_pnl_long_put() {
        let option = setup_option(
            Side::Long,
            OptionStyle::Put,
            pos!(100.0),
            pos!(105.0),
            pos!(1.0),
            pos!(30.0),
        );
        let position = Position::new(option, pos!(5.0), Utc::now(), Positive::ONE, Positive::ONE);
        assert_eq!(position.break_even().unwrap(), 93.0);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_unrealized_pnl_long_put_size() {
        let option = setup_option(
            Side::Long,
            OptionStyle::Put,
            pos!(100.0),
            pos!(105.0),
            pos!(10.0),
            pos!(30.0),
        );
        let position = Position::new(option, pos!(5.0), Utc::now(), Positive::ONE, Positive::ONE);
        assert_eq!(position.break_even().unwrap(), 93.0);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_unrealized_pnl_short_put() {
        let option = setup_option(
            Side::Short,
            OptionStyle::Put,
            pos!(100.0),
            pos!(105.0),
            pos!(1.0),
            pos!(30.0),
        );
        let position = Position::new(option, pos!(5.0), Utc::now(), Positive::ONE, Positive::ONE);
        assert_eq!(position.break_even().unwrap(), 97.0);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_unrealized_pnl_short_put_size() {
        let option = setup_option(
            Side::Short,
            OptionStyle::Put,
            pos!(100.0),
            pos!(105.0),
            pos!(10.0),
            pos!(30.0),
        );
        let position = Position::new(option, pos!(5.0), Utc::now(), Positive::ONE, Positive::ONE);
        assert_eq!(position.break_even().unwrap(), 97.0);
    }
}

#[cfg(test)]
mod tests_position_max_loss_profit {
    use super::*;
    use crate::model::types::{ExpirationDate, OptionStyle, OptionType, Side};
    use crate::pos;
    use approx::assert_relative_eq;
    use rust_decimal_macros::dec;

    fn setup_option(
        side: Side,
        option_style: OptionStyle,
        strike_price: Positive,
        underlying_price: Positive,
        quantity: Positive,
        expiration_days: Positive,
    ) -> Options {
        Options {
            option_type: OptionType::European,
            side,
            underlying_symbol: "".to_string(),
            strike_price,
            expiration_date: ExpirationDate::Days(expiration_days),
            implied_volatility: pos!(0.2),
            quantity,
            underlying_price,
            risk_free_rate: dec!(0.01),
            option_style,
            dividend_yield: Positive::ZERO,
            exotic_params: None,
        }
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_unrealized_pnl_long_call() {
        let option = setup_option(
            Side::Long,
            OptionStyle::Call,
            pos!(100.0),
            pos!(105.0),
            pos!(1.0),
            pos!(30.0),
        );
        let position = Position::new(option, pos!(5.0), Utc::now(), Positive::ONE, Positive::ONE);
        assert_relative_eq!(position.max_loss().unwrap().to_f64(), 7.0, epsilon = 0.001);
        assert_eq!(position.max_profit().unwrap(), Positive::INFINITY);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_unrealized_pnl_long_call_size() {
        let option = setup_option(
            Side::Long,
            OptionStyle::Call,
            pos!(100.0),
            pos!(105.0),
            pos!(10.0),
            pos!(30.0),
        );
        let position = Position::new(option, pos!(5.0), Utc::now(), Positive::ONE, Positive::ONE);
        assert_relative_eq!(position.max_loss().unwrap().to_f64(), 70.0, epsilon = 0.001);
        assert_eq!(position.max_profit().unwrap(), Positive::INFINITY);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_unrealized_pnl_short_call() {
        let option = setup_option(
            Side::Short,
            OptionStyle::Call,
            pos!(100.0),
            pos!(105.0),
            pos!(1.0),
            pos!(30.0),
        );
        let position = Position::new(option, pos!(5.0), Utc::now(), Positive::ONE, Positive::ONE);
        assert_relative_eq!(position.max_loss().unwrap(), Positive::INFINITY);
        assert_relative_eq!(
            position.max_profit().unwrap().to_f64(),
            3.0,
            epsilon = 0.001
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_unrealized_pnl_short_call_size() {
        let option = setup_option(
            Side::Short,
            OptionStyle::Call,
            pos!(100.0),
            pos!(105.0),
            pos!(10.0),
            pos!(30.0),
        );
        let position = Position::new(option, pos!(5.0), Utc::now(), Positive::ONE, Positive::ONE);
        assert_relative_eq!(position.max_loss().unwrap(), Positive::INFINITY);
        assert_relative_eq!(
            position.max_profit().unwrap().to_f64(),
            30.0,
            epsilon = 0.001
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_unrealized_pnl_long_put() {
        let option = setup_option(
            Side::Long,
            OptionStyle::Put,
            pos!(100.0),
            pos!(105.0),
            pos!(1.0),
            pos!(30.0),
        );
        let position = Position::new(option, pos!(5.0), Utc::now(), Positive::ONE, Positive::ONE);
        assert_relative_eq!(position.max_loss().unwrap().to_f64(), 7.0, epsilon = 0.001);
        assert_relative_eq!(position.max_profit().unwrap(), Positive::INFINITY);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_unrealized_pnl_long_put_size() {
        let option = setup_option(
            Side::Long,
            OptionStyle::Put,
            pos!(100.0),
            pos!(105.0),
            pos!(10.0),
            pos!(30.0),
        );
        let position = Position::new(option, pos!(5.0), Utc::now(), Positive::ONE, Positive::ONE);
        assert_relative_eq!(position.max_loss().unwrap().to_f64(), 70.0, epsilon = 0.001);
        assert_relative_eq!(position.max_profit().unwrap(), Positive::INFINITY);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_unrealized_pnl_short_put() {
        let option = setup_option(
            Side::Short,
            OptionStyle::Put,
            pos!(100.0),
            pos!(105.0),
            pos!(1.0),
            pos!(30.0),
        );
        let position = Position::new(option, pos!(5.0), Utc::now(), Positive::ONE, Positive::ONE);
        assert_relative_eq!(position.max_loss().unwrap(), Positive::INFINITY);
        assert_relative_eq!(
            position.max_profit().unwrap().to_f64(),
            3.0,
            epsilon = 0.001
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_unrealized_pnl_short_put_size() {
        let option = setup_option(
            Side::Short,
            OptionStyle::Put,
            pos!(100.0),
            pos!(105.0),
            pos!(10.0),
            pos!(30.0),
        );
        let position = Position::new(option, pos!(5.0), Utc::now(), Positive::ONE, Positive::ONE);
        assert_relative_eq!(position.max_loss().unwrap(), Positive::INFINITY);
        assert_relative_eq!(
            position.max_profit().unwrap().to_f64(),
            30.0,
            epsilon = 0.001
        );
    }
}

#[cfg(test)]
mod tests_update_from_option_data {
    use super::*;
    use crate::{pos, spos};
    use rust_decimal_macros::dec;

    fn create_test_option_data() -> OptionData {
        OptionData::new(
            pos!(110.0),
            spos!(9.5),
            spos!(10.0),
            spos!(8.5),
            spos!(9.0),
            spos!(0.25),
            Some(dec!(-0.3)),
            Some(dec!(0.3)),
            Some(dec!(0.3)),
            None,
            None,
        )
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_update_long_call() {
        let mut position = Position::default();
        position.option.side = Side::Long;
        position.option.option_style = OptionStyle::Call;

        let option_data = create_test_option_data();
        position.update_from_option_data(&option_data);

        assert_eq!(position.option.strike_price, pos!(110.0));
        assert_eq!(position.option.implied_volatility, 0.25);
        assert_eq!(position.premium, 10.0); // call_ask
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_update_short_call() {
        let mut position = Position::default();
        position.option.side = Side::Short;
        position.option.option_style = OptionStyle::Call;

        let option_data = create_test_option_data();
        position.update_from_option_data(&option_data);

        assert_eq!(position.premium, 9.5); // call_bid
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_update_long_put() {
        let mut position = Position::default();
        position.option.side = Side::Long;
        position.option.option_style = OptionStyle::Put;

        let option_data = create_test_option_data();
        position.update_from_option_data(&option_data);

        assert_eq!(position.premium, 9.0); // put_ask
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
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
    use crate::pos;

    fn setup_basic_position(side: Side) -> Position {
        let option = Options {
            side,
            quantity: pos!(1.0),
            ..Default::default()
        };

        Position::new(option, pos!(5.0), Utc::now(), Positive::ONE, Positive::ONE)
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_premium_received_long() {
        let position = setup_basic_position(Side::Long);
        assert_eq!(position.premium_received().unwrap(), Positive::ZERO);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_premium_received_short() {
        let position = setup_basic_position(Side::Short);
        assert_eq!(position.premium_received().unwrap(), 5.0);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_net_premium_received_long() {
        let position = setup_basic_position(Side::Long);
        assert_eq!(position.net_premium_received().unwrap(), 0.0);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_net_premium_received_short() {
        let position = setup_basic_position(Side::Short);
        assert_eq!(position.net_premium_received().unwrap(), 3.0); // 5.0 - 2.0 (fees)
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_premium_received_with_quantity() {
        let side = Side::Short;
        let option = Options {
            side,
            quantity: pos!(10.0),
            ..Default::default()
        };

        let position = Position::new(option, pos!(5.0), Utc::now(), Positive::ONE, Positive::ONE);
        assert_eq!(position.premium_received().unwrap(), 50.0);
    }
}

#[cfg(test)]
mod tests_pnl_calculator {
    use super::*;
    use crate::{OptionType, assert_decimal_eq, pos};
    use rust_decimal_macros::dec;

    fn setup_test_position(side: Side, option_style: OptionStyle) -> Position {
        let option = Options::new(
            OptionType::European,
            side,
            "AAPL".to_string(),
            pos!(100.0),
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),
            Positive::ONE,
            pos!(100.0),
            dec!(0.05),
            option_style,
            pos!(0.01),
            None,
        );

        Position::new(option, pos!(5.0), Utc::now(), Positive::ONE, Positive::ONE)
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_calculate_pnl_long_call_no_changes() {
        let position = setup_test_position(Side::Long, OptionStyle::Call);
        let pnl = position
            .calculate_pnl(&pos!(100.0), ExpirationDate::Days(pos!(30.0)), &pos!(0.2))
            .unwrap();

        assert_eq!(pnl.unrealized.unwrap(), Decimal::ZERO); // 5.0 - 2.4933 - 2.0 (fees)
        assert_eq!(position.total_cost().unwrap(), 7.0);
        assert_eq!(position.premium_received().unwrap(), 0.0);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_calculate_pnl_long_call_price_up() {
        let position = setup_test_position(Side::Long, OptionStyle::Call);
        let pnl = position
            .calculate_pnl(&pos!(107.0), ExpirationDate::Days(pos!(30.0)), &pos!(0.2))
            .unwrap();

        assert_decimal_eq!(pnl.unrealized.unwrap(), dec!(5.2150), dec!(0.0001));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_calculate_pnl_long_call_vol_down() {
        let position = setup_test_position(Side::Long, OptionStyle::Call);
        let pnl = position
            .calculate_pnl(&pos!(100.0), ExpirationDate::Days(pos!(30.0)), &pos!(0.1))
            .unwrap();

        assert_decimal_eq!(pnl.unrealized.unwrap(), dec!(-1.1352), dec!(0.0001));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_calculate_pnl_long_call_date_closer() {
        let position = setup_test_position(Side::Long, OptionStyle::Call);
        let pnl = position
            .calculate_pnl(&pos!(100.0), ExpirationDate::Days(pos!(3.0)), &pos!(0.2))
            .unwrap();

        assert_decimal_eq!(pnl.unrealized.unwrap(), dec!(-1.7494), dec!(0.0001));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_calculate_pnl_short_call_no_changes() {
        let position = setup_test_position(Side::Short, OptionStyle::Call);
        let pnl = position
            .calculate_pnl(&pos!(100.0), ExpirationDate::Days(pos!(30.0)), &pos!(0.2))
            .unwrap();

        assert_eq!(pnl.unrealized.unwrap(), Decimal::ZERO); // 5.0 - 2.4933 - 2.0 (fees)
        assert_eq!(position.total_cost().unwrap(), 2.0);
        assert_eq!(position.premium_received().unwrap(), 5.0);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_calculate_pnl_short_call_price_up() {
        let position = setup_test_position(Side::Short, OptionStyle::Call);
        let pnl = position
            .calculate_pnl(&pos!(107.0), ExpirationDate::Days(pos!(30.0)), &pos!(0.2))
            .unwrap();

        assert_decimal_eq!(pnl.unrealized.unwrap(), dec!(-5.2150), dec!(0.0001));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_calculate_pnl_short_call_price_down() {
        let position = setup_test_position(Side::Short, OptionStyle::Call);
        let pnl = position
            .calculate_pnl(&pos!(97.0), ExpirationDate::Days(pos!(30.0)), &pos!(0.2))
            .unwrap();

        assert_decimal_eq!(pnl.unrealized.unwrap(), dec!(1.3069), dec!(0.0001));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_calculate_pnl_short_call_vol_down() {
        let position = setup_test_position(Side::Short, OptionStyle::Call);
        let pnl = position
            .calculate_pnl(&pos!(100.0), ExpirationDate::Days(pos!(30.0)), &pos!(0.1))
            .unwrap();

        assert_decimal_eq!(pnl.unrealized.unwrap(), dec!(1.1352), dec!(0.0001));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_calculate_pnl_short_call_vol_up() {
        let position = setup_test_position(Side::Short, OptionStyle::Call);
        let pnl = position
            .calculate_pnl(&pos!(100.0), ExpirationDate::Days(pos!(30.0)), &pos!(0.3))
            .unwrap();

        assert_decimal_eq!(pnl.unrealized.unwrap(), dec!(-1.1386), dec!(0.0001));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_calculate_pnl_short_call_date_closer() {
        let position = setup_test_position(Side::Short, OptionStyle::Call);
        let pnl = position
            .calculate_pnl(&pos!(100.0), ExpirationDate::Days(pos!(3.0)), &pos!(0.2))
            .unwrap();

        assert_decimal_eq!(pnl.unrealized.unwrap(), dec!(1.7494), dec!(0.0001));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_calculate_pnl_short_call_date_further() {
        let position = setup_test_position(Side::Short, OptionStyle::Call);
        let pnl = position
            .calculate_pnl(&pos!(100.0), ExpirationDate::Days(pos!(40.0)), &pos!(0.2))
            .unwrap();

        assert_decimal_eq!(pnl.unrealized.unwrap(), dec!(-0.4224), dec!(0.0001));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_calculate_pnl_at_expiration_long_call() {
        let position = setup_test_position(Side::Long, OptionStyle::Call);
        let pnl = position.calculate_pnl_at_expiration(&pos!(110.0)).unwrap();

        assert_eq!(pnl.realized.unwrap(), dec!(3.0)); // 10.0 - 7.0 (total cost)
        assert_eq!(position.total_cost().unwrap(), 7.0);
        assert_eq!(position.premium_received().unwrap(), 0.0);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_calculate_pnl_at_expiration_short_put() {
        let position = setup_test_position(Side::Short, OptionStyle::Put);
        let pnl = position.calculate_pnl_at_expiration(&pos!(90.0)).unwrap();

        assert_eq!(pnl.realized.unwrap(), dec!(-7.0)); // -10.0 + 5.0 (premium) - 2.0 (fees)
        assert_eq!(position.total_cost().unwrap(), 2.0);
        assert_eq!(position.premium_received().unwrap(), 5.0);
    }
}

#[cfg(test)]
mod tests_graph {
    use super::*;
    use crate::pos;

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_title() {
        let position = Position::default();
        assert_eq!(position.title(), position.option.title());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_values() {
        let position = Position::default();
        let prices = vec![pos!(90.0), pos!(100.0), pos!(110.0)];
        let values = position.get_values(&prices);

        assert_eq!(values.len(), 3);
        assert!(!values.iter().any(|&x| x.is_nan()));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_vertical_lines() {
        let position = Position::default();
        let lines = position.get_vertical_lines();
        assert_eq!(lines.len(), 0);
    }
}

#[cfg(test)]
mod tests_position_serde {
    use super::*;
    use crate::model::utils::create_sample_position;
    use crate::pos;
    use serde_json;

    #[test]
    fn test_position_serialization() {
        let position = create_sample_position(
            OptionStyle::Call,
            Side::Short,
            pos!(90.0),
            pos!(1.0),
            pos!(95.0),
            pos!(0.2),
        );
        let serialized = serde_json::to_string(&position).unwrap();

        // Verify the serialized string contains expected fields
        assert!(serialized.contains("\"option\""));
        assert!(serialized.contains("\"premium\""));
        assert!(serialized.contains("\"date\""));
        assert!(serialized.contains("\"open_fee\""));
        assert!(serialized.contains("\"close_fee\""));
        assert!(serialized.contains("AAPL"));
        assert!(serialized.contains("95"));
    }

    #[test]
    fn test_position_deserialization() {
        let position = create_sample_position(
            OptionStyle::Call,
            Side::Short,
            pos!(90.0),
            pos!(1.0),
            pos!(95.0),
            pos!(0.2),
        );
        let serialized = serde_json::to_string(&position).unwrap();
        let deserialized: Position = serde_json::from_str(&serialized).unwrap();

        assert_eq!(position, deserialized);
        assert_eq!(deserialized.option.underlying_symbol, "AAPL");
        assert_eq!(deserialized.option.strike_price, pos!(95.0));
        assert_eq!(deserialized.premium, pos!(5.0));
        assert_eq!(deserialized.open_fee, pos!(0.5));
        assert_eq!(deserialized.close_fee, pos!(0.5));
    }

    #[test]
    fn test_position_json_structure() {
        let position = create_sample_position(
            OptionStyle::Call,
            Side::Short,
            pos!(90.0),
            pos!(1.0),
            pos!(95.0),
            pos!(0.2),
        );
        let serialized = serde_json::to_string_pretty(&position).unwrap();

        // Print the pretty JSON for debugging
        println!("Serialized Position:\n{}", serialized);

        let value: serde_json::Value = serde_json::from_str(&serialized).unwrap();

        // Test structure
        assert!(value.is_object());
        assert!(value.get("option").is_some());
        assert!(value.get("premium").is_some());
        assert!(value.get("date").is_some());
        assert!(value.get("open_fee").is_some());
        assert!(value.get("close_fee").is_some());
    }

    #[test]
    fn test_position_deserialize_invalid_json() {
        let invalid_json = r#"{
            "option": null,
            "premium": 5.0,
            "date": "2024-01-01T00:00:00Z",
            "open_fee": 1.0,
            "close_fee": 1.0
        }"#;

        let result: Result<Position, serde_json::Error> = serde_json::from_str(invalid_json);
        assert!(result.is_err());
    }

    #[test]
    fn test_position_roundtrip() {
        let original = create_sample_position(
            OptionStyle::Call,
            Side::Short,
            pos!(90.0),
            pos!(1.0),
            pos!(95.0),
            pos!(0.2),
        );
        let serialized = serde_json::to_string(&original).unwrap();
        let deserialized: Position = serde_json::from_str(&serialized).unwrap();
        let reserialized = serde_json::to_string(&deserialized).unwrap();

        assert_eq!(serialized, reserialized);
        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_position_with_different_option_types() {
        // Test with a Put option
        let put_position = create_sample_position(
            OptionStyle::Put,
            Side::Short,
            pos!(90.0),
            pos!(1.0),
            pos!(95.0),
            pos!(0.2),
        );

        let serialized = serde_json::to_string(&put_position).unwrap();
        let deserialized: Position = serde_json::from_str(&serialized).unwrap();

        assert_eq!(put_position, deserialized);
        assert_eq!(deserialized.option.option_style, OptionStyle::Put);

        // Test with a Short position
        let short_position = create_sample_position(
            OptionStyle::Call,
            Side::Short,
            pos!(90.0),
            pos!(1.0),
            pos!(95.0),
            pos!(0.2),
        );

        let serialized = serde_json::to_string(&short_position).unwrap();
        let deserialized: Position = serde_json::from_str(&serialized).unwrap();

        assert_eq!(short_position, deserialized);
        assert_eq!(deserialized.option.side, Side::Short);
    }
}
