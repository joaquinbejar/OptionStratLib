/******************************************************************************
    Author: Joaquín Béjar García
    Email: jb@taunais.com 
    Date: 20/1/25
 ******************************************************************************/
use rust_decimal::Decimal;
use crate::geometrics::GeometricObject;
use crate::surfaces::{Point3D, Surface};


/// Creates a planar surface defined by starting points and a normal vector.
///
/// This function generates a 3-dimensional planar surface by calculating evenly spaced points
/// in a grid pattern. The surface is defined by x and y ranges, with the z-coordinate 
/// calculated based on the plane equation:
///
/// ```text
/// ax + by + cz + d = 0
/// ```
/// where (a,b,c) is the normal vector of the plane.
///
/// # Parameters
/// - `x_start`: Starting x-coordinate (as a `Decimal`)
/// - `x_end`: Ending x-coordinate (as a `Decimal`)
/// - `y_start`: Starting y-coordinate (as a `Decimal`) 
/// - `y_end`: Ending y-coordinate (as a `Decimal`)
/// - `normal`: Tuple (a,b,c) representing the normal vector of the plane
/// - `d`: The d-coefficient in the plane equation
///
/// # Returns
/// A `Surface` instance containing a grid of points representing the planar surface.
///
/// # Behavior
/// - Creates a 10x10 grid of points by default
/// - Points are evenly spaced in both x and y directions
/// - Z-coordinate is calculated using the plane equation
pub fn create_planar_surface(
    x_start: Decimal,
    x_end: Decimal,
    y_start: Decimal,
    y_end: Decimal,
    normal: (Decimal, Decimal, Decimal),
    d: Decimal,
) -> Surface {
    let x_steps = 10;
    let y_steps = 10;
    let x_step_size = (x_end - x_start) / Decimal::from(x_steps);
    let y_step_size = (y_end - y_start) / Decimal::from(y_steps);

    let (a, b, c) = normal;
    let mut points = Vec::new();

    for i in 0..=x_steps {
        for j in 0..=y_steps {
            let x = x_start + x_step_size * Decimal::from(i);
            let y = y_start + y_step_size * Decimal::from(j);
            // Using plane equation ax + by + cz + d = 0
            // Therefore z = -(ax + by + d)/c
            let z = -(a * x + b * y + d) / c;
            points.push(Point3D::new(x, y, z));
        }
    }

    Surface::from_vector(points.iter().collect())
}

/// Creates a constant height surface where z is constant for all x,y points.
///
/// This function generates a 3-dimensional surface that maintains a constant height (z-value)
/// across all points in the x-y plane, effectively creating a horizontal plane.
///
/// # Parameters
/// - `x_start`: Starting x-coordinate (as a `Decimal`)
/// - `x_end`: Ending x-coordinate (as a `Decimal`) 
/// - `y_start`: Starting y-coordinate (as a `Decimal`)
/// - `y_end`: Ending y-coordinate (as a `Decimal`)
/// - `height`: The constant z-value for all points (as a `Decimal`)
///
/// # Returns
/// A `Surface` instance representing a horizontal plane at the specified height.
///
/// # Behavior
/// - Creates a 10x10 grid of points by default
/// - Points are evenly spaced in both x and y directions
/// - All points have the same z-coordinate equal to height
pub fn create_constant_surface(
    x_start: Decimal,
    x_end: Decimal,
    y_start: Decimal,
    y_end: Decimal,
    height: Decimal,
) -> Surface {
    let x_steps = 10;
    let y_steps = 10;
    let x_step_size = (x_end - x_start) / Decimal::from(x_steps);
    let y_step_size = (y_end - y_start) / Decimal::from(y_steps);

    let mut points = Vec::new();

    for i in 0..=x_steps {
        for j in 0..=y_steps {
            let x = x_start + x_step_size * Decimal::from(i);
            let y = y_start + y_step_size * Decimal::from(j);
            points.push(Point3D::new(x, y, height));
        }
    }

    Surface::from_vector(points.iter().collect())
}

/// Creates a paraboloid surface.
///
/// This function generates a 3-dimensional paraboloid surface of the form:
/// ```text
/// z = ax² + by²
/// ```
/// where a and b are coefficients that control the shape of the paraboloid.
///
/// # Parameters
/// - `x_start`: Starting x-coordinate (as a `Decimal`)
/// - `x_end`: Ending x-coordinate (as a `Decimal`)
/// - `y_start`: Starting y-coordinate (as a `Decimal`)
/// - `y_end`: Ending y-coordinate (as a `Decimal`)
/// - `a`: Coefficient for x² term
/// - `b`: Coefficient for y² term
///
/// # Returns
/// A `Surface` instance representing a paraboloid surface.
///
/// # Behavior
/// - Creates a 10x10 grid of points by default
/// - Points are evenly spaced in both x and y directions
/// - Z-coordinate is calculated using the paraboloid equation
pub fn create_paraboloid_surface(
    x_start: Decimal,
    x_end: Decimal,
    y_start: Decimal,
    y_end: Decimal,
    a: Decimal,
    b: Decimal,
) -> Surface {
    let x_steps = 10;
    let y_steps = 10;
    let x_step_size = (x_end - x_start) / Decimal::from(x_steps);
    let y_step_size = (y_end - y_start) / Decimal::from(y_steps);

    let mut points = Vec::new();

    for i in 0..=x_steps {
        for j in 0..=y_steps {
            let x = x_start + x_step_size * Decimal::from(i);
            let y = y_start + y_step_size * Decimal::from(j);
            let z = a * x * x + b * y * y;
            points.push(Point3D::new(x, y, z));
        }
    }

    Surface::new(points.into_iter().collect())
}


#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_create_constant_surface() {
        let surface = create_constant_surface(
            dec!(0),
            dec!(10),
            dec!(0),
            dec!(10),
            dec!(5),
        );

        // Check if we have the correct number of points (11x11 grid)
        assert_eq!(surface.points.len(), 121);

        // Check if all points have z = 5
        assert!(surface.points.iter().all(|p| p.z == dec!(5)));

        // Check extremes of x and y coordinates
        let x_coords: Vec<_> = surface.points.iter().map(|p| p.x).collect();
        let y_coords: Vec<_> = surface.points.iter().map(|p| p.y).collect();

        assert!(x_coords.contains(&dec!(0)));
        assert!(x_coords.contains(&dec!(10)));
        assert!(y_coords.contains(&dec!(0)));
        assert!(y_coords.contains(&dec!(10)));
    }

    #[test]
    fn test_create_planar_surface() {
        let surface = create_planar_surface(
            dec!(0),
            dec!(10),
            dec!(0),
            dec!(10),
            (dec!(0), dec!(0), dec!(1)), // normal vector pointing up
            dec!(5), // d coefficient
        );

        // Check if we have the correct number of points
        assert_eq!(surface.points.len(), 121);

        // For this configuration (normal = (0,0,1)), all points should have z = -5
        assert!(surface.points.iter().all(|p| p.z == dec!(-5)));

        // Check extremes of x and y coordinates
        let x_coords: Vec<_> = surface.points.iter().map(|p| p.x).collect();
        let y_coords: Vec<_> = surface.points.iter().map(|p| p.y).collect();

        assert!(x_coords.contains(&dec!(0)));
        assert!(x_coords.contains(&dec!(10)));
        assert!(y_coords.contains(&dec!(0)));
        assert!(y_coords.contains(&dec!(10)));
    }

    #[test]
    fn test_create_paraboloid_surface() {
        let surface = create_paraboloid_surface(
            dec!(-1),
            dec!(1),
            dec!(-1),
            dec!(1),
            dec!(1), // a coefficient
            dec!(1), // b coefficient
        );

        // Check if we have the correct number of points
        assert_eq!(surface.points.len(), 121);

        // Check some key points:
        // At origin (0,0), z should be 0
        let origin = surface.points.iter()
            .find(|p| p.x == dec!(0) && p.y == dec!(0))
            .unwrap();
        assert_eq!(origin.z, dec!(0));

        // At (1,1), z should be 2 (1² + 1²)
        let corner = surface.points.iter()
            .find(|p| p.x == dec!(1) && p.y == dec!(1))
            .unwrap();
        assert_eq!(corner.z, dec!(2));

        // At (-1,-1), z should also be 2 ((-1)² + (-1)²)
        let opposite_corner = surface.points.iter()
            .find(|p| p.x == dec!(-1) && p.y == dec!(-1))
            .unwrap();
        assert_eq!(opposite_corner.z, dec!(2));
    }
}