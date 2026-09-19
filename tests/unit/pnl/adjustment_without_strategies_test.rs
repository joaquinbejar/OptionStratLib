//! The P&L contract consumes `DeltaAdjustment` without importing anything
//! from `strategies` (multi-crate roadmap M1-06, #503).
//!
//! This file imports only `optionstratlib::pnl`, the core model and the
//! pricing error: once `optionstratlib-analytics` is extracted it becomes the
//! analytics-only consumer fixture for adjustment P&L.

use chrono::Utc;
use optionstratlib::error::PricingError;
use optionstratlib::pnl::{DeltaAdjustment, DeltaAdjustmentSameSize, PnL, PnLCalculator};
use optionstratlib::{ExpirationDate, OptionStyle, Side};
use positive::{Positive, pos_or_panic};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

/// A calculator that prices each adjustment as one unit of realized P&L per
/// contract or share, to show the contract is usable without a strategy.
struct FlatAdjustmentPnl;

impl PnLCalculator for FlatAdjustmentPnl {
    fn calculate_pnl(
        &self,
        _underlying_price: &Positive,
        _expiration_date: ExpirationDate,
        _implied_volatility: &Positive,
    ) -> Result<PnL, PricingError> {
        Ok(PnL::new(
            None,
            None,
            Positive::ZERO,
            Positive::ZERO,
            Utc::now(),
        ))
    }

    fn calculate_pnl_at_expiration(
        &self,
        _underlying_price: &Positive,
    ) -> Result<PnL, PricingError> {
        Ok(PnL::new(
            None,
            None,
            Positive::ZERO,
            Positive::ZERO,
            Utc::now(),
        ))
    }

    fn adjustments_pnl(&self, adjustments: &DeltaAdjustment) -> Result<PnL, PricingError> {
        let realized = match adjustments {
            DeltaAdjustment::BuyOptions { quantity, .. }
            | DeltaAdjustment::BuyUnderlying(quantity) => -quantity.to_dec(),
            DeltaAdjustment::SellOptions { quantity, .. }
            | DeltaAdjustment::SellUnderlying(quantity) => quantity.to_dec(),
            DeltaAdjustment::NoAdjustmentNeeded => Decimal::ZERO,
            DeltaAdjustment::SameSize(pair) => {
                let first = self.adjustments_pnl(&pair.first)?;
                let second = self.adjustments_pnl(&pair.second)?;
                first
                    .realized
                    .unwrap_or_default()
                    .checked_add(second.realized.unwrap_or_default())
                    .ok_or_else(|| {
                        PricingError::method_error("adjustments_pnl", "realized overflow")
                    })?
            }
        };
        Ok(PnL::new(
            Some(realized),
            None,
            Positive::ZERO,
            Positive::ZERO,
            Utc::now(),
        ))
    }
}

#[test]
fn test_adjustment_pnl_is_computable_without_strategies() {
    let calc = FlatAdjustmentPnl;
    let sell = DeltaAdjustment::SellOptions {
        quantity: pos_or_panic!(3.0),
        strike: Positive::HUNDRED,
        option_style: OptionStyle::Call,
        side: Side::Short,
    };
    let buy = DeltaAdjustment::BuyUnderlying(pos_or_panic!(2.0));
    let pair = DeltaAdjustment::SameSize(DeltaAdjustmentSameSize {
        first: Box::new(sell),
        second: Box::new(buy),
    });

    let pnl = calc.adjustments_pnl(&pair);
    assert!(pnl.is_ok(), "adjustment P&L must be computable: {pnl:?}");
    let pnl = pnl.unwrap_or_else(|_| unreachable!());
    assert_eq!(pnl.realized, Some(dec!(1.0)));
    assert_eq!(
        format!("{}", DeltaAdjustment::NoAdjustmentNeeded),
        "No adjustment needed"
    );
}
