/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 26/2/25
******************************************************************************/
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PnLRange {
    /// Lower bound of this PnL bucket (inclusive)
    pub lower: i32,

    /// Upper bound of this PnL bucket (exclusive)
    pub upper: i32,
}
