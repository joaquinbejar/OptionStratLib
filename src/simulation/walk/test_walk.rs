/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 26/2/25
******************************************************************************/

use super::*;
#[cfg(test)]
mod tests_random_walk {
    use super::*;
    use crate::pos;

    struct TestWalk {
        values: Vec<Positive>,
    }

    impl TestWalk {
        fn new() -> Self {
            Self { values: Vec::new() }
        }
    }

    impl Walkable for TestWalk {
        fn get_y_values(&self) -> &Vec<Positive> {
            &self.values
        }

        fn get_y_values_ref(&mut self) -> &mut Vec<Positive> {
            &mut self.values
        }

        fn get_randon_walk(&self) -> Result<RandomWalkGraph, Box<dyn Error>> {
            Ok(RandomWalkGraph::default())
        }
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_walk_initialization() {
        let mut walk = TestWalk::new();
        let initial_price = pos!(100.0);

        walk.generate_random_walk(10, initial_price, 0.0, pos!(1.0), pos!(0.01))
            .unwrap();

        assert_eq!(walk.values.len(), 10);
        assert_eq!(walk.values[0], initial_price);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_random_walk_length() {
        let mut walk = TestWalk::new();
        let n_steps = 100;
        let initial_price = pos!(100.0);
        let mean = 0.0;
        let std_dev = pos!(1.0);
        let std_dev_change = pos!(0.01);
        walk.generate_random_walk(n_steps, initial_price, mean, std_dev, std_dev_change)
            .unwrap();
        assert_eq!(walk.values.len(), n_steps);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_random_walk_starts_at_initial_price() {
        let mut walk = TestWalk::new();

        let initial_price = pos!(100.0);
        walk.generate_random_walk(10, initial_price, 0.0, pos!(1.0), pos!(0.01))
            .unwrap();
        assert_eq!(walk.values[0], initial_price);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_all_values_are_positive() {
        let mut walk = TestWalk::new();
        let n_steps = 1000;
        let initial_price = pos!(100.0);
        let mean = 0.0;
        let std_dev = pos!(1.0);
        let std_dev_change = pos!(0.01);
        walk.generate_random_walk(n_steps, initial_price, mean, std_dev, std_dev_change)
            .unwrap();
        assert!(walk.values.iter().all(|x| x > 0.0));
    }
    

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_zero_std_dev_change() {
        let mut walk = TestWalk::new();

        let n_steps = 100;
        let initial_price = pos!(100.0);
        let mean = 0.0;
        let std_dev = pos!(1.0);
        let std_dev_change = Positive::ZERO;

        walk.generate_random_walk(n_steps, initial_price, mean, std_dev, std_dev_change)
            .unwrap();
        assert_eq!(walk.values.len(), n_steps);
        assert!(walk.values.iter().all(|x| x > 0.0));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_edge_cases() {
        let mut walk = TestWalk::new();

        walk.generate_random_walk(1, pos!(100.0), 0.0, pos!(1.0), pos!(0.01))
            .unwrap();
        assert_eq!(walk.values.len(), 1);
        assert_eq!(walk.values[0], 100.0);

        walk.generate_random_walk(100, pos!(0.1), 0.0, pos!(0.01), pos!(0.001))
            .unwrap();
        assert!(walk.values.iter().all(|x| x >= 0.0));

        walk.generate_random_walk(100, pos!(1e6), 0.0, pos!(100.0), pos!(1.0))
            .unwrap();
        assert!(walk.values.iter().all(|x| x >= 0.0));
    }

    #[test]
    fn test_zero_steps_should_error() {
        let mut walk = TestWalk::new();
        let result = walk.generate_random_walk(0, pos!(100.0), 0.0, pos!(1.0), pos!(0.01));
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod tests_iterator {
    use super::*;
    use crate::utils::logger::setup_logger_with_level;
    use crate::{pos, spos};
    use rust_decimal_macros::dec;
    use tracing::debug;

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_random_walk_iterator() {
        setup_logger_with_level("debug");
        let years = 3.0;
        let steps = 252 * years as usize; // 2 years

        let mut walk = RandomWalkGraph::new(
            "Test Walk".to_string(),
            Some(dec!(0.05)), // risk_free_rate
            Some(pos!(0.02)), // dividend_yield
            TimeFrame::Day,   // time_frame (2 years)
            4,                // volatility_window
            spos!(0.2),       // initial_volatility
        );

        walk.generate_random_walk(
            steps,       // n_steps
            pos!(100.0), // initial_price
            0.0,         // mean
            pos!(0.2),   // std_dev
            pos!(0.01),  // std_dev_change
        )
        .unwrap();

        for (i, params) in walk.enumerate() {
            debug!(
                "Step {}: Price={}, IV={:?}, Days to Exp={:?}",
                i, params.underlying_price, params.implied_volatility, params.expiration_date
            );

            assert!(
                params.underlying_price > Positive::ZERO,
                "Price should be positive"
            );
            assert!(
                params.risk_free_rate >= Decimal::ZERO,
                "Risk-free rate should be non-negative"
            );
            assert!(
                params.dividend_yield >= Positive::ZERO,
                "Dividend yield should be non-negative"
            );

            if let Some(iv) = params.implied_volatility {
                assert!(
                    iv >= Positive::ZERO,
                    "Implied volatility should be positive"
                );
            }
        }
    }
}

#[cfg(test)]
mod tests_random_walk_timeframe {
    use super::*;
    use crate::pos;
    
    // Mock struct for testing
    struct TestWalker {
        values: Vec<Positive>,
    }

    impl TestWalker {
        fn new() -> Self {
            Self { values: Vec::new() }
        }
    }

    impl Walkable for TestWalker {
        fn get_y_values(&self) -> &Vec<Positive> {
            &self.values
        }

        fn get_y_values_ref(&mut self) -> &mut Vec<Positive> {
            &mut self.values
        }

        // Implementing required generate_random_walk
        fn generate_random_walk(
            &mut self,
            n_steps: usize,
            initial_price: Positive,
            _mean: f64,
            _std_dev: Positive,
            _std_dev_change: Positive,
        ) -> Result<(), Box<dyn Error>> {
            // Simple implementation for testing
            self.values = vec![initial_price; n_steps];
            Ok(())
        }

        fn get_randon_walk(&self) -> Result<RandomWalkGraph, Box<dyn Error>> {
            Ok(RandomWalkGraph::default())
        }
    }

    #[test]
    fn test_zero_steps() {
        let mut walker = TestWalker::new();
        let result = walker.generate_random_walk_timeframe(
            0,
            pos!(100.0),
            0.0,
            pos!(0.2),
            pos!(0.1),
            TimeFrame::Day,
            None,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_initial_price() {
        let mut walker = TestWalker::new();
        let initial_price = pos!(100.0);

        walker
            .generate_random_walk_timeframe(
                100,
                initial_price,
                0.0,
                pos!(0.2),
                pos!(0.0), // No volatility change
                TimeFrame::Day,
                None,
            )
            .unwrap();

        assert!(!walker.values.is_empty());
        assert_eq!(walker.values[0], initial_price);
    }

    #[test]
    fn test_correct_number_of_steps() {
        let mut walker = TestWalker::new();
        let n_steps = 100;

        // Usando valores más realistas para la volatilidad
        walker
            .generate_random_walk_timeframe(
                n_steps,
                pos!(100.0), // initial price
                0.0,         // mean
                pos!(0.01),  // std_dev (1% volatilidad diaria)
                pos!(0.001), // std_dev_change (0.1% VoV)
                TimeFrame::Day,
                None,
            )
            .unwrap();

        assert_eq!(walker.values.len(), n_steps);
    }
    
    #[test]
    fn test_small_valid_volatility() {
        let mut walker = TestWalker::new();
        let n_steps = 100;

        walker
            .generate_random_walk_timeframe(
                n_steps,
                pos!(100.0),
                0.0,
                pos!(0.0001),
                pos!(0.00001),
                TimeFrame::Day,
                Some((pos!(0.00001), pos!(0.001))),
            )
            .unwrap();

        assert_eq!(walker.values.len(), n_steps);
    }

    #[test]
    fn test_different_timeframes() {
        let mut walker = TestWalker::new();
        let initial_price = pos!(100.0);
        let daily_vol = pos!(0.2);

        // Run with daily timeframe
        walker
            .generate_random_walk_timeframe(
                100,
                initial_price,
                0.0,
                daily_vol,
                pos!(0.0),
                TimeFrame::Day,
                None,
            )
            .unwrap();
        let daily_values = walker.values.clone();

        // Run with minute timeframe
        walker
            .generate_random_walk_timeframe(
                100,
                initial_price,
                0.0,
                daily_vol,
                pos!(0.0),
                TimeFrame::Minute,
                None,
            )
            .unwrap();
        let minute_values = walker.values.clone();

        // Minute values should have smaller changes between steps
        let daily_changes: Vec<f64> = daily_values
            .windows(2)
            .map(|w| (w[1].to_f64() - w[0].to_f64()).abs())
            .collect();
        let minute_changes: Vec<f64> = minute_values
            .windows(2)
            .map(|w| (w[1].to_f64() - w[0].to_f64()).abs())
            .collect();

        let avg_daily_change: f64 = daily_changes.iter().sum::<f64>() / daily_changes.len() as f64;
        let avg_minute_change: f64 =
            minute_changes.iter().sum::<f64>() / minute_changes.len() as f64;

        assert!(
            avg_minute_change < avg_daily_change,
            "Average minute change ({}) should be less than average daily change ({})",
            avg_minute_change,
            avg_daily_change
        );
    }

    #[test]
    fn test_no_negative_values() {
        let mut walker = TestWalker::new();

        walker
            .generate_random_walk_timeframe(
                1000,
                pos!(100.0),
                -0.1,      // Negative drift
                pos!(0.5), // High volatility
                pos!(0.1),
                TimeFrame::Day,
                None,
            )
            .unwrap();

        // Check that no values are negative
        assert!(walker.values.iter().all(|&v| v >= Positive::ZERO));
    }
    
}

#[cfg(test)]
mod tests_random_walk_iterator {
    use super::*;
    use crate::model::types::ExpirationDate;
    use crate::pos;
    use rust_decimal_macros::dec;

    // Helper function to create a test RandomWalkGraph
    fn create_test_walk() -> RandomWalkGraph {
        let mut walk = RandomWalkGraph::new(
            "Test Walk".to_string(),
            Some(dec!(0.05)), // risk_free_rate
            Some(pos!(0.02)), // dividend_yield
            TimeFrame::Day,
            5,               // volatility_window
            Some(pos!(0.2)), // initial_volatility
        );

        // Add some test values
        walk.values = vec![
            pos!(100.0),
            pos!(101.0),
            pos!(102.0),
            pos!(103.0),
            pos!(104.0),
        ];

        walk
    }

    #[test]
    fn test_iterator_creation() {
        let walk = create_test_walk();
        let iter = walk.iter();
        assert_eq!(iter.current_index, 0);
    }

    #[test]
    fn test_iterator_complete_traversal() {
        let walk = create_test_walk();
        let iter = walk.iter();

        let count = iter.count();
        assert_eq!(count, 5, "Iterator should yield exactly 5 items");
    }

    #[test]
    fn test_iterator_values() {
        let walk = create_test_walk();
        let mut iter = walk.iter();

        // Check first item
        let first = iter.next().unwrap();
        assert_eq!(first.underlying_price, pos!(100.0));
        assert_eq!(first.risk_free_rate, dec!(0.05));
        assert_eq!(first.dividend_yield, pos!(0.02));

        // Check days to expiration decreases
        let days_remaining = match first.expiration_date {
            ExpirationDate::Days(days) => days,
            _ => panic!("Expected Days variant"),
        };
        assert_eq!(days_remaining, pos!(5.0));
    }

    #[test]
    fn test_iterator_empty_walk() {
        let walk = RandomWalkGraph::new(
            "Empty Walk".to_string(),
            None,
            None,
            TimeFrame::Day,
            5,
            None,
        );
        let mut iter = walk.iter();
        assert!(iter.next().is_none());
    }

    #[test]
    fn test_iterator_remaining_days() {
        let walk = create_test_walk();
        let mut iter = walk.iter();

        // First item should have maximum remaining days
        let first = iter.next().unwrap();
        match first.expiration_date {
            ExpirationDate::Days(days) => assert_eq!(days, pos!(5.0)),
            _ => panic!("Expected Days variant"),
        }

        // Skip to last item
        for _ in 0..3 {
            iter.next();
        }
        let last = iter.next().unwrap();
        match last.expiration_date {
            ExpirationDate::Days(days) => assert_eq!(days, pos!(1.0)),
            _ => panic!("Expected Days variant"),
        }
    }

    #[test]
    fn test_iterator_multiple_traversals() {
        let walk = create_test_walk();

        // First traversal
        let count1 = walk.iter().count();
        assert_eq!(count1, 5);

        // Second traversal
        let count2 = walk.iter().count();
        assert_eq!(count2, 5, "Second traversal should yield same count");
    }

    #[test]
    fn test_iterator_partial_traversal() {
        let walk = create_test_walk();
        let iter = walk.iter();

        // Take only first 3 items
        let taken: Vec<_> = iter.take(3).collect();
        assert_eq!(taken.len(), 3);
    }

    #[test]
    fn test_default_values() {
        let mut walk = RandomWalkGraph::new(
            "Default Walk".to_string(),
            None, // No risk_free_rate
            None, // No dividend_yield
            TimeFrame::Day,
            5,
            None, // No initial_volatility
        );
        walk.values = vec![pos!(100.0)];

        let item = walk.iter().next().unwrap();
        assert_eq!(item.risk_free_rate, Decimal::ZERO);
        assert_eq!(item.dividend_yield, Positive::ZERO);
    }

    #[test]
    fn test_iterator_implied_volatility() {
        let mut walk = create_test_walk();
        walk.initial_volatility = Some(pos!(0.2));

        let first = walk.iter().next().unwrap();
        assert!(first.implied_volatility.is_some());
    }

    #[test]
    fn test_iterator_skipping() {
        let walk = create_test_walk();
        let mut iter = walk.iter().skip(2);

        let first_after_skip = iter.next().unwrap();
        assert_eq!(first_after_skip.underlying_price, pos!(102.0));
    }
}

#[cfg(test)]
mod tests_curvable {
    use super::*;
    use crate::pos;
    use crate::utils::time::TimeFrame;
    use rust_decimal_macros::dec;

    // Helper function to create a test graph
    fn create_test_graph() -> RandomWalkGraph {
        let mut graph = RandomWalkGraph::new(
            "Test".to_string(),
            None, // risk_free_rate
            None, // dividend_yield
            TimeFrame::Day,
            4,    // volatility_window
            None, // initial_volatility
        );

        // Add some test values
        graph.values = vec![pos!(1.0), pos!(2.0), pos!(3.0), pos!(4.0), pos!(5.0)];

        graph
    }

    #[test]
    fn test_curve_empty_graph() {
        let graph = RandomWalkGraph::new("Empty".to_string(), None, None, TimeFrame::Day, 4, None);

        let curve = graph.curve().unwrap();
        assert_eq!(curve.points.len(), 0);
    }

    #[test]
    fn test_curve_with_values() {
        let graph = create_test_graph();
        let curve = graph.curve().unwrap();

        // Check number of points
        assert_eq!(curve.points.len(), 5);

        // Check specific points - they should be (index, value) pairs
        let points: Vec<_> = curve.points.iter().collect();
        assert_eq!(points[0].x, dec!(0));
        assert_eq!(points[0].y, dec!(1.0));

        assert_eq!(points[2].x, dec!(2));
        assert_eq!(points[2].y, dec!(3.0));

        assert_eq!(points[4].x, dec!(4));
        assert_eq!(points[4].y, dec!(5.0));
    }

    #[test]
    fn test_curve_ordering() {
        let graph = create_test_graph();
        let curve = graph.curve().unwrap();

        // Points should be ordered by x coordinate
        let points: Vec<_> = curve.points.iter().collect();
        for i in 0..points.len() - 1 {
            assert!(points[i].x < points[i + 1].x);
        }
    }

    #[test]
    fn test_curve_point_conversion() {
        let mut graph = create_test_graph();
        graph.values = vec![pos!(1.23), pos!(4.56)];

        let curve = graph.curve().unwrap();
        let points: Vec<_> = curve.points.iter().collect();

        // Check decimal precision is maintained
        assert_eq!(points[0].y, dec!(1.23));
        assert_eq!(points[1].y, dec!(4.56));
    }

    #[test]
    fn test_curve_large_values() {
        let mut graph = create_test_graph();
        graph.values = vec![pos!(1000000.0), pos!(2000000.0), pos!(3000000.0)];

        let curve = graph.curve().unwrap();
        let points: Vec<_> = curve.points.iter().collect();

        // Check handling of large numbers
        assert_eq!(points[0].y, dec!(1000000.0));
        assert_eq!(points[1].y, dec!(2000000.0));
        assert_eq!(points[2].y, dec!(3000000.0));
    }

    #[test]
    fn test_curve_with_single_value() {
        let mut graph = create_test_graph();
        graph.values = vec![pos!(1.0)];

        let curve = graph.curve().unwrap();
        assert_eq!(curve.points.len(), 1);

        let point = curve.points.iter().next().unwrap();
        assert_eq!(point.x, dec!(0));
        assert_eq!(point.y, dec!(1.0));
    }
}
