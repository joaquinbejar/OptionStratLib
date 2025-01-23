/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 26/8/24
******************************************************************************/
use crate::geometrics::{BasicMetrics, ShapeMetrics};

pub struct AnalysisResult {
    pub statistics: BasicMetrics,
    pub shape_metrics: ShapeMetrics,
}
