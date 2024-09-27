/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 27/9/24
******************************************************************************/
use crate::constants::TOLERANCE;

#[allow(dead_code)]
pub(crate) fn approx_equal(a: f64, b: f64) -> bool {
    (a - b).abs() < TOLERANCE
}
