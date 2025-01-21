/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 26/8/24
******************************************************************************/

use rust_decimal::Decimal;
use std::collections::BTreeSet;
use std::error::Error;

type ResultPoint<Point> =  Result<Point, Box<dyn Error>>;

#[derive(Debug, Clone)]
pub enum ConstructionParams {
    D2 {
        t_start: Decimal,
        t_end: Decimal,
        steps: usize,
    },
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

pub enum ConstructionMethod<Point, Input> {
    FromData {
        points: BTreeSet<Point>,
    },
    Parametric {
        f: Box<dyn Fn(Input) -> ResultPoint<Point> + Send + Sync>,
        params: ConstructionParams,
    },
}
