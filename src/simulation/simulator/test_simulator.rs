/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 26/2/25
******************************************************************************/
use super::*;

#[cfg(test)]
mod tests_simulator {
    use super::*;
    use crate::pos;
    use crate::simulation::Walkable;
    use crate::utils::time::TimeFrame;
    use crate::visualization::utils::Graph;
    use rust_decimal_macros::dec;

    // Helper function to create a basic simulator
    fn create_test_simulator() -> Simulator {
        let config = SimulationConfig {
            risk_free_rate: Some(dec!(0.05)),
            dividend_yield: Some(pos!(0.02)),
            time_frame: TimeFrame::Day,
            volatility_window: 10,
            initial_volatility: Some(pos!(0.2)),
        };
        Simulator::new(config)
    }

    #[test]
    fn test_simulator_creation() {
        let simulator = create_test_simulator();
        assert!(simulator.get_walk_ids().is_empty());

        let config = simulator.get_config();
        assert_eq!(config.risk_free_rate, Some(dec!(0.05)));
        assert_eq!(config.dividend_yield, Some(pos!(0.02)));
        assert_eq!(config.volatility_window, 10);
        assert_eq!(config.initial_volatility, Some(pos!(0.2)));
    }

    #[test]
    fn test_add_and_get_walk() {
        let mut simulator = create_test_simulator();

        // Add a walk
        let walk_id = WalkId::new("test_walk");
        simulator.add_walk("test_walk", "Test Walk".to_string());

        // Verify walk exists
        let walk = simulator.get_walk(&walk_id);
        assert!(walk.is_some());
        assert_eq!(walk.unwrap().title(), "Test Walk");
    }

    #[test]
    fn test_remove_walk() {
        let mut simulator = create_test_simulator();
        let walk_id = WalkId::new("test_walk");

        // Add and then remove a walk
        simulator.add_walk("test_walk", "Test Walk".to_string());
        let removed_walk = simulator.remove_walk(&walk_id);

        assert!(removed_walk.is_some());
        assert!(simulator.get_walk(&walk_id).is_none());
    }

    #[test]
    fn test_update_config() {
        let mut simulator = create_test_simulator();

        let new_config = SimulationConfig {
            risk_free_rate: Some(dec!(0.06)),
            dividend_yield: Some(pos!(0.03)),
            time_frame: TimeFrame::Week,
            volatility_window: 20,
            initial_volatility: Some(pos!(0.25)),
        };

        simulator.update_config(new_config);

        let config = simulator.get_config();
        assert_eq!(config.risk_free_rate, Some(dec!(0.06)));
        assert_eq!(config.dividend_yield, Some(pos!(0.03)));
        assert_eq!(config.volatility_window, 20);
        assert_eq!(config.initial_volatility, Some(pos!(0.25)));
    }

    #[test]
    fn test_multiple_walks() {
        let mut simulator = create_test_simulator();

        // Add multiple walks
        let walk_ids: Vec<WalkId> = (0..5)
            .map(|i| {
                let id = format!("walk_{}", i);
                simulator.add_walk(&id, format!("Walk {}", i));
                WalkId::new(id)
            })
            .collect();

        // Verify all walks were added
        assert_eq!(simulator.get_walk_ids().len(), 5);

        // Verify each walk exists
        for id in walk_ids {
            assert!(simulator.get_walk(&id).is_some());
        }
    }

    #[test]
    fn test_walk_modification() {
        let mut simulator = create_test_simulator();
        let walk_id = WalkId::new("test_walk");

        // Add walk and get mutable reference
        simulator.add_walk("test_walk", "Test Walk".to_string());
        let walk = simulator.get_walk_mut(&walk_id).unwrap();

        // Modify walk
        walk.generate_random_walk(10, pos!(100.0), 0.0, pos!(0.2), pos!(0.01))
            .unwrap();

        // Verify modification
        assert!(!walk.get_y_values().is_empty());
        assert_eq!(walk.get_y_values().len(), 10);
    }

    #[test]
    fn test_default_config() {
        let config = SimulationConfig::default();
        assert_eq!(config.risk_free_rate, None);
        assert_eq!(config.dividend_yield, None);
        assert_eq!(config.time_frame, TimeFrame::Day);
        assert_eq!(config.volatility_window, 4);
        assert_eq!(config.initial_volatility, None);
    }

    #[test]
    fn test_nonexistent_walk() {
        let simulator = create_test_simulator();
        let walk_id = WalkId::new("nonexistent");
        assert!(simulator.get_walk(&walk_id).is_none());
    }

    #[test]
    fn test_walk_id_creation_and_equality() {
        let id1 = WalkId::new("test");
        let id2 = WalkId::new("test");
        let id3 = WalkId::new("different");

        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }
}

#[cfg(test)]
mod tests_walk_id {
    use super::*;

    #[test]
    fn test_walk_id_creation() {
        let id = WalkId::new("test");
        assert_eq!(id.0, "test");
    }

    #[test]
    fn test_walk_id_clone() {
        let id1 = WalkId::new("test");
        let id2 = id1.clone();
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_walk_id_hash() {
        use std::collections::HashSet;

        let mut set = HashSet::new();
        let id1 = WalkId::new("test");
        let id2 = WalkId::new("test");
        let id3 = WalkId::new("different");

        set.insert(id1.clone());
        assert!(set.contains(&id2));
        assert!(!set.contains(&id3));
    }
}

#[cfg(test)]
mod tests_simulation_config {
    use super::*;
    use crate::pos;
    use rust_decimal_macros::dec;

    #[test]
    fn test_config_clone() {
        let config = SimulationConfig {
            risk_free_rate: Some(dec!(0.05)),
            dividend_yield: Some(pos!(0.02)),
            time_frame: TimeFrame::Day,
            volatility_window: 10,
            initial_volatility: Some(pos!(0.2)),
        };

        let cloned = config.clone();
        assert_eq!(cloned.risk_free_rate, config.risk_free_rate);
        assert_eq!(cloned.dividend_yield, config.dividend_yield);
        assert_eq!(cloned.time_frame, config.time_frame);
        assert_eq!(cloned.volatility_window, config.volatility_window);
        assert_eq!(cloned.initial_volatility, config.initial_volatility);
    }

    #[test]
    fn test_config_with_none_values() {
        let config = SimulationConfig {
            risk_free_rate: None,
            dividend_yield: None,
            time_frame: TimeFrame::Day,
            volatility_window: 10,
            initial_volatility: None,
        };

        assert!(config.risk_free_rate.is_none());
        assert!(config.dividend_yield.is_none());
        assert!(config.initial_volatility.is_none());
    }

    #[test]
    fn test_config_different_timeframes() {
        let timeframes = vec![
            TimeFrame::Day,
            TimeFrame::Week,
            TimeFrame::Month,
            TimeFrame::Year,
        ];

        for timeframe in timeframes {
            let config = SimulationConfig {
                risk_free_rate: None,
                dividend_yield: None,
                time_frame: timeframe,
                volatility_window: 10,
                initial_volatility: None,
            };
            assert_eq!(config.time_frame, timeframe);
        }
    }
}

#[cfg(test)]
mod tests_surfacable {
    use super::*;
    use crate::pos;
    use crate::utils::time::TimeFrame;

    use rust_decimal_macros::dec;

    // Helper function to create a test simulator with walks
    fn create_test_simulator() -> Simulator {
        let config = SimulationConfig {
            risk_free_rate: None,
            dividend_yield: None,
            time_frame: TimeFrame::Day,
            volatility_window: 4,
            initial_volatility: None,
        };

        let mut simulator = Simulator::new(config);

        // Add two walks with known values
        let walk1 = simulator.add_walk("WALK1", "First Walk".to_string());
        walk1.values = vec![pos!(1.0), pos!(2.0), pos!(3.0)];

        let walk2 = simulator.add_walk("WALK2", "Second Walk".to_string());
        walk2.values = vec![pos!(4.0), pos!(5.0), pos!(6.0)];

        simulator
    }

    #[test]
    fn test_surface_empty_simulator() {
        let simulator = Simulator::new(SimulationConfig::default());
        let result = simulator.surface();
        assert!(result.is_ok());
        assert_eq!(result.unwrap().points.len(), 0);
    }

    #[test]
    fn test_surface_with_single_walk() {
        let mut simulator = Simulator::new(SimulationConfig::default());
        let walk = simulator.add_walk("WALK1", "Single Walk".to_string());
        walk.values = vec![pos!(1.0), pos!(2.0), pos!(3.0)];

        let surface = simulator.surface().unwrap();
        assert_eq!(surface.points.len(), 3);

        // Convert points to Vec for easier testing
        let points: Vec<_> = surface.points.iter().collect();

        // Check points - z coordinate (i) should be 0 for all points as it's the first walk
        assert_eq!(points[0].x, dec!(0)); // i coordinate
        assert_eq!(points[0].y, dec!(0)); // x coordinate from point
        assert_eq!(points[0].z, dec!(1.0)); // y coordinate from point

        assert_eq!(points[1].x, dec!(0));
        assert_eq!(points[1].y, dec!(1));
        assert_eq!(points[1].z, dec!(2.0));
    }

    #[test]
    fn test_surface_with_multiple_walks() {
        let simulator = create_test_simulator();
        let surface = simulator.surface().unwrap();

        // Should have 6 points total (3 points from each walk)
        assert_eq!(surface.points.len(), 6);

        // Convert to Vec for easier testing
        let points: Vec<_> = surface.points.iter().collect();

        // Check points from first walk (i = 0)
        let walk1_points: Vec<_> = points.iter().filter(|p| p.x == dec!(0)).collect();
        assert_eq!(walk1_points.len(), 3);

        // Check points from second walk (i = 1)
        let walk2_points: Vec<_> = points.iter().filter(|p| p.x == dec!(1)).collect();
        assert_eq!(walk2_points.len(), 3);
    }

    #[test]
    fn test_surface_point_ordering() {
        let simulator = create_test_simulator();
        let surface = simulator.surface().unwrap();
        let points: Vec<_> = surface.points.iter().collect();

        // Points should be ordered first by walk index (x), then by time (y)
        for i in 0..points.len() - 1 {
            if points[i].x == points[i + 1].x {
                assert!(points[i].y <= points[i + 1].y);
            } else {
                assert!(points[i].x < points[i + 1].x);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pos;
    use crate::visualization::utils::Graph;
    use rust_decimal_macros::dec;
    use std::collections::HashMap;

    // Helper to create a test simulator with a standard configuration
    fn create_test_simulator() -> Simulator {
        let config = SimulationConfig {
            risk_free_rate: Some(dec!(0.05)),
            dividend_yield: Some(pos!(0.02)),
            time_frame: TimeFrame::Day,
            volatility_window: 10,
            initial_volatility: Some(pos!(0.2)),
        };
        Simulator::new(config)
    }

    #[test]
    fn test_simulator_creation() {
        let simulator = create_test_simulator();

        // Verify no walks exist initially
        assert!(simulator.get_walk_ids().is_empty());

        // Verify config values are correct
        let config = simulator.get_config();
        assert_eq!(config.risk_free_rate, Some(dec!(0.05)));
        assert_eq!(config.dividend_yield, Some(pos!(0.02)));
        assert_eq!(config.time_frame, TimeFrame::Day);
        assert_eq!(config.volatility_window, 10);
        assert_eq!(config.initial_volatility, Some(pos!(0.2)));
    }

    #[test]
    fn test_add_walk() {
        let mut simulator = create_test_simulator();

        // Add a walk
        let walk = simulator.add_walk("test_walk", "Test Walk".to_string());

        // Check that the walk was initialized correctly
        assert_eq!(walk.title(), "Test Walk");

        // Verify walk exists in simulator
        let walk_id = WalkId::new("test_walk");
        assert!(simulator.get_walk(&walk_id).is_some());
    }

    #[test]
    fn test_get_walk() {
        let mut simulator = create_test_simulator();

        // Add a walk and retrieve it
        simulator.add_walk("test_walk", "Test Walk".to_string());
        let walk_id = WalkId::new("test_walk");

        let walk = simulator.get_walk(&walk_id);
        assert!(walk.is_some());
        assert_eq!(walk.unwrap().title(), "Test Walk");

        // Try retrieving a non-existent walk
        let nonexistent_id = WalkId::new("nonexistent");
        assert!(simulator.get_walk(&nonexistent_id).is_none());
    }

    #[test]
    fn test_get_walk_mut() {
        let mut simulator = create_test_simulator();

        // Add a walk and retrieve mutable reference
        simulator.add_walk("test_walk", "Test Walk".to_string());
        let walk_id = WalkId::new("test_walk");

        // Modify the walk title using mutable reference
        let walk = simulator.get_walk_mut(&walk_id);
        assert!(walk.is_some());
        let _walk = walk.unwrap();
        // Assuming there's a method to update the title or some other property
        // walk.set_title("Updated Title".to_string());

        // Verify the walk was updated
        let updated_walk = simulator.get_walk(&walk_id);
        assert!(updated_walk.is_some());
        // assert_eq!(updated_walk.unwrap().title(), "Updated Title");
    }

    #[test]
    fn test_remove_walk() {
        let mut simulator = create_test_simulator();

        // Add a walk
        simulator.add_walk("test_walk", "Test Walk".to_string());
        let walk_id = WalkId::new("test_walk");

        // Verify walk exists
        assert!(simulator.get_walk(&walk_id).is_some());

        // Remove the walk
        let removed_walk = simulator.remove_walk(&walk_id);

        // Verify removal was successful
        assert!(removed_walk.is_some());
        assert_eq!(removed_walk.unwrap().title(), "Test Walk");
        assert!(simulator.get_walk(&walk_id).is_none());

        // Try removing a non-existent walk
        let nonexistent_id = WalkId::new("nonexistent");
        assert!(simulator.remove_walk(&nonexistent_id).is_none());
    }

    #[test]
    fn test_get_walk_ids() {
        let mut simulator = create_test_simulator();

        // Initially no walks
        assert!(simulator.get_walk_ids().is_empty());

        // Add multiple walks
        simulator.add_walk("walk1", "Walk 1".to_string());
        simulator.add_walk("walk2", "Walk 2".to_string());
        simulator.add_walk("walk3", "Walk 3".to_string());

        // Get IDs and verify count
        let ids = simulator.get_walk_ids();
        assert_eq!(ids.len(), 3);

        // Verify all IDs are present
        let id_strings: Vec<String> = ids.iter().map(|id| id.as_str().to_string()).collect();
        assert!(id_strings.contains(&"walk1".to_string()));
        assert!(id_strings.contains(&"walk2".to_string()));
        assert!(id_strings.contains(&"walk3".to_string()));
    }

    #[test]
    fn test_update_config() {
        let mut simulator = create_test_simulator();

        // Create new config
        let new_config = SimulationConfig {
            risk_free_rate: Some(dec!(0.06)),
            dividend_yield: Some(pos!(0.03)),
            time_frame: TimeFrame::Week,
            volatility_window: 20,
            initial_volatility: Some(pos!(0.25)),
        };

        // Update config
        simulator.update_config(new_config);

        // Verify config is updated
        let config = simulator.get_config();
        assert_eq!(config.risk_free_rate, Some(dec!(0.06)));
        assert_eq!(config.dividend_yield, Some(pos!(0.03)));
        assert_eq!(config.time_frame, TimeFrame::Week);
        assert_eq!(config.volatility_window, 20);
        assert_eq!(config.initial_volatility, Some(pos!(0.25)));
    }

    #[test]
    fn test_generate_random_walks() {
        let mut simulator = create_test_simulator();

        // Add walks
        simulator.add_walk("walk1", "Walk 1".to_string());
        simulator.add_walk("walk2", "Walk 2".to_string());

        // Setup initial prices
        let mut initial_prices = HashMap::new();
        initial_prices.insert(WalkId::new("walk1"), pos!(100.0));
        initial_prices.insert(WalkId::new("walk2"), pos!(200.0));

        // Generate random walks
        let result = simulator.generate_random_walks(
            10, // n_steps
            &initial_prices,
            0.0,        // mean
            pos!(0.2),  // std_dev
            pos!(0.01), // std_dev_change
        );

        assert!(result.is_ok());

        // Verify walks have data
        let walk1 = simulator.get_walk(&WalkId::new("walk1")).unwrap();
        let walk2 = simulator.get_walk(&WalkId::new("walk2")).unwrap();

        assert_eq!(walk1.get_y_values().len(), 10);
        assert_eq!(walk2.get_y_values().len(), 10);

        // First value should be the initial price
        assert_eq!(walk1.get_y_values()[0], pos!(100.0));
        assert_eq!(walk2.get_y_values()[0], pos!(200.0));
    }

    #[test]
    fn test_generate_random_walks_missing_initial_price() {
        let mut simulator = create_test_simulator();

        // Add walks
        simulator.add_walk("walk1", "Walk 1".to_string());
        simulator.add_walk("walk2", "Walk 2".to_string());

        // Setup incomplete initial prices
        let mut initial_prices = HashMap::new();
        initial_prices.insert(WalkId::new("walk1"), pos!(100.0));
        // Missing initial price for walk2

        // Attempt to generate random walks
        let result =
            simulator.generate_random_walks(10, &initial_prices, 0.0, pos!(0.2), pos!(0.01));

        // Should fail due to missing initial price
        assert!(result.is_err());
        let error = result.unwrap_err().to_string();
        assert!(error.contains("No initial price provided for walk"));
    }

    #[test]
    fn test_surface_generation() {
        let mut simulator = create_test_simulator();

        // Add walks with known values
        let walk1 = simulator.add_walk("walk1", "Walk 1".to_string());
        walk1.values = vec![pos!(1.0), pos!(2.0), pos!(3.0)];

        let walk2 = simulator.add_walk("walk2", "Walk 2".to_string());
        walk2.values = vec![pos!(4.0), pos!(5.0), pos!(6.0)];

        // Generate surface
        let surface = simulator.surface();
        assert!(surface.is_ok());

        let surface = surface.unwrap();
        assert_eq!(surface.points.len(), 6); // 3 points from each walk

        // Convert points to Vec for easier testing
        let points: Vec<_> = surface.points.iter().collect();

        // First walk points (i=0)
        assert_eq!(points[0].x, dec!(0)); // Walk index
        assert_eq!(points[0].y, dec!(0)); // Point index within walk
        assert_eq!(points[0].z, dec!(1.0)); // Value

        assert_eq!(points[1].x, dec!(0));
        assert_eq!(points[1].y, dec!(1));
        assert_eq!(points[1].z, dec!(2.0));

        assert_eq!(points[2].x, dec!(0));
        assert_eq!(points[2].y, dec!(2));
        assert_eq!(points[2].z, dec!(3.0));

        // Second walk points (i=1)
        assert_eq!(points[3].x, dec!(1));
        assert_eq!(points[3].y, dec!(0));
        assert_eq!(points[3].z, dec!(4.0));

        assert_eq!(points[4].x, dec!(1));
        assert_eq!(points[4].y, dec!(1));
        assert_eq!(points[4].z, dec!(5.0));

        assert_eq!(points[5].x, dec!(1));
        assert_eq!(points[5].y, dec!(2));
        assert_eq!(points[5].z, dec!(6.0));
    }
}

// Separate test module for WalkId
#[cfg(test)]
mod walk_id_tests {
    use super::*;
    use std::collections::{HashMap, HashSet};

    #[test]
    fn test_walk_id_creation() {
        let id = WalkId::new("test_walk");
        assert_eq!(id.as_str(), "test_walk");

        // Test with different types that can convert to String
        let id_from_string = WalkId::new(String::from("test_walk"));
        assert_eq!(id_from_string.as_str(), "test_walk");
    }

    #[test]
    fn test_walk_id_equality() {
        let id1 = WalkId::new("test_walk");
        let id2 = WalkId::new("test_walk");
        let id3 = WalkId::new("different_walk");

        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
        assert_ne!(id2, id3);
    }

    #[test]
    fn test_walk_id_clone() {
        let id1 = WalkId::new("test_walk");
        let id2 = id1.clone();

        assert_eq!(id1, id2);
    }

    #[test]
    fn test_walk_id_hash() {
        let id1 = WalkId::new("test_walk");
        let id2 = WalkId::new("test_walk");
        let id3 = WalkId::new("different_walk");

        // Test in HashSet
        let mut set = HashSet::new();
        set.insert(id1.clone());

        assert!(set.contains(&id2));
        assert!(!set.contains(&id3));

        // Test in HashMap
        let mut map = HashMap::new();
        map.insert(id1, "value1");

        assert!(map.contains_key(&id2));
        assert!(!map.contains_key(&id3));
    }

    #[test]
    fn test_walk_id_as_str() {
        let id = WalkId::new("test_walk");
        assert_eq!(id.as_str(), "test_walk");

        // Test with string containing special characters
        let id_special = WalkId::new("test-walk@123");
        assert_eq!(id_special.as_str(), "test-walk@123");
    }
}

// Separate test module for SimulationConfig
#[cfg(test)]
mod simulation_config_tests {
    use super::*;
    use crate::pos;
    use rust_decimal_macros::dec;

    #[test]
    fn test_simulation_config_default() {
        let config = SimulationConfig::default();

        // Verify default values
        assert_eq!(config.risk_free_rate, None);
        assert_eq!(config.dividend_yield, None);
        assert_eq!(config.time_frame, TimeFrame::Day);
        assert_eq!(config.volatility_window, 4);
        assert_eq!(config.initial_volatility, None);
    }

    #[test]
    fn test_simulation_config_custom() {
        let config = SimulationConfig {
            risk_free_rate: Some(dec!(0.05)),
            dividend_yield: Some(pos!(0.02)),
            time_frame: TimeFrame::Week,
            volatility_window: 10,
            initial_volatility: Some(pos!(0.2)),
        };

        // Verify custom values
        assert_eq!(config.risk_free_rate, Some(dec!(0.05)));
        assert_eq!(config.dividend_yield, Some(pos!(0.02)));
        assert_eq!(config.time_frame, TimeFrame::Week);
        assert_eq!(config.volatility_window, 10);
        assert_eq!(config.initial_volatility, Some(pos!(0.2)));
    }

    #[test]
    fn test_simulation_config_clone() {
        let config = SimulationConfig {
            risk_free_rate: Some(dec!(0.05)),
            dividend_yield: Some(pos!(0.02)),
            time_frame: TimeFrame::Month,
            volatility_window: 10,
            initial_volatility: Some(pos!(0.2)),
        };

        // Clone the config
        let cloned_config = config.clone();

        // Verify cloned values match original
        assert_eq!(cloned_config.risk_free_rate, config.risk_free_rate);
        assert_eq!(cloned_config.dividend_yield, config.dividend_yield);
        assert_eq!(cloned_config.time_frame, config.time_frame);
        assert_eq!(cloned_config.volatility_window, config.volatility_window);
        assert_eq!(cloned_config.initial_volatility, config.initial_volatility);
    }

    #[test]
    fn test_simulation_config_with_different_timeframes() {
        // Test with Day timeframe
        let config_day = SimulationConfig {
            risk_free_rate: None,
            dividend_yield: None,
            time_frame: TimeFrame::Day,
            volatility_window: 5,
            initial_volatility: None,
        };
        assert_eq!(config_day.time_frame, TimeFrame::Day);

        // Test with Week timeframe
        let config_week = SimulationConfig {
            risk_free_rate: None,
            dividend_yield: None,
            time_frame: TimeFrame::Week,
            volatility_window: 5,
            initial_volatility: None,
        };
        assert_eq!(config_week.time_frame, TimeFrame::Week);

        // Test with Month timeframe
        let config_month = SimulationConfig {
            risk_free_rate: None,
            dividend_yield: None,
            time_frame: TimeFrame::Month,
            volatility_window: 5,
            initial_volatility: None,
        };
        assert_eq!(config_month.time_frame, TimeFrame::Month);

        // Test with Year timeframe
        let config_year = SimulationConfig {
            risk_free_rate: None,
            dividend_yield: None,
            time_frame: TimeFrame::Year,
            volatility_window: 5,
            initial_volatility: None,
        };
        assert_eq!(config_year.time_frame, TimeFrame::Year);
    }

    #[test]
    fn test_simulation_config_partial_initialization() {
        // Test with only some fields specified
        let config = SimulationConfig {
            risk_free_rate: Some(dec!(0.05)),
            dividend_yield: None,
            time_frame: TimeFrame::Day,
            volatility_window: 10,
            initial_volatility: None,
        };

        assert_eq!(config.risk_free_rate, Some(dec!(0.05)));
        assert_eq!(config.dividend_yield, None);
        assert_eq!(config.initial_volatility, None);
    }
}
