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
use crate::error::strategies::{BreakEvenErrorKind, StrategyError};
use crate::error::OperationErrorKind;
use crate::model::position::Position;
use crate::strategies::utils::{calculate_price_range, FindOptimalSide, OptimizationCriteria};
use crate::{Positive, Side};
use itertools::Itertools;
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

    fn max_profit(&self) -> Result<Positive, StrategyError> {
        Err(StrategyError::operation_not_supported(
            "max_profit",
            std::any::type_name::<Self>(),
        ))
    }

    fn max_profit_iter(&mut self) -> Result<Positive, StrategyError> {
        self.max_profit()
    }

    fn max_loss(&self) -> Result<Positive, StrategyError> {
        Err(StrategyError::operation_not_supported(
            "max_loss",
            std::any::type_name::<Self>(),
        ))
    }

    fn max_loss_iter(&mut self) -> Result<Positive, StrategyError> {
        self.max_loss()
    }

    /// Calculates the total cost (premium paid Long - premium get short) of the strategy.
    ///
    /// # Returns
    /// `f64` - The total cost will be zero if the strategy is not applicable.
    ///
    fn total_cost(&self) -> Result<Positive, PositionError> {
        let positions = self.get_positions()?;
        let costs = positions
            .iter()
            .map(|p| p.total_cost().unwrap())
            .sum::<Positive>();

        Ok(costs)
    }

    fn net_cost(&self) -> Result<Decimal, PositionError> {
        let positions = self.get_positions()?;
        let costs = positions
            .iter()
            .map(|p| p.net_cost().unwrap())
            .sum::<Decimal>();

        Ok(costs)
    }

    fn net_premium_received(&self) -> Result<Positive, StrategyError> {
        let positions = self.get_positions()?;
        let costs = positions
            .iter()
            .filter(|p| p.option.side == Side::Long)
            .map(|p| p.net_cost().unwrap())
            .sum::<Decimal>();

        let premiums = positions
            .iter()
            .filter(|p| p.option.side == Side::Short)
            .map(|p| p.net_premium_received().unwrap())
            .sum::<Positive>();

        match premiums > costs {
            true => Ok(premiums - costs),
            false => Ok(Positive::ZERO),
        }
    }

    fn fees(&self) -> Result<Positive, StrategyError> {
        let mut fee = Positive::ZERO;
        let positions = match self.get_positions() {
            Ok(positions) => positions,
            Err(err) => {
                return Err(StrategyError::OperationError(
                    OperationErrorKind::InvalidParameters {
                        operation: "get_positions".to_string(),
                        reason: err.to_string(),
                    },
                ))
            }
        };

        for position in positions {
            fee += position.fees()?;
        }
        Ok(fee)
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

    fn range_to_show(&self) -> Result<(Positive, Positive), StrategyError> {
        let mut all_points = self.get_break_even_points()?.clone();
        let (first_strike, last_strike) = self.max_min_strikes()?;
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
        Ok((start_price, end_price))
    }

    fn best_range_to_show(&self, step: Positive) -> Result<Vec<Positive>, StrategyError> {
        let (start_price, end_price) = self.range_to_show()?;
        Ok(calculate_price_range(start_price, end_price, step))
    }

    fn strikes(&self) -> Result<Vec<Positive>, StrategyError> {
        let positions = match self.get_positions() {
            Ok(positions) => positions,
            Err(_) => {
                return Err(StrategyError::OperationError(
                    OperationErrorKind::InvalidParameters {
                        operation: "get_positions".to_string(),
                        reason: "No positions found".to_string(),
                    },
                ))
            }
        };

        let strikes: Vec<Positive> = positions
            .iter()
            .map(|leg| leg.option.strike_price)
            .collect::<Vec<_>>()
            .into_iter()
            .sorted()
            .collect();

        Ok(strikes)
    }

    fn max_min_strikes(&self) -> Result<(Positive, Positive), StrategyError> {
        let strikes = self.strikes()?;

        if strikes.is_empty() {
            return Err(StrategyError::OperationError(
                OperationErrorKind::InvalidParameters {
                    operation: "max_min_strikes".to_string(),
                    reason: "No strikes found".to_string(),
                },
            ));
        }

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

        Ok((min, max))
    }

    fn get_break_even_points(&self) -> Result<&Vec<Positive>, StrategyError> {
        Err(StrategyError::operation_not_supported(
            "get_break_even_points",
            std::any::type_name::<Self>(),
        ))
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
    fn range_of_profit(&self) -> Result<Positive, StrategyError> {
        let mut break_even_points = self.get_break_even_points()?.clone();
        match break_even_points.len() {
            0 => Err(StrategyError::BreakEvenError(
                BreakEvenErrorKind::NoBreakEvenPoints,
            )),
            1 => Ok(Positive::INFINITY),
            2 => Ok(break_even_points[1] - break_even_points[0]),
            _ => {
                // sort break even points and then get last minus first
                break_even_points.sort_by(|a, b| a.partial_cmp(b).unwrap());
                Ok(*break_even_points.last().unwrap() - *break_even_points.first().unwrap())
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
    use crate::model::position::Position;
    use crate::model::types::{OptionStyle, Side};
    use crate::model::utils::create_sample_option_simplest;
    use crate::pos;
    use rust_decimal_macros::dec;

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
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
        break_even_points: Vec<Positive>,
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
        fn get_break_even_points(&self) -> Result<&Vec<Positive>, StrategyError> {
            Ok(&self.break_even_points)
        }

        fn max_profit(&self) -> Result<Positive, StrategyError> {
            Ok(Positive::THOUSAND)
        }

        fn max_loss(&self) -> Result<Positive, StrategyError> {
            Ok(pos!(500.0))
        }

        fn total_cost(&self) -> Result<Positive, PositionError> {
            Ok(pos!(200.0))
        }

        fn net_premium_received(&self) -> Result<Positive, StrategyError> {
            Ok(pos!(300.0))
        }

        fn fees(&self) -> Result<Positive, StrategyError> {
            Ok(pos!(50.0))
        }

        fn profit_area(&self) -> Result<Decimal, StrategyError> {
            Ok(dec!(5000.0))
        }

        fn profit_ratio(&self) -> Result<Decimal, StrategyError> {
            Ok(dec!(2.0))
        }
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_strategies_trait() {
        let mut mock_strategy = MockStrategy {
            legs: Vec::new(),
            break_even_points: vec![Positive::HUNDRED],
        };

        // Test add_leg and get_legs
        let option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        let position = Position::new(
            option,
            Positive::ONE,
            Default::default(),
            Positive::ZERO,
            Positive::ZERO,
        );
        mock_strategy
            .add_position(&position.clone())
            .expect("Error adding position");

        // Test other methods
        assert_eq!(
            mock_strategy.get_break_even_points().unwrap(),
            &vec![Positive::HUNDRED]
        );
        assert_eq!(mock_strategy.max_profit().unwrap_or(Positive::ZERO), 1000.0);
        assert_eq!(mock_strategy.max_loss().unwrap_or(Positive::ZERO), 500.0);
        assert_eq!(mock_strategy.total_cost().unwrap().to_f64(), 200.0);
        assert_eq!(mock_strategy.net_premium_received().unwrap(), dec!(300.0));
        assert_eq!(mock_strategy.fees().unwrap(), dec!(50.0));
        assert_eq!(mock_strategy.profit_area().unwrap(), dec!(5000.0));
        assert_eq!(mock_strategy.profit_ratio().unwrap(), dec!(2.0));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
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
        assert!(strategy.total_cost().is_err());
        assert!(strategy.profit_area().is_err());
        assert!(strategy.profit_ratio().is_err());
        assert!(strategy.validate());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_strategies_add_leg_panic() {
        struct PanicStrategy;
        impl Validable for PanicStrategy {}
        impl Positionable for PanicStrategy {}
        impl Strategies for PanicStrategy {}

        let mut strategy = PanicStrategy;
        let option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        let position = Position::new(
            option,
            Positive::ONE,
            Default::default(),
            Positive::ZERO,
            Positive::ZERO,
        );

        assert!(strategy.add_position(&position).is_err());
    }
}

#[cfg(test)]
mod tests_strategies_extended {
    use super::*;
    use crate::model::position::Position;
    use crate::model::types::{OptionStyle, Side};
    use crate::model::utils::create_sample_option_simplest;
    use crate::pos;

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_strategy_enum() {
        assert_ne!(StrategyType::BullCallSpread, StrategyType::BearCallSpread);
        assert_eq!(StrategyType::Custom, StrategyType::Custom);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_strategy_new_with_legs() {
        let mut strategy = Strategy::new(
            "Test Strategy".to_string(),
            StrategyType::BullCallSpread,
            "Test Description".to_string(),
        );
        let option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        let position = Position::new(
            option,
            Positive::ONE,
            Default::default(),
            Positive::ZERO,
            Positive::ZERO,
        );

        strategy.legs.push(position);

        assert_eq!(strategy.legs.len(), 1);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_strategies_get_legs_panic() {
        struct PanicStrategy;
        impl Validable for PanicStrategy {}
        impl Positionable for PanicStrategy {}
        impl Strategies for PanicStrategy {}

        let strategy = PanicStrategy;
        assert!(strategy.get_positions().is_err());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_strategies_break_even_panic() {
        struct PanicStrategy;
        impl Validable for PanicStrategy {}
        impl Positionable for PanicStrategy {}
        impl Strategies for PanicStrategy {}

        let strategy = PanicStrategy;
        assert!(strategy.get_break_even_points().is_err());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_strategies_net_premium_received_panic() {
        struct PanicStrategy;
        impl Validable for PanicStrategy {}
        impl Positionable for PanicStrategy {}
        impl Strategies for PanicStrategy {}

        let strategy = PanicStrategy;
        assert!(strategy.net_premium_received().is_err());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_strategies_fees_panic() {
        struct PanicStrategy;
        impl Validable for PanicStrategy {}
        impl Positionable for PanicStrategy {}
        impl Strategies for PanicStrategy {}

        let strategy = PanicStrategy;
        assert!(strategy.fees().is_err());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_strategies_max_profit_iter() {
        struct TestStrategy;
        impl Validable for TestStrategy {}
        impl Positionable for TestStrategy {}
        impl Strategies for TestStrategy {
            fn max_profit(&self) -> Result<Positive, StrategyError> {
                Ok(pos!(100.0))
            }
        }

        let mut strategy = TestStrategy;
        assert_eq!(strategy.max_profit_iter().unwrap().to_f64(), 100.0);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_strategies_max_loss_iter() {
        struct TestStrategy;
        impl Validable for TestStrategy {}
        impl Positionable for TestStrategy {}
        impl Strategies for TestStrategy {
            fn max_loss(&self) -> Result<Positive, StrategyError> {
                Ok(pos!(50.0))
            }
        }

        let mut strategy = TestStrategy;
        assert_eq!(strategy.max_loss_iter().unwrap().to_f64(), 50.0);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
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
        assert_eq!(strategy.strikes().unwrap(), Vec::<Positive>::new());
        assert!(strategy.max_min_strikes().is_err());
    }
}

#[cfg(test)]
mod tests_strategy_type {
    use super::*;

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_strategy_type_equality() {
        assert_eq!(StrategyType::BullCallSpread, StrategyType::BullCallSpread);
        assert_ne!(StrategyType::BullCallSpread, StrategyType::BearCallSpread);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_strategy_type_clone() {
        let strategy = StrategyType::IronCondor;
        let cloned = strategy.clone();
        assert_eq!(strategy, cloned);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_strategy_type_debug() {
        let strategy = StrategyType::Straddle;
        let debug_string = format!("{:?}", strategy);
        assert_eq!(debug_string, "Straddle");
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
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
    use crate::{pos, Side};

    struct TestStrategy {
        strikes: Vec<Positive>,
        underlying_price: Positive,
        break_even_points: Vec<Positive>,
    }

    impl TestStrategy {
        fn new(
            strikes: Vec<Positive>,
            underlying_price: Positive,
            break_even_points: Vec<Positive>,
        ) -> Self {
            Self {
                strikes,
                underlying_price,
                break_even_points,
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
        fn get_break_even_points(&self) -> Result<&Vec<Positive>, StrategyError> {
            Ok(&self.break_even_points)
        }
        fn max_profit(&self) -> Result<Positive, StrategyError> {
            Ok(Positive::ZERO)
        }
        fn max_loss(&self) -> Result<Positive, StrategyError> {
            Ok(Positive::ZERO)
        }
        fn total_cost(&self) -> Result<Positive, PositionError> {
            Ok(Positive::ZERO)
        }
        fn net_premium_received(&self) -> Result<Positive, StrategyError> {
            let positions = self.get_positions()?;
            let costs = positions
                .iter()
                .filter(|p| p.option.side == Side::Long)
                .map(|p| p.net_cost().unwrap())
                .sum::<Decimal>();

            let premiums = positions
                .iter()
                .filter(|p| p.option.side == Side::Short)
                .map(|p| p.net_premium_received().unwrap())
                .sum::<Positive>();

            match premiums > costs {
                true => Ok(premiums - costs),
                false => Err(StrategyError::OperationError(
                    OperationErrorKind::InvalidParameters {
                        operation: "Net premium received".to_string(),
                        reason: "Net premium received is negative".to_string(),
                    },
                )),
            }
        }
        fn fees(&self) -> Result<Positive, StrategyError> {
            Ok(Positive::ZERO)
        }
        fn profit_area(&self) -> Result<Decimal, StrategyError> {
            Ok(Decimal::ZERO)
        }
        fn profit_ratio(&self) -> Result<Decimal, StrategyError> {
            Ok(Decimal::ZERO)
        }
        fn best_range_to_show(&self, _step: Positive) -> Result<Vec<Positive>, StrategyError> {
            Ok(vec![])
        }
        fn strikes(&self) -> Result<Vec<Positive>, StrategyError> {
            Ok(self.strikes.clone())
        }
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_empty_strikes() {
        let strategy = TestStrategy::new(vec![], Positive::ZERO, vec![]);
        assert!(strategy.max_min_strikes().is_err());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_single_strike() {
        let strike = pos!(100.0);
        let strategy = TestStrategy::new(vec![strike], Positive::ZERO, vec![]);
        assert_eq!(strategy.max_min_strikes().unwrap(), (strike, strike));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_multiple_strikes_no_underlying() {
        let strikes = vec![pos!(90.0), pos!(100.0), pos!(110.0)];
        let strategy = TestStrategy::new(strikes.clone(), Positive::ZERO, vec![]);
        assert_eq!(
            strategy.max_min_strikes().unwrap(),
            (*strikes.first().unwrap(), *strikes.last().unwrap())
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_underlying_price_between_strikes() {
        let strikes = vec![pos!(90.0), pos!(110.0)];
        let underlying = pos!(100.0);
        let strategy = TestStrategy::new(strikes, underlying, vec![]);
        assert_eq!(
            strategy.max_min_strikes().unwrap(),
            (pos!(90.0), pos!(110.0))
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_underlying_price_below_min_strike() {
        let strikes = vec![pos!(100.0), pos!(110.0)];
        let underlying = pos!(90.0);
        let strategy = TestStrategy::new(strikes, underlying, vec![]);
        assert_eq!(
            strategy.max_min_strikes().unwrap(),
            (pos!(90.0), pos!(110.0))
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_underlying_price_above_max_strike() {
        let strikes = vec![pos!(90.0), pos!(100.0)];
        let underlying = pos!(110.0);
        let strategy = TestStrategy::new(strikes, underlying, vec![]);
        assert_eq!(
            strategy.max_min_strikes().unwrap(),
            (pos!(90.0), pos!(110.0))
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_strikes_with_duplicates() {
        let strikes = vec![pos!(100.0), pos!(100.0), pos!(110.0)];
        let strategy = TestStrategy::new(strikes, Positive::ZERO, vec![]);
        assert_eq!(
            strategy.max_min_strikes().unwrap(),
            (pos!(100.0), pos!(110.0))
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_underlying_equals_min_strike() {
        let strikes = vec![pos!(100.0), pos!(110.0)];
        let underlying = pos!(100.0);
        let strategy = TestStrategy::new(strikes, underlying, vec![]);
        assert_eq!(
            strategy.max_min_strikes().unwrap(),
            (pos!(100.0), pos!(110.0))
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_underlying_equals_max_strike() {
        let strikes = vec![pos!(90.0), pos!(100.0)];
        let underlying = pos!(100.0);
        let strategy = TestStrategy::new(strikes, underlying, vec![]);
        assert_eq!(
            strategy.max_min_strikes().unwrap(),
            (pos!(90.0), pos!(100.0))
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_unordered_strikes() {
        let strikes = vec![pos!(110.0), pos!(90.0), pos!(100.0)];
        let strategy = TestStrategy::new(strikes, Positive::ZERO, vec![]);
        assert_eq!(
            strategy.max_min_strikes().unwrap(),
            (pos!(90.0), pos!(110.0))
        );
    }
}

#[cfg(test)]
mod tests_best_range_to_show {
    use super::*;
    use crate::pos;

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

        fn get_break_even_points(&self) -> Result<&Vec<Positive>, StrategyError> {
            Ok(&self.break_even_points)
        }

        fn strikes(&self) -> Result<Vec<Positive>, StrategyError> {
            Ok(self.strikes.clone())
        }
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
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
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
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
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
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
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
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
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
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
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
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

        fn get_break_even_points(&self) -> Result<&Vec<Positive>, StrategyError> {
            Ok(&self.break_even_points)
        }

        fn strikes(&self) -> Result<Vec<Positive>, StrategyError> {
            Ok(self.strikes.clone())
        }
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_basic_range() {
        let strategy = TestStrategy::new(
            pos!(100.0),
            vec![pos!(90.0), pos!(110.0)],
            vec![pos!(95.0), pos!(105.0)],
        );
        let (start, end) = strategy.range_to_show().unwrap();
        assert!(start < pos!(90.0));
        assert!(end > pos!(110.0));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_range_with_far_strikes() {
        let strategy = TestStrategy::new(
            pos!(100.0),
            vec![pos!(90.0), pos!(110.0)],
            vec![pos!(80.0), pos!(120.0)],
        );
        let (start, end) = strategy.range_to_show().unwrap();
        assert!(start < pos!(80.0));
        assert!(end > pos!(120.0));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_range_with_underlying_outside_strikes() {
        let strategy = TestStrategy::new(
            pos!(150.0),
            vec![pos!(90.0), pos!(110.0)],
            vec![pos!(95.0), pos!(105.0)],
        );
        let (_start, end) = strategy.range_to_show().unwrap();
        assert!(end > pos!(150.0));
    }
}

#[cfg(test)]
mod tests_range_of_profit {
    use super::*;
    use crate::pos;

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
        fn get_break_even_points(&self) -> Result<&Vec<Positive>, StrategyError> {
            Ok(&self.break_even_points)
        }
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_no_break_even_points() {
        let strategy = TestStrategy::new(vec![]);
        assert!(strategy.range_of_profit().is_err());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_single_break_even_point() {
        let strategy = TestStrategy::new(vec![pos!(100.0)]);
        assert_eq!(strategy.range_of_profit().unwrap(), Positive::INFINITY);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_two_break_even_points() {
        let strategy = TestStrategy::new(vec![pos!(90.0), pos!(110.0)]);
        assert_eq!(strategy.range_of_profit().unwrap(), pos!(20.0));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_multiple_break_even_points() {
        let strategy = TestStrategy::new(vec![pos!(80.0), pos!(100.0), pos!(120.0)]);
        assert_eq!(strategy.range_of_profit().unwrap(), pos!(40.0));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_unordered_break_even_points() {
        let strategy = TestStrategy::new(vec![pos!(120.0), pos!(80.0), pos!(100.0)]);
        assert_eq!(strategy.range_of_profit().unwrap(), pos!(40.0));
    }
}

#[cfg(test)]
mod tests_strategy_methods {
    use super::*;

    #[test]
    fn test_get_underlying_price_panic() {
        struct TestStrategy;
        impl Validable for TestStrategy {}
        impl Positionable for TestStrategy {}
        impl Strategies for TestStrategy {}

        let strategy = TestStrategy;
        let result = std::panic::catch_unwind(|| strategy.get_underlying_price());
        assert!(result.is_err());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_strategy_type_debug_all_variants() {
        let variants = vec![
            StrategyType::BullCallSpread,
            StrategyType::BearCallSpread,
            StrategyType::BullPutSpread,
            StrategyType::BearPutSpread,
            StrategyType::LongButterflySpread,
            StrategyType::ShortButterflySpread,
            StrategyType::IronCondor,
            StrategyType::IronButterfly,
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

        for variant in variants {
            let debug_string = format!("{:?}", variant);
            assert!(!debug_string.is_empty());
        }
    }
}

#[cfg(test)]
mod tests_optimizable {
    use super::*;
    use crate::{pos, spos};
    use rust_decimal_macros::dec;

    struct TestOptimizableStrategy;

    impl Validable for TestOptimizableStrategy {
        fn validate(&self) -> bool {
            true
        }
    }

    impl Positionable for TestOptimizableStrategy {}

    impl Strategies for TestOptimizableStrategy {}

    impl Optimizable for TestOptimizableStrategy {
        type Strategy = Self;
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_is_valid_long_option() {
        let strategy = TestOptimizableStrategy;
        let option_data = OptionData::new(
            pos!(100.0),     // strike_price
            spos!(5.0),      // call_bid
            spos!(5.5),      // call_ask
            spos!(4.0),      // put_bid
            spos!(4.5),      // put_ask
            spos!(0.2),      // implied_volatility
            Some(dec!(0.5)), // delta
            spos!(1000.0),   // volume
            Some(100),       // open_interest
        );
        assert!(strategy.is_valid_long_option(&option_data, &FindOptimalSide::All));
        assert!(strategy.is_valid_long_option(
            &option_data,
            &FindOptimalSide::Range(pos!(90.0), pos!(110.0))
        ));
    }

    #[test]
    #[should_panic]
    fn test_is_valid_long_option_upper_panic() {
        let strategy = TestOptimizableStrategy;
        let option_data = OptionData::new(
            pos!(100.0),     // strike_price
            spos!(5.0),      // call_bid
            spos!(5.5),      // call_ask
            spos!(4.0),      // put_bid
            spos!(4.5),      // put_ask
            spos!(0.2),      // implied_volatility
            Some(dec!(0.5)), // delta
            spos!(1000.0),   // volume
            Some(100),       // open_interest
        );
        assert!(strategy.is_valid_long_option(&option_data, &FindOptimalSide::Upper));
    }

    #[test]
    #[should_panic]
    fn test_is_valid_long_option_lower_panic() {
        let strategy = TestOptimizableStrategy;
        let option_data = OptionData::new(
            pos!(100.0),     // strike_price
            spos!(5.0),      // call_bid
            spos!(5.5),      // call_ask
            spos!(4.0),      // put_bid
            spos!(4.5),      // put_ask
            spos!(0.2),      // implied_volatility
            Some(dec!(0.5)), // delta
            spos!(1000.0),   // volume
            Some(100),       // open_interest
        );
        assert!(strategy.is_valid_long_option(&option_data, &FindOptimalSide::Lower));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_is_valid_short_option() {
        let strategy = TestOptimizableStrategy;
        let option_data = OptionData::new(
            pos!(100.0),     // strike_price
            spos!(5.0),      // call_bid
            spos!(5.5),      // call_ask
            spos!(4.0),      // put_bid
            spos!(4.5),      // put_ask
            spos!(0.2),      // implied_volatility
            Some(dec!(0.5)), // delta
            spos!(1000.0),   // volume
            Some(100),       // open_interest
        );
        assert!(strategy.is_valid_short_option(&option_data, &FindOptimalSide::All));
        assert!(strategy.is_valid_short_option(
            &option_data,
            &FindOptimalSide::Range(pos!(90.0), pos!(110.0))
        ));
    }

    #[test]
    #[should_panic]
    fn test_is_valid_short_option_upper_panic() {
        let strategy = TestOptimizableStrategy;
        let option_data = OptionData::new(
            pos!(100.0),     // strike_price
            spos!(5.0),      // call_bid
            spos!(5.5),      // call_ask
            spos!(4.0),      // put_bid
            spos!(4.5),      // put_ask
            spos!(0.2),      // implied_volatility
            Some(dec!(0.5)), // delta
            spos!(1000.0),   // volume
            Some(100),       // open_interest
        );
        assert!(strategy.is_valid_short_option(&option_data, &FindOptimalSide::Upper));
    }

    #[test]
    #[should_panic]
    fn test_is_valid_short_option_lower_panic() {
        let strategy = TestOptimizableStrategy;
        let option_data = OptionData::new(
            pos!(100.0),     // strike_price
            spos!(5.0),      // call_bid
            spos!(5.5),      // call_ask
            spos!(4.0),      // put_bid
            spos!(4.5),      // put_ask
            spos!(0.2),      // implied_volatility
            Some(dec!(0.5)), // delta
            spos!(1000.0),   // volume
            Some(100),       // open_interest
        );
        assert!(strategy.is_valid_short_option(&option_data, &FindOptimalSide::Lower));
    }
}

#[cfg(test)]
mod tests_strategy_net_operations {
    use super::*;
    use crate::model::position::Position;
    use crate::model::types::{OptionStyle, Side};
    use crate::model::utils::create_sample_option_simplest;
    use crate::pos;
    use chrono::{TimeZone, Utc};

    struct TestStrategy {
        positions: Vec<Position>,
    }

    impl TestStrategy {
        fn new() -> Self {
            Self {
                positions: Vec::new(),
            }
        }
    }

    impl Validable for TestStrategy {}

    impl Positionable for TestStrategy {
        fn add_position(&mut self, position: &Position) -> Result<(), PositionError> {
            self.positions.push(position.clone());
            Ok(())
        }

        fn get_positions(&self) -> Result<Vec<&Position>, PositionError> {
            Ok(self.positions.iter().collect())
        }
    }

    impl Strategies for TestStrategy {}

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_net_cost_calculation() {
        let mut strategy = TestStrategy::new();
        let option_long = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        let option_short = create_sample_option_simplest(OptionStyle::Call, Side::Short);

        let position_long = Position::new(
            option_long,
            Positive::ONE,
            Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(),
            pos!(1.0),
            pos!(0.5),
        );
        let position_short = Position::new(
            option_short,
            Positive::ONE,
            Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(),
            pos!(1.0),
            pos!(0.5),
        );

        strategy.add_position(&position_long).unwrap();
        strategy.add_position(&position_short).unwrap();

        let result = strategy.net_cost().unwrap();
        assert!(result > Decimal::ZERO);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_net_premium_received_calculation() {
        let mut strategy = TestStrategy::new();
        let option_long = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        let option_short = create_sample_option_simplest(OptionStyle::Call, Side::Short);

        let fixed_date = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        let position_long =
            Position::new(option_long, Positive::ONE, fixed_date, pos!(1.0), pos!(0.5));
        let position_short = Position::new(
            option_short,
            Positive::ONE,
            fixed_date,
            pos!(1.0),
            pos!(0.5),
        );

        strategy.add_position(&position_long).unwrap();
        strategy.add_position(&position_short).unwrap();

        let result = strategy.net_premium_received().unwrap();
        assert!(result == Positive::ZERO);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_fees_calculation() {
        let mut strategy = TestStrategy::new();
        let option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        let fixed_date = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        let position = Position::new(option, Positive::ONE, fixed_date, pos!(1.0), pos!(0.5));

        strategy.add_position(&position).unwrap();

        let result = strategy.fees().unwrap();
        assert!(result > Positive::ZERO);
    }
}
