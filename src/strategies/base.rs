/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 21/8/24
******************************************************************************/
use crate::chains::chain::{OptionChain, OptionData};
use crate::chains::utils::OptionDataGroup;
use crate::chains::StrategyLegs;
use crate::constants::{STRIKE_PRICE_LOWER_BOUND_MULTIPLIER, STRIKE_PRICE_UPPER_BOUND_MULTIPLIER};
use crate::error::position::PositionError;
use crate::error::strategies::StrategyError;
use crate::model::position::Position;
use crate::strategies::utils::{calculate_price_range, FindOptimalSide, OptimizationCriteria};
use crate::Positive;
use rust_decimal::Decimal;
use std::f64;
use tracing::error;

/// This enum represents different types of trading strategies.
/// Each variant represents a specific strategy type.
#[derive(Clone, Debug, PartialEq)]
pub enum StrategyType {
    BullCallSpread,
    BearCallSpread,
    BullPutSpread,
    BearPutSpread,
    LongButterflySpread,
    ShortButterflySpread,
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
    pub break_even_points: Vec<Positive>,
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

pub trait Strategies: Validable + Positionable {
    fn get_underlying_price(&self) -> Positive {
        panic!("Underlying price is not applicable for this strategy");
    }

    // Maintained for Back-compatibility
    fn break_even(&self) -> Vec<Positive> {
        self.get_break_even_points()
    }

    fn max_profit(&self) -> Result<Positive, StrategyError> {
        Err(StrategyError::operation_not_supported(
            "max_profit",
            std::any::type_name::<Self>(),
        ))
    }

    fn max_profit_iter(&mut self) -> Positive {
        self.max_profit().unwrap_or(Positive::ZERO)
    }

    fn max_loss(&self) -> Result<Positive, StrategyError> {
        Err(StrategyError::operation_not_supported(
            "max_loss",
            std::any::type_name::<Self>(),
        ))
    }

    fn max_loss_iter(&mut self) -> Positive {
        self.max_loss().unwrap_or(Positive::ZERO)
    }

    /// Calculates the total cost (premium paid Long - premium get short) of the strategy.
    ///
    /// # Returns
    /// `f64` - The total cost will be zero if the strategy is not applicable.
    ///
    fn total_cost(&self) -> Positive {
        Positive::ZERO
    }

    fn net_premium_received(&self) -> Result<Decimal, StrategyError> {
        Err(StrategyError::operation_not_supported(
            "net_premium_received",
            std::any::type_name::<Self>(),
        ))
    }

    fn fees(&self) -> Result<Decimal, StrategyError> {
        Err(StrategyError::operation_not_supported(
            "fees",
            std::any::type_name::<Self>(),
        ))
    }

    fn profit_area(&self) -> Result<Decimal, StrategyError> {
        Err(StrategyError::operation_not_supported(
            "profit_area",
            std::any::type_name::<Self>(),
        ))
    }

    fn profit_ratio(&self) -> Result<Decimal, StrategyError> {
        Err(StrategyError::operation_not_supported(
            "profit_ratio",
            std::any::type_name::<Self>(),
        ))
    }

    fn range_to_show(&self) -> (Positive, Positive) {
        let mut all_points = self.get_break_even_points();
        let (first_strike, last_strike) = self.max_min_strikes();
        let underlying_price = self.get_underlying_price();

        // Calculate the largest difference from the underlying price
        let max_diff = (last_strike.value() - underlying_price.value())
            .abs()
            .max((first_strike.value() - underlying_price.value()).abs());

        // Calculate limits in a single step
        all_points.push(
            (underlying_price - max_diff)
                .max(Positive::ZERO)
                .min(first_strike),
        );
        all_points.push((underlying_price + max_diff).max(last_strike));
        all_points.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let start_price = *all_points.first().unwrap() * STRIKE_PRICE_LOWER_BOUND_MULTIPLIER;
        let end_price = *all_points.last().unwrap() * STRIKE_PRICE_UPPER_BOUND_MULTIPLIER;
        (start_price, end_price)
    }

    fn best_range_to_show(&self, step: Positive) -> Option<Vec<Positive>> {
        let (start_price, end_price) = self.range_to_show();
        Some(calculate_price_range(start_price, end_price, step))
    }

    fn strikes(&self) -> Vec<Positive> {
        self.get_positions()
            .unwrap_or_default()
            .iter()
            .map(|leg| leg.option.strike_price)
            .collect()
    }

    fn max_min_strikes(&self) -> (Positive, Positive) {
        if self.strikes().is_empty() {
            return (Positive::ZERO, Positive::ZERO);
        }

        let strikes = self.strikes();
        let mut min = strikes
            .iter()
            .cloned()
            .fold(Positive::INFINITY, Positive::min);
        let mut max = strikes.iter().cloned().fold(Positive::ZERO, Positive::max);

        // If underlying_price is not Positive::ZERO, adjust min and max values
        let underlying_price = self.get_underlying_price();
        if underlying_price != Positive::ZERO {
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

    fn get_break_even_points(&self) -> Vec<Positive> {
        panic!("Break even points is not applicable for this strategy");
    }

    /// Calculates the range of profit based on break-even points for any strategy that implements
    /// the `Strategies` trait. Break-even points are determined using the `get_break_even_points` method.
    ///
    /// # Returns
    ///
    /// * `None` - if there are less than two break-even points.
    /// * `Some(Positive)` - the difference between the highest and lowest break-even points,
    ///   or the difference between the first and second break-even points if there are exactly two.
    ///
    fn range_of_profit(&self) -> Option<Positive> {
        match self.get_break_even_points().len() {
            0 => None,
            1 => Some(Positive::INFINITY),
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

    /// Filters and generates combinations of options data from the given `OptionChain`.
    ///
    /// # Parameters
    /// - `&self`: A reference to the current object/context that holds the filtering logic or required data.
    /// - `_option_chain`: A reference to an `OptionChain` object that contains relevant financial information
    ///   such as options data, underlying price, and expiration date.
    /// - `_side`: A `FindOptimalSide` value that specifies the filtering strategy for finding combinations of
    ///   options. It can specify:
    ///     - `Upper`: Consider options higher than a certain threshold.
    ///     - `Lower`: Consider options lower than a certain threshold.
    ///     - `All`: Include all options.
    ///     - `Range(start, end)`: Consider options within a specified range.
    ///
    /// # Returns
    /// - An iterator that yields `OptionDataGroup` items. These items represent combinations of options data filtered
    ///   based on the given criteria. The `OptionDataGroup` can represent combinations of 2, 3, 4, or any number
    ///   of options depending on the grouping logic.
    ///
    /// **Note**:
    /// - The current implementation returns an empty iterator (`std::iter::empty()`) as a placeholder.
    /// - You may modify this method to implement the actual filtering and combination logic based on the
    ///   provided `OptionChain` and `FindOptimalSide` criteria.
    ///
    /// # See Also
    /// - `FindOptimalSide` for the strategy enumeration.
    /// - `OptionDataGroup` for the structure of grouped combinations.
    /// - `OptionChain` for the full structure being processed.
    fn filter_combinations<'a>(
        &'a self,
        _option_chain: &'a OptionChain,
        _side: FindOptimalSide,
    ) -> impl Iterator<Item = OptionDataGroup<'a>> {
        error!("Filter combinations is not applicable for this strategy");
        std::iter::empty()
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
        long.call_ask.unwrap_or(Positive::ZERO) > Positive::ZERO
            && short.call_bid.unwrap_or(Positive::ZERO) > Positive::ZERO
    }

    fn create_strategy(&self, _chain: &OptionChain, _legs: &StrategyLegs) -> Self::Strategy {
        panic!("Create strategy is not applicable for this strategy");
    }
}

pub trait Positionable {
    fn add_position(&mut self, _position: &Position) -> Result<(), PositionError> {
        Err(PositionError::unsupported_operation(
            std::any::type_name::<Self>(),
            "add_position",
        ))
    }

    fn get_positions(&self) -> Result<Vec<&Position>, PositionError> {
        Err(PositionError::unsupported_operation(
            std::any::type_name::<Self>(),
            "get_positions",
        ))
    }
}

#[cfg(test)]
mod tests_strategies {
    use super::*;
    use crate::f2p;
    use crate::model::position::Position;
    use crate::model::types::{OptionStyle, Side};
    use crate::model::utils::create_sample_option_simplest;
    use rust_decimal_macros::dec;

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

    impl Positionable for MockStrategy {
        fn add_position(&mut self, position: &Position) -> Result<(), PositionError> {
            self.legs.push(position.clone());
            Ok(())
        }

        fn get_positions(&self) -> Result<Vec<&Position>, PositionError> {
            Ok(self.legs.iter().collect())
        }
    }

    impl Strategies for MockStrategy {
        fn break_even(&self) -> Vec<Positive> {
            vec![Positive::HUNDRED]
        }

        fn max_profit(&self) -> Result<Positive, StrategyError> {
            Ok(Positive::THOUSAND)
        }

        fn max_loss(&self) -> Result<Positive, StrategyError> {
            Ok(f2p!(500.0))
        }

        fn total_cost(&self) -> Positive {
            f2p!(200.0)
        }

        fn net_premium_received(&self) -> Result<Decimal, StrategyError> {
            Ok(dec!(300.0))
        }

        fn fees(&self) -> Result<Decimal, StrategyError> {
            Ok(dec!(50.0))
        }

        fn profit_area(&self) -> Result<Decimal, StrategyError> {
            Ok(dec!(5000.0))
        }

        fn profit_ratio(&self) -> Result<Decimal, StrategyError> {
            Ok(dec!(2.0))
        }
    }

    #[test]
    fn test_strategies_trait() {
        let mut mock_strategy = MockStrategy { legs: Vec::new() };

        // Test add_leg and get_legs
        let option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        let position = Position::new(option, 1.0, Default::default(), 0.0, 0.0);
        mock_strategy
            .add_position(&position.clone())
            .expect("Error adding position");

        // Test other methods
        assert_eq!(mock_strategy.break_even(), vec![Positive::HUNDRED]);
        assert_eq!(mock_strategy.max_profit().unwrap_or(Positive::ZERO), 1000.0);
        assert_eq!(mock_strategy.max_loss().unwrap_or(Positive::ZERO), 500.0);
        assert_eq!(mock_strategy.total_cost(), 200.0);
        assert_eq!(mock_strategy.net_premium_received().unwrap(), dec!(300.0));
        assert_eq!(mock_strategy.fees().unwrap(), dec!(50.0));
        assert_eq!(mock_strategy.profit_area().unwrap(), dec!(5000.0));
        assert_eq!(mock_strategy.profit_ratio().unwrap(), dec!(2.0));
    }

    #[test]
    fn test_strategies_default_methods() {
        struct DefaultStrategy;
        impl Validable for DefaultStrategy {
            fn validate(&self) -> bool {
                true
            }
        }
        impl Positionable for DefaultStrategy {}
        impl Strategies for DefaultStrategy {}

        let strategy = DefaultStrategy;

        assert_eq!(
            strategy.max_profit().unwrap_or(Positive::ZERO),
            Positive::ZERO
        );
        assert_eq!(
            strategy.max_loss().unwrap_or(Positive::ZERO),
            Positive::ZERO
        );
        assert_eq!(strategy.total_cost(), Positive::ZERO);
        assert!(strategy.profit_area().is_err());
        assert!(strategy.profit_ratio().is_err());
        assert!(strategy.validate());
    }

    #[test]
    fn test_strategies_add_leg_panic() {
        struct PanicStrategy;
        impl Validable for PanicStrategy {}
        impl Positionable for PanicStrategy {}
        impl Strategies for PanicStrategy {}

        let mut strategy = PanicStrategy;
        let option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        let position = Position::new(option, 1.0, Default::default(), 0.0, 0.0);
        assert!(strategy.add_position(&position).is_err());
    }
}

#[cfg(test)]
mod tests_strategies_extended {
    use super::*;
    use crate::f2p;
    use crate::model::position::Position;
    use crate::model::types::{OptionStyle, Side};
    use crate::model::utils::create_sample_option_simplest;

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
    fn test_strategies_get_legs_panic() {
        struct PanicStrategy;
        impl Validable for PanicStrategy {}
        impl Positionable for PanicStrategy {}
        impl Strategies for PanicStrategy {}

        let strategy = PanicStrategy;
        assert!(strategy.get_positions().is_err());
    }

    #[test]
    #[should_panic(expected = "Break even points is not applicable for this strategy")]
    fn test_strategies_break_even_panic() {
        struct PanicStrategy;
        impl Validable for PanicStrategy {}
        impl Positionable for PanicStrategy {}
        impl Strategies for PanicStrategy {}

        let strategy = PanicStrategy;
        strategy.break_even();
    }

    #[test]
    fn test_strategies_net_premium_received_panic() {
        struct PanicStrategy;
        impl Validable for PanicStrategy {}
        impl Positionable for PanicStrategy {}
        impl Strategies for PanicStrategy {}

        let strategy = PanicStrategy;
        assert!(strategy.net_premium_received().is_err());
    }

    #[test]
    fn test_strategies_fees_panic() {
        struct PanicStrategy;
        impl Validable for PanicStrategy {}
        impl Positionable for PanicStrategy {}
        impl Strategies for PanicStrategy {}

        let strategy = PanicStrategy;
        assert!(strategy.fees().is_err());
    }

    #[test]
    fn test_strategies_max_profit_iter() {
        struct TestStrategy;
        impl Validable for TestStrategy {}
        impl Positionable for TestStrategy {}
        impl Strategies for TestStrategy {
            fn max_profit(&self) -> Result<Positive, StrategyError> {
                Ok(f2p!(100.0))
            }
        }

        let mut strategy = TestStrategy;
        assert_eq!(strategy.max_profit_iter(), 100.0);
    }

    #[test]
    fn test_strategies_max_loss_iter() {
        struct TestStrategy;
        impl Validable for TestStrategy {}
        impl Positionable for TestStrategy {}
        impl Strategies for TestStrategy {
            fn max_loss(&self) -> Result<Positive, StrategyError> {
                Ok(f2p!(50.0))
            }
        }

        let mut strategy = TestStrategy;
        assert_eq!(strategy.max_loss_iter(), 50.0);
    }

    #[test]
    fn test_strategies_empty_strikes() {
        struct EmptyStrategy;
        impl Validable for EmptyStrategy {}
        impl Positionable for EmptyStrategy {
            fn get_positions(&self) -> Result<Vec<&Position>, PositionError> {
                Ok(vec![])
            }
        }
        impl Strategies for EmptyStrategy {}

        let strategy = EmptyStrategy;
        assert_eq!(strategy.strikes(), Vec::<Positive>::new());
        assert_eq!(strategy.max_min_strikes(), (Positive::ZERO, Positive::ZERO));
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
    use crate::f2p;

    struct TestStrategy {
        strikes: Vec<Positive>,
        underlying_price: Positive,
    }

    impl TestStrategy {
        fn new(strikes: Vec<Positive>, underlying_price: Positive) -> Self {
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

    impl Positionable for TestStrategy {}

    impl Strategies for TestStrategy {
        fn get_underlying_price(&self) -> Positive {
            self.underlying_price
        }
        fn break_even(&self) -> Vec<Positive> {
            vec![]
        }
        fn max_profit(&self) -> Result<Positive, StrategyError> {
            Ok(Positive::ZERO)
        }
        fn max_loss(&self) -> Result<Positive, StrategyError> {
            Ok(Positive::ZERO)
        }
        fn total_cost(&self) -> Positive {
            Positive::ZERO
        }
        fn net_premium_received(&self) -> Result<Decimal, StrategyError> {
            Ok(Decimal::ZERO)
        }
        fn fees(&self) -> Result<Decimal, StrategyError> {
            Ok(Decimal::ZERO)
        }
        fn profit_area(&self) -> Result<Decimal, StrategyError> {
            Ok(Decimal::ZERO)
        }
        fn profit_ratio(&self) -> Result<Decimal, StrategyError> {
            Ok(Decimal::ZERO)
        }
        fn best_range_to_show(&self, _step: Positive) -> Option<Vec<Positive>> {
            None
        }
        fn strikes(&self) -> Vec<Positive> {
            self.strikes.clone()
        }
    }

    #[test]
    fn test_empty_strikes() {
        let strategy = TestStrategy::new(vec![], Positive::ZERO);
        assert_eq!(strategy.max_min_strikes(), (Positive::ZERO, Positive::ZERO));
    }

    #[test]
    fn test_single_strike() {
        let strike = f2p!(100.0);
        let strategy = TestStrategy::new(vec![strike], Positive::ZERO);
        assert_eq!(strategy.max_min_strikes(), (strike, strike));
    }

    #[test]
    fn test_multiple_strikes_no_underlying() {
        let strikes = vec![f2p!(90.0), f2p!(100.0), f2p!(110.0)];
        let strategy = TestStrategy::new(strikes.clone(), Positive::ZERO);
        assert_eq!(
            strategy.max_min_strikes(),
            (*strikes.first().unwrap(), *strikes.last().unwrap())
        );
    }

    #[test]
    fn test_underlying_price_between_strikes() {
        let strikes = vec![f2p!(90.0), f2p!(110.0)];
        let underlying = f2p!(100.0);
        let strategy = TestStrategy::new(strikes, underlying);
        assert_eq!(strategy.max_min_strikes(), (f2p!(90.0), f2p!(110.0)));
    }

    #[test]
    fn test_underlying_price_below_min_strike() {
        let strikes = vec![f2p!(100.0), f2p!(110.0)];
        let underlying = f2p!(90.0);
        let strategy = TestStrategy::new(strikes, underlying);
        assert_eq!(strategy.max_min_strikes(), (f2p!(90.0), f2p!(110.0)));
    }

    #[test]
    fn test_underlying_price_above_max_strike() {
        let strikes = vec![f2p!(90.0), f2p!(100.0)];
        let underlying = f2p!(110.0);
        let strategy = TestStrategy::new(strikes, underlying);
        assert_eq!(strategy.max_min_strikes(), (f2p!(90.0), f2p!(110.0)));
    }

    #[test]
    fn test_strikes_with_duplicates() {
        let strikes = vec![f2p!(100.0), f2p!(100.0), f2p!(110.0)];
        let strategy = TestStrategy::new(strikes, Positive::ZERO);
        assert_eq!(strategy.max_min_strikes(), (f2p!(100.0), f2p!(110.0)));
    }

    #[test]
    fn test_underlying_equals_min_strike() {
        let strikes = vec![f2p!(100.0), f2p!(110.0)];
        let underlying = f2p!(100.0);
        let strategy = TestStrategy::new(strikes, underlying);
        assert_eq!(strategy.max_min_strikes(), (f2p!(100.0), f2p!(110.0)));
    }

    #[test]
    fn test_underlying_equals_max_strike() {
        let strikes = vec![f2p!(90.0), f2p!(100.0)];
        let underlying = f2p!(100.0);
        let strategy = TestStrategy::new(strikes, underlying);
        assert_eq!(strategy.max_min_strikes(), (f2p!(90.0), f2p!(100.0)));
    }

    #[test]
    fn test_unordered_strikes() {
        let strikes = vec![f2p!(110.0), f2p!(90.0), f2p!(100.0)];
        let strategy = TestStrategy::new(strikes, Positive::ZERO);
        assert_eq!(strategy.max_min_strikes(), (f2p!(90.0), f2p!(110.0)));
    }
}

#[cfg(test)]
mod tests_best_range_to_show {
    use super::*;
    use crate::f2p;

    struct TestStrategy {
        underlying_price: Positive,
        break_even_points: Vec<Positive>,
        strikes: Vec<Positive>,
    }

    impl TestStrategy {
        fn new(
            underlying_price: Positive,
            break_even_points: Vec<Positive>,
            strikes: Vec<Positive>,
        ) -> Self {
            Self {
                underlying_price,
                break_even_points,
                strikes,
            }
        }
    }

    impl Validable for TestStrategy {}

    impl Positionable for TestStrategy {}

    impl Strategies for TestStrategy {
        fn get_underlying_price(&self) -> Positive {
            self.underlying_price
        }

        fn get_break_even_points(&self) -> Vec<Positive> {
            self.break_even_points.clone()
        }

        fn strikes(&self) -> Vec<Positive> {
            self.strikes.clone()
        }
    }

    #[test]
    fn test_basic_range_with_step() {
        let strategy = TestStrategy::new(
            f2p!(100.0),
            vec![f2p!(90.0), f2p!(110.0)],
            vec![f2p!(95.0), f2p!(105.0)],
        );
        let range = strategy.best_range_to_show(f2p!(5.0)).unwrap();
        assert!(!range.is_empty());
        assert_eq!(range[1] - range[0], f2p!(5.0));
    }

    #[test]
    fn test_range_with_small_step() {
        let strategy = TestStrategy::new(
            f2p!(100.0),
            vec![f2p!(95.0), f2p!(105.0)],
            vec![f2p!(97.0), f2p!(103.0)],
        );
        let range = strategy.best_range_to_show(f2p!(1.0)).unwrap();
        assert!(!range.is_empty());
        assert_eq!(range[1] - range[0], f2p!(1.0));
    }

    #[test]
    fn test_range_boundaries() {
        let strategy = TestStrategy::new(
            f2p!(100.0),
            vec![f2p!(90.0), f2p!(110.0)],
            vec![f2p!(95.0), f2p!(105.0)],
        );
        let range = strategy.best_range_to_show(f2p!(5.0)).unwrap();
        assert!(range.first().unwrap() < &f2p!(90.0));
        assert!(range.last().unwrap() > &f2p!(110.0));
    }

    #[test]
    fn test_range_step_size() {
        let strategy = TestStrategy::new(
            f2p!(100.0),
            vec![f2p!(90.0), f2p!(110.0)],
            vec![f2p!(95.0), f2p!(105.0)],
        );
        let step = f2p!(5.0);
        let range = strategy.best_range_to_show(step).unwrap();

        for i in 1..range.len() {
            assert_eq!(range[i] - range[i - 1], step);
        }
    }

    #[test]
    fn test_range_includes_underlying() {
        let underlying_price = f2p!(100.0);
        let strategy = TestStrategy::new(
            underlying_price,
            vec![f2p!(90.0), f2p!(110.0)],
            vec![f2p!(95.0), f2p!(105.0)],
        );
        let range = strategy.best_range_to_show(f2p!(5.0)).unwrap();

        assert!(range.iter().any(|&price| price <= underlying_price));
        assert!(range.iter().any(|&price| price >= underlying_price));
    }

    #[test]
    fn test_range_with_extreme_values() {
        let strategy = TestStrategy::new(
            f2p!(100.0),
            vec![f2p!(50.0), f2p!(150.0)],
            vec![f2p!(75.0), f2p!(125.0)],
        );
        let range = strategy.best_range_to_show(f2p!(10.0)).unwrap();

        assert!(range.first().unwrap() <= &f2p!(50.0));
        assert!(range.last().unwrap() >= &f2p!(150.0));
    }
}

#[cfg(test)]
mod tests_range_to_show {
    use super::*;
    use crate::f2p;

    struct TestStrategy {
        underlying_price: Positive,
        break_even_points: Vec<Positive>,
        strikes: Vec<Positive>,
    }

    impl TestStrategy {
        fn new(
            underlying_price: Positive,
            break_even_points: Vec<Positive>,
            strikes: Vec<Positive>,
        ) -> Self {
            Self {
                underlying_price,
                break_even_points,
                strikes,
            }
        }
    }

    impl Validable for TestStrategy {}

    impl Positionable for TestStrategy {}

    impl Strategies for TestStrategy {
        fn get_underlying_price(&self) -> Positive {
            self.underlying_price
        }

        fn get_break_even_points(&self) -> Vec<Positive> {
            self.break_even_points.clone()
        }

        fn strikes(&self) -> Vec<Positive> {
            self.strikes.clone()
        }
    }

    #[test]
    fn test_basic_range() {
        let strategy = TestStrategy::new(
            f2p!(100.0),
            vec![f2p!(90.0), f2p!(110.0)],
            vec![f2p!(95.0), f2p!(105.0)],
        );
        let (start, end) = strategy.range_to_show();
        assert!(start < f2p!(90.0));
        assert!(end > f2p!(110.0));
    }

    #[test]
    fn test_range_with_far_strikes() {
        let strategy = TestStrategy::new(
            f2p!(100.0),
            vec![f2p!(90.0), f2p!(110.0)],
            vec![f2p!(80.0), f2p!(120.0)],
        );
        let (start, end) = strategy.range_to_show();
        assert!(start < f2p!(80.0));
        assert!(end > f2p!(120.0));
    }

    #[test]
    fn test_range_with_underlying_outside_strikes() {
        let strategy = TestStrategy::new(
            f2p!(150.0),
            vec![f2p!(90.0), f2p!(110.0)],
            vec![f2p!(95.0), f2p!(105.0)],
        );
        let (_start, end) = strategy.range_to_show();
        assert!(end > f2p!(150.0));
    }
}

#[cfg(test)]
mod tests_range_of_profit {
    use super::*;
    use crate::f2p;

    struct TestStrategy {
        break_even_points: Vec<Positive>,
    }

    impl TestStrategy {
        fn new(break_even_points: Vec<Positive>) -> Self {
            Self { break_even_points }
        }
    }

    impl Validable for TestStrategy {}

    impl Positionable for TestStrategy {}

    impl Strategies for TestStrategy {
        fn get_break_even_points(&self) -> Vec<Positive> {
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
        let strategy = TestStrategy::new(vec![f2p!(100.0)]);
        assert_eq!(strategy.range_of_profit(), Some(Positive::INFINITY));
    }

    #[test]
    fn test_two_break_even_points() {
        let strategy = TestStrategy::new(vec![f2p!(90.0), f2p!(110.0)]);
        assert_eq!(strategy.range_of_profit(), Some(f2p!(20.0)));
    }

    #[test]
    fn test_multiple_break_even_points() {
        let strategy = TestStrategy::new(vec![f2p!(80.0), f2p!(100.0), f2p!(120.0)]);
        assert_eq!(strategy.range_of_profit(), Some(f2p!(40.0)));
    }

    #[test]
    fn test_unordered_break_even_points() {
        let strategy = TestStrategy::new(vec![f2p!(120.0), f2p!(80.0), f2p!(100.0)]);
        assert_eq!(strategy.range_of_profit(), Some(f2p!(40.0)));
    }
}
