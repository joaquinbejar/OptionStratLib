/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 26/8/24
******************************************************************************/
use crate::curves::Point2D;
use crate::error::curves::CurvesError;
use rust_decimal::Decimal;
use std::collections::BTreeSet;

pub enum CurveConstructionMethod {
    FromData {
        points: BTreeSet<Point2D>,
    },
    Parametric {
        f: Box<dyn Fn(Decimal) -> Result<Point2D, CurvesError> + Send + Sync>,
        t_start: Decimal,
        t_end: Decimal,
        steps: usize,
    },
}
