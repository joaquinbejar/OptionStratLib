/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 21/8/24
******************************************************************************/

use crate::constants::ZERO;
use crate::model::chain::OptionChain;
use crate::model::position::Position;
use crate::model::types::{PositiveF64, PZERO};
use crate::strategies::utils::{FindOptimalSide, OptimizationCriteria};

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
    PoorMansCoveredCall,
    CallButterfly,
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
    pub break_even_points: Vec<PositiveF64>,
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
}

pub trait Strategies {
    fn add_leg(&mut self, _position: Position) {
        panic!("Add leg is not applicable for this strategy");
    }

    fn get_legs(&self) -> Vec<Position> {
        panic!("Legs is not applicable for this strategy");
    }

    fn break_even(&self) -> Vec<PositiveF64> {
        panic!("Break even is not applicable for this strategy");
    }

    fn max_profit(&self) -> f64 {
        ZERO
    }

    fn max_profit_iter(&mut self) -> f64 {
        self.max_profit()
    }

    fn max_loss(&self) -> f64 {
        ZERO
    }

    fn max_loss_iter(&mut self) -> f64 {
        self.max_loss()
    }

    fn total_cost(&self) -> f64 {
        ZERO
    }

    fn net_premium_received(&self) -> f64 {
        panic!("Net premium received is not applicable");
    }

    fn fees(&self) -> f64 {
        panic!("Fees is not applicable for this strategy");
    }

    fn profit_area(&self) -> f64 {
        ZERO
    }

    fn profit_ratio(&self) -> f64 {
        ZERO
    }

    fn best_ratio(&mut self, _option_chain: &OptionChain, _side: FindOptimalSide) {
        panic!("Best ratio is not applicable for this strategy");
    }

    fn best_area(&mut self, _option_chain: &OptionChain, _side: FindOptimalSide) {
        panic!("Best area is not applicable for this strategy");
    }

    fn validate(&self) -> bool {
        true
    }

    fn best_range_to_show(&self, _step: PositiveF64) -> Option<Vec<PositiveF64>> {
        None
    }

    fn strikes(&self) -> Vec<PositiveF64> {
        self.get_legs()
            .iter()
            .map(|leg| leg.option.strike_price)
            .collect()
    }

    fn max_min_strikes(&self) -> (PositiveF64, PositiveF64) {
        if self.strikes().is_empty() {
            return (PZERO, PZERO);
        }
        let strikes = self.strikes();

        let max = strikes
            .iter()
            .cloned()
            .fold(PositiveF64::new(0.0).unwrap(), PositiveF64::max);
        let min = strikes
            .iter()
            .cloned()
            .fold(PositiveF64::new(f64::INFINITY).unwrap(), PositiveF64::min);

        (min, max)
    }
}

pub(crate) trait Optimizable {
    fn find_optimal(
        &mut self,
        _option_chain: &OptionChain,
        _side: FindOptimalSide,
        _criteria: OptimizationCriteria,
    ) {
        panic!("Find optimal is not applicable for this strategy");
    }
}
