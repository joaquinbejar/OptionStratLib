/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 26/8/24
******************************************************************************/

use crate::curves::analysis::metrics::{BasicMetrics, ShapeMetrics};

pub struct CurveAnalysisResult {
    pub statistics: BasicMetrics,
    pub shape_metrics: ShapeMetrics,
}
