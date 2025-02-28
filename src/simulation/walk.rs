/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 22/10/24
******************************************************************************/
#[cfg(test)]
mod test_walk;

use crate::chains::utils::OptionDataPriceParams;
use crate::constants::ZERO;
use crate::curves::{Curvable, Curve, Point2D};
use crate::error::CurveError;
use crate::geometrics::GeometricObject;
use crate::model::types::ExpirationDate;
use crate::pricing::payoff::Profit;
use crate::simulation::model::WalkResult;
use crate::strategies::Strategable;
use crate::utils::time::{convert_time_frame, units_per_year, TimeFrame};
use crate::visualization::model::ChartPoint;
use crate::visualization::utils::Graph;
use crate::{pos, Positive};
use num_traits::FromPrimitive;
use rand::distributions::Distribution;
use rand::thread_rng;
use rust_decimal::Decimal;
use statrs::distribution::Normal;
use std::collections::HashMap;
use std::error::Error;
use tracing::{info, trace};

/// The `Walkable` trait defines a generic structure for creating and manipulating
/// entities capable of simulating or managing a random walk sequence of values.
/// Implementations of this trait must handle a vector of `Positive` values, which
/// serve as the primary storage for the y-axis values used in simulations or computations.
pub trait Walkable {
    /// Provides read-only access to the vector of `Positive` values representing
    /// the y-axis data points of a structure implementing this method.
    ///
    /// # Returns
    /// A reference to a `Vec<Positive>` containing the stored y-axis values.
    ///
    /// # Purpose
    /// This method is typically used to retrieve the internal `Positive` values
    /// for further processing, analysis, or visualization while maintaining
    /// the immutability of the vector.
    fn get_y_values(&self) -> &Vec<Positive>;

    /// Computes and retrieves the x-axis values corresponding to the y-axis values
    /// stored within the structure as a `Vec<Positive>`.
    ///
    /// # Implementation
    /// The x-axis values are derived sequentially starting from `0` and are converted
    /// into `Positive` values. The number of x-axis values aligns with the length
    /// of the stored y-axis vector.
    ///
    /// # Returns
    /// A `Vec<Positive>` containing the x-axis values.
    ///
    /// # Remarks
    /// - This function assumes that `Positive` supports conversion from `f64`.
    /// - The computed x-values are purely derived from the y-values' indices
    ///   and are not stored internally.
    fn get_x_values(&self) -> Vec<Positive> {
        (0..self.get_y_values().len())
            .map(|i| Positive::from(i as f64))
            .collect()
    }

    /// Provides mutable access to the vector of `Positive` values representing
    /// the y-axis data points of a structure implementing this method.
    ///
    /// # Returns
    /// A mutable reference to a `Vec<Positive>` containing the stored y-axis values.
    ///
    /// # Purpose
    /// This method is intended for situations where modifications to the y-axis
    /// data are required, such as appending, updating, or resizing the vector.
    /// It ensures controlled and safe mutable access.
    fn get_y_values_ref(&mut self) -> &mut Vec<Positive>;

    /// Generates a random walk sequence of values using a normal distribution.
    ///
    /// # Arguments
    /// * `n_steps` - The total number of steps to generate in the random walk.
    /// * `initial_price` - The starting value of the sequence, represented as a `Positive`.
    /// * `mean` - The mean value for the normal distribution of price changes.
    /// * `std_dev` - The initial standard deviation (volatility) for the normal distribution.
    /// * `std_dev_change` - The daily change in volatility (volatility of volatility or VoV).
    ///
    /// # Errors
    /// Returns an error if:
    /// * `n_steps` is zero, as a random walk requires at least one step.
    ///
    /// # Behavior
    /// The function:
    /// 1. Ensures `n_steps` is greater than zero.
    /// 2. Initializes the random number generator and prepares the vector of output values.
    /// 3. Adjusts volatility dynamically based on `std_dev_change`.
    /// 4. Calculates steps using a normal distribution with the given `mean` and `std_dev`.
    /// 5. Converts all computed values into `Positive` and updates the underlying vector.
    fn generate_random_walk(
        &mut self,
        n_steps: usize,
        initial_price: Positive,
        drift: f64,
        volatility: Positive,
        volatility_change: Positive,
    ) -> Result<(), Box<dyn Error>> {
        if n_steps == 0 {
            return Err(Box::from("Number of steps must be greater than zero"));
        }

        let mut rng = thread_rng();
        let mut current_volatility = volatility;

        let dt = (1.0 / 252.0) as f64; // Correct - daily time step for annual parameters
        let sqrt_dt = dt.sqrt();

        let values = self.get_y_values_ref();
        values.clear();
        values.reserve(n_steps);
        values.push(initial_price);

        let mut current_price = initial_price;
        let mut volatilities = Vec::with_capacity(n_steps);
        volatilities.push(current_volatility); // Add initial volatility

        for _ in 0..n_steps - 1 {
            // Potentially update volatility (stochastic volatility model)
            if volatility_change > Positive::ZERO {
                current_volatility = Normal::new(volatility.into(), volatility_change.into())
                    .unwrap()
                    .sample(&mut rng)
                    .max(ZERO)
                    .into();
            }

            volatilities.push(current_volatility);

            // Generate a standard normal random variable
            let z = Normal::new(0.0, 1.0).unwrap().sample(&mut rng);

            // Calculate price movement using the log-normal model
            // S(t+dt) = S(t) * exp((mu - 0.5*sigma^2)*dt + sigma*sqrt(dt)*z)
            let vol = current_volatility.to_f64();
            let drift_term = (drift - 0.5 * vol * vol) * dt;
            let volatility_term = vol * sqrt_dt * z;

            let log_return = drift_term + volatility_term;
            let next_price = current_price.to_f64() * f64::exp(log_return);

            // Ensure price doesn't go below zero
            current_price = pos!(next_price.max(ZERO));
            values.push(current_price);

            trace!(
                "Current price: {}, Volatility: {}",
                current_price,
                current_volatility
            );
        }
        
        for vol in volatilities {
            if !vol.is_zero() {
                self.save_volatility(vol)?;
            } else {
                self.save_volatility(volatility)?;
            }
        }

        Ok(())
    }

    /// Generates a random walk sequence of values over a specified timeframe,
    /// adjusting for volatility changes for the given periods.
    ///
    /// # Arguments
    /// * `n_steps` - The total number of steps in the random walk.
    /// * `initial_price` - The initial value of the sequence as a `Positive`.
    /// * `mean` - The mean of the price change distribution.
    /// * `std_dev` - Daily volatility (standard deviation) represented as `Positive`.
    /// * `std_dev_change` - Daily change in volatility (VoV) represented as `Positive`.
    /// * `time_frame` - The timeframe over which the simulation takes place.
    /// * `volatility_limits` - An optional tuple representing the minimum and maximum volatility limits.
    ///
    /// # Errors
    /// Returns an error if:
    /// * `n_steps` is zero, as a random walk requires at least one step.
    ///
    /// # Behavior
    /// 1. Converts daily volatility (`std_dev`) and its change (`std_dev_change`) to the target timeframe.
    /// 2. Dynamically adjusts volatility based on the provided volatility change (`std_dev_change`).
    /// 3. Constrains volatility within the specified limits, if provided.
    /// 4. Computes step values using a normal distribution adjusted for the timeframe's volatility.
    /// 5. Updates the internal storage of `Positive` values with the generated sequence.
    #[allow(clippy::too_many_arguments)]
    fn generate_random_walk_timeframe(
        &mut self,
        n_steps: usize,
        initial_price: Positive,
        drift: f64,
        volatility: Positive,        // daily volatility
        volatility_change: Positive, // daily VoV
        time_frame: TimeFrame,
        volatility_limits: Option<(Positive, Positive)>,
    ) -> Result<(), Box<dyn Error>> {
        if n_steps == 0 {
            return Err(Box::from("Number of steps must be greater than zero"));
        }

        let mut rng = thread_rng();
        let mut current_volatility = volatility;

        // Calculate dt based on the time_frame
        let periods_per_year = units_per_year(&time_frame);
        let dt = 1.0 / periods_per_year.to_f64();
        let sqrt_dt = dt.sqrt();

        let values = self.get_y_values_ref();
        values.clear();
        values.reserve(n_steps);
        values.push(initial_price);

        let mut current_price = initial_price;
        let mut volatilities = Vec::with_capacity(n_steps);
        volatilities.push(current_volatility); // Add initial volatility

        for _ in 0..n_steps - 1 {
            // Potentially update volatility (stochastic volatility model)
            if volatility_change > Positive::ZERO {
                // Use a smaller step for volatility evolution
                // This makes volatility more stable than using completely random changes
                let vol_drift = 0.0; // Mean-reverting to initial volatility
                let vol_term = volatility_change.to_f64()
                    * sqrt_dt
                    * Normal::new(0.0, 1.0).unwrap().sample(&mut rng);
                let new_vol = current_volatility.to_f64() * f64::exp(vol_drift * dt + vol_term);

                // Apply volatility limits if provided
                if let Some((min_vol, max_vol)) = volatility_limits {
                    current_volatility = pos!(new_vol.min(max_vol.to_f64()).max(min_vol.to_f64()));
                } else {
                    // Default limits if none provided to avoid extreme values
                    let default_min = volatility.to_f64() * 0.5;
                    let default_max = volatility.to_f64() * 2.0;
                    current_volatility = pos!(new_vol.min(default_max).max(default_min));
                }
            }

            volatilities.push(current_volatility);

            // Generate a standard normal random variable
            let z = Normal::new(0.0, 1.0).unwrap().sample(&mut rng);

            // Calculate price movement using the log-normal model
            let vol = current_volatility.to_f64();
            let drift_term = (drift - 0.5 * vol * vol) * dt;
            let volatility_term = vol * sqrt_dt * z;

            let log_return = drift_term + volatility_term;
            let next_price = current_price.to_f64() * f64::exp(log_return);

            // Ensure price doesn't go below zero
            current_price = pos!(next_price.max(ZERO));
            values.push(current_price);

            trace!(
                "Current price: {}, Volatility: {}",
                current_price,
                current_volatility
            );
        }

        for vol in volatilities {
            if !vol.is_zero() {
                self.save_volatility(vol)?;
            } else {
                self.save_volatility(volatility)?;
            }
        }

        Ok(())
    }

    fn save_volatility(&mut self, _volatility: Positive) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    fn get_volatilities(&self) -> Result<Vec<Positive>, Box<dyn Error>> {
        unimplemented!()
    }

    fn get_randon_walk(&self) -> Result<RandomWalkGraph, Box<dyn Error>>;

    fn walk_strategy<S>(
        &self,
        strategy: &mut S,
        time_frame: TimeFrame,
    ) -> Result<WalkResult, Box<dyn Error>>
    where
        S: Strategable,
    {
        // Verify that walks exist for the simulation
        if self.get_x_values().is_empty() {
            return Err("No walks available for strategy simulation".into());
        }

        let values = self.get_y_values();
        let values_len: Positive = Decimal::from_usize(values.len()).unwrap().into();
        let days = convert_time_frame(values_len, &time_frame, &TimeFrame::Day);
        strategy.set_expiration_date(ExpirationDate::Days(days))?;

        let initially = values.first().unwrap().to_dec();
        let finally = values.last().unwrap().to_dec();
        let volatilities = self.get_volatilities()?;

        // Validate volatility data
        if volatilities.len() != values.len() {
            return Err(format!(
                "Volatility data length ({}) doesn't match price data length ({})",
                volatilities.len(),
                values.len()
            )
            .into());
        }

        let mut walk_result = WalkResult {
            initially,
            finally,
            payoff: strategy.calculate_profit_at(Positive(finally))?,
            change_percentage: (finally - initially) / initially * Decimal::ONE_HUNDRED,
            diff: finally - initially,
            max_value: (Decimal::ZERO, Decimal::MIN), // Initialize with minimum possible value
            min_value: (Decimal::ZERO, Decimal::MAX), // Initialize with maximum possible value
            positive_points: Vec::new(),
            negative_points: Vec::new(),
            pnl_at_prices: HashMap::new(),
            extra_metrics: HashMap::new(),
            volatilities: volatilities.clone(),
        };

        // Debug output to check volatilities
        info!(
            "Using {} volatility values for {} price points",
            volatilities.len(),
            values.len()
        );

        // Reverse i for expiration date descending
        for (i, (price, volatility)) in values.iter().zip(volatilities.iter()).enumerate().rev() {
            let price_dec = price.to_dec();
            let days_left = convert_time_frame(pos!(i as f64), &time_frame, &TimeFrame::Day);

            // Debug log to track calculations
            info!("Step {}: Params: Underlying Price: {}, Expiration: {} Years, Implied Volatility: {}", 
               i, price, days_left.to_f64() / 365.0, volatility);

            let pnl = if !days_left.is_zero() {
                 strategy.calculate_pnl(price, ExpirationDate::Days(days_left), volatility)?
            } else {
                strategy.calculate_pnl_at_expiration(price)?
            };
            
            // Ensure unwrap is safe
            let pnl_unrealized = match pnl.unrealized {
                Some(value) => value,
                None => {
                    match pnl.realized {
                        Some(value) => value,
                        None => return Err(
                            format!("No unrealized PnL calculated for price: {}", price_dec).into(),
                        )
                    }
                }
            };

            walk_result.pnl_at_prices.insert(price_dec, pnl_unrealized);

            // Separate into positive and negative points
            if pnl_unrealized >= Decimal::ZERO {
                walk_result
                    .positive_points
                    .push((price_dec, pnl_unrealized));
            } else {
                walk_result
                    .negative_points
                    .push((price_dec, pnl_unrealized));
            }

            // Update max and min values
            if pnl_unrealized > walk_result.max_value.1 {
                walk_result.max_value = (price_dec, pnl_unrealized);
            }
            if pnl_unrealized < walk_result.min_value.1 {
                walk_result.min_value = (price_dec, pnl_unrealized);
            }
        }

        Ok(walk_result)
    }
}

/// A structure implementing a specific type of `Walkable` trait called `RandomWalkGraph`.
/// This struct is primarily used for simulating and representing a random walk graph,
/// which is often used to model and analyze financial asset price movements. It includes
/// various optional parameters, such as risk-free rate and dividend yield, to allow for
/// advanced financial modeling.
///
/// ## Fields
///
/// - **values** (`Vec<Positive>`):  
///   A vector that represents the sequence of y-axis values in the random walk.
///   These values are strictly positive and correspond to asset prices or other
///   similar measures over time.
///
/// - **title_text** (`String`):  
///   A string representing the title of the graph. This title is typically
///   intended for user-facing visualization or labeling purposes.
///
/// - **current_index** (`usize`):  
///   Tracks the current position of traversal within the random walk.
///   This is particularly useful when using `RandomWalkGraph` as an iterator,
///   processing the random walk step-by-step.
///
/// - **risk_free_rate** (`Option<Decimal>`):  
///   An optional field that represents the risk-free rate of return used in
///   financial modeling, often expressed as a decimal. If defined, this parameter
///   can influence the random walk's stochastic drift.
///
/// - **dividend_yield** (`Option<Positive>`):  
///   An optional percentage value representing the dividend yield of the underlying
///   asset. If set, it directly affects the behavior of the financial model, e.g.,
///   altering the drift component of the walk.
///
/// - **time_frame** (`TimeFrame`):  
///   Specifies the timeframe granularity of the random walk. Can be daily, weekly,
///   monthly, or a custom period expressed as a `Positive`. This affects the
///   overall scaling of parameters like volatility and drift.
///
/// - **volatility_window** (`usize`):  
///   Defines the size of the rolling window used for volatility calculations during
///   the random walk generation process. Larger windows provide smoother estimations,
///   while smaller windows react more quickly to variability.
///
/// - **initial_volatility** (`Option<Positive>`):  
///   The initial value of volatility used when starting the random walk. This parameter
///   is optional and, if absent, may default to other internally derived volatility measures.
///
/// ## Purpose
///
/// The `RandomWalkGraph` struct is designed to combine financial modeling techniques,
/// such as volatility tracking and dividend adjustments, with tools for simulation
/// and visualization. It provides a basis for implementing geometric Brownian motion
/// in asset price modeling, with support for stochastic volatility updates.
///
/// ## Characteristics
///
/// - Effective for simulating and storing random walk data.
/// - Enforces all computed values as strictly positive using `Positive`.
/// - Highly customizable with parameters like risk-free rate, dividend yield,
///   timeframe, and rolling volatility window.
/// - Can be used in combination with the `Walkable` trait for generating price paths
///   programmatically and iterating through the results sequentially.
pub struct RandomWalkGraph {
    /// Values representing the y-axis data of the random walk.
    pub(crate) values: Vec<Positive>,
    /// Text for the graph's title.
    title_text: String,
    /// Tracks the current index for traversing the graph.
    current_index: usize,
    /// Optional risk-free rate used in calculations.
    risk_free_rate: Option<Decimal>,
    /// Optional dividend yield percentage.
    dividend_yield: Option<Positive>,
    /// Specifies the timeframe (e.g., daily, weekly) for the graph calculations.
    time_frame: TimeFrame,
    /// Determines the window size used in volatility calculations.
    volatility_window: usize,
    /// Optional initial volatility of the random walk.
    initial_volatility: Option<Positive>,

    pub volatilities: Vec<Positive>,
}

impl RandomWalkGraph {
    /// Creates a new `RandomWalkGraph` instance.
    ///
    /// Initializes a random walk graph with desired parameters including title, optional
    /// financial parameters such as risk-free rate and dividend yield, timeframe, and other
    /// properties for volatility calculations.
    ///
    /// # Arguments
    ///
    /// * `title` - A string representing the title of the graph.
    /// * `risk_free_rate` - An optional risk-free rate represented as a `Decimal`, used in financial calculations.
    /// * `dividend_yield` - An optional dividend yield value of type `Positive`.
    /// * `time_frame` - The `TimeFrame` enum specifying the timeframe for the graph data (e.g., daily, weekly).
    /// * `volatility_window` - An `usize` value determining the number of past data points used for calculating historical volatility.
    /// * `initial_volatility` - An optional `Positive` value denoting the initial volatility for the random walk; used when insufficient data exists for calculation.
    ///
    /// # Returns
    ///
    /// Returns a new `RandomWalkGraph` instance with the given properties initialized.
    pub fn new(
        title: String,
        risk_free_rate: Option<Decimal>,
        dividend_yield: Option<Positive>,
        time_frame: TimeFrame,
        volatility_window: usize,
        initial_volatility: Option<Positive>,
    ) -> Self {
        Self {
            values: Vec::new(),
            title_text: title,
            current_index: 0,
            risk_free_rate,
            dividend_yield,
            time_frame,
            volatility_window,
            initial_volatility,
            volatilities: Vec::new(),
        }
    }

    /// Calculates the current volatility of the random walk data.
    ///
    /// This method computes the volatility of the random walk based on historical returns
    /// over a rolling window. It defaults to the `initial_volatility` if insufficient data exists
    /// or if the computed volatility is invalid.
    ///
    /// # Returns
    ///
    /// * `Some(Positive)` - The computed volatility when it is valid and non-negative.
    /// * `None` - If the volatility could not be calculated or results in an invalid value.
    fn calculate_current_volatility(&self) -> Option<Positive> {
        let window_size = self.volatility_window.min(self.values.len());

        // Always use initial volatility as our base/target level
        let target_volatility = self.initial_volatility?;

        // If we don't have enough data points yet, just return the initial volatility
        if self.current_index < window_size + 1 {
            return Some(target_volatility);
        }

        // Get returns from the appropriate window
        let start_idx = self.current_index.saturating_sub(window_size);
        let end_idx = self.current_index;

        // Calculate log returns instead of simple returns
        let log_returns: Vec<f64> = self.values[start_idx..end_idx]
            .windows(2)
            .map(|w| (w[1].to_f64() / w[0].to_f64()).ln())
            .collect();

        if log_returns.is_empty() {
            return Some(target_volatility);
        }

        // Calculate the standard deviation of log returns
        let mean = log_returns.iter().sum::<f64>() / log_returns.len() as f64;

        // Use n-1 for sample standard deviation
        let divisor = if log_returns.len() > 1 {
            log_returns.len() - 1
        } else {
            1
        } as f64;
        let variance = log_returns.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / divisor;

        // Convert to annualized volatility (no multiplication by 100!)
        // Get the number of periods per year for annualization
        let periods_per_year = units_per_year(&self.time_frame).to_f64();

        let realized_volatility = variance.sqrt() * periods_per_year.sqrt();

        // Blend with target volatility for stability (optional)
        let blend_factor = 0.7; // 70% weight to realized vol, 30% to target
        let blended_volatility =
            blend_factor * realized_volatility + (1.0 - blend_factor) * target_volatility.to_f64();

        // Apply reasonable limits to prevent extreme changes
        let min_vol = target_volatility.to_f64() * 0.5;
        let max_vol = target_volatility.to_f64() * 1.5;
        let limited_volatility = blended_volatility.max(min_vol).min(max_vol);

        Some(pos!(limited_volatility))
    }

    /// Resets the current iterator index to its initial state.
    ///
    /// This method is used to restart graph iteration from the beginning by setting
    /// the `current_index` to `0`.
    pub fn reset_iterator(&mut self) {
        self.current_index = 0;
    }

    /// Retrieves the remaining time in the random walk in steps.
    ///
    /// Calculates the number of steps remaining for iteration in the graph.
    ///
    /// # Returns
    ///
    /// Returns a `Positive` value representing the remaining steps available for the iteration.
    fn get_remaining_time(&self) -> Positive {
        pos!((self.values.len() - self.current_index) as f64)
    }

    /// Generates a list of `ChartPoint` objects representing data points of the random walk.
    ///
    /// This method transforms the stored values in the graph into `ChartPoint` objects
    /// with labeled coordinates for chart visualization. Each step is labeled with its index.
    ///
    /// # Returns
    ///
    /// A vector of `ChartPoint<(f64, Positive)>` where the first component of the tuple represents
    /// the x-coordinate (e.g., step index), and the second component represents the y-coordinate
    /// (e.g., value).
    pub fn get_points(&self) -> Vec<ChartPoint<(f64, Positive)>> {
        self.values
            .iter()
            .enumerate()
            .map(|(index, &value)| {
                ChartPoint::new((index as f64, value), format!("Step {}", index))
            })
            .collect()
    }

    /// Creates an iterator for traversing the `RandomWalkGraph`.
    ///
    /// This method returns a `RandomWalkIterator` initialized at the starting index.
    ///
    /// # Returns
    ///
    /// A `RandomWalkIterator` instance for iterating over the graph's data points.
    pub fn iter(&self) -> RandomWalkIterator {
        RandomWalkIterator {
            walk: self,
            current_index: 0,
        }
    }
}

impl Default for RandomWalkGraph {
    fn default() -> Self {
        Self {
            values: Vec::new(),
            title_text: "Random Walk".to_string(),
            current_index: 0,
            risk_free_rate: None,
            dividend_yield: None,
            time_frame: TimeFrame::Day,
            volatility_window: 4,
            initial_volatility: None,
            volatilities: Vec::new(),
        }
    }
}

/// Implements the `Walkable` trait for the `RandomWalkGraph` structure, providing
/// functionality for managing its y-values and performing random walk simulations.
///
/// # Trait Implementation
///
/// This implementation allows the `RandomWalkGraph` to access and modify its stored
/// y-values, as well as create new instances of random walk graphs. The operations
/// are encapsulated in the following methods:
///
/// ## Methods
///
/// ### `get_y_values`
///
/// Retrieves an immutable reference to the vector of `Positive` y-values stored in the graph.
///
/// - **Panic**: This method will panic if the `values` vector is empty, ensuring that
///   there are always data points available for operations.
///
/// ### `get_y_values_ref`
///
/// Retrieves a mutable reference to the vector of `Positive` y-values stored in the graph.
/// This allows modification of the graph's data points.
///
/// ### `get_randon_walk`
///
/// Creates a new `RandomWalkGraph` instance with the same data as the current graph.
/// This method effectively clones the current graph, ensuring no modification to the
/// original instance.
///
/// - **Returns**: A `Result` wrapping the new `RandomWalkGraph` instance if successful,
///   or an error (`Box<dyn Error>`) otherwise.
///
/// # Notes
///
/// - The trait ensures the `values` field is non-empty during operations, maintaining
///   the structural integrity of the graph.
/// - The `RandomWalkGraph` structure stores additional metadata (e.g., title text,
///   current index, risk-free rate) relevant for financial simulations or graph visualizations.
///
/// # Implementation Details
///
/// - The `Positive` struct is used for y-values, ensuring that the graph's data points
///   only represent positive numbers.
/// - The trait methods leverage the `Self` keyword to simplify the construction of
///   derived instances of the graph.
///
/// This trait implementation provides essential functionality for working with random
/// walk simulations while enforcing logical data constraints.
impl Walkable for RandomWalkGraph {
    fn get_y_values(&self) -> &Vec<Positive> {
        assert_ne!(
            self.values.len(),
            0,
            "Walkable::get_y_values: values should not be empty"
        );
        &self.values
    }

    fn get_y_values_ref(&mut self) -> &mut Vec<Positive> {
        &mut self.values
    }

    fn get_randon_walk(&self) -> Result<RandomWalkGraph, Box<dyn Error>> {
        Ok(Self {
            values: self.values.clone(),
            title_text: self.title_text.clone(),
            current_index: self.current_index,
            risk_free_rate: self.risk_free_rate,
            dividend_yield: self.dividend_yield,
            time_frame: self.time_frame,
            volatility_window: self.volatility_window,
            initial_volatility: self.initial_volatility,
            volatilities: self.volatilities.clone(),
        })
    }

    fn save_volatility(&mut self, volatility: Positive) -> Result<(), Box<dyn Error>> {
        self.volatilities.push(volatility);
        Ok(())
    }

    fn get_volatilities(&self) -> Result<Vec<Positive>, Box<dyn Error>> {
        Ok(self.volatilities.clone())
    }
}

/// Implements a `Profit` trait for `RandomWalkGraph`, providing functionality
/// for calculating potential profit at a given price level.
impl Profit for RandomWalkGraph {
    /// Calculates the profit at a specified price, returning it as a `Decimal`.
    ///
    /// # Arguments
    /// * `price` - The price at which the profit is being calculated, represented as a `Positive`.
    ///
    /// # Returns
    /// A `Decimal` value representing the calculated profit.
    fn calculate_profit_at(&self, price: Positive) -> Result<Decimal, Box<dyn Error>> {
        Ok(price.to_dec())
    }
}

/// Implements the `Graph` trait for the `RandomWalkGraph` struct, enabling it to
/// provide its title and process y-axis values for graphical visualization.
///
/// The `Graph` trait represents an abstraction for a dataset that can be visualized.
/// It defines methods such as `title` for obtaining the title of the graph and
/// `get_values` for retrieving numerical data required to plot the graph.
///
/// # Methods
///
/// ## `title`
///
/// Retrieves the title of the graph as a `String`.
///
/// - **Purpose**:
///   Provides the textual title for the corresponding graph associated with the
///   `RandomWalkGraph` instance.
///
/// - **Returns**:
///   A `String` containing the graph title. The value is cloned from the `title_text`
///   field of the `RandomWalkGraph`.
///
/// - **Example**:
///   This method is typically used when rendering the graph to set its title text.
///
/// ## `get_values`
///
/// Processes the y-axis data for the graph by transforming the internal representation
/// of values into a vector of `f64`.
///
/// - **Arguments**:
///   - `_data`: A slice of `Positive` values. This is part of the method signature
///     required by the `Graph` trait but is not utilized in this implementation.
///
/// - **Purpose**:
///   Accesses the `values` vector held by the `RandomWalkGraph` instance,
///   converts each `Positive` value into an `f64` using the `Positive::to_f64` method,
///   and collects the results into a new `Vec<f64>`.
///
/// - **Logging Details**:
///   - Logs the total number of `values` using `info!`.
///   - Logs the first and last values in the `values` vector if they're present.
///
/// - **Returns**:
///   A `Vec<f64>` containing the converted y-axis values that are ready to be
///   used for graph plotting.
///
/// - **Panics**:
///   This implementation assumes safe access to the `values` vector and will panic
///   if the vector is empty (e.g., in `unwrap` calls for logging).
///
/// # Notes
/// - This implementation of `Graph` is specialized for the `RandomWalkGraph`
///   struct, which holds metadata and internal data for simulating random walks.
/// - The `values` field in `RandomWalkGraph` must contain valid `Positive` elements
///   that can be converted to `f64` for visualization purposes.
impl Graph for RandomWalkGraph {
    /// Retrieves the title text for the graph.
    fn title(&self) -> String {
        self.title_text.clone()
    }

    /// Processes y-axis values from the graph and converts them into a vector of `f64`.
    ///
    /// # Arguments
    /// * `_data` - A slice of `Positive` values, optionally usable during processing.
    ///
    /// # Returns
    /// A vector of `f64` values as the processed y-axis data.
    fn get_values(&self, _data: &[Positive]) -> Vec<f64> {
        info!("Number of values: {}", self.values.len());
        info!("First value: {:?}", self.values.first().unwrap());
        info!("Last value: {:?}", self.values.last().unwrap());
        self.values.iter().map(|x| x.to_f64()).collect()
    }
}

/// Iterator implementation for `RandomWalkGraph` which generates `OptionDataPriceParams`.
///
/// This iterator traverses through a `RandomWalkGraph` object, producing
/// `OptionDataPriceParams` for each element in the underlying vector of price values.
///
/// # Type Alias
///
/// * `type Item` - Specifies the type of item produced by the iterator,
///   which is `OptionDataPriceParams`.
///
/// # Methods
///
/// * `next(&mut self) -> Option<Self::Item>` - Advances the iterator and
///   returns the next set of option data parameters. If all values have been
///   processed, it returns `None`.
///
///   - Checks if the `current_index` surpasses the length of the `values` vector.
///     If true, iteration stops by returning `None`.
///   - Extracts risk-free rate and dividend yield from their respective options,
///     defaulting to zero if not available.
///   - Retrieves the current price and calculates the remaining days using
///     `get_remaining_time()`.
///   - Determines the expiration date based on the remaining days available until expiration.
///   - Computes the current implied volatility using `calculate_current_volatility()`.
///   - Increments `current_index` to progress through the `values`.
///   - Returns a wrapped `Some()` with fields populated in `OptionDataPriceParams`.
///
/// # Fields
///
/// - `underlying_price`: Current price of the asset.
/// - `expiration_date`: Date at which the option expires, computed as
///   a number of days from the current index.
/// - `implied_volatility`: Estimated volatility of the asset over the
///   remaining time period.
/// - `risk_free_rate`: Interest rate assumed for risk-free investments.
/// - `dividend_yield`: Expected return from dividends, if applicable.
///
/// This design is useful for simulations or models where price and
/// volatility data need to be processed in a time-series format.
impl Iterator for RandomWalkGraph {
    type Item = OptionDataPriceParams;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_index >= self.values.len() {
            return None;
        }

        let risk_free_rate: Decimal = self.risk_free_rate.unwrap_or(Decimal::ZERO);
        let dividend_yield: Positive = self.dividend_yield.unwrap_or(Positive::ZERO);
        let price = self.values[self.current_index];
        let remaining_days = self.get_remaining_time();
        let expiration_date = ExpirationDate::Days(remaining_days);
        let implied_volatility = self.calculate_current_volatility();
        self.current_index += 1;

        Some(OptionDataPriceParams {
            underlying_price: price,
            expiration_date,
            implied_volatility,
            risk_free_rate,
            dividend_yield,
            underlying_symbol: None,
        })
    }
}

/// A custom iterator for iterating through the values of a `RandomWalkGraph`
/// while generating `OptionDataPriceParams` for each step.
///
/// This iterator allows sequential access to the parameters required for
/// option pricing at each step in a random walk simulation.
///
/// # Fields
/// - `walk`: A reference to the `RandomWalkGraph` being traversed.
/// - `current_index`: The current position in the random walk values.
///
/// # Example Overview
/// The iterator consumes the random walk values step by step, calculating relevant
/// option pricing parameters at each step. These parameters include underlying price,
/// implied volatility, expiration date, and others.
///
/// # Iterator Behavior
/// Implements the `Iterator` trait to provide the `next` method, which:
/// 1. Checks whether the end of the walk values (`self.walk.values`) is reached.
///    - If the `current_index` exceeds or equals the length of the values, it returns `None`.
/// 2. Constructs an `OptionDataPriceParams` object for the next step in the walk.
///    - Retrieves:
///      - Risk-free rate (`self.walk.risk_free_rate`)
///      - Dividend yield (`self.walk.dividend_yield`)
///      - Current underlying price from the `walk.values`.
///      - Remaining days until expiration.
///      - Implied volatility from the `calculate_current_volatility` method.
/// 3. Increments `current_index` and returns the constructed `OptionDataPriceParams`.
///
/// # Notes:
/// - If the optional financial parameters `risk_free_rate` or `dividend_yield` are
///   not provided, default values (`Decimal::ZERO` and `Positive::ZERO`) are used.
/// - The remaining time to expiration is expressed in days, calculated from the
///   difference between the length of the `values` and the current iterator index.
///
/// # Performance:
/// - The iterator is designed for efficient sequential processing of the random
///   walk data without requiring explicit indexing by the user.
///
/// # See Also:
/// - [`RandomWalkGraph`]: Contains the random walk data and methods for calculations.
/// - [`OptionDataPriceParams`]: Struct generated at each step containing calculated
///   parameters for pricing options.
pub struct RandomWalkIterator<'a> {
    walk: &'a RandomWalkGraph,
    current_index: usize,
}

impl Iterator for RandomWalkIterator<'_> {
    type Item = OptionDataPriceParams;

    /// Advances the iterator and returns the `OptionDataPriceParams` for the current
    /// step in the random walk. Returns `None` if the iterator reaches the end of the walk.
    ///
    /// # Details
    /// - Retrieves the current underlying price from the `walk.values`.
    /// - Calculates implied volatility using the `walk.calculate_current_volatility` method.
    /// - Computes the remaining days to expiration.
    /// - Uses default values if optional fields (`risk_free_rate` and `dividend_yield`) are unset.
    /// - Advances the `current_index` by 1 for the next iteration.
    ///
    /// # Returns:
    /// - `Some(OptionDataPriceParams)` containing the parameters for the current step.
    /// - `None` if the iterator has completed traversing the random walk values.
    fn next(&mut self) -> Option<Self::Item> {
        if self.current_index >= self.walk.values.len() {
            return None; // End of iterator.
        }
        let risk_free_rate = self.walk.risk_free_rate.unwrap_or(Decimal::ZERO);
        let dividend_yield = self.walk.dividend_yield.unwrap_or(Positive::ZERO);
        let price = self.walk.values[self.current_index];
        let remaining_days = pos!((self.walk.values.len() - self.current_index) as f64);
        let expiration_date = ExpirationDate::Days(remaining_days);
        let implied_volatility = self.walk.calculate_current_volatility();
        self.current_index += 1;
        Some(OptionDataPriceParams {
            underlying_price: price,
            expiration_date,
            implied_volatility,
            risk_free_rate,
            dividend_yield,
            underlying_symbol: None,
        })
    }
}

/// Implements the `Curvable` trait for the `RandomWalkGraph` struct, enabling
/// the generation of a `Curve` representation from its internal state.
///
/// # Functionality
///
/// The `curve` method constructs a `Curve` object by transforming the `values`
/// of the `RandomWalkGraph` into a set of `Point2D` points, where:
/// - The x-coordinate corresponds to the index of the value in the vector.
/// - The y-coordinate is the `Decimal` representation of the `Positive` values stored in the `values` vector.
///
/// This transformation is achieved using the `Point2D::new` constructor and the
/// `.to_dec()` method from the `Positive` struct. The resulting collection of
/// points is passed to `Curve::from_vector`, which generates a `Curve` object.
///
/// # Implementation Details
///
/// - **Iterative Mapping**: The method iterates over the `values` vector, enumerating the indices
///   and transforming each value into a `Point2D` object.
/// - **Curve Construction**: Once the points are collected, `Curve::from_vector` constructs a valid `Curve`
///   object, encapsulating the set of points and their associated properties.
///
/// # Returns
/// - **Success**: Returns a `Curve` object wrapped in `Ok` if all operations are successful.
/// - **Failure**: Propagates any errors encountered during curve construction (e.g., invalid points)
///   through the `Result` type.
///
/// # Errors
/// Potential errors from the `Curve::from_vector` function or invalid `Point2D` conversions
/// (if any) are propagated via the `CurveError` type.
///
/// # Example
/// While examples are excluded as per the request, this method is typically used in scenarios
/// where the `RandomWalkGraph` needs to be analyzed, visualized, or used in mathematical computations.
///
/// # Dependencies
/// - Relies on the `Positive` implementation for converting values to `Decimal` (`to_dec` method).
/// - Uses `Point2D::new` for point creation.
/// - Utilizes `Curve::from_vector` to finalize curve representation.
///
/// # See Also
/// - [`Curvable`]: The trait that defines the `curve` method.
/// - [`Curve`]: The resulting curve object.
/// - [`RandomWalkGraph`]: The struct for which this method is implemented.
impl Curvable for RandomWalkGraph {
    fn curve(&self) -> Result<Curve, CurveError> {
        let points = self
            .values
            .iter()
            .enumerate()
            .map(|(i, p)| Point2D::new(i, p.to_dec()))
            .collect();
        Ok(Curve::from_vector(points))
    }
}
