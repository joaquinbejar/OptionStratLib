/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 26/9/24
******************************************************************************/
use optionstratlib::chains::chain::OptionChain;
use optionstratlib::utils::logger::setup_logger;
use optionstratlib::Positive;
use optionstratlib::{f2p, spos};
use tracing::info;

fn main() {
    setup_logger();
    let mut chain = OptionChain::new(
        "SP500",
        f2p!(5781.88),
        "18 oct 2024".to_string(),
        None,
        None,
    );

    chain.add_option(
        f2p!(5520.0),
        spos!(274.26),
        spos!(276.06),
        spos!(13.22),
        spos!(14.90),
        spos!(16.31),
        Some(0.5),
        spos!(1.0),
        Some(300),
    );
    chain.add_option(
        f2p!(5525.0),
        spos!(269.62),
        spos!(271.42),
        spos!(13.54),
        spos!(15.27),
        spos!(16.205),
        Some(0.5),
        None,
        None,
    );
    chain.add_option(
        f2p!(5530.0),
        spos!(265.00),
        spos!(266.80),
        spos!(13.88),
        spos!(15.65),
        spos!(16.1),
        Some(0.5),
        None,
        None,
    );
    chain.add_option(
        f2p!(5540.0),
        spos!(255.78),
        spos!(257.58),
        spos!(14.62),
        spos!(16.42),
        spos!(15.89),
        Some(0.5),
        None,
        None,
    );
    chain.add_option(
        f2p!(5550.0),
        spos!(246.61),
        spos!(248.41),
        spos!(15.42),
        spos!(17.22),
        spos!(15.68),
        Some(0.5),
        None,
        None,
    );
    chain.add_option(
        f2p!(5560.0),
        spos!(237.49),
        spos!(239.29),
        spos!(15.96),
        spos!(18.07),
        spos!(15.47),
        Some(0.5),
        None,
        None,
    );
    chain.add_option(
        f2p!(5570.0),
        spos!(228.42),
        spos!(230.22),
        spos!(17.18),
        spos!(18.98),
        spos!(15.26),
        Some(0.5),
        None,
        None,
    );
    chain.add_option(
        f2p!(5575.0),
        spos!(223.91),
        spos!(225.71),
        spos!(17.65),
        spos!(19.45),
        spos!(15.155),
        Some(0.5),
        None,
        None,
    );
    chain.add_option(
        f2p!(5580.0),
        spos!(219.42),
        spos!(221.22),
        spos!(18.14),
        spos!(19.94),
        spos!(15.05),
        Some(0.5),
        None,
        None,
    );
    chain.add_option(
        f2p!(5590.0),
        spos!(210.48),
        spos!(212.28),
        spos!(18.07),
        spos!(20.23),
        spos!(14.84),
        Some(0.5),
        None,
        None,
    );
    chain.add_option(
        f2p!(5600.0),
        spos!(201.60),
        spos!(203.40),
        spos!(19.17),
        spos!(20.97),
        spos!(14.63),
        Some(0.5),
        None,
        None,
    );
    chain.add_option(
        f2p!(5610.0),
        spos!(192.80),
        spos!(194.60),
        spos!(21.43),
        spos!(23.23),
        spos!(14.42),
        Some(0.5),
        None,
        None,
    );
    chain.add_option(
        f2p!(5620.0),
        spos!(184.08),
        spos!(185.88),
        spos!(22.68),
        spos!(24.48),
        spos!(14.21),
        Some(0.5),
        None,
        None,
    );
    chain.add_option(
        f2p!(5625.0),
        spos!(179.75),
        spos!(181.55),
        spos!(23.33),
        spos!(25.13),
        spos!(14.105),
        Some(0.5),
        None,
        None,
    );
    chain.add_option(
        f2p!(5630.0),
        spos!(175.44),
        spos!(177.24),
        spos!(24.01),
        spos!(25.81),
        spos!(14.0),
        Some(0.5),
        None,
        None,
    );
    chain.add_option(
        f2p!(5640.0),
        spos!(167.02),
        spos!(168.82),
        spos!(25.56),
        spos!(27.36),
        spos!(13.82),
        Some(0.5),
        None,
        None,
    );
    chain.add_option(
        f2p!(5650.0),
        spos!(158.71),
        spos!(160.51),
        spos!(27.22),
        spos!(29.02),
        spos!(13.64),
        Some(0.5),
        None,
        None,
    );
    chain.add_option(
        f2p!(5660.0),
        spos!(150.51),
        spos!(152.31),
        spos!(28.99),
        spos!(30.79),
        spos!(13.46),
        Some(0.5),
        None,
        None,
    );
    chain.add_option(
        f2p!(5670.0),
        spos!(142.43),
        spos!(144.23),
        spos!(30.89),
        spos!(32.69),
        spos!(13.28),
        Some(0.5),
        None,
        None,
    );
    chain.add_option(
        f2p!(5675.0),
        spos!(138.44),
        spos!(140.24),
        spos!(31.88),
        spos!(33.68),
        spos!(13.19),
        Some(0.5),
        None,
        None,
    );
    chain.add_option(
        f2p!(5680.0),
        spos!(134.48),
        spos!(136.28),
        spos!(32.91),
        spos!(34.71),
        spos!(13.1),
        Some(0.5),
        None,
        None,
    );
    chain.add_option(
        f2p!(5690.0),
        spos!(126.67),
        spos!(128.47),
        spos!(35.07),
        spos!(36.87),
        spos!(12.92),
        Some(0.5),
        None,
        None,
    );
    chain.add_option(
        f2p!(5700.0),
        spos!(119.01),
        spos!(120.81),
        spos!(37.37),
        spos!(39.17),
        spos!(12.74),
        Some(0.5),
        None,
        None,
    );
    chain.add_option(
        f2p!(5710.0),
        spos!(111.50),
        spos!(113.30),
        spos!(39.83),
        spos!(41.63),
        spos!(12.56),
        Some(0.5),
        None,
        None,
    );
    chain.add_option(
        f2p!(5720.0),
        spos!(104.16),
        spos!(105.96),
        spos!(42.46),
        spos!(44.26),
        spos!(12.38),
        Some(0.5),
        None,
        None,
    );
    chain.add_option(
        f2p!(5730.0),
        spos!(96.99),
        spos!(98.79),
        spos!(45.27),
        spos!(47.07),
        spos!(12.2),
        Some(0.5),
        None,
        None,
    );
    chain.add_option(
        f2p!(5740.0),
        spos!(90.02),
        spos!(91.82),
        spos!(48.26),
        spos!(50.06),
        spos!(12.02),
        Some(0.5),
        None,
        None,
    );
    chain.add_option(
        f2p!(5750.0),
        spos!(83.24),
        spos!(85.04),
        spos!(51.45),
        spos!(53.25),
        spos!(11.84),
        Some(0.5),
        None,
        None,
    );
    chain.add_option(
        f2p!(5760.0),
        spos!(76.67),
        spos!(78.47),
        spos!(54.85),
        spos!(56.65),
        spos!(11.66),
        Some(0.5),
        None,
        None,
    );
    chain.add_option(
        f2p!(5770.0),
        spos!(70.32),
        spos!(72.12),
        spos!(58.47),
        spos!(60.27),
        spos!(11.48),
        Some(0.5),
        None,
        None,
    );
    chain.add_option(
        f2p!(5780.0),
        spos!(64.20),
        spos!(66.00),
        spos!(62.32),
        spos!(64.12),
        spos!(11.3),
        Some(0.5),
        None,
        None,
    );
    chain.add_option(
        f2p!(5790.0),
        spos!(58.50),
        spos!(60.30),
        spos!(66.59),
        spos!(68.39),
        spos!(11.15),
        Some(0.5),
        None,
        None,
    );
    chain.add_option(
        f2p!(5800.0),
        spos!(53.04),
        spos!(54.84),
        spos!(71.11),
        spos!(72.91),
        spos!(11.0),
        Some(0.5),
        None,
        None,
    );
    chain.add_option(
        f2p!(5810.0),
        spos!(47.85),
        spos!(49.65),
        spos!(75.89),
        spos!(77.69),
        spos!(10.85),
        Some(0.5),
        None,
        None,
    );
    chain.add_option(
        f2p!(5820.0),
        spos!(42.93),
        spos!(44.73),
        spos!(80.94),
        spos!(82.74),
        spos!(10.7),
        Some(0.5),
        None,
        None,
    );
    chain.add_option(
        f2p!(5830.0),
        spos!(38.29),
        spos!(40.09),
        spos!(86.27),
        spos!(88.07),
        spos!(10.55),
        Some(0.5),
        None,
        None,
    );
    chain.add_option(
        f2p!(5840.0),
        spos!(33.93),
        spos!(35.73),
        spos!(91.87),
        spos!(93.67),
        spos!(10.4),
        Some(0.5),
        None,
        None,
    );
    chain.add_option(
        f2p!(5850.0),
        spos!(29.85),
        spos!(31.65),
        spos!(97.77),
        spos!(99.57),
        spos!(10.25),
        Some(0.5),
        None,
        None,
    );
    chain.add_option(
        f2p!(5900.0),
        spos!(16.05),
        spos!(17.85),
        spos!(133.82),
        spos!(135.62),
        spos!(10.058),
        Some(0.5),
        None,
        None,
    );
    chain.add_option(
        f2p!(5950.0),
        spos!(8.05),
        spos!(9.08),
        spos!(175.28),
        spos!(177.08),
        spos!(9.928),
        Some(0.5),
        None,
        None,
    );
    chain.add_option(
        f2p!(6000.0),
        spos!(3.38),
        spos!(4.38),
        spos!(220.45),
        spos!(222.25),
        spos!(9.798),
        Some(0.5),
        None,
        None,
    );
    chain.add_option(
        f2p!(6050.0),
        spos!(1.06),
        spos!(2.06),
        spos!(267.98),
        spos!(269.78),
        spos!(9.668),
        Some(0.5),
        None,
        None,
    );
    chain.add_option(
        f2p!(6100.0),
        spos!(0.05),
        spos!(1.05),
        spos!(316.82),
        spos!(318.62),
        spos!(9.538),
        Some(0.5),
        None,
        None,
    );
    chain.add_option(
        f2p!(6150.0),
        spos!(0.00),
        spos!(1.00),
        spos!(366.30),
        spos!(368.10),
        spos!(9.46),
        Some(0.5),
        None,
        None,
    );
    chain.add_option(
        f2p!(6200.0),
        spos!(0.00),
        spos!(1.00),
        spos!(416.03),
        spos!(417.83),
        spos!(9.46),
        Some(0.5),
        None,
        None,
    );

    info!("\n{}", chain);

    chain.save_to_csv("./examples/Chains").unwrap();
    chain.save_to_json("./examples/Chains").unwrap();
    let _ = OptionChain::load_from_csv("./examples/Chains/SP500-18-oct-2024-5781.88.csv").unwrap();
    let _ =
        OptionChain::load_from_json("./examples/Chains/SP500-18-oct-2024-5781.88.json").unwrap();
}
