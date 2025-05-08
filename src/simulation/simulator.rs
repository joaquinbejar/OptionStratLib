use crate::pricing::Profit;
use crate::simulation::WalkParams;
use crate::simulation::randomwalk::RandomWalk;
use crate::simulation::steps::Step;
use crate::strategies::base::BasicAble;
use crate::utils::Len;
use crate::visualization::utils::{
    Graph, GraphBackend, calculate_axis_range, draw_points_on_chart, draw_vertical_lines_on_chart,
    random_color,
};
use crate::{Positive, build_chart_inverted};
use plotters::element::PathElement;
use plotters::prelude::{
    BLACK, BitMapBackend, IntoDrawingArea, LineSeries, RGBColor, SeriesLabelPosition, WHITE,
};
use plotters::style::Color;
use rust_decimal::Decimal;
use std::error::Error;
use std::fmt::Display;
use std::ops::{AddAssign, Index, IndexMut};

/// Represents a generic simulator for managing and simulating random walks.
///
/// # Type Parameters
/// * `X`: A type that represents the state or value within the random walk. It must adhere to the following bounds:
///    - `Copy`: Allows for efficient copying of values.
///    - `Into<Positive>`: Ensures values can be converted into a `Positive` type (potentially for validation or numerical operations).
///    - `AddAssign`: Allows addition and assignment (`+=`) operations.
///    - `Display`: Enables the formatting of values as strings for user-facing output.
///
/// * `Y`: A type that represents the step or transition within the random walk. It must adhere to the following bounds:
///    - `Into<Positive>`: Ensures values can be converted into a `Positive` type.
///    - `Display`: Enables the formatting of values as strings for user-facing output.
///    - `Clone`: Allows for creating deep copies of the values.
///
/// # Fields
/// * `title` (`String`): The name or description of the simulator, primarily used for identification or display purposes.
/// * `random_walks` (`Vec<RandomWalk<X, Y>>`): A collection of `RandomWalk` instances, where each random walk adheres to the defined types `X` and `Y`.
///
/// # Usage
/// This struct is used as a high-level container to manage multiple random walks and perform simulations. Adding specific
/// functionality such as initializing, running simulations, or generating statistical data depends on additional methods provided
/// separately.
///
/// Note: This struct is generic and requires types provided for both state (`X`) and step/transition (`Y`) that meet the respective
/// trait bounds.
pub struct Simulator<X, Y>
where
    X: Copy + Into<Positive> + AddAssign + Display,
    Y: Into<Positive> + Display + Clone,
{
    title: String,
    random_walks: Vec<RandomWalk<X, Y>>,
}

impl<X, Y> Simulator<X, Y>
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
    pub fn new<F>(title: String, size: usize, params: &WalkParams<X, Y>, generator: F) -> Self
    where
        F: Fn(&WalkParams<X, Y>) -> Vec<Step<X, Y>> + Clone,
        X: Copy + Into<Positive> + AddAssign + Display,
        Y: Into<Positive> + Display + Clone,
    {
        let mut random_walks = Vec::new();
        for i in 0..size {
            let title = format!("{}_{}", title, i);
            let random_walk = RandomWalk::new(title, params, &generator);
            random_walks.push(random_walk);
        }
        Self {
            title,
            random_walks,
        }
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

    /// Retrieves the steps of the random walks contained within the current object.
    ///
    /// This method returns a vector of references to `RandomWalk` instances stored
    /// in the `random_walks` collection member of the struct. Each `RandomWalk`
    /// instance represents a step in the random walk process.
    ///
    /// # Returns
    ///
    /// A `Vec` containing references to `RandomWalk<X, Y>` values, where
    /// `X` and `Y` are the types used within the random walk structure.
    ///
    /// # Note
    ///
    /// The returned vector contains borrowed references to the `RandomWalk`
    /// elements within the struct, and the lifetime of these references
    /// is tied to the lifetime of the parent object.
    pub fn get_random_walks(&self) -> Vec<&RandomWalk<X, Y>> {
        self.random_walks.iter().collect::<Vec<&RandomWalk<X, Y>>>()
    } 

    /// Retrieves a reference to the `RandomWalk` at the specified index.
    ///
    /// # Arguments
    ///
    /// * `index` - The index of the desired `RandomWalk` within the `random_walks` collection.
    ///
    /// # Returns
    ///
    /// A reference to the `RandomWalk<X, Y>` located at the given `index`.
    ///
    /// # Panics
    ///
    /// Panics if the `index` is out of bounds for the `random_walks` collection.
    ///
    pub fn get_random_walk(&self, index: usize) -> &RandomWalk<X, Y> {
        &self.random_walks[index]
    }

    /// Retrieves a mutable reference to a `RandomWalk` at the specified index.
    ///
    /// # Arguments
    ///
    /// * `index` - The zero-based index of the `RandomWalk` to access within the `random_walks` collection.
    ///
    /// # Returns
    ///
    /// A mutable reference to the `RandomWalk` object at the given index.
    ///
    /// # Panics
    ///
    /// This function panics if the provided `index` is out of bounds, i.e., if `index >= self.random_walks.len()`.
    ///
    pub fn get_random_walk_mut(&mut self, index: usize) -> &mut RandomWalk<X, Y> {
        &mut self.random_walks[index]
    }

    /// Returns a reference to the first `RandomWalk` element in the `random_walks` collection, if it exists.
    ///
    /// # Returns
    /// - `Some(&RandomWalk<X, Y>)` if the `random_walks` collection is not empty.
    /// - `None` if the `random_walks` collection is empty.
    ///
    pub fn first(&self) -> Option<&RandomWalk<X, Y>> {
        self.random_walks.first()
    }

    /// Returns the last random walk in the collection, if it exists.
    ///
    /// # Returns
    /// - `Some(&RandomWalk<X, Y>)`: A reference to the last `RandomWalk` in the collection.
    /// - `None`: If the collection is empty.
    ///
    /// # Note
    /// The `last` method does not consume the collection; it returns a read-only reference to the last element.
    pub fn last(&self) -> Option<&RandomWalk<X, Y>> {
        self.random_walks.last()
    }
    
    /// Retrieves a nested vector of references to `Step<X, Y>` objects.
    ///
    /// This function iterates over the elements of the current container (`self`)
    /// assuming it implements `IntoIterator`, and for each element, 
    /// calls its `get_steps` method. The results are then collected into a 
    /// two-dimensional `Vec` structure. 
    ///
    /// # Returns
    /// A `Vec` where each inner vector contains references to `Step<X, Y>` objects.
    ///
    /// # Type Parameters
    /// - `X`: The type of the first generic parameter in `Step`.
    /// - `Y`: The type of the second generic parameter in `Step`.
    ///
    pub fn get_steps(&self) -> Vec<Vec<&Step<X, Y>>> {
        self.into_iter()
            .map(|step| step.get_steps())
            .collect()
    }
    

    pub fn last_steps(&self) -> Vec<&Step<X, Y>> {
        self.into_iter()
            .map(|step| step.last().unwrap())
            .collect()
    }

    pub fn last_values(&self) -> Vec<&Step<X, Y>> {
        self.into_iter()
            .map(|step| step.last().unwrap())
            .collect()
    }
}

impl<X, Y> Len for Simulator<X, Y>
where
    X: Copy + Into<Positive> + AddAssign + Display,
    Y: Into<Positive> + Display + Clone,
{
    /// Returns the number of elements in the `random_walks` collection.
    ///
    /// # Returns
    /// - `usize`: The total count of elements in the `random_walks` collection.
    ///
    /// This method is typically used when you need to determine the size
    /// of the internal `random_walks` data structure.
    fn len(&self) -> usize {
        self.random_walks.len()
    }

    /// Checks if the `random_walks` collection is empty.
    ///
    /// # Returns
    /// * `true` - If the `random_walks` collection contains no elements.
    /// * `false` - If the `random_walks` collection contains one or more elements.
    ///
    fn is_empty(&self) -> bool {
        self.random_walks.is_empty()
    }
}

impl<X, Y> Index<usize> for Simulator<X, Y>
where
    X: Copy + Into<Positive> + AddAssign + Display,
    Y: Into<Positive> + Display + Clone,
{
    /// Defines an alias `Output` for the type `RandomWalk<X, Y>`.
    ///
    /// # Type Parameters
    /// - `X`: Represents the type of the first parameter used in the `RandomWalk`.
    /// - `Y`: Represents the type of the second parameter used in the `RandomWalk`.
    ///
    /// `Output` can be used as a shorthand to refer to a `RandomWalk` instance
    /// with specific `X` and `Y` types, improving code readability and reducing
    /// verbosity in the type definitions or method signatures.
    type Output = RandomWalk<X, Y>;

    /// Retrieves a reference to the element at the specified index in the `random_walks` vector.
    ///
    /// # Parameters
    /// - `index`: The zero-based index of the element to retrieve from the `random_walks` vector.
    ///
    /// # Returns
    /// A reference to the element at the specified `index` in the `random_walks` vector.
    ///
    /// # Panics
    /// This function will panic if the given `index` is out of bounds, i.e., greater than or equal to
    /// the length of the `random_walks` vector.
    ///
    /// Note: This implementation assumes that `Self` implements the `Index` trait and
    /// that `random_walks` is a field in the implementing struct.
    fn index(&self, index: usize) -> &Self::Output {
        &self.random_walks[index]
    }
}

impl<X, Y> IndexMut<usize> for Simulator<X, Y>
where
    X: Copy + Into<Positive> + AddAssign + Display,
    Y: Into<Positive> + Display + Clone,
{
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.random_walks[index]
    }
}

impl<X, Y> Display for Simulator<X, Y>
where
    X: Copy + Into<Positive> + AddAssign + Display,
    Y: Into<Positive> + Display + Clone,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.title)?;
        for random_walk in &self.random_walks {
            writeln!(f, "\t{}", random_walk)?;
        }
        Ok(())
    }
}

impl<X, Y> Profit for Simulator<X, Y>
where
    X: AddAssign + Copy + Display + Into<Positive>,
    Y: Clone + Display + Into<Positive>,
{
    fn calculate_profit_at(&self, _price: &Positive) -> Result<Decimal, Box<dyn Error>> {
        Err("Profit calculation not implemented for Simulator".into())
    }
}

impl<X, Y> BasicAble for Simulator<X, Y>
where
    X: AddAssign + Copy + Display + Into<Positive>,
    Y: Clone + Display + Into<Positive>,
{
    fn get_title(&self) -> String {
        self.title.clone()
    }
}

impl<X, Y> Graph for Simulator<X, Y>
where
    X: Copy + Into<Positive> + AddAssign + Display,
    Y: Into<Positive> + Display + Clone,
{
    fn graph(&self, backend: GraphBackend, title_size: u32) -> Result<(), Box<dyn Error>> {
        let all_x_values: Vec<Positive> = self
            .random_walks
            .iter()
            .flat_map(|walk| walk.get_x_values())
            .collect();

        if all_x_values.is_empty() {
            return Err("No valid X values to plot".into());
        }

        let all_y_values: Vec<f64> = self
            .random_walks
            .iter()
            .flat_map(|walk| walk.get_y_values())
            .collect();

        if all_y_values.is_empty() {
            return Err("No valid Y values to plot".into());
        }

        let (max_x_value, min_x_value, max_y_value, min_y_value) =
            calculate_axis_range(&all_x_values, &all_y_values, Some(1.005));

        let root = match backend {
            GraphBackend::Bitmap { file_path, size } => {
                let root = BitMapBackend::new(file_path, size).into_drawing_area();
                root.fill(&WHITE)?;
                root
            }
        };

        let mut chart = build_chart_inverted!(
            &root,
            self.get_title(),
            title_size,
            min_x_value.to_f64(),
            max_x_value.to_f64(),
            min_y_value,
            max_y_value
        );

        chart
            .configure_mesh()
            .x_labels(20)
            .y_labels(20)
            .x_label_formatter(&|x| format!("{:.2}", x))
            .y_label_formatter(&|y| format!("{:.2}", y))
            .draw()?;

        let colors: Vec<RGBColor> = (0..self.random_walks.len())
            .map(|_| random_color())
            .collect();

        for (i, walk) in self.random_walks.iter().enumerate() {
            let x_values: Vec<f64> = walk.get_x_values().iter().map(|x| x.to_f64()).collect();

            let y_values = walk.get_y_values();

            if !x_values.is_empty() && !y_values.is_empty() && x_values.len() == y_values.len() {
                let color_index = i % colors.len();
                let line_color = colors[color_index];

                chart
                    .draw_series(LineSeries::new(
                        x_values.iter().zip(y_values.iter()).map(|(&x, &y)| (x, y)),
                        line_color,
                    ))?
                    .label(format!("Walk {}", i))
                    .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], line_color));
            }
        }

        chart
            .configure_series_labels()
            .background_style(WHITE.mix(0.8))
            .border_style(BLACK)
            .position(SeriesLabelPosition::UpperLeft) // Posición en la parte superior izquierda
            .draw()?;

        draw_points_on_chart(&mut chart, &self.get_points())?;
        draw_vertical_lines_on_chart(&mut chart, &self.get_vertical_lines())?;

        root.present()?;
        Ok(())
    }

    fn get_x_values(&self) -> Vec<Positive> {
        self.random_walks
            .iter()
            .flat_map(|step| step.get_x_values())
            .collect()
    }

    fn get_y_values(&self) -> Vec<f64> {
        self.random_walks
            .iter()
            .flat_map(|step| step.get_y_values())
            .collect()
    }
}

impl<'a, X, Y> IntoIterator for &'a Simulator<X, Y>
where
    X: Copy + Into<Positive> + AddAssign + Display,
    Y: Into<Positive> + Display + Clone,
{
    type Item = &'a RandomWalk<X, Y>;
    type IntoIter = std::slice::Iter<'a, RandomWalk<X, Y>>;

    fn into_iter(self) -> Self::IntoIter {
        self.random_walks.iter()
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use super::*;
    use crate::chains::generator_positive;
    use crate::simulation::{
        WalkParams, WalkType, WalkTypeAble,
        steps::{Step, Xstep, Ystep},
    };
    use crate::utils::{TimeFrame, time::convert_time_frame, setup_logger};
    use crate::{ExpirationDate, Positive, pos};
    use rust_decimal_macros::dec;
    use tracing::{debug, info};

    // Helper structs and functions for testing
    struct TestWalker;

    impl TestWalker {
        fn new() -> Self {
            TestWalker {}
        }
    }
    impl WalkTypeAble<Positive, Positive> for TestWalker {}

    fn test_generator(params: &WalkParams<Positive, Positive>) -> Vec<Step<Positive, Positive>> {
        vec![params.init_step.clone()]
    }

    // Test Simulator creation
    #[test]
    fn test_simulator_creation() {
        let walker = Box::new(TestWalker);
        let initial_price = pos!(100.0);
        let init_step = Step {
            x: Xstep::new(
                Positive::ONE,
                TimeFrame::Minute,
                ExpirationDate::Days(pos!(30.0)),
            ),
            y: Ystep::new(0, initial_price),
        };

        let walk_params = WalkParams {
            size: 5,
            init_step,
            walk_type: WalkType::GeometricBrownian {
                dt: convert_time_frame(pos!(1.0) / pos!(30.0), &TimeFrame::Minute, &TimeFrame::Day),
                drift: dec!(0.0),
                volatility: pos!(0.2),
            },
            walker,
        };

        let simulator = Simulator::new(
            "Test Simulator".to_string(),
            5,
            &walk_params,
            test_generator,
        );

        assert_eq!(simulator.get_title(), "Test Simulator");
        assert_eq!(simulator.len(), 5);
        assert!(!simulator.is_empty());
    }

    // Test title methods
    #[test]
    fn test_simulator_title_methods() {
        let walker = Box::new(TestWalker);
        let initial_price = pos!(100.0);
        let init_step = Step {
            x: Xstep::new(
                Positive::ONE,
                TimeFrame::Minute,
                ExpirationDate::Days(pos!(30.0)),
            ),
            y: Ystep::new(0, initial_price),
        };

        let walk_params = WalkParams {
            size: 3,
            init_step,
            walk_type: WalkType::GeometricBrownian {
                dt: convert_time_frame(pos!(1.0) / pos!(30.0), &TimeFrame::Minute, &TimeFrame::Day),
                drift: dec!(0.0),
                volatility: pos!(0.2),
            },
            walker,
        };

        let mut simulator = Simulator::new(
            "Original Title".to_string(),
            3,
            &walk_params,
            test_generator,
        );

        assert_eq!(simulator.get_title(), "Original Title");

        simulator.set_title("New Title".to_string());
        assert_eq!(simulator.get_title(), "New Title");
    }

    // Test step access methods
    #[test]
    fn test_simulator_step_access() {
        let walker = Box::new(TestWalker);
        let initial_price = pos!(100.0);
        let init_step = Step {
            x: Xstep::new(
                Positive::ONE,
                TimeFrame::Minute,
                ExpirationDate::Days(pos!(30.0)),
            ),
            y: Ystep::new(0, initial_price),
        };

        let walk_params = WalkParams {
            size: 3,
            init_step,
            walk_type: WalkType::GeometricBrownian {
                dt: convert_time_frame(pos!(1.0) / pos!(30.0), &TimeFrame::Minute, &TimeFrame::Day),
                drift: dec!(0.0),
                volatility: pos!(0.2),
            },
            walker,
        };

        let simulator = Simulator::new(
            "Test Simulator".to_string(),
            3,
            &walk_params,
            test_generator,
        );

        // Test get_steps
        let steps = simulator.get_random_walks();
        assert_eq!(steps.len(), 3);

        // Test get_step
        let step = simulator.get_random_walk(1);
        assert_eq!(step.get_title(), "Test Simulator_1");

        // Test first and last
        assert!(simulator.first().is_some());
        assert!(simulator.last().is_some());
        assert_eq!(simulator.first().unwrap().get_title(), "Test Simulator_0");
        assert_eq!(simulator.last().unwrap().get_title(), "Test Simulator_2");
    }

    // Test Index and IndexMut traits
    #[test]
    fn test_simulator_indexing() {
        let walker = Box::new(TestWalker);
        let initial_price = pos!(100.0);
        let init_step = Step {
            x: Xstep::new(
                Positive::ONE,
                TimeFrame::Minute,
                ExpirationDate::Days(pos!(30.0)),
            ),
            y: Ystep::new(0, initial_price),
        };

        let walk_params = WalkParams {
            size: 3,
            init_step,
            walk_type: WalkType::GeometricBrownian {
                dt: convert_time_frame(pos!(1.0) / pos!(30.0), &TimeFrame::Minute, &TimeFrame::Day),
                drift: dec!(0.0),
                volatility: pos!(0.2),
            },
            walker,
        };

        let mut simulator = Simulator::new(
            "Test Simulator".to_string(),
            3,
            &walk_params,
            test_generator,
        );

        // Test immutable indexing
        assert_eq!(simulator[0].get_title(), "Test Simulator_0");
        assert_eq!(simulator[1].get_title(), "Test Simulator_1");
        assert_eq!(simulator[2].get_title(), "Test Simulator_2");

        // Test mutable indexing
        simulator[1].set_title("Modified Title".to_string());
        assert_eq!(simulator[1].get_title(), "Modified Title");
    }

    // Test display formatting
    #[test]
    fn test_simulator_display() {
        let walker = Box::new(TestWalker);
        let initial_price = pos!(100.0);
        let init_step = Step {
            x: Xstep::new(
                Positive::ONE,
                TimeFrame::Minute,
                ExpirationDate::Days(pos!(30.0)),
            ),
            y: Ystep::new(0, initial_price),
        };

        let walk_params = WalkParams {
            size: 2,
            init_step,
            walk_type: WalkType::GeometricBrownian {
                dt: convert_time_frame(pos!(1.0) / pos!(30.0), &TimeFrame::Minute, &TimeFrame::Day),
                drift: dec!(0.0),
                volatility: pos!(0.2),
            },
            walker,
        };

        let simulator = Simulator::new("Display Test".to_string(), 2, &walk_params, test_generator);

        let display_output = format!("{}", simulator);
        assert!(display_output.starts_with("Display Test"));
        assert!(display_output.contains("Display Test_0"));
        assert!(display_output.contains("Display Test_1"));
    }

    // Test simulator with empty collection
    #[test]
    fn test_simulator_empty() {
        let simulator: Simulator<Positive, Positive> = Simulator {
            title: "Empty Simulator".to_string(),
            random_walks: Vec::new(),
        };

        assert_eq!(simulator.get_title(), "Empty Simulator");
        assert_eq!(simulator.len(), 0);
        assert!(simulator.is_empty());
        assert!(simulator.first().is_none());
        assert!(simulator.last().is_none());
    }

    // Test panic scenarios (these would typically be in separate test functions)
    #[test]
    #[should_panic(expected = "index out of bounds")]
    fn test_simulator_index_out_of_bounds() {
        let walker = Box::new(TestWalker);
        let initial_price = pos!(100.0);
        let init_step = Step {
            x: Xstep::new(
                Positive::ONE,
                TimeFrame::Minute,
                ExpirationDate::Days(pos!(30.0)),
            ),
            y: Ystep::new(0, initial_price),
        };

        let walk_params = WalkParams {
            size: 3,
            init_step,
            walk_type: WalkType::GeometricBrownian {
                dt: convert_time_frame(pos!(1.0) / pos!(30.0), &TimeFrame::Minute, &TimeFrame::Day),
                drift: dec!(0.0),
                volatility: pos!(0.2),
            },
            walker,
        };

        let simulator = Simulator::new("Panic Test".to_string(), 3, &walk_params, test_generator);

        // This should panic
        let _ = simulator[3];
    }

    #[test]
    fn test_simulator_graph() -> Result<(), Box<dyn Error>> {
        struct Walker {}

        impl Walker {
            fn new() -> Self {
                Walker {}
            }
        }

        impl WalkTypeAble<Positive, Positive> for Walker {}

        let simulator_size: usize = 5;
        let n_steps = 10;
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

        let mut simulator = Simulator::new(
            "Simulator".to_string(),
            simulator_size,
            &walk_params,
            generator_positive,
        );

        let y_values = simulator.get_y_values();
        let x_values = simulator.get_x_values();

        assert_eq!(y_values.len(), simulator_size * n_steps);
        assert_eq!(x_values.len(), simulator_size * n_steps);

        let mut iter = simulator.into_iter();
        assert!(iter.any(|step| step.get_y_values().len() == n_steps));
        assert!(iter.any(|step| step.get_x_values().len() == n_steps));
        assert!(simulator.calculate_profit_at(&pos!(100.0)).is_err());

        let step = simulator.get_random_walk_mut(0);
        assert!(step.first().is_some());

        let file_path = "Draws/Simulation/simulator_test.png";
        assert!(
            simulator
                .graph(
                    GraphBackend::Bitmap {
                        file_path,
                        size: (1200, 800),
                    },
                    20,
                )
                .is_ok()
        );

        assert!(std::fs::remove_file(file_path).is_ok());

        Ok(())
    }
    
    #[test]
    fn test_full_simulation() -> Result<(), Box<dyn Error>> {
        setup_logger();
        let simulator_size: usize = 15;
        let n_steps = 120;
        let initial_price = pos!(100.0);
        let std_dev = pos!(20.0);
        let walker = Box::new(TestWalker::new());
        let days = pos!(30.0);

        let walk_params = WalkParams {
            size: n_steps,
            init_step: Step {
                x: Xstep::new(Positive::ONE, TimeFrame::Minute, ExpirationDate::Days(days)),
                y: Ystep::new(0, initial_price),
            },
            walk_type: WalkType::GeometricBrownian {
                dt: convert_time_frame(pos!(1.0) / days, &TimeFrame::Minute, &TimeFrame::Day), // TODO
                drift: dec!(0.0),
                volatility: std_dev,
            },
            walker,
        };
        
        assert_eq!(walk_params.size, n_steps);
        assert_eq!(walk_params.init_step.get_value(), &pos!(100.0));
        assert_eq!(walk_params.y(), &pos!(100.0));

        let simulator = Simulator::new(
            "Simulator".to_string(),
            simulator_size,
            &walk_params,
            generator_positive,
        );
        debug!("Simulator: {}", simulator);
        // println!("{}", simulator);
        assert_eq!(simulator.get_title(), "Simulator");
        assert_eq!(simulator.len(), simulator_size);
        
        
        let random_walk = simulator[0].clone();
        assert_eq!(random_walk.get_title(), "Simulator_0");
        assert_eq!(random_walk.len(), n_steps);
        
        let step = random_walk[0].clone();
        assert_eq!(*step.get_index(), Positive::ONE);
        let step_string = format!("{}", step);
        assert_eq!(step.to_string(), step_string);
        
        let y_step = step.get_y_step();
        assert_eq!(*y_step.index(), 0);
        assert_eq!(*y_step.value(), pos!(100.0));

        let x_step = step.get_x_step();
        assert_eq!(*x_step.index(), 0);
        assert_eq!(*x_step.step_size_in_time(), Positive::ONE);
        assert_eq!(x_step.time_unit(), &TimeFrame::Minute);
        assert_eq!(x_step.days_left()?, pos!(30.0));
        
        
        
        
        let next_step = step.next(pos!(200.0));
        assert!(next_step.is_ok());
        let next_step = next_step?;
        assert_eq!(next_step.get_value(), &pos!(200.0));
        let next_step_string = format!("{}", next_step);
        assert_eq!(next_step.to_string(), next_step_string);
        
        let previous_step = step.previous(pos!(50.0))?;
        assert_eq!(previous_step.get_value(), &pos!(50.0));
        let previous_step_string = format!("{}", previous_step);
        assert_eq!(previous_step.to_string(), previous_step_string);
        
        let x_step = step.get_x_step();
        let next_x_step = x_step.next();
        assert!(next_x_step.is_ok());
        let next_x_step = next_x_step?;
        assert_eq!(*next_x_step.index(), 1);
        assert_eq!(*next_x_step.step_size_in_time(), Positive::ONE);
        let next_x_step_string = format!("{}", next_x_step);
        assert_eq!(next_x_step.to_string(), next_x_step_string);
        
        let y_step = step.get_y_step();
        assert_eq!(*y_step.index(), 0);
        assert_eq!(*y_step.value(), pos!(100.0));
        assert_eq!(y_step.positive(), pos!(100.0));
        

        let last_steps: Vec<&Step<Positive,Positive>> = simulator
            .into_iter()
            .map(|step| step.last().unwrap())
            .collect();
        info!("Last Steps: {:?}", last_steps);
        assert_eq!(last_steps.len(), simulator_size);

        let last_values: Vec<&Positive> = simulator
            .into_iter()
            .map(|step| step.last().unwrap().get_value())
            .collect();
        info!("Last Values: {:?}", last_values);
        assert_eq!(last_values.len(), simulator_size);
        

        let file_name = "Draws/Simulation/test_simulator.png";
        simulator.graph(
            GraphBackend::Bitmap {
                file_path: file_name,
                size: (1200, 800),
            },
            20,
        )?;
        let remove_result = fs::remove_file(file_name);
        assert!(remove_result.is_ok());

        Ok(())
    }
}
