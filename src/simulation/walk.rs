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
use crate::simulation::types::Walktypable;
use crate::simulation::utils::calculate_extra_metrics;
use crate::strategies::Strategable;
use crate::utils::Len;
use crate::utils::time::{TimeFrame, convert_time_frame, units_per_year};
use crate::visualization::model::ChartPoint;
use crate::visualization::utils::Graph;
use crate::{Positive, pos};
use num_traits::FromPrimitive;
use rand::distributions::Distribution;
use rand::thread_rng as rng;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use statrs::distribution::Normal;
use std::collections::HashMap;
use std::error::Error;
use tracing::{debug, info, trace, warn};

/// The `Walkable` trait defines a generic structure for creating and manipulating
/// entities capable of simulating or managing a random walk sequence of values.
/// Implementations of this trait must handle a vector of `Positive` values, which
/// serve as the primary storage for the y-axis values used in simulations or computations.
pub trait Walkable<Xtype, Ytype>
where
    Ytype: Walktypable,
{
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
    fn get_y_values(&self) -> &Vec<Ytype>;

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

    /// Returns a reference to the vector of `Xtype` elements associated with this object.
    ///
    /// This method provides read-only access to the collection of `Xtype` elements
    /// stored within this object. The default implementation is a placeholder
    /// that will panic if called, indicating that implementing types must
    /// override this method with their own implementation.
    ///
    /// # Returns
    ///
    /// * `&Vec<Xtype>` - A reference to the vector containing the `Xtype` elements.
    ///
    /// # Panics
    ///
    /// By default, this method panics with a message indicating that it has not been
    /// implemented for the type in question. Implementing types should override this
    /// method to provide their own implementation.
    ///
    fn get_x_type(&self) -> &Vec<Xtype> {
        unimplemented!("get_x_type not implemented for this type")
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
    fn get_y_values_ref(&mut self) -> &mut Vec<Ytype>;

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
        initial_price: Ytype,
        drift: f64,
        volatility: Positive,
        volatility_change: Positive,
    ) -> Result<(), Box<dyn Error>> {
        if n_steps == 0 {
            return Err(Box::from("Number of steps must be greater than zero"));
        }

        let mut thread_rng = rng();
        let mut current_volatility = volatility;

        let dt: f64 = 1.0 / 252.0;
        let sqrt_dt = dt.sqrt();

        let values = self.get_y_values_ref();
        values.clear();
        values.reserve(n_steps);
        values.push(initial_price.clone());

        let mut volatilities = Vec::with_capacity(n_steps);
        volatilities.push(current_volatility); // Add initial volatility

        for _ in 0..n_steps - 1 {
            // Potentially update volatility (stochastic volatility model)
            if volatility_change > Positive::ZERO {
                current_volatility = Normal::new(volatility.into(), volatility_change.into())
                    .unwrap()
                    .sample(&mut thread_rng)
                    .max(ZERO)
                    .into();
            }

            volatilities.push(current_volatility);

            // Generate a standard normal random variable
            let z = Normal::new(0.0, 1.0).unwrap().sample(&mut thread_rng);

            // Calculate price movement using the log-normal model
            // S(t+dt) = S(t) * exp((mu - 0.5*sigma^2)*dt + sigma*sqrt(dt)*z)
            let vol = current_volatility.to_f64();
            let drift_term = (drift - 0.5 * vol * vol) * dt;
            let volatility_term = vol * sqrt_dt * z;

            let log_return = drift_term + volatility_term;
            let current_price = initial_price.walk_next(log_return)?;
            values.push(current_price.clone());

            trace!(
                "Current price: {}, Volatility: {}",
                current_price, current_volatility
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
        initial_price: Ytype,
        drift: f64,
        volatility: Positive,        // daily volatility
        volatility_change: Positive, // daily VoV
        time_frame: TimeFrame,
        volatility_limits: Option<(Positive, Positive)>,
    ) -> Result<(), Box<dyn Error>> {
        if n_steps == 0 {
            return Err(Box::from("Number of steps must be greater than zero"));
        }

        let mut thread_rng = rng();
        let mut current_volatility = volatility;

        // Calculate dt based on the time_frame
        let periods_per_year = units_per_year(&time_frame);
        let dt = 1.0 / periods_per_year.to_f64();
        let sqrt_dt = dt.sqrt();

        let values = self.get_y_values_ref();
        values.clear();
        values.reserve(n_steps);
        values.push(initial_price.clone());

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
                    * Normal::new(0.0, 1.0).unwrap().sample(&mut thread_rng);
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
            let z = Normal::new(0.0, 1.0).unwrap().sample(&mut thread_rng);

            // Calculate price movement using the log-normal model
            let vol = current_volatility.to_f64();
            let drift_term = (drift - 0.5 * vol * vol) * dt;
            let volatility_term = vol * sqrt_dt * z;

            let log_return = drift_term + volatility_term;
            let current_price = initial_price.walk_next(log_return)?;
            values.push(current_price.clone());

            trace!(
                "Current price: {}, Volatility: {}",
                current_price, current_volatility
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

    /// Stores the provided volatility value in the internal storage.
    ///
    /// This function is a placeholder implementation that currently does nothing but
    /// will be used to save volatility values for future reference or analysis.
    ///
    /// # Parameters
    /// * `_volatility` - A `Positive` value representing the volatility to be stored
    ///
    /// # Returns
    /// * `Result<(), Box<dyn Error>>` - Ok(()) on success or an error if the operation fails
    fn save_volatility(&mut self, _volatility: Positive) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    /// Retrieves all stored volatility values.
    ///
    /// This function is intended to return the collection of volatility values that have been
    /// previously saved. Currently unimplemented.
    ///
    /// # Returns
    /// * `Result<Vec<Positive>, Box<dyn Error>>` - A vector of `Positive` volatility values on success
    ///   or an error if the operation fails
    fn get_volatilities(&self) -> Result<Vec<Positive>, Box<dyn Error>> {
        unimplemented!()
    }

    /// Generates and returns a random walk graph based on the current configuration.
    ///
    /// This function creates a new `RandomWalkGraph` instance that can be used for
    /// price path simulation and analysis. The random walk will be generated using
    /// the parameters stored in the current instance.
    ///
    /// # Returns
    /// * `Result<RandomWalkGraph, Box<dyn Error>>` - A random walk graph instance on success
    ///   or an error if the generation process fails
    fn get_random_walk(&self) -> Result<RandomWalkGraph<Ytype>, Box<dyn Error>>;

    /// Simulates a strategy over a price walk and analyzes its performance.
    ///
    /// This method applies a trading strategy to historical price data (a "walk") and
    /// evaluates how the strategy would have performed over that period. It calculates
    /// profit and loss at different points, tracks maximum gains and drawdowns, and
    /// collects performance metrics throughout the price movement.
    ///
    /// # Arguments
    ///
    /// * `strategy` - A mutable reference to any type implementing the `Strategable` trait
    /// * `time_frame` - The time frame used for the simulation (e.g., Day, Week, Month)
    ///
    /// # Returns
    ///
    /// * `Result<WalkResult, Box<dyn Error>>` - Either a `WalkResult` containing detailed
    ///   performance metrics or an error if the simulation couldn't be completed
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * No walk data exists
    /// * Volatility data length doesn't match price data length
    /// * PnL calculation fails for any price point
    ///
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
        };

        let values = self.get_y_values();
        let values_len: Positive = Decimal::from_usize(values.len()).unwrap().into();
        let days = convert_time_frame(values_len, &time_frame, &TimeFrame::Day);
        strategy.set_expiration_date(ExpirationDate::Days(days))?;

        let initially = values.first().unwrap().walk_dec()?;
        let finally = values.last().unwrap().walk_dec()?;
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

        let values_positive = values
            .iter()
            .map(|x| x.walk_positive().unwrap())
            .collect::<Vec<_>>();
        let extra_metrics = calculate_extra_metrics(&values_positive).unwrap_or_else(|err| {
            warn!("Failed to calculate extra metrics: {}", err);
            HashMap::new()
        });

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
            extra_metrics,
            volatilities: volatilities.clone(),
        };

        // Debug output to check volatilities
        debug!(
            "Using {} volatility values for {} price points",
            volatilities.len(),
            values.len()
        );

        // Reverse i for expiration date descending
        for (i, (price, volatility)) in values.iter().zip(volatilities.iter()).enumerate().rev() {
            let price_dec = price.walk_dec()?;
            let days_left = convert_time_frame(pos!(i as f64), &time_frame, &TimeFrame::Day);

            // Debug log to track calculations
            debug!(
                "Step {}: Params: Underlying Price: {}, Expiration: {} Years, Implied Volatility: {}",
                i,
                price,
                days_left.to_f64() / 365.0,
                volatility
            );

            let underlying_price = price.walk_positive()?;
            let pnl = if !days_left.is_zero() {
                strategy.calculate_pnl(
                    &underlying_price,
                    ExpirationDate::Days(days_left),
                    volatility,
                )?
            } else {
                strategy.calculate_pnl_at_expiration(&underlying_price)?
            };

            // Ensure unwrap is safe
            let pnl_unrealized = match pnl.unrealized {
                Some(value) => value,
                None => match pnl.realized {
                    Some(value) => value,
                    None => {
                        return Err(format!(
                            "No unrealized PnL calculated for price: {}",
                            price_dec
                        )
                        .into());
                    }
                },
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

/// # RandomWalkGraph
///
/// A structure implementing the `Walkable` trait specialized for random walk simulation.
/// This struct models financial asset price movements with advanced features for
/// sophisticated financial simulations.
///
/// `RandomWalkGraph` provides functionality for modeling asset price paths using random walk
/// principles, with support for financial parameters like risk-free rates and dividend yields.
/// It can be used to generate, store, and analyze price paths with customizable characteristics.
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
/// - **volatilities** (`Vec<Positive>`):  
///   Stores the calculated volatility values at each step of the random walk,
///   allowing for analysis of volatility patterns and potentially implementing
///   stochastic volatility models.
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RandomWalkGraph<Ytype = Positive>
where
    Ytype: Walktypable,
{
    /// Values representing the y-axis data of the random walk.
    pub(crate) values: Vec<Ytype>,

    /// Text for the graph's title.
    title_text: String,

    /// Tracks the current index for traversing the graph.
    current_index: usize,

    /// Optional risk-free rate used in calculations.
    risk_free_rate: Option<Decimal>,

    /// Optional dividend yield percentage.
    dividend_yield: Option<Positive>,

    /// Specifies the timeframe (e.g., daily, weekly) for the graph calculations.
    pub(crate) time_frame: TimeFrame,

    /// Determines the window size used in volatility calculations.
    volatility_window: usize,

    /// Optional initial volatility of the random walk.
    initial_volatility: Option<Positive>,

    /// Records the calculated volatility at each step of the random walk.
    pub volatilities: Vec<Positive>,
}

/// Implements the `Len` trait for `RandomWalkGraph`, providing methods to check the
/// size and emptiness of the random walk data.
///
/// This implementation allows `RandomWalkGraph` instances to report their length and
/// empty status, which is determined by the underlying `values` vector that stores
/// the sequence of price points in the random walk.
///
/// ## Methods
///
/// - `len()`: Returns the number of price points in the random walk
/// - `is_empty()`: Indicates whether the random walk contains any data points
///
/// ## Example
///
/// ```
/// use optionstratlib::Positive;
/// use optionstratlib::simulation::RandomWalkGraph;
/// use optionstratlib::utils::{Len, TimeFrame};
/// let walk: RandomWalkGraph<Positive>  = RandomWalkGraph::new("".to_string(), None, None, TimeFrame::Microsecond, 0, None);
/// let point_count = walk.len();
/// let has_data = !walk.is_empty();
/// ```
impl<Ytype: Walktypable> Len for RandomWalkGraph<Ytype> {
    /// Returns the number of price points in the random walk graph.
    ///
    /// This method directly reflects the size of the internal `values` vector,
    /// which represents the sequence of price points in the simulation.
    fn len(&self) -> usize {
        self.values.len()
    }

    /// Checks if the random walk graph contains any data points.
    ///
    /// Returns `true` if there are no price points in the graph,
    /// `false` otherwise.
    fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

impl<Ytype: Walktypable> RandomWalkGraph<Ytype> {
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

    /// Retrieves the initial volatility of the random walk.
    ///
    /// This function returns an `Option<Positive>` representing the initial
    /// volatility value. If no initial volatility was set, it returns `None`.
    ///
    /// # Returns
    ///
    /// An `Option<Positive>` containing the initial volatility, or `None` if not set.
    pub fn get_initial_volatility(&self) -> Option<Positive> {
        self.initial_volatility
    }

    /// Sets the initial volatility of the random walk.
    ///
    /// This function allows updating the initial volatility of the random walk
    /// with a new `Positive` value wrapped in an `Option`.
    ///
    /// # Arguments
    ///
    /// * `initial_volatility` - A reference to an `Option<Positive>` containing
    ///   the new initial volatility value.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure. On success, it returns `Ok(())`.
    /// On failure, it returns an `Err` containing a boxed `Error` trait object
    /// describing the error.
    pub fn set_initial_volatility(
        &mut self,
        initial_volatility: &Option<Positive>,
    ) -> Result<(), Box<dyn Error>> {
        self.initial_volatility = *initial_volatility;
        Ok(())
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
            .map(|w| (w[1].walk_f64() / w[0].walk_f64()).ln())
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
            .map(|(index, value)| {
                ChartPoint::new(
                    (index as f64, value.walk_positive().unwrap()),
                    format!("Step {}", index),
                )
            })
            .collect()
    }

    /// Creates a fresh iterator for this random walk graph.
    ///
    /// This method returns a clone of the current instance with the iterator position
    /// reset to the beginning (index 0). This allows multiple iterations over the same
    /// random walk data without modifying the original instance.
    ///
    /// # Returns
    ///
    /// A new `RandomWalkGraph` instance with the same data but with `current_index` reset to 0,
    /// ready for iteration from the beginning.
    ///
    /// # Examples
    ///
    /// ```rust
    ///
    /// use optionstratlib::Positive;
    /// use optionstratlib::simulation::RandomWalkGraph;
    /// use optionstratlib::utils::{Len, TimeFrame};
    /// let mut walk : RandomWalkGraph<Positive>  = RandomWalkGraph::new("".to_string(), None, None, TimeFrame::Microsecond, 0, None);
    ///
    /// // Consume half the walk in some operation
    /// for _ in 0..walk.len() / 2 {
    ///     let _ = walk.next();
    /// }
    /// ```
    ///
    /// # Note
    ///
    /// This method is particularly useful when you need to perform multiple passes over
    /// the walk data, such as for analysis or visualization purposes.
    pub fn iter(&self) -> Self
    where
        Self: Clone,
    {
        let mut cloned = self.clone();
        cloned.current_index = 0;
        cloned
    }
}

impl<Ytype: Walktypable> Default for RandomWalkGraph<Ytype> {
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
/// ### `get_random_walk`
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
impl<Ytype: Walktypable> Walkable<Positive, Ytype> for RandomWalkGraph<Ytype> {
    fn get_y_values(&self) -> &Vec<Ytype> {
        assert_ne!(
            self.values.len(),
            0,
            "Walkable::get_y_values: values should not be empty"
        );
        &self.values
    }

    fn get_y_values_ref(&mut self) -> &mut Vec<Ytype> {
        &mut self.values
    }

    fn save_volatility(&mut self, volatility: Positive) -> Result<(), Box<dyn Error>> {
        self.volatilities.push(volatility);
        Ok(())
    }

    fn get_volatilities(&self) -> Result<Vec<Positive>, Box<dyn Error>> {
        Ok(self.volatilities.clone())
    }

    fn get_random_walk(&self) -> Result<RandomWalkGraph<Ytype>, Box<dyn Error>> {
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
}

/// Implements a `Profit` trait for `RandomWalkGraph`, providing functionality
/// for calculating potential profit at a given price level.
impl<Ytype: Walktypable> Profit for RandomWalkGraph<Ytype> {
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
impl<Ytype: Walktypable> Graph for RandomWalkGraph<Ytype> {
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
        info!("First value: {}", self.values.first().unwrap());
        info!("Last value: {}", self.values.last().unwrap());
        self.values.iter().map(|x| x.walk_f64()).collect()
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
impl<Ytype: Walktypable> Iterator for RandomWalkGraph<Ytype> {
    type Item = OptionDataPriceParams;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_index >= self.values.len() {
            return None; // End of iterator.
        }
        let risk_free_rate = self.risk_free_rate.unwrap_or(Decimal::ZERO);
        let dividend_yield = self.dividend_yield.unwrap_or(Positive::ZERO);
        let price: Ytype = self.values[self.current_index].clone();
        let remaining_step = pos!((self.values.len() - self.current_index) as f64);
        let left_time = convert_time_frame(remaining_step, &self.time_frame, &TimeFrame::Day);
        if left_time.is_zero() {
            return None;
        }
        let expiration_date = ExpirationDate::Days(left_time);
        let implied_volatility = self.calculate_current_volatility();
        self.current_index += 1;
        Some(OptionDataPriceParams {
            underlying_price: price.walk_positive().unwrap(),
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
impl<Ytype: Walktypable> Curvable for RandomWalkGraph<Ytype> {
    fn curve(&self) -> Result<Curve, CurveError> {
        let points = self
            .values
            .iter()
            .enumerate()
            .map(|(i, p)| Point2D::new(i, p.walk_dec().unwrap()))
            .collect();
        Ok(Curve::from_vector(points))
    }
}
