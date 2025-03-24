/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 23/3/25
******************************************************************************/
use crate::Positive;
use crate::simulation::steps::Step;
use crate::simulation::walk::WalkParams;
use crate::utils::Len;
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
pub struct RandomWalk<X, Y>
where
    X: Copy + Into<Positive> + AddAssign + Display,
    Y: Copy + Into<Positive> + Display,
{
    /// The descriptive title of the random walk
    title: String,

    /// The collection of steps that make up the random walk path
    steps: Vec<Step<X, Y>>,
}

impl<X, Y> RandomWalk<X, Y>
where
    X: Copy + Into<Positive> + AddAssign + Display,
    Y: Copy + Into<Positive> + Display,
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
    pub fn new<F>(title: String, params: WalkParams<X, Y>, generator: F) -> Self
    where
        F: FnOnce(WalkParams<X, Y>) -> Vec<Step<X, Y>>,
        X: Copy + Into<Positive> + AddAssign + Display,
        Y: Copy + Into<Positive> + Display,
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
    Y: Copy + Into<Positive> + Display,
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
    Y: Copy + Into<Positive> + Display,
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
    Y: Copy + Into<Positive> + Display,
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
    Y: Copy + Into<Positive> + Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "RandomWalk Title: {}, Steps:  ", self.title)?;
        for step in &self.steps {
            write!(f, "\t{}", step)?;
        }
        Ok(())
    }
}
