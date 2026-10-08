//! Test-only support: a small priced option chain the optimiser unit tests
//! search. It is compiled only for this crate's own unit tests and is not
//! part of the public API (#814).

use optionstratlib_core::model::{ExpirationDate, Positive};
use optionstratlib_core::pos_or_panic;
use optionstratlib_market::chains::OptionChain;
use optionstratlib_market::chains::utils::{OptionChainBuildParams, OptionDataPriceParams};
use rust_decimal_macros::dec;

/// Thirteen strikes, 85 to 115 every 2.5, on a spot of 100 thirty days
/// from expiry, priced by Black-Scholes at a 20% volatility with a mild
/// skew. Call quotes fall and put quotes rise with the strike, so every
/// searching strategy finds valid candidates in it, the 95 to 105 range
/// included.
pub(crate) fn priced_chain() -> OptionChain {
    let params = OptionChainBuildParams::new(
        "TEST".to_string(),
        Some(Positive::HUNDRED),
        6,
        Some(pos_or_panic!(2.5)),
        dec!(-0.2),
        dec!(0.1),
        pos_or_panic!(0.02),
        2,
        OptionDataPriceParams::new(
            Some(Box::new(Positive::HUNDRED)),
            Some(ExpirationDate::Days(pos_or_panic!(30.0))),
            Some(dec!(0.05)),
            Some(Positive::ZERO),
            Some("TEST".to_string()),
        ),
        pos_or_panic!(0.2),
    );
    OptionChain::build_chain(&params).unwrap_or_else(|e| panic!("the priced chain builds: {e}"))
}
