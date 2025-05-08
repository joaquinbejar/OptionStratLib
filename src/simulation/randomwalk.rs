/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 23/3/25
******************************************************************************/
use crate::constants::{DARK_GREEN, DARK_RED};
use crate::pricing::Profit;
use crate::simulation::WalkParams;
use crate::simulation::steps::Step;
use crate::strategies::base::BasicAble;
use crate::utils::Len;
use crate::visualization::utils::{
    Graph, GraphBackend, calculate_axis_range, draw_points_on_chart, draw_vertical_lines_on_chart,
};
use crate::{Positive, build_chart_inverted, create_drawing_area};
use plotters::prelude::{BLACK, BitMapBackend, IntoDrawingArea, LineSeries, WHITE};
use rust_decimal::Decimal;
use std::error::Error;
use std::fmt::Display;
use std::ops::{AddAssign, Index, IndexMut};

/// A struct that represents a two-dimensional random walk simulation.
///
/// `RandomWalk` stores a sequence of steps that describe a path in a two-dimensional space,
/// typically used for financial modeling, time series analysis, or statistical simulations.
/// It maintains both the steps of the random walk and a descriptive title.
///
/// # Type Parameters
///
/// * `X` - The type for x-axis values (typically representing time or sequence position),
///   which must implement `AddAssign` (allowing values to be accumulated), be convertible
///   to `Positive`, and be `Copy`.
///
/// * `Y` - The type for y-axis values (typically representing price, value, or position),
///   which must implement `AddAssign`, be convertible to `Positive`, be `Copy`, and implement
///   the `Walktypable` trait for additional functionality.
///
#[derive(Debug, Clone, Default)]
pub struct RandomWalk<X, Y>
where
    X: Copy + Into<Positive> + AddAssign + Display,
    Y: Into<Positive> + Display + Clone,
{
    /// The descriptive title of the random walk
    title: String,

    /// The collection of steps that make up the random walk path
    steps: Vec<Step<X, Y>>,
}

impl<X, Y> RandomWalk<X, Y>
where
    X: Copy + Into<Positive> + AddAssign + Display,
    Y: Into<Positive> + Display + Clone,
{
    /// Creates a new random walk instance with the given title and steps.
    ///
    /// This constructor takes a title, walk parameters, and a generator function
    /// that produces the actual steps of the random walk based on the provided parameters.
    ///
    /// # Parameters
    ///
    /// * `title` - A descriptive title for the random walk
    /// * `params` - Parameters that define the properties of the random walk
    /// * `generator` - A function that generates the steps of the random walk
    ///
    /// # Returns
    ///
    /// A new `RandomWalk` instance with the generated steps.
    ///
    pub fn new<F>(title: String, params: &WalkParams<X, Y>, generator: F) -> Self
    where
        F: FnOnce(&WalkParams<X, Y>) -> Vec<Step<X, Y>>,
        X: Copy + Into<Positive> + AddAssign + Display,
        Y: Into<Positive> + Display + Clone,
    {
        let steps = generator(params);
        Self { title, steps }
    }

    /// Returns the title of the random walk.
    ///
    /// # Returns
    ///
    /// A string slice containing the title of the random walk.
    pub fn get_title(&self) -> &str {
        &self.title
    }

    /// Updates the title of the random walk.
    ///
    /// # Parameters
    ///
    /// * `title` - The new title to set
    pub fn set_title(&mut self, title: String) {
        self.title = title;
    }

    /// Returns a vector of references to all steps in the random walk.
    ///
    /// # Returns
    ///
    /// A vector containing references to all steps in the random walk.
    pub fn get_steps(&self) -> Vec<&Step<X, Y>> {
        self.steps.iter().collect::<Vec<&Step<X, Y>>>()
    }

    /// Returns a reference to the step at the specified index.
    ///
    /// # Parameters
    ///
    /// * `index` - The zero-based index of the step to retrieve
    ///
    /// # Returns
    ///
    /// A reference to the step at the specified index.
    ///
    /// # Panics
    ///
    /// Panics if the index is out of bounds.
    pub fn get_step(&self, index: usize) -> &Step<X, Y> {
        &self.steps[index]
    }

    /// Returns a mutable reference to the step at the specified index.
    ///
    /// # Parameters
    ///
    /// * `index` - The zero-based index of the step to retrieve
    ///
    /// # Returns
    ///
    /// A mutable reference to the step at the specified index.
    ///
    /// # Panics
    ///
    /// Panics if the index is out of bounds.
    pub fn get_step_mut(&mut self, index: usize) -> &mut Step<X, Y> {
        &mut self.steps[index]
    }

    /// Returns a reference to the first step in the random walk, if any.
    ///
    /// # Returns
    ///
    /// * `Some(&Step<X, Y>)` - A reference to the first step if the random walk is not empty
    /// * `None` - If the random walk has no steps
    pub fn first(&self) -> Option<&Step<X, Y>> {
        self.steps.first()
    }

    /// Returns a reference to the last step in the random walk, if any.
    ///
    /// # Returns
    ///
    /// * `Some(&Step<X, Y>)` - A reference to the last step if the random walk is not empty
    /// * `None` - If the random walk has no steps
    pub fn last(&self) -> Option<&Step<X, Y>> {
        self.steps.last()
    }
}

/// Implementation of the `Len` trait for `RandomWalk<X, Y>`.
///
/// This implementation provides methods to determine the length and emptiness
/// of a random walk by delegating to the underlying `steps` collection.
///
/// # Type Parameters
///
/// * `X` - The type for x-axis values (typically time or sequence position),
///   which must implement `AddAssign`, be convertible to `Positive`, and be `Copy`.
///
/// * `Y` - The type for y-axis values (typically price or value),
///   which must implement `AddAssign`, be convertible to `Positive`, be `Copy`,
///   and implement the `Walktypable` trait.
impl<X, Y> Len for RandomWalk<X, Y>
where
    X: Copy + Into<Positive> + AddAssign + Display,
    Y: Into<Positive> + Display + Clone,
{
    /// Returns the number of steps in the random walk.
    ///
    /// # Returns
    ///
    /// A `usize` representing the number of steps.
    fn len(&self) -> usize {
        self.steps.len()
    }

    /// Determines whether the random walk contains any steps.
    ///
    /// # Returns
    ///
    /// `true` if the random walk has no steps, `false` otherwise.
    fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }
}

/// Implementation of the `Index` trait for `RandomWalk<X, Y>`.
///
/// This allows accessing the steps of a random walk using array indexing notation:
/// `walk[index]`.
///
/// # Type Parameters
///
/// * `X` - The type for x-axis values, with constraints as described above.
/// * `Y` - The type for y-axis values, with constraints as described above.
impl<X, Y> Index<usize> for RandomWalk<X, Y>
where
    X: Copy + Into<Positive> + AddAssign + Display,
    Y: Into<Positive> + Display + Clone,
{
    /// The type returned when indexing the random walk.
    type Output = Step<X, Y>;

    /// Provides read access to a specific step in the random walk by index.
    ///
    /// # Parameters
    ///
    /// * `index` - The zero-based index of the step to access.
    ///
    /// # Returns
    ///
    /// A reference to the `Step<X, Y>` at the specified index.
    ///
    /// # Panics
    ///
    /// Panics if the index is out of bounds.
    fn index(&self, index: usize) -> &Self::Output {
        &self.steps[index]
    }
}

/// Implementation of the `IndexMut` trait for `RandomWalk<X, Y>`.
///
/// This allows modifying steps in a random walk using array indexing notation:
/// `walk[index] = new_step`.
///
/// # Type Parameters
///
/// * `X` - The type for x-axis values, with constraints as described above.
/// * `Y` - The type for y-axis values, with constraints as described above.
impl<X, Y> IndexMut<usize> for RandomWalk<X, Y>
where
    X: Copy + Into<Positive> + AddAssign + Display,
    Y: Into<Positive> + Display + Clone,
{
    /// Provides mutable access to a specific step in the random walk by index.
    ///
    /// # Parameters
    ///
    /// * `index` - The zero-based index of the step to modify.
    ///
    /// # Returns
    ///
    /// A mutable reference to the `Step<X, Y>` at the specified index.
    ///
    /// # Panics
    ///
    /// Panics if the index is out of bounds.
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.steps[index]
    }
}

impl<X, Y> Display for RandomWalk<X, Y>
where
    X: Copy + Into<Positive> + AddAssign + Display,
    Y: Into<Positive> + Display + Clone,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "RandomWalk Title: {}, Steps:  ", self.title)?;
        for step in &self.steps {
            write!(f, "\t{}", step)?;
        }
        Ok(())
    }
}

impl<X, Y> Profit for RandomWalk<X, Y>
where
    X: AddAssign + Copy + Display + Into<Positive>,
    Y: Into<Positive> + Display + Clone,
{
    fn calculate_profit_at(&self, _price: &Positive) -> Result<Decimal, Box<dyn Error>> {
        Err("Profit calculation not implemented for RandomWalk".into())
    }
}

impl<X, Y> BasicAble for RandomWalk<X, Y>
where
    X: AddAssign + Copy + Display + Into<Positive>,
    Y: Clone + Display + Into<Positive>,
{
    fn get_title(&self) -> String {
        self.title.clone()
    }
}

impl<X, Y> Graph for RandomWalk<X, Y>
where
    X: Copy + Into<Positive> + AddAssign + Display,
    Y: Into<Positive> + Display + Clone,
{
    fn graph(&self, backend: GraphBackend, title_size: u32) -> Result<(), Box<dyn Error>> {
        // Get X values from the random walk
        let x_values = self.get_x_values();
        let x_axis_data: &[Positive] = &x_values;

        // Check if there are valid X values to plot
        if x_axis_data.is_empty() {
            return Err("No valid values to plot".into());
        }

        // Get Y values from the random walk
        let y_axis_data: Vec<f64> = self.get_y_values();

        // Check if there are valid Y values to plot
        if y_axis_data.is_empty() {
            return Err("No valid values to plot".into());
        }

        // Calculate the range for both axes
        let (max_x_value, min_x_value, max_y_value, min_y_value) =
            calculate_axis_range(x_axis_data, &y_axis_data, Some(1.005));

        // Set up the drawing area based on the backend
        let root = match backend {
            GraphBackend::Bitmap { file_path, size } => {
                create_drawing_area!(file_path, size.0, size.1)
            }
        };

        // Here's the key change: We manually build the chart with inverted X-axis
        let mut chart = build_chart_inverted!(
            &root,
            self.get_title(),
            title_size,
            min_x_value.to_f64(),
            max_x_value.to_f64(),
            min_y_value,
            max_y_value
        );

        // Configure the chart same as the original
        chart
            .configure_mesh()
            .disable_mesh() // Keep the original style (no grid)
            .x_labels(20)
            .y_labels(20)
            .draw()?;

        // Draw a horizontal line at y = 0
        chart.draw_series(LineSeries::new(
            vec![(max_x_value.to_f64(), 0.0), (min_x_value.to_f64(), 0.0)],
            &BLACK,
        ))?;

        // Draw the line segments using the original style
        let mut last_point: Option<(f64, f64)> = None;
        for (&x, &y) in x_axis_data.iter().zip(y_axis_data.iter()) {
            if let Some((last_x, last_y)) = last_point {
                let color_to_use = if y > 0.0 { &DARK_GREEN } else { &DARK_RED };

                let points: Vec<(f64, f64)> = vec![(last_x, last_y), (x.to_f64(), y)];
                chart.draw_series(LineSeries::new(points, color_to_use))?;
            }
            last_point = Some((x.to_f64(), y));
        }

        // Draw any additional points and vertical lines
        draw_points_on_chart(&mut chart, &self.get_points())?;
        draw_vertical_lines_on_chart(&mut chart, &self.get_vertical_lines())?;

        // Present the final chart
        root.present()?;
        Ok(())
    }

    fn get_x_values(&self) -> Vec<Positive> {
        self.steps
            .iter()
            .map(|step| step.get_graph_x_in_days_left())
            .collect()
    }

    fn get_y_values(&self) -> Vec<f64> {
        self.steps
            .iter()
            .map(|step| step.get_graph_y_value().to_f64())
            .collect()
    }
}

#[cfg(test)]
mod tests_random_walk {
    use super::*;
    use crate::ExpirationDate;
    use crate::Positive;
    use crate::chains::generator_positive;
    use crate::pos;
    use crate::simulation::WalkParams;
    use crate::simulation::WalkType;
    use crate::simulation::WalkTypeAble;
    use crate::simulation::steps::{Step, Xstep, Ystep};
    use crate::utils::TimeFrame;
    use crate::utils::time::convert_time_frame;
    use num_traits::ToPrimitive;
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;
    use std::error::Error;
    use std::fmt::Display;
    use std::ops::AddAssign;

    // Mock implementation of WalkTypeAble for testing
    struct TestWalker {}

    impl<X, Y> WalkTypeAble<X, Y> for TestWalker
    where
        X: Copy + Into<Positive> + AddAssign + Display,
        Y: Into<Positive> + Display + Clone,
    {
        // We'll implement the simplest possible method for testing
        fn brownian(&self, params: &WalkParams<X, Y>) -> Result<Vec<Positive>, Box<dyn Error>> {
            let mut values = Vec::new();
            let init_value: Positive = params.ystep_as_positive();
            values.push(init_value);

            // Generate some simple steps for test purposes
            for i in 1..params.size {
                values.push(pos!(init_value.value().to_f64().unwrap() + i as f64));
            }

            Ok(values)
        }
    }

    // Helper function to create a walk parameters struct for testing
    fn create_test_params<X, Y>(
        size: usize,
        x_value: X,
        y_value: Y,
        walk_type: WalkType,
    ) -> WalkParams<X, Y>
    where
        X: Copy + Into<Positive> + AddAssign + Display,
        Y: Into<Positive> + Display + Clone,
    {
        let init_step = Step::new(
            x_value,
            TimeFrame::Day,
            ExpirationDate::Days(pos!(30.0)),
            y_value,
        );

        WalkParams {
            size,
            init_step,
            walk_type,
            walker: Box::new(TestWalker {}),
        }
    }

    // Helper function to generate test steps for a random walk
    fn generate_test_steps<X, Y>(params: &WalkParams<X, Y>) -> Vec<Step<X, Y>>
    where
        X: Copy + Into<Positive> + AddAssign + Display,
        Y: Into<Positive> + Display + Clone,
    {
        let mut steps = Vec::new();
        steps.push(params.init_step.clone());

        let test_walker = TestWalker {};
        let values = test_walker.brownian(params).unwrap();

        let mut current_step = params.init_step.clone();

        // Skip the first value as it's the initial step
        for _value in values.iter().skip(1) {
            // Convert Positive back to Y type (for test we'll just use the same value)
            let new_y_value = current_step.y.value();

            // Create next step
            let next_step = current_step.next(new_y_value.clone()).unwrap();
            steps.push(next_step.clone());

            current_step = next_step;
        }

        steps
    }

    #[test]
    fn test_random_walk_creation() {
        let params = create_test_params(
            5,
            1.0,
            100.0,
            WalkType::Brownian {
                dt: pos!(1.0),
                drift: Decimal::ZERO,
                volatility: pos!(0.2),
            },
        );

        let title = "Test Random Walk".to_string();
        let walk = RandomWalk::new(title.clone(), &params, generate_test_steps);

        assert_eq!(walk.get_title(), title);
        assert_eq!(walk.len(), 5);
        assert!(!walk.is_empty());
    }

    #[test]
    fn test_random_walk_empty() {
        let params = create_test_params(
            0,
            1.0,
            100.0,
            WalkType::Brownian {
                dt: pos!(1.0),
                drift: Decimal::ZERO,
                volatility: pos!(0.2),
            },
        );

        let title = "Empty Walk".to_string();
        let walk = RandomWalk::new(title.clone(), &params, |_| Vec::new());

        assert_eq!(walk.get_title(), title);
        assert_eq!(walk.len(), 0);
        assert!(walk.is_empty());
        assert!(walk.first().is_none());
        assert!(walk.last().is_none());
    }

    #[test]
    fn test_random_walk_title_update() {
        let params = create_test_params(
            5,
            1.0,
            100.0,
            WalkType::Brownian {
                dt: pos!(1.0),
                drift: Decimal::ZERO,
                volatility: pos!(0.2),
            },
        );

        let title = "Initial Title".to_string();
        let mut walk = RandomWalk::new(title, &params, generate_test_steps);

        let new_title = "Updated Title".to_string();
        walk.set_title(new_title.clone());

        assert_eq!(walk.get_title(), new_title);
    }

    #[test]
    fn test_random_walk_first_last() {
        let params = create_test_params(
            5,
            1.0,
            100.0,
            WalkType::Brownian {
                dt: pos!(1.0),
                drift: Decimal::ZERO,
                volatility: pos!(0.2),
            },
        );

        let walk = RandomWalk::new("Test Walk".to_string(), &params, generate_test_steps);

        let first = walk.first().unwrap();
        let last = walk.last().unwrap();

        assert_eq!(*first.x.index(), 0);
        assert_eq!(*last.x.index(), 4); // 5 steps, zero-indexed
    }

    #[test]
    fn test_random_walk_get_steps() {
        let params = create_test_params(
            5,
            1.0,
            100.0,
            WalkType::Brownian {
                dt: pos!(1.0),
                drift: Decimal::ZERO,
                volatility: pos!(0.2),
            },
        );

        let walk = RandomWalk::new("Test Walk".to_string(), &params, generate_test_steps);

        let steps = walk.get_steps();
        assert_eq!(steps.len(), 5);

        // Test that steps have sequential index values
        for (i, step) in steps.iter().enumerate() {
            assert_eq!(*step.x.index(), i as i32);
        }
    }

    #[test]
    fn test_random_walk_get_step() {
        let params = create_test_params(
            5,
            1.0,
            100.0,
            WalkType::Brownian {
                dt: pos!(1.0),
                drift: Decimal::ZERO,
                volatility: pos!(0.2),
            },
        );

        let walk = RandomWalk::new("Test Walk".to_string(), &params, generate_test_steps);

        let step_0 = walk.get_step(0);
        let step_3 = walk.get_step(3);

        assert_eq!(*step_0.x.index(), 0);
        assert_eq!(*step_3.x.index(), 3);
    }

    #[test]
    #[should_panic(expected = "index out of bounds")]
    fn test_random_walk_get_step_out_of_bounds() {
        let params = create_test_params(
            5,
            1.0,
            100.0,
            WalkType::Brownian {
                dt: pos!(1.0),
                drift: Decimal::ZERO,
                volatility: pos!(0.2),
            },
        );

        let walk = RandomWalk::new("Test Walk".to_string(), &params, generate_test_steps);

        // This should panic
        let _step = walk.get_step(10);
    }

    #[test]
    fn test_random_walk_get_step_mut() {
        let params = create_test_params(
            5,
            1.0,
            100.0,
            WalkType::Brownian {
                dt: pos!(1.0),
                drift: Decimal::ZERO,
                volatility: pos!(0.2),
            },
        );

        let mut walk = RandomWalk::new("Test Walk".to_string(), &params, generate_test_steps);

        // Get a mutable reference and verify initial state
        let step_2 = walk.get_step_mut(2);
        assert_eq!(*step_2.x.index(), 2);

        // Get a new step by calling next on the current step
        let new_y_value = *step_2.y.value();
        let new_step = step_2.clone();
        *step_2 = new_step.next(new_y_value * 2.0).unwrap();

        // Verify the step was updated
        assert_eq!(*walk.get_step(2).x.index(), 3);
    }

    #[test]
    fn test_random_walk_index_operator() {
        let params = create_test_params(
            5,
            1.0,
            100.0,
            WalkType::Brownian {
                dt: pos!(1.0),
                drift: Decimal::ZERO,
                volatility: pos!(0.2),
            },
        );

        let walk = RandomWalk::new("Test Walk".to_string(), &params, generate_test_steps);

        // Test read access via index operator
        let step_1 = &walk[1];
        assert_eq!(*step_1.x.index(), 1);

        // Test comparison between get_step and index operator
        assert_eq!(*walk.get_step(3).x.index(), *walk[3].x.index());
    }

    #[test]
    fn test_random_walk_index_mut_operator() {
        let params = create_test_params(
            5,
            1.0,
            100.0,
            WalkType::Brownian {
                dt: pos!(1.0),
                drift: Decimal::ZERO,
                volatility: pos!(0.2),
            },
        );

        let mut walk = RandomWalk::new("Test Walk".to_string(), &params, generate_test_steps);

        // Get initial step via index
        let initial_index = *walk[2].x.index();

        // Modify step via index_mut operator
        let new_y_value = *walk[2].y.value();
        let new_step = walk[2].clone();
        walk[2] = new_step.next(new_y_value * 2.0).unwrap();

        // Verify the change
        assert_ne!(*walk[2].x.index(), initial_index);
    }

    #[test]
    fn test_random_walk_display() {
        let params = create_test_params(
            3,
            1.0,
            100.0,
            WalkType::Brownian {
                dt: pos!(1.0),
                drift: Decimal::ZERO,
                volatility: pos!(0.2),
            },
        );

        let walk = RandomWalk::new("Display Test".to_string(), &params, generate_test_steps);

        // Test that the display output contains the title
        let display_output = format!("{}", walk);
        assert!(display_output.contains("Display Test"));
    }

    #[test]
    fn test_random_walk_graph_implementation() {
        let params = create_test_params(
            5,
            1.0,
            100.0,
            WalkType::Brownian {
                dt: pos!(1.0),
                drift: Decimal::ZERO,
                volatility: pos!(0.2),
            },
        );

        let walk = RandomWalk::new("Graph Test".to_string(), &params, generate_test_steps);

        // Test Graph implementation methods
        assert_eq!(walk.get_title(), "Graph Test");

        let x_values = walk.get_x_values();
        assert_eq!(x_values.len(), 5);

        let y_values = walk.get_y_values();
        assert_eq!(y_values.len(), 5);
    }

    #[test]
    fn test_with_different_types() {
        // Test with custom types for X and Y
        #[derive(Debug, Copy, Clone, PartialEq)]
        struct TestX(f64);

        impl Display for TestX {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl AddAssign for TestX {
            fn add_assign(&mut self, other: Self) {
                self.0 += other.0;
            }
        }

        impl From<TestX> for Positive {
            fn from(val: TestX) -> Self {
                pos!(val.0)
            }
        }

        #[derive(Debug, Copy, Clone, PartialEq)]
        struct TestY(f64);

        impl Display for TestY {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl From<TestY> for Positive {
            fn from(val: TestY) -> Self {
                pos!(val.0)
            }
        }

        // Create params with custom types
        let params = create_test_params(
            3,
            TestX(1.0),
            TestY(100.0),
            WalkType::Brownian {
                dt: pos!(1.0),
                drift: Decimal::ZERO,
                volatility: pos!(0.2),
            },
        );

        // Custom generator for TestX and TestY
        let generator = |params: &WalkParams<TestX, TestY>| {
            let mut steps = Vec::new();
            steps.push(params.init_step.clone());

            let mut current_step = params.init_step.clone();
            for i in 1..params.size {
                let next_step = current_step.next(TestY((100.0 + i as f64) * 1.1)).unwrap();
                steps.push(next_step.clone());
                current_step = next_step;
            }

            steps
        };

        let walk = RandomWalk::new("Custom Types Test".to_string(), &params, generator);

        assert_eq!(walk.len(), 3);
        assert_eq!(*walk[0].y.value(), TestY(100.0));
    }

    #[test]
    fn test_graph() -> Result<(), Box<dyn Error>> {
        struct Walker {}

        impl Walker {
            fn new() -> Self {
                Walker {}
            }
        }

        impl WalkTypeAble<Positive, Positive> for Walker {}

        let n_steps = 43_200; // 30 days in minutes
        let initial_price = pos!(100.0);
        let std_dev = pos!(20.0);
        let walker = Box::new(Walker::new());
        let days = pos!(30.0);

        let walk_params = WalkParams {
            size: n_steps,
            init_step: Step {
                x: Xstep::new(Positive::ONE, TimeFrame::Minute, ExpirationDate::Days(days)),
                y: Ystep::new(0, initial_price),
            },
            walk_type: WalkType::GeometricBrownian {
                dt: convert_time_frame(pos!(1.0) / days, &TimeFrame::Minute, &TimeFrame::Day),
                drift: dec!(0.0),
                volatility: std_dev,
            },
            walker,
        };

        let random_walk =
            RandomWalk::new("Random Walk".to_string(), &walk_params, generator_positive);
        assert!(random_walk.calculate_profit_at(&pos!(100.0)).is_err());

        let steps = random_walk.get_steps();
        let y = steps.first().unwrap().y.clone();
        assert_eq!(y.value(), 100.0);
        let result_next = steps.first().unwrap().next(pos!(100.0));
        assert!(result_next.is_ok());
        let next = result_next.unwrap();
        assert!(
            next.to_string()
                .contains("Step { x: Xstep { index: 1, value: 1, time_unit: Minute, datetime:")
        );

        let previous_next = steps.first().unwrap().previous(pos!(100.0));
        assert!(previous_next.is_ok());
        let previous = previous_next.unwrap();
        assert!(
            previous
                .to_string()
                .contains("Step { x: Xstep { index: -1, value: 1, time_unit: Minute, datetime:")
        );

        assert_eq!(next.get_graph_x_value()?, Decimal::ONE);

        let file_path = "Draws/Simulation/random_walk_test.png";
        random_walk.graph(
            GraphBackend::Bitmap {
                file_path,
                size: (1200, 800),
            },
            20,
        )?;
        assert!(std::fs::remove_file(file_path).is_ok());

        Ok(())
    }
}
