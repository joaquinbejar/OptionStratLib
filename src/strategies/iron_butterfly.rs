/*
Iron Butterfly Strategy

An iron butterfly involves selling both a put and call at the same strike price (at-the-money)
and buying a put at a lower strike and a call at a higher strike.
This strategy is used when very low volatility in the underlying asset's price is expected.

Key characteristics:
- Maximum profit at the short strike price
- Limited risk
- High probability of small profit
- Requires very low volatility
*/
use super::base::{Optimizable, Positionable, Strategies, StrategyType, Validable};
use crate::chains::chain::OptionChain;
use crate::chains::utils::OptionDataGroup;
use crate::chains::StrategyLegs;
use crate::constants::{DARK_BLUE, DARK_GREEN, ZERO};
use crate::error::position::PositionError;
use crate::error::strategies::{ProfitLossErrorKind, StrategyError};
use crate::greeks::equations::{Greek, Greeks};
use crate::model::position::Position;
use crate::model::types::{ExpirationDate, OptionStyle, OptionType, Side};
use crate::pricing::payoff::Profit;
use crate::strategies::delta_neutral::{
    DeltaAdjustment, DeltaInfo, DeltaNeutrality, DELTA_THRESHOLD,
};
use crate::strategies::utils::{FindOptimalSide, OptimizationCriteria};
use crate::visualization::model::{ChartPoint, ChartVerticalLine, LabelOffsetType};
use crate::visualization::utils::Graph;
use crate::Options;
use crate::{d2fu, f2p, Positive};
use chrono::Utc;
use num_traits::{FromPrimitive, ToPrimitive};
use plotters::prelude::full_palette::ORANGE;
use plotters::prelude::{ShapeStyle, RED};
use rust_decimal::Decimal;
use tracing::{error, info};

const IRON_BUTTERFLY_DESCRIPTION: &str =
    "An Iron Butterfly is a neutral options strategy combining selling an at-the-money put and call \
    while buying an out-of-the-money call and an out-of-the-money put. The short options have the same \
    strike price. This strategy profits from low volatility and time decay, with maximum profit when \
    the underlying price equals the strike price of the short options at expiration.";

#[derive(Clone, Debug)]
pub struct IronButterfly {
    pub name: String,
    pub kind: StrategyType,
    pub description: String,
    pub break_even_points: Vec<Positive>,
    short_call: Position,
    short_put: Position,
    long_call: Position,
    long_put: Position,
}

impl IronButterfly {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        underlying_symbol: String,
        underlying_price: Positive,
        short_strike: Positive,
        long_call_strike: Positive,
        long_put_strike: Positive,
        expiration: ExpirationDate,
        implied_volatility: f64,
        risk_free_rate: f64,
        dividend_yield: f64,
        quantity: Positive,
        premium_short_call: f64,
        premium_short_put: f64,
        premium_long_call: f64,
        premium_long_put: f64,
        open_fee: f64,
        close_fee: f64,
    ) -> Self {
        let mut strategy = IronButterfly {
            name: "Iron Butterfly".to_string(),
            kind: StrategyType::IronButterfly,
            description: IRON_BUTTERFLY_DESCRIPTION.to_string(),
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
            short_strike,
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
        strategy
            .add_position(&short_call.clone())
            .expect("Invalid short call");

        // Short Put
        let short_put_option = Options::new(
            OptionType::European,
            Side::Short,
            underlying_symbol.clone(),
            short_strike,
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
        strategy
            .add_position(&short_put.clone())
            .expect("Invalid short put");

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
        strategy
            .add_position(&long_call.clone())
            .expect("Invalid long call");

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
        strategy
            .add_position(&long_put.clone())
            .expect("Invalid long put");

        // Calculate break-even points
        let net_credit = (strategy.long_put.premium + strategy.long_call.premium)
            + strategy.fees().unwrap().to_f64().unwrap()
            - (strategy.short_put.premium + strategy.short_call.premium);
        strategy.break_even_points.push(short_strike + net_credit);
        strategy.break_even_points.push(short_strike - net_credit);

        strategy
    }
}

impl Validable for IronButterfly {
    fn validate(&self) -> bool {
        let order = self.long_put.option.strike_price < self.short_put.option.strike_price
            && self.short_put.option.strike_price == self.short_call.option.strike_price
            && self.short_call.option.strike_price < self.long_call.option.strike_price;

        if !order {
            error!("Invalid order of strikes or short strikes not equal");
        }

        self.short_call.validate()
            && self.short_put.validate()
            && self.long_call.validate()
            && self.long_put.validate()
            && order
    }
}

impl Positionable for IronButterfly {
    fn add_position(&mut self, position: &Position) -> Result<(), PositionError> {
        match (
            position.option.option_style.clone(),
            position.option.side.clone(),
        ) {
            (OptionStyle::Call, Side::Short) => {
                self.short_call = position.clone();
                Ok(())
            }
            (OptionStyle::Put, Side::Short) => {
                self.short_put = position.clone();
                Ok(())
            }
            (OptionStyle::Call, Side::Long) => {
                self.long_call = position.clone();
                Ok(())
            }
            (OptionStyle::Put, Side::Long) => {
                self.long_put = position.clone();
                Ok(())
            }
        }
    }

    fn get_positions(&self) -> Result<Vec<&Position>, PositionError> {
        Ok(vec![
            &self.short_call,
            &self.short_put,
            &self.long_call,
            &self.long_put,
        ])
    }
}

impl Strategies for IronButterfly {
    fn get_underlying_price(&self) -> Positive {
        self.long_put.option.underlying_price
    }

    fn max_profit(&self) -> Result<Positive, StrategyError> {
        let left_profit = self.calculate_profit_at(self.short_call.option.strike_price);
        let right_profit = self.calculate_profit_at(self.short_put.option.strike_price);
        if left_profit < ZERO || right_profit < ZERO {
            return Err(StrategyError::ProfitLossError(
                ProfitLossErrorKind::MaxProfitError {
                    reason: "Max profit is negative".to_string(),
                },
            ));
        }

        Ok(f2p!(
            self.calculate_profit_at(self.short_call.option.strike_price)
        ))
    }

    fn max_loss(&self) -> Result<Positive, StrategyError> {
        let left_loss = self.calculate_profit_at(self.long_put.option.strike_price);
        let right_loss = self.calculate_profit_at(self.long_call.option.strike_price);
        if left_loss > ZERO || right_loss > ZERO {
            return Err(StrategyError::ProfitLossError(
                ProfitLossErrorKind::MaxLossError {
                    reason: "Max loss is negative".to_string(),
                },
            ));
        }
        Ok(f2p!(left_loss.abs().max(right_loss.abs())))
    }

    fn total_cost(&self) -> Positive {
        f2p!(
            self.short_call.net_cost()
                + self.short_put.net_cost()
                + self.long_call.net_cost()
                + self.long_put.net_cost()
        )
    }

    fn net_premium_received(&self) -> Result<Decimal, StrategyError> {
        let net_prem = self.short_call.net_premium_received()
            + self.short_put.net_premium_received()
            - self.long_call.total_cost()
            - self.long_put.total_cost();
        Ok(Decimal::from_f64(net_prem).unwrap())
    }

    fn fees(&self) -> Result<Decimal, StrategyError> {
        let result = self.short_call.open_fee
            + self.short_call.close_fee
            + self.short_put.open_fee
            + self.short_put.close_fee
            + self.long_call.open_fee
            + self.long_call.close_fee
            + self.long_put.open_fee
            + self.long_put.close_fee;
        Ok(Decimal::from_f64(result).unwrap())
    }

    fn profit_area(&self) -> Result<Decimal, StrategyError> {
        let inner_width =
            (self.short_call.option.strike_price - self.short_put.option.strike_price).to_f64();
        let outer_width =
            (self.long_call.option.strike_price - self.long_put.option.strike_price).to_f64();
        let height = self.max_profit().unwrap_or(Positive::ZERO);

        let inner_area = inner_width * height;
        let outer_triangles = (outer_width - inner_width) * height / 2.0;

        let result =
            (inner_area + outer_triangles) / self.short_call.option.underlying_price.to_f64();
        Ok(Decimal::from_f64(result).unwrap())
    }

    fn profit_ratio(&self) -> Result<Decimal, StrategyError> {
        let max_profit = self.max_profit().unwrap_or(Positive::ZERO);
        let max_loss = self.max_loss().unwrap_or(Positive::ZERO);
        match (max_profit, max_loss) {
            (value, _) if value == Positive::ZERO => Ok(Decimal::ZERO),
            (_, value) if value == Positive::ZERO => Ok(Decimal::MAX),
            _ => Ok((max_profit / max_loss * 100.0).into()),
        }
    }

    fn get_break_even_points(&self) -> Result<&Vec<Positive>, StrategyError> {
        Ok(&self.break_even_points)
    }
}

impl Optimizable for IronButterfly {
    type Strategy = IronButterfly;

    fn filter_combinations<'a>(
        &'a self,
        option_chain: &'a OptionChain,
        side: FindOptimalSide,
    ) -> impl Iterator<Item = OptionDataGroup<'a>> {
        let underlying_price = self.get_underlying_price();
        let strategy = self.clone();
        option_chain
            .get_triple_iter()
            // Filter out invalid combinations based on FindOptimalSide
            .filter(move |(low, mid, high)| {
                low.is_valid_optimal_side(underlying_price, &side)
                    && mid.is_valid_optimal_side(underlying_price, &side)
                    && high.is_valid_optimal_side(underlying_price, &side)
            })
            // Filter out options with invalid bid/ask prices
            .filter(|(low, mid, high)| {
                low.put_ask.unwrap_or(Positive::ZERO) > Positive::ZERO
                    && mid.put_bid.unwrap_or(Positive::ZERO) > Positive::ZERO
                    && high.call_ask.unwrap_or(Positive::ZERO) > Positive::ZERO
            })
            // Filter out options that don't meet strategy constraints
            .filter(move |(low, mid, high)| {
                let legs = StrategyLegs::FourLegs {
                    first: low,
                    second: mid,
                    third: mid,
                    fourth: high,
                };
                let strategy = strategy.create_strategy(option_chain, &legs);
                strategy.validate() && strategy.max_profit().is_ok() && strategy.max_loss().is_ok()
            })
            // Map to OptionDataGroup
            .map(move |(low, mid, high)| OptionDataGroup::Three(low, mid, high))
    }

    fn find_optimal(
        &mut self,
        option_chain: &OptionChain,
        side: FindOptimalSide,
        criteria: OptimizationCriteria,
    ) {
        let mut best_value = Decimal::MIN;
        let strategy_clone = self.clone();
        let options_iter = strategy_clone.filter_combinations(option_chain, side);

        for option_data_group in options_iter {
            // Unpack the OptionDataGroup into individual options
            let (low, mid, high) = match option_data_group {
                OptionDataGroup::Three(first, second, third) => (first, second, third),
                _ => panic!("Invalid OptionDataGroup"),
            };

            let legs = StrategyLegs::FourLegs {
                first: low,
                second: mid,
                third: mid,
                fourth: high,
            };
            let strategy = self.create_strategy(option_chain, &legs);
            // Calculate the current value based on the optimization criteria
            let current_value = match criteria {
                OptimizationCriteria::Ratio => strategy.profit_ratio().unwrap(),
                OptimizationCriteria::Area => strategy.profit_area().unwrap(),
            };

            if current_value > best_value {
                // Update the best value and replace the current strategy
                info!("Found better value: {}", current_value);
                best_value = current_value;
                *self = strategy.clone();
            }
        }
    }

    fn create_strategy(&self, chain: &OptionChain, legs: &StrategyLegs) -> Self::Strategy {
        match legs {
            StrategyLegs::FourLegs {
                first: long_put,
                second: short_strike,
                third: _,
                fourth: long_call,
            } => IronButterfly::new(
                chain.symbol.clone(),
                chain.underlying_price,
                short_strike.strike_price,
                long_call.strike_price,
                long_put.strike_price,
                self.short_call.option.expiration_date.clone(),
                short_strike.implied_volatility.unwrap().to_f64() / 100.0,
                self.short_call.option.risk_free_rate,
                self.short_call.option.dividend_yield,
                self.short_call.option.quantity,
                short_strike.call_bid.unwrap().to_f64(),
                short_strike.put_bid.unwrap().to_f64(),
                long_call.call_ask.unwrap().to_f64(),
                long_put.put_ask.unwrap().to_f64(),
                self.fees().unwrap().to_f64().unwrap() / 8.0,
                self.fees().unwrap().to_f64().unwrap() / 8.0,
            ),
            _ => panic!("Invalid number of legs for Iron Butterfly strategy"),
        }
    }
}

impl Profit for IronButterfly {
    fn calculate_profit_at(&self, price: Positive) -> f64 {
        let price = Some(price);
        self.short_call.pnl_at_expiration(&price)
            + self.short_put.pnl_at_expiration(&price)
            + self.long_call.pnl_at_expiration(&price)
            + self.long_put.pnl_at_expiration(&price)
    }
}

impl Graph for IronButterfly {
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
            format!(
                "Expire: {}",
                self.short_put.option.expiration_date.get_date_string()
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
            x_coordinate: self.short_call.option.underlying_price.to_f64(),
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
        let max_profit = self.max_profit().unwrap_or(Positive::ZERO);
        let short_call_strike_price = &self.short_call.option.strike_price;
        let short_put_strike_price = &self.short_put.option.strike_price;
        let long_call_strike_price = &self.long_call.option.strike_price;
        let long_put_strike_price = &self.long_put.option.strike_price;
        let current_price = &self.short_call.option.underlying_price;

        points.push(ChartPoint {
            coordinates: (self.break_even_points[0].to_f64(), 0.0),
            label: format!("Left Break Even\n\n{}", self.break_even_points[0]),
            label_offset: LabelOffsetType::Relative(5.0, 5.0),
            point_color: DARK_BLUE,
            label_color: DARK_BLUE,
            point_size: 5,
            font_size: 18,
        });

        points.push(ChartPoint {
            coordinates: (self.break_even_points[1].to_f64(), 0.0),
            label: format!("Right Break Even\n\n{}", self.break_even_points[1]),
            label_offset: LabelOffsetType::Relative(5.0, 5.0),
            point_color: DARK_BLUE,
            label_color: DARK_BLUE,
            point_size: 5,
            font_size: 18,
        });

        let coordiantes: (f64, f64) = (
            short_call_strike_price.to_f64() / 2000.0,
            max_profit.to_f64() / 5.0,
        );
        points.push(ChartPoint {
            coordinates: (short_call_strike_price.to_f64(), max_profit.to_f64()),
            label: format!(
                "Max Profit {:.2} at {:.0}",
                max_profit, short_call_strike_price
            ),
            label_offset: LabelOffsetType::Relative(coordiantes.0, coordiantes.1),
            point_color: DARK_GREEN,
            label_color: DARK_GREEN,
            point_size: 5,
            font_size: 18,
        });

        let loss = self.calculate_profit_at(*long_call_strike_price);
        let coordinates: (f64, f64) = (-short_put_strike_price.to_f64() / 35.0, loss / 50.0);
        points.push(ChartPoint {
            coordinates: (self.long_call.option.strike_price.to_f64(), loss),
            label: format!(
                "Right Max Loss {:.2} at {:.0}",
                loss, self.long_call.option.strike_price
            ),
            label_offset: LabelOffsetType::Relative(coordinates.0, coordinates.1),
            point_color: RED,
            label_color: RED,
            point_size: 5,
            font_size: 18,
        });

        let loss = self.calculate_profit_at(*long_put_strike_price);

        let coordinates: (f64, f64) = (long_put_strike_price.to_f64() / 2000.0, loss / 50.0);
        points.push(ChartPoint {
            coordinates: (long_put_strike_price.to_f64(), loss),
            label: format!("Left Max Loss {:.2} at {:.0}", loss, long_put_strike_price),
            label_offset: LabelOffsetType::Relative(coordinates.0, coordinates.1),
            point_color: RED,
            label_color: RED,
            point_size: 5,
            font_size: 18,
        });

        points.push(self.get_point_at_price(*current_price));

        points
    }
}

impl Greeks for IronButterfly {
    fn greeks(&self) -> Greek {
        let long_call_greek = self.long_call.greeks();
        let long_put_greek = self.long_put.greeks();
        let short_call_greek = self.short_call.greeks();
        let short_put_greek = self.short_put.greeks();

        Greek {
            delta: long_call_greek.delta
                + long_put_greek.delta
                + short_call_greek.delta
                + short_put_greek.delta,
            gamma: long_call_greek.gamma
                + long_put_greek.gamma
                + short_call_greek.gamma
                + short_put_greek.gamma,
            theta: long_call_greek.theta
                + long_put_greek.theta
                + short_call_greek.theta
                + short_put_greek.theta,
            vega: long_call_greek.vega
                + long_put_greek.vega
                + short_call_greek.vega
                + short_put_greek.vega,
            rho: long_call_greek.rho
                + long_put_greek.rho
                + short_call_greek.rho
                + short_put_greek.rho,
            rho_d: long_call_greek.rho_d
                + long_put_greek.rho_d
                + short_call_greek.rho_d
                + short_put_greek.rho_d,
        }
    }
}

impl DeltaNeutrality for IronButterfly {
    fn calculate_net_delta(&self) -> DeltaInfo {
        let long_call_delta = self.long_call.option.delta();
        let long_put_delta = self.long_put.option.delta();
        let short_call_delta = self.short_call.option.delta();
        let short_put_delta = self.short_put.option.delta();
        let threshold = DELTA_THRESHOLD;
        let l_c_delta = d2fu!(long_call_delta.unwrap()).unwrap();
        let l_p_delta = d2fu!(long_put_delta.unwrap()).unwrap();
        let s_c_delta = d2fu!(short_call_delta.unwrap()).unwrap();
        let s_p_delta = d2fu!(short_put_delta.unwrap()).unwrap();

        let delta = l_c_delta + l_p_delta + s_c_delta + s_p_delta;
        DeltaInfo {
            net_delta: delta,
            individual_deltas: vec![l_c_delta, l_p_delta, s_c_delta, s_p_delta],
            is_neutral: (delta).abs() < threshold,
            underlying_price: self.long_call.option.underlying_price,
            neutrality_threshold: threshold,
        }
    }

    fn get_atm_strike(&self) -> Positive {
        self.long_call.option.underlying_price
    }

    fn generate_delta_reducing_adjustments(&self) -> Vec<DeltaAdjustment> {
        let net_delta = self.calculate_net_delta().net_delta;
        let l_p_delta = d2fu!(self.long_put.option.delta().unwrap()).unwrap();
        let s_c_delta = d2fu!(self.short_call.option.delta().unwrap()).unwrap();
        vec![
            DeltaAdjustment::BuyOptions {
                quantity: f2p!((net_delta.abs() / l_p_delta).abs()) * self.long_put.option.quantity,
                strike: self.long_put.option.strike_price,
                option_type: OptionStyle::Put,
            },
            DeltaAdjustment::SellOptions {
                quantity: f2p!((net_delta.abs() / s_c_delta).abs())
                    * self.short_call.option.quantity,
                strike: self.short_call.option.strike_price,
                option_type: OptionStyle::Call,
            },
        ]
    }

    fn generate_delta_increasing_adjustments(&self) -> Vec<DeltaAdjustment> {
        let net_delta = self.calculate_net_delta().net_delta;
        let s_p_delta = d2fu!(self.short_put.option.delta().unwrap()).unwrap();
        let l_c_delta = d2fu!(self.long_call.option.delta().unwrap()).unwrap();

        vec![
            DeltaAdjustment::BuyOptions {
                quantity: f2p!((net_delta.abs() / l_c_delta).abs())
                    * self.long_call.option.quantity,
                strike: self.long_call.option.strike_price,
                option_type: OptionStyle::Call,
            },
            DeltaAdjustment::SellOptions {
                quantity: f2p!((net_delta.abs() / s_p_delta).abs())
                    * self.short_put.option.quantity,
                strike: self.short_put.option.strike_price,
                option_type: OptionStyle::Put,
            },
        ]
    }
}

#[cfg(test)]
mod tests_iron_butterfly {
    use super::*;
    use crate::f2p;
    use chrono::{TimeZone, Utc};

    #[test]
    fn test_iron_butterfly_creation() {
        let date = Utc.with_ymd_and_hms(2024, 12, 1, 0, 0, 0).unwrap();
        let butterfly = IronButterfly::new(
            "AAPL".to_string(),
            f2p!(150.0), // underlying price
            f2p!(150.0), // short strike (at the money)
            f2p!(160.0), // long call strike
            f2p!(140.0), // long put strike
            ExpirationDate::DateTime(date),
            0.2,           // implied volatility
            0.01,          // risk free rate
            0.02,          // dividend yield
            Positive::ONE, // quantity
            1.5,           // premium short call
            1.5,           // premium short put
            1.0,           // premium long call
            1.0,           // premium long put
            5.0,           // open fee
            5.0,           // close fee
        );

        assert_eq!(butterfly.name, "Iron Butterfly");
        assert_eq!(
            butterfly.description,
            IRON_BUTTERFLY_DESCRIPTION.to_string()
        );
        assert_eq!(butterfly.kind, StrategyType::IronButterfly);
        assert_eq!(butterfly.break_even_points.len(), 2);
        assert_eq!(butterfly.short_call.option.strike_price, 150.0);
        assert_eq!(butterfly.short_put.option.strike_price, 150.0);
        assert_eq!(butterfly.long_call.option.strike_price, 160.0);
        assert_eq!(butterfly.long_put.option.strike_price, 140.0);
    }

    #[test]
    fn test_max_loss() {
        let date = Utc.with_ymd_and_hms(2024, 12, 1, 0, 0, 0).unwrap();
        let butterfly = IronButterfly::new(
            "AAPL".to_string(),
            f2p!(100.0), // underlying price
            f2p!(100.0), // short strike (at the money)
            f2p!(110.0), // long call strike
            f2p!(90.0),  // long put strike
            ExpirationDate::DateTime(date),
            0.2,
            0.01,
            0.02,
            Positive::ONE,
            1.5,
            1.5,
            1.0,
            1.0,
            5.0,
            5.0,
        );

        // Max loss should be width of the wing minus net credit received
        let expected_loss: Positive = Positive::TEN - butterfly.net_premium_received().unwrap();
        assert_eq!(butterfly.max_loss().unwrap(), expected_loss);
    }

    #[test]
    fn test_max_profit() {
        let date = Utc.with_ymd_and_hms(2024, 12, 1, 0, 0, 0).unwrap();
        let butterfly = IronButterfly::new(
            "AAPL".to_string(),
            f2p!(100.0), // underlying price
            f2p!(100.0), // short strike (at the money)
            f2p!(110.0), // long call strike
            f2p!(90.0),  // long put strike
            ExpirationDate::DateTime(date),
            0.2,
            0.01,
            0.02,
            Positive::ONE,
            3.5,
            3.5,
            2.0,
            2.0,
            0.07,
            0.07,
        );

        let expected_profit: Positive = butterfly.net_premium_received().unwrap().into();
        assert_eq!(butterfly.max_profit().unwrap(), expected_profit);
    }

    #[test]
    fn test_get_break_even_points() {
        let date = Utc.with_ymd_and_hms(2024, 12, 1, 0, 0, 0).unwrap();
        let butterfly = IronButterfly::new(
            "AAPL".to_string(),
            f2p!(100.0), // underlying price
            f2p!(100.0), // short strike (at the money)
            f2p!(110.0), // long call strike
            f2p!(90.0),  // long put strike
            ExpirationDate::DateTime(date),
            0.2,
            0.01,
            0.02,
            Positive::ONE,
            1.5,
            1.5,
            1.0,
            1.0,
            5.0,
            5.0,
        );

        assert_eq!(
            butterfly.get_break_even_points().unwrap()[0],
            butterfly.break_even_points[0]
        );
        assert_eq!(
            butterfly.get_break_even_points().unwrap()[1],
            butterfly.break_even_points[1]
        );

        // Break-even points should be equidistant from short strike
        let distance_up = butterfly.break_even_points[1] - butterfly.short_call.option.strike_price;
        let distance_down =
            butterfly.short_put.option.strike_price - butterfly.break_even_points[0];
        assert!((distance_up - distance_down) < f2p!(0.01));
    }

    #[test]
    fn test_fees() {
        let date = Utc.with_ymd_and_hms(2024, 12, 1, 0, 0, 0).unwrap();
        let butterfly = IronButterfly::new(
            "AAPL".to_string(),
            f2p!(100.0),
            f2p!(100.0),
            f2p!(110.0),
            f2p!(90.0),
            ExpirationDate::DateTime(date),
            0.2,
            0.01,
            0.02,
            Positive::ONE,
            1.5,
            1.5,
            1.0,
            1.0,
            5.0,
            5.0,
        );

        let expected_fees = butterfly.short_call.open_fee
            + butterfly.short_call.close_fee
            + butterfly.short_put.open_fee
            + butterfly.short_put.close_fee
            + butterfly.long_call.open_fee
            + butterfly.long_call.close_fee
            + butterfly.long_put.open_fee
            + butterfly.long_put.close_fee;
        assert_eq!(butterfly.fees().unwrap().to_f64().unwrap(), expected_fees);
    }

    #[test]
    fn test_calculate_profit_at() {
        let date = Utc.with_ymd_and_hms(2024, 12, 1, 0, 0, 0).unwrap();
        let butterfly = IronButterfly::new(
            "AAPL".to_string(),
            f2p!(100.0),
            f2p!(100.0),
            f2p!(110.0),
            f2p!(90.0),
            ExpirationDate::DateTime(date),
            0.2,
            0.01,
            0.02,
            Positive::ONE,
            1.5,
            1.5,
            1.0,
            1.0,
            5.0,
            5.0,
        );

        // Test at short strike (maximum profit point)
        let price = butterfly.short_call.option.strike_price;
        let expected_profit = butterfly.short_call.pnl_at_expiration(&Some(price))
            + butterfly.short_put.pnl_at_expiration(&Some(price))
            + butterfly.long_call.pnl_at_expiration(&Some(price))
            + butterfly.long_put.pnl_at_expiration(&Some(price));
        assert_eq!(butterfly.calculate_profit_at(price), expected_profit);
    }
}

#[cfg(test)]
mod tests_iron_butterfly_validable {
    use super::*;
    use crate::f2p;
    use crate::model::types::ExpirationDate;

    fn create_valid_position(
        side: Side,
        option_style: OptionStyle,
        strike_price: Positive,
        quantity: Positive,
    ) -> Position {
        Position::new(
            Options::new(
                OptionType::European,
                side,
                "TEST".to_string(),
                strike_price,
                ExpirationDate::Days(30.0),
                0.20,
                quantity,
                f2p!(100.0),
                0.05,
                option_style,
                0.0,
                None,
            ),
            1.0,
            Utc::now(),
            0.0,
            0.0,
        )
    }

    fn create_valid_butterfly() -> IronButterfly {
        IronButterfly::new(
            "TEST".to_string(),
            f2p!(100.0), // underlying_price
            f2p!(100.0), // short strike (both call and put)
            f2p!(110.0), // long call strike
            f2p!(90.0),  // long put strike
            ExpirationDate::Days(30.0),
            0.20,      // implied_volatility
            0.05,      // risk_free_rate
            0.0,       // dividend_yield
            f2p!(1.0), // quantity
            2.0,       // premium_short_call
            2.0,       // premium_short_put
            1.0,       // premium_long_call
            1.0,       // premium_long_put
            0.0,       // open_fee
            0.0,       // closing fee
        )
    }

    #[test]
    fn test_validate_valid_butterfly() {
        let butterfly = create_valid_butterfly();
        assert!(butterfly.validate());
    }

    #[test]
    fn test_validate_invalid_short_call() {
        let mut butterfly = create_valid_butterfly();
        // Make short call invalid by setting quantity to zero
        butterfly.short_call =
            create_valid_position(Side::Short, OptionStyle::Call, f2p!(100.0), Positive::ZERO);
        assert!(!butterfly.validate());
    }

    #[test]
    fn test_validate_invalid_short_put() {
        let mut butterfly = create_valid_butterfly();
        // Make short put invalid by setting quantity to zero
        butterfly.short_put =
            create_valid_position(Side::Short, OptionStyle::Put, f2p!(100.0), Positive::ZERO);
        assert!(!butterfly.validate());
    }

    #[test]
    fn test_validate_invalid_long_call() {
        let mut butterfly = create_valid_butterfly();
        // Make long call invalid by setting quantity to zero
        butterfly.long_call =
            create_valid_position(Side::Long, OptionStyle::Call, f2p!(110.0), Positive::ZERO);
        assert!(!butterfly.validate());
    }

    #[test]
    fn test_validate_invalid_long_put() {
        let mut butterfly = create_valid_butterfly();
        // Make long put invalid by setting quantity to zero
        butterfly.long_put =
            create_valid_position(Side::Long, OptionStyle::Put, f2p!(90.0), Positive::ZERO);
        assert!(!butterfly.validate());
    }

    #[test]
    fn test_validate_all_invalid() {
        let mut butterfly = create_valid_butterfly();
        // Make all positions invalid
        butterfly.short_call =
            create_valid_position(Side::Short, OptionStyle::Call, f2p!(100.0), Positive::ZERO);
        butterfly.short_put =
            create_valid_position(Side::Short, OptionStyle::Put, f2p!(100.0), Positive::ZERO);
        butterfly.long_call =
            create_valid_position(Side::Long, OptionStyle::Call, f2p!(110.0), Positive::ZERO);
        butterfly.long_put =
            create_valid_position(Side::Long, OptionStyle::Put, f2p!(90.0), Positive::ZERO);
        assert!(!butterfly.validate());
    }

    #[test]
    fn test_validate_different_short_strikes() {
        let mut butterfly = create_valid_butterfly();
        // Make short strikes different
        butterfly.short_call =
            create_valid_position(Side::Short, OptionStyle::Call, f2p!(105.0), f2p!(1.0));
        butterfly.short_put =
            create_valid_position(Side::Short, OptionStyle::Put, f2p!(95.0), f2p!(1.0));
        assert!(!butterfly.validate());
    }

    #[test]
    fn test_validate_inverted_strikes() {
        let mut butterfly = create_valid_butterfly();
        // Invert the strikes
        butterfly.long_put =
            create_valid_position(Side::Long, OptionStyle::Put, f2p!(105.0), f2p!(1.0));
        butterfly.short_put =
            create_valid_position(Side::Short, OptionStyle::Put, f2p!(110.0), f2p!(1.0));
        assert!(!butterfly.validate());
    }
}

#[cfg(test)]
mod tests_iron_butterfly_strategies {
    use super::*;
    use crate::f2p;
    use crate::model::types::ExpirationDate;

    fn create_test_butterfly() -> IronButterfly {
        IronButterfly::new(
            "TEST".to_string(),
            f2p!(100.0), // underlying_price
            f2p!(100.0), // short strike (both call and put)
            f2p!(110.0), // long call strike
            f2p!(90.0),  // long put strike
            ExpirationDate::Days(30.0),
            0.20,      // implied_volatility
            0.05,      // risk_free_rate
            0.0,       // dividend_yield
            f2p!(1.0), // quantity
            2.0,       // premium_short_call
            2.0,       // premium_short_put
            1.0,       // premium_long_call
            1.0,       // premium_long_put
            0.5,       // open_fee
            0.5,       // closing fee
        )
    }

    #[test]
    fn test_add_leg() {
        let mut butterfly = create_test_butterfly();

        // Test adding a short call at the same strike as short put
        let new_short_call = Position::new(
            Options::new(
                OptionType::European,
                Side::Short,
                "TEST".to_string(),
                f2p!(100.0),
                ExpirationDate::Days(30.0),
                0.20,
                f2p!(1.0),
                f2p!(100.0),
                0.05,
                OptionStyle::Call,
                0.0,
                None,
            ),
            2.5,
            Utc::now(),
            0.5,
            0.5,
        );
        butterfly
            .add_position(&new_short_call.clone())
            .expect("Failed to add short call");
        assert_eq!(
            butterfly.short_call.option.strike_price,
            butterfly.short_put.option.strike_price
        );

        // Test adding a long put
        let new_long_put = Position::new(
            Options::new(
                OptionType::European,
                Side::Long,
                "TEST".to_string(),
                f2p!(90.0),
                ExpirationDate::Days(30.0),
                0.20,
                f2p!(1.0),
                f2p!(100.0),
                0.05,
                OptionStyle::Put,
                0.0,
                None,
            ),
            1.5,
            Utc::now(),
            0.5,
            0.5,
        );
        butterfly
            .add_position(&new_long_put.clone())
            .expect("Failed to add long put");
        assert_eq!(butterfly.long_put.option.strike_price, f2p!(90.0));
    }

    #[test]
    fn test_get_legs() {
        let butterfly = create_test_butterfly();
        let legs = butterfly.get_positions().expect("Failed to get legs");

        assert_eq!(legs.len(), 4);
        assert_eq!(legs[0].option.option_style, OptionStyle::Call);
        assert_eq!(legs[0].option.side, Side::Short);
        assert_eq!(legs[1].option.option_style, OptionStyle::Put);
        assert_eq!(legs[1].option.side, Side::Short);
        assert_eq!(legs[2].option.option_style, OptionStyle::Call);
        assert_eq!(legs[2].option.side, Side::Long);
        assert_eq!(legs[3].option.option_style, OptionStyle::Put);
        assert_eq!(legs[3].option.side, Side::Long);

        // Verify short strikes are equal
        assert_eq!(legs[0].option.strike_price, legs[1].option.strike_price);
    }

    #[test]
    fn test_get_break_even_points() {
        let butterfly = create_test_butterfly();
        let break_even_points = butterfly.get_break_even_points().unwrap();

        assert_eq!(break_even_points.len(), 2);

        // Break-even points should be equidistant from short strike
        let short_strike = butterfly.short_call.option.strike_price;
        let upper_distance = break_even_points[1] - short_strike;
        let lower_distance = short_strike - break_even_points[0];
        assert!((upper_distance - lower_distance) < f2p!(0.01));
    }

    #[test]
    fn test_max_profit() {
        let butterfly = create_test_butterfly();
        assert!(butterfly.max_profit().is_err());
    }

    #[test]
    fn test_max_loss() {
        let butterfly = create_test_butterfly();
        let max_loss = butterfly.max_loss().unwrap();

        // Max loss should be equal at both wings
        let loss_at_long_put =
            butterfly.calculate_profit_at(butterfly.long_put.option.strike_price);
        let loss_at_long_call =
            butterfly.calculate_profit_at(butterfly.long_call.option.strike_price);
        assert!((loss_at_long_put - loss_at_long_call).abs() < 0.01);
        assert_eq!(max_loss, f2p!(loss_at_long_put.abs()));
    }

    #[test]
    fn test_total_cost() {
        let butterfly = create_test_butterfly();
        let total_cost = butterfly.total_cost();
        let expected_cost = f2p!(6.0); // 2.0 + 2.0 + 1.0 + 1.0
        assert_eq!(total_cost, expected_cost);
    }

    #[test]
    fn test_net_premium_received() {
        let butterfly = create_test_butterfly();
        assert_eq!(
            butterfly.net_premium_received().unwrap().to_f64().unwrap(),
            -2.0
        );
    }

    #[test]
    fn test_fees() {
        let butterfly = create_test_butterfly();
        let expected_fees = 4.0; // (0.5 + 0.5) * 4 legs
        assert_eq!(butterfly.fees().unwrap().to_f64().unwrap(), expected_fees);
    }

    #[test]
    fn test_profit_area() {
        let butterfly = create_test_butterfly();
        // Profit area should be smaller than Iron Condor due to higher concentration
        assert!(butterfly.profit_area().unwrap().to_f64().unwrap() < 1.0);
    }

    #[test]
    fn test_with_multiple_contracts() {
        let butterfly = IronButterfly::new(
            "TEST".to_string(),
            f2p!(100.0),
            f2p!(100.0),
            f2p!(110.0),
            f2p!(90.0),
            ExpirationDate::Days(30.0),
            0.20,
            0.05,
            0.0,
            f2p!(2.0), // quantity = 2
            2.0,
            2.0,
            1.0,
            1.0,
            0.5,
            0.5,
        );

        assert_eq!(
            butterfly.net_premium_received().unwrap().to_f64().unwrap(),
            -4.0
        );
    }

    #[test]
    fn test_with_asymmetric_premiums() {
        let butterfly = IronButterfly::new(
            "TEST".to_string(),
            f2p!(100.0),
            f2p!(100.0),
            f2p!(110.0),
            f2p!(90.0),
            ExpirationDate::Days(30.0),
            0.20,
            0.05,
            0.0,
            f2p!(1.0),
            3.0, // Higher short call premium
            2.0, // Lower short put premium
            1.0,
            1.0,
            0.5,
            0.5,
        );

        assert_eq!(
            butterfly.net_premium_received().unwrap().to_f64().unwrap(),
            -1.0
        );
    }
}

#[cfg(test)]
mod tests_iron_butterfly_optimizable {
    use super::*;
    use crate::chains::chain::OptionData;
    use crate::f2p;
    use crate::model::types::ExpirationDate;
    use crate::spos;

    fn create_test_butterfly() -> IronButterfly {
        IronButterfly::new(
            "TEST".to_string(),
            f2p!(100.0), // underlying_price
            f2p!(100.0), // short strike (both call and put)
            f2p!(110.0), // long call strike
            f2p!(90.0),  // long put strike
            ExpirationDate::Days(30.0),
            0.20,      // implied_volatility
            0.05,      // risk_free_rate
            0.0,       // dividend_yield
            f2p!(1.0), // quantity
            2.0,       // premium_short_call
            2.0,       // premium_short_put
            1.0,       // premium_long_call
            1.0,       // premium_long_put
            0.5,       // open_fee
            0.5,       // closing fee
        )
    }

    fn create_test_chain() -> OptionChain {
        let mut chain = OptionChain::new("TEST", f2p!(100.0), "2024-12-31".to_string(), None, None);

        // Add options at various strikes
        for strike in [85.0, 90.0, 95.0, 100.0, 105.0, 110.0, 115.0] {
            chain.add_option(
                f2p!(strike),
                spos!(5.0),   // call_bid
                spos!(5.2),   // call_ask
                spos!(5.0),   // put_bid
                spos!(5.2),   // put_ask
                spos!(0.2),   // implied_volatility
                None,         // delta
                spos!(100.0), // volume
                Some(50),     // open_interest
            );
        }
        chain
    }

    #[test]
    fn test_find_optimal_at_the_money() {
        let mut butterfly = create_test_butterfly();
        let chain = create_test_chain();

        butterfly.find_optimal(&chain, FindOptimalSide::All, OptimizationCriteria::Ratio);

        assert!(butterfly.validate());
        // Short strikes should be at or very near the money
        let diff = (butterfly.short_call.option.strike_price.to_f64()
            - chain.underlying_price.to_f64())
        .abs();
        assert!(diff <= 5.0); // Allow some flexibility in strike selection
        assert_eq!(
            butterfly.short_call.option.strike_price,
            butterfly.short_put.option.strike_price
        );
    }

    #[test]
    fn test_find_optimal_symmetric_wings() {
        let mut butterfly = create_test_butterfly();
        let chain = create_test_chain();

        butterfly.find_optimal(&chain, FindOptimalSide::All, OptimizationCriteria::Area);

        assert!(butterfly.validate());
        // Wings should be roughly symmetric
        let upper_wing =
            butterfly.long_call.option.strike_price - butterfly.short_call.option.strike_price;
        let lower_wing =
            butterfly.short_put.option.strike_price - butterfly.long_put.option.strike_price;
        assert!((upper_wing - lower_wing).to_f64().abs() <= 5.0);
    }

    #[test]
    fn test_find_optimal_range() {
        let mut butterfly = create_test_butterfly();
        let chain = create_test_chain();

        butterfly.find_optimal(
            &chain,
            FindOptimalSide::Range(f2p!(95.0), f2p!(105.0)),
            OptimizationCriteria::Ratio,
        );

        assert!(butterfly.validate());
        // Short strikes should be within the specified range
        assert!(butterfly.short_call.option.strike_price >= f2p!(95.0));
        assert!(butterfly.short_call.option.strike_price <= f2p!(105.0));
        // And should be equal
        assert_eq!(
            butterfly.short_call.option.strike_price,
            butterfly.short_put.option.strike_price
        );
    }

    #[test]
    fn test_is_valid_long_option() {
        let butterfly = create_test_butterfly();
        let option = OptionData::new(
            f2p!(90.0),
            spos!(5.0),
            spos!(5.2),
            spos!(5.0),
            spos!(5.2),
            spos!(0.2),
            None,
            spos!(100.0),
            Some(50),
        );

        // Test with different sides
        assert!(butterfly.is_valid_long_option(&option, &FindOptimalSide::All));
        assert!(butterfly.is_valid_long_option(&option, &FindOptimalSide::Lower));
        assert!(!butterfly.is_valid_long_option(&option, &FindOptimalSide::Upper));
    }

    #[test]
    fn test_is_valid_short_option() {
        let butterfly = create_test_butterfly();
        let option = OptionData::new(
            f2p!(100.0), // At the money
            spos!(5.0),
            spos!(5.2),
            spos!(5.0),
            spos!(5.2),
            spos!(0.2),
            None,
            spos!(100.0),
            Some(50),
        );

        // Test with different sides - should prefer at-the-money options
        assert!(butterfly.is_valid_short_option(&option, &FindOptimalSide::All));
        assert!(butterfly
            .is_valid_short_option(&option, &FindOptimalSide::Range(f2p!(95.0), f2p!(105.0))));
    }

    #[test]
    fn test_create_strategy() {
        let butterfly = create_test_butterfly();
        let chain = create_test_chain();
        let options: Vec<&OptionData> = chain.options.iter().collect();

        let legs = StrategyLegs::FourLegs {
            first: options[1],  // 90.0 strike for long put
            second: options[3], // 100.0 strike for both shorts
            third: options[3],  // 100.0 strike for both shorts
            fourth: options[5], // 110.0 strike for long call
        };

        let new_strategy = butterfly.create_strategy(&chain, &legs);
        assert!(new_strategy.validate());
        assert_eq!(new_strategy.long_put.option.strike_price, f2p!(90.0));
        assert_eq!(new_strategy.short_put.option.strike_price, f2p!(100.0));
        assert_eq!(new_strategy.short_call.option.strike_price, f2p!(100.0));
        assert_eq!(new_strategy.long_call.option.strike_price, f2p!(110.0));
    }

    #[test]
    #[should_panic(expected = "Invalid number of legs for Iron Butterfly strategy")]
    fn test_create_strategy_invalid_legs() {
        let butterfly = create_test_butterfly();
        let chain = create_test_chain();
        let options: Vec<&OptionData> = chain.options.iter().collect();

        let legs = StrategyLegs::TwoLegs {
            first: options[0],
            second: options[1],
        };

        let _ = butterfly.create_strategy(&chain, &legs);
    }
}

#[cfg(test)]
mod tests_iron_butterfly_profit {
    use super::*;
    use crate::f2p;
    use crate::model::types::ExpirationDate;

    fn create_test_butterfly() -> IronButterfly {
        IronButterfly::new(
            "TEST".to_string(),
            f2p!(100.0), // underlying_price
            f2p!(100.0), // short strike (both call and put)
            f2p!(110.0), // long call strike
            f2p!(90.0),  // long put strike
            ExpirationDate::Days(30.0),
            0.20,      // implied_volatility
            0.05,      // risk_free_rate
            0.0,       // dividend_yield
            f2p!(1.0), // quantity
            2.0,       // premium_short_call
            2.0,       // premium_short_put
            1.0,       // premium_long_call
            1.0,       // premium_long_put
            0.0,       // open_fee
            0.0,       // closing fee
        )
    }

    #[test]
    fn test_profit_at_max_profit_price() {
        let butterfly = create_test_butterfly();
        let profit = butterfly.calculate_profit_at(butterfly.short_call.option.strike_price);
        // Net premium = (2.0 + 2.0) - (1.0 + 1.0) = 2.0
        assert_eq!(profit, 2.0);
    }

    #[test]
    fn test_profit_below_long_put() {
        let butterfly = create_test_butterfly();
        let profit = butterfly.calculate_profit_at(f2p!(85.0));
        // Maximum loss = width of wing - net premium = 10 - 2 = 8
        assert_eq!(profit, -8.0);
    }

    #[test]
    fn test_profit_at_long_put() {
        let butterfly = create_test_butterfly();
        let profit = butterfly.calculate_profit_at(butterfly.long_put.option.strike_price);
        // Maximum loss = width of wing - net premium = 10 - 2 = 8
        assert_eq!(profit, -8.0);
    }

    #[test]
    fn test_profit_between_put_wing() {
        let butterfly = create_test_butterfly();
        let profit = butterfly.calculate_profit_at(f2p!(95.0));
        let max_loss = -8.0;
        let max_profit = 2.0;
        assert!(profit > max_loss && profit < max_profit);
    }

    #[test]
    fn test_profit_at_short_strike() {
        let butterfly = create_test_butterfly();
        let profit = butterfly.calculate_profit_at(butterfly.short_call.option.strike_price);
        // Maximum profit = net premium = 2.0
        assert_eq!(profit, 2.0);
    }

    #[test]
    fn test_profit_between_call_wing() {
        let butterfly = create_test_butterfly();
        let profit = butterfly.calculate_profit_at(f2p!(105.0));
        let max_loss = -8.0;
        let max_profit = 2.0;
        assert!(profit > max_loss && profit < max_profit);
    }

    #[test]
    fn test_profit_at_long_call() {
        let butterfly = create_test_butterfly();
        let profit = butterfly.calculate_profit_at(butterfly.long_call.option.strike_price);
        // Maximum loss = width of wing - net premium = 10 - 2 = 8
        assert_eq!(profit, -8.0);
    }

    #[test]
    fn test_profit_above_long_call() {
        let butterfly = create_test_butterfly();
        let profit = butterfly.calculate_profit_at(f2p!(115.0));
        // Maximum loss = width of wing - net premium = 10 - 2 = 8
        assert_eq!(profit, -8.0);
    }

    #[test]
    fn test_profit_with_fees() {
        let butterfly = IronButterfly::new(
            "TEST".to_string(),
            f2p!(100.0),
            f2p!(100.0),
            f2p!(110.0),
            f2p!(90.0),
            ExpirationDate::Days(30.0),
            0.20,
            0.05,
            0.0,
            f2p!(1.0),
            2.0,
            2.0,
            1.0,
            1.0,
            0.5, // open_fee
            0.5, // closing fee
        );

        let profit = butterfly.calculate_profit_at(f2p!(100.0));
        // Net premium = 2.0 - fees = 2.0 - 4.0 = -2.0
        assert_eq!(profit, -2.0);
    }

    #[test]
    fn test_profit_with_multiple_contracts() {
        let butterfly = IronButterfly::new(
            "TEST".to_string(),
            f2p!(100.0),
            f2p!(100.0),
            f2p!(110.0),
            f2p!(90.0),
            ExpirationDate::Days(30.0),
            0.20,
            0.05,
            0.0,
            f2p!(2.0), // quantity = 2
            2.0,
            2.0,
            1.0,
            1.0,
            0.0,
            0.0,
        );

        let profit = butterfly.calculate_profit_at(butterfly.short_call.option.strike_price);
        // Net premium * quantity = 2.0 * 2 = 4.0
        assert_eq!(profit, 4.0);
    }

    #[test]
    fn test_profit_at_break_even_points() {
        let butterfly = create_test_butterfly();

        // Break-evens should be equidistant from short strike
        let short_strike = butterfly.short_call.option.strike_price;
        let lower_break_even = f2p!((short_strike - 2.0).to_f64());
        let upper_break_even = f2p!((short_strike + 2.0).to_f64());

        let lower_profit = butterfly.calculate_profit_at(lower_break_even);
        let upper_profit = butterfly.calculate_profit_at(upper_break_even);

        assert!(lower_profit.abs() < 0.001);
        assert!(upper_profit.abs() < 0.001);

        // Break-evens should be equidistant from short strike
        assert!(
            (lower_break_even.to_f64() - short_strike.to_f64()).abs()
                == (upper_break_even.to_f64() - short_strike.to_f64()).abs()
        );
    }

    #[test]
    fn test_symmetric_profits() {
        let butterfly = create_test_butterfly();
        let short_strike = butterfly.short_call.option.strike_price;

        // Test points equidistant from short strike should have equal profits
        for offset in [2.0, 4.0, 6.0, 8.0] {
            let up_profit = butterfly.calculate_profit_at(f2p!((short_strike + offset).to_f64()));
            let down_profit = butterfly.calculate_profit_at(f2p!((short_strike - offset).to_f64()));
            assert!((up_profit - down_profit).abs() < 0.001);
        }
    }
}

#[cfg(test)]
mod tests_iron_butterfly_graph {
    use super::*;
    use crate::f2p;
    use crate::model::types::ExpirationDate;

    fn create_test_butterfly() -> IronButterfly {
        IronButterfly::new(
            "TEST".to_string(),
            f2p!(100.0), // underlying_price
            f2p!(100.0), // short strike (both call and put)
            f2p!(110.0), // long call strike
            f2p!(90.0),  // long put strike
            ExpirationDate::Days(30.0),
            0.20,      // implied_volatility
            0.05,      // risk_free_rate
            0.0,       // dividend_yield
            f2p!(2.0), // quantity
            2.0,       // premium_short_call
            2.0,       // premium_short_put
            1.0,       // premium_long_call
            1.0,       // premium_long_put
            0.0,       // open_fee
            0.0,       // closing fee
        )
    }

    #[test]
    fn test_title_format() {
        let butterfly = create_test_butterfly();
        let title = butterfly.title();

        assert!(title.contains("IronButterfly Strategy"));
        assert!(title.contains("TEST")); // Symbol
        assert!(title.contains("Size 2")); // Quantity

        assert!(title.contains("Long Put: $90"));
        assert!(title.contains("Short Put: $100")); // Common strike for both shorts
        assert!(title.contains("Short Call: $100"));
        assert!(title.contains("Long Call: $110"));

        assert!(title.contains("Expire:"));
    }

    #[test]
    fn test_get_vertical_lines() {
        let butterfly = create_test_butterfly();
        let lines = butterfly.get_vertical_lines();

        assert_eq!(lines.len(), 1); // Current price and short strike

        // Current price line
        assert_eq!(lines[0].x_coordinate, 100.0);
        assert_eq!(lines[0].y_range, (-50000.0, 50000.0));
        assert!(lines[0].label.contains("Current Price: 100"));
    }

    #[test]
    fn test_get_points() {
        let butterfly = create_test_butterfly();
        let points = butterfly.get_points();

        assert_eq!(points.len(), 6); // Break evens, max profit, and max losses

        let lower_break_even = &points[0];
        let upper_break_even = &points[1];
        assert_eq!(lower_break_even.coordinates.1, 0.0);
        assert_eq!(upper_break_even.coordinates.1, 0.0);

        // Break evens should be equidistant from short strike
        let short_strike = butterfly.short_call.option.strike_price.to_f64();
        let lower_distance = short_strike - lower_break_even.coordinates.0;
        let upper_distance = upper_break_even.coordinates.0 - short_strike;
        assert!((lower_distance - upper_distance).abs() < 0.001);

        let max_profit = &points[2];
        assert_eq!(max_profit.coordinates.0, short_strike);
        assert!(max_profit.label.contains("Max Profit"));

        let left_max_loss = &points[3];
        let right_max_loss = &points[4];
        assert_eq!(left_max_loss.coordinates.0, 110.0);
        assert_eq!(right_max_loss.coordinates.0, 90.0);
        // Max losses should be equal
        assert_eq!(left_max_loss.coordinates.1, right_max_loss.coordinates.1);
    }

    #[test]
    fn test_point_colors() {
        let butterfly = create_test_butterfly();
        let points = butterfly.get_points();

        for point in &points {
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

    #[test]
    fn test_point_styles() {
        let butterfly = create_test_butterfly();
        let points = butterfly.get_points();

        for point in points {
            assert_eq!(point.point_size, 5);
            assert_eq!(point.font_size, 18);
            assert_eq!(point.label_color, point.point_color);
        }
    }

    #[test]
    fn test_zero_profit_points() {
        let mut butterfly = create_test_butterfly();
        butterfly.short_call.premium = 1.0;
        butterfly.short_put.premium = 1.0;
        butterfly.long_call.premium = 1.0;
        butterfly.long_put.premium = 1.0;

        let points = butterfly.get_points();
        let max_profit_point = &points[2];

        assert_eq!(max_profit_point.coordinates.1, 0.0);
        assert!(max_profit_point.label.contains("0.00"));
    }

    #[test]
    fn test_points_with_different_quantities() {
        let butterfly = create_test_butterfly();
        let points = butterfly.get_points();

        let max_profit_point = &points[2];
        let max_loss_point = &points[3];

        // With quantity = 2, all profits/losses should be doubled
        assert_eq!(max_profit_point.coordinates.1, 4.0); // 2 * 2.0
        assert_eq!(max_loss_point.coordinates.1, -16.0); // 2 * -8.0
    }

    #[test]
    fn test_profit_curve_symmetry() {
        let butterfly = create_test_butterfly();
        let short_strike = butterfly.short_call.option.strike_price.to_f64();

        // Test points equidistant from short strike should have equal profits
        for offset in [2.0, 4.0, 6.0, 8.0] {
            let profit_up = butterfly.calculate_profit_at(f2p!(short_strike + offset));
            let profit_down = butterfly.calculate_profit_at(f2p!(short_strike - offset));
            assert!((profit_up - profit_down).abs() < 0.001);
        }
    }
}

#[cfg(test)]
mod tests_iron_condor_delta {
    use super::*;
    use crate::model::types::{ExpirationDate, OptionStyle};
    use crate::strategies::delta_neutral::DELTA_THRESHOLD;
    use crate::strategies::delta_neutral::{DeltaAdjustment, DeltaNeutrality};
    use crate::strategies::iron_butterfly::IronButterfly;
    use crate::{d2fu, f2p};
    use approx::assert_relative_eq;

    fn get_strategy(underlying_price: Positive) -> IronButterfly {
        IronButterfly::new(
            "GOLD".to_string(),
            underlying_price, // underlying_price
            f2p!(2725.0),     // short_call_strike
            f2p!(2800.0),     // long_call_strike
            f2p!(2500.0),     // long_put_strike
            ExpirationDate::Days(30.0),
            0.1548,    // implied_volatility
            0.05,      // risk_free_rate
            0.0,       // dividend_yield
            f2p!(1.0), // quantity
            38.8,      // premium_short_call
            30.4,      // premium_short_put
            23.3,      // premium_long_call
            16.8,      // premium_long_put
            0.96,      // open_fee
            0.96,      // close_fee
        )
    }

    #[test]
    fn create_test_reducing_adjustments() {
        let strategy = get_strategy(f2p!(2900.0));

        assert_relative_eq!(
            strategy.calculate_net_delta().net_delta,
            -0.053677,
            epsilon = 0.0001
        );
        assert!(!strategy.is_delta_neutral());
        let suggestion = strategy.suggest_delta_adjustments();
        assert_eq!(
            suggestion[0],
            DeltaAdjustment::BuyOptions {
                quantity: f2p!(0.06566830350547599),
                strike: f2p!(2800.0),
                option_type: OptionStyle::Call
            }
        );
        assert_eq!(
            suggestion[1],
            DeltaAdjustment::SellOptions {
                quantity: f2p!(0.8309463413138212),
                strike: f2p!(2725.0),
                option_type: OptionStyle::Put
            }
        );

        let mut option = strategy.long_call.option.clone();
        option.quantity = f2p!(0.06566830350547599);
        let delta = d2fu!(option.delta().unwrap()).unwrap();
        assert_relative_eq!(delta, 0.053677, epsilon = 0.0001);
        assert_relative_eq!(
            delta + strategy.calculate_net_delta().net_delta,
            0.0,
            epsilon = DELTA_THRESHOLD
        );
    }

    #[test]
    fn create_test_increasing_adjustments() {
        let strategy = get_strategy(f2p!(2500.0));

        assert_relative_eq!(
            strategy.calculate_net_delta().net_delta,
            0.485367,
            epsilon = 0.0001
        );
        assert!(!strategy.is_delta_neutral());
        let suggestion = strategy.suggest_delta_adjustments();
        assert_eq!(
            suggestion[0],
            DeltaAdjustment::BuyOptions {
                quantity: f2p!(1.068371502787395),
                strike: f2p!(2500.0),
                option_type: OptionStyle::Put
            }
        );
        assert_eq!(
            suggestion[1],
            DeltaAdjustment::SellOptions {
                quantity: f2p!(14.339865587583922),
                strike: f2p!(2725.0),
                option_type: OptionStyle::Call
            }
        );

        let mut option = strategy.long_put.option.clone();
        option.quantity = f2p!(1.068371502787395);
        let delta = d2fu!(option.delta().unwrap()).unwrap();
        assert_relative_eq!(delta, -0.485367, epsilon = 0.0001);
        assert_relative_eq!(
            delta + strategy.calculate_net_delta().net_delta,
            0.0,
            epsilon = DELTA_THRESHOLD
        );
    }

    #[test]
    fn create_test_no_adjustments() {
        let strategy = get_strategy(f2p!(2100.0));

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
    use super::*;
    use crate::model::types::{ExpirationDate, OptionStyle};
    use crate::strategies::delta_neutral::DELTA_THRESHOLD;
    use crate::strategies::delta_neutral::{DeltaAdjustment, DeltaNeutrality};
    use crate::strategies::iron_butterfly::IronButterfly;
    use crate::{d2fu, f2p};
    use approx::assert_relative_eq;

    fn get_strategy(underlying_price: Positive) -> IronButterfly {
        IronButterfly::new(
            "GOLD".to_string(),
            underlying_price, // underlying_price
            f2p!(2725.0),     // short_call_strike
            f2p!(2800.0),     // long_call_strike
            f2p!(2500.0),     // long_put_strike
            ExpirationDate::Days(30.0),
            0.1548,    // implied_volatility
            0.05,      // risk_free_rate
            0.0,       // dividend_yield
            f2p!(2.0), // quantity
            38.8,      // premium_short_call
            30.4,      // premium_short_put
            23.3,      // premium_long_call
            16.8,      // premium_long_put
            0.96,      // open_fee
            0.96,      // close_fee
        )
    }

    #[test]
    fn create_test_reducing_adjustments() {
        let strategy = get_strategy(f2p!(2900.0));

        assert_relative_eq!(
            strategy.calculate_net_delta().net_delta,
            -0.107354,
            epsilon = 0.0001
        );
        assert!(!strategy.is_delta_neutral());
        let suggestion = strategy.suggest_delta_adjustments();
        assert_eq!(
            suggestion[0],
            DeltaAdjustment::BuyOptions {
                quantity: f2p!(0.13133660701095198),
                strike: f2p!(2800.0),
                option_type: OptionStyle::Call
            }
        );
        assert_eq!(
            suggestion[1],
            DeltaAdjustment::SellOptions {
                quantity: f2p!(1.6618926826276423),
                strike: f2p!(2725.0),
                option_type: OptionStyle::Put
            }
        );

        let mut option = strategy.long_call.option.clone();
        option.quantity = f2p!(0.13133660701095198);
        let delta = d2fu!(option.delta().unwrap()).unwrap();
        assert_relative_eq!(delta, 0.10735, epsilon = 0.0001);
        assert_relative_eq!(
            delta + strategy.calculate_net_delta().net_delta,
            0.0,
            epsilon = DELTA_THRESHOLD
        );
    }

    #[test]
    fn create_test_increasing_adjustments() {
        let strategy = get_strategy(f2p!(2700.0));
        let size = 0.5645;
        let delta = f2p!(1.219357854222914);
        assert_relative_eq!(
            strategy.calculate_net_delta().net_delta,
            size,
            epsilon = 0.0001
        );
        assert!(!strategy.is_delta_neutral());
        let suggestion = strategy.suggest_delta_adjustments();
        assert_eq!(
            suggestion[0],
            DeltaAdjustment::BuyOptions {
                quantity: f2p!(17.514_681_033_591_1),
                strike: f2p!(2500.0),
                option_type: OptionStyle::Put
            }
        );
        assert_eq!(
            suggestion[1],
            DeltaAdjustment::SellOptions {
                quantity: delta,
                strike: f2p!(2725.0),
                option_type: OptionStyle::Call
            }
        );

        let mut option = strategy.short_call.option.clone();
        option.quantity = delta;
        let delta = d2fu!(option.delta().unwrap()).unwrap();
        assert_relative_eq!(delta, -size, epsilon = 0.0001);
        assert_relative_eq!(
            delta + strategy.calculate_net_delta().net_delta,
            0.0,
            epsilon = DELTA_THRESHOLD
        );
    }

    #[test]
    fn create_test_no_adjustments() {
        let strategy = get_strategy(f2p!(2100.0));

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
