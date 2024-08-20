/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 18/8/24
******************************************************************************/
use crate::constants::ZERO;
use crate::greeks::equations::{Greek, Greeks};
use crate::model::option::Options;
use crate::model::types::{ExpirationDate, Side};
use crate::pnl::utils::{PnL, PnLCalculator};
use crate::visualization::utils::Graph;
use chrono::{DateTime, Utc};
use plotters::element::{Circle, EmptyElement, Text};
use plotters::prelude::*;
use std::error::Error;

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

    fn check_premium(mut premium: f64) -> f64 {
        if premium < ZERO {
            premium *= -1.0;
        }
        premium
    }

    fn break_even(&self) -> f64 {
        // TODO: tests it
        match self.option.side {
            Side::Long => self.option.strike_price + self.premium - self.total_cost(),
            Side::Short => self.option.strike_price - self.premium - self.total_cost(),
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
    fn graph(&self, data: &[f64], file_path: &str) -> Result<(), Box<dyn Error>> {
        // Generate PNL at expiration for each price in the data vector
        let pnl_values: Vec<f64> = data
            .iter()
            .map(|&price| self.pnl_at_expiration(Some(price)))
            .collect();

        // Set up the drawing area with a 1200x800 pixel canvas
        let root = BitMapBackend::new(file_path, (1200, 800)).into_drawing_area();
        root.fill(&WHITE)?;

        // Determine the range for the X and Y axes
        let max_price = data.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let min_price = data.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_pnl_value = pnl_values.iter().cloned().fold(f64::NEG_INFINITY, f64::max) * 1.2;
        let min_pnl_value = pnl_values.iter().cloned().fold(f64::INFINITY, f64::min) * 1.3;

        let title: String = self.title();

        // Build the chart with specified margins and label sizes
        let mut chart = ChartBuilder::on(&root)
            .caption(title, ("sans-serif", 30))
            .margin(10)
            .top_x_label_area_size(40)
            .x_label_area_size(40)
            .y_label_area_size(60)
            .right_y_label_area_size(60)
            .build_cartesian_2d(min_price..max_price, min_pnl_value..max_pnl_value)?;

        // Configure and draw the mesh grid
        chart.configure_mesh().x_labels(20).y_labels(20).draw()?;

        // Draw a horizontal line at y = 0 to indicate break-even
        chart.draw_series(LineSeries::new(
            vec![(min_price, 0.0), (max_price, 0.0)],
            &BLACK,
        ))?;

        // Draw the line series representing PNL values
        chart.draw_series(LineSeries::new(
            data.iter().cloned().zip(pnl_values.iter().cloned()),
            &BLUE,
        ))?;

        // Draw points on the graph with labels for the PNL values
        for (i, (&price, &value)) in data.iter().zip(pnl_values.iter()).enumerate() {
            let point_color = if value >= 0.0 { &GREEN } else { &RED };
            let label_offset = if value >= 0.0 { (20, 0) } else { (-20, -20) };
            let size = 4;

            chart.draw_series(PointSeries::of_element(
                vec![(price, value)],
                size,
                point_color,
                &|coord, size, style| {
                    let element =
                        EmptyElement::at(coord) + Circle::new((0, 0), size, style.filled());

                    if i % 10 == 0 {
                        element
                            + Text::new(
                                format!("{:.2}", value),
                                (label_offset.0, label_offset.1),
                                ("sans-serif", 15).into_font(),
                            )
                    } else {
                        EmptyElement::at(coord)
                            + Circle::new((0, 0), size - 2, style.filled())
                            + Text::new(
                                String::new(),
                                (label_offset.0, label_offset.1),
                                ("sans-serif", 15).into_font(),
                            )
                    }
                },
            ))?;
        }

        // Finalize and render the chart
        root.present()?;
        Ok(())
    }

    fn title(&self) -> String {
        self.option.title()
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
