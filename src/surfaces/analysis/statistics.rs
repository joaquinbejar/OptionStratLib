
/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 20/1/25
******************************************************************************/

use crate::surfaces::analysis::metrics::{BasicMetrics, ShapeMetrics};

pub struct SurfaceAnalysisResult {
    pub statistics: BasicMetrics,
    pub shape_metrics: ShapeMetrics,
}