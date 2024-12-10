/*
Long Butterfly Spread Strategy

A long butterfly spread involves:
1. Buy one call at a lower strike price (ITM)
2. Sell two calls at a middle strike price (ATM)
3. Buy one call at a higher strike price (OTM)

All options have the same expiration date.

Key characteristics:
- Limited profit potential
- Limited risk
- Profit is highest when price is exactly at middle strike at expiration
- Maximum loss is limited to the net premium paid
- All options must have same expiration date
*/

use super::base::{Optimizable, Strategies, StrategyType, Validable};
use crate::chains::chain::{OptionChain, OptionData};
use crate::chains::StrategyLegs;
use crate::constants::{DARK_BLUE, DARK_GREEN, ZERO};
use crate::model::option::Options;
use crate::model::position::Position;
use crate::model::types::{ExpirationDate, OptionStyle, OptionType, PositiveF64, Side, PZERO};
use crate::model::utils::mean_and_std;
use crate::model::ProfitLossRange;
use crate::pos;
use crate::pricing::payoff::Profit;
use crate::strategies::probabilities::{ProbabilityAnalysis, VolatilityAdjustment};
use crate::strategies::utils::{FindOptimalSide, OptimizationCriteria};
use crate::visualization::model::{ChartPoint, ChartVerticalLine, LabelOffsetType};
use crate::visualization::utils::Graph;
use chrono::Utc;
use plotters::prelude::full_palette::ORANGE;
use plotters::prelude::{ShapeStyle, RED};
use tracing::debug;

const LONG_BUTTERFLY_DESCRIPTION: &str =
    "A long butterfly spread is created by buying one call at a lower strike price, \
    selling two calls at a middle strike price, and buying one call at a higher strike price, \
    all with the same expiration date. This strategy profits when the underlying price stays \
    near the middle strike price at expiration.";

#[derive(Clone, Debug)]
pub struct LongButterfly {
    pub name: String,
    pub kind: StrategyType,
    pub description: String,
    pub break_even_points: Vec<PositiveF64>,
    long_call_low: Position,
    short_calls: Position,
    long_call_high: Position,
}

impl LongButterfly {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        underlying_symbol: String,
        underlying_price: PositiveF64,
        low_strike: PositiveF64,
        middle_strike: PositiveF64,
        high_strike: PositiveF64,
        expiration: ExpirationDate,
        implied_volatility: f64,
        risk_free_rate: f64,
        dividend_yield: f64,
        quantity: PositiveF64,
        premium_low: f64,
        premium_middle: f64,
        premium_high: f64,
        fees: f64,
    ) -> Self {
        let mut strategy = LongButterfly {
            name: "Long Butterfly".to_string(),
            kind: StrategyType::LongButterfly,
            description: LONG_BUTTERFLY_DESCRIPTION.to_string(),
            break_even_points: Vec::new(),
            long_call_low: Position::default(),
            short_calls: Position::default(),
            long_call_high: Position::default(),
        };

        // Create long call at lower strike
        let long_call_low = Options::new(
            OptionType::European,
            Side::Long,
            underlying_symbol.clone(),
            low_strike,
            expiration.clone(),
            implied_volatility,
            quantity,
            underlying_price,
            risk_free_rate,
            OptionStyle::Call,
            dividend_yield,
            None,
        );
        strategy.long_call_low = Position::new(
            long_call_low,
            premium_low,
            Utc::now(),
            fees / 3.0,
            fees / 3.0,
        );

        // Create two short calls at middle strike
        let short_calls = Options::new(
            OptionType::European,
            Side::Short,
            underlying_symbol.clone(),
            middle_strike,
            expiration.clone(),
            implied_volatility,
            quantity * 2.0, // Double quantity for middle strike
            underlying_price,
            risk_free_rate,
            OptionStyle::Call,
            dividend_yield,
            None,
        );
        strategy.short_calls = Position::new(
            short_calls,
            premium_middle,
            Utc::now(),
            fees / 3.0,
            fees / 3.0,
        );

        // Create long call at higher strike
        let long_call_high = Options::new(
            OptionType::European,
            Side::Long,
            underlying_symbol,
            high_strike,
            expiration,
            implied_volatility,
            quantity,
            underlying_price,
            risk_free_rate,
            OptionStyle::Call,
            dividend_yield,
            None,
        );
        strategy.long_call_high = Position::new(
            long_call_high,
            premium_high,
            Utc::now(),
            fees / 3.0,
            fees / 3.0,
        );

        strategy.validate();

        // Calculate break-even points
        // let net_debit = strategy.net_premium_received().abs();
        // strategy.break_even_points.push(low_strike + net_debit);
        // strategy.break_even_points.push(high_strike - net_debit);


        let left_profit = strategy.calculate_profit_at(low_strike) / quantity.value();
        let right_profit = strategy.calculate_profit_at(high_strike) / quantity.value();
        strategy.break_even_points.push(low_strike - left_profit);
        strategy.break_even_points.push(high_strike + right_profit);
        
        
        strategy
    }
}

// Implementación para Long Butterfly
impl Validable for LongButterfly {
    fn validate(&self) -> bool {
        // Validar que todas las posiciones son válidas
        if !self.long_call_low.validate() {
            debug!("Long call (low strike) is invalid");
            return false;
        }
        if !self.short_calls.validate() {
            debug!("Short calls (middle strike) are invalid");
            return false;
        }
        if !self.long_call_high.validate() {
            debug!("Long call (high strike) is invalid");
            return false;
        }

        // Validar el orden correcto de los strikes
        if self.long_call_low.option.strike_price >= self.short_calls.option.strike_price {
            debug!("Low strike must be lower than middle strike");
            return false;
        }
        if self.short_calls.option.strike_price >= self.long_call_high.option.strike_price {
            debug!("Middle strike must be lower than high strike");
            return false;
        }

        // Validar que las cantidades son correctas
        if self.short_calls.option.quantity != self.long_call_low.option.quantity * 2.0 {
            debug!("Middle strike quantity must be double the wing quantities");
            return false;
        }
        if self.long_call_low.option.quantity != self.long_call_high.option.quantity {
            debug!("Wing quantities must be equal");
            return false;
        }

        // Validar que el ancho de las alas es igual
        let lower_wing =
            self.short_calls.option.strike_price - self.long_call_low.option.strike_price;
        let upper_wing =
            self.long_call_high.option.strike_price - self.short_calls.option.strike_price;
        if lower_wing != upper_wing {
            debug!("Wings must be symmetrical");
            return false;
        }

        // Validar que todas las opciones tienen la misma fecha de expiración
        if self.long_call_low.option.expiration_date != self.short_calls.option.expiration_date
            || self.short_calls.option.expiration_date != self.long_call_high.option.expiration_date
        {
            debug!("All options must have the same expiration date");
            return false;
        }

        true
    }
}

// Implementación para Long Butterfly
impl Optimizable for LongButterfly {
    type Strategy = LongButterfly;

    fn find_optimal(
        &mut self,
        option_chain: &OptionChain,
        side: FindOptimalSide,
        criteria: OptimizationCriteria,
    ) {
        let mut best_value = f64::NEG_INFINITY;
        let options: Vec<&OptionData> = option_chain.options.iter().collect();

        for (i, low_strike) in options.iter().enumerate() {
            if !self.is_valid_long_option(low_strike, &side) {
                continue;
            }

            for (_j, middle_strike) in options.iter().enumerate().skip(i + 1) {
                if !self.is_valid_short_option(middle_strike, &side) {
                    continue;
                }

                let middle_to_low =
                    middle_strike.strike_price.value() - low_strike.strike_price.value();
                let target_high_strike = middle_strike.strike_price.value() + middle_to_low;

                if let Some(high_strike) = options.iter().find(|&opt| {
                    opt.strike_price.value() == target_high_strike
                        && self.is_valid_long_option(opt, &side)
                }) {
                    // Verificar precios válidos
                    if !self.are_valid_prices(&StrategyLegs::ThreeLegs {
                        first: low_strike,
                        second: middle_strike,
                        third: high_strike,
                    }) {
                        continue;
                    }

                    // Crear y validar nueva estrategia
                    let new_strategy = self.create_strategy(
                        option_chain,
                        &StrategyLegs::ThreeLegs {
                            first: low_strike,
                            second: middle_strike,
                            third: high_strike,
                        },
                    );

                    if !new_strategy.validate() {
                        continue;
                    }

                    if new_strategy.max_profit().is_err() || new_strategy.max_loss().is_err() {
                        continue;
                    }

                    let current_value = match criteria {
                        OptimizationCriteria::Ratio => new_strategy.profit_ratio(),
                        OptimizationCriteria::Area => new_strategy.profit_area(),
                    };

                    if current_value > best_value {
                        best_value = current_value;
                        *self = new_strategy;
                    }
                }
            }
        }
    }

    fn are_valid_prices(&self, legs: &StrategyLegs) -> bool {
        match legs {
            StrategyLegs::ThreeLegs {
                first: low_strike,
                second: middle_strike,
                third: high_strike,
            } => {
                low_strike.call_ask.unwrap_or(PZERO) > PZERO
                    && middle_strike.call_bid.unwrap_or(PZERO) > PZERO
                    && high_strike.call_ask.unwrap_or(PZERO) > PZERO
            }
            _ => false,
        }
    }

    fn create_strategy(&self, chain: &OptionChain, legs: &StrategyLegs) -> Self::Strategy {
        match legs {
            StrategyLegs::ThreeLegs {
                first: low_strike,
                second: middle_strike,
                third: high_strike,
            } => LongButterfly::new(
                chain.symbol.clone(),
                chain.underlying_price,
                low_strike.strike_price,
                middle_strike.strike_price,
                high_strike.strike_price,
                self.long_call_low.option.expiration_date.clone(),
                middle_strike.implied_volatility.unwrap().value() / 100.0,
                self.long_call_low.option.risk_free_rate,
                self.long_call_low.option.dividend_yield,
                self.long_call_low.option.quantity,
                low_strike.call_ask.unwrap().value(),
                middle_strike.call_bid.unwrap().value(),
                high_strike.call_ask.unwrap().value(),
                self.fees(),
            ),
            _ => panic!("Invalid number of legs for Long Butterfly strategy"),
        }
    }
}

// Implementación para Long Butterfly
impl Strategies for LongButterfly {
    fn get_underlying_price(&self) -> PositiveF64 {
        self.long_call_low.option.underlying_price
    }

    fn add_leg(&mut self, position: Position) {
        match (&position.option.side, position.option.quantity) {
            (Side::Long, q) if q == self.long_call_low.option.quantity => {
                if position.option.strike_price < self.short_calls.option.strike_price {
                    self.long_call_low = position;
                } else {
                    self.long_call_high = position;
                }
            }
            (Side::Short, _) => self.short_calls = position,
            _ => debug!("Invalid position for Long Butterfly strategy"),
        }
    }

    fn get_legs(&self) -> Vec<Position> {
        vec![
            self.long_call_low.clone(),
            self.short_calls.clone(),
            self.long_call_high.clone(),
        ]
    }

    fn max_profit(&self) -> Result<PositiveF64, &str> {
        let profit = self.calculate_profit_at(self.short_calls.option.strike_price);
        println!("profit: {}", profit);
        if profit > ZERO {
            Ok(pos!(profit))
        } else {
            Err("Profit is negative")
        }
    }

    fn max_loss(&self) -> Result<PositiveF64, &str> {
        let left_loss = self.calculate_profit_at(self.long_call_low.option.strike_price);
        let right_loss = self.calculate_profit_at(self.long_call_high.option.strike_price);
        let max_loss = left_loss.abs().max(right_loss.abs());
        if max_loss < ZERO {
            Ok(pos!(max_loss))
        } else {
            Err("Loss is negative")
        }
    }

    fn total_cost(&self) -> PositiveF64 {
        pos!(
            self.long_call_low.net_cost()
                + self.short_calls.net_cost()
                + self.long_call_high.net_cost()
        )
    }

    fn net_premium_received(&self) -> f64 {
        self.short_calls.net_premium_received()
            - self.long_call_low.net_cost()
            - self.long_call_high.net_cost()
    }

    fn fees(&self) -> f64 {
        self.long_call_low.open_fee
            + self.long_call_low.close_fee
            + self.short_calls.open_fee
            + self.short_calls.close_fee
            + self.long_call_high.open_fee
            + self.long_call_high.close_fee
    }

    fn profit_area(&self) -> f64 {
        let high = self.max_profit().unwrap_or(PZERO);
        let base = self.short_calls.option.strike_price - self.long_call_low.option.strike_price;
        (high * base / 100.0).value()
    }

    fn profit_ratio(&self) -> f64 {
        let max_profit = self.max_profit().unwrap_or(PZERO);
        let max_loss = self.max_loss().unwrap_or(PZERO);
        match (max_profit, max_loss) {
            (PZERO, _) => ZERO,
            (_, PZERO) => f64::INFINITY,
            _ => (max_profit / max_loss * 100.0).value(),
        }
    }

    fn get_break_even_points(&self) -> Vec<PositiveF64> {
        self.break_even_points.clone()
    }
}

// Implementación para Long Butterfly
impl Profit for LongButterfly {
    fn calculate_profit_at(&self, price: PositiveF64) -> f64 {
        let price = Some(price);
        self.long_call_low.pnl_at_expiration(&price)
            + self.short_calls.pnl_at_expiration(&price)
            + self.long_call_high.pnl_at_expiration(&price)
    }
}

// Implementación para Long Butterfly
impl Graph for LongButterfly {
    fn title(&self) -> String {
        let strategy_title = format!(
            "{:?} Strategy on {} Size {}:",
            self.kind,
            self.long_call_low.option.underlying_symbol,
            self.long_call_low.option.quantity
        );

        let leg_titles = vec![
            format!(
                "Long Call Low Strike: ${}",
                self.long_call_low.option.strike_price
            ),
            format!(
                "Short Calls Middle Strike: ${}",
                self.short_calls.option.strike_price
            ),
            format!(
                "Long Call High Strike: ${}",
                self.long_call_high.option.strike_price
            ),
            format!("Expire: {}", self.long_call_low.option.expiration_date),
        ];

        format!("{}\n\t{}", strategy_title, leg_titles.join("\n\t"))
    }

    fn get_vertical_lines(&self) -> Vec<ChartVerticalLine<f64, f64>> {
        vec![ChartVerticalLine {
            x_coordinate: self.long_call_low.option.underlying_price.value(),
            y_range: (-50000.0, 50000.0),
            label: format!(
                "Current Price: {}",
                self.long_call_low.option.underlying_price
            ),
            label_offset: (5.0, 5.0),
            line_color: ORANGE,
            label_color: ORANGE,
            line_style: ShapeStyle::from(&ORANGE).stroke_width(2),
            font_size: 18,
        }]
    }

    fn get_points(&self) -> Vec<ChartPoint<(f64, f64)>> {
        let mut points = Vec::new();
        let max_profit = self.max_profit().unwrap_or(PZERO);

        let left_loss = self
            .calculate_profit_at(self.long_call_low.option.strike_price)
            .abs();
        let right_loss = self
            .calculate_profit_at(self.long_call_high.option.strike_price)
            .abs();

        // Break-even points
        points.extend(
            self.break_even_points
                .iter()
                .enumerate()
                .map(|(i, &price)| ChartPoint {
                    coordinates: (price.value(), 0.0),
                    label: format!(
                        "Break Even {}: {:.2}",
                        if i == 0 { "Lower" } else { "Upper" },
                        price
                    ),
                    label_offset: LabelOffsetType::Relative(5.0, 5.0),
                    point_color: DARK_BLUE,
                    label_color: DARK_BLUE,
                    point_size: 5,
                    font_size: 18,
                }),
        );

        // Maximum profit point (at middle strike)
        points.push(ChartPoint {
            coordinates: (
                self.short_calls.option.strike_price.value(),
                max_profit.value(),
            ),
            label: format!("Max Profit {:.2}", max_profit),
            label_offset: LabelOffsetType::Relative(5.0, 10.0),
            point_color: DARK_GREEN,
            label_color: DARK_GREEN,
            point_size: 5,
            font_size: 18,
        });

        // Maximum loss points (at wing strikes)
        points.push(ChartPoint {
            coordinates: (self.long_call_low.option.strike_price.value(), -left_loss),
            label: format!("Left Loss {:.2}", left_loss),
            label_offset: LabelOffsetType::Relative(-50.0, -10.0),
            point_color: RED,
            label_color: RED,
            point_size: 5,
            font_size: 18,
        });

        points.push(ChartPoint {
            coordinates: (self.long_call_high.option.strike_price.value(), -right_loss),
            label: format!("Max Loss {:.2}", right_loss),
            label_offset: LabelOffsetType::Relative(5.0, -10.0),
            point_color: RED,
            label_color: RED,
            point_size: 5,
            font_size: 18,
        });

        // Current price point
        points.push(self.get_point_at_price(self.long_call_low.option.underlying_price));

        points
    }
}

// Implementación para Long Butterfly
impl ProbabilityAnalysis for LongButterfly {
    fn get_expiration(&self) -> Result<ExpirationDate, String> {
        Ok(self.long_call_low.option.expiration_date.clone())
    }

    fn get_risk_free_rate(&self) -> Option<f64> {
        Some(self.long_call_low.option.risk_free_rate)
    }

    fn get_profit_ranges(&self) -> Result<Vec<ProfitLossRange>, String> {
        let break_even_points = self.get_break_even_points();

        // En Long Butterfly, el rango de beneficio está entre los puntos de equilibrio
        let (mean_volatility, std_dev) = mean_and_std(vec![
            pos!(self.long_call_low.option.implied_volatility),
            pos!(self.short_calls.option.implied_volatility),
            pos!(self.long_call_high.option.implied_volatility),
        ]);

        let mut profit_range = ProfitLossRange::new(
            Some(break_even_points[0]),
            Some(break_even_points[1]),
            pos!(self.max_profit()?.value()),
        )?;

        profit_range.calculate_probability(
            self.get_underlying_price(),
            Some(VolatilityAdjustment {
                base_volatility: mean_volatility,
                std_dev_adjustment: std_dev,
            }),
            None,
            self.get_expiration()?,
            self.get_risk_free_rate(),
        )?;

        Ok(vec![profit_range])
    }

    fn get_loss_ranges(&self) -> Result<Vec<ProfitLossRange>, String> {
        let mut ranges = Vec::new();
        let break_even_points = self.get_break_even_points();

        let (mean_volatility, std_dev) = mean_and_std(vec![
            pos!(self.long_call_low.option.implied_volatility),
            pos!(self.short_calls.option.implied_volatility),
            pos!(self.long_call_high.option.implied_volatility),
        ]);

        let volatility_adjustment = Some(VolatilityAdjustment {
            base_volatility: mean_volatility,
            std_dev_adjustment: std_dev,
        });

        // Rango de pérdida inferior (por debajo del break-even inferior)
        let mut lower_loss_range = ProfitLossRange::new(
            Some(self.long_call_low.option.strike_price),
            Some(break_even_points[0]),
            pos!(self.max_loss()?.value()),
        )?;

        lower_loss_range.calculate_probability(
            self.get_underlying_price(),
            volatility_adjustment.clone(),
            None,
            self.get_expiration()?,
            self.get_risk_free_rate(),
        )?;

        ranges.push(lower_loss_range);

        // Rango de pérdida superior (por encima del break-even superior)
        let mut upper_loss_range = ProfitLossRange::new(
            Some(break_even_points[1]),
            Some(self.long_call_high.option.strike_price),
            pos!(self.max_loss()?.value()),
        )?;

        upper_loss_range.calculate_probability(
            self.get_underlying_price(),
            volatility_adjustment,
            None,
            self.get_expiration()?,
            self.get_risk_free_rate(),
        )?;

        ranges.push(upper_loss_range);

        Ok(ranges)
    }
}

/*
Short Butterfly Spread Strategy

A short butterfly spread involves:
1. Sell one call at a lower strike price (ITM)
2. Buy two calls at a middle strike price (ATM)
3. Sell one call at a higher strike price (OTM)

All options have the same expiration date.

Key characteristics:
- Limited profit potential
- Limited risk
- Loss is highest when price is exactly at middle strike at expiration
- Maximum profit is limited to the net premium received
- All options must have same expiration date
*/

#[derive(Clone, Debug)]
pub struct ShortButterfly {
    pub name: String,
    pub kind: StrategyType,
    pub description: String,
    pub break_even_points: Vec<PositiveF64>,
    short_call_low: Position,
    long_calls: Position,
    short_call_high: Position,
}

const SHORT_BUTTERFLY_DESCRIPTION: &str =
    "A short butterfly spread is created by selling one call at a lower strike price, \
    buying two calls at a middle strike price, and selling one call at a higher strike price, \
    all with the same expiration date. This strategy profits when the underlying price moves \
    significantly away from the middle strike price in either direction.";

impl ShortButterfly {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        underlying_symbol: String,
        underlying_price: PositiveF64,
        low_strike: PositiveF64,
        middle_strike: PositiveF64,
        high_strike: PositiveF64,
        expiration: ExpirationDate,
        implied_volatility: f64,
        risk_free_rate: f64,
        dividend_yield: f64,
        quantity: PositiveF64,
        premium_low: f64,
        premium_middle: f64,
        premium_high: f64,
        fees: f64,
    ) -> Self {
        let mut strategy = ShortButterfly {
            name: "Short Butterfly".to_string(),
            kind: StrategyType::ShortButterfly,
            description: SHORT_BUTTERFLY_DESCRIPTION.to_string(),
            break_even_points: Vec::new(),
            short_call_low: Position::default(),
            long_calls: Position::default(),
            short_call_high: Position::default(),
        };

        // Create short call at lower strike
        let short_call_low = Options::new(
            OptionType::European,
            Side::Short,
            underlying_symbol.clone(),
            low_strike,
            expiration.clone(),
            implied_volatility,
            quantity,
            underlying_price,
            risk_free_rate,
            OptionStyle::Call,
            dividend_yield,
            None,
        );
        strategy.short_call_low = Position::new(
            short_call_low,
            premium_low,
            Utc::now(),
            fees / 8.0,
            fees / 8.0,
        );

        // Create two long calls at middle strike
        let long_calls = Options::new(
            OptionType::European,
            Side::Long,
            underlying_symbol.clone(),
            middle_strike,
            expiration.clone(),
            implied_volatility,
            quantity * 2.0, // Double quantity for middle strike
            underlying_price,
            risk_free_rate,
            OptionStyle::Call,
            dividend_yield,
            None,
        );
        strategy.long_calls = Position::new(
            long_calls,
            premium_middle,
            Utc::now(),
            fees / 8.0,
            fees / 8.0,
        );

        // Create short call at higher strike
        let short_call_high = Options::new(
            OptionType::European,
            Side::Short,
            underlying_symbol,
            high_strike,
            expiration,
            implied_volatility,
            quantity,
            underlying_price,
            risk_free_rate,
            OptionStyle::Call,
            dividend_yield,
            None,
        );
        strategy.short_call_high = Position::new(
            short_call_high,
            premium_high,
            Utc::now(),
            fees / 8.0,
            fees / 8.0,
        );

        strategy.validate();

        // Calculate break-even points

        let left_profit = strategy.calculate_profit_at(low_strike) / quantity.value();
        let right_profit = strategy.calculate_profit_at(high_strike) / quantity.value();
        strategy.break_even_points.push(low_strike + left_profit);
        strategy.break_even_points.push(high_strike - right_profit);

        strategy
    }
}

// Implementación para Short Butterfly
impl Validable for ShortButterfly {
    fn validate(&self) -> bool {
        // Validar que todas las posiciones son válidas
        if !self.short_call_low.validate() {
            debug!("Short call (low strike) is invalid");
            return false;
        }
        if !self.long_calls.validate() {
            debug!("Long calls (middle strike) are invalid");
            return false;
        }
        if !self.short_call_high.validate() {
            debug!("Short call (high strike) is invalid");
            return false;
        }

        // Validar el orden correcto de los strikes
        if self.short_call_low.option.strike_price >= self.long_calls.option.strike_price {
            debug!("Low strike must be lower than middle strike");
            return false;
        }
        if self.long_calls.option.strike_price >= self.short_call_high.option.strike_price {
            debug!("Middle strike must be lower than high strike");
            return false;
        }

        // Validar que las cantidades son correctas
        if self.long_calls.option.quantity != self.short_call_low.option.quantity * 2.0 {
            debug!("Middle strike quantity must be double the wing quantities");
            return false;
        }
        if self.short_call_low.option.quantity != self.short_call_high.option.quantity {
            debug!("Wing quantities must be equal");
            return false;
        }

        // Validar que el ancho de las alas es igual
        let lower_wing =
            self.long_calls.option.strike_price - self.short_call_low.option.strike_price;
        let upper_wing =
            self.short_call_high.option.strike_price - self.long_calls.option.strike_price;
        if lower_wing != upper_wing {
            debug!("Wings must be symmetrical");
            return false;
        }

        // Validar que todas las opciones tienen la misma fecha de expiración
        if self.short_call_low.option.expiration_date != self.long_calls.option.expiration_date
            || self.long_calls.option.expiration_date != self.short_call_high.option.expiration_date
        {
            debug!("All options must have the same expiration date");
            return false;
        }

        true
    }
}

// Implementación para Short Butterfly
impl Optimizable for ShortButterfly {
    type Strategy = ShortButterfly;

    fn find_optimal(
        &mut self,
        option_chain: &OptionChain,
        side: FindOptimalSide,
        criteria: OptimizationCriteria,
    ) {
        let mut best_value = f64::NEG_INFINITY;
        let options: Vec<&OptionData> = option_chain.options.iter().collect();

        for (i, low_strike) in options.iter().enumerate() {
            if !self.is_valid_short_option(low_strike, &side) {
                continue;
            }

            for (_j, middle_strike) in options.iter().enumerate().skip(i + 1) {
                if !self.is_valid_long_option(middle_strike, &side) {
                    continue;
                }

                // Encontrar un strike alto que mantenga la simetría
                let middle_to_low =
                    middle_strike.strike_price.value() - low_strike.strike_price.value();
                let target_high_strike = middle_strike.strike_price.value() + middle_to_low;

                if let Some(high_strike) = options.iter().find(|&opt| {
                    opt.strike_price.value() == target_high_strike
                        && self.is_valid_short_option(opt, &side)
                }) {
                    // Verificar precios válidos
                    if !self.are_valid_prices(&StrategyLegs::ThreeLegs {
                        first: low_strike,
                        second: middle_strike,
                        third: high_strike,
                    }) {
                        continue;
                    }

                    // Crear y validar nueva estrategia
                    let new_strategy = self.create_strategy(
                        option_chain,
                        &StrategyLegs::ThreeLegs {
                            first: low_strike,
                            second: middle_strike,
                            third: high_strike,
                        },
                    );

                    if !new_strategy.validate() {
                        continue;
                    }

                    if new_strategy.max_profit().is_err() || new_strategy.max_loss().is_err() {
                        continue;
                    }

                    let current_value = match criteria {
                        OptimizationCriteria::Ratio => new_strategy.profit_ratio(),
                        OptimizationCriteria::Area => new_strategy.profit_area(),
                    };

                    if current_value > best_value {
                        best_value = current_value;
                        *self = new_strategy;
                    }
                }
            }
        }
    }

    fn are_valid_prices(&self, legs: &StrategyLegs) -> bool {
        match legs {
            StrategyLegs::ThreeLegs {
                first: low_strike,
                second: middle_strike,
                third: high_strike,
            } => {
                low_strike.call_bid.unwrap_or(PZERO) > PZERO
                    && middle_strike.call_ask.unwrap_or(PZERO) > PZERO
                    && high_strike.call_bid.unwrap_or(PZERO) > PZERO
            }
            _ => false,
        }
    }

    fn create_strategy(&self, chain: &OptionChain, legs: &StrategyLegs) -> Self::Strategy {
        match legs {
            StrategyLegs::ThreeLegs {
                first: low_strike,
                second: middle_strike,
                third: high_strike,
            } => ShortButterfly::new(
                chain.symbol.clone(),
                chain.underlying_price,
                low_strike.strike_price,
                middle_strike.strike_price,
                high_strike.strike_price,
                self.short_call_low.option.expiration_date.clone(),
                middle_strike.implied_volatility.unwrap().value() / 100.0,
                self.short_call_low.option.risk_free_rate,
                self.short_call_low.option.dividend_yield,
                self.short_call_low.option.quantity,
                low_strike.call_bid.unwrap().value(),
                middle_strike.call_ask.unwrap().value(),
                high_strike.call_bid.unwrap().value(),
                self.fees(),
            ),
            _ => panic!("Invalid number of legs for Short Butterfly strategy"),
        }
    }
}

// Implementación para Short Butterfly
impl Strategies for ShortButterfly {
    fn get_underlying_price(&self) -> PositiveF64 {
        self.short_call_low.option.underlying_price
    }

    fn add_leg(&mut self, position: Position) {
        match (&position.option.side, position.option.quantity) {
            (Side::Short, q) if q == self.short_call_low.option.quantity => {
                if position.option.strike_price < self.long_calls.option.strike_price {
                    self.short_call_low = position;
                } else {
                    self.short_call_high = position;
                }
            }
            (Side::Long, _) => self.long_calls = position,
            _ => debug!("Invalid position for Short Butterfly strategy"),
        }
    }

    fn get_legs(&self) -> Vec<Position> {
        vec![
            self.short_call_low.clone(),
            self.long_calls.clone(),
            self.short_call_high.clone(),
        ]
    }

    fn max_profit(&self) -> Result<PositiveF64, &str> {
        let left_profit = self.calculate_profit_at(self.short_call_low.option.strike_price);
        let right_profit = self.calculate_profit_at(self.short_call_high.option.strike_price);
        let max_profit = left_profit.max(right_profit);
        if max_profit > ZERO {
            Ok(pos!(max_profit))
        } else {
            Err("Profit is negative")
        }
    }

    fn max_loss(&self) -> Result<PositiveF64, &str> {
        let loss = self.calculate_profit_at(self.long_calls.option.strike_price);
        if loss < ZERO {
            Ok(pos!(loss.abs()))
        } else {
            Err("Loss is negative")
        }
    }

    fn total_cost(&self) -> PositiveF64 {
        pos!(
            self.short_call_low.net_cost()
                + self.long_calls.net_cost()
                + self.short_call_high.net_cost()
        )
    }

    fn net_premium_received(&self) -> f64 {
        self.short_call_low.net_premium_received() + self.short_call_high.net_premium_received()
            - self.long_calls.net_cost()
    }

    fn fees(&self) -> f64 {
        self.short_call_low.open_fee
            + self.short_call_low.close_fee
            + self.long_calls.open_fee
            + self.long_calls.close_fee
            + self.short_call_high.open_fee
            + self.short_call_high.close_fee
    }

    fn profit_area(&self) -> f64 {
        let high = self.max_profit().unwrap_or(PZERO);
        let base = self.long_calls.option.strike_price - self.short_call_low.option.strike_price;
        (high * base / 100.0).value()
    }

    fn profit_ratio(&self) -> f64 {
        let max_profit = self.max_profit().unwrap_or(PZERO);
        let max_loss = self.max_loss().unwrap_or(PZERO);
        match (max_profit, max_loss) {
            (PZERO, _) => ZERO,
            (_, PZERO) => f64::INFINITY,
            _ => (max_profit / max_loss * 100.0).value(),
        }
    }

    fn get_break_even_points(&self) -> Vec<PositiveF64> {
        self.break_even_points.clone()
    }
}

// Implementación para Short Butterfly
impl Profit for ShortButterfly {
    fn calculate_profit_at(&self, price: PositiveF64) -> f64 {
        let price = Some(price);
        // Sumar el PnL de cada posición:
        // Short call bajo strike + Long calls medio strike + Short call alto strike
        self.short_call_low.pnl_at_expiration(&price)
            + self.long_calls.pnl_at_expiration(&price)
            + self.short_call_high.pnl_at_expiration(&price)
    }
}

// Implementación para Short Butterfly
impl Graph for ShortButterfly {
    fn title(&self) -> String {
        let strategy_title = format!(
            "{:?} Strategy on {} Size {}:",
            self.kind,
            self.short_call_low.option.underlying_symbol,
            self.short_call_low.option.quantity
        );

        let leg_titles = vec![
            format!(
                "Short Call Low Strike: ${}",
                self.short_call_low.option.strike_price
            ),
            format!(
                "Long Calls Middle Strike: ${}",
                self.long_calls.option.strike_price
            ),
            format!(
                "Short Call High Strike: ${}",
                self.short_call_high.option.strike_price
            ),
            format!("Expire: {}", self.short_call_low.option.expiration_date),
        ];

        format!("{}\n\t{}", strategy_title, leg_titles.join("\n\t"))
    }

    fn get_vertical_lines(&self) -> Vec<ChartVerticalLine<f64, f64>> {
        vec![ChartVerticalLine {
            x_coordinate: self.short_call_low.option.underlying_price.value(),
            y_range: (-50000.0, 50000.0),
            label: format!(
                "Current Price: {}",
                self.short_call_low.option.underlying_price
            ),
            label_offset: (5.0, 5.0),
            line_color: ORANGE,
            label_color: ORANGE,
            line_style: ShapeStyle::from(&ORANGE).stroke_width(2),
            font_size: 18,
        }]
    }

    fn get_points(&self) -> Vec<ChartPoint<(f64, f64)>> {
        let mut points = Vec::new();
        let max_loss = self.max_loss().unwrap_or(PZERO);
        let left_profit = self.calculate_profit_at(self.short_call_low.option.strike_price);
        let right_profit = self.calculate_profit_at(self.short_call_high.option.strike_price);

        // Break-even points
        points.extend(
            self.break_even_points
                .iter()
                .enumerate()
                .map(|(i, &price)| ChartPoint {
                    coordinates: (price.value(), 0.0),
                    label: format!(
                        "Break Even {}: {:.2}",
                        if i == 0 { "Lower" } else { "Upper" },
                        price
                    ),
                    label_offset: LabelOffsetType::Relative(5.0, 5.0),
                    point_color: DARK_BLUE,
                    label_color: DARK_BLUE,
                    point_size: 5,
                    font_size: 18,
                }),
        );

        // Maximum loss point (at middle strike)
        points.push(ChartPoint {
            coordinates: (
                self.long_calls.option.strike_price.value(),
                -max_loss.value(),
            ),
            label: format!("Max Loss {:.2}", max_loss),
            label_offset: LabelOffsetType::Relative(5.0, -10.0),
            point_color: RED,
            label_color: RED,
            point_size: 5,
            font_size: 18,
        });

        // Maximum profit points (at wing strikes)
        points.push(ChartPoint {
            coordinates: (self.short_call_low.option.strike_price.value(), left_profit),
            label: format!("Left Max Profit {:.2}", left_profit),
            label_offset: LabelOffsetType::Relative(-30.0, 10.0),
            point_color: DARK_GREEN,
            label_color: DARK_GREEN,
            point_size: 5,
            font_size: 18,
        });

        points.push(ChartPoint {
            coordinates: (
                self.short_call_high.option.strike_price.value(),
                right_profit,
            ),
            label: format!("Right Max Profit {:.2}", right_profit),
            label_offset: LabelOffsetType::Relative(5.0, 10.0),
            point_color: DARK_GREEN,
            label_color: DARK_GREEN,
            point_size: 5,
            font_size: 18,
        });

        // Current price point
        points.push(self.get_point_at_price(self.short_call_low.option.underlying_price));

        points
    }
}

// Implementación para Short Butterfly
impl ProbabilityAnalysis for ShortButterfly {
    fn get_expiration(&self) -> Result<ExpirationDate, String> {
        Ok(self.short_call_low.option.expiration_date.clone())
    }

    fn get_risk_free_rate(&self) -> Option<f64> {
        Some(self.short_call_low.option.risk_free_rate)
    }

    fn get_profit_ranges(&self) -> Result<Vec<ProfitLossRange>, String> {
        let mut ranges = Vec::new();
        let break_even_points = self.get_break_even_points();

        let (mean_volatility, std_dev) = mean_and_std(vec![
            pos!(self.short_call_low.option.implied_volatility),
            pos!(self.long_calls.option.implied_volatility),
            pos!(self.short_call_high.option.implied_volatility),
        ]);

        let volatility_adjustment = Some(VolatilityAdjustment {
            base_volatility: mean_volatility,
            std_dev_adjustment: std_dev,
        });

        // Rango de beneficio inferior (por debajo del break-even inferior)
        let mut lower_profit_range = ProfitLossRange::new(
            Some(self.short_call_low.option.strike_price),
            Some(break_even_points[0]),
            pos!(self.max_profit()?.value()),
        )?;

        lower_profit_range.calculate_probability(
            self.get_underlying_price(),
            volatility_adjustment.clone(),
            None,
            self.get_expiration()?,
            self.get_risk_free_rate(),
        )?;

        ranges.push(lower_profit_range);

        // Rango de beneficio superior (por encima del break-even superior)
        let mut upper_profit_range = ProfitLossRange::new(
            Some(break_even_points[1]),
            Some(self.short_call_high.option.strike_price),
            pos!(self.max_profit()?.value()),
        )?;

        upper_profit_range.calculate_probability(
            self.get_underlying_price(),
            volatility_adjustment,
            None,
            self.get_expiration()?,
            self.get_risk_free_rate(),
        )?;

        ranges.push(upper_profit_range);

        Ok(ranges)
    }

    fn get_loss_ranges(&self) -> Result<Vec<ProfitLossRange>, String> {
        let break_even_points = self.get_break_even_points();

        // En Short Butterfly, el rango de pérdida está entre los puntos de equilibrio
        let (mean_volatility, std_dev) = mean_and_std(vec![
            pos!(self.short_call_low.option.implied_volatility),
            pos!(self.long_calls.option.implied_volatility),
            pos!(self.short_call_high.option.implied_volatility),
        ]);

        let mut loss_range = ProfitLossRange::new(
            Some(break_even_points[0]),
            Some(break_even_points[1]),
            pos!(self.max_loss()?.value()),
        )?;

        loss_range.calculate_probability(
            self.get_underlying_price(),
            Some(VolatilityAdjustment {
                base_volatility: mean_volatility,
                std_dev_adjustment: std_dev,
            }),
            None,
            self.get_expiration()?,
            self.get_risk_free_rate(),
        )?;

        Ok(vec![loss_range])
    }
}

#[cfg(test)]
mod tests_butterfly_graph {
    use super::*;
    use crate::model::types::ExpirationDate;
    use crate::pos;

    fn create_test_long_butterfly() -> LongButterfly {
        LongButterfly::new(
            "TEST".to_string(),
            pos!(100.0),
            pos!(90.0),
            pos!(100.0),
            pos!(110.0),
            ExpirationDate::Days(30.0),
            0.20,
            0.05,
            0.0,
            pos!(1.0),
            3.0,
            2.0,
            1.0,
            0.0,
        )
    }

    fn create_test_short_butterfly() -> ShortButterfly {
        ShortButterfly::new(
            "TEST".to_string(),
            pos!(100.0),
            pos!(90.0),
            pos!(100.0),
            pos!(110.0),
            ExpirationDate::Days(30.0),
            0.20,
            0.05,
            0.0,
            pos!(1.0),
            3.0,
            2.0,
            1.0,
            0.0,
        )
    }

    #[test]
    fn test_long_butterfly_title() {
        let butterfly = create_test_long_butterfly();
        let title = butterfly.title();

        assert!(title.contains("LongButterfly Strategy"));
        assert!(title.contains("TEST"));
        assert!(title.contains("Size 1"));
        assert!(title.contains("90"));
        assert!(title.contains("100"));
        assert!(title.contains("110"));
        assert!(title.contains("Expire"));
    }

    #[test]
    fn test_short_butterfly_title() {
        let butterfly = create_test_short_butterfly();
        let title = butterfly.title();

        assert!(title.contains("ShortButterfly Strategy"));
        assert!(title.contains("TEST"));
        assert!(title.contains("Size 1"));
        assert!(title.contains("90"));
        assert!(title.contains("100"));
        assert!(title.contains("110"));
        assert!(title.contains("Expire"));
    }

    #[test]
    fn test_long_butterfly_vertical_lines() {
        let butterfly = create_test_long_butterfly();
        let lines = butterfly.get_vertical_lines();

        assert_eq!(lines.len(), 1);
        let line = &lines[0];
        assert_eq!(line.x_coordinate, 100.0);
        assert!(line.label.contains("Current Price"));
    }

    #[test]
    fn test_long_butterfly_points() {
        let butterfly = create_test_long_butterfly();
        let points = butterfly.get_points();

        // Should have 6 points: 2 break-even, 1 max profit, 2 max loss, 1 current price
        assert_eq!(points.len(), 6);

        // Verify break-even points
        let break_even_points: Vec<&ChartPoint<(f64, f64)>> = points
            .iter()
            .filter(|p| p.label.contains("Break Even"))
            .collect();
        assert_eq!(break_even_points.len(), 2);
        assert_eq!(break_even_points[0].coordinates.1, 0.0);
        assert_eq!(break_even_points[1].coordinates.1, 0.0);

        // Verify max profit point
        let max_profit_point = points
            .iter()
            .find(|p| p.label.contains("Max Profit"))
            .unwrap();
        assert_eq!(max_profit_point.coordinates.0, 100.0);
        assert!(max_profit_point.coordinates.1 > 0.0);

        // Verify max loss points
        let max_loss_points: Vec<&ChartPoint<(f64, f64)>> = points
            .iter()
            .filter(|p| p.label.contains("Max Loss"))
            .collect();
        assert_eq!(max_loss_points.len(), 2);
        assert!(max_loss_points[0].coordinates.1 < 0.0);
        assert!(max_loss_points[1].coordinates.1 < 0.0);
    }

    #[test]
    fn test_short_butterfly_points() {
        let butterfly = create_test_short_butterfly();
        let points = butterfly.get_points();

        // Should have 6 points: 2 break-even, 2 max profit, 1 max loss, 1 current price
        assert_eq!(points.len(), 6);

        // Verify break-even points
        let break_even_points: Vec<&ChartPoint<(f64, f64)>> = points
            .iter()
            .filter(|p| p.label.contains("Break Even"))
            .collect();
        assert_eq!(break_even_points.len(), 2);
        assert_eq!(break_even_points[0].coordinates.1, 0.0);
        assert_eq!(break_even_points[1].coordinates.1, 0.0);

        // Verify max profit points
        let max_profit_points: Vec<&ChartPoint<(f64, f64)>> = points
            .iter()
            .filter(|p| p.label.contains("Max Profit"))
            .collect();
        assert_eq!(max_profit_points.len(), 2);
        assert!(max_profit_points[0].coordinates.1 > 0.0);
        assert!(max_profit_points[1].coordinates.1 > 0.0);

        // Verify max loss point
        let max_loss_point = points
            .iter()
            .find(|p| p.label.contains("Max Loss"))
            .unwrap();
        assert_eq!(max_loss_point.coordinates.0, 100.0);
        assert!(max_loss_point.coordinates.1 < 0.0);
    }

    #[test]
    fn test_point_colors() {
        let long_butterfly = create_test_long_butterfly();
        let short_butterfly = create_test_short_butterfly();

        for points in [long_butterfly.get_points(), short_butterfly.get_points()] {
            for point in points {
                match point.label.as_str() {
                    label if label.contains("Break Even") => {
                        assert_eq!(point.point_color, DARK_BLUE);
                    }
                    label if label.contains("Max Profit") => {
                        assert_eq!(point.point_color, DARK_GREEN);
                    }
                    label if label.contains("Max Loss") => {
                        assert_eq!(point.point_color, RED);
                    }
                    _ => {}
                }
            }
        }
    }
}

#[cfg(test)]
mod tests_butterfly_profit {
    use super::*;
    use crate::model::types::ExpirationDate;
    use crate::pos;

    fn create_test_long_butterfly() -> LongButterfly {
        LongButterfly::new(
            "TEST".to_string(),
            pos!(100.0), // underlying_price
            pos!(90.0),  // low_strike
            pos!(100.0), // middle_strike
            pos!(110.0), // high_strike
            ExpirationDate::Days(30.0),
            0.20,      // implied_volatility
            0.05,      // risk_free_rate
            0.0,       // dividend_yield
            pos!(1.0), // quantity
            3.0,       // premium_low
            2.0,       // premium_middle
            1.0,       // premium_high
            0.0,       // fees
        )
    }

    fn create_test_short_butterfly() -> ShortButterfly {
        ShortButterfly::new(
            "TEST".to_string(),
            pos!(100.0), // underlying_price
            pos!(90.0),  // low_strike
            pos!(100.0), // middle_strike
            pos!(110.0), // high_strike
            ExpirationDate::Days(30.0),
            0.20,      // implied_volatility
            0.05,      // risk_free_rate
            0.0,       // dividend_yield
            pos!(1.0), // quantity
            3.0,       // premium_low
            2.0,       // premium_middle
            1.0,       // premium_high
            0.0,       // fees
        )
    }

    // Tests para Long Butterfly
    #[test]
    fn test_long_butterfly_profit_at_max_profit_price() {
        let butterfly = create_test_long_butterfly();
        let profit = butterfly.calculate_profit_at(pos!(100.0));
        // En el strike medio, deberíamos tener el beneficio máximo
        assert_eq!(profit, 8.0); // (100 - 90) - 2*(100 - 100) - (110 - 100) - net_premium
    }

    #[test]
    fn test_long_butterfly_profit_below_lowest_strike() {
        let butterfly = create_test_long_butterfly();
        let profit = butterfly.calculate_profit_at(pos!(85.0));
        // Por debajo del strike más bajo, pérdida máxima
        assert_eq!(profit, -4.0); // -net_premium
    }

    #[test]
    fn test_long_butterfly_profit_above_highest_strike() {
        let butterfly = create_test_long_butterfly();
        let profit = butterfly.calculate_profit_at(pos!(115.0));
        // Por encima del strike más alto, pérdida máxima
        assert_eq!(profit, -4.0); // -net_premium
    }

    #[test]
    fn test_long_butterfly_profit_at_break_even() {
        let butterfly = create_test_long_butterfly();
        // Los puntos de equilibrio están entre los strikes extremos y el medio
        let profit_lower = butterfly.calculate_profit_at(pos!(95.0));
        let profit_upper = butterfly.calculate_profit_at(pos!(105.0));
        assert!(profit_lower.abs() < 0.001); // Cerca de cero
        assert!(profit_upper.abs() < 0.001); // Cerca de cero
    }

    // Tests para Short Butterfly
    #[test]
    fn test_short_butterfly_profit_at_max_loss_price() {
        let butterfly = create_test_short_butterfly();
        let profit = butterfly.calculate_profit_at(pos!(100.0));
        // En el strike medio, deberíamos tener la pérdida máxima
        assert_eq!(profit, -8.0); // -(100 - 90) + 2*(100 - 100) + (110 - 100) + net_premium
    }

    #[test]
    fn test_short_butterfly_profit_below_lowest_strike() {
        let butterfly = create_test_short_butterfly();
        let profit = butterfly.calculate_profit_at(pos!(85.0));
        // Por debajo del strike más bajo, beneficio máximo
        assert_eq!(profit, 4.0); // net_premium
    }

    #[test]
    fn test_short_butterfly_profit_above_highest_strike() {
        let butterfly = create_test_short_butterfly();
        let profit = butterfly.calculate_profit_at(pos!(115.0));
        // Por encima del strike más alto, beneficio máximo
        assert_eq!(profit, 4.0); // net_premium
    }

    #[test]
    fn test_short_butterfly_profit_at_break_even() {
        let butterfly = create_test_short_butterfly();
        // Los puntos de equilibrio están entre los strikes extremos y el medio
        let profit_lower = butterfly.calculate_profit_at(pos!(95.0));
        let profit_upper = butterfly.calculate_profit_at(pos!(105.0));
        assert!(profit_lower.abs() < 0.001); // Cerca de cero
        assert!(profit_upper.abs() < 0.001); // Cerca de cero
    }

    #[test]
    fn test_butterfly_profits_with_fees() {
        // Crear butterflies con comisiones
        let long_butterfly = LongButterfly::new(
            "TEST".to_string(),
            pos!(100.0),
            pos!(90.0),
            pos!(100.0),
            pos!(110.0),
            ExpirationDate::Days(30.0),
            0.20,
            0.05,
            0.0,
            pos!(1.0),
            3.0,
            2.0,
            1.0,
            3.0, // fees
        );

        let short_butterfly = ShortButterfly::new(
            "TEST".to_string(),
            pos!(100.0),
            pos!(90.0),
            pos!(100.0),
            pos!(110.0),
            ExpirationDate::Days(30.0),
            0.20,
            0.05,
            0.0,
            pos!(1.0),
            3.0,
            2.0,
            1.0,
            3.0, // fees
        );

        // Los beneficios deberían ser menores debido a las comisiones
        assert!(long_butterfly.calculate_profit_at(pos!(100.0)) < 8.0);
        assert!(short_butterfly.calculate_profit_at(pos!(85.0)) < 4.0);
    }

    #[test]
    fn test_butterfly_profits_with_quantity() {
        // Crear butterflies con cantidad = 2
        let long_butterfly = LongButterfly::new(
            "TEST".to_string(),
            pos!(100.0),
            pos!(90.0),
            pos!(100.0),
            pos!(110.0),
            ExpirationDate::Days(30.0),
            0.20,
            0.05,
            0.0,
            pos!(2.0), // quantity = 2
            3.0,
            2.0,
            1.0,
            0.0,
        );

        let short_butterfly = ShortButterfly::new(
            "TEST".to_string(),
            pos!(100.0),
            pos!(90.0),
            pos!(100.0),
            pos!(110.0),
            ExpirationDate::Days(30.0),
            0.20,
            0.05,
            0.0,
            pos!(2.0), // quantity = 2
            3.0,
            2.0,
            1.0,
            0.0,
        );

        // Los beneficios y pérdidas deberían ser el doble
        assert_eq!(long_butterfly.calculate_profit_at(pos!(100.0)), 16.0); // 2 * 8.0
        assert_eq!(short_butterfly.calculate_profit_at(pos!(85.0)), 8.0); // 2 * 4.0
    }
}
