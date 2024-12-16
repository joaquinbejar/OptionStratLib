/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 25/9/24
******************************************************************************/
use super::base::{Optimizable, Positionable, Strategies, StrategyType, Validable};
use crate::chains::chain::{OptionChain, OptionData};
use crate::constants::DARK_BLUE;
use crate::constants::{DARK_GREEN, ZERO};
use crate::greeks::equations::{Greek, Greeks};
use crate::model::option::Options;
use crate::model::position::Position;
use crate::model::types::{ExpirationDate, OptionStyle, OptionType, PositiveF64, Side, PZERO};
use crate::pos;
use crate::pricing::payoff::Profit;
use crate::strategies::delta_neutral::{
    DeltaAdjustment, DeltaInfo, DeltaNeutrality, DELTA_THRESHOLD,
};
use crate::strategies::utils::{FindOptimalSide, OptimizationCriteria};
use crate::visualization::model::{ChartPoint, ChartVerticalLine, LabelOffsetType};
use crate::visualization::utils::Graph;
use chrono::Utc;
use plotters::prelude::{ShapeStyle, RED};
use plotters::style::full_palette::ORANGE;
use tracing::{debug, error};

const RATIO_CALL_SPREAD_DESCRIPTION: &str =
    "A Ratio Call Spread involves buying one call option and selling multiple call options \
    at a higher strike price. This strategy is used when a moderate rise in the underlying \
    asset's price is expected, but with limited upside potential.";

#[derive(Clone, Debug)]
pub struct CallButterfly {
    pub name: String,
    pub kind: StrategyType,
    pub description: String,
    pub break_even_points: Vec<PositiveF64>,
    long_call: Position,
    short_call_low: Position,
    short_call_high: Position,
    underlying_price: PositiveF64,
}

impl CallButterfly {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        underlying_symbol: String,
        underlying_price: PositiveF64,
        long_call_strike: PositiveF64,
        short_call_low_strike: PositiveF64,
        short_call_high_strike: PositiveF64,
        expiration: ExpirationDate,
        implied_volatility: f64,
        risk_free_rate: f64,
        dividend_yield: f64,
        quantity: PositiveF64,
        premium_long_call: f64,
        premium_short_call_low: f64,
        premium_short_call_high: f64,
        open_fee_long: f64,
        close_fee_long: f64,
        open_fee_short_low: f64,
        close_fee_short_low: f64,
        open_fee_short_high: f64,
        close_fee_short_high: f64,
    ) -> Self {
        let mut strategy = CallButterfly {
            name: underlying_symbol.to_string(),
            kind: StrategyType::CallButterfly,
            description: RATIO_CALL_SPREAD_DESCRIPTION.to_string(),
            break_even_points: Vec::new(),
            long_call: Position::default(),
            short_call_low: Position::default(),
            short_call_high: Position::default(),
            underlying_price,
        };
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
            open_fee_long,
            close_fee_long,
        );
        strategy
            .add_position(&long_call.clone())
            .expect("Invalid short call");
        strategy.long_call = long_call;

        let short_call_low_option = Options::new(
            OptionType::European,
            Side::Short,
            underlying_symbol.clone(),
            short_call_low_strike,
            expiration.clone(),
            implied_volatility,
            quantity,
            underlying_price,
            risk_free_rate,
            OptionStyle::Call,
            dividend_yield,
            None,
        );
        let short_call_low = Position::new(
            short_call_low_option,
            premium_short_call_low,
            Utc::now(),
            open_fee_short_low,
            close_fee_short_low,
        );
        strategy
            .add_position(&short_call_low.clone())
            .expect("Invalid long call itm");
        strategy.short_call_low = short_call_low;

        let short_call_high_option = Options::new(
            OptionType::European,
            Side::Short,
            underlying_symbol.clone(),
            short_call_high_strike,
            expiration.clone(),
            implied_volatility,
            quantity,
            underlying_price,
            risk_free_rate,
            OptionStyle::Call,
            dividend_yield,
            None,
        );
        let short_call_high = Position::new(
            short_call_high_option,
            premium_short_call_high,
            Utc::now(),
            open_fee_short_high,
            close_fee_short_high,
        );
        strategy
            .add_position(&short_call_high.clone())
            .expect("Invalid long call otm");
        strategy.short_call_high = short_call_high;

        // Calculate break-even points
        strategy.break_even_points.push(
            strategy.long_call.option.strike_price
                - strategy.calculate_profit_at(strategy.long_call.option.strike_price) / quantity,
        );

        strategy.break_even_points.push(
            strategy.short_call_high.option.strike_price
                + strategy.calculate_profit_at(strategy.short_call_high.option.strike_price)
                    / quantity,
        );

        strategy
    }

    fn is_valid_short_option(&self, short_option: &OptionData, side: &FindOptimalSide) -> bool {
        match side {
            FindOptimalSide::Upper => short_option.strike_price >= self.underlying_price,
            FindOptimalSide::Lower => short_option.strike_price <= self.underlying_price,
            FindOptimalSide::All => true,
            FindOptimalSide::Range(start, end) => {
                short_option.strike_price >= *start && short_option.strike_price <= *end
            }
        }
    }

    fn are_valid_prices(
        &self,
        long_itm: &OptionData,
        long_otm: &OptionData,
        short_option: &OptionData,
    ) -> bool {
        if !long_itm.valid_call() || !long_otm.valid_call() || !short_option.valid_call() {
            return false;
        };
        long_itm.call_ask.unwrap() > PZERO
            && long_itm.call_ask.unwrap() > PZERO
            && short_option.call_bid.unwrap() > PZERO
    }

    fn create_strategy(
        &self,
        option_chain: &OptionChain,
        long_call: &OptionData,
        short_call_low: &OptionData,
        short_call_high: &OptionData,
    ) -> CallButterfly {
        if !long_call.validate() || !short_call_low.validate() || !short_call_high.validate() {
            panic!("Invalid options");
        }
        CallButterfly::new(
            option_chain.symbol.clone(),
            option_chain.underlying_price,
            long_call.strike_price,
            short_call_low.strike_price,
            short_call_high.strike_price,
            self.long_call.option.expiration_date.clone(),
            long_call.implied_volatility.unwrap().value(),
            self.long_call.option.risk_free_rate,
            self.long_call.option.dividend_yield,
            self.long_call.option.quantity,
            long_call.call_ask.unwrap().value(),
            short_call_low.call_bid.unwrap().value(),
            short_call_high.call_bid.unwrap().value(),
            self.long_call.open_fee,
            self.long_call.close_fee,
            self.short_call_low.open_fee,
            self.short_call_low.close_fee,
            self.short_call_high.open_fee,
            self.short_call_high.close_fee,
        )
    }
}

impl Default for CallButterfly {
    fn default() -> Self {
        CallButterfly::new(
            "".to_string(),
            PZERO,
            PZERO,
            PZERO,
            PZERO,
            ExpirationDate::Days(0.0),
            ZERO,
            ZERO,
            ZERO,
            pos!(1.0),
            ZERO,
            ZERO,
            ZERO,
            ZERO,
            ZERO,
            ZERO,
            ZERO,
            ZERO,
            ZERO,
        )
    }
}

impl Positionable for CallButterfly {
    fn add_position(&mut self, position: &Position) -> Result<(), String> {
        match position.option.side {
            Side::Short => {
                if position.option.strike_price >= self.long_call.option.strike_price {
                    self.short_call_high = position.clone();
                    Ok(())
                } else {
                    self.short_call_low = position.clone();
                    Ok(())
                }
            }
            Side::Long => {
                self.long_call = position.clone();
                Ok(())
            }
        }
    }

    fn get_positions(&self) -> Result<Vec<&Position>, String> {
        Ok(vec![
            &self.long_call,
            &self.short_call_low,
            &self.short_call_high,
        ])
    }
}

impl Strategies for CallButterfly {
    fn get_underlying_price(&self) -> PositiveF64 {
        self.underlying_price
    }

    fn break_even(&self) -> Vec<PositiveF64> {
        self.break_even_points.clone()
    }

    fn max_profit(&self) -> Result<PositiveF64, &str> {
        Ok(self
            .calculate_profit_at(self.long_call.option.strike_price)
            .abs()
            .into())
    }

    fn max_loss(&self) -> Result<PositiveF64, &str> {
        let lower_loss = self.calculate_profit_at(self.short_call_low.option.strike_price);
        let upper_loss = self.calculate_profit_at(self.short_call_high.option.strike_price);
        let result = match (lower_loss > ZERO, upper_loss > ZERO) {
            (true, true) => PZERO,
            (true, false) => upper_loss.abs().into(),
            (false, true) => lower_loss.abs().into(),
            (false, false) => lower_loss.abs().max(upper_loss.abs()).into(),
        };
        Ok(result)
    }

    fn total_cost(&self) -> PositiveF64 {
        pos!(
            self.short_call_low.net_cost() + self.short_call_high.net_cost()
                - self.long_call.net_cost()
        )
    }

    fn net_premium_received(&self) -> f64 {
        self.long_call.net_premium_received()
    }

    fn fees(&self) -> f64 {
        self.short_call_low.open_fee
            + self.short_call_low.close_fee
            + self.short_call_high.open_fee
            + self.short_call_high.close_fee
            + self.long_call.open_fee * self.long_call.option.quantity
            + self.long_call.close_fee * self.long_call.option.quantity
    }

    fn profit_area(&self) -> f64 {
        let range = self.long_call.option.strike_price - self.short_call_low.option.strike_price;
        let max_profit = self.max_profit().unwrap_or(PZERO);
        (range.value() * max_profit / 2.0) / self.underlying_price * 100.0
    }

    fn profit_ratio(&self) -> f64 {
        match (self.max_profit(), self.max_loss()) {
            (Ok(max_profit), Ok(max_loss)) => (max_profit / max_loss * 100.0).value(),
            _ => 0.0,
        }
    }

    fn get_break_even_points(&self) -> Vec<PositiveF64> {
        self.break_even_points.clone()
    }
}

impl Validable for CallButterfly {
    fn validate(&self) -> bool {
        if self.name.is_empty() {
            error!("Symbol is required");
            return false;
        }
        if !self.long_call.validate() {
            return false;
        }
        if !self.short_call_low.validate() {
            return false;
        }
        if !self.short_call_high.validate() {
            return false;
        }
        if self.underlying_price <= PZERO {
            error!("Underlying price must be greater than zero");
            return false;
        }
        if self.long_call.option.strike_price >= self.short_call_low.option.strike_price {
            error!("Long call strike price must be less than short call strike price");
            return false;
        }
        if self.short_call_low.option.strike_price >= self.short_call_high.option.strike_price {
            error!("Short call low strike price must be less than short call high strike price");
            return false;
        }
        true
    }
}

impl Optimizable for CallButterfly {
    type Strategy = CallButterfly;
    fn find_optimal(
        &mut self,
        option_chain: &OptionChain,
        side: FindOptimalSide,
        criteria: OptimizationCriteria,
    ) {
        let options: Vec<&OptionData> = option_chain.options.iter().collect();
        let mut best_value = f64::NEG_INFINITY;

        for short_index in 1..options.len() - 1 {
            let short_option = &options[short_index];
            if !self.is_valid_short_option(short_option, &side) {
                debug!("Skipping short option: {}", short_option.strike_price);
                continue;
            }

            for long_itm_index in 0..short_index {
                let long_otm_index = short_index + (short_index - long_itm_index);

                if long_otm_index >= options.len() {
                    continue;
                }

                let long_itm = &options[long_itm_index];
                let long_otm = &options[long_otm_index];

                if !self.are_valid_prices(long_itm, long_otm, short_option) {
                    continue;
                }

                let strategy = self.create_strategy(option_chain, long_itm, long_otm, short_option);

                if !strategy.validate() {
                    panic!("Invalid strategy");
                }

                let current_value = match criteria {
                    OptimizationCriteria::Ratio => strategy.profit_ratio(),
                    OptimizationCriteria::Area => strategy.profit_area(),
                };

                debug!(
                    "{}: {:.2}%",
                    if matches!(criteria, OptimizationCriteria::Ratio) {
                        "Ratio"
                    } else {
                        "Area"
                    },
                    current_value
                );

                if current_value > best_value {
                    best_value = current_value;
                    self.clone_from(&strategy);
                }
            }
        }
    }
}

impl Profit for CallButterfly {
    fn calculate_profit_at(&self, price: PositiveF64) -> f64 {
        let price = Some(price);
        let long_call_itm_profit = self.long_call.pnl_at_expiration(&price);
        let long_call_otm_profit = self.short_call_low.pnl_at_expiration(&price);
        let short_call_profit = self.short_call_high.pnl_at_expiration(&price);
        long_call_itm_profit + long_call_otm_profit + short_call_profit
    }
}

impl Graph for CallButterfly {
    fn title(&self) -> String {
        let strategy_title = format!("Ratio Call Spread Strategy: {:?}", self.kind);
        let long_call_itm_title = self.long_call.title();
        let long_call_otm_title = self.short_call_low.title();
        let short_call_title = self.short_call_high.title();

        format!(
            "{}\n\t{}\n\t{}\n\t{}",
            strategy_title, long_call_itm_title, long_call_otm_title, short_call_title
        )
    }

    fn get_vertical_lines(&self) -> Vec<ChartVerticalLine<f64, f64>> {
        let vertical_lines = vec![ChartVerticalLine {
            x_coordinate: self.long_call.option.underlying_price.value(),
            y_range: (f64::NEG_INFINITY, f64::INFINITY),
            label: format!(
                "Current Price: {:.2}",
                self.long_call.option.underlying_price
            ),
            label_offset: (-24.0, -1.0),
            line_color: ORANGE,
            label_color: ORANGE,
            line_style: ShapeStyle::from(&ORANGE).stroke_width(2),
            font_size: 18,
        }];

        vertical_lines
    }

    fn get_points(&self) -> Vec<ChartPoint<(f64, f64)>> {
        let mut points: Vec<ChartPoint<(f64, f64)>> = Vec::new();
        let max_profit = self.max_profit().unwrap_or(PZERO);

        points.push(ChartPoint {
            coordinates: (self.break_even_points[0].value(), 0.0),
            label: format!("Low Break Even\n\n{}", self.break_even_points[0]),
            label_offset: LabelOffsetType::Relative(-55.0, 5.0),
            point_color: DARK_BLUE,
            label_color: DARK_BLUE,
            point_size: 5,
            font_size: 18,
        });

        points.push(ChartPoint {
            coordinates: (self.break_even_points[1].value(), 0.0),
            label: format!("High Break Even\n\n{}", self.break_even_points[0]),
            label_offset: LabelOffsetType::Relative(3.0, 5.0),
            point_color: DARK_BLUE,
            label_color: DARK_BLUE,
            point_size: 5,
            font_size: 18,
        });

        points.push(ChartPoint {
            coordinates: (
                self.long_call.option.strike_price.value(),
                self.calculate_profit_at(self.long_call.option.strike_price),
            ),
            label: format!("Left Loss\n\n{:.2}", max_profit),
            label_offset: LabelOffsetType::Relative(3.0, 3.0),
            point_color: RED,
            label_color: RED,
            point_size: 5,
            font_size: 18,
        });

        let lower_loss = self.calculate_profit_at(self.short_call_low.option.strike_price);
        let upper_loss = self.calculate_profit_at(self.short_call_high.option.strike_price);

        points.push(ChartPoint {
            coordinates: (self.short_call_low.option.strike_price.value(), lower_loss),
            label: format!("Left High {:.2}", lower_loss),
            label_offset: LabelOffsetType::Relative(3.0, -3.0),
            point_color: DARK_GREEN,
            label_color: DARK_GREEN,
            point_size: 5,
            font_size: 18,
        });

        points.push(ChartPoint {
            coordinates: (self.short_call_high.option.strike_price.value(), upper_loss),
            label: format!("Right High {:.2}", upper_loss),
            label_offset: LabelOffsetType::Relative(3.0, 3.0),
            point_color: DARK_GREEN,
            label_color: DARK_GREEN,
            point_size: 5,
            font_size: 18,
        });

        points.push(self.get_point_at_price(self.underlying_price));

        points
    }
}

impl Greeks for CallButterfly {
    fn greeks(&self) -> Greek {
        let short_call_low_greek = self.short_call_low.greeks();
        let short_call_high_greek = self.short_call_high.greeks();
        let long_call_greek = self.long_call.greeks();

        Greek {
            delta: short_call_low_greek.delta + short_call_high_greek.delta + long_call_greek.delta,
            gamma: short_call_low_greek.gamma + short_call_high_greek.gamma + long_call_greek.gamma,
            theta: short_call_low_greek.theta + short_call_high_greek.theta + long_call_greek.theta,
            vega: short_call_low_greek.vega + short_call_high_greek.vega + long_call_greek.vega,
            rho: short_call_low_greek.rho + short_call_high_greek.rho + long_call_greek.rho,
            rho_d: short_call_low_greek.rho_d + short_call_high_greek.rho_d + long_call_greek.rho_d,
        }
    }
}

impl DeltaNeutrality for CallButterfly {
    fn calculate_net_delta(&self) -> DeltaInfo {
        let long_call_itm_delta = self.short_call_low.option.delta();
        let long_call_otm_delta = self.short_call_high.option.delta();
        let short_call_delta = self.long_call.option.delta();
        let threshold = DELTA_THRESHOLD;
        let delta = long_call_itm_delta + long_call_otm_delta + short_call_delta;
        DeltaInfo {
            net_delta: delta,
            individual_deltas: vec![long_call_itm_delta, long_call_otm_delta, short_call_delta],
            is_neutral: (delta).abs() < threshold,
            underlying_price: self.short_call_low.option.underlying_price,
            neutrality_threshold: threshold,
        }
    }

    fn get_atm_strike(&self) -> PositiveF64 {
        self.short_call_low.option.underlying_price
    }

    fn generate_delta_reducing_adjustments(&self) -> Vec<DeltaAdjustment> {
        let net_delta = self.calculate_net_delta().net_delta;
        vec![
            DeltaAdjustment::SellOptions {
                quantity: pos!((net_delta.abs() / self.short_call_low.option.delta()).abs())
                    * self.short_call_low.option.quantity,
                strike: self.short_call_low.option.strike_price,
                option_type: OptionStyle::Call,
            },
            DeltaAdjustment::SellOptions {
                quantity: pos!((net_delta.abs() / self.short_call_high.option.delta()).abs())
                    * self.short_call_high.option.quantity,
                strike: self.short_call_high.option.strike_price,
                option_type: OptionStyle::Call,
            },
        ]
    }

    fn generate_delta_increasing_adjustments(&self) -> Vec<DeltaAdjustment> {
        let net_delta = self.calculate_net_delta().net_delta;

        vec![DeltaAdjustment::BuyOptions {
            quantity: pos!((net_delta.abs() / self.long_call.option.delta()).abs())
                * self.long_call.option.quantity,
            strike: self.long_call.option.strike_price,
            option_type: OptionStyle::Call,
        }]
    }
}

#[cfg(test)]
mod tests_call_butterfly {
    use super::*;
    use crate::constants::ZERO;
    use approx::assert_relative_eq;

    fn setup() -> CallButterfly {
        CallButterfly::new(
            "AAPL".to_string(),
            pos!(150.0),
            pos!(155.0),
            pos!(160.0),
            pos!(157.5),
            ExpirationDate::Days(30.0),
            0.2,
            0.01,
            0.02,
            pos!(1.0),
            45.0,
            30.0,
            20.5,
            20.0,
            0.1,
            0.1,
            0.1,
            0.1,
            0.1,
        )
    }

    #[test]
    fn test_new() {
        let strategy = setup();
        assert_eq!(strategy.name, "AAPL");
        assert_eq!(strategy.kind, StrategyType::CallButterfly);
        assert!(strategy
            .description
            .contains("A Ratio Call Spread involves"));
    }

    #[test]
    fn test_break_even() {
        let strategy = setup();
        assert_eq!(strategy.break_even()[0], 170.0);
    }

    #[test]
    fn test_calculate_profit_at() {
        let strategy = setup();
        let price = 157.0;
        assert!(strategy.calculate_profit_at(pos!(price)) < ZERO);
    }

    #[test]
    fn test_max_profit() {
        let strategy = setup();
        assert!(strategy.max_profit().unwrap_or(PZERO) > PZERO);
    }

    #[test]
    fn test_net_premium_received() {
        let strategy = setup();
        assert_eq!(strategy.net_premium_received(), 0.0);
    }

    #[test]
    fn test_fees() {
        let strategy = setup();
        assert_relative_eq!(strategy.fees(), 20.5, epsilon = f64::EPSILON);
    }

    #[test]
    fn test_graph_methods() {
        let strategy = setup();

        let vertical_lines = strategy.get_vertical_lines();
        assert_eq!(vertical_lines.len(), 1);
        assert_eq!(vertical_lines[0].label, "Current Price: 150.00");

        let data = vec![
            pos!(150.0),
            pos!(155.0),
            pos!(160.0),
            pos!(165.0),
            pos!(170.0),
        ];
        let values = strategy.get_values(&data);
        for (i, &price) in data.iter().enumerate() {
            assert_eq!(values[i], strategy.calculate_profit_at(price));
        }

        let title = strategy.title();
        assert!(title.contains("Ratio Call Spread Strategy"));
        assert!(title.contains("Call"));
    }
}

#[cfg(test)]
mod tests_call_butterfly_validation {
    use super::*;

    fn setup_basic_strategy() -> CallButterfly {
        CallButterfly::new(
            "AAPL".to_string(),
            pos!(150.0),
            pos!(145.0),
            pos!(150.0),
            pos!(155.0),
            ExpirationDate::Days(30.0),
            0.2,
            0.01,
            0.02,
            pos!(1.0),
            7.0,
            5.0,
            3.0,
            4.0,
            0.1,
            0.1,
            0.1,
            0.1,
            0.1,
        )
    }

    #[test]
    fn test_validate_empty_symbol() {
        let mut strategy = setup_basic_strategy();
        strategy.name = "".to_string();
        assert!(!strategy.validate());
    }

    #[test]
    fn test_validate_invalid_underlying_price() {
        let mut strategy = setup_basic_strategy();
        strategy.underlying_price = PZERO;
        assert!(!strategy.validate());
    }

    #[test]
    fn test_validate_valid_strategy() {
        let strategy = setup_basic_strategy();
        assert!(strategy.validate());
    }
}

#[cfg(test)]
mod tests_call_butterfly_pnl {
    use super::*;

    fn setup_test_strategy() -> CallButterfly {
        CallButterfly::new(
            "AAPL".to_string(),
            pos!(150.0),
            pos!(145.0),
            pos!(150.0),
            pos!(155.0),
            ExpirationDate::Days(30.0),
            0.2,
            0.01,
            0.02,
            pos!(1.0),
            7.0,
            5.0,
            3.0,
            4.0,
            0.1,
            0.1,
            0.1,
            0.1,
            0.1,
        )
    }

    #[test]
    fn test_profit_below_lower_strike() {
        let strategy = setup_test_strategy();
        let profit = strategy.calculate_profit_at(pos!(140.0));
        assert!(profit <= 0.0);
    }

    #[test]
    fn test_profit_above_upper_strike() {
        let strategy = setup_test_strategy();
        let profit = strategy.calculate_profit_at(pos!(160.0));
        assert!(profit <= 0.0);
    }

    #[test]
    fn test_profit_ratio() {
        let strategy = setup_test_strategy();
        let ratio = strategy.profit_ratio();
        assert!(ratio > 0.0);
    }
}

#[cfg(test)]
mod tests_call_butterfly_graph {
    use super::*;
    use crate::model::types::ExpirationDate;
    use approx::assert_relative_eq;

    fn setup_test_strategy() -> CallButterfly {
        CallButterfly::new(
            "AAPL".to_string(),
            pos!(150.0),
            pos!(145.0),
            pos!(155.0),
            pos!(150.0),
            ExpirationDate::Days(30.0),
            0.2,
            0.01,
            0.02,
            pos!(1.0),
            7.0,
            5.0,
            3.0,
            4.0,
            0.1,
            0.1,
            0.1,
            0.1,
            0.1,
        )
    }

    #[test]
    fn test_vertical_lines() {
        let butterfly = setup_test_strategy();
        let lines = butterfly.get_vertical_lines();

        assert_eq!(lines.len(), 1, "Should have exactly one vertical line");

        let line = &lines[0];
        assert_relative_eq!(line.x_coordinate, 150.0, epsilon = 0.001);
        assert_eq!(line.y_range, (f64::NEG_INFINITY, f64::INFINITY));
        assert!(line.label.contains("Current Price: 150.00"));
        assert_eq!(line.font_size, 18);
    }

    #[test]
    fn test_points_count_and_labels() {
        let butterfly = setup_test_strategy();
        let points = butterfly.get_points();

        // Should have 6 points: 2 break-even, 1 left loss, 2 max profit points, and current price
        assert_eq!(points.len(), 6, "Should have exactly 6 points");

        let labels: Vec<&str> = points.iter().map(|p| p.label.as_str()).collect();

        // Verify all required labels are present
        assert!(labels.iter().any(|&l| l.contains("Low Break Even")));
        assert!(labels.iter().any(|&l| l.contains("High Break Even")));
        assert!(labels.iter().any(|&l| l.contains("Left Loss")));
        assert!(labels.iter().any(|&l| l.contains("Left High")));
        assert!(labels.iter().any(|&l| l.contains("Right High")));
    }

    #[test]
    fn test_points_coordinates() {
        let butterfly = setup_test_strategy();
        let points = butterfly.get_points();

        // Check for points at strike prices
        let strike_coordinates: Vec<f64> = points.iter().map(|p| p.coordinates.0).collect();

        assert!(
            strike_coordinates
                .iter()
                .any(|&x| (x - 145.0).abs() < 0.001),
            "Missing point near 145.0 strike"
        );
        assert!(
            strike_coordinates
                .iter()
                .any(|&x| (x - 150.0).abs() < 0.001),
            "Missing point near 150.0 strike"
        );
        assert!(
            strike_coordinates
                .iter()
                .any(|&x| (x - 155.0).abs() < 0.001),
            "Missing point near 155.0 strike"
        );
    }

    #[test]
    fn test_point_styling() {
        let butterfly = setup_test_strategy();
        let points = butterfly.get_points();

        for point in points {
            assert_eq!(point.point_size, 5, "Point size should be 5");
            assert_eq!(point.font_size, 18, "Font size should be 18");

            // Verify label offset is set
            match point.label_offset {
                LabelOffsetType::Relative(x, y) => {
                    assert!(x != 0.0, "X offset should not be 0");
                    assert!(y != 0.0, "Y offset should not be 0");
                }
                _ => panic!("Expected Relative label offset"),
            }
        }
    }

    #[test]
    fn test_break_even_points() {
        let butterfly = setup_test_strategy();
        let points = butterfly.get_points();

        // Find break-even points
        let break_even_points: Vec<&ChartPoint<(f64, f64)>> = points
            .iter()
            .filter(|p| p.label.contains("Break Even"))
            .collect();

        assert_eq!(
            break_even_points.len(),
            2,
            "Should have exactly 2 break-even points"
        );

        // Verify break-even points have y-coordinate = 0
        for point in break_even_points {
            assert_relative_eq!(point.coordinates.1, 0.0, epsilon = 0.001);
        }
    }
}

#[cfg(test)]
mod tests_iron_condor_delta {
    use crate::model::types::{ExpirationDate, OptionStyle, PositiveF64};
    use crate::pos;
    use crate::strategies::call_butterfly::CallButterfly;
    use crate::strategies::delta_neutral::DELTA_THRESHOLD;
    use crate::strategies::delta_neutral::{DeltaAdjustment, DeltaNeutrality};
    use approx::assert_relative_eq;

    fn get_strategy(underlying_price: PositiveF64) -> CallButterfly {
        CallButterfly::new(
            "SP500".to_string(),
            underlying_price, // underlying_price
            pos!(5750.0),     // long_strike_itm
            pos!(5850.0),     // long_strike_otm
            pos!(5800.0),     // short_strike
            ExpirationDate::Days(2.0),
            0.18,      // implied_volatility
            0.05,      // risk_free_rate
            0.0,       // dividend_yield
            pos!(1.0), // long quantity
            95.8,      // short_quantity
            85.04,     // premium_long_itm
            31.65,     // premium_long_otm
            53.04,     // premium_short
            0.78,      // open_fee_long
            0.78,      // close_fee_long
            0.73,      // close_fee_short
            0.73,      // close_fee_short
            0.73,      // close_fee_short
        )
    }

    #[test]
    fn create_test_reducing_adjustments() {
        let strategy = get_strategy(pos!(5901.88));

        assert_relative_eq!(
            strategy.calculate_net_delta().net_delta,
            -0.687410,
            epsilon = 0.0001
        );
        assert!(!strategy.is_delta_neutral());
        let suggestion = strategy.suggest_delta_adjustments();
        assert_eq!(
            suggestion[0],
            DeltaAdjustment::BuyOptions {
                quantity: pos!(0.7040502965074416),
                strike: pos!(5750.0),
                option_type: OptionStyle::Call
            }
        );

        let mut option = strategy.long_call.option.clone();
        option.quantity = pos!(0.7040502965074416);
        assert_relative_eq!(option.delta(), 0.687410, epsilon = 0.0001);
        assert_relative_eq!(
            option.delta() + strategy.calculate_net_delta().net_delta,
            0.0,
            epsilon = DELTA_THRESHOLD
        );
    }

    #[test]
    fn create_test_increasing_adjustments() {
        let strategy = get_strategy(pos!(5781.88));

        assert_relative_eq!(
            strategy.calculate_net_delta().net_delta,
            0.055904,
            epsilon = 0.0001
        );
        assert!(!strategy.is_delta_neutral());
        let suggestion = strategy.suggest_delta_adjustments();
        assert_eq!(
            suggestion[0],
            DeltaAdjustment::SellOptions {
                quantity: pos!(0.2835618144021529),
                strike: pos!(5850.0),
                option_type: OptionStyle::Call
            }
        );
        assert_eq!(
            suggestion[1],
            DeltaAdjustment::SellOptions {
                quantity: pos!(0.1338190182607821),
                strike: pos!(5800.0),
                option_type: OptionStyle::Call
            }
        );

        let mut option = strategy.short_call_low.option.clone();
        option.quantity = pos!(0.2835618144021529);
        assert_relative_eq!(option.delta(), -0.055904, epsilon = 0.0001);
        assert_relative_eq!(
            option.delta() + strategy.calculate_net_delta().net_delta,
            0.0,
            epsilon = DELTA_THRESHOLD
        );
    }

    #[test]
    fn create_test_no_adjustments() {
        let strategy = get_strategy(pos!(5795.0));

        assert_relative_eq!(
            strategy.calculate_net_delta().net_delta,
            0.0,
            epsilon = DELTA_THRESHOLD
        );
        assert!(strategy.is_delta_neutral());
        let suggestion = strategy.suggest_delta_adjustments();
        assert_eq!(suggestion[0], DeltaAdjustment::NoAdjustmentNeeded);
    }
}

#[cfg(test)]
mod tests_iron_condor_delta_size {
    use crate::model::types::{ExpirationDate, OptionStyle, PositiveF64};
    use crate::pos;
    use crate::strategies::call_butterfly::CallButterfly;
    use crate::strategies::delta_neutral::DELTA_THRESHOLD;
    use crate::strategies::delta_neutral::{DeltaAdjustment, DeltaNeutrality};
    use approx::assert_relative_eq;

    fn get_strategy(underlying_price: PositiveF64) -> CallButterfly {
        CallButterfly::new(
            "SP500".to_string(),
            underlying_price, // underlying_price
            pos!(5750.0),     // long_strike_itm
            pos!(5850.0),     // long_strike_otm
            pos!(5800.0),     // short_strike
            ExpirationDate::Days(2.0),
            0.18,      // implied_volatility
            0.05,      // risk_free_rate
            0.0,       // dividend_yield
            pos!(1.0), // long quantity
            97.8,      // short_quantity
            85.04,     // premium_long_itm
            31.65,     // premium_long_otm
            53.04,     // premium_short
            0.78,      // open_fee_long
            0.78,      // close_fee_long
            0.73,      // close_fee_short
            0.73,      // close_fee_short
            0.73,
        )
    }

    #[test]
    fn create_test_reducing_adjustments() {
        let strategy = get_strategy(pos!(5881.88));

        assert_relative_eq!(
            strategy.calculate_net_delta().net_delta,
            -0.5699325,
            epsilon = 0.0001
        );
        assert!(!strategy.is_delta_neutral());
        let suggestion = strategy.suggest_delta_adjustments();
        assert_eq!(
            suggestion[0],
            DeltaAdjustment::BuyOptions {
                quantity: pos!(0.5948524360242091),
                strike: pos!(5750.0),
                option_type: OptionStyle::Call
            }
        );

        let mut option = strategy.long_call.option.clone();
        option.quantity = pos!(0.5948524360242091);
        assert_relative_eq!(option.delta(), 0.56993, epsilon = 0.0001);
        assert_relative_eq!(
            option.delta() + strategy.calculate_net_delta().net_delta,
            0.0,
            epsilon = DELTA_THRESHOLD
        );
    }

    #[test]
    fn create_test_increasing_adjustments() {
        let strategy = get_strategy(pos!(5781.88));

        assert_relative_eq!(
            strategy.calculate_net_delta().net_delta,
            0.05590,
            epsilon = 0.0001
        );
        assert!(!strategy.is_delta_neutral());
        let suggestion = strategy.suggest_delta_adjustments();

        assert_eq!(
            suggestion[0],
            DeltaAdjustment::SellOptions {
                quantity: pos!(0.2835618144021529),
                strike: pos!(5850.0),
                option_type: OptionStyle::Call
            }
        );
        assert_eq!(
            suggestion[1],
            DeltaAdjustment::SellOptions {
                quantity: pos!(0.1338190182607821),
                strike: pos!(5800.0),
                option_type: OptionStyle::Call
            }
        );

        let mut option = strategy.short_call_low.option.clone();
        option.quantity = pos!(0.2835618144021529);
        assert_relative_eq!(option.delta(), -0.05590, epsilon = 0.0001);
        assert_relative_eq!(
            option.delta() + strategy.calculate_net_delta().net_delta,
            0.0,
            epsilon = DELTA_THRESHOLD
        );
    }

    #[test]
    fn create_test_no_adjustments() {
        let strategy = get_strategy(pos!(5795.0));

        assert_relative_eq!(
            strategy.calculate_net_delta().net_delta,
            0.0,
            epsilon = DELTA_THRESHOLD
        );
        assert!(strategy.is_delta_neutral());
        let suggestion = strategy.suggest_delta_adjustments();
        assert_eq!(suggestion[0], DeltaAdjustment::NoAdjustmentNeeded);
    }
}
