/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 26/8/24
******************************************************************************/

#[derive(Debug, Clone, Copy)]
pub enum InterpolationType {
    Linear,
    Bilinear,
    Cubic,
    Spline,
}
