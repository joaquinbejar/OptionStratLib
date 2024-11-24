/*
Strangle Strategy

A strangle involves simultaneously buying an out-of-the-money call and an out-of-the-money put option with the same expiration date.
This strategy is similar to a straddle but typically has a lower cost and requires a larger price move to become profitable.

Key characteristics:
- Unlimited profit potential
- Lower cost than a straddle
- Requires a larger price move to become profitable
*/
use super::base::{Optimizable, Strategies, StrategyType, Validable};
use crate::chains::chain::{OptionChain, OptionData};
use crate::constants::{DARK_BLUE, DARK_GREEN};
use crate::model::option::Options;
use crate::model::position::Position;
use crate::model::types::{ExpirationDate, OptionStyle, OptionType, PositiveF64, Side, PZERO};
use crate::pos;
use crate::pricing::payoff::Profit;
use crate::strategies::utils::{calculate_price_range, FindOptimalSide, OptimizationCriteria};
use crate::visualization::model::{ChartPoint, ChartVerticalLine};
use crate::visualization::utils::Graph;
use chrono::Utc;
use plotters::prelude::full_palette::ORANGE;
use plotters::prelude::{ShapeStyle, RED};
use std::f64;
use tracing::{debug, error, info, trace};

const SHORT_STRANGLE_DESCRIPTION: &str =
    "A short strangle involves selling an out-of-the-money call and an \
out-of-the-money put with the same expiration date. This strategy is used when low volatility \
is expected and the underlying asset's price is anticipated to remain stable.";

#[derive(Clone, Debug)]
pub struct ShortStrangle {
    pub name: String,
    pub kind: StrategyType,
    pub description: String,
    pub break_even_points: Vec<PositiveF64>,
    short_call: Position,
    short_put: Position,
}

impl ShortStrangle {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        underlying_symbol: String,
        underlying_price: PositiveF64,
        mut call_strike: PositiveF64,
        mut put_strike: PositiveF64,
        expiration: ExpirationDate,
        implied_volatility: f64,
        risk_free_rate: f64,
        dividend_yield: f64,
        quantity: PositiveF64,
        premium_short_call: f64,
        premium_short_put: f64,
        open_fee_short_call: f64,
        close_fee_short_call: f64,
        open_fee_short_put: f64,
        close_fee_short_put: f64,
    ) -> Self {
        if call_strike == PZERO {
            call_strike = underlying_price * 1.1;
        }
        if put_strike == PZERO {
            put_strike = underlying_price * 0.9;
        }
        let mut strategy = ShortStrangle {
            name: "Short Strangle".to_string(),
            kind: StrategyType::Strangle,
            description: SHORT_STRANGLE_DESCRIPTION.to_string(),
            break_even_points: Vec::new(),
            short_call: Position::default(),
            short_put: Position::default(),
        };

        let short_call_option = Options::new(
            OptionType::European,
            Side::Short,
            underlying_symbol.clone(),
            call_strike,
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
            open_fee_short_call,
            close_fee_short_call,
        );
        strategy.add_leg(short_call.clone());

        let short_put_option = Options::new(
            OptionType::European,
            Side::Short,
            underlying_symbol,
            put_strike,
            expiration,
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
            open_fee_short_put,
            close_fee_short_put,
        );
        strategy.add_leg(short_put.clone());

        let net_quantity = (short_call.option.quantity + short_put.option.quantity) / 2.0;
        strategy
            .break_even_points
            .push(put_strike - strategy.net_premium_received() / net_quantity);
        strategy
            .break_even_points
            .push(call_strike + strategy.net_premium_received() / net_quantity);

        strategy
    }
}

impl Strategies for ShortStrangle {
    fn add_leg(&mut self, position: Position) {
        match position.option.option_style {
            OptionStyle::Call => self.short_call = position,
            OptionStyle::Put => self.short_put = position,
        }
    }

    fn break_even(&self) -> Vec<PositiveF64> {
        // Short strangle has two break-even points, we'll return the lower one
        vec![self.short_put.option.strike_price + self.net_premium_received()]
    }

    fn max_profit(&self) -> f64 {
        self.net_premium_received()
    }

    fn max_loss(&self) -> f64 {
        f64::INFINITY // Theoretically unlimited
    }

    fn total_cost(&self) -> f64 {
        self.short_call.net_cost() + self.short_put.net_cost()
    }

    fn net_premium_received(&self) -> f64 {
        self.short_call.net_premium_received() + self.short_put.net_premium_received()
    }

    fn fees(&self) -> f64 {
        self.short_call.open_fee
            + self.short_call.close_fee
            + self.short_put.open_fee
            + self.short_put.close_fee
    }

    fn profit_area(&self) -> f64 {
        let strike_diff = self.short_call.option.strike_price - self.short_put.option.strike_price;
        let inner_square = strike_diff * self.max_profit();
        let break_even_diff = self.break_even_points[1] - self.break_even_points[0];
        let outer_square = break_even_diff * self.max_profit();
        let triangles = (outer_square - inner_square) / 2.0;
        ((inner_square + triangles) / self.short_call.option.underlying_price).value()
    }

    fn profit_ratio(&self) -> f64 {
        let break_even_diff = self.break_even_points[1] - self.break_even_points[0];
        // info!("Max Profit: {} Break Even Diff: {}, Ratio: {}", self.max_profit(), break_even_diff, self.max_profit() / break_even_diff * 100.0);
        self.max_profit() / break_even_diff * 100.0
    }

    fn best_ratio(&mut self, option_chain: &OptionChain, side: FindOptimalSide) {
        self.find_optimal(option_chain, side, OptimizationCriteria::Ratio);
    }

    fn best_area(&mut self, option_chain: &OptionChain, side: FindOptimalSide) {
        self.find_optimal(option_chain, side, OptimizationCriteria::Area);
    }

    fn best_range_to_show(&self, step: PositiveF64) -> Option<Vec<PositiveF64>> {
        let (first_option, last_option) = (self.break_even_points[0], self.break_even_points[1]);
        let start_price = first_option - self.max_profit();
        let end_price = last_option + self.max_profit();
        Some(calculate_price_range(start_price, end_price, step))
    }
}

impl Validable for ShortStrangle {
    fn validate(&self) -> bool {
        self.short_call.validate()
            && self.short_put.validate()
            && self.short_call.option.strike_price > self.short_put.option.strike_price
    }
}

impl Optimizable for ShortStrangle {
    type Strategy = ShortStrangle;

    fn find_optimal(
        &mut self,
        option_chain: &OptionChain,
        side: FindOptimalSide,
        criteria: OptimizationCriteria,
    ) {
        let options: Vec<&OptionData> = option_chain.options.iter().collect();
        let mut best_value = f64::NEG_INFINITY;

        for call_index in 0..options.len() {
            let call_option = &options[call_index];

            for put_option in &options[..call_index] {
                if call_option.strike_price <= put_option.strike_price {
                    error!(
                        "Invalid strike prices CALL: {:#?} PUT: {:#?}",
                        call_option.strike_price, put_option.strike_price
                    );
                    continue;
                }

                if !self.is_valid_short_option(put_option, &side)
                    || !self.is_valid_short_option(call_option, &side)
                {
                    continue;
                }

                if !self.are_valid_prices(call_option, put_option) {
                    error!("Invalid Bid prices  Put({}): {:?} Call({}): {:?} ", 
                        put_option.strike_price,
                        put_option.put_bid.unwrap_or(PZERO), 
                        call_option.strike_price,
                        call_option.call_bid.unwrap_or(PZERO));
                    continue;
                }

                debug!("Creating Strategy");
                let strategy: ShortStrangle =
                    self.create_strategy(option_chain, call_option, put_option);

                if !strategy.validate() {
                    continue;
                }

                let current_value = match criteria {
                    OptimizationCriteria::Ratio => strategy.profit_ratio(),
                    OptimizationCriteria::Area => strategy.profit_area(),
                };

                if current_value > best_value {
                    best_value = current_value;
                    *self = strategy.clone();
                }
            }
        }
        debug!("Best Value: {}", best_value);
    }

    fn is_valid_short_option(&self, option: &OptionData, side: &FindOptimalSide) -> bool {
        let underlying_price = match (
            self.short_put.option.underlying_price,
            self.short_call.option.underlying_price,
        ) {
            (PZERO, PZERO) => PZERO,
            (PZERO, call) => call,
            (put, _) => put,
        };
        if underlying_price == PZERO {
            error!("Invalid underlying_price option");
            return false;
        }

        match side {
            FindOptimalSide::Upper => {
                let valid = option.strike_price >= underlying_price;
                if !valid {
                    debug!(
                        "Option is out of range: {} <= {}",
                        option.strike_price, underlying_price
                    );
                }
                valid
            }
            FindOptimalSide::Lower => {
                let valid = option.strike_price <= underlying_price;
                if !valid {
                    debug!(
                        "Option is out of range: {} >= {}",
                        option.strike_price, underlying_price
                    );
                }
                valid
            }
            FindOptimalSide::All => true,
            FindOptimalSide::Range(start, end) => {
                let valid = option.strike_price >= *start && option.strike_price <= *end;
                if !valid {
                    debug!(
                        "Option is out of range: {} >= {} && {} <= {}",
                        option.strike_price, *start, option.strike_price, *end
                    );
                }
                valid
            }
        }
    }

    fn are_valid_prices(&self, call: &OptionData, put: &OptionData) -> bool {
        call.call_bid.unwrap() > PZERO && put.put_bid.unwrap() > PZERO
    }

    fn create_strategy(
        &self,
        chain: &OptionChain,
        call: &OptionData,
        put: &OptionData,
    ) -> ShortStrangle {
        if !call.validate() || !put.validate() {
            panic!("Invalid options");
        }
        ShortStrangle::new(
            chain.symbol.clone(),
            chain.underlying_price,
            call.strike_price,
            put.strike_price,
            self.short_call.option.expiration_date.clone(),
            call.implied_volatility.unwrap().value() / 100.0,
            self.short_call.option.risk_free_rate,
            self.short_call.option.dividend_yield,
            self.short_call.option.quantity,
            call.call_bid.unwrap().value(),
            put.put_bid.unwrap().value(),
            self.short_call.open_fee,
            self.short_call.close_fee,
            self.short_put.open_fee,
            self.short_put.close_fee,
        )
    }
}

impl Profit for ShortStrangle {
    fn calculate_profit_at(&self, price: PositiveF64) -> f64 {
        let price = Some(price);
        trace!(
            "Price: {:.2} Strike: {} Call: {:.2} Strike: {} Put: {:.2} Profit: {:.2}",
            price.unwrap(),
            self.short_call.option.strike_price,
            self.short_call.pnl_at_expiration(&price),
            self.short_put.option.strike_price,
            self.short_put.pnl_at_expiration(&price),
            self.short_call.pnl_at_expiration(&price) + self.short_put.pnl_at_expiration(&price)
        );
        self.short_call.pnl_at_expiration(&price) + self.short_put.pnl_at_expiration(&price)
    }
}

impl Graph for ShortStrangle {
    fn title(&self) -> String {
        let strategy_title = format!("Short {:?} Strategy: ", self.kind);
        let leg_titles: Vec<String> = [self.short_call.title(), self.short_put.title()]
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
        let max_value = self.max_profit() * 1.2;
        let min_value = self.max_profit() * -1.2;

        let vertical_lines = vec![ChartVerticalLine {
            x_coordinate: self.short_call.option.underlying_price.value(),
            y_range: (min_value, max_value),
            label: format!(
                "Current Price: {:.2}",
                self.short_call.option.underlying_price
            ),
            label_offset: (4.0, -1.0),
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
            label: format!("Low Break Even\n\n{}", self.break_even_points[0]),
            label_offset: (0.0, -10.0),
            point_color: DARK_BLUE,
            label_color: DARK_BLUE,
            point_size: 5,
            font_size: 18,
        });

        points.push(ChartPoint {
            coordinates: (self.break_even_points[1].value(), 0.0),
            label: format!("High Break Even\n\n{}", self.break_even_points[1]),
            label_offset: (-230.0, -10.0),
            point_color: DARK_BLUE,
            label_color: DARK_BLUE,
            point_size: 5,
            font_size: 18,
        });

        let coordiantes: (f64, f64) = (
            self.short_put.option.strike_price.value() / 250.0,
            self.max_profit() / 15.0,
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
            -self.short_put.option.strike_price.value() / 30.0,
            self.max_profit() / 15.0,
        );
        points.push(ChartPoint {
            coordinates: (
                self.short_put.option.strike_price.value(),
                self.max_profit(),
            ),
            label: format!(
                "Max Profit {:.2} at {:.0}",
                self.max_profit(),
                self.short_put.option.strike_price
            ),
            label_offset: coordiantes,
            point_color: DARK_GREEN,
            label_color: DARK_GREEN,
            point_size: 5,
            font_size: 18,
        });
        points.push(self.get_point_at_price(self.short_put.option.underlying_price));

        points
    }
}

const LONG_STRANGLE_DESCRIPTION: &str =
    "A long strangle involves buying an out-of-the-money call and an \
out-of-the-money put with the same expiration date. This strategy is used when high volatility \
is expected and a significant move in the underlying asset's price is anticipated, but the \
direction is uncertain.";

#[derive(Clone, Debug)]
pub struct LongStrangle {
    pub name: String,
    pub kind: StrategyType,
    pub description: String,
    pub break_even_points: Vec<PositiveF64>,
    long_call: Position,
    long_put: Position,
}

impl LongStrangle {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        underlying_symbol: String,
        underlying_price: PositiveF64,
        mut call_strike: PositiveF64,
        mut put_strike: PositiveF64,
        expiration: ExpirationDate,
        implied_volatility: f64,
        risk_free_rate: f64,
        dividend_yield: f64,
        quantity: PositiveF64,
        premium_long_call: f64,
        premium_long_put: f64,
        open_fee_long_call: f64,
        close_fee_long_call: f64,
        open_fee_long_put: f64,
        close_fee_long_put: f64,
    ) -> Self {
        if call_strike == PZERO {
            call_strike = underlying_price * 1.1;
        }
        if put_strike == PZERO {
            put_strike = underlying_price * 0.9;
        }
        let mut strategy = LongStrangle {
            name: "Long Strangle".to_string(),
            kind: StrategyType::Strangle,
            description: LONG_STRANGLE_DESCRIPTION.to_string(),
            break_even_points: Vec::new(),
            long_call: Position::default(),
            long_put: Position::default(),
        };

        let long_call_option = Options::new(
            OptionType::European,
            Side::Long,
            underlying_symbol.clone(),
            call_strike,
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
            open_fee_long_call,
            close_fee_long_call,
        );
        strategy.add_leg(long_call.clone());

        let long_put_option = Options::new(
            OptionType::European,
            Side::Long,
            underlying_symbol,
            put_strike,
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
            open_fee_long_put,
            close_fee_long_put,
        );
        strategy.add_leg(long_put.clone());

        let net_quantity = (long_call.option.quantity + long_put.option.quantity) / pos!(2.0);

        strategy
            .break_even_points
            .push(put_strike - strategy.total_cost() / net_quantity);

        strategy
            .break_even_points
            .push(call_strike + strategy.total_cost() / net_quantity);

        strategy
    }
}

impl Strategies for LongStrangle {
    fn add_leg(&mut self, position: Position) {
        match position.option.option_style {
            OptionStyle::Call => self.long_call = position,
            OptionStyle::Put => self.long_put = position,
        }
    }

    fn break_even(&self) -> Vec<PositiveF64> {
        // Long strangle has two break-even points, we'll return the lower one
        vec![self.long_put.option.strike_price - self.total_cost()]
    }

    fn max_profit(&self) -> f64 {
        f64::INFINITY // Theoretically unlimited
    }

    fn max_loss(&self) -> f64 {
        self.total_cost()
    }

    fn total_cost(&self) -> f64 {
        self.long_call.net_cost() + self.long_put.net_cost()
    }

    fn net_premium_received(&self) -> f64 {
        0.0 // Long strangle doesn't receive premium
    }

    fn fees(&self) -> f64 {
        self.long_call.open_fee
            + self.long_call.close_fee
            + self.long_put.open_fee
            + self.long_put.close_fee
    }

    fn profit_area(&self) -> f64 {
        let strike_diff = self.long_call.option.strike_price - self.long_put.option.strike_price;
        let inner_square = strike_diff * self.max_loss();
        let break_even_diff = self.break_even_points[1] - self.break_even_points[0];
        let outer_square = break_even_diff * self.max_loss();
        let triangles = (outer_square - inner_square) / 2.0;
        let loss_area =
            ((inner_square + triangles) / self.long_call.option.underlying_price).value();
        1.0 / loss_area // Invert the value to get the profit area: the lower, the better
    }

    fn profit_ratio(&self) -> f64 {
        let break_even_diff = self.break_even_points[1] - self.break_even_points[0];
        let ratio = self.max_loss() / break_even_diff * 100.0;
        1.0 / ratio // Invert the value to get the profit ratio: the lower, the better
    }

    fn best_ratio(&mut self, option_chain: &OptionChain, side: FindOptimalSide) {
        self.find_optimal(option_chain, side, OptimizationCriteria::Ratio);
    }

    fn best_area(&mut self, option_chain: &OptionChain, side: FindOptimalSide) {
        self.find_optimal(option_chain, side, OptimizationCriteria::Area);
    }

    fn best_range_to_show(&self, step: PositiveF64) -> Option<Vec<PositiveF64>> {
        let (first_option, last_option) = (self.break_even_points[0], self.break_even_points[1]);
        info!("First: {} Last: {}", first_option, last_option);
        let diff = last_option - first_option;
        debug!(
            "First break even point: {} Last break even point: {}",
            first_option, last_option
        );
        let start_price = first_option - diff;
        debug!("Start price: {}", start_price);
        let end_price = last_option + diff;
        debug!("End price: {}", end_price);
        Some(calculate_price_range(start_price, end_price, step))
    }
}

impl Validable for LongStrangle {
    fn validate(&self) -> bool {
        self.long_call.validate()
            && self.long_put.validate()
            && self.long_call.option.strike_price > self.long_put.option.strike_price
            && self.long_put.option.strike_price > PZERO
    }
}

impl Optimizable for LongStrangle {
    type Strategy = LongStrangle;
    fn find_optimal(
        &mut self,
        option_chain: &OptionChain,
        side: FindOptimalSide,
        criteria: OptimizationCriteria,
    ) {
        let options: Vec<&OptionData> = option_chain.options.iter().collect();
        let mut best_value = f64::NEG_INFINITY;

        for call_index in 0..options.len() {
            let call_option = &options[call_index];
            
            for put_option in &options[..call_index] {
                trace!("Call: {:#?} Put: {:#?}", call_option.strike_price, put_option.strike_price);
                if call_option.strike_price <= put_option.strike_price {
                    error!(
                        "Invalid strike prices Put: {:#?} Call: {:#?} ", 
                        put_option.strike_price, 
                        call_option.strike_price);
                    continue;
                }

                if !self.is_valid_long_option(put_option, &side)
                    || !self.is_valid_long_option(call_option, &side)
                {
                    error!("Invalid option");
                    continue;
                }

                if !self.are_valid_prices(call_option, put_option) {
                    error!("Invalid Ask prices Put: {:#?} Call: {:#?} ", 
                        put_option.put_ask, 
                        call_option.call_ask);
                    continue;
                }

                let strategy: LongStrangle =
                    self.create_strategy(option_chain, call_option, put_option);


                if !strategy.validate() {
                    error!("Invalid strategy");
                    continue;
                }

                let current_value = match criteria {
                    OptimizationCriteria::Ratio => strategy.profit_ratio(),
                    OptimizationCriteria::Area => strategy.profit_area(),
                };

                if current_value > best_value {
                    best_value = current_value;
                    *self = strategy.clone();
                }
            }
        }
    }

    fn is_valid_long_option(&self, option: &OptionData, side: &FindOptimalSide) -> bool {
        let underlying_price = match (
            self.long_put.option.underlying_price,
            self.long_call.option.underlying_price,
        ) {
            (PZERO, PZERO) => PZERO,
            (PZERO, call) => call,
            (put, _) => put,
        };
        if underlying_price == PZERO {
            error!("Invalid underlying_price option");
            return false;
        }

        match side {
            FindOptimalSide::Upper => {
                let valid = option.strike_price >= underlying_price;
                if !valid {
                    debug!(
                        "Option is out of range: {} <= {}",
                        option.strike_price, underlying_price
                    );
                }
                valid
            }
            FindOptimalSide::Lower => {
                let valid = option.strike_price <= underlying_price;
                if !valid {
                    debug!(
                        "Option is out of range: {} >= {}",
                        option.strike_price, underlying_price
                    );
                }
                valid
            }
            FindOptimalSide::All => true,
            FindOptimalSide::Range(start, end) => {
                let valid = option.strike_price >= *start && option.strike_price <= *end;
                if !valid {
                    debug!(
                        "Option is out of range: {} >= {} && {} <= {}",
                        option.strike_price, *start, option.strike_price, *end
                    );
                }
                valid
            }
        }
    }

    fn are_valid_prices(&self, call: &OptionData, put: &OptionData) -> bool {
        call.call_ask.unwrap() > PZERO && put.put_ask.unwrap() > PZERO
    }

    fn create_strategy(
        &self,
        chain: &OptionChain,
        call: &OptionData,
        put: &OptionData,
    ) -> LongStrangle {
        LongStrangle::new(
            chain.symbol.clone(),
            chain.underlying_price,
            call.strike_price,
            put.strike_price,
            self.long_call.option.expiration_date.clone(),
            call.implied_volatility.unwrap().value() / 100.0,
            self.long_call.option.risk_free_rate,
            self.long_call.option.dividend_yield,
            self.long_call.option.quantity,
            call.call_ask.unwrap().value(),
            put.put_ask.unwrap().value(),
            self.long_call.open_fee,
            self.long_call.close_fee,
            self.long_put.open_fee,
            self.long_put.close_fee,
        )
    }
}

impl Profit for LongStrangle {
    fn calculate_profit_at(&self, price: PositiveF64) -> f64 {
        let price = Some(price);
        self.long_call.pnl_at_expiration(&price) + self.long_put.pnl_at_expiration(&price)
    }
}

impl Graph for LongStrangle {
    fn title(&self) -> String {
        let strategy_title = format!("Long {:?} Strategy: ", self.kind);
        let leg_titles: Vec<String> = [self.long_call.title(), self.long_put.title()]
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
        let max_value = f64::INFINITY;
        let min_value = f64::NEG_INFINITY;

        let vertical_lines = vec![ChartVerticalLine {
            x_coordinate: self.long_call.option.underlying_price.value(),
            y_range: (min_value, max_value),
            label: format!(
                "Current Price: {:.2}",
                self.long_call.option.underlying_price
            ),
            label_offset: (4.0, -50.0),
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
            label: format!("Low Break Even {}", self.break_even_points[0]),
            label_offset: (10.0, -10.0),
            point_color: DARK_BLUE,
            label_color: DARK_BLUE,
            point_size: 5,
            font_size: 18,
        });

        points.push(ChartPoint {
            coordinates: (self.break_even_points[1].value(), 0.0),
            label: format!("High Break Even {}", self.break_even_points[1]),
            label_offset: (-60.0, -10.0),
            point_color: DARK_BLUE,
            label_color: DARK_BLUE,
            point_size: 5,
            font_size: 18,
        });

        points.push(ChartPoint {
            coordinates: (self.long_call.option.strike_price.value(), -self.max_loss()),
            label: format!(
                "Max Loss {:.2} at {:.0}",
                self.max_loss(),
                self.long_call.option.strike_price
            ),
            label_offset: (0.0, -20.0),
            point_color: RED,
            label_color: RED,
            point_size: 5,
            font_size: 18,
        });

        points.push(ChartPoint {
            coordinates: (self.long_put.option.strike_price.value(), -self.max_loss()),
            label: format!(
                "Max Loss {:.2} at {:.0}",
                self.max_loss(),
                self.long_put.option.strike_price
            ),
            label_offset: (-500.0, -20.0),
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
mod tests_short_strangle {
    use super::*;
    use crate::chains::utils::{OptionChainBuildParams, OptionDataPriceParams};
    use crate::{pos, spos};

    fn setup() -> ShortStrangle {
        ShortStrangle::new(
            "AAPL".to_string(),
            pos!(150.0),
            pos!(155.0),
            pos!(145.0),
            ExpirationDate::Days(30.0),
            0.2,
            0.01,
            0.02,
            pos!(100.0),
            2.0,
            1.5,
            0.1,
            0.1,
            0.1,
            0.1,
        )
    }

    fn wrong_setup() -> ShortStrangle {
        ShortStrangle::new(
            "AAPL".to_string(),
            pos!(150.0),
            pos!(145.0),
            pos!(155.0),
            ExpirationDate::Days(30.0),
            0.2,
            0.01,
            0.02,
            pos!(100.0),
            2.0,
            1.5,
            0.1,
            0.1,
            0.1,
            0.1,
        )
    }

    #[test]
    fn test_new() {
        let strategy = setup();
        assert_eq!(strategy.name, "Short Strangle");
        assert_eq!(strategy.kind, StrategyType::Strangle);
        assert_eq!(
            strategy.description,
            "A short strangle involves selling an out-of-the-money call and an \
out-of-the-money put with the same expiration date. This strategy is used when low volatility \
is expected and the underlying asset's price is anticipated to remain stable."
        );
    }

    #[test]
    fn test_validate() {
        let strategy = setup();
        let wrong_strategy = wrong_setup();
        assert!(strategy.validate());
        assert!(!wrong_strategy.validate());
    }

    #[test]
    fn test_break_even() {
        let strategy = setup();
        assert_eq!(strategy.break_even()[0], 455.0);
    }

    #[test]
    fn test_calculate_profit_at() {
        let strategy = setup();
        let price = 150.0;
        assert_eq!(strategy.calculate_profit_at(pos!(price)), 310.0);
    }

    #[test]
    fn test_max_profit() {
        let strategy = setup();
        assert_eq!(strategy.max_profit(), strategy.net_premium_received());
    }

    #[test]
    fn test_max_loss() {
        let strategy = setup();
        assert_eq!(strategy.max_loss(), f64::INFINITY);
    }

    #[test]
    fn test_total_cost() {
        let strategy = setup();
        assert_eq!(
            strategy.total_cost(),
            strategy.short_call.net_cost() + strategy.short_put.net_cost()
        );
    }

    #[test]
    fn test_net_premium_received() {
        let strategy = setup();
        assert_eq!(
            strategy.net_premium_received(),
            strategy.short_call.net_premium_received() + strategy.short_put.net_premium_received()
        );
    }

    #[test]
    fn test_fees() {
        let strategy = setup();
        let expected_fees = 0.4;
        assert_eq!(strategy.fees(), expected_fees);
    }

    #[test]
    fn test_area() {
        let strategy = setup();
        assert_eq!(strategy.profit_area(), 27.07333333333332);
    }

    #[test]
    fn test_graph_methods() {
        let strategy = setup();

        let vertical_lines = strategy.get_vertical_lines();
        assert_eq!(vertical_lines.len(), 1);
        assert_eq!(vertical_lines[0].label, "Current Price: 150.00");

        let data = vec![
            pos!(140.0),
            pos!(145.0),
            pos!(150.0),
            pos!(155.0),
            pos!(160.0),
        ];
        let values = strategy.get_values(&data);
        for (i, &price) in data.iter().enumerate() {
            assert_eq!(values[i], strategy.calculate_profit_at(price));
        }

        let title = strategy.title();
        assert!(title.contains("Short Strangle Strategy"));
        assert!(title.contains("Call"));
        assert!(title.contains("Put"));
    }

    #[test]
    fn test_add_leg() {
        let mut strategy = setup();
        let original_call = strategy.short_call.clone();
        let original_put = strategy.short_put.clone();

        // Test adding a new call leg
        strategy.add_leg(original_call.clone());
        assert_eq!(strategy.short_call, original_call);

        // Test adding a new put leg
        strategy.add_leg(original_put.clone());
        assert_eq!(strategy.short_put, original_put);
    }

    #[test]
    fn test_profit_ratio() {
        let strategy = setup();
        let break_even_diff = strategy.break_even_points[1] - strategy.break_even_points[0];
        let expected_ratio = strategy.max_profit() / break_even_diff * 100.0;
        assert_eq!(strategy.profit_ratio(), expected_ratio);
    }

    #[test]
    fn test_best_ratio() {
        let mut strategy = setup();
        let option_chain = create_test_option_chain();

        strategy.best_ratio(&option_chain, FindOptimalSide::All);
        assert!(strategy.validate());
    }

    #[test]
    fn test_best_area() {
        let mut strategy = setup();
        let option_chain = create_test_option_chain();

        strategy.best_area(&option_chain, FindOptimalSide::All);
        assert!(strategy.validate());
    }

    #[test]
    fn test_best_range_to_show() {
        let strategy = setup();
        let step = pos!(1.0);

        let range = strategy.best_range_to_show(step).unwrap();
        assert!(!range.is_empty());
        assert!(range[0] <= strategy.break_even_points[0]);
        assert!(*range.last().unwrap() >= strategy.break_even_points[1]);
    }

    #[test]
    fn test_is_valid_short_option() {
        let strategy = setup();
        let option_chain = create_test_option_chain();
        let option_data = option_chain.options.first().unwrap();
        let min_strike = option_chain.options.first().unwrap().strike_price;
        let max_strike = option_chain.options.last().unwrap().strike_price;

        // Test FindOptimalSide::Upper
        assert!(strategy.is_valid_short_option(&option_data, &FindOptimalSide::Upper));

        // Test FindOptimalSide::Lower
        assert!(!strategy.is_valid_short_option(&option_data, &FindOptimalSide::Lower));

        // Test FindOptimalSide::All
        assert!(strategy.is_valid_short_option(&option_data, &FindOptimalSide::All));

        // Test FindOptimalSide::Range
        assert!(strategy.is_valid_short_option(
            &option_data,
            &FindOptimalSide::Range(min_strike, max_strike)
        ));
    }

    #[test]
    fn test_are_valid_prices() {
        let strategy = setup();
        let option_chain = create_test_option_chain();
        let call_option = option_chain.options.last().unwrap();
        let put_option = option_chain.options.first().unwrap();

        assert!(strategy.are_valid_prices(&call_option, &put_option));

        let mut invalid_call = call_option.clone();
        invalid_call.call_ask = Some(pos!(0.0));
        assert!(!strategy.are_valid_prices(&invalid_call, &put_option));
    }

    #[test]
    fn test_create_strategy() {
        let strategy = setup();
        let chain = create_test_option_chain();
        let call_option = chain.options.first().unwrap();
        let put_option = chain.options.last().unwrap();

        let new_strategy = strategy.create_strategy(&chain, &call_option, &put_option);
        assert!(!new_strategy.validate());

        let call_option = chain.options.last().unwrap();
        let put_option = chain.options.first().unwrap();

        let new_strategy = strategy.create_strategy(&chain, &call_option, &put_option);
        assert!(new_strategy.validate());
    }

    #[test]
    fn test_get_points() {
        let strategy = setup();
        let points = strategy.get_points();

        // Debería tener 5 puntos: 2 break-even, 2 max profit, 1 current price
        assert_eq!(points.len(), 5);

        // Verificar que los puntos de break-even están presentes
        let break_even_points: Vec<f64> = points[0..2].iter().map(|p| p.coordinates.0).collect();
        assert!(break_even_points.contains(&strategy.break_even_points[0].value()));
        assert!(break_even_points.contains(&strategy.break_even_points[1].value()));
    }

    fn create_test_option_chain() -> OptionChain {
        let option_data_price_params = OptionDataPriceParams::new(
            pos!(1150.0),
            ExpirationDate::Days(30.0),
            spos!(0.2),
            0.01,
            0.02,
        );
        let option_chain_build_params = OptionChainBuildParams::new(
            "AAPL".to_string(),
            spos!(1.0),
            10,
            pos!(10.0),
            0.00001,
            pos!(0.01),
            2,
            option_data_price_params,
        );
        OptionChain::build_chain(&option_chain_build_params)
    }
}

#[cfg(test)]
mod tests_long_strangle {
    use super::*;
    use crate::chains::utils::{OptionChainBuildParams, OptionDataPriceParams};
    use crate::{pos, spos};

    #[test]
    fn test_long_strangle_new() {
        let underlying_symbol = "AAPL".to_string();
        let underlying_price = pos!(150.0);
        let call_strike = pos!(160.0);
        let put_strike = pos!(140.0);
        let expiration = ExpirationDate::default();
        let implied_volatility = 0.25;
        let risk_free_rate = 0.01;
        let dividend_yield = 0.02;
        let quantity = pos!(10.0);
        let premium_long_call = 5.0;
        let premium_long_put = 5.0;
        let open_fee_long_call = 0.5;
        let close_fee_long_call = 0.5;
        let open_fee_long_put = 0.5;
        let close_fee_long_put = 0.5;

        let strategy = LongStrangle::new(
            underlying_symbol.clone(),
            underlying_price,
            call_strike,
            put_strike,
            expiration.clone(),
            implied_volatility,
            risk_free_rate,
            dividend_yield,
            quantity,
            premium_long_call,
            premium_long_put,
            open_fee_long_call,
            close_fee_long_call,
            open_fee_long_put,
            close_fee_long_put,
        );

        assert_eq!(strategy.name, "Long Strangle");
        assert_eq!(strategy.kind, StrategyType::Strangle);
        assert_eq!(strategy.description, LONG_STRANGLE_DESCRIPTION);

        let break_even_points = vec![128.0, 172.0];
        assert_eq!(strategy.break_even_points, break_even_points);
    }

    #[test]
    fn test_break_even() {
        let long_strangle = setup_long_strangle();
        assert_eq!(
            long_strangle.break_even()[0],
            long_strangle.long_put.option.strike_price.value() - long_strangle.total_cost()
        );
    }

    #[test]
    fn test_total_cost() {
        let long_strangle = setup_long_strangle();
        assert_eq!(
            long_strangle.total_cost(),
            long_strangle.long_call.net_cost() + long_strangle.long_put.net_cost()
        );
    }

    #[test]
    fn test_calculate_profit_at() {
        let long_strangle = setup_long_strangle();
        let price = pos!(150.0);
        let expected_profit = long_strangle.long_call.pnl_at_expiration(&Some(price))
            + long_strangle.long_put.pnl_at_expiration(&Some(price));
        assert_eq!(long_strangle.calculate_profit_at(price), expected_profit);
    }

    fn setup_long_strangle() -> LongStrangle {
        LongStrangle::new(
            "AAPL".to_string(),
            pos!(150.0),
            pos!(160.0),
            pos!(140.0),
            ExpirationDate::Days(30.0),
            0.25,
            0.01,
            0.02,
            pos!(10.0),
            5.0,
            5.0,
            0.5,
            0.5,
            0.5,
            0.5,
        )
    }

    fn wrong_setup_long_strangle() -> LongStrangle {
        // Setup with put strike higher than call strike
        LongStrangle::new(
            "AAPL".to_string(),
            pos!(150.0),
            pos!(140.0), // Call strike lower than put
            pos!(160.0), // Put strike higher than call
            ExpirationDate::Days(30.0),
            0.25,
            0.01,
            0.02,
            pos!(10.0),
            5.0,
            5.0,
            0.5,
            0.5,
            0.5,
            0.5,
        )
    }

    #[test]
    fn test_new() {
        let strategy = setup_long_strangle();
        assert_eq!(strategy.name, "Long Strangle");
        assert_eq!(strategy.kind, StrategyType::Strangle);
        assert_eq!(strategy.description, LONG_STRANGLE_DESCRIPTION);
    }

    #[test]
    fn test_validate() {
        let strategy = setup_long_strangle();
        let wrong_strategy = wrong_setup_long_strangle();
        assert!(strategy.validate());
        assert!(!wrong_strategy.validate());
    }

    #[test]
    fn test_max_profit() {
        let strategy = setup_long_strangle();
        assert_eq!(strategy.max_profit(), f64::INFINITY);
    }

    #[test]
    fn test_max_loss() {
        let strategy = setup_long_strangle();
        assert_eq!(strategy.max_loss(), strategy.total_cost());
    }

    #[test]
    fn test_fees() {
        let strategy = setup_long_strangle();
        let expected_fees = 2.0; // 0.5 * 4 fees
        assert_eq!(strategy.fees(), expected_fees);
    }

    #[test]
    fn test_net_premium_received() {
        let strategy = setup_long_strangle();
        assert_eq!(strategy.net_premium_received(), 0.0);
    }

    #[test]
    fn test_profit_area() {
        let strategy = setup_long_strangle();
        let area = strategy.profit_area();
        assert!(area > 0.0);
    }

    #[test]
    fn test_profit_ratio() {
        let strategy = setup_long_strangle();
        let break_even_diff = strategy.break_even_points[1] - strategy.break_even_points[0];
        let expected_ratio = 1.0 / (strategy.max_loss() / break_even_diff * 100.0);
        assert_eq!(strategy.profit_ratio(), expected_ratio);
    }

    #[test]
    fn test_add_leg() {
        let mut strategy = setup_long_strangle();
        let original_call = strategy.long_call.clone();
        let original_put = strategy.long_put.clone();

        strategy.add_leg(original_call.clone());
        assert_eq!(strategy.long_call, original_call);

        strategy.add_leg(original_put.clone());
        assert_eq!(strategy.long_put, original_put);
    }

    #[test]
    fn test_graph_methods() {
        let strategy = setup_long_strangle();

        // Test vertical lines
        let vertical_lines = strategy.get_vertical_lines();
        assert_eq!(vertical_lines.len(), 1);
        assert_eq!(vertical_lines[0].label, "Current Price: 150.00");

        // Test values calculation
        let data = vec![
            pos!(130.0),
            pos!(140.0),
            pos!(150.0),
            pos!(160.0),
            pos!(170.0),
        ];
        let values = strategy.get_values(&data);
        for (i, &price) in data.iter().enumerate() {
            assert_eq!(values[i], strategy.calculate_profit_at(price));
        }

        // Test title
        let title = strategy.title();
        assert!(title.contains("Long Strangle Strategy"));
        assert!(title.contains("Call"));
        assert!(title.contains("Put"));
    }

    #[test]
    fn test_best_ratio() {
        let mut strategy = setup_long_strangle();
        let option_chain = create_test_option_chain();

        strategy.best_ratio(&option_chain, FindOptimalSide::All);
        assert!(strategy.validate());
    }

    #[test]
    fn test_best_area() {
        let mut strategy = setup_long_strangle();
        let option_chain = create_test_option_chain();

        strategy.best_area(&option_chain, FindOptimalSide::All);
        assert!(strategy.validate());
    }

    #[test]
    fn test_best_range_to_show() {
        let strategy = setup_long_strangle();
        let step = pos!(1.0);

        let range = strategy.best_range_to_show(step).unwrap();
        assert!(!range.is_empty());
        assert!(range[0] <= strategy.break_even_points[0]);
        assert!(*range.last().unwrap() >= strategy.break_even_points[1]);
    }

    #[test]
    fn test_is_valid_long_option() {
        let strategy = setup_long_strangle();
        let option_chain = create_test_option_chain();
        let option_data = option_chain.options.last().unwrap();
        let min_strike = option_chain.options.first().unwrap().strike_price;
        let max_strike = option_chain.options.last().unwrap().strike_price;

        assert!(strategy.is_valid_long_option(&option_data, &FindOptimalSide::Upper));
        assert!(!strategy.is_valid_long_option(&option_data, &FindOptimalSide::Lower));
        assert!(strategy.is_valid_long_option(&option_data, &FindOptimalSide::All));
        assert!(strategy.is_valid_long_option(
            &option_data,
            &FindOptimalSide::Range(min_strike, max_strike)
        ));
    }

    #[test]
    fn test_are_valid_prices() {
        let strategy = setup_long_strangle();
        let option_chain = create_test_option_chain();
        let call_option = option_chain.options.first().unwrap();
        let put_option = option_chain.options.last().unwrap();

        assert!(strategy.are_valid_prices(&call_option, &put_option));

        let mut invalid_call = call_option.clone();
        invalid_call.call_ask = Some(pos!(0.0));
        assert!(!strategy.are_valid_prices(&invalid_call, &put_option));
    }

    #[test]
    fn test_create_strategy() {
        let strategy = setup_long_strangle();
        let chain = create_test_option_chain();
        let call_option = chain.options.first().unwrap();
        let put_option = chain.options.last().unwrap();
        let new_strategy = strategy.create_strategy(&chain, &call_option, &put_option);
        assert!(!new_strategy.validate());
        let call_option = chain.options.last().unwrap();
        let put_option = chain.options.first().unwrap();
        let new_strategy = strategy.create_strategy(&chain, &call_option, &put_option);
        assert!(new_strategy.validate());
    }

    #[test]
    fn test_get_points() {
        let strategy = setup_long_strangle();
        let points = strategy.get_points();

        // Should have 5 points: 2 break-even, 2 max loss, 1 current price
        assert_eq!(points.len(), 5);

        let break_even_points: Vec<f64> = points[0..2].iter().map(|p| p.coordinates.0).collect();
        assert!(break_even_points.contains(&strategy.break_even_points[0].value()));
        assert!(break_even_points.contains(&strategy.break_even_points[1].value()));
    }

    fn create_test_option_chain() -> OptionChain {
        let option_data_price_params = OptionDataPriceParams::new(
            pos!(150.0),
            ExpirationDate::Days(30.0),
            spos!(0.65),
            0.01,
            0.02,
        );
        let option_chain_build_params = OptionChainBuildParams::new(
            "AAPL".to_string(),
            spos!(1.0),
            10,
            pos!(5.0),
            0.00001,
            pos!(0.01),
            2,
            option_data_price_params,
        );
        OptionChain::build_chain(&option_chain_build_params)
    }
}
