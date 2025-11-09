//! Example demonstrating the use of ExitPolicy enum.
//!
//! This example shows how to create and use different exit policies
//! for option trading strategies.

use optionstratlib::pos;
use optionstratlib::simulation::ExitPolicy;
use rust_decimal_macros::dec;

fn main() {
    println!("\n========== EXIT POLICY EXAMPLES ==========\n");

    // Simple profit target
    let profit_target = ExitPolicy::profit_target(dec!(0.5));
    println!("1. Simple Profit Target:");
    println!("   {}", profit_target);
    println!("   Composite: {}", profit_target.is_composite());
    println!("   Conditions: {}\n", profit_target.condition_count());

    // Simple stop loss
    let stop_loss = ExitPolicy::stop_loss(dec!(1.0));
    println!("2. Simple Stop Loss:");
    println!("   {}", stop_loss);
    println!("   Composite: {}", stop_loss.is_composite());
    println!("   Conditions: {}\n", stop_loss.condition_count());

    // Combined profit or loss
    let profit_or_loss = ExitPolicy::profit_or_loss(dec!(0.5), dec!(1.0));
    println!("3. Profit Target OR Stop Loss:");
    println!("   {}", profit_or_loss);
    println!("   Composite: {}", profit_or_loss.is_composite());
    println!("   Conditions: {}\n", profit_or_loss.condition_count());

    // Time-limited profit target
    let profit_or_time = ExitPolicy::profit_or_time(dec!(0.5), 5000);
    println!("4. Profit Target OR Time Limit:");
    println!("   {}", profit_or_time);
    println!("   Composite: {}", profit_or_time.is_composite());
    println!("   Conditions: {}\n", profit_or_time.condition_count());

    // Fixed price exit
    let fixed_price = ExitPolicy::FixedPrice(pos!(50.0));
    println!("5. Fixed Price Exit:");
    println!("   {}", fixed_price);
    println!("   Composite: {}", fixed_price.is_composite());
    println!("   Conditions: {}\n", fixed_price.condition_count());

    // Price range exit
    let price_range = ExitPolicy::Or(vec![
        ExitPolicy::MinPrice(pos!(5.0)),
        ExitPolicy::MaxPrice(pos!(100.0)),
    ]);
    println!("6. Price Range Exit (Min OR Max):");
    println!("   {}", price_range);
    println!("   Composite: {}", price_range.is_composite());
    println!("   Conditions: {}\n", price_range.condition_count());

    // Delta-based exit
    let delta_exit = ExitPolicy::DeltaThreshold(dec!(0.7));
    println!("7. Delta Threshold Exit:");
    println!("   {}", delta_exit);
    println!("   Composite: {}", delta_exit.is_composite());
    println!("   Conditions: {}\n", delta_exit.condition_count());

    // Underlying price exits
    let underlying_exits = ExitPolicy::Or(vec![
        ExitPolicy::UnderlyingBelow(pos!(3900.0)),
        ExitPolicy::UnderlyingAbove(pos!(4100.0)),
    ]);
    println!("8. Underlying Price Range Exit:");
    println!("   {}", underlying_exits);
    println!("   Composite: {}", underlying_exits.is_composite());
    println!("   Conditions: {}\n", underlying_exits.condition_count());

    // Complex nested policy
    let complex_policy = ExitPolicy::Or(vec![
        ExitPolicy::And(vec![
            ExitPolicy::ProfitPercent(dec!(0.3)),
            ExitPolicy::TimeSteps(2000),
        ]),
        ExitPolicy::LossPercent(dec!(1.0)),
        ExitPolicy::DaysToExpiration(pos!(1.0)),
    ]);
    println!("9. Complex Nested Policy:");
    println!("   {}", complex_policy);
    println!("   Composite: {}", complex_policy.is_composite());
    println!("   Conditions: {}\n", complex_policy.condition_count());

    // Hold to expiration
    let hold_to_exp = ExitPolicy::Expiration;
    println!("10. Hold to Expiration:");
    println!("    {}", hold_to_exp);
    println!("    Composite: {}", hold_to_exp.is_composite());
    println!("    Conditions: {}\n", hold_to_exp.condition_count());

    println!("==========================================\n");

    // Demonstrate serialization
    println!("Serialization Example:");
    let policy = ExitPolicy::profit_or_loss(dec!(0.5), dec!(1.0));
    let json = serde_json::to_string_pretty(&policy).unwrap();
    println!("{}\n", json);

    println!("==========================================\n");
}
