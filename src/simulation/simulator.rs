/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 22/2/25
******************************************************************************/
#[cfg(test)]
mod test_simulator;

use crate::curves::Curvable;
use crate::error::SurfaceError;
use crate::simulation::model::SimulationResult;
use crate::simulation::{RandomWalkGraph, Walkable};
use crate::strategies::Strategable;
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
use rust_decimal::{Decimal, MathematicalOps};
use std::collections::{BTreeSet, HashMap};
use std::error::Error;
use std::sync::Arc;
use num_traits::{FromPrimitive, ToPrimitive};
use crate::pnl::model::PnLRange;
use crate::risk::RiskMetricsSimulation;

/// Represents the configuration for a random walk simulation.
///
/// This configuration struct is designed to standardize and unify simulation parameters
/// necessary for random walk generation in financial or mathematical applications.
/// It can be reused across multiple simulation instances to maintain consistent settings.
///
/// # Fields
///
/// - `risk_free_rate`:
///   Optional field representing the annualized risk-free interest rate. Typically used in
///   financial modeling for discounting and theoretical pricing.
///
/// - `dividend_yield`:
///   Optional field representing the annualized rate of dividends for an asset, expressed
///   as a percentage. This value is modeled as a `Positive` type to ensure it is non-negative.
///
/// - `time_frame`:
///   An instance of `TimeFrame`, which specifies the granularity of data or intervals
///   used in the simulation (e.g., daily, monthly, yearly). Custom timeframes can also be
///   defined using positive values for periods per year.
///
/// - `volatility_window`:
///   Specifies the rolling window size used for volatility calculations, such as historical
///   volatility estimation or stochastic volatility modeling. This value defines the
///   number of data points considered in rolling computations.
///
/// - `initial_volatility`:
///   Optional field representing the starting volatility of the asset or system being modeled.
///   This volatility acts as a baseline for time-varying processes and is constrained to be non-negative
///   using the `Positive` type.
///
/// # Remarks
///
/// - By encapsulating these parameters in a single struct, the configuration provides
///   a modular and flexible design for managing simulation settings.
/// - The use of `Option` for certain fields allows for default or optional parameters, reducing
///   the burden of defining every value.
/// - The `Positive` type ensures that inherently non-negative values (e.g., `dividend_yield`)
///   are appropriately constrained during compile-time or runtime.
///
/// # Example Use Cases
///
/// - Configuring a simulation for a geometric Brownian motion model of asset.
/// - Applying a stochastic volatility process with an initial baseline volatility.
/// - Managing simulation parameters across multiple timeframes for comparative analysis.
///
/// # Related Types
///
/// - `Positive`: Encapsulates a strictly positive `Decimal` value, ensuring constraints on
///   non-negative financial or mathematical parameters.
/// - `TimeFrame`: Enumerates various standard and custom timeframes that can be applied
///   to simulations for data scaling or aggregation.
#[derive(Clone)]
pub struct SimulationConfig {
    pub risk_free_rate: Option<Decimal>,
    pub dividend_yield: Option<Positive>,
    pub time_frame: TimeFrame,
    pub volatility_window: usize,
    pub initial_volatility: Option<Positive>,
}

/// Implements the `Default` trait for the `SimulationConfig` struct.
///
/// This implementation provides default values for a `SimulationConfig` object,
/// used to configure random walk simulations. These defaults are designed to be
/// reasonable starting points but can be customized as needed.
///
/// ## Default Values
/// - `risk_free_rate`: `None`
/// - `dividend_yield`: `None`
/// - `time_frame`: `TimeFrame::Day`
/// - `volatility_window`: `4`
/// - `initial_volatility`: `None`
///
/// These defaults assume that:
/// - The risk-free rate and dividend yield are not specified (`None`).
/// - The simulation will operate on daily data (`TimeFrame::Day`).
/// - The volatility will be calculated based on a rolling window of size `4`.
/// - The initial volatility is not pre-defined.
///
/// The `Default` implementation allows the creation of `SimulationConfig`
/// with minimal specification, providing flexibility for specialized configurations.
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

/// Represents a named identifier for tracking specific random walks.
///
/// This structure is used to uniquely identify random walk simulations within the framework.
/// Its primary purpose is to associate a descriptive, human-readable name to each random walk,
/// facilitating easier management, logging, and debugging in the context of financial simulations
/// or other stochastic processes.
///
/// ## Key Features:
/// - **Unique Identification**: Ensures random walks are distinctly named.
/// - **Debugging Support**: Implements the `Debug` trait for easier introspection.
/// - **Cloneable**: Implements the `Clone` trait to allow duplication of identifiers,
///   useful for sharing identifiers across threads or cloned simulations.
/// - **Hashable**: Implements the `Hash` trait, suitable for use in hash maps or sets.
/// - **Equality Comparisons**: Fully supports equality (`Eq` trait) and partial equality (`PartialEq` trait),
///   enabling comparisons between walk identifiers.
///
/// ## Usage Notes:
/// - Can be used as a key in hash-based collections like `HashMap` or `HashSet`.
/// - Encouraged to use descriptive naming conventions for easier readability and debugging.
///
/// ## Example Use Cases:
/// - Associating identifiers with random walks in financial modeling simulations.
/// - Tracking individual simulations within a larger stochastic framework.
/// - Debugging results by correlating output data to specific identifiers.
///
/// ## Related Components:
/// - Part of the **Random Walk Module** (`mod.rs`).
/// - Often used in conjunction with the `Simulator` and `RandomWalkGraph` components to
///   provide full traceability of stochastic simulations.
///
/// ## Implementation Details:
/// - Internally wraps a `String` for maximum flexibility in naming.
/// - Lightweight and efficient for frequent comparisons or use within hash-based collections.
///
/// ## Derives:
/// - `Debug`: Enables debugging output with developer-friendly formatting.
/// - `Clone`: Allows the entire identifier to be cloned easily.
/// - `Hash`: Makes the struct compatible with hash-based collections.
/// - `Eq` and `PartialEq`: Provides full equality comparison support.
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct WalkId(String);

/// Implementation of the `WalkId` struct, which serves as a unique identifier
/// for random walks within the simulator module. This identifier encapsulates
/// a `String` value and provides basic utility methods for creation and retrieval
/// of the identifier as a string.
impl WalkId {
    /// Constructs a new `WalkId` instance from a value that can be converted into a `String`.
    ///
    /// # Parameters
    ///
    /// - `id`: A value that implements the `Into<String>` trait, allowing for flexible input types,
    ///         such as `&str`, `String`, or other compatible types, to create a `WalkId`.
    ///
    /// # Returns
    ///
    /// A new `WalkId` containing the provided identifier string.
    ///
    /// # Examples
    ///
    /// - Creating a `WalkId` from a string literal:
    ///   ```
    ///   use optionstratlib::simulation::WalkId;
    ///   let walk_id = WalkId::new("random_walk_1");
    ///   ```
    ///
    /// - Creating a `WalkId` from an existing `String`:
    ///   ```
    ///   use optionstratlib::simulation::WalkId;
    ///   let walk_id = WalkId::new(String::from("unique_walk_id"));
    ///   ```
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Returns the string representation of the `WalkId`.
    ///
    /// # Returns
    ///
    /// A borrowed reference to the encapsulated `String` of the `WalkId`.
    ///
    /// This method provides a read-only view of the identifier, allowing
    /// consumers to access the string data without the need to clone it.
    ///
    /// # Examples
    ///
    /// - Accessing `WalkId` as a string slice:
    ///   ```
    ///   use optionstratlib::simulation::WalkId;
    ///   let walk_id = WalkId::new("example_walk");
    ///   assert_eq!(walk_id.as_str(), "example_walk");
    ///   ```
    ///
    /// - Using `WalkId` in string formatting:
    ///   ```
    ///   use optionstratlib::simulation::WalkId;
    ///   let walk_id = WalkId::new("formatted_walk");
    ///   println!("Walk Identifier: {}", walk_id.as_str());
    ///   ```
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Represents a simulator for managing and running multiple random walk simulations.
///
/// This struct is designed to:
/// - Store shared configuration used across all random walks.
/// - Maintain individual simulation graphs for different random walks identified by unique IDs.
/// - Optionally store correlation data between different random walks.
///
/// ## Fields
///
/// - `config`:
///     - Shared configuration of type [`SimulationConfig`] that includes parameters such as
///       risk-free rate, dividend yield, and time frame.
///     - This configuration is intended to be reused by all random walks managed by this simulator.
///
/// - `walks`:
///     - A mapping of [`WalkId`] (unique identifiers for individual random walks)
///       to their corresponding [`RandomWalkGraph`] instances.
///     - Each graph represents a self-contained simulation of a random walk with its own data and parameters.
///
/// - `correlations`:
///     - An optional field containing a hash map of tuples of [`WalkId`] pairs to their correlation coefficients (`f64`).
///     - This is useful for analyzing relationships between different random walks managed by the simulator,
///       such as simulating correlated paths for financial assets.
///
/// ## Use Case
///
/// The `Simulator` struct is designed for scenarios where multiple random walk simulations need to be managed
/// concurrently. Examples include financial asset pricing models, Monte Carlo simulations, or correlated stochastic processes.
///
/// ## Thread Safety
///
/// - Uses `Arc` to allow safe sharing of the configuration across threads.
/// - The `walks` and `correlations` fields assume no inherent thread-safety and should be externally synchronized
///   if the data is concurrently accessed or modified.
///
/// # Related Types
///
/// - [`WalkId`]: Represents an identifier for a specific random walk within the simulator.
/// - [`SimulationConfig`]: Provides shared configuration parameters for simulations.
/// - [`RandomWalkGraph`]: Encapsulates the data and functionality for a specific random walk simulation.
///
/// ## Example Workflow
///
/// 1. Configure the `SimulationConfig` for the simulation environment.
/// 2. Create instances of `RandomWalkGraph` for individual random walks.
/// 3. Use the `Simulator` to manage the graphs and optionally analyze correlations between them.
pub struct Simulator {
    /// Shared configuration across walks
    config: Arc<SimulationConfig>,
    /// Map of walk IDs to their corresponding graphs
    walks: HashMap<WalkId, RandomWalkGraph>,
    /// Optional correlation matrix between walks
    correlations: Option<HashMap<(WalkId, WalkId), f64>>,
}

impl Simulator {
    /// Creates a new simulator with the given configuration.
    ///
    /// # Parameters
    /// - `config`: The configuration to initialize the simulator with.
    ///
    /// # Returns
    /// - A new instance of the `Simulator`.
    pub fn new(config: SimulationConfig) -> Self {
        Self {
            config: Arc::new(config),
            walks: HashMap::new(),
            correlations: None,
        }
    }

    /// Adds a new random walk to the simulator.
    ///
    /// # Parameters
    /// - `id`: The unique identifier for the random walk.
    /// - `title`: A human-readable title for the random walk.
    ///
    /// # Returns
    /// - A mutable reference to the newly created `RandomWalkGraph`.
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

    /// Removes a random walk from the simulator.
    ///
    /// # Parameters
    /// - `id`: The unique identifier of the random walk to remove.
    ///
    /// # Returns
    /// - `Some(RandomWalkGraph)` if the walk was successfully removed.
    /// - `None` if no walk with the given ID existed.
    pub fn remove_walk(&mut self, id: &WalkId) -> Option<RandomWalkGraph> {
        self.walks.remove(id)
    }

    /// Gets a reference to a random walk by its ID.
    ///
    /// # Parameters
    /// - `id`: The unique identifier of the desired random walk.
    ///
    /// # Returns
    /// - `Some(&RandomWalkGraph)` if the walk exists.
    /// - `None` if the walk does not exist.
    pub fn get_walk(&self, id: &WalkId) -> Option<&RandomWalkGraph> {
        self.walks.get(id)
    }

    /// Gets a mutable reference to a random walk by its ID.
    ///
    /// # Parameters
    /// - `id`: The unique identifier of the desired random walk.
    ///
    /// # Returns
    /// - `Some(&mut RandomWalkGraph)` if the walk exists.
    /// - `None` if the walk does not exist.
    pub fn get_walk_mut(&mut self, id: &WalkId) -> Option<&mut RandomWalkGraph> {
        self.walks.get_mut(id)
    }

    /// Sets the correlation between two random walks.
    ///
    /// # Parameters
    /// - `_id1`: The unique identifier of the first random walk.
    /// - `_id2`: The unique identifier of the second random walk.
    ///
    /// # Remarks
    /// The implementation is currently incomplete and marked as `todo!`.
    pub fn set_correlation(&mut self, _id1: &WalkId, _id2: &WalkId) {
        if self.correlations.is_none() {
            self.correlations = Some(HashMap::new());
        }
        todo!("Set correlation between two walks");
    }

    /// Generates random walks for each walk in the simulator.
    ///
    /// # Parameters
    /// - `n_steps`: The number of steps in the random walk.
    /// - `initial`: A map of initial for each random walk.
    /// - `mean`: The mean of the random walk.
    /// - `std_dev`: The standard deviation of the random walk.
    /// - `std_dev_change`: The standard deviation of volatility changes.
    ///
    /// # Returns
    /// - `Ok(())` if all random walks were successfully generated.
    /// - `Err(Box<dyn Error>)` if an error occurs during generation.
    pub fn generate_random_walks(
        &mut self,
        n_steps: usize,
        initial: &HashMap<WalkId, Positive>,
        mean: f64,
        std_dev: Positive,
        std_dev_change: Positive,
    ) -> Result<(), Box<dyn Error>> {
        let results: Result<Vec<_>, _> = self
            .walks
            .iter_mut()
            .map(|(id, walk)| {
                let first = initial
                    .get(id)
                    .ok_or_else(|| format!("No initial provided for walk {}", id.as_str()))?;
                walk.generate_random_walk(n_steps, *first, mean, std_dev, std_dev_change)
            })
            .collect();
        results?;
        Ok(())
    }

    /// Retrieves all random walk IDs in the simulator.
    ///
    /// # Returns
    /// - A vector containing all walk IDs registered in the simulator.
    pub fn get_walk_ids(&self) -> Vec<WalkId> {
        self.walks.keys().cloned().collect()
    }

    /// Gets the current simulator configuration.
    ///
    /// # Returns
    /// - A reference to the current `SimulationConfig`.
    pub fn get_config(&self) -> &SimulationConfig {
        &self.config
    }

    /// Updates the simulator configuration.
    ///
    /// # Parameters
    /// - `new_config`: The new configuration to apply to the simulator.
    ///
    /// # Remarks
    /// This will replace the existing configuration.
    pub fn update_config(&mut self, new_config: SimulationConfig) {
        self.config = Arc::new(new_config);
    }

    /// Plots all random walks in the simulator on a single graph.
    ///
    /// # Parameters
    /// - `backend`: Specifies the backend to be used for rendering the graph.
    /// - `title_size`: The size of the font for the graph title.
    ///
    /// # Returns
    /// - `Ok(())` if the graph was successfully generated and rendered.
    /// - `Err(Box<dyn Error>)` if an error occurs during the graphing process.
    pub fn graph(&self, backend: GraphBackend, title_size: u32) -> Result<(), Box<dyn Error>> {
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

        let mut all_points = Vec::new();
        for walk in self.walks.values() {
            let walk_points = walk.get_points();
            if !walk_points.is_empty() {
                all_points.extend(walk_points);
            }
        }

        if all_points.is_empty() {
            return Err("No data points to plot".into());
        }

        let x_values: Vec<f64> = all_points.iter().map(|p| p.coordinates.0).collect();
        let y_values: Vec<f64> = all_points
            .iter()
            .map(|p| p.coordinates.1.to_f64())
            .collect();

        let x_min = x_values.iter().cloned().fold(f64::INFINITY, f64::min);
        let x_max = x_values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let y_min = y_values.iter().cloned().fold(f64::INFINITY, f64::min);
        let y_max = y_values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

        let x_buffer = (x_max - x_min) * 0.05;
        let y_buffer = (y_max - y_min) * 0.05;

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

        chart
            .configure_mesh()
            .x_desc("Time")
            .y_desc("Value")
            .draw()?;

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

        chart
            .configure_series_labels()
            .background_style(WHITE.mix(0.8))
            .border_style(BLACK)
            .position(SeriesLabelPosition::UpperRight)
            .draw()?;

        root.present()?;
        Ok(())
    }

    /// Simulates the performance of a strategy across all random walks.
    ///
    /// This method runs a Monte Carlo simulation using the random walks generated in the simulator
    /// and evaluates the performance of the provided strategy.
    ///
    /// # Parameters
    /// - `strategy`: An immutable reference to a strategy implementing the Strategable trait
    ///
    /// # Returns
    /// - `Ok(SimulationResult)`: A structured result containing risk metrics and performance statistics
    /// - `Err(Box<dyn Error>)`: If the simulation fails due to insufficient data or calculation errors
    ///
    /// # Type Parameters
    /// - `S`: The concrete strategy type that implements Strategable
    /// - `T`: The associated Strategy type from the Strategable trait
    pub fn simulate_strategy<S, T>(&self, strategy: S) -> Result<SimulationResult, Box<dyn Error>>
    where
        S: Strategable<Strategy = T>,
    {
        // Verify that walks exist for the simulation
        if self.walks.is_empty() {
            return Err("No walks available for strategy simulation".into());
        }

        // Get simulation configuration
        let iterations = self.walks.len();
        let risk_free_rate = self.config.risk_free_rate.unwrap_or(Decimal::ZERO);

        // Arrays to store simulation results
        let mut profits = Vec::with_capacity(iterations);
        let mut max_drawdowns = Vec::with_capacity(iterations);

        // For each walk in the simulation (treating each walk as one iteration)
        for (_, walk) in &self.walks {
            // Get the points from this walk
            let points = walk.get_points();
            if points.is_empty() {
                return Err("Walk contains no points for simulation".into());
            }

            // Extract the final price from the walk
            let final_price = points.last().unwrap().coordinates.1;
            let final_price_positive = final_price;

            // Calculate the PnL at expiration for this price path
            let pnl = strategy.calculate_profit_at(final_price_positive)?;
            profits.push(pnl);

            // Calculate maximum drawdown for this path
            let mut max_price = Positive::ZERO;
            let mut max_drawdown = Positive::ZERO;

            for point in points {
                let price = point.coordinates.1;

                // Skip if price is zero or negative (shouldn't happen with stock prices)
                if price <= Decimal::ZERO {
                    continue;
                }

                // Update maximum price seen so far
                if price > max_price {
                    max_price = price;
                }
                // Calculate drawdown if current price is below maximum
                else if max_price > Decimal::ZERO {
                    let drawdown = (max_price - price) / max_price;
                    if drawdown > max_drawdown {
                        max_drawdown = drawdown;
                    }
                }
            }

            max_drawdowns.push(max_drawdown);
        }

        // Sort results for statistical calculations
        profits.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        // Calculate basic statistics
        let profit_count = profits.iter().filter(|&&p| p > Decimal::ZERO).count();
        let loss_count = iterations - profit_count;

        let profit_probability = Decimal::from_f64((profit_count as f64 / iterations as f64) * 100.0)
            .unwrap_or(Decimal::ZERO);
        let loss_probability = Decimal::from_f64(100.0).unwrap_or(Decimal::ZERO) - profit_probability;

        let max_profit = *profits.last().unwrap_or(&Decimal::ZERO);
        let max_loss = profits.first().map(|&p| p.abs()).unwrap_or(Decimal::ZERO);

        // Calculate average PnL
        let sum: Decimal = profits.iter().sum();
        let average_pnl = if iterations > 0 { sum / Decimal::from(iterations) } else { Decimal::ZERO };

        // Calculate standard deviation
        let variance_sum: Decimal = profits
            .iter()
            .map(|&p| (p - average_pnl).powu(2))
            .sum();

        let pnl_std_dev = Decimal::from_f64(
            (variance_sum / Decimal::from(iterations))
                .to_f64()
                .unwrap_or(0.0)
                .sqrt(),
        )
            .unwrap_or(Decimal::ZERO);

        // Calculate Value at Risk (VaR) and Conditional VaR (CVaR)
        let var_95_index = (iterations as f64 * 0.05) as usize;
        let var_99_index = (iterations as f64 * 0.01) as usize;

        // Handle edge cases for small number of iterations
        let var_95 = if var_95_index < profits.len() {
            -profits[var_95_index]
        } else {
            Decimal::ZERO
        };

        let var_99 = if var_99_index < profits.len() {
            -profits[var_99_index]
        } else {
            Decimal::ZERO
        };

        // Calculate CVaR (Expected Shortfall)
        let cvar_95 = if var_95_index > 0 {
            let cvar_95_sum: Decimal = profits.iter().take(var_95_index).sum();
            -cvar_95_sum / Decimal::from(var_95_index)
        } else {
            Decimal::ZERO
        };

        // Calculate probability of severe loss (>50% of investment)
        // Assuming we can get this from the strategy
        let severe_loss_threshold = Decimal::from_f64(0.5).unwrap_or(Decimal::ZERO);

        let severe_loss_count = profits
            .iter()
            .filter(|&&p| p < -severe_loss_threshold)
            .count();

        let severe_loss_probability = Decimal::from_f64(
            (severe_loss_count as f64 / iterations as f64) * 100.0
        ).unwrap_or(Decimal::ZERO);

        // Calculate Sharpe Ratio
        let sharpe_ratio = if pnl_std_dev > Decimal::ZERO {
            (average_pnl - risk_free_rate) / pnl_std_dev
        } else {
            Decimal::ZERO
        };

        // Create PnL distribution for histogram
        let mut pnl_distribution = HashMap::new();

        // Convert to i32 for histogram ranges
        let min_pnl = (profits.first().copied().unwrap_or(Decimal::ZERO).to_f64().unwrap_or(0.0) * 100.0) as i32;
        let max_pnl = (profits.last().copied().unwrap_or(Decimal::ZERO).to_f64().unwrap_or(0.0) * 100.0) as i32;

        // Create buckets for histogram
        let num_buckets = 20;
        let bucket_size = if max_pnl > min_pnl {
            ((max_pnl - min_pnl) / num_buckets).max(1)
        } else {
            1
        };

        for i in 0..num_buckets {
            let lower = min_pnl + i * bucket_size;
            let upper = lower + bucket_size;
            let range = PnLRange { lower, upper };

            let count = profits
                .iter()
                .filter(|&&p| {
                    let p_int = (p.to_f64().unwrap_or(0.0) * 100.0) as i32;
                    p_int >= lower && p_int < upper
                })
                .count();

            let probability = Decimal::from_f64(count as f64 / iterations as f64)
                .unwrap_or(Decimal::ZERO);

            pnl_distribution.insert(range, probability);
        }

        // Convert values to specific types
        let profit_probability_positive = Positive::from(profit_probability);

        let loss_probability_positive = Positive::from(loss_probability);

        let max_profit_positive = Positive::from(max_profit);

        let max_loss_positive = Positive::from(max_loss);

        let pnl_std_dev_positive = Positive::from(pnl_std_dev);

        let severe_loss_probability_positive = Positive::from(severe_loss_probability);

        let max_drawdown_positive = Positive::new(
                max_drawdowns.iter()
                    .fold(0.0, |max, &d| f64::max(max, d.to_f64())) * 100.0
        )?;

        // Build and return the simulation result
        let result = SimulationResult {
            iterations: iterations as u32,
            profit_probability: profit_probability_positive,
            loss_probability: loss_probability_positive,
            max_profit: max_profit_positive,
            max_loss: max_loss_positive,
            average_pnl: average_pnl,
            pnl_std_dev: pnl_std_dev_positive,
            risk_levels: RiskMetricsSimulation {
                var_95,
                var_99,
                cvar_95,
                severe_loss_probability: severe_loss_probability_positive,
                max_drawdown: max_drawdown_positive,
                sharpe_ratio,
            },
            pnl_distribution,
            additional_metrics: HashMap::new(),
        };

        Ok(result)
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
