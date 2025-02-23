/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 22/2/25
******************************************************************************/

use crate::curves::Curvable;
use crate::error::SurfaceError;
use crate::simulation::{RandomWalkGraph, Walkable};
use crate::surfaces::{Point3D, Surfacable, Surface};
use crate::utils::time::TimeFrame;
use crate::visualization::utils::{random_color, GraphBackend};
use crate::Positive;
#[cfg(not(target_arch = "wasm32"))]
use plotters::backend::BitMapBackend;
use plotters::prelude::{
    ChartBuilder, IntoDrawingArea, IntoFont, LineSeries, PathElement, SeriesLabelPosition, BLACK,
    WHITE,
};
use plotters::style::Color;
#[cfg(target_arch = "wasm32")]
use plotters_canvas::CanvasBackend;
use rust_decimal::Decimal;
use std::collections::{BTreeSet, HashMap};
use std::error::Error;
use std::sync::Arc;
use tracing::{debug, warn};

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
        let results: Result<Vec<_>, _> = self
            .walks
            .iter_mut()
            .map(|(id, walk)| {
                let initial_price = initial_prices
                    .get(id)
                    .ok_or_else(|| format!("No initial price provided for walk {}", id.as_str()))?;

                walk.generate_random_walk(n_steps, *initial_price, mean, std_dev, std_dev_change)
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
    pub fn graph(&self, backend: GraphBackend, title_size: u32) -> Result<(), Box<dyn Error>> {
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
            debug!("Walk points length: {}", walk_points.len());

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
        let y_values: Vec<f64> = all_points
            .iter()
            .map(|p| p.coordinates.1.to_f64())
            .collect();

        let x_min = x_values
            .iter()
            .cloned()
            .fold(f64::INFINITY, |a, b| a.min(b));
        let x_max = x_values
            .iter()
            .cloned()
            .fold(f64::NEG_INFINITY, |a, b| a.max(b));
        let y_min = y_values
            .iter()
            .cloned()
            .fold(f64::INFINITY, |a, b| a.min(b));
        let y_max = y_values
            .iter()
            .cloned()
            .fold(f64::NEG_INFINITY, |a, b| a.max(b));

        // Add a small buffer to the ranges
        let x_buffer = (x_max - x_min) * 0.05;
        let y_buffer = (y_max - y_min) * 0.05;

        // Create chart
        let mut chart = ChartBuilder::on(&root)
            .caption(
                "Random Walks Simulation",
                ("sans-serif", title_size).into_font(),
            )
            .margin(10)
            .x_label_area_size(40)
            .y_label_area_size(40)
            .build_cartesian_2d(
                (x_min - x_buffer)..(x_max + x_buffer),
                (y_min - y_buffer)..(y_max + y_buffer),
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
                    walk_points
                        .iter()
                        .map(|p| (p.coordinates.0, p.coordinates.1.to_f64())),
                    color.clone().stroke_width(2),
                ))?
                .label(id.as_str())
                .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], color));
        }

        // Draw the legend
        chart
            .configure_series_labels()
            .background_style(WHITE.mix(0.8))
            .border_style(BLACK)
            .position(SeriesLabelPosition::UpperRight)
            .draw()?;

        // Render the chart
        root.present()?;
        Ok(())
    }
}

/// Implementation of the `Surfacable` trait for the `Simulator` type.
///
/// This implementation allows the `Simulator` object to generate a [`Surface`]
/// representation based on the random walks it manages.
///
/// # Process
/// The `surface` method generates a surface by:
/// 1. Iterating over all random walks in the simulator.
/// 2. Extracting their associated `Curve` points via each walk's `curve()` method.
/// 3. Mapping these points into a collection of three-dimensional points, [`Point3D`],
///    where each point's `z` coordinate is represented by the index of the walk,
///    and its `x` and `y` coordinates are taken from the walk's `Curve` data.
/// 4. Constructing a [`Surface`] using the resulting set of [`Point3D`] points.
///
/// # Returns
/// - **`Ok(Surface)`**: If the surface is successfully created from the `Point3D` points.
/// - **`Err(SurfaceError)`**: If any errors occur during surface construction. Possible
///   errors include:
///   - Invalid point conversion via [`Point3D::from_tuple`] (e.g., invalid coordinate transformations).
///   - Failures in the associated random walks' `curve()` methods.
///   - Issues during the collection or instantiation of the final surface.
///
/// # Notes
/// - The resulting surface's points are stored in a `BTreeSet`, which inherently ensures
///   that the points are sorted and unique. This provides a natural order and prevents
///   duplicate points from being included.
/// - The `z` coordinate of each `Point3D` is determined by the index of the walk in the simulator.
/// - This implementation heavily relies on the [`Surface::new`] and [`Point3D::from_tuple`]
///   helper methods.
///
/// # Implementation Details
/// - `Simulator` maintains its walks in a `HashMap`. The `surface` method iterates through the
///   walks using the `enumerate()` function, which provides a unique index for each walk.
/// - The method uses the `flat_map()` iterator to efficiently transform the collection of walks
///   into the desired set of points.
///
/// # Errors
/// The method returns a [`SurfaceError`] in any of the following cases:
/// - If the `curve()` method of a random walk fails (e.g., invalid curve generation or
///   missing values).
/// - If a conversion error occurs while creating `Point3D` instances (e.g., invalid
///   input arguments).
/// - If issues occur while constructing the `Surface` itself.
///
/// # Example
/// This implementation allows the `Simulator` to generate a 3D surface representation of
/// random walks, which can subsequently be visualized, analyzed, or processed.
///
/// # See Also
/// - [`Surface`]: The resulting 3D surface representation.
/// - [`Point3D`]: Used to represent points in 3D space in the generated surface.
/// - [`SurfaceError`]: Enumerates possible error types during surface generation.
impl Surfacable for Simulator {
    fn surface(&self) -> Result<Surface, SurfaceError> {
        let points: BTreeSet<Point3D> = self
            .walks
            .iter()
            .enumerate()
            .flat_map(|(i, (_, walk))| {
                let curve = walk.curve().unwrap();
                let points2d = curve.points;

                points2d
                    .into_iter()
                    .map(move |point| Point3D::from_tuple(i, point.x, point.y).unwrap())
            })
            .collect();

        Ok(Surface::new(points))
    }
}

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

    #[test]
    fn test_surface_walks_with_different_lengths() {
        let mut simulator = Simulator::new(SimulationConfig::default());

        // Add walks with different lengths
        let walk1 = simulator.add_walk("WALK1", "Short Walk".to_string());
        walk1.values = vec![pos!(1.0), pos!(2.0)];

        let walk2 = simulator.add_walk("WALK2", "Long Walk".to_string());
        walk2.values = vec![pos!(3.0), pos!(4.0), pos!(5.0), pos!(6.0)];

        let surface = simulator.surface().unwrap();
        assert_eq!(surface.points.len(), 6); // Total points from both walks

        // Verify points from each walk
        let points: Vec<_> = surface.points.iter().collect();
        let walk1_points: Vec<_> = points.iter().filter(|p| p.x == dec!(0)).collect();
        let walk2_points: Vec<_> = points.iter().filter(|p| p.x == dec!(1)).collect();

        assert_eq!(walk1_points.len(), 2);
        assert_eq!(walk2_points.len(), 4);
    }
}
