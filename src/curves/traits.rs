/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 23/2/25
******************************************************************************/
use crate::curves::Curve;
use crate::error::CurveError;

/// A trait that defines the behavior of any object that can produce a curve representation.
///
/// Types implementing the `Curvable` trait must provide the `curve` method, which generates
/// a `Curve` object based on the internal state of the implementer. This method returns
/// a `Result`, allowing for proper error handling in situations where the curve
/// cannot be generated due to various issues (e.g., invalid data or computation errors).
///
/// # Method
///
/// - `curve`
///   - **Returns**:
///     - A `Result` containing:
///       - `Curve`: On success, a valid representation of the curve.
///       - `CurveError`: On failure, detailed information about why the curve could not be generated.
///
/// # Error Handling
///
/// Given the reliance on precise data and operations, the `Curvable` trait integrates
/// tightly with the `CurveError` type to handle potential issues, such as:
/// - Invalid points or coordinates (`Point2DError`)
/// - Issues in curve construction (`ConstructionError`)
/// - Errors during interpolation (`InterpolationError`)
/// - General computation or operational failures (`OperationError`, `StdError`)
///
/// # Example Usage
///
/// This trait forms the basis for creating highly customizable and precise curve objects,
/// ensuring compatibility with mathematical, computational, or graphical operations.
///
/// Implementing this trait allows an object to seamlessly interact with the higher-level
/// functionalities in the `curves` module, such as visualization, analysis, and transformation.
///
/// # See Also
/// - [`Curve`]: Represents the mathematical curve generated by this trait.
/// - [`CurveError`]: Error type encapsulating issues encountered during curve generation.
pub trait Curvable {
    /// Generates a `Curve` representation of the implementer.
    ///
    /// The `curve` method is the core functionality of this trait. It is expected
    /// to compute and return a `Curve` object that accurately describes the
    /// implementer's structure or state in the context of a two-dimensional curve.
    ///
    /// # Returns
    ///
    /// - `Ok(Curve)`: When the curve is successfully generated.
    /// - `Err(CurveError)`: If curve generation fails for any reason.
    ///
    /// # Errors
    ///
    /// The method may return a `CurveError` in scenarios such as:
    /// - **Point2DError**: If an invalid or missing 2D point is encountered.
    /// - **ConstructionError**: When the curve cannot be initialized due to invalid input.
    /// - **InterpolationError**: If there are issues during interpolation of the curve's points.
    /// - **AnalysisError**: In cases where analytical operations on the curve fail.
    ///
    /// This ensures robust error handling for downstream processes and applications.
    fn curve(&self) -> Result<Curve, CurveError>;
}
