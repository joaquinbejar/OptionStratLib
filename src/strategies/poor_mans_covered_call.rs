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

use super::base::{Strategies, StrategyType};
use crate::model::option::Options;
use crate::model::position::Position;
use crate::model::types::{ExpirationDate, OptionStyle, OptionType, PositiveF64, Side, PZERO};
use crate::visualization::model::ChartVerticalLine;
use crate::visualization::utils::Graph;
use chrono::Utc;
use plotters::prelude::{ShapeStyle, BLACK};
use crate::spos;

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

    fn calculate_profit_at(&self, price: PositiveF64) -> f64 {
        self.long_call.pnl_at_expiration(Some(price))
            + self.short_call.pnl_at_expiration(Some(price))
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
        f64::INFINITY
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

    fn get_values<T>(&self, data: &[T]) -> Vec<f64> {
        data.iter()
            .map(|&price| self.calculate_profit_at(price))
            .collect()
    }

    fn get_vertical_lines(&self) -> Vec<ChartVerticalLine<PositiveF64, f64>> {
        let vertical_lines = vec![ChartVerticalLine {
            x_coordinate: PZERO, // TODO: underlying price
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
