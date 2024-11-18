/*
Iron Condor Strategy

An iron condor involves selling an out-of-the-money put spread and an out-of-the-money call spread with the same expiration date.
This strategy is used when little volatility in the underlying asset's price is expected.

Key characteristics:
- Limited profit potential
- Limited risk
- Profit is highest when the underlying asset price remains between the two sold options at expiration
*/
use super::base::{Strategies, StrategyType, Validable};
use crate::constants::{
    DARK_BLUE, DARK_GREEN, STRIKE_PRICE_LOWER_BOUND_MULTIPLIER, STRIKE_PRICE_UPPER_BOUND_MULTIPLIER,
};
use crate::model::option::Options;
use crate::model::position::Position;
use crate::model::types::{ExpirationDate, OptionStyle, OptionType, PositiveF64, Side};
use crate::pricing::payoff::Profit;
use crate::strategies::utils::calculate_price_range;
use crate::visualization::model::{ChartPoint, ChartVerticalLine};
use crate::visualization::utils::Graph;
use chrono::Utc;
use plotters::prelude::full_palette::ORANGE;
use plotters::prelude::{ShapeStyle, RED};

const IRON_CONDOR_DESCRIPTION: &str =
    "An Iron Condor is a neutral options strategy combining a bull put spread with a bear call spread. \
    It involves selling an out-of-the-money put and call while buying further out-of-the-money put and call options. \
    This strategy is used when low volatility is expected and the underlying asset's price is anticipated to remain \
    within a specific range.";

pub struct IronCondor {
    pub name: String,
    pub kind: StrategyType,
    pub description: String,
    pub break_even_points: Vec<PositiveF64>,
    short_call: Position,
    short_put: Position,
    long_call: Position,
    long_put: Position,
}

impl IronCondor {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        underlying_symbol: String,
        underlying_price: PositiveF64,
        short_call_strike: PositiveF64,
        short_put_strike: PositiveF64,
        long_call_strike: PositiveF64,
        long_put_strike: PositiveF64,
        expiration: ExpirationDate,
        implied_volatility: f64,
        risk_free_rate: f64,
        dividend_yield: f64,
        quantity: PositiveF64,
        premium_short_call: f64,
        premium_short_put: f64,
        premium_long_call: f64,
        premium_long_put: f64,
        open_fee: f64,
        close_fee: f64,
    ) -> Self {
        let mut strategy = IronCondor {
            name: "Iron Condor".to_string(),
            kind: StrategyType::IronCondor,
            description: IRON_CONDOR_DESCRIPTION.to_string(),
            break_even_points: Vec::new(),
            short_call: Position::default(),
            short_put: Position::default(),
            long_call: Position::default(),
            long_put: Position::default(),
        };

        // Short Call
        let short_call_option = Options::new(
            OptionType::European,
            Side::Short,
            underlying_symbol.clone(),
            short_call_strike,
            expiration.clone(),
            implied_volatility,
            quantity,
            underlying_price,
            risk_free_rate,
            OptionStyle::Call,
            dividend_yield,
            None,
        );
        let short_call = Position::new(
            short_call_option,
            premium_short_call,
            Utc::now(),
            open_fee,
            close_fee,
        );
        strategy.add_leg(short_call.clone());

        // Short Put
        let short_put_option = Options::new(
            OptionType::European,
            Side::Short,
            underlying_symbol.clone(),
            short_put_strike,
            expiration.clone(),
            implied_volatility,
            quantity,
            underlying_price,
            risk_free_rate,
            OptionStyle::Put,
            dividend_yield,
            None,
        );
        let short_put = Position::new(
            short_put_option,
            premium_short_put,
            Utc::now(),
            open_fee,
            close_fee,
        );
        strategy.add_leg(short_put.clone());

        // Long Call
        let long_call_option = Options::new(
            OptionType::European,
            Side::Long,
            underlying_symbol.clone(),
            long_call_strike,
            expiration.clone(),
            implied_volatility,
            quantity,
            underlying_price,
            risk_free_rate,
            OptionStyle::Call,
            dividend_yield,
            None,
        );
        let long_call = Position::new(
            long_call_option,
            premium_long_call,
            Utc::now(),
            open_fee,
            close_fee,
        );
        strategy.add_leg(long_call.clone());

        // Long Put
        let long_put_option = Options::new(
            OptionType::European,
            Side::Long,
            underlying_symbol,
            long_put_strike,
            expiration,
            implied_volatility,
            quantity,
            underlying_price,
            risk_free_rate,
            OptionStyle::Put,
            dividend_yield,
            None,
        );
        let long_put = Position::new(
            long_put_option,
            premium_long_put,
            Utc::now(),
            open_fee,
            close_fee,
        );
        strategy.add_leg(long_put.clone());

        // Calculate break-even points
        let net_credit = (strategy.long_put.premium + strategy.long_call.premium) + strategy.fees()
            - (strategy.short_put.premium + strategy.short_call.premium);
        strategy
            .break_even_points
            .push(short_put_strike + net_credit);
        strategy
            .break_even_points
            .push(short_call_strike - net_credit);

        strategy
    }
}

impl Validable for IronCondor {
    fn validate(&self) -> bool {
        self.short_call.validate()
            && self.short_put.validate()
            && self.long_call.validate()
            && self.long_put.validate()
    }
}

impl Strategies for IronCondor {
    fn add_leg(&mut self, position: Position) {
        match (
            position.option.option_style.clone(),
            position.option.side.clone(),
        ) {
            (OptionStyle::Call, Side::Short) => self.short_call = position,
            (OptionStyle::Put, Side::Short) => self.short_put = position,
            (OptionStyle::Call, Side::Long) => self.long_call = position,
            (OptionStyle::Put, Side::Long) => self.long_put = position,
        }
    }

    fn get_legs(&self) -> Vec<Position> {
        vec![
            self.short_call.clone(),
            self.short_put.clone(),
            self.long_call.clone(),
            self.long_put.clone(),
        ]
    }

    fn break_even(&self) -> Vec<PositiveF64> {
        // Iron Condor has two break-even points, we'll return the lower one
        self.break_even_points.clone()
    }

    fn max_profit(&self) -> f64 {
        self.net_premium_received()
    }

    fn max_loss(&self) -> f64 {
        let call_wing_width =
            (self.long_call.option.strike_price - self.short_call.option.strike_price).value()
                * self.long_call.option.quantity.value()
                - self.net_premium_received();
        let put_wing_width =
            (self.short_put.option.strike_price - self.long_put.option.strike_price).value()
                * self.short_put.option.quantity.value()
                - self.net_premium_received();

        call_wing_width.max(put_wing_width)
    }

    fn total_cost(&self) -> f64 {
        self.short_call.net_cost()
            + self.short_put.net_cost()
            + self.long_call.net_cost()
            + self.long_put.net_cost()
    }

    fn net_premium_received(&self) -> f64 {
        self.short_call.net_premium_received() + self.short_put.net_premium_received()
            - self.long_put.total_cost()
            - self.long_call.total_cost()
    }

    fn fees(&self) -> f64 {
        self.short_call.open_fee
            + self.short_call.close_fee
            + self.short_put.open_fee
            + self.short_put.close_fee
            + self.long_call.open_fee
            + self.long_call.close_fee
            + self.long_put.open_fee
            + self.long_put.close_fee
    }

    fn profit_area(&self) -> f64 {
        let inner_width =
            (self.short_call.option.strike_price - self.short_put.option.strike_price).value();
        let outer_width =
            (self.long_call.option.strike_price - self.long_put.option.strike_price).value();
        let height = self.max_profit();

        let inner_area = inner_width * height;
        let outer_triangles = (outer_width - inner_width) * height / 2.0;

        (inner_area + outer_triangles) / self.short_call.option.underlying_price.value()
    }

    fn best_range_to_show(&self, step: PositiveF64) -> Option<Vec<PositiveF64>> {
        let (first_option, last_option) =
            (self.long_put.option.clone(), self.long_call.option.clone());
        let start_price = first_option.strike_price * STRIKE_PRICE_LOWER_BOUND_MULTIPLIER;
        let end_price = last_option.strike_price * STRIKE_PRICE_UPPER_BOUND_MULTIPLIER;
        Some(calculate_price_range(start_price, end_price, step))
    }
}

impl Profit for IronCondor {
    fn calculate_profit_at(&self, price: PositiveF64) -> f64 {
        let price = Some(price);
        self.short_call.pnl_at_expiration(&price)
            + self.short_put.pnl_at_expiration(&price)
            + self.long_call.pnl_at_expiration(&price)
            + self.long_put.pnl_at_expiration(&price)
    }
}

impl Graph for IronCondor {
    fn title(&self) -> String {
        let strategy_title = format!(
            "{:?} Strategy on {} Size {}:",
            self.kind, self.short_put.option.underlying_symbol, self.short_put.option.quantity
        );
        let leg_titles: Vec<String> = [
            format!("Long Put: ${}", self.long_put.option.strike_price),
            format!("Short Put: ${}", self.short_put.option.strike_price),
            format!("Short Call: ${}", self.short_call.option.strike_price),
            format!("Long Call: ${}", self.long_call.option.strike_price),
            format!("Expire: {}", self.short_put.option.expiration_date),
        ]
            .iter()
            .map(|leg| leg.to_string())
            .collect();

        if leg_titles.is_empty() {
            strategy_title
        } else {
            format!("{}\n\t{}", strategy_title, leg_titles.join("\n\t"))
        }
    }

    fn get_vertical_lines(&self) -> Vec<ChartVerticalLine<f64, f64>> {
        let vertical_lines = vec![ChartVerticalLine {
            x_coordinate: self.short_call.option.underlying_price.value(),
            y_range: (-50000.0, 50000.0),
            label: format!("Current Price: {}", self.short_call.option.underlying_price),
            label_offset: (5.0, 5.0),
            line_color: ORANGE,
            label_color: ORANGE,
            line_style: ShapeStyle::from(&ORANGE).stroke_width(2),
            font_size: 18,
        }];

        vertical_lines
    }

    fn get_points(&self) -> Vec<ChartPoint<(f64, f64)>> {
        let mut points: Vec<ChartPoint<(f64, f64)>> = Vec::new();

        points.push(ChartPoint {
            coordinates: (self.break_even_points[0].value(), 0.0),
            label: format!("Left Break Even\n\n{}", self.break_even_points[0]),
            label_offset: (5.0, 5.0),
            point_color: DARK_BLUE,
            label_color: DARK_BLUE,
            point_size: 5,
            font_size: 18,
        });

        points.push(ChartPoint {
            coordinates: (self.break_even_points[1].value(), 0.0),
            label: format!("Right Break Even\n\n{}", self.break_even_points[1]),
            label_offset: (5.0, 5.0),
            point_color: DARK_BLUE,
            label_color: DARK_BLUE,
            point_size: 5,
            font_size: 18,
        });

        let coordiantes: (f64, f64) = (
            self.short_call.option.strike_price.value() / 2000.0,
            self.max_profit() / 5.0,
        );
        points.push(ChartPoint {
            coordinates: (
                self.short_call.option.strike_price.value(),
                self.max_profit(),
            ),
            label: format!(
                "High Max Profit {:.2} at {:.0}",
                self.max_profit(),
                self.short_call.option.strike_price
            ),
            label_offset: coordiantes,
            point_color: DARK_GREEN,
            label_color: DARK_GREEN,
            point_size: 5,
            font_size: 18,
        });

        let coordiantes: (f64, f64) = (
            self.short_put.option.strike_price.value() / 2000.0,
            self.max_profit() / 5.0,
        );
        points.push(ChartPoint {
            coordinates: (
                self.short_put.option.strike_price.value(),
                self.max_profit(),
            ),
            label: format!(
                "Low Max Profit {:.2} at {:.0}",
                self.max_profit(),
                self.short_put.option.strike_price
            ),
            label_offset: coordiantes,
            point_color: DARK_GREEN,
            label_color: DARK_GREEN,
            point_size: 5,
            font_size: 18,
        });

        let loss = self.calculate_profit_at(self.long_call.option.strike_price);
        let coordiantes: (f64, f64) = (
            self.long_call.option.strike_price.value() / 2000.0,
            loss / 50.0,
        );
        points.push(ChartPoint {
            coordinates: (self.long_call.option.strike_price.value(), loss),
            label: format!(
                "Right Max Loss {:.2} at {:.0}",
                loss, self.long_call.option.strike_price
            ),
            label_offset: coordiantes,
            point_color: RED,
            label_color: RED,
            point_size: 5,
            font_size: 18,
        });

        let loss = self.calculate_profit_at(self.long_put.option.strike_price);
        let coordiantes: (f64, f64) = (
            self.long_put.option.strike_price.value() / 2000.0,
            loss / 50.0,
        );
        points.push(ChartPoint {
            coordinates: (self.long_put.option.strike_price.value(), loss),
            label: format!(
                "Left Max Loss {:.2} at {:.0}",
                loss, self.long_put.option.strike_price
            ),
            label_offset: coordiantes,
            point_color: RED,
            label_color: RED,
            point_size: 5,
            font_size: 18,
        });

        points.push(self.get_point_at_price(self.long_call.option.underlying_price));

        points
    }
}

#[cfg(test)]
mod tests_iron_condor {
    use super::*;
    use crate::model::types::SIZE_ONE;
    use crate::pos;
    use chrono::{TimeZone, Utc};

    #[test]
    fn test_iron_condor_creation() {
        let date = Utc.with_ymd_and_hms(2024, 12, 1, 0, 0, 0).unwrap();
        let iron_condor = IronCondor::new(
            "AAPL".to_string(),
            pos!(150.0),
            pos!(155.0),
            pos!(145.0),
            pos!(160.0),
            pos!(140.0),
            ExpirationDate::DateTime(date),
            0.2,
            0.01,
            0.02,
            SIZE_ONE,
            1.5,
            1.0,
            2.0,
            1.8,
            5.0,
            5.0,
        );

        assert_eq!(iron_condor.name, "Iron Condor");
        assert_eq!(iron_condor.description, IRON_CONDOR_DESCRIPTION.to_string());
        assert_eq!(iron_condor.kind, StrategyType::IronCondor);
        assert_eq!(iron_condor.break_even_points.len(), 2);
        assert_eq!(iron_condor.short_call.option.strike_price, 155.0);
        assert_eq!(iron_condor.short_put.option.strike_price, 145.0);
        assert_eq!(iron_condor.long_call.option.strike_price, 160.0);
        assert_eq!(iron_condor.long_put.option.strike_price, 140.0);
    }

    #[test]
    fn test_max_loss() {
        let date = Utc.with_ymd_and_hms(2024, 12, 1, 0, 0, 0).unwrap();
        let iron_condor = IronCondor::new(
            "AAPL".to_string(),
            pos!(150.0),
            pos!(120.0),
            pos!(110.0),
            pos!(130.0),
            pos!(100.0),
            ExpirationDate::DateTime(date),
            0.2,
            0.01,
            0.02,
            SIZE_ONE,
            1.5,
            1.0,
            2.0,
            1.8,
            5.0,
            5.0,
        );

        assert_eq!(iron_condor.max_loss(), 51.3);
    }

    #[test]
    fn test_max_profit() {
        let date = Utc.with_ymd_and_hms(2024, 12, 1, 0, 0, 0).unwrap();
        let iron_condor = IronCondor::new(
            "AAPL".to_string(),
            pos!(150.0),
            pos!(155.0),
            pos!(145.0),
            pos!(160.0),
            pos!(140.0),
            ExpirationDate::DateTime(date),
            0.2,
            0.01,
            0.02,
            SIZE_ONE,
            1.5,
            1.0,
            2.0,
            1.8,
            5.0,
            5.0,
        );

        let expected_profit = iron_condor.net_premium_received();
        assert_eq!(iron_condor.max_profit(), expected_profit);
    }

    #[test]
    fn test_break_even() {
        let date = Utc.with_ymd_and_hms(2024, 12, 1, 0, 0, 0).unwrap();
        let iron_condor = IronCondor::new(
            "AAPL".to_string(),
            pos!(150.0),
            pos!(155.0),
            pos!(145.0),
            pos!(160.0),
            pos!(140.0),
            ExpirationDate::DateTime(date),
            0.2,
            0.01,
            0.02,
            SIZE_ONE,
            1.5,
            1.0,
            2.0,
            1.8,
            5.0,
            5.0,
        );

        assert_eq!(
            iron_condor.break_even()[0],
            iron_condor.break_even_points[0]
        );
    }

    #[test]
    fn test_fees() {
        let date = Utc.with_ymd_and_hms(2024, 12, 1, 0, 0, 0).unwrap();
        let iron_condor = IronCondor::new(
            "AAPL".to_string(),
            pos!(150.0),
            pos!(155.0),
            pos!(145.0),
            pos!(160.0),
            pos!(140.0),
            ExpirationDate::DateTime(date),
            0.2,
            0.01,
            0.02,
            SIZE_ONE,
            1.5,
            1.0,
            2.0,
            1.8,
            5.0,
            5.0,
        );

        let expected_fees = iron_condor.short_call.open_fee
            + iron_condor.short_call.close_fee
            + iron_condor.short_put.open_fee
            + iron_condor.short_put.close_fee
            + iron_condor.long_call.open_fee
            + iron_condor.long_call.close_fee
            + iron_condor.long_put.open_fee
            + iron_condor.long_put.close_fee;
        assert_eq!(iron_condor.fees(), expected_fees);
    }

    #[test]
    fn test_calculate_profit_at() {
        let date = Utc.with_ymd_and_hms(2024, 12, 1, 0, 0, 0).unwrap();
        let iron_condor = IronCondor::new(
            "AAPL".to_string(),
            pos!(150.0),
            pos!(155.0),
            pos!(145.0),
            pos!(160.0),
            pos!(140.0),
            ExpirationDate::DateTime(date),
            0.2,
            0.01,
            0.02,
            SIZE_ONE,
            1.5,
            1.0,
            2.0,
            1.8,
            5.0,
            5.0,
        );

        let price = pos!(150.0);
        let expected_profit = iron_condor.short_call.pnl_at_expiration(&Some(price))
            + iron_condor.short_put.pnl_at_expiration(&Some(price))
            + iron_condor.long_call.pnl_at_expiration(&Some(price))
            + iron_condor.long_put.pnl_at_expiration(&Some(price));
        assert_eq!(iron_condor.calculate_profit_at(price), expected_profit);
    }
}
