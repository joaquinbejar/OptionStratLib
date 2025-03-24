/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 22/2/25
******************************************************************************/

/*
/// Implementation of the `Surfacable` trait for the `Simulator` type.
///
/// This implementation allows the `Simulator` object to generate a [`Surface`]
/// representation based on the random walks it manages.
///
/// # Process
/// The `surface` method generates a surface by:
/// 1. Iterating over all random walks in the simulator.
/// 2. Extracting their associated `Curve` points via each walk's `curve()` method.
/// 3. Mapping these points into a collection of three-dimensional points, [`Point3D`],
///    where each point's `z` coordinate is represented by the index of the walk,
///    and its `x` and `y` coordinates are taken from the walk's `Curve` data.
/// 4. Constructing a [`Surface`] using the resulting set of [`Point3D`] points.
///
/// # Returns
/// - **`Ok(Surface)`**: If the surface is successfully created from the `Point3D` points.
/// - **`Err(SurfaceError)`**: If any errors occur during surface construction. Possible
///   errors include:
///   - Invalid point conversion via [`Point3D::from_tuple`] (e.g., invalid coordinate transformations).
///   - Failures in the associated random walks' `curve()` methods.
///   - Issues during the collection or instantiation of the final surface.
///
/// # Notes
/// - The resulting surface's points are stored in a `BTreeSet`, which inherently ensures
///   that the points are sorted and unique. This provides a natural order and prevents
///   duplicate points from being included.
/// - The `z` coordinate of each `Point3D` is determined by the index of the walk in the simulator.
/// - This implementation heavily relies on the [`Surface::new`] and [`Point3D::from_tuple`]
///   helper methods.
///
/// # Implementation Details
/// - `Simulator` maintains its walks in a `HashMap`. The `surface` method iterates through the
///   walks using the `enumerate()` function, which provides a unique index for each walk.
/// - The method uses the `flat_map()` iterator to efficiently transform the collection of walks
///   into the desired set of points.
///
/// # Errors
/// The method returns a [`SurfaceError`] in any of the following cases:
/// - If the `curve()` method of a random walk fails (e.g., invalid curve generation or
///   missing values).
/// - If a conversion error occurs while creating `Point3D` instances (e.g., invalid
///   input arguments).
/// - If issues occur while constructing the `Surface` itself.
///
/// # Example
/// This implementation allows the `Simulator` to generate a 3D surface representation of
/// random walks, which can subsequently be visualized, analyzed, or processed.
///
/// # See Also
/// - [`Surface`]: The resulting 3D surface representation.
/// - [`Point3D`]: Used to represent points in 3D space in the generated surface.
/// - [`SurfaceError`]: Enumerates possible error types during surface generation.
// impl Surfacable for Simulator {
//     fn surface(&self) -> Result<Surface, SurfaceError> {
//         let points: BTreeSet<Point3D> = self
//             .walks
//             .iter()
//             .enumerate()
//             .flat_map(|(i, (_, walk))| {
//                 let curve = walk.curve().unwrap();
//                 let points2d = curve.points;
//
//                 points2d
//                     .into_iter()
//                     .map(move |point| Point3D::from_tuple(i, point.x, point.y).unwrap())
//             })
//             .collect();
//
//         Ok(Surface::new(points))
//     }
// }
*/
