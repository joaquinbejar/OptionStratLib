/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 26/9/24
******************************************************************************/
use optionstratlib::model::chain::OptionChain;
use optionstratlib::model::types::PositiveF64;
use optionstratlib::pos;
use optionstratlib::utils::logger::setup_logger;
use tracing::info;

fn main() {
    setup_logger();
    let mut chain = OptionChain::new("SP500", pos!(5781.88), "18 oct 2024".to_string());

    chain.add_option(pos!(5520.0), 274.26, 276.06, 13.22, 14.90, 16.31);
    chain.add_option(pos!(5525.0), 269.62, 271.42, 13.54, 15.27, 16.205);
    chain.add_option(pos!(5530.0), 265.00, 266.80, 13.88, 15.65, 16.1);
    chain.add_option(pos!(5540.0), 255.78, 257.58, 14.62, 16.42, 15.89);
    chain.add_option(pos!(5550.0), 246.61, 248.41, 15.42, 17.22, 15.68);
    chain.add_option(pos!(5560.0), 237.49, 239.29, 15.96, 18.07, 15.47);
    chain.add_option(pos!(5570.0), 228.42, 230.22, 17.18, 18.98, 15.26);
    chain.add_option(pos!(5575.0), 223.91, 225.71, 17.65, 19.45, 15.155);
    chain.add_option(pos!(5580.0), 219.42, 221.22, 18.14, 19.94, 15.05);
    chain.add_option(pos!(5590.0), 210.48, 212.28, 18.07, 20.23, 14.84);
    chain.add_option(pos!(5600.0), 201.60, 203.40, 19.17, 20.97, 14.63);
    chain.add_option(pos!(5610.0), 192.80, 194.60, 21.43, 23.23, 14.42);
    chain.add_option(pos!(5620.0), 184.08, 185.88, 22.68, 24.48, 14.21);
    chain.add_option(pos!(5625.0), 179.75, 181.55, 23.33, 25.13, 14.105);
    chain.add_option(pos!(5630.0), 175.44, 177.24, 24.01, 25.81, 14.0);
    chain.add_option(pos!(5640.0), 167.02, 168.82, 25.56, 27.36, 13.82);
    chain.add_option(pos!(5650.0), 158.71, 160.51, 27.22, 29.02, 13.64);
    chain.add_option(pos!(5660.0), 150.51, 152.31, 28.99, 30.79, 13.46);
    chain.add_option(pos!(5670.0), 142.43, 144.23, 30.89, 32.69, 13.28);
    chain.add_option(pos!(5675.0), 138.44, 140.24, 31.88, 33.68, 13.19);
    chain.add_option(pos!(5680.0), 134.48, 136.28, 32.91, 34.71, 13.1);
    chain.add_option(pos!(5690.0), 126.67, 128.47, 35.07, 36.87, 12.92);
    chain.add_option(pos!(5700.0), 119.01, 120.81, 37.37, 39.17, 12.74);
    chain.add_option(pos!(5710.0), 111.50, 113.30, 39.83, 41.63, 12.56);
    chain.add_option(pos!(5720.0), 104.16, 105.96, 42.46, 44.26, 12.38);
    chain.add_option(pos!(5730.0), 96.99, 98.79, 45.27, 47.07, 12.2);
    chain.add_option(pos!(5740.0), 90.02, 91.82, 48.26, 50.06, 12.02);
    chain.add_option(pos!(5750.0), 83.24, 85.04, 51.45, 53.25, 11.84);
    chain.add_option(pos!(5760.0), 76.67, 78.47, 54.85, 56.65, 11.66);
    chain.add_option(pos!(5770.0), 70.32, 72.12, 58.47, 60.27, 11.48);
    chain.add_option(pos!(5780.0), 64.20, 66.00, 62.32, 64.12, 11.3);
    chain.add_option(pos!(5790.0), 58.50, 60.30, 66.59, 68.39, 11.15);
    chain.add_option(pos!(5800.0), 53.04, 54.84, 71.11, 72.91, 11.0);
    chain.add_option(pos!(5810.0), 47.85, 49.65, 75.89, 77.69, 10.85);
    chain.add_option(pos!(5820.0), 42.93, 44.73, 80.94, 82.74, 10.7);
    chain.add_option(pos!(5830.0), 38.29, 40.09, 86.27, 88.07, 10.55);
    chain.add_option(pos!(5840.0), 33.93, 35.73, 91.87, 93.67, 10.4);
    chain.add_option(pos!(5850.0), 29.85, 31.65, 97.77, 99.57, 10.25);
    chain.add_option(pos!(5900.0), 16.05, 17.85, 133.82, 135.62, 10.058);
    chain.add_option(pos!(5950.0), 8.05, 9.08, 175.28, 177.08, 9.928);
    chain.add_option(pos!(6000.0), 3.38, 4.38, 220.45, 222.25, 9.798);
    chain.add_option(pos!(6050.0), 1.06, 2.06, 267.98, 269.78, 9.668);
    chain.add_option(pos!(6100.0), 0.05, 1.05, 316.82, 318.62, 9.538);
    chain.add_option(pos!(6150.0), 0.00, 1.00, 366.30, 368.10, 9.46);
    chain.add_option(pos!(6200.0), 0.00, 1.00, 416.03, 417.83, 9.46);

    info!("\n{}", chain);

    chain.save_to_csv("./examples/Chains").unwrap();
    chain.save_to_json("./examples/Chains").unwrap();
    let _ = OptionChain::load_from_csv("./examples/Chains/SP500-18-oct-2024-5781.88.csv").unwrap();
    let _ =
        OptionChain::load_from_json("./examples/Chains/SP500-18-oct-2024-5781.88.json").unwrap();
}
