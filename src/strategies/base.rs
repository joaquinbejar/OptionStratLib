/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 21/8/24
******************************************************************************/

use crate::constants::ZERO;
use crate::chains::chain::OptionChain;
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

#[cfg(test)]
mod tests_strategies {
    use super::*;
    use crate::model::position::Position;
    use crate::model::types::{OptionStyle, PositiveF64, Side};
    use crate::model::utils::create_sample_option_simplest;

    #[test]
    fn test_strategy_new() {
        let strategy = Strategy::new(
            "Test Strategy".to_string(),
            StrategyType::Custom,
            "Test Description".to_string(),
        );

        assert_eq!(strategy.name, "Test Strategy");
        assert_eq!(strategy.kind, StrategyType::Custom);
        assert_eq!(strategy.description, "Test Description");
        assert!(strategy.legs.is_empty());
        assert_eq!(strategy.max_profit, None);
        assert_eq!(strategy.max_loss, None);
        assert!(strategy.break_even_points.is_empty());
    }

    struct MockStrategy {
        legs: Vec<Position>,
    }

    impl Strategies for MockStrategy {
        fn add_leg(&mut self, position: Position) {
            self.legs.push(position);
        }

        fn get_legs(&self) -> Vec<Position> {
            self.legs.clone()
        }

        fn break_even(&self) -> Vec<PositiveF64> {
            vec![PositiveF64::new(100.0).unwrap()]
        }

        fn max_profit(&self) -> f64 {
            1000.0
        }

        fn max_loss(&self) -> f64 {
            500.0
        }

        fn total_cost(&self) -> f64 {
            200.0
        }

        fn net_premium_received(&self) -> f64 {
            300.0
        }

        fn fees(&self) -> f64 {
            50.0
        }

        fn profit_area(&self) -> f64 {
            5000.0
        }

        fn profit_ratio(&self) -> f64 {
            2.0
        }
    }

    #[test]
    fn test_strategies_trait() {
        let mut mock_strategy = MockStrategy { legs: Vec::new() };

        // Test add_leg and get_legs
        let option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        let position = Position::new(option, 1.0, Default::default(), 0.0, 0.0);
        mock_strategy.add_leg(position.clone());
        // assert_eq!(mock_strategy.get_legs(), vec![position]);

        // Test other methods
        assert_eq!(
            mock_strategy.break_even(),
            vec![PositiveF64::new(100.0).unwrap()]
        );
        assert_eq!(mock_strategy.max_profit(), 1000.0);
        assert_eq!(mock_strategy.max_loss(), 500.0);
        assert_eq!(mock_strategy.total_cost(), 200.0);
        assert_eq!(mock_strategy.net_premium_received(), 300.0);
        assert_eq!(mock_strategy.fees(), 50.0);
        assert_eq!(mock_strategy.profit_area(), 5000.0);
        assert_eq!(mock_strategy.profit_ratio(), 2.0);
    }

    #[test]
    fn test_strategies_default_methods() {
        struct DefaultStrategy;
        impl Strategies for DefaultStrategy {}

        let strategy = DefaultStrategy;

        assert_eq!(strategy.max_profit(), ZERO);
        assert_eq!(strategy.max_loss(), ZERO);
        assert_eq!(strategy.total_cost(), ZERO);
        assert_eq!(strategy.profit_area(), ZERO);
        assert_eq!(strategy.profit_ratio(), ZERO);
        assert!(strategy.validate());
        assert_eq!(
            strategy.best_range_to_show(PositiveF64::new(1.0).unwrap()),
            None
        );
    }

    #[test]
    #[should_panic(expected = "Add leg is not applicable for this strategy")]
    fn test_strategies_add_leg_panic() {
        struct PanicStrategy;
        impl Strategies for PanicStrategy {}

        let mut strategy = PanicStrategy;
        let option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        let position = Position::new(option, 1.0, Default::default(), 0.0, 0.0);
        strategy.add_leg(position);
    }

    #[test]
    fn test_strategies_strikes_and_max_min_strikes() {
        struct StrikeStrategy {
            legs: Vec<Position>,
        }

        impl Strategies for StrikeStrategy {
            fn get_legs(&self) -> Vec<Position> {
                self.legs.clone()
            }
        }

        let option1 = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        let option2 = create_sample_option_simplest(OptionStyle::Put, Side::Short);

        let strategy = StrikeStrategy {
            legs: vec![
                Position::new(option1, 1.0, Default::default(), 0.0, 0.0),
                Position::new(option2, 1.0, Default::default(), 0.0, 0.0),
            ],
        };

        assert_eq!(
            strategy.strikes(),
            vec![
                PositiveF64::new(100.0).unwrap(),
                PositiveF64::new(100.0).unwrap()
            ]
        );
        assert_eq!(
            strategy.max_min_strikes(),
            (
                PositiveF64::new(100.0).unwrap(),
                PositiveF64::new(100.0).unwrap()
            )
        );
    }
}

#[cfg(test)]
mod tests_strategies_extended {
    use super::*;
    use crate::model::position::Position;
    use crate::model::types::{OptionStyle, PositiveF64, Side};
    use crate::model::utils::create_sample_option_simplest;
    use crate::pos;

    #[test]
    fn test_strategy_enum() {
        assert_ne!(StrategyType::BullCallSpread, StrategyType::BearCallSpread);
        assert_eq!(StrategyType::Custom, StrategyType::Custom);
    }

    #[test]
    fn test_strategy_new_with_legs() {
        let mut strategy = Strategy::new(
            "Test Strategy".to_string(),
            StrategyType::BullCallSpread,
            "Test Description".to_string(),
        );
        let option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        let position = Position::new(option, 1.0, Default::default(), 0.0, 0.0);
        strategy.legs.push(position);

        assert_eq!(strategy.legs.len(), 1);
    }

    #[test]
    #[should_panic(expected = "Legs is not applicable for this strategy")]
    fn test_strategies_get_legs_panic() {
        struct PanicStrategy;
        impl Strategies for PanicStrategy {}

        let strategy = PanicStrategy;
        strategy.get_legs();
    }

    #[test]
    #[should_panic(expected = "Break even is not applicable for this strategy")]
    fn test_strategies_break_even_panic() {
        struct PanicStrategy;
        impl Strategies for PanicStrategy {}

        let strategy = PanicStrategy;
        strategy.break_even();
    }

    #[test]
    #[should_panic(expected = "Net premium received is not applicable")]
    fn test_strategies_net_premium_received_panic() {
        struct PanicStrategy;
        impl Strategies for PanicStrategy {}

        let strategy = PanicStrategy;
        strategy.net_premium_received();
    }

    #[test]
    #[should_panic(expected = "Fees is not applicable for this strategy")]
    fn test_strategies_fees_panic() {
        struct PanicStrategy;
        impl Strategies for PanicStrategy {}

        let strategy = PanicStrategy;
        strategy.fees();
    }

    #[test]
    #[should_panic(expected = "Best ratio is not applicable for this strategy")]
    fn test_strategies_best_ratio_panic() {
        struct PanicStrategy;
        impl Strategies for PanicStrategy {}

        let mut strategy = PanicStrategy;

        let option_chain = OptionChain::new("TEST", pos!(100.0), Default::default());

        strategy.best_ratio(&option_chain, FindOptimalSide::All);
    }

    #[test]
    #[should_panic(expected = "Best area is not applicable for this strategy")]
    fn test_strategies_best_area_panic() {
        struct PanicStrategy;
        impl Strategies for PanicStrategy {}

        let mut strategy = PanicStrategy;

        let option_chain = OptionChain::new("TEST", pos!(100.0), Default::default());

        strategy.best_area(&option_chain, FindOptimalSide::All);
    }

    #[test]
    fn test_strategies_max_profit_iter() {
        struct TestStrategy;
        impl Strategies for TestStrategy {
            fn max_profit(&self) -> f64 {
                100.0
            }
        }

        let mut strategy = TestStrategy;
        assert_eq!(strategy.max_profit_iter(), 100.0);
    }

    #[test]
    fn test_strategies_max_loss_iter() {
        struct TestStrategy;
        impl Strategies for TestStrategy {
            fn max_loss(&self) -> f64 {
                50.0
            }
        }

        let mut strategy = TestStrategy;
        assert_eq!(strategy.max_loss_iter(), 50.0);
    }

    #[test]
    fn test_strategies_empty_strikes() {
        struct EmptyStrategy;
        impl Strategies for EmptyStrategy {
            fn get_legs(&self) -> Vec<Position> {
                vec![]
            }
        }

        let strategy = EmptyStrategy;
        assert_eq!(strategy.strikes(), Vec::<PositiveF64>::new());
        assert_eq!(strategy.max_min_strikes(), (PZERO, PZERO));
    }
}

#[cfg(test)]
mod tests_strategy_type {
    use super::*;

    #[test]
    fn test_strategy_type_equality() {
        assert_eq!(StrategyType::BullCallSpread, StrategyType::BullCallSpread);
        assert_ne!(StrategyType::BullCallSpread, StrategyType::BearCallSpread);
    }

    #[test]
    fn test_strategy_type_clone() {
        let strategy = StrategyType::IronCondor;
        let cloned = strategy.clone();
        assert_eq!(strategy, cloned);
    }

    #[test]
    fn test_strategy_type_debug() {
        let strategy = StrategyType::Straddle;
        let debug_string = format!("{:?}", strategy);
        assert_eq!(debug_string, "Straddle");
    }

    #[test]
    fn test_all_strategy_types() {
        let strategies = [
            StrategyType::BullCallSpread,
            StrategyType::BearCallSpread,
            StrategyType::BullPutSpread,
            StrategyType::BearPutSpread,
            StrategyType::IronCondor,
            StrategyType::Straddle,
            StrategyType::Strangle,
            StrategyType::CoveredCall,
            StrategyType::ProtectivePut,
            StrategyType::Collar,
            StrategyType::LongCall,
            StrategyType::LongPut,
            StrategyType::ShortCall,
            StrategyType::ShortPut,
            StrategyType::PoorMansCoveredCall,
            StrategyType::CallButterfly,
            StrategyType::Custom,
        ];

        for (i, strategy) in strategies.iter().enumerate() {
            for (j, other_strategy) in strategies.iter().enumerate() {
                if i == j {
                    assert_eq!(strategy, other_strategy);
                } else {
                    assert_ne!(strategy, other_strategy);
                }
            }
        }
    }
}
