/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 21/8/24
******************************************************************************/

use crate::model::position::Position;

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

    pub fn calculate_profit_at(&self, price: f64) -> f64 {
        self.legs
            .iter()
            .map(|leg| leg.pnl_at_expiration(Some(price)))
            .sum()
    }
}

/// This enum represents different types of trading strategies.
/// Each variant represents a specific strategy type.
#[derive(Clone, Debug)]
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
