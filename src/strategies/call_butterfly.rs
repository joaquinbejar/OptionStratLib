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
    long_call_itm: Position,
    long_call_otm: Position,
    short_call: Position,
    underlying_price: PositiveF64,
}

impl CallButterfly {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        underlying_symbol: String,
        underlying_price: PositiveF64,
        long_strike_itm: PositiveF64,
        long_strike_otm: PositiveF64,
        short_strike: PositiveF64,
        expiration: ExpirationDate,
        implied_volatility: f64,
        risk_free_rate: f64,
        dividend_yield: f64,
        long_quantity: PositiveF64,
        short_quantity: PositiveF64,
        premium_long_itm: f64,
        premium_long_otm: f64,
        premium_short: f64,
        open_fee_long: f64,
        close_fee_long: f64,
        open_fee_short: f64,
        close_fee_short: f64,
    ) -> Self {
        let mut strategy = CallButterfly {
            name: underlying_symbol.to_string(),
            kind: StrategyType::CallButterfly,
            description: RATIO_CALL_SPREAD_DESCRIPTION.to_string(),
            break_even_points: Vec::new(),
            long_call_itm: Position::default(),
            long_call_otm: Position::default(),
            short_call: Position::default(),
            underlying_price,
        };
        let short_call_option = Options::new(
            OptionType::European,
            Side::Short,
            underlying_symbol.clone(),
            short_strike,
            expiration.clone(),
            implied_volatility,
            short_quantity,
            underlying_price,
            risk_free_rate,
            OptionStyle::Call,
            dividend_yield,
            None,
        );
        let short_call = Position::new(
            short_call_option,
            premium_short,
            Utc::now(),
            open_fee_short,
            close_fee_short,
        );
        strategy
            .add_position(&short_call.clone())
            .expect("Invalid short call");
        strategy.short_call = short_call;

        let long_call_itm_option = Options::new(
            OptionType::European,
            Side::Long,
            underlying_symbol.clone(),
            long_strike_itm,
            expiration.clone(),
            implied_volatility,
            long_quantity,
            underlying_price,
            risk_free_rate,
            OptionStyle::Call,
            dividend_yield,
            None,
        );
        let long_call_itm = Position::new(
            long_call_itm_option,
            premium_long_itm,
            Utc::now(),
            open_fee_long,
            close_fee_long,
        );
        strategy
            .add_position(&long_call_itm.clone())
            .expect("Invalid long call itm");
        strategy.long_call_itm = long_call_itm;

        let long_call_otm_option = Options::new(
            OptionType::European,
            Side::Long,
            underlying_symbol.clone(),
            long_strike_otm,
            expiration.clone(),
            implied_volatility,
            long_quantity,
            underlying_price,
            risk_free_rate,
            OptionStyle::Call,
            dividend_yield,
            None,
        );
        let long_call_otm = Position::new(
            long_call_otm_option,
            premium_long_otm,
            Utc::now(),
            open_fee_long,
            close_fee_long,
        );
        strategy
            .add_position(&long_call_otm.clone())
            .expect("Invalid long call otm");
        strategy.long_call_otm = long_call_otm;

        // Calculate break-even points
        let loss_at_itm_strike =
            strategy.calculate_profit_at(strategy.long_call_itm.option.strike_price);
        let loss_at_otm_strike =
            strategy.calculate_profit_at(strategy.long_call_otm.option.strike_price);

        let first_bep =
            strategy.long_call_itm.option.strike_price - (loss_at_itm_strike / long_quantity);
        strategy.break_even_points.push(first_bep);

        let second_bep =
            strategy.long_call_otm.option.strike_price + (loss_at_otm_strike / long_quantity);
        strategy.break_even_points.push(second_bep);

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
        long_itm: &OptionData,
        long_otm: &OptionData,
        short_option: &OptionData,
    ) -> CallButterfly {
        if !short_option.validate() || !long_itm.validate() || !long_otm.validate() {
            panic!("Invalid options");
        }
        CallButterfly::new(
            option_chain.symbol.clone(),
            option_chain.underlying_price,
            long_itm.strike_price,
            long_otm.strike_price,
            short_option.strike_price,
            self.short_call.option.expiration_date.clone(),
            short_option.implied_volatility.unwrap().value(),
            self.long_call_itm.option.risk_free_rate,
            self.long_call_itm.option.dividend_yield,
            self.long_call_itm.option.quantity,
            self.short_call.option.quantity,
            long_itm.call_ask.unwrap().value(),
            long_otm.call_ask.unwrap().value(),
            short_option.call_bid.unwrap().value(),
            self.long_call_itm.open_fee,
            self.long_call_itm.close_fee,
            self.short_call.open_fee,
            self.short_call.close_fee,
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
            0.0,
            0.0,
            0.0,
            pos!(1.0),
            pos!(2.0),
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
        )
    }
}

impl Positionable for CallButterfly {
    fn add_position(&mut self, position: &Position) -> Result<(), String> {
        match position.option.side {
            Side::Long => {
                if position.option.strike_price >= self.short_call.option.strike_price {
                    self.long_call_otm = position.clone();
                    Ok(())
                } else {
                    self.long_call_itm = position.clone();
                    Ok(())
                }
            }
            Side::Short => {
                self.short_call = position.clone();
                Ok(())
            }
        }
    }

    fn get_positions(&self) -> Result<Vec<&Position>, String> {
        Ok(vec![
            &self.long_call_itm,
            &self.long_call_otm,
            &self.short_call,
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
            .calculate_profit_at(self.short_call.option.strike_price)
            .abs()
            .into())
    }

    fn max_loss(&self) -> Result<PositiveF64, &str> {
        let lower_loss = self.calculate_profit_at(self.long_call_itm.option.strike_price);
        let upper_loss = self.calculate_profit_at(self.long_call_otm.option.strike_price);
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
            self.long_call_itm.net_cost() + self.long_call_otm.net_cost()
                - self.short_call.net_cost()
        )
    }

    fn net_premium_received(&self) -> f64 {
        self.short_call.net_premium_received()
    }

    fn fees(&self) -> f64 {
        self.long_call_itm.open_fee
            + self.long_call_itm.close_fee
            + self.long_call_otm.open_fee
            + self.long_call_otm.close_fee
            + self.short_call.open_fee * self.short_call.option.quantity
            + self.short_call.close_fee * self.short_call.option.quantity
    }

    fn profit_area(&self) -> f64 {
        let range = self.short_call.option.strike_price - self.long_call_itm.option.strike_price;
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
        if !self.long_call_itm.validate() {
            return false;
        }
        if !self.long_call_otm.validate() {
            return false;
        }
        if !self.short_call.validate() {
            return false;
        }
        if self.underlying_price <= PZERO {
            error!("Underlying price must be greater than zero");
            return false;
        }
        if self.short_call.option.quantity != self.long_call_itm.option.quantity * 2.0 {
            error!("Short call quantity must be twice the long call quantity and currently is short: {} and long: {}",
                self.short_call.option.quantity, self.long_call_itm.option.quantity);
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
        let long_call_itm_profit = self.long_call_itm.pnl_at_expiration(&price);
        let long_call_otm_profit = self.long_call_otm.pnl_at_expiration(&price);
        let short_call_profit = self.short_call.pnl_at_expiration(&price);
        long_call_itm_profit + long_call_otm_profit + short_call_profit
    }
}

impl Graph for CallButterfly {
    fn title(&self) -> String {
        let strategy_title = format!("Ratio Call Spread Strategy: {:?}", self.kind);
        let long_call_itm_title = self.long_call_itm.title();
        let long_call_otm_title = self.long_call_otm.title();
        let short_call_title = self.short_call.title();

        format!(
            "{}\n\t{}\n\t{}\n\t{}",
            strategy_title, long_call_itm_title, long_call_otm_title, short_call_title
        )
    }

    fn get_vertical_lines(&self) -> Vec<ChartVerticalLine<f64, f64>> {
        let vertical_lines = vec![ChartVerticalLine {
            x_coordinate: self.short_call.option.underlying_price.value(),
            y_range: (f64::NEG_INFINITY, f64::INFINITY),
            label: format!(
                "Current Price: {:.2}",
                self.short_call.option.underlying_price
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
            label_offset: LabelOffsetType::Relative(-26.0, 2.0),
            point_color: DARK_BLUE,
            label_color: DARK_BLUE,
            point_size: 5,
            font_size: 18,
        });

        points.push(ChartPoint {
            coordinates: (self.break_even_points[1].value(), 0.0),
            label: format!("High Break Even\n\n{}", self.break_even_points[1]),
            label_offset: LabelOffsetType::Relative(1.0, 2.0),
            point_color: DARK_BLUE,
            label_color: DARK_BLUE,
            point_size: 5,
            font_size: 18,
        });

        points.push(ChartPoint {
            coordinates: (
                self.short_call.option.strike_price.value(),
                max_profit.value(),
            ),
            label: format!("Max Profit\n\n{:.2}", max_profit),
            label_offset: LabelOffsetType::Relative(2.0, 1.0),
            point_color: DARK_GREEN,
            label_color: DARK_GREEN,
            point_size: 5,
            font_size: 18,
        });

        let lower_loss = self.calculate_profit_at(self.long_call_itm.option.strike_price);
        let upper_loss = self.calculate_profit_at(self.long_call_otm.option.strike_price);

        points.push(ChartPoint {
            coordinates: (self.long_call_itm.option.strike_price.value(), lower_loss),
            label: format!("Left Low {:.2}", lower_loss),
            label_offset: LabelOffsetType::Relative(0.0, -1.0),
            point_color: RED,
            label_color: RED,
            point_size: 5,
            font_size: 18,
        });

        points.push(ChartPoint {
            coordinates: (self.long_call_otm.option.strike_price.value(), upper_loss),
            label: format!("Right Low {:.2}", upper_loss),
            label_offset: LabelOffsetType::Relative(-18.0, -1.0),
            point_color: RED,
            label_color: RED,
            point_size: 5,
            font_size: 18,
        });

        points.push(self.get_point_at_price(self.underlying_price));

        points
    }
}

impl Greeks for CallButterfly {
    fn greeks(&self) -> Greek {
        let long_call_itm_greek = self.long_call_itm.greeks();
        let long_call_otm_greek = self.long_call_otm.greeks();
        let short_call_greek = self.short_call.greeks();

        Greek {
            delta: long_call_itm_greek.delta + long_call_otm_greek.delta + short_call_greek.delta,
            gamma: long_call_itm_greek.gamma + long_call_otm_greek.gamma + short_call_greek.gamma,
            theta: long_call_itm_greek.theta + long_call_otm_greek.theta + short_call_greek.theta,
            vega: long_call_itm_greek.vega + long_call_otm_greek.vega + short_call_greek.vega,
            rho: long_call_itm_greek.rho + long_call_otm_greek.rho + short_call_greek.rho,
            rho_d: long_call_itm_greek.rho_d + long_call_otm_greek.rho_d + short_call_greek.rho_d,
        }
    }
}

impl DeltaNeutrality for CallButterfly {
    fn calculate_net_delta(&self) -> DeltaInfo {
        let long_call_itm_delta = self.long_call_itm.option.delta();
        let long_call_otm_delta = self.long_call_otm.option.delta();
        let short_call_delta = self.short_call.option.delta();
        let threshold = DELTA_THRESHOLD;
        let delta = long_call_itm_delta + long_call_otm_delta + short_call_delta;
        DeltaInfo {
            net_delta: delta,
            individual_deltas: vec![long_call_itm_delta, long_call_otm_delta, short_call_delta],
            is_neutral: (delta).abs() < threshold,
            underlying_price: self.long_call_itm.option.underlying_price,
            neutrality_threshold: threshold,
        }
    }

    fn get_atm_strike(&self) -> PositiveF64 {
        self.long_call_itm.option.underlying_price
    }

    fn generate_delta_reducing_adjustments(&self) -> Vec<DeltaAdjustment> {
        let net_delta = self.calculate_net_delta().net_delta;
        vec![DeltaAdjustment::SellOptions {
            quantity: pos!((net_delta.abs() / self.short_call.option.delta()).abs())
                * self.short_call.option.quantity,
            strike: self.short_call.option.strike_price,
            option_type: OptionStyle::Call,
        }]
    }

    fn generate_delta_increasing_adjustments(&self) -> Vec<DeltaAdjustment> {
        let net_delta = self.calculate_net_delta().net_delta;

        vec![
            DeltaAdjustment::BuyOptions {
                quantity: pos!((net_delta.abs() / self.long_call_itm.option.delta()).abs())
                    * self.long_call_itm.option.quantity,
                strike: self.long_call_itm.option.strike_price,
                option_type: OptionStyle::Call,
            },
            DeltaAdjustment::BuyOptions {
                quantity: pos!((net_delta.abs() / self.long_call_otm.option.delta()).abs())
                    * self.long_call_otm.option.quantity,
                strike: self.long_call_otm.option.strike_price,
                option_type: OptionStyle::Call,
            },
        ]
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
            pos!(2.0),
            30.0,
            20.5,
            20.0,
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
        assert_eq!(strategy.break_even()[0], 166.3);
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
    fn test_max_loss() {
        let strategy = setup();
        assert_eq!(strategy.max_loss().unwrap_or(PZERO), strategy.total_cost());
    }

    #[test]
    fn test_total_cost() {
        let strategy = setup();
        assert!(strategy.total_cost() > PZERO);
    }

    #[test]
    fn test_net_premium_received() {
        let strategy = setup();
        assert_eq!(strategy.net_premium_received(), 39.6);
    }

    #[test]
    fn test_fees() {
        let strategy = setup();
        assert_relative_eq!(strategy.fees(), 0.8, epsilon = f64::EPSILON);
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
            pos!(155.0),
            pos!(150.0),
            ExpirationDate::Days(30.0),
            0.2,
            0.01,
            0.02,
            pos!(1.0),
            pos!(2.0),
            5.0,
            3.0,
            4.0,
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
    fn test_validate_invalid_quantity_ratio() {
        let mut strategy = setup_basic_strategy();
        strategy.short_call.option.quantity = pos!(1.0); // Should be 2x long quantity
        assert!(!strategy.validate());
    }

    #[test]
    fn test_validate_valid_strategy() {
        let strategy = setup_basic_strategy();
        assert!(strategy.validate());
    }
}

#[cfg(test)]
mod tests_call_butterfly_optimization {
    use super::*;
    use crate::spos;

    fn create_test_option_chain() -> OptionChain {
        let mut chain = OptionChain::new("AAPL", pos!(150.0), "2024-01-01".to_string());

        // Add options at various strikes
        for strike in [145.0, 147.5, 150.0, 152.5, 155.0].iter() {
            chain.add_option(
                pos!(*strike),
                spos!(3.0),
                spos!(3.2),
                spos!(2.8),
                spos!(3.0),
                spos!(0.2),
                Some(0.5),
                spos!(100.0),
                Some(50),
            );
        }
        chain
    }

    #[test]
    fn test_is_valid_short_option_upper() {
        let strategy = CallButterfly::default();
        let option = OptionData::new(
            pos!(155.0),
            spos!(3.0),
            spos!(3.2),
            spos!(2.8),
            spos!(3.0),
            spos!(0.2),
            None,
            None,
            None,
        );
        assert!(strategy.is_valid_short_option(&option, &FindOptimalSide::Upper));
    }

    #[test]
    fn test_are_valid_prices() {
        let strategy = CallButterfly::default();
        let long_itm = OptionData::new(
            pos!(145.0),
            spos!(3.0),
            spos!(3.2),
            spos!(2.8),
            spos!(3.0),
            spos!(0.2),
            None,
            None,
            None,
        );
        let long_otm = OptionData::new(
            pos!(155.0),
            spos!(3.0),
            spos!(3.2),
            spos!(2.8),
            spos!(3.0),
            spos!(0.2),
            None,
            None,
            None,
        );
        let short = OptionData::new(
            pos!(150.0),
            spos!(3.0),
            spos!(3.2),
            spos!(2.8),
            spos!(3.0),
            spos!(0.2),
            None,
            None,
            None,
        );
        assert!(strategy.are_valid_prices(&long_itm, &long_otm, &short));
    }

    #[test]
    fn test_find_optimal_ratio() {
        let mut strategy = CallButterfly::default();
        let chain = create_test_option_chain();
        strategy.find_optimal(&chain, FindOptimalSide::All, OptimizationCriteria::Ratio);
        assert!(strategy.validate());
    }

    #[test]
    fn test_find_optimal_area() {
        let mut strategy = CallButterfly::default();
        let chain = create_test_option_chain();
        strategy.find_optimal(&chain, FindOptimalSide::All, OptimizationCriteria::Area);
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
            pos!(155.0),
            pos!(150.0),
            ExpirationDate::Days(30.0),
            0.2,
            0.01,
            0.02,
            pos!(1.0),
            pos!(2.0),
            5.0,
            3.0,
            4.0,
            0.1,
            0.1,
            0.1,
            0.1,
        )
    }

    #[test]
    fn test_profit_at_max_point() {
        let strategy = setup_test_strategy();
        let profit = strategy.calculate_profit_at(strategy.short_call.option.strike_price);
        assert_eq!(profit, strategy.max_profit().unwrap_or(PZERO).value());
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

    #[test]
    fn test_profit_area() {
        let strategy = setup_test_strategy();
        let area = strategy.profit_area();
        assert!(area > 0.0);
    }
}

#[cfg(test)]
mod tests_call_butterfly_graph {
    use super::*;

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
            pos!(2.0),
            5.0,
            3.0,
            4.0,
            0.1,
            0.1,
            0.1,
            0.1,
        )
    }

    #[test]
    fn test_get_points() {
        let strategy = setup_test_strategy();
        let points = strategy.get_points();
        assert!(!points.is_empty());

        let has_break_even = points.iter().any(|p| p.label.contains("Break Even"));
        let has_max_profit = points.iter().any(|p| p.label.contains("Max Profit"));
        let has_low_point = points.iter().any(|p| p.label.contains("Low"));

        assert!(has_break_even);
        assert!(has_max_profit);
        assert!(has_low_point);
    }

    #[test]
    fn test_get_vertical_lines() {
        let strategy = setup_test_strategy();
        let lines = strategy.get_vertical_lines();
        assert_eq!(lines.len(), 1);
        assert!(lines[0].label.contains("Current Price"));
    }

    #[test]
    fn test_best_range_to_show() {
        let strategy = setup_test_strategy();
        let range = strategy.best_range_to_show(pos!(5.0)).unwrap();
        assert!(!range.is_empty());
        assert!(range[0] < strategy.long_call_itm.option.strike_price);
        assert!(range[range.len() - 1] > strategy.long_call_otm.option.strike_price);
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
            pos!(2.0), // short_quantity
            85.04,     // premium_long_itm
            31.65,     // premium_long_otm
            53.04,     // premium_short
            0.78,      // open_fee_long
            0.78,      // close_fee_long
            0.73,      // close_fee_short
            0.73,      // close_fee_short
        )
    }

    #[test]
    fn create_test_short_strangle_reducing_adjustments() {
        let strategy = get_strategy(pos!(5901.88));

        assert_relative_eq!(
            strategy.calculate_net_delta().net_delta,
            -0.086598,
            epsilon = 0.0001
        );
        assert!(!strategy.is_delta_neutral());
        let suggestion = strategy.suggest_delta_adjustments();
        assert_eq!(
            suggestion[0],
            DeltaAdjustment::BuyOptions {
                quantity: pos!(0.08869424268674732),
                strike: pos!(5750.0),
                option_type: OptionStyle::Call
            }
        );
        assert_eq!(
            suggestion[1],
            DeltaAdjustment::BuyOptions {
                quantity: pos!(0.11472018606079874),
                strike: pos!(5850.0),
                option_type: OptionStyle::Call
            }
        );

        let mut option = strategy.long_call_itm.option.clone();
        option.quantity = pos!(0.08869424268674732);
        assert_relative_eq!(option.delta(), 0.086598, epsilon = 0.0001);
        assert_relative_eq!(
            option.delta() + strategy.calculate_net_delta().net_delta,
            0.0,
            epsilon = DELTA_THRESHOLD
        );
    }

    #[test]
    fn create_test_short_strangle_increasing_adjustments() {
        let strategy = get_strategy(pos!(5781.88));

        assert_relative_eq!(
            strategy.calculate_net_delta().net_delta,
            0.032444,
            epsilon = 0.0001
        );
        assert!(!strategy.is_delta_neutral());
        let suggestion = strategy.suggest_delta_adjustments();
        assert_eq!(
            suggestion[0],
            DeltaAdjustment::SellOptions {
                quantity: pos!(0.07766273391000812),
                strike: pos!(5800.0),
                option_type: OptionStyle::Call
            }
        );

        let mut option = strategy.short_call.option.clone();
        option.quantity = pos!(0.07766273391000812);
        assert_relative_eq!(option.delta(), -0.032444, epsilon = 0.0001);
        assert_relative_eq!(
            option.delta() + strategy.calculate_net_delta().net_delta,
            0.0,
            epsilon = DELTA_THRESHOLD
        );
    }

    #[test]
    fn create_test_short_strangle_no_adjustments() {
        let strategy = get_strategy(pos!(5800.0));

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
            pos!(2.0), // short_quantity
            85.04,     // premium_long_itm
            31.65,     // premium_long_otm
            53.04,     // premium_short
            0.78,      // open_fee_long
            0.78,      // close_fee_long
            0.73,      // close_fee_short
            0.73,      // close_fee_short
        )
    }

    #[test]
    fn create_test_short_strangle_reducing_adjustments() {
        let strategy = get_strategy(pos!(5881.88));

        assert_relative_eq!(
            strategy.calculate_net_delta().net_delta,
            -0.0931943,
            epsilon = 0.0001
        );
        assert!(!strategy.is_delta_neutral());
        let suggestion = strategy.suggest_delta_adjustments();
        assert_eq!(
            suggestion[0],
            DeltaAdjustment::BuyOptions {
                quantity: pos!(0.09726918791103065),
                strike: pos!(5750.0),
                option_type: OptionStyle::Call
            }
        );
        assert_eq!(
            suggestion[1],
            DeltaAdjustment::BuyOptions {
                quantity: pos!(0.13945831929041605),
                strike: pos!(5850.0),
                option_type: OptionStyle::Call
            }
        );

        let mut option = strategy.long_call_otm.option.clone();
        option.quantity = pos!(0.13945831929041605);
        assert_relative_eq!(option.delta(), 0.09319, epsilon = 0.0001);
        assert_relative_eq!(
            option.delta() + strategy.calculate_net_delta().net_delta,
            0.0,
            epsilon = DELTA_THRESHOLD
        );
    }

    #[test]
    fn create_test_short_strangle_increasing_adjustments() {
        let strategy = get_strategy(pos!(5781.88));

        assert_relative_eq!(
            strategy.calculate_net_delta().net_delta,
            0.03244,
            epsilon = 0.0001
        );
        assert!(!strategy.is_delta_neutral());
        let suggestion = strategy.suggest_delta_adjustments();

        assert_eq!(
            suggestion[0],
            DeltaAdjustment::SellOptions {
                quantity: pos!(0.07766273391000812),
                strike: pos!(5800.0),
                option_type: OptionStyle::Call
            }
        );

        let mut option = strategy.short_call.option.clone();
        option.quantity = pos!(0.07766273391000812);
        assert_relative_eq!(option.delta(), -0.032444, epsilon = 0.0001);
        assert_relative_eq!(
            option.delta() + strategy.calculate_net_delta().net_delta,
            0.0,
            epsilon = DELTA_THRESHOLD
        );
    }

    #[test]
    fn create_test_short_strangle_no_adjustments() {
        let strategy = get_strategy(pos!(5800.0));

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
