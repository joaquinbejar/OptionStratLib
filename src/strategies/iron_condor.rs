/*
Iron Condor Strategy

An iron condor involves selling an out-of-the-money put spread and an out-of-the-money call spread with the same expiration date.
This strategy is used when little volatility in the underlying asset's price is expected.

Key characteristics:
- Limited profit potential
- Limited risk
- Profit is highest when the underlying asset price remains between the two sold options at expiration
*/
use super::base::{Strategies, StrategyType};
use crate::model::option::Options;
use crate::model::position::Position;
use crate::model::types::{ExpirationDate, OptionStyle, OptionType, Side};
use crate::visualization::utils::Graph;
use chrono::Utc;

const IRON_CONDOR_DESCRIPTION: &str =
    "An Iron Condor is a neutral options strategy combining a bull put spread with a bear call spread. \
    It involves selling an out-of-the-money put and call while buying further out-of-the-money put and call options. \
    This strategy is used when low volatility is expected and the underlying asset's price is anticipated to remain \
    within a specific range.";

pub struct IronCondor {
    pub name: String,
    pub kind: StrategyType,
    pub description: String,
    pub break_even_points: Vec<f64>,
    short_call: Position,
    short_put: Position,
    long_call: Position,
    long_put: Position,
}

impl IronCondor {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        underlying_symbol: String,
        underlying_price: f64,
        short_call_strike: f64,
        short_put_strike: f64,
        long_call_strike: f64,
        long_put_strike: f64,
        expiration: ExpirationDate,
        implied_volatility: f64,
        risk_free_rate: f64,
        dividend_yield: f64,
        quantity: u32,
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

    fn break_even(&self) -> f64 {
        // Iron Condor has two break-even points, we'll return the lower one
        self.break_even_points[0]
    }

    fn calculate_profit_at(&self, price: f64) -> f64 {
        self.short_call.pnl_at_expiration(Some(price))
            + self.short_put.pnl_at_expiration(Some(price))
            + self.long_call.pnl_at_expiration(Some(price))
            + self.long_put.pnl_at_expiration(Some(price))
    }

    fn max_profit(&self) -> f64 {
        self.net_premium_received()
    }

    fn max_loss(&self) -> f64 {
        let call_wing_width = (self.long_call.option.strike_price
            - self.short_call.option.strike_price)
            * (self.long_call.option.quantity as f64)
            - self.net_premium_received();
        let put_wing_width = (self.short_put.option.strike_price
            - self.long_put.option.strike_price)
            * (self.short_put.option.quantity as f64)
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
            // - self.short_call.total_cost() - self.short_put.total_cost()
            - self.long_put.total_cost() - self.long_call.total_cost()
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

    fn area(&self) -> f64 {
        let inner_width = self.short_call.option.strike_price - self.short_put.option.strike_price;
        let outer_width = self.long_call.option.strike_price - self.long_put.option.strike_price;
        let height = self.max_profit();

        let inner_area = inner_width * height;
        let outer_triangles = (outer_width - inner_width) * height / 2.0;

        (inner_area + outer_triangles) / self.short_call.option.underlying_price
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

    fn get_values(&self, data: &[f64]) -> Vec<f64> {
        data.iter()
            .map(|&price| self.calculate_profit_at(price))
            .collect()
    }

    fn get_vertical_lines(&self) -> Vec<(String, f64)> {
        self.break_even_points
            .iter()
            .enumerate()
            .map(|(i, &point)| (format!("Break Even {}", i + 1), point))
            .collect()
    }
}

#[cfg(test)]
mod tests_iron_condor {
    use super::*;
    use chrono::{TimeZone, Utc};

    // fn create_mock_position(strike: f64, style: OptionStyle, side: Side) -> Position {
    //     let date = Utc.with_ymd_and_hms(2024, 12, 1, 0, 0, 0).unwrap();
    //     let option = Options {
    //         option_type: OptionType::European,
    //         side,
    //         underlying_symbol: "AAPL".to_string(),
    //         strike_price: strike,
    //         expiration_date: ExpirationDate::DateTime(date),
    //         implied_volatility: 0.2,
    //         quantity: 1,
    //         underlying_price: 150.0,
    //         risk_free_rate: 0.01,
    //         option_style: style,
    //         dividend_yield: 0.02,
    //         exotic_params: None,
    //     };
    //
    //     Position {
    //         option,
    //         premium: strike * 0.01,
    //         date: Utc::now(),
    //         open_fee: 2.0,
    //         close_fee: 2.0,
    //     }
    // }

    #[test]
    fn test_iron_condor_creation() {
        let date = Utc.with_ymd_and_hms(2024, 12, 1, 0, 0, 0).unwrap();
        let iron_condor = IronCondor::new(
            "AAPL".to_string(),
            150.0,
            155.0,
            145.0,
            160.0,
            140.0,
            ExpirationDate::DateTime(date),
            0.2,
            0.01,
            0.02,
            1,
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
        assert!(iron_condor.short_call.option.strike_price == 155.0);
        assert!(iron_condor.short_put.option.strike_price == 145.0);
        assert!(iron_condor.long_call.option.strike_price == 160.0);
        assert!(iron_condor.long_put.option.strike_price == 140.0);
    }

    #[test]
    fn test_max_loss() {
        let date = Utc.with_ymd_and_hms(2024, 12, 1, 0, 0, 0).unwrap();
        let iron_condor = IronCondor::new(
            "AAPL".to_string(),
            150.0,
            120.0,
            110.0,
            130.0,
            100.0,
            ExpirationDate::DateTime(date),
            0.2,
            0.01,
            0.02,
            1,
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
            150.0,
            155.0,
            145.0,
            160.0,
            140.0,
            ExpirationDate::DateTime(date),
            0.2,
            0.01,
            0.02,
            1,
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
            150.0,
            155.0,
            145.0,
            160.0,
            140.0,
            ExpirationDate::DateTime(date),
            0.2,
            0.01,
            0.02,
            1,
            1.5,
            1.0,
            2.0,
            1.8,
            5.0,
            5.0,
        );

        assert_eq!(iron_condor.break_even(), iron_condor.break_even_points[0]);
    }

    #[test]
    fn test_fees() {
        let date = Utc.with_ymd_and_hms(2024, 12, 1, 0, 0, 0).unwrap();
        let iron_condor = IronCondor::new(
            "AAPL".to_string(),
            150.0,
            155.0,
            145.0,
            160.0,
            140.0,
            ExpirationDate::DateTime(date),
            0.2,
            0.01,
            0.02,
            1,
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
            150.0,
            155.0,
            145.0,
            160.0,
            140.0,
            ExpirationDate::DateTime(date),
            0.2,
            0.01,
            0.02,
            1,
            1.5,
            1.0,
            2.0,
            1.8,
            5.0,
            5.0,
        );

        let price = 150.0;
        let expected_profit = iron_condor.short_call.pnl_at_expiration(Some(price))
            + iron_condor.short_put.pnl_at_expiration(Some(price))
            + iron_condor.long_call.pnl_at_expiration(Some(price))
            + iron_condor.long_put.pnl_at_expiration(Some(price));
        assert_eq!(iron_condor.calculate_profit_at(price), expected_profit);
    }
}
