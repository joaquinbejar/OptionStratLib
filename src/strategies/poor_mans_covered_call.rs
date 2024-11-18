/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 25/9/24
******************************************************************************/
/*
    The "Poor Man's Covered Call" is an options strategy designed to simulate a traditional covered call,
    but with a lower capital requirement. In a standard covered call, an investor holds a long position
    in the underlying asset (e.g., a stock) and sells a call option against it to generate premium income.
    This strategy works well for neutral to slightly bullish market outlooks.
    However, instead of purchasing the underlying asset (which can be capital-intensive), the "Poor Man's
    Covered Call" involves buying a deep-in-the-money LEAP (Long-term Equity Anticipation Security) call
    option with a long expiration date and selling a short-term out-of-the-money call option against it.
    By using a LEAP, the investor still benefits from the movement of the underlying asset while avoiding
    the need to purchase it outright. The premium collected from selling the short-term call generates income
    and helps offset the cost of the LEAP.
    The strategy has two main components:
    1. **Long LEAP Call**: This serves as a substitute for holding the underlying asset. The deep-in-the-money
       LEAP behaves similarly to the underlying asset's price movement but costs a fraction of its price.
       The LEAP should have a delta close to 1, meaning it moves nearly dollar-for-dollar with the underlying asset.
    2. **Short Call**: A short-term out-of-the-money call is sold against the long LEAP. This generates premium
       income, and if the underlying asset's price rises above the strike price of the short call, the investor may
       need to sell the asset (or close the position), locking in potential gains.
    The goal is to capture some upside potential of the underlying asset while reducing risk through a lower capital
    commitment. The key risks involve the loss of the premium collected if the underlying asset does not move favorably
    and potential limitations on profits if the underlying asset's price rises sharply, triggering the short call.
    This strategy is often used by investors who are moderately bullish on an asset but wish to reduce the cost
    and risk associated with traditional covered call strategies.
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

const PMCC_DESCRIPTION: &str =
    "A Poor Man's Covered Call (PMCC) is an options strategy that simulates a covered call \
    using long-term equity anticipation securities (LEAPS) instead of the underlying stock. \
    It involves buying a long-term in-the-money call option and selling a short-term out-of-the-money call option. \
    This strategy aims to generate income while reducing the capital required compared to a traditional covered call.";

pub struct PoorMansCoveredCall {
    pub name: String,
    pub kind: StrategyType,
    pub description: String,
    pub break_even_points: Vec<PositiveF64>,
    long_call: Position,
    short_call: Position,
}

impl PoorMansCoveredCall {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        underlying_symbol: String,
        underlying_price: PositiveF64,
        long_call_strike: PositiveF64,
        short_call_strike: PositiveF64,
        long_call_expiration: ExpirationDate,
        short_call_expiration: ExpirationDate,
        implied_volatility: f64,
        risk_free_rate: f64,
        dividend_yield: f64,
        quantity: PositiveF64,
        premium_long_call: f64,
        premium_short_call: f64,
        open_fee_long_call: f64,
        close_fee_long_call: f64,
        open_fee_short_call: f64,
        close_fee_short_call: f64,
    ) -> Self {
        let mut strategy = PoorMansCoveredCall {
            name: "Poor Man's Covered Call".to_string(),
            kind: StrategyType::PoorMansCoveredCall,
            description: PMCC_DESCRIPTION.to_string(),
            break_even_points: Vec::new(),
            long_call: Position::default(),
            short_call: Position::default(),
        };

        // Long Call (LEAPS)
        let long_call_option = Options::new(
            OptionType::European,
            Side::Long,
            underlying_symbol.clone(),
            long_call_strike,
            long_call_expiration,
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
            open_fee_long_call,
            close_fee_long_call,
        );
        strategy.add_leg(long_call.clone());

        // Short Call
        let short_call_option = Options::new(
            OptionType::European,
            Side::Short,
            underlying_symbol,
            short_call_strike,
            short_call_expiration,
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
            open_fee_short_call,
            close_fee_short_call,
        );
        strategy.add_leg(short_call.clone());

        // Calculate break-even point
        let net_debit =
            (strategy.long_call.max_loss() - strategy.short_call.max_profit()) / quantity;
        strategy
            .break_even_points
            .push(long_call_strike + net_debit);
        strategy
    }
}

impl Validable for PoorMansCoveredCall {
    fn validate(&self) -> bool {
        self.short_call.validate() && self.long_call.validate()
    }
}

impl Strategies for PoorMansCoveredCall {
    fn add_leg(&mut self, position: Position) {
        match (
            position.option.option_style.clone(),
            position.option.side.clone(),
        ) {
            (OptionStyle::Call, Side::Long) => self.long_call = position,
            (OptionStyle::Call, Side::Short) => self.short_call = position,
            _ => panic!("Invalid option type for Poor Man's Covered Call strategy"),
        }
    }

    fn break_even(&self) -> Vec<PositiveF64> {
        self.break_even_points.clone()
    }

    fn max_profit(&self) -> f64 {
        let max_profit_price = self.short_call.option.strike_price;
        self.calculate_profit_at(max_profit_price)
    }

    fn max_loss(&self) -> f64 {
        self.long_call.max_loss() - self.short_call.max_profit()
    }

    fn total_cost(&self) -> f64 {
        self.long_call.net_cost() + self.short_call.net_cost()
    }

    fn net_premium_received(&self) -> f64 {
        panic!("Net premium received is not applicable");
    }

    fn fees(&self) -> f64 {
        (self.long_call.open_fee + self.long_call.close_fee) * self.long_call.option.quantity
            + (self.short_call.open_fee + self.short_call.close_fee)
                * self.short_call.option.quantity
    }

    fn profit_area(&self) -> f64 {
        let base = (self.short_call.option.strike_price
            - (self.short_call.option.strike_price - self.max_profit()))
        .value();
        let high = self.max_profit();
        base * high / 200.0
    }

    fn profit_ratio(&self) -> f64 {
        (self.max_profit() / self.max_loss()).abs() * 100.0
    }

    fn best_range_to_show(&self, step: PositiveF64) -> Option<Vec<PositiveF64>> {
        let (first_option, last_option) = (
            self.long_call.option.clone(),
            self.short_call.option.clone(),
        );
        let start_price = first_option.strike_price * STRIKE_PRICE_LOWER_BOUND_MULTIPLIER;
        let end_price = last_option.strike_price * STRIKE_PRICE_UPPER_BOUND_MULTIPLIER;
        Some(calculate_price_range(start_price, end_price, step))
    }
}

impl Profit for PoorMansCoveredCall {
    fn calculate_profit_at(&self, price: PositiveF64) -> f64 {
        let price = Some(price);
        self.long_call.pnl_at_expiration(&price) + self.short_call.pnl_at_expiration(&price)
    }
}

impl Graph for PoorMansCoveredCall {
    fn title(&self) -> String {
        let strategy_title = format!(
            "{:?} Strategy on {} Size {}:",
            self.kind, self.long_call.option.underlying_symbol, self.long_call.option.quantity
        );
        let leg_titles: Vec<String> = [
            format!("Long Call (LEAPS): ${}", self.long_call.option.strike_price),
            format!("Short Call: ${}", self.short_call.option.strike_price),
            format!(
                "Long Call Expiry: {}",
                self.long_call.option.expiration_date
            ),
            format!(
                "Short Call Expiry: {}",
                self.short_call.option.expiration_date
            ),
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
            label: format!("Break Even\n\n{}", self.break_even_points[0]),
            label_offset: (-30.0, 15.0),
            point_color: DARK_BLUE,
            label_color: DARK_BLUE,
            point_size: 5,
            font_size: 18,
        });

        let coordiantes: (f64, f64) = (
            self.short_call.option.strike_price.value() / 2000.0,
            self.max_profit() / 10.0,
        );
        points.push(ChartPoint {
            coordinates: (
                self.short_call.option.strike_price.value(),
                self.max_profit(),
            ),
            label: format!(
                "Max Profit {:.2} at {:.0}",
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
            self.long_call.option.strike_price.value() / 2000.0,
            -self.max_loss() / 50.0,
        );
        points.push(ChartPoint {
            coordinates: (self.long_call.option.strike_price.value(), -self.max_loss()),
            label: format!(
                "Max Loss {:.2} at {:.0}",
                self.max_loss(),
                self.long_call.option.strike_price
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
mod tests {
    use super::*;
    use crate::model::types::ExpirationDate;
    use crate::pos;

    fn create_pmcc_strategy() -> PoorMansCoveredCall {
        let underlying_symbol = "AAPL".to_string();
        let underlying_price = pos!(150.0);
        let long_call_strike = pos!(140.0);
        let short_call_strike = pos!(160.0);
        let long_call_expiration = ExpirationDate::Days(365.0);
        let short_call_expiration = ExpirationDate::Days(30.0);
        let implied_volatility = 0.20;
        let risk_free_rate = 0.01;
        let dividend_yield = 0.005;
        let quantity = pos!(1.0);
        let premium_long_call = 15.0;
        let premium_short_call = 5.0;
        let open_fee_long_call = 1.0;
        let close_fee_long_call = 1.0;
        let open_fee_short_call = 0.5;
        let close_fee_short_call = 0.5;

        PoorMansCoveredCall::new(
            underlying_symbol,
            underlying_price,
            long_call_strike,
            short_call_strike,
            long_call_expiration,
            short_call_expiration,
            implied_volatility,
            risk_free_rate,
            dividend_yield,
            quantity,
            premium_long_call,
            premium_short_call,
            open_fee_long_call,
            close_fee_long_call,
            open_fee_short_call,
            close_fee_short_call,
        )
    }

    #[test]
    fn test_create_pmcc_strategy() {
        let pmcc = create_pmcc_strategy();
        assert_eq!(pmcc.name, "Poor Man's Covered Call");
        assert_eq!(pmcc.long_call.option.strike_price, pos!(140.0));
        assert_eq!(pmcc.short_call.option.strike_price, pos!(160.0));
    }

    #[test]
    fn test_max_profit() {
        let pmcc = create_pmcc_strategy();
        let max_profit = pmcc.max_profit();
        assert!(max_profit > 0.0);
    }

    #[test]
    fn test_max_loss() {
        let pmcc = create_pmcc_strategy();
        let max_loss = pmcc.max_loss();
        assert!(max_loss > 0.0);
    }

    #[test]
    fn test_break_even() {
        let pmcc = create_pmcc_strategy();
        let break_even = pmcc.break_even();
        assert_eq!(break_even.len(), 1);
        assert!(break_even[0].value() > 0.0);
    }

    #[test]
    fn test_total_cost() {
        let pmcc = create_pmcc_strategy();
        let total_cost = pmcc.total_cost();
        assert!(total_cost > 0.0);
    }

    #[test]
    fn test_fees() {
        let pmcc = create_pmcc_strategy();
        let fees = pmcc.fees();
        assert!(fees > 0.0);
    }

    #[test]
    fn test_profit_area() {
        let pmcc = create_pmcc_strategy();
        let profit_area = pmcc.profit_area();
        assert!(profit_area > 0.0);
    }

    #[test]
    fn test_profit_ratio() {
        let pmcc = create_pmcc_strategy();
        let profit_ratio = pmcc.profit_ratio();
        assert!(profit_ratio > 0.0);
    }

    #[test]
    fn test_best_range_to_show() {
        let pmcc = create_pmcc_strategy();
        let step = pos!(1.0);
        let range = pmcc.best_range_to_show(step);
        assert!(range.is_some());
        let range_values = range.unwrap();
        assert!(!range_values.is_empty());
    }

    #[test]
    fn test_calculate_profit_at() {
        let pmcc = create_pmcc_strategy();
        let profit = pmcc.calculate_profit_at(pos!(150.0));
        assert!(profit >= -pmcc.max_loss() && profit <= pmcc.max_profit());
    }

    #[test]
    fn test_graph_title() {
        let pmcc = create_pmcc_strategy();
        let title = pmcc.title();
        assert!(title.contains("PoorMansCoveredCall Strategy"));
    }

    #[test]
    fn test_vertical_lines() {
        let pmcc = create_pmcc_strategy();
        let vertical_lines = pmcc.get_vertical_lines();
        assert_eq!(vertical_lines.len(), 1);
        assert_eq!(vertical_lines[0].x_coordinate, 150.0);
    }

    #[test]
    fn test_graph_points() {
        let pmcc = create_pmcc_strategy();
        let points = pmcc.get_points();
        assert!(!points.is_empty());
    }
}
