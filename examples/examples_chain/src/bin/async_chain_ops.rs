use optionstratlib::prelude::*;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("--- Async Option Chain Operations ---");

    // 1. Create a dummy chain
    let chain = OptionChain::new(
        "AAPL",
        pos_or_panic!(150.0),
        "2024-12-20".to_string(),
        None,
        None,
    );

    // 2. Save it asynchronously to JSON
    let json_path = "async_chain.json";
    println!("Saving chain to {} asynchronously...", json_path);
    chain.save_to_json_async(json_path).await?;
    println!("Successfully saved to JSON.");

    // 3. Load it asynchronously from JSON
    println!("Loading chain from {} asynchronously...", json_path);
    let loaded_json = OptionChain::load_from_json_async(json_path).await?;
    println!(
        "Successfully loaded from JSON. Symbol: {}",
        loaded_json.symbol
    );

    // 4. Save it asynchronously to CSV
    let csv_path = "async_chain.csv";
    println!("Saving chain to {} asynchronously...", csv_path);
    chain.save_to_csv_async(csv_path).await?;
    println!("Successfully saved to CSV.");

    // 5. Load it asynchronously from CSV
    println!("Loading chain from {} asynchronously...", csv_path);
    let loaded_csv = OptionChain::load_from_csv_async(csv_path).await?;
    println!(
        "Successfully loaded from CSV. Symbol: {}",
        loaded_csv.symbol
    );

    // Clean up
    std::fs::remove_file(json_path)?;
    std::fs::remove_file(csv_path)?;

    println!("--- Async Chain Operations Completed ---");
    Ok(())
}
