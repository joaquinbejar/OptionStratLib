/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 26/8/24
******************************************************************************/

#[allow(dead_code)]
pub struct CurveAnalysisResult {
    pub mean: f64,
    pub median: f64,
    pub std_dev: f64,
    pub skew: f64,
    pub kurtosis: f64,
}
