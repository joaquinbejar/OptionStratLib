use crate::visualization::styles::PlotType;

/// A trait defining the type of graph or plot used for visualization purposes.
///
/// The `GraphType` trait is designed to provide a standardized way to determine
/// the visualization type for a given data representation. Implementors of this
/// trait should define the specific plot type that suits their data or visualization needs.
///
/// # Usage
/// Implement this trait for any data structure or context that requires a
/// well-defined graph type for displaying data.
///
pub trait GraphType {
    /// Returns the type of plot to be used for visualization.
    ///
    /// # Description
    /// The `plot_type` function determines and returns the type of plot that should
    /// be used. This information can be useful when configuring or rendering visual
    /// data representations. The returned `PlotType` is typically an enum or specific
    /// type representing various available plot types (e.g., line plot, bar chart, scatter plot, etc.).
    ///
    /// # Returns
    /// * `PlotType` - The specific type of plot identified for use.
    ///
    fn plot_type() -> PlotType;
}
