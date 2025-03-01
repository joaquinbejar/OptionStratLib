/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 23/2/25
******************************************************************************/
use crate::error::SurfaceError;
use crate::surfaces::Surface;

/// A trait for objects that can generate a mathematical surface in 3D space.
///
/// This trait defines a single method, [`Surfacable::surface`], which is responsible for
/// calculating or constructing a [`Surface`] representation of the object that implements it.
/// The surface may be created through direct construction or as the result of some computational process.
///
/// # Errors
/// The [`Surfacable::surface`] method returns a [`Result`] containing the generated surface on success.
/// If an error occurs during the surface generation process, a [`SurfaceError`] is returned instead.
/// Potential errors could include:
/// - Invalid inputs or parameters leading to a [`SurfaceError::Point3DError`] or [`SurfaceError::ConstructionError`].
/// - Failures during surface computation due to invalid operations (e.g., [`SurfaceError::OperationError`]).
/// - General-purpose errors, such as I/O or analysis issues, represented as [`SurfaceError::StdError`]
///   or [`SurfaceError::AnalysisError`].
///
/// # Implementors
///  of this trait should define how their specific type generates a [`Surface`].
/// This could involve:
/// - Utilizing existing 3D geometry to build the surface.
/// - Analytical or procedural computations to construct a surface from data.
/// - Interactions with external processes or datasets.
///
/// # See Also
/// For more details on specific error variants, refer to the [`SurfaceError`] enum.
/// For details about the underlying mathematical representation, refer to the [`Surface`] struct.
///
/// # Example Use Cases
/// This trait can be used in scenarios where multiple types need to provide a unified interface
/// for surface generation. For instance, different shapes (spheres, planes, or curves) may implement
/// `Surfacable` so that they can all produce [`Surface`] outputs in a consistent manner.
///
/// # Related Modules
/// - **`crate::surfaces::surface`**: Contains the [`Surface`] structure and its related components.
/// - **`crate::error::surfaces`**: Contains the [`SurfaceError`] type and its variants for error representation.
///
/// # Method
/// - `surface()`:
///   - Returns: `Result<Surface, SurfaceError>`
///   - Description: Generates a surface or returns an error if something goes wrong during the process.
pub trait Surfacable {
    fn surface(&self) -> Result<Surface, SurfaceError>;
}
