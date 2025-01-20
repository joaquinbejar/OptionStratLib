/******************************************************************************
    Author: Joaquín Béjar García
    Email: jb@taunais.com 
    Date: 20/1/25
 ******************************************************************************/
use std::collections::BTreeSet;
use rust_decimal::Decimal;
use crate::error::SurfaceError;
use crate::surfaces::Point3D;

#[derive(Debug, Clone)]
pub enum SurfaceConstructionMethod {
    FromData {
        points: BTreeSet<Point3D>,
    },
    Parametric {
        f: Box<dyn Fn(Decimal) -> Result<Point3D, SurfaceError> + Send + Sync>,
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
