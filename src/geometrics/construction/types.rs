/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 26/8/24
******************************************************************************/

use rust_decimal::Decimal;
use std::collections::BTreeSet;
use std::error::Error;
use utoipa::ToSchema;

/// A result type for geometric point operations that may fail.
///
/// This type alias provides a consistent way to handle point generation operations
/// that could result in errors, encapsulating the resulting point or an error.
pub type ResultPoint<Point> = Result<Point, Box<dyn Error>>;

/// Parameters for constructing geometric objects in different dimensions.
///
/// This enum provides configuration options for different dimensional spaces
/// when generating geometric objects through parametric equations or sampling.
/// It supports both 2D curves (with a single parameter t) and 3D surfaces
/// (with two parameters x and y).
#[derive(Debug, Clone, ToSchema)]
pub enum ConstructionParams {
    /// Parameters for constructing a 2D curve with a single parameter t.
    D2 {
        /// Starting value for the parameter t
        t_start: Decimal,

        /// Ending value for the parameter t
        t_end: Decimal,

        /// Number of sampling points between t_start and t_end
        steps: usize,
    },

    /// Parameters for constructing a 3D surface with parameters x and y.
    D3 {
        /// Start parameter for x
        x_start: Decimal,

        /// End parameter for x  
        x_end: Decimal,

        /// Start parameter for y
        y_start: Decimal,

        /// End parameter for y
        y_end: Decimal,

        /// Number of steps in x direction
        x_steps: usize,

        /// Number of steps in y direction
        y_steps: usize,
    },
}

/// Defines methods for constructing geometric objects.
///
/// This enum provides two primary approaches to creating geometric objects:
/// - Direct construction from a set of predefined points
/// - Parametric generation using a mathematical function
///
/// The generic parameters allow flexibility in the types of points and input parameters used:
/// - `Point`: The type representing a coordinate in the geometric space
/// - `Input`: The parameter type passed to parametric functions (typically `Decimal` for 2D or `(Decimal, Decimal)` for 3D)
pub enum ConstructionMethod<Point, Input> {
    /// Construct a geometric object from an explicit set of points.
    ///
    /// This method uses a sorted collection of points to directly define
    /// the geometry without requiring any computation.
    FromData {
        /// Ordered collection of points that define the geometric object
        points: BTreeSet<Point>,
    },

    /// Construct a geometric object using a parametric function.
    ///
    /// This method generates points by evaluating a mathematical function
    /// at specific parameter values defined by the construction parameters.
    Parametric {
        /// Function that maps from parameter space to point coordinates.
        /// Must be thread-safe to support parallel computation.
        f: Box<dyn Fn(Input) -> ResultPoint<Point> + Send + Sync>,

        /// Parameters defining the domain for sampling the parametric function
        params: ConstructionParams,
    },
}
