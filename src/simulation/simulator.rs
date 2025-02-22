/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 22/2/25
******************************************************************************/

use crate::simulation::{RandomWalkGraph, Walkable};
use crate::utils::time::TimeFrame;
use crate:: Positive;
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use plotters::backend::BitMapBackend;
use plotters::prelude::{ChartBuilder, IntoDrawingArea, IntoFont, LineSeries, PathElement, SeriesLabelPosition, BLACK, WHITE};
use crate::visualization::utils::{random_color, GraphBackend};
use plotters::style::Color;
use tracing::{debug, info, warn};

/// Configuration for a random walk simulation.
/// This struct contains shared configuration that can be reused across multiple walks.
#[derive(Clone)]
pub struct SimulationConfig {
    pub risk_free_rate: Option<Decimal>,
    pub dividend_yield: Option<Positive>,
    pub time_frame: TimeFrame,
    pub volatility_window: usize,
    pub initial_volatility: Option<Positive>,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            risk_free_rate: None,
            dividend_yield: None,
            time_frame: TimeFrame::Day,
            volatility_window: 4,
            initial_volatility: None,
        }
    }
}

/// A named identifier for a specific random walk within the simulator
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct WalkId(String);

impl WalkId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Main simulator struct that manages multiple random walks
pub struct Simulator {
    /// Shared configuration across walks
    config: Arc<SimulationConfig>,
    /// Map of walk IDs to their corresponding graphs
    walks: HashMap<WalkId, RandomWalkGraph>,
    /// Optional correlation matrix between walks
    correlations: Option<HashMap<(WalkId, WalkId), f64>>,
}

impl Simulator {
    /// Creates a new simulator with the given configuration
    pub fn new(config: SimulationConfig) -> Self {
        Self {
            config: Arc::new(config),
            walks: HashMap::new(),
            correlations: None,
        }
    }

    /// Adds a new random walk to the simulator
    pub fn add_walk(&mut self, id: impl Into<String>, title: String) -> &mut RandomWalkGraph {
        let walk_id = WalkId::new(id);
        let walk = RandomWalkGraph::new(
            title,
            self.config.risk_free_rate,
            self.config.dividend_yield,
            self.config.time_frame,
            self.config.volatility_window,
            self.config.initial_volatility,
        );

        self.walks.entry(walk_id.clone()).or_insert(walk)
    }

    /// Removes a walk from the simulator
    pub fn remove_walk(&mut self, id: &WalkId) -> Option<RandomWalkGraph> {
        self.walks.remove(id)
    }

    /// Gets a reference to a walk by its ID
    pub fn get_walk(&self, id: &WalkId) -> Option<&RandomWalkGraph> {
        self.walks.get(id)
    }

    /// Gets a mutable reference to a walk by its ID
    pub fn get_walk_mut(&mut self, id: &WalkId) -> Option<&mut RandomWalkGraph> {
        self.walks.get_mut(id)
    }

    /// Sets the correlation between two walks
    pub fn set_correlation(&mut self, _id1: &WalkId, _id2: &WalkId) {
        if self.correlations.is_none() {
            self.correlations = Some(HashMap::new());
        }
        todo!("Set correlation between two walks");
    }

    pub fn generate_random_walks(
        &mut self,
        n_steps: usize,
        initial_prices: &HashMap<WalkId, Positive>,
        mean: f64,
        std_dev: Positive,
        std_dev_change: Positive,
    ) -> Result<(), Box<dyn Error>> {
        let results: Result<Vec<_>, _> = self.walks
            .iter_mut()
            .map(|(id, walk)| {
                let initial_price = initial_prices.get(id)
                    .ok_or_else(|| format!("No initial price provided for walk {}", id.as_str()))?;

                walk.generate_random_walk(
                    n_steps,
                    *initial_price,
                    mean,
                    std_dev,
                    std_dev_change,
                )
            })
            .collect();

        results?;
        Ok(())
    }

    /// Gets all walk IDs in the simulator
    pub fn get_walk_ids(&self) -> Vec<WalkId> {
        self.walks.keys().cloned().collect()
    }

    /// Gets the current configuration
    pub fn get_config(&self) -> &SimulationConfig {
        &self.config
    }

    /// Updates the configuration
    pub fn update_config(&mut self, new_config: SimulationConfig) {
        self.config = Arc::new(new_config);
    }

    /// Plots all random walks in the simulator on a single graph.
    ///
    /// # Parameters
    /// - `file_path`: Path where the graph will be saved
    /// - `plot_size`: Size of the plot as (width, height)
    /// - `title_size`: Size of the title font
    ///
    /// # Returns
    /// - `Result<(), Box<dyn Error>>`: Ok if successful, Error otherwise
    pub fn graph(
        &self,
        backend: GraphBackend,
        title_size: u32,
    ) -> Result<(), Box<dyn Error>> {
        // Validate that there are walks to plot
        if self.walks.is_empty() {
            return Err("No walks to plot".into());
        }

        let root = match backend {
            #[cfg(not(target_arch = "wasm32"))]
            GraphBackend::Bitmap { file_path, size } => {
                let root = BitMapBackend::new(file_path, size).into_drawing_area();
                root.fill(&WHITE)?;
                root
            }
            #[cfg(target_arch = "wasm32")]
            GraphBackend::Canvas { canvas } => {
                let root = CanvasBackend::with_canvas_object(canvas)
                    .unwrap()
                    .into_drawing_area();
                root.fill(&WHITE)?;
                root
            }
        };

        // Collect and validate data points
        let mut all_points = Vec::new();
        for (id, walk) in &self.walks {
            
            let walk_points = walk.get_points();
            info!("Walk points length: {}", walk_points.len());

            if !walk_points.is_empty() {
                for (index, point) in walk_points.iter().enumerate() {
                    debug!("Point {}: {:?}", index, point.coordinates);
                }
                all_points.extend(walk_points);
            } else {
                warn!("No points for walk {}", id.as_str());
            }
        }

        // Ensure we have points to plot
        if all_points.is_empty() {
            for (id, walk) in &self.walks {
                let points = walk.get_points();
                debug!("Walk {}: {} points", id.as_str(), points.len());
            }
            return Err("No data points to plot".into());
        }

        // Calculate plot ranges with a small buffer
        let x_values: Vec<f64> = all_points.iter().map(|p| p.coordinates.0).collect();
        let y_values: Vec<f64> = all_points.iter().map(|p| p.coordinates.1.to_f64()).collect();

        let x_min = x_values.iter().cloned().fold(f64::INFINITY, |a, b| a.min(b));
        let x_max = x_values.iter().cloned().fold(f64::NEG_INFINITY, |a, b| a.max(b));
        let y_min = y_values.iter().cloned().fold(f64::INFINITY, |a, b| a.min(b));
        let y_max = y_values.iter().cloned().fold(f64::NEG_INFINITY, |a, b| a.max(b));


        // Add a small buffer to the ranges
        let x_buffer = (x_max - x_min) * 0.05;
        let y_buffer = (y_max - y_min) * 0.05;

        // Create chart
        let mut chart = ChartBuilder::on(&root)
            .caption("Random Walks Simulation", ("sans-serif", title_size).into_font())
            .margin(10)
            .x_label_area_size(40)
            .y_label_area_size(40)
            .build_cartesian_2d(
                (x_min - x_buffer)..(x_max + x_buffer),
                (y_min - y_buffer)..(y_max + y_buffer)
            )?;

        // Configure and draw mesh
        chart
            .configure_mesh()
            .x_desc("Time")
            .y_desc("Value")
            .draw()?;

        // Draw each random walk with a different color
        for (id, walk) in &self.walks {
            let walk_points = walk.get_points();
            let color = random_color();

            chart
                .draw_series(LineSeries::new(
                    walk_points.iter().map(|p| (p.coordinates.0, p.coordinates.1.to_f64())),
                    color.clone().stroke_width(2),
                ))?
                .label(id.as_str())
                .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], color));
        }

        // Draw the legend
        chart
            .configure_series_labels()
            .background_style(&WHITE.mix(0.8))
            .border_style(&BLACK)
            .position(SeriesLabelPosition::UpperRight)
            .draw()?;

        // Render the chart
        root.present()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests_simulator {
    use super::*;
    use rust_decimal_macros::dec;
    use crate::utils::time::TimeFrame;
    use crate::pos;
    use crate::simulation::Walkable;
    use crate::visualization::utils::Graph;

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
    fn test_set_and_get_correlation() {
        let mut simulator = create_test_simulator();

        let walk_id1 = WalkId::new("walk1");
        let walk_id2 = WalkId::new("walk2");

        simulator.add_walk("walk1", "Walk 1".to_string());
        simulator.add_walk("walk2", "Walk 2".to_string());

        simulator.set_correlation(&walk_id1, &walk_id2, 0.5);

        if let Some(correlations) = &simulator.correlations {
            assert_eq!(correlations.get(&(walk_id1.clone(), walk_id2.clone())), Some(&0.5));
            // Test symmetric correlation
            assert_eq!(correlations.get(&(walk_id2.clone(), walk_id1.clone())), Some(&0.5));
        } else {
            panic!("Correlations should be initialized");
        }
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
        walk.generate_random_walk(
            10,
            pos!(100.0),
            0.0,
            pos!(0.2),
            pos!(0.01)
        ).unwrap();

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
                time_frame: timeframe.clone(),
                volatility_window: 10,
                initial_volatility: None,
            };
            assert_eq!(config.time_frame, timeframe);
        }
    }
}