/// Represents the different types of interpolation methods supported in the library.
///
/// This enum defines the available interpolation techniques that can be used in various
/// mathematical modeling, data analysis, and visualization scenarios. Each interpolation
/// method has unique characteristics making it suitable for different applications.
///
/// # Variants
/// - **`Linear`**:  
///   Performs linear interpolation. This method calculates interpolated values
///   by connecting two adjacent points on a line and using the slope between
///   them to determine new values.
///
/// - **`Cubic`**:  
///   Utilizes cubic interpolation, which involves fitting a cubic polynomial
///   between data points to provide a smoother curve. This is commonly used for
///   precise curve fitting and smoothing applications.
///
/// - **`Spline`**:  
///   Implements spline interpolation, typically involving piecewise polynomials
///   (e.g., cubic splines) that ensure smooth transitions between data points
///   while maintaining continuity of derivatives at the joins. Spline methods
///   are widely used in mathematical modeling and graphical applications.
///
/// - **`Bilinear`**:  
///   A two-dimensional extension of linear interpolation. Commonly used in grid-based
///   data interpolation, bilinear interpolation calculates the value at a point within
///   a cell of a 2D grid based on the values at the vertices of the cell.
///
/// # Usage
/// This enumeration is typically used as part of a larger interpolation framework,
/// providing a way to select the desired interpolation method. Each variant corresponds
/// to a specific interpolation strategy, and implementations for these can be found in
/// their respective modules (e.g., `linear`, `cubic`, `spline`, `bilinear`).
///
/// # Integration
/// The `InterpolationType` enum is re-exported in the library's root module to make it
/// easily accessible as part of the public API. It works in conjunction with traits such as
/// `Interpolate`, allowing users to dynamically specify the desired interpolation type
/// when performing operations.
///
/// # Example Use Case
/// While the `InterpolationType` enum itself does not contain any methods or logic,
/// it serves as a configuration point for choosing the interpolation method to be used
/// in a context where multiple types of interpolation are supported.
///
/// # See Also
/// - [`crate::geometrics::LinearInterpolation`]: Implements linear interpolation.
/// - [`crate::geometrics::CubicInterpolation`]: Implements cubic interpolation.
/// - [`crate::geometrics::SplineInterpolation`]: Implements spline interpolation.
/// - [`crate::geometrics::BiLinearInterpolation`]: Implements bilinear interpolation.
///
/// This enum is part of a modular design, with each interpolation type defined in its own
/// module for clarity and separation of concerns.
///
#[derive(Debug, Clone, Copy)]
pub enum InterpolationType {
    /// Linear interpolation that calculates values by drawing straight lines between data points.
    /// Efficient but may lack smoothness for certain applications.
    Linear,

    /// Cubic interpolation that uses third-degree polynomials to create smoother curves
    /// between data points, providing better continuity than linear interpolation.
    Cubic,

    /// Spline interpolation that ensures smooth curves with continuous first and second
    /// derivatives at the connection points between interpolation segments.
    Spline,

    /// Bilinear interpolation for two-dimensional data, performing linear interpolation
    /// first in one direction and then in the other, used commonly in image processing
    /// and spatial data analysis.
    Bilinear,
}