/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 20/8/24
******************************************************************************/

pub trait Graph {
    fn graph(&self, data: &[f64], file_path: &str) -> Result<(), Box<dyn std::error::Error>>;

    fn title(&self) -> String;
}
