/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 21/8/24
******************************************************************************/
use crate::chains::chain::{OptionChain, OptionData};
use crate::chains::StrategyLegs;
use crate::constants::{
    STRIKE_PRICE_LOWER_BOUND_MULTIPLIER, STRIKE_PRICE_UPPER_BOUND_MULTIPLIER, ZERO,
};
use crate::model::position::Position;
use crate::model::types::{PositiveF64, PZERO};
use crate::strategies::utils::{calculate_price_range, FindOptimalSide, OptimizationCriteria};
use crate::{pos, spos};
use std::f64;

/// This enum represents different types of trading strategies.
/// Each variant represents a specific strategy type.
#[derive(Clone, Debug, PartialEq)]
pub enum StrategyType {
    BullCallSpread,
    BearCallSpread,
    BullPutSpread,
    BearPutSpread,
    IronCondor,
    IronButterfly,
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

pub trait Strategies: Validable {
    fn get_underlying_price(&self) -> PositiveF64 {
        panic!("Underlying price is not applicable for this strategy");
    }

    fn add_leg(&mut self, _position: Position) {
        panic!("Add leg is not applicable for this strategy");
    }

    fn get_legs(&self) -> Vec<Position> {
        panic!("Legs is not applicable for this strategy");
    }

    // Maintained for Back-compatibility
    fn break_even(&self) -> Vec<PositiveF64> {
        self.get_break_even_points()
    }

    fn max_profit(&self) -> Result<PositiveF64, &str> {
        Err("Max profit is not applicable for this strategy")
    }

    fn max_profit_iter(&mut self) -> PositiveF64 {
        self.max_profit().unwrap_or(PZERO)
    }

    fn max_loss(&self) -> Result<PositiveF64, &str> {
        Err("Max loss is not applicable for this strategy")
    }

    fn max_loss_iter(&mut self) -> PositiveF64 {
        self.max_loss().unwrap_or(PZERO)
    }

    /// Calculates the total cost (premium paid Long - premium get short) of the strategy.
    ///
    /// # Returns
    /// `f64` - The total cost will be zero if the strategy is not applicable.
    ///
    fn total_cost(&self) -> PositiveF64 {
        PZERO
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

    fn range_to_show(&self) -> (PositiveF64, PositiveF64) {
        let mut all_points = self.get_break_even_points();
        let (first_strike, last_strike) = self.max_min_strikes();
        let underlying_price = self.get_underlying_price();

        // Calculate the largest difference from the underlying price
        let max_diff = pos!((last_strike.value() - underlying_price.value())
            .abs()
            .max((first_strike.value() - underlying_price.value()).abs()));

        // Calculate limits in a single step
        all_points
            .push(pos!((underlying_price.value() - max_diff.value()).max(0.0)).min(first_strike));
        all_points.push(pos!(underlying_price.value() + max_diff.value()).max(last_strike));
        all_points.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let start_price = *all_points.first().unwrap() * STRIKE_PRICE_LOWER_BOUND_MULTIPLIER;
        let end_price = *all_points.last().unwrap() * STRIKE_PRICE_UPPER_BOUND_MULTIPLIER;
        (start_price, end_price)
    }

    fn best_range_to_show(&self, step: PositiveF64) -> Option<Vec<PositiveF64>> {
        let (start_price, end_price) = self.range_to_show();
        Some(calculate_price_range(start_price, end_price, step))
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
        let mut min = strikes
            .iter()
            .cloned()
            .fold(PositiveF64::new(f64::INFINITY).unwrap(), PositiveF64::min);
        let mut max = strikes
            .iter()
            .cloned()
            .fold(PositiveF64::new(0.0).unwrap(), PositiveF64::max);

        // If underlying_price is not PZERO, adjust min and max values
        let underlying_price = self.get_underlying_price();
        if underlying_price != PZERO {
            // If min is greater than underlying_price, use underlying_price as min
            if min > underlying_price {
                min = underlying_price;
            }
            // If underlying_price is greater than max, use underlying_price as max
            if underlying_price > max {
                max = underlying_price;
            }
        }

        (min, max)
    }

    fn get_break_even_points(&self) -> Vec<PositiveF64> {
        panic!("Break even points is not applicable for this strategy");
    }

    /// Calculates the range of profit based on break-even points for any strategy that implements
    /// the `Strategies` trait. Break-even points are determined using the `get_break_even_points` method.
    ///
    /// # Returns
    ///
    /// * `None` - if there are less than two break-even points.
    /// * `Some(PositiveF64)` - the difference between the highest and lowest break-even points,
    ///   or the difference between the first and second break-even points if there are exactly two.
    ///
    fn range_of_profit(&self) -> Option<PositiveF64> {
        match self.get_break_even_points().len() {
            0 => None,
            1 => spos!(f64::INFINITY),
            2 => Some(self.get_break_even_points()[1] - self.get_break_even_points()[0]),
            _ => {
                // sort break even points and then get last minus first
                let mut break_even_points = self.get_break_even_points();
                break_even_points.sort_by(|a, b| a.partial_cmp(b).unwrap());
                Some(*break_even_points.last().unwrap() - *break_even_points.first().unwrap())
            }
        }
    }
}

pub trait Validable {
    fn validate(&self) -> bool {
        panic!("Validate is not applicable for this strategy");
    }
}

pub trait Optimizable: Validable + Strategies {
    type Strategy: Strategies;

    fn best_ratio(&mut self, option_chain: &OptionChain, side: FindOptimalSide) {
        self.find_optimal(option_chain, side, OptimizationCriteria::Ratio);
    }

    fn best_area(&mut self, option_chain: &OptionChain, side: FindOptimalSide) {
        self.find_optimal(option_chain, side, OptimizationCriteria::Area);
    }

    fn find_optimal(
        &mut self,
        _option_chain: &OptionChain,
        _side: FindOptimalSide,
        _criteria: OptimizationCriteria,
    ) {
        panic!("Find optimal is not applicable for this strategy");
    }

    fn is_valid_short_option(&self, option: &OptionData, side: &FindOptimalSide) -> bool {
        self.is_valid_long_option(option, side)
    }

    fn is_valid_long_option(&self, option: &OptionData, side: &FindOptimalSide) -> bool {
        match side {
            FindOptimalSide::Upper => option.strike_price >= self.get_underlying_price(),
            FindOptimalSide::Lower => option.strike_price <= self.get_underlying_price(),
            FindOptimalSide::All => true,
            FindOptimalSide::Range(start, end) => {
                option.strike_price >= *start && option.strike_price <= *end
            }
        }
    }

    fn are_valid_prices(&self, legs: &StrategyLegs) -> bool {
        // by default, we assume Options are one long call and one short call
        let (long, short) = match legs {
            StrategyLegs::TwoLegs { first, second } => (first, second),
            _ => panic!("Invalid number of legs for this strategy"),
        };
        long.call_ask.unwrap_or(PZERO) > PZERO && short.call_bid.unwrap_or(PZERO) > PZERO
    }

    fn create_strategy(&self, _chain: &OptionChain, _legs: &StrategyLegs) -> Self::Strategy {
        panic!("Create strategy is not applicable for this strategy");
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

    impl Validable for MockStrategy {}

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

        fn max_profit(&self) -> Result<PositiveF64, &str> {
            Ok(pos!(1000.0))
        }

        fn max_loss(&self) -> Result<PositiveF64, &str> {
            Ok(pos!(500.0))
        }

        fn total_cost(&self) -> PositiveF64 {
            pos!(200.0)
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
        assert_eq!(mock_strategy.max_profit().unwrap_or(PZERO), 1000.0);
        assert_eq!(mock_strategy.max_loss().unwrap_or(PZERO), 500.0);
        assert_eq!(mock_strategy.total_cost(), 200.0);
        assert_eq!(mock_strategy.net_premium_received(), 300.0);
        assert_eq!(mock_strategy.fees(), 50.0);
        assert_eq!(mock_strategy.profit_area(), 5000.0);
        assert_eq!(mock_strategy.profit_ratio(), 2.0);
    }

    #[test]
    fn test_strategies_default_methods() {
        struct DefaultStrategy;
        impl Validable for DefaultStrategy {
            fn validate(&self) -> bool {
                true
            }
        }
        impl Strategies for DefaultStrategy {}

        let strategy = DefaultStrategy;

        assert_eq!(strategy.max_profit().unwrap_or(PZERO), ZERO);
        assert_eq!(strategy.max_loss().unwrap_or(PZERO), ZERO);
        assert_eq!(strategy.total_cost(), ZERO);
        assert_eq!(strategy.profit_area(), ZERO);
        assert_eq!(strategy.profit_ratio(), ZERO);
        assert!(strategy.validate());
    }

    #[test]
    #[should_panic(expected = "Add leg is not applicable for this strategy")]
    fn test_strategies_add_leg_panic() {
        struct PanicStrategy;
        impl Validable for PanicStrategy {}
        impl Strategies for PanicStrategy {}

        let mut strategy = PanicStrategy;
        let option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        let position = Position::new(option, 1.0, Default::default(), 0.0, 0.0);
        strategy.add_leg(position);
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
        impl Validable for PanicStrategy {}
        impl Strategies for PanicStrategy {}

        let strategy = PanicStrategy;
        strategy.get_legs();
    }

    #[test]
    #[should_panic(expected = "Break even points is not applicable for this strategy")]
    fn test_strategies_break_even_panic() {
        struct PanicStrategy;
        impl Validable for PanicStrategy {}
        impl Strategies for PanicStrategy {}

        let strategy = PanicStrategy;
        strategy.break_even();
    }

    #[test]
    #[should_panic(expected = "Net premium received is not applicable")]
    fn test_strategies_net_premium_received_panic() {
        struct PanicStrategy;
        impl Validable for PanicStrategy {}
        impl Strategies for PanicStrategy {}

        let strategy = PanicStrategy;
        strategy.net_premium_received();
    }

    #[test]
    #[should_panic(expected = "Fees is not applicable for this strategy")]
    fn test_strategies_fees_panic() {
        struct PanicStrategy;
        impl Validable for PanicStrategy {}
        impl Strategies for PanicStrategy {}

        let strategy = PanicStrategy;
        strategy.fees();
    }

    #[test]
    fn test_strategies_max_profit_iter() {
        struct TestStrategy;
        impl Validable for TestStrategy {}
        impl Strategies for TestStrategy {
            fn max_profit(&self) -> Result<PositiveF64, &str> {
                Ok(pos!(100.0))
            }
        }

        let mut strategy = TestStrategy;
        assert_eq!(strategy.max_profit_iter(), 100.0);
    }

    #[test]
    fn test_strategies_max_loss_iter() {
        struct TestStrategy;
        impl Validable for TestStrategy {}
        impl Strategies for TestStrategy {
            fn max_loss(&self) -> Result<PositiveF64, &str> {
                Ok(pos!(50.0))
            }
        }

        let mut strategy = TestStrategy;
        assert_eq!(strategy.max_loss_iter(), 50.0);
    }

    #[test]
    fn test_strategies_empty_strikes() {
        struct EmptyStrategy;
        impl Validable for EmptyStrategy {}
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

#[cfg(test)]
mod tests_max_min_strikes {
    use super::*;
    use crate::pos;

    struct TestStrategy {
        strikes: Vec<PositiveF64>,
        underlying_price: PositiveF64,
    }

    impl TestStrategy {
        fn new(strikes: Vec<PositiveF64>, underlying_price: PositiveF64) -> Self {
            Self {
                strikes,
                underlying_price,
            }
        }
    }

    impl Validable for TestStrategy {
        fn validate(&self) -> bool {
            true
        }
    }

    impl Strategies for TestStrategy {
        fn strikes(&self) -> Vec<PositiveF64> {
            self.strikes.clone()
        }

        fn get_underlying_price(&self) -> PositiveF64 {
            self.underlying_price
        }

        // Implement other required methods with default behavior
        fn add_leg(&mut self, _position: Position) {}
        fn break_even(&self) -> Vec<PositiveF64> {
            vec![]
        }
        fn max_profit(&self) -> Result<PositiveF64, &str> {
            Ok(PZERO)
        }
        fn max_loss(&self) -> Result<PositiveF64, &str> {
            Ok(PZERO)
        }
        fn total_cost(&self) -> PositiveF64 {
            PZERO
        }
        fn net_premium_received(&self) -> f64 {
            0.0
        }
        fn fees(&self) -> f64 {
            0.0
        }
        fn profit_area(&self) -> f64 {
            0.0
        }
        fn profit_ratio(&self) -> f64 {
            0.0
        }
        fn best_range_to_show(&self, _step: PositiveF64) -> Option<Vec<PositiveF64>> {
            None
        }
    }

    #[test]
    fn test_empty_strikes() {
        let strategy = TestStrategy::new(vec![], PZERO);
        assert_eq!(strategy.max_min_strikes(), (PZERO, PZERO));
    }

    #[test]
    fn test_single_strike() {
        let strike = pos!(100.0);
        let strategy = TestStrategy::new(vec![strike], PZERO);
        assert_eq!(strategy.max_min_strikes(), (strike, strike));
    }

    #[test]
    fn test_multiple_strikes_no_underlying() {
        let strikes = vec![pos!(90.0), pos!(100.0), pos!(110.0)];
        let strategy = TestStrategy::new(strikes.clone(), PZERO);
        assert_eq!(
            strategy.max_min_strikes(),
            (*strikes.first().unwrap(), *strikes.last().unwrap())
        );
    }

    #[test]
    fn test_underlying_price_between_strikes() {
        let strikes = vec![pos!(90.0), pos!(110.0)];
        let underlying = pos!(100.0);
        let strategy = TestStrategy::new(strikes, underlying);
        assert_eq!(strategy.max_min_strikes(), (pos!(90.0), pos!(110.0)));
    }

    #[test]
    fn test_underlying_price_below_min_strike() {
        let strikes = vec![pos!(100.0), pos!(110.0)];
        let underlying = pos!(90.0);
        let strategy = TestStrategy::new(strikes, underlying);
        assert_eq!(strategy.max_min_strikes(), (pos!(90.0), pos!(110.0)));
    }

    #[test]
    fn test_underlying_price_above_max_strike() {
        let strikes = vec![pos!(90.0), pos!(100.0)];
        let underlying = pos!(110.0);
        let strategy = TestStrategy::new(strikes, underlying);
        assert_eq!(strategy.max_min_strikes(), (pos!(90.0), pos!(110.0)));
    }

    #[test]
    fn test_strikes_with_duplicates() {
        let strikes = vec![pos!(100.0), pos!(100.0), pos!(110.0)];
        let strategy = TestStrategy::new(strikes, PZERO);
        assert_eq!(strategy.max_min_strikes(), (pos!(100.0), pos!(110.0)));
    }

    #[test]
    fn test_underlying_equals_min_strike() {
        let strikes = vec![pos!(100.0), pos!(110.0)];
        let underlying = pos!(100.0);
        let strategy = TestStrategy::new(strikes, underlying);
        assert_eq!(strategy.max_min_strikes(), (pos!(100.0), pos!(110.0)));
    }

    #[test]
    fn test_underlying_equals_max_strike() {
        let strikes = vec![pos!(90.0), pos!(100.0)];
        let underlying = pos!(100.0);
        let strategy = TestStrategy::new(strikes, underlying);
        assert_eq!(strategy.max_min_strikes(), (pos!(90.0), pos!(100.0)));
    }

    #[test]
    fn test_unordered_strikes() {
        let strikes = vec![pos!(110.0), pos!(90.0), pos!(100.0)];
        let strategy = TestStrategy::new(strikes, PZERO);
        assert_eq!(strategy.max_min_strikes(), (pos!(90.0), pos!(110.0)));
    }
}

#[cfg(test)]
mod tests_best_range_to_show {
    use super::*;
    use crate::pos;

    struct TestStrategy {
        underlying_price: PositiveF64,
        break_even_points: Vec<PositiveF64>,
        strikes: Vec<PositiveF64>,
    }

    impl TestStrategy {
        fn new(
            underlying_price: PositiveF64,
            break_even_points: Vec<PositiveF64>,
            strikes: Vec<PositiveF64>,
        ) -> Self {
            Self {
                underlying_price,
                break_even_points,
                strikes,
            }
        }
    }

    impl Validable for TestStrategy {}

    impl Strategies for TestStrategy {
        fn get_underlying_price(&self) -> PositiveF64 {
            self.underlying_price
        }

        fn get_break_even_points(&self) -> Vec<PositiveF64> {
            self.break_even_points.clone()
        }

        fn strikes(&self) -> Vec<PositiveF64> {
            self.strikes.clone()
        }
    }

    #[test]
    fn test_basic_range_with_step() {
        let strategy = TestStrategy::new(
            pos!(100.0),
            vec![pos!(90.0), pos!(110.0)],
            vec![pos!(95.0), pos!(105.0)],
        );
        let range = strategy.best_range_to_show(pos!(5.0)).unwrap();
        assert!(!range.is_empty());
        assert_eq!(range[1] - range[0], pos!(5.0));
    }

    #[test]
    fn test_range_with_small_step() {
        let strategy = TestStrategy::new(
            pos!(100.0),
            vec![pos!(95.0), pos!(105.0)],
            vec![pos!(97.0), pos!(103.0)],
        );
        let range = strategy.best_range_to_show(pos!(1.0)).unwrap();
        assert!(!range.is_empty());
        assert_eq!(range[1] - range[0], pos!(1.0));
    }

    #[test]
    fn test_range_boundaries() {
        let strategy = TestStrategy::new(
            pos!(100.0),
            vec![pos!(90.0), pos!(110.0)],
            vec![pos!(95.0), pos!(105.0)],
        );
        let range = strategy.best_range_to_show(pos!(5.0)).unwrap();
        assert!(range.first().unwrap() < &pos!(90.0));
        assert!(range.last().unwrap() > &pos!(110.0));
    }

    #[test]
    fn test_range_step_size() {
        let strategy = TestStrategy::new(
            pos!(100.0),
            vec![pos!(90.0), pos!(110.0)],
            vec![pos!(95.0), pos!(105.0)],
        );
        let step = pos!(5.0);
        let range = strategy.best_range_to_show(step).unwrap();

        for i in 1..range.len() {
            assert_eq!(range[i] - range[i - 1], step);
        }
    }

    #[test]
    fn test_range_includes_underlying() {
        let underlying_price = pos!(100.0);
        let strategy = TestStrategy::new(
            underlying_price,
            vec![pos!(90.0), pos!(110.0)],
            vec![pos!(95.0), pos!(105.0)],
        );
        let range = strategy.best_range_to_show(pos!(5.0)).unwrap();

        assert!(range.iter().any(|&price| price <= underlying_price));
        assert!(range.iter().any(|&price| price >= underlying_price));
    }

    #[test]
    fn test_range_with_extreme_values() {
        let strategy = TestStrategy::new(
            pos!(100.0),
            vec![pos!(50.0), pos!(150.0)],
            vec![pos!(75.0), pos!(125.0)],
        );
        let range = strategy.best_range_to_show(pos!(10.0)).unwrap();

        assert!(range.first().unwrap() <= &pos!(50.0));
        assert!(range.last().unwrap() >= &pos!(150.0));
    }
}

#[cfg(test)]
mod tests_range_to_show {
    use super::*;
    use crate::pos;

    struct TestStrategy {
        underlying_price: PositiveF64,
        break_even_points: Vec<PositiveF64>,
        strikes: Vec<PositiveF64>,
    }

    impl TestStrategy {
        fn new(
            underlying_price: PositiveF64,
            break_even_points: Vec<PositiveF64>,
            strikes: Vec<PositiveF64>,
        ) -> Self {
            Self {
                underlying_price,
                break_even_points,
                strikes,
            }
        }
    }

    impl Validable for TestStrategy {}

    impl Strategies for TestStrategy {
        fn get_underlying_price(&self) -> PositiveF64 {
            self.underlying_price
        }

        fn get_break_even_points(&self) -> Vec<PositiveF64> {
            self.break_even_points.clone()
        }

        fn strikes(&self) -> Vec<PositiveF64> {
            self.strikes.clone()
        }
    }

    #[test]
    fn test_basic_range() {
        let strategy = TestStrategy::new(
            pos!(100.0),
            vec![pos!(90.0), pos!(110.0)],
            vec![pos!(95.0), pos!(105.0)],
        );
        let (start, end) = strategy.range_to_show();
        assert!(start < pos!(90.0));
        assert!(end > pos!(110.0));
    }

    #[test]
    fn test_range_with_far_strikes() {
        let strategy = TestStrategy::new(
            pos!(100.0),
            vec![pos!(90.0), pos!(110.0)],
            vec![pos!(80.0), pos!(120.0)],
        );
        let (start, end) = strategy.range_to_show();
        assert!(start < pos!(80.0));
        assert!(end > pos!(120.0));
    }

    #[test]
    fn test_range_with_underlying_outside_strikes() {
        let strategy = TestStrategy::new(
            pos!(150.0),
            vec![pos!(90.0), pos!(110.0)],
            vec![pos!(95.0), pos!(105.0)],
        );
        let (_start, end) = strategy.range_to_show();
        assert!(end > pos!(150.0));
    }
}

#[cfg(test)]
mod tests_range_of_profit {
    use super::*;
    use crate::pos;

    struct TestStrategy {
        break_even_points: Vec<PositiveF64>,
    }

    impl TestStrategy {
        fn new(break_even_points: Vec<PositiveF64>) -> Self {
            Self { break_even_points }
        }
    }

    impl Validable for TestStrategy {}

    impl Strategies for TestStrategy {
        fn get_break_even_points(&self) -> Vec<PositiveF64> {
            self.break_even_points.clone()
        }
    }

    #[test]
    fn test_no_break_even_points() {
        let strategy = TestStrategy::new(vec![]);
        assert_eq!(strategy.range_of_profit(), None);
    }

    #[test]
    fn test_single_break_even_point() {
        let strategy = TestStrategy::new(vec![pos!(100.0)]);
        assert_eq!(strategy.range_of_profit(), Some(pos!(f64::INFINITY)));
    }

    #[test]
    fn test_two_break_even_points() {
        let strategy = TestStrategy::new(vec![pos!(90.0), pos!(110.0)]);
        assert_eq!(strategy.range_of_profit(), Some(pos!(20.0)));
    }

    #[test]
    fn test_multiple_break_even_points() {
        let strategy = TestStrategy::new(vec![pos!(80.0), pos!(100.0), pos!(120.0)]);
        assert_eq!(strategy.range_of_profit(), Some(pos!(40.0)));
    }

    #[test]
    fn test_unordered_break_even_points() {
        let strategy = TestStrategy::new(vec![pos!(120.0), pos!(80.0), pos!(100.0)]);
        assert_eq!(strategy.range_of_profit(), Some(pos!(40.0)));
    }
}
