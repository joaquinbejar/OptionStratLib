use crate::pricing::Profit;
use crate::simulation::WalkParams;
use crate::simulation::randomwalk::RandomWalk;
use crate::simulation::steps::Step;
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

    pub fn get_steps(&self) -> Vec<&RandomWalk<X, Y>> {
        self.random_walks.iter().collect::<Vec<&RandomWalk<X, Y>>>()
    }

    pub fn get_step(&self, index: usize) -> &RandomWalk<X, Y> {
        &self.random_walks[index]
    }

    pub fn get_step_mut(&mut self, index: usize) -> &mut RandomWalk<X, Y> {
        &mut self.random_walks[index]
    }

    pub fn first(&self) -> Option<&RandomWalk<X, Y>> {
        self.random_walks.first()
    }

    pub fn last(&self) -> Option<&RandomWalk<X, Y>> {
        self.random_walks.last()
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
    fn calculate_profit_at(&self, _price: Positive) -> Result<Decimal, Box<dyn Error>> {
        unimplemented!()
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
            self.title(),
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
                        line_color.clone(),
                    ))?
                    .label(&format!("Walk {}", i))
                    .legend(move |(x, y)| {
                        PathElement::new(vec![(x, y), (x + 20, y)], line_color.clone())
                    });
            }
        }

        chart
            .configure_series_labels()
            .background_style(&WHITE.mix(0.8))
            .border_style(&BLACK)
            .position(SeriesLabelPosition::UpperLeft) // Posición en la parte superior izquierda
            .draw()?;

        draw_points_on_chart(&mut chart, &self.get_points())?;
        draw_vertical_lines_on_chart(&mut chart, &self.get_vertical_lines())?;

        root.present()?;
        Ok(())
    }

    fn title(&self) -> String {
        self.title.clone()
    }

    fn get_x_values(&self) -> Vec<Positive> {
        self.random_walks
            .iter()
            .map(|step| step.get_x_values())
            .flatten()
            .collect()
    }

    fn get_y_values(&self) -> Vec<f64> {
        self.random_walks
            .iter()
            .map(|step| step.get_y_values())
            .flatten()
            .collect()
    }
}
