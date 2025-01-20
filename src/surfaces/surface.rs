/******************************************************************************
    Author: Joaquín Béjar García
    Email: jb@taunais.com 
    Date: 20/1/25
 ******************************************************************************/
use std::collections::BTreeSet;
use rayon::iter::IntoParallelIterator;
use rust_decimal::Decimal;
use crate::curves::{Curve, Point2D};
use crate::error::{CurvesError, SurfaceError};
use crate::surfaces::construction::SurfaceConstructionMethod;
use crate::surfaces::Point3D;
use crate::surfaces::types::Axis;

/// Represents a mathematical surface in 3D space
#[derive(Debug, Clone)]
pub struct Surface {
    /// Collection of 3D points defining the surface
    pub points: BTreeSet<Point3D>,
    pub x_range: (Decimal, Decimal),
    pub y_range: (Decimal, Decimal),

}

impl Surface {
    
    pub fn new(points: BTreeSet<Point3D>) -> Self {
        let x_range = Self::calculate_range(points.iter().map(|p| p.x));
        let y_range = Self::calculate_range(points.iter().map(|p| p.y));
        Self {
            points,
            x_range,
            y_range,
        }
    }

    pub fn from_vector(points: Vec<Point3D>) -> Self {
        let x_range = Self::calculate_range(points.iter().map(|p| p.x));
        let y_range = Self::calculate_range(points.iter().map(|p| p.y));
        let points = points.into_iter().collect();
        Surface { points, x_range , y_range }
    }

    pub fn calculate_range<I>(iter: I) -> (Decimal, Decimal)
    where
        I: Iterator<Item = Decimal>,
    {
        iter.fold((Decimal::MAX, Decimal::MIN), |(min, max), val| {
            (min.min(val), max.max(val))
        })
    }

    pub fn construct(method: SurfaceConstructionMethod) -> Result<Self, SurfaceError> {
        match method {
            SurfaceConstructionMethod::FromData { points } => {
                if points.is_empty() {
                    return Err(SurfaceError::Point3DError {
                        reason: "Empty points array",
                    });
                }
                Ok(Surface::new(points))
            }

            SurfaceConstructionMethod::Parametric {
                f,
                x_start,
                x_end,
                y_start,
                y_end,
                x_steps,
                y_steps,
            } => {
                let step_size = (x_end - x_start) / Decimal::from(x_steps);
                let points: Result<BTreeSet<Point3D>, SurfaceError> = (0..=x_steps)
                    .into_par_iter()
                    .map(|i| {
                        let t = x_start + step_size * Decimal::from(i);
                        f(t).map_err(|e| CurvesError::ConstructionError(e.to_string()))
                    })
                    .collect();

                points.map(Surface::new)
            }
        }
    }

    pub fn vector(&self) -> Vec<&Point3D> {
        self.points.iter().collect()
    }
    
    pub fn get_curve(&self, axis: Axis) -> Curve {
        let points = self.points.iter().map(|p| {
            match axis {
                Axis::X => Point2D::new(p.y, p.z),
                Axis::Y => Point2D::new(p.x, p.z),
                Axis::Z => Point2D::new(p.x, p.y),
            }
        }).collect();
        
        Curve::new(points)
    }

}