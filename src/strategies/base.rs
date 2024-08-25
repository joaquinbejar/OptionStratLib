/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 21/8/24
******************************************************************************/
use crate::constants::ZERO;
use crate::model::position::Position;
use crate::model::types::Side;
use crate::visualization::utils::Graph;

/// This enum represents different types of trading strategies.
/// Each variant represents a specific strategy type.
#[derive(Clone, Debug, PartialEq)]
pub enum StrategyType {
    BullCallSpread,
    BearCallSpread,
    BullPutSpread,
    BearPutSpread,
    IronCondor,
    Straddle,
    Strangle,
    CoveredCall,
    ProtectivePut,
    Collar,
    LongCall,
    LongPut,
    ShortCall,
    ShortPut,
    Custom,
}

/// Represents a trading strategy.
///
/// A strategy consists of the following properties:
///
/// - `name`: The name of the strategy.
/// - `kind`: The type of the strategy.
/// - `description`: A description of the strategy.
/// - `legs`: A vector of positions that make up the strategy.
/// - `max_profit`: The maximum potential profit of the strategy (optional).
/// - `max_loss`: The maximum potential loss of the strategy (optional).
/// - `break_even_points`: A vector of break-even points for the strategy.
pub struct Strategy {
    pub name: String,
    pub kind: StrategyType,
    pub description: String,
    pub legs: Vec<Position>,
    pub max_profit: Option<f64>,
    pub max_loss: Option<f64>,
    pub break_even_points: Vec<f64>,
}

impl Strategy {
    pub fn new(name: String, kind: StrategyType, description: String) -> Self {
        Strategy {
            name,
            kind,
            description,
            legs: Vec::new(),
            max_profit: None,
            max_loss: None,
            break_even_points: Vec::new(),
        }
    }

    pub fn add_leg(&mut self, position: Position) {
        self.legs.push(position);
    }

    pub fn set_max_profit(&mut self, max_profit: f64) {
        self.max_profit = Some(max_profit);
    }

    pub fn set_max_loss(&mut self, max_loss: f64) {
        self.max_loss = Some(max_loss);
    }

    pub fn add_break_even_point(&mut self, point: f64) {
        self.break_even_points.push(point);
    }

    pub fn break_even(&self) -> f64 {
        match self.kind {
            StrategyType::BullCallSpread => {
                let long_call = self
                    .legs
                    .iter()
                    .find(|leg| leg.option.side == Side::Long)
                    .unwrap();
                let short_call = self
                    .legs
                    .iter()
                    .find(|leg| leg.option.side == Side::Short)
                    .unwrap();
                long_call.option.strike_price
                    + (long_call.total_cost() - short_call.net_premium_received())
                        / long_call.option.quantity as f64
            }
            StrategyType::BearCallSpread => ZERO,
            StrategyType::BullPutSpread => ZERO,
            StrategyType::BearPutSpread => ZERO,
            StrategyType::IronCondor => ZERO,
            StrategyType::Straddle => ZERO,
            StrategyType::Strangle => ZERO,
            StrategyType::CoveredCall => ZERO,
            StrategyType::ProtectivePut => ZERO,
            StrategyType::Collar => ZERO,
            StrategyType::LongCall => ZERO,
            StrategyType::LongPut => ZERO,
            StrategyType::ShortCall => ZERO,
            StrategyType::ShortPut => ZERO,
            StrategyType::Custom => ZERO,
        }
    }

    pub fn calculate_profit_at(&self, price: f64) -> f64 {
        self.legs
            .iter()
            .map(|leg| leg.pnl_at_expiration(Some(price)))
            .sum()
    }
}

impl Graph for Strategy {

    fn title(&self) -> String {
        let strategy_title = format!("Strategy: {} - {:?}", self.name, self.kind);
        let leg_titles: Vec<String> = self.legs.iter().map(|leg| leg.title()).collect();

        if leg_titles.is_empty() {
            strategy_title
        } else {
            format!("{}\n{}", strategy_title, leg_titles.join("\n"))
        }
    }

    fn get_values(&self, data: &[f64]) -> Vec<f64> {
        data.iter()
            .map(|&price| self.calculate_profit_at(price))
            .collect()
    }

    fn get_vertical_lines(&self) -> Vec<(String, f64)> {
        [("Break Even".to_string(), self.break_even())].to_vec()
    }
}

pub trait Strategies {
    fn add_leg(&mut self, position: Position);
    fn break_even(&self) -> f64;

    fn calculate_profit_at(&self, price: f64) -> f64;

    fn max_profit(&self) -> f64;

    fn max_loss(&self) -> f64;

    fn total_cost(&self) -> f64;

    fn net_premium_received(&self) -> f64;
}
