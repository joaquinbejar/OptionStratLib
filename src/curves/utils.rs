/******************************************************************************
    Author: Joaquín Béjar García
    Email: jb@taunais.com 
    Date: 9/1/25
 ******************************************************************************/


use crate::curves::{Curve, Point2D};
use rust_decimal::Decimal;

/// Create test utility functions for curve generation
pub(crate) fn create_linear_curve(start: Decimal, end: Decimal, slope: Decimal) -> Curve {
    let steps = 10;
    let step_size = (end - start) / Decimal::from(steps);

    let points: Vec<Point2D> = (0..=steps)
        .map(|i| {
            let x = start + step_size * Decimal::from(i);
            let y = slope * x;
            Point2D::new(x, y)
        })
        .collect();

    Curve::new(points)
}

/// Create test utility functions for curve generation
pub(crate) fn create_constant_curve(start: Decimal, end: Decimal, value: Decimal) -> Curve {
    let steps = 10;
    let step_size = (end - start) / Decimal::from(steps);

    let points: Vec<Point2D> = (0..=steps)
        .map(|i| {
            let x = start + step_size * Decimal::from(i);
            Point2D::new(x, value)
        })
        .collect();

    Curve::new(points)
}
