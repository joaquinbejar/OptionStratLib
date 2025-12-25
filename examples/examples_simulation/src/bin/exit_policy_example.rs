//! Example demonstrating the use of ExitPolicy enum.
//!
//! This example shows how to create and use different exit policies
//! for option trading strategies.

use optionstratlib::prelude::*;

fn main() {
    setup_logger();
    info!("\n========== EXIT POLICY EXAMPLES ==========\n");

    // Simple profit target
    let profit_target = ExitPolicy::profit_target(dec!(0.5));
    info!("1. Simple Profit Target:");
    info!("   {}", profit_target);
    info!("   Composite: {}", profit_target.is_composite());
    info!("   Conditions: {}\n", profit_target.condition_count());

    // Simple stop loss
    let stop_loss = ExitPolicy::stop_loss(dec!(1.0));
    info!("2. Simple Stop Loss:");
    info!("   {}", stop_loss);
    info!("   Composite: {}", stop_loss.is_composite());
    info!("   Conditions: {}\n", stop_loss.condition_count());

    // Combined profit or loss
    let profit_or_loss = ExitPolicy::profit_or_loss(dec!(0.5), dec!(1.0));
    info!("3. Profit Target OR Stop Loss:");
    info!("   {}", profit_or_loss);
    info!("   Composite: {}", profit_or_loss.is_composite());
    info!("   Conditions: {}\n", profit_or_loss.condition_count());

    // Time-limited profit target
    let profit_or_time = ExitPolicy::profit_or_time(dec!(0.5), 5000);
    info!("4. Profit Target OR Time Limit:");
    info!("   {}", profit_or_time);
    info!("   Composite: {}", profit_or_time.is_composite());
    info!("   Conditions: {}\n", profit_or_time.condition_count());

    // Fixed price exit
    let fixed_price = ExitPolicy::FixedPrice(pos_or_panic!(50.0));
    info!("5. Fixed Price Exit:");
    info!("   {}", fixed_price);
    info!("   Composite: {}", fixed_price.is_composite());
    info!("   Conditions: {}\n", fixed_price.condition_count());

    // Price range exit
    let price_range = ExitPolicy::Or(vec![
        ExitPolicy::MinPrice(pos_or_panic!(5.0)),
        ExitPolicy::MaxPrice(pos_or_panic!(100.0)),
    ]);
    info!("6. Price Range Exit (Min OR Max):");
    info!("   {}", price_range);
    info!("   Composite: {}", price_range.is_composite());
    info!("   Conditions: {}\n", price_range.condition_count());

    // Delta-based exit
    let delta_exit = ExitPolicy::DeltaThreshold(dec!(0.7));
    info!("7. Delta Threshold Exit:");
    info!("   {}", delta_exit);
    info!("   Composite: {}", delta_exit.is_composite());
    info!("   Conditions: {}\n", delta_exit.condition_count());

    // Underlying price exits
    let underlying_exits = ExitPolicy::Or(vec![
        ExitPolicy::UnderlyingBelow(pos_or_panic!(3900.0)),
        ExitPolicy::UnderlyingAbove(pos_or_panic!(4100.0)),
    ]);
    info!("8. Underlying Price Range Exit:");
    info!("   {}", underlying_exits);
    info!("   Composite: {}", underlying_exits.is_composite());
    info!("   Conditions: {}\n", underlying_exits.condition_count());

    // Complex nested policy
    let complex_policy = ExitPolicy::Or(vec![
        ExitPolicy::And(vec![
            ExitPolicy::ProfitPercent(dec!(0.3)),
            ExitPolicy::TimeSteps(2000),
        ]),
        ExitPolicy::LossPercent(dec!(1.0)),
        ExitPolicy::DaysToExpiration(pos_or_panic!(1.0)),
    ]);
    info!("9. Complex Nested Policy:");
    info!("   {}", complex_policy);
    info!("   Composite: {}", complex_policy.is_composite());
    info!("   Conditions: {}\n", complex_policy.condition_count());

    // Hold to expiration
    let hold_to_exp = ExitPolicy::Expiration;
    info!("10. Hold to Expiration:");
    info!("    {}", hold_to_exp);
    info!("    Composite: {}", hold_to_exp.is_composite());
    info!("    Conditions: {}\n", hold_to_exp.condition_count());

    info!("==========================================\n");

    // Demonstrate serialization
    info!("Serialization Example:");
    let policy = ExitPolicy::profit_or_loss(dec!(0.5), dec!(1.0));
    let json = serde_json::to_string_pretty(&policy).unwrap();
    info!("{}\n", json);

    info!("==========================================\n");
}
