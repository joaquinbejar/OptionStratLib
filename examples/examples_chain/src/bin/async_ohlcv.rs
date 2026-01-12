use optionstratlib::utils::read_ohlcv_from_zip_async;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("--- Async OHLCV Reading ---");

    let zip_path = "../../examples/Data/cl-1m-sample.zip"; // Path relative to the example run dir

    println!("Reading OHLCV data from {} asynchronously...", zip_path);

    // We'll read without date filters first
    let result: Result<Vec<optionstratlib::utils::OhlcvCandle>, optionstratlib::error::OhlcvError> =
        read_ohlcv_from_zip_async(zip_path.to_string(), None, None).await;

    match result {
        Ok(candles) => {
            println!("Successfully read {} candles.", candles.len());
            if let Some(first) = candles.first() {
                println!("First candle: Date={}, Close={}", first.date, first.close);
            }
        }
        Err(e) => {
            eprintln!("Error reading OHLCV: {}", e);
            // Don't fail the example if file is not found, just report it
        }
    }

    println!("--- Async OHLCV Reading Completed ---");
    Ok(())
}
