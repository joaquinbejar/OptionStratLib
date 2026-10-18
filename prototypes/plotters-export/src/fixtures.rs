//! The three charts both renderers draw, built deterministically.
//!
//! * [`Case::Payoff`]: a real `BullCallSpread` charted through the library's
//!   own `Graph` implementation: profit/loss with negative and positive
//!   values, a zero crossing, two slope changes, a zero line, the current
//!   price marker and the break-even text label.
//! * [`Case::Curves`]: Black-Scholes call values over the underlying price
//!   for four volatilities plus the intrinsic value, five named series in a
//!   legend.
//! * [`Case::Surface`]: an implied-volatility surface over strike and days
//!   to expiry with skew, smile curvature and an inverted short end.
//! * [`Case::Text`]: not one of the compared charts, but a compact chart of
//!   awkward text for #923 (text as SVG outlines): pairs kerning would
//!   tighten, a ligature candidate, padding spaces, accents, Greek, currency
//!   signs, a true minus, scientific ticks and characters SVG escapes.

use optionstratlib_core::model::{
    ExpirationDate, OptionStyle, OptionType, Options, Positive, Side,
};
use optionstratlib_pricing::pricing::black_scholes_model::black_scholes;
use optionstratlib_strategies::strategies::BullCallSpread;
use optionstratlib_visualization::visualization::{
    ColorScheme, Graph, GraphConfig, GraphData, LineStyle, Series2D, Surface3D, TraceMode,
};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use thiserror::Error;

/// Why a fixture could not be built.
#[derive(Debug, Error)]
pub enum FixtureError {
    /// A library constructor rejected the fixture's parameters.
    #[error("{case}: {reason}")]
    Library {
        /// The fixture being built.
        case: &'static str,
        /// The library's error, as text.
        reason: String,
    },
    /// Decimal arithmetic overflowed.
    #[error("{0}: decimal arithmetic overflowed")]
    Overflow(&'static str),
}

/// The charts of the comparison.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Case {
    /// A bull call spread's profit/loss.
    Payoff,
    /// Black-Scholes call values for four volatilities.
    Curves,
    /// An implied-volatility surface.
    Surface,
    /// The text fixture of #923; not part of [`Case::ALL`].
    Text,
}

impl Case {
    /// The three compared charts, in report order.
    pub const ALL: [Case; 3] = [Case::Payoff, Case::Curves, Case::Surface];

    /// The compared charts and the text fixture.
    pub const EVERY: [Case; 4] = [Case::Payoff, Case::Curves, Case::Surface, Case::Text];

    /// File-name stem of the case.
    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Case::Payoff => "payoff",
            Case::Curves => "curves",
            Case::Surface => "surface",
            Case::Text => "text",
        }
    }

    /// The case named `slug`, if any.
    #[must_use]
    pub fn from_slug(slug: &str) -> Option<Case> {
        Case::EVERY.into_iter().find(|case| case.slug() == slug)
    }

    /// Builds the case's chart.
    ///
    /// # Errors
    ///
    /// [`FixtureError`] when a library constructor rejects the parameters or
    /// the arithmetic overflows; neither happens with the fixed inputs.
    pub fn build(self) -> Result<Fixture, FixtureError> {
        match self {
            Case::Payoff => payoff(),
            Case::Curves => curves(),
            Case::Surface => surface(),
            Case::Text => text(),
        }
    }
}

/// A chart with its configuration, drawable by both renderers.
#[derive(Debug, Clone, PartialEq)]
pub struct Fixture {
    /// The chart data.
    pub data: GraphData,
    /// The chart configuration.
    pub config: GraphConfig,
}

impl Graph for Fixture {
    fn graph_data(&self) -> GraphData {
        self.data.clone()
    }

    fn graph_config(&self) -> GraphConfig {
        self.config.clone()
    }
}

#[cold]
fn library_error(case: &'static str, error: impl std::fmt::Display) -> FixtureError {
    FixtureError::Library {
        case,
        reason: error.to_string(),
    }
}

fn positive(case: &'static str, value: Decimal) -> Result<Positive, FixtureError> {
    Positive::new_decimal(value).map_err(|error| library_error(case, error))
}

/// The bull call spread of the direct visualization example.
fn payoff() -> Result<Fixture, FixtureError> {
    const CASE: &str = "payoff";
    let p = |value: Decimal| positive(CASE, value);
    let strategy = BullCallSpread::new(
        "SP500".to_string(),
        p(dec!(5781.88))?,
        p(dec!(5750))?,
        p(dec!(5820))?,
        ExpirationDate::Days(Positive::TWO),
        p(dec!(0.18))?,
        dec!(0.05),
        Positive::ZERO,
        Positive::TWO,
        p(dec!(85.04))?,
        p(dec!(29.85))?,
        p(dec!(0.78))?,
        p(dec!(0.78))?,
        p(dec!(0.73))?,
        p(dec!(0.73))?,
    )
    .map_err(|error| library_error(CASE, error))?;
    Ok(Fixture {
        data: strategy.graph_data(),
        config: strategy.graph_config(),
    })
}

const CURVE_VOLATILITIES: [Decimal; 4] = [dec!(0.10), dec!(0.20), dec!(0.30), dec!(0.40)];
const CURVE_STRIKE: Decimal = dec!(100);
const CURVE_DAYS: Decimal = dec!(30);
const CURVE_RATE: Decimal = dec!(0.05);
const CURVE_FIRST_PRICE: i64 = 70;
const CURVE_LAST_PRICE: i64 = 130;

fn call(underlying: Decimal, volatility: Decimal) -> Result<Options, FixtureError> {
    const CASE: &str = "curves";
    Ok(Options::new(
        OptionType::European,
        Side::Long,
        "XYZ".to_string(),
        positive(CASE, CURVE_STRIKE)?,
        ExpirationDate::Days(positive(CASE, CURVE_DAYS)?),
        positive(CASE, volatility)?,
        Positive::ONE,
        positive(CASE, underlying)?,
        CURVE_RATE,
        OptionStyle::Call,
        Positive::ZERO,
        None,
    ))
}

fn curves() -> Result<Fixture, FixtureError> {
    const CASE: &str = "curves";
    let prices: Vec<Decimal> = (CURVE_FIRST_PRICE..=CURVE_LAST_PRICE)
        .map(Decimal::from)
        .collect();
    let mut series = Vec::with_capacity(CURVE_VOLATILITIES.len() + 1);
    for volatility in CURVE_VOLATILITIES {
        let mut values = Vec::with_capacity(prices.len());
        for price in &prices {
            let value =
                black_scholes(&call(*price, volatility)?).map_err(|e| library_error(CASE, e))?;
            values.push(value.round_dp(4));
        }
        let percent = volatility
            .checked_mul(dec!(100))
            .ok_or(FixtureError::Overflow(CASE))?
            .normalize();
        series.push(Series2D {
            x: prices.clone(),
            y: values,
            name: format!("σ = {percent}%"),
            mode: TraceMode::Lines,
            line_color: None,
            line_width: None,
        });
    }
    let intrinsic = prices
        .iter()
        .map(|price| {
            price
                .checked_sub(CURVE_STRIKE)
                .map(|value| value.max(Decimal::ZERO))
                .ok_or(FixtureError::Overflow(CASE))
        })
        .collect::<Result<_, _>>()?;
    series.push(Series2D {
        x: prices,
        y: intrinsic,
        name: "Intrinsic value".to_string(),
        mode: TraceMode::LinesMarkers,
        line_color: Some("#7f7f7f".to_string()),
        line_width: Some(1.0),
    });
    Ok(Fixture {
        data: GraphData::MultiSeries(series),
        config: GraphConfig {
            title: "Call value by implied volatility (K = 100, 30 days)".to_string(),
            width: 1280,
            height: 720,
            x_label: Some("Underlying price".to_string()),
            y_label: Some("Call value".to_string()),
            z_label: None,
            line_style: LineStyle::Solid,
            color_scheme: ColorScheme::Default,
            legend: None,
            show_legend: true,
        },
    })
}

const SURFACE_STRIKES: (i64, i64, i64) = (70, 130, 5);
/// Days to expiry, with each expiry's at-the-money level, skew slope and
/// smile curvature (per unit of moneyness `(K - 100) / 100`): an inverted
/// short end, flattening skew and curvature with time.
const SURFACE_EXPIRIES: [(i64, Decimal, Decimal, Decimal); 7] = [
    (7, dec!(0.240), dec!(-0.45), dec!(0.90)),
    (14, dec!(0.225), dec!(-0.38), dec!(0.75)),
    (30, dec!(0.210), dec!(-0.30), dec!(0.60)),
    (60, dec!(0.205), dec!(-0.24), dec!(0.45)),
    (90, dec!(0.200), dec!(-0.20), dec!(0.38)),
    (180, dec!(0.198), dec!(-0.15), dec!(0.30)),
    (365, dec!(0.196), dec!(-0.11), dec!(0.22)),
];

fn surface() -> Result<Fixture, FixtureError> {
    const CASE: &str = "surface";
    let (first, last, step) = SURFACE_STRIKES;
    let strikes: Vec<i64> = (first..=last)
        .step_by(step.unsigned_abs() as usize)
        .collect();
    let capacity = strikes
        .len()
        .checked_mul(SURFACE_EXPIRIES.len())
        .unwrap_or_default();
    let (mut x, mut y, mut z) = (
        Vec::with_capacity(capacity),
        Vec::with_capacity(capacity),
        Vec::with_capacity(capacity),
    );
    for strike in strikes {
        let strike = Decimal::from(strike);
        let moneyness = strike
            .checked_sub(dec!(100))
            .and_then(|offset| offset.checked_div(dec!(100)))
            .ok_or(FixtureError::Overflow(CASE))?;
        for (days, atm, skew, curvature) in SURFACE_EXPIRIES {
            let volatility = skew
                .checked_mul(moneyness)
                .and_then(|slope| atm.checked_add(slope))
                .and_then(|level| {
                    let smile = curvature.checked_mul(moneyness.checked_mul(moneyness)?)?;
                    level.checked_add(smile)
                })
                .ok_or(FixtureError::Overflow(CASE))?;
            x.push(strike);
            y.push(Decimal::from(days));
            z.push(volatility.round_dp(4));
        }
    }
    Ok(Fixture {
        data: GraphData::GraphSurface(Surface3D {
            x,
            y,
            z,
            name: "Implied volatility".to_string(),
        }),
        config: GraphConfig {
            title: "Implied volatility surface".to_string(),
            width: 1280,
            height: 720,
            x_label: Some("Strike".to_string()),
            y_label: Some("Days to expiry".to_string()),
            z_label: Some("Implied volatility".to_string()),
            line_style: LineStyle::Solid,
            color_scheme: ColorScheme::Viridis,
            legend: None,
            show_legend: false,
        },
    })
}

/// Labels of the text fixture: one series per awkward-text group. Every
/// character is in the bundled Open Sans.
pub const TEXT_SERIES: [&str; 4] = [
    "  AV To fi  ",
    "café · ñandú · Ωμέγα αβγ",
    "€ £ ¥ $ 1.5e−7",
    "& < > \" ' $",
];
/// Title of the text fixture.
pub const TEXT_TITLE: &str = "Text: AV To fi, café, Ωμέγα, € £ ¥ $ & < > \" '";

/// Points of the text fixture's series: tiny values on both sides of zero,
/// so the y ticks are scientific with a true minus.
const TEXT_POINTS: i64 = 11;
const TEXT_SLOPE: Decimal = dec!(0.0000003);

fn text() -> Result<Fixture, FixtureError> {
    const CASE: &str = "text";
    let x: Vec<Decimal> = (0..TEXT_POINTS).map(Decimal::from).collect();
    let mut series = Vec::with_capacity(TEXT_SERIES.len());
    let modes = [
        TraceMode::Lines,
        TraceMode::LinesMarkers,
        TraceMode::Markers,
        TraceMode::TextLabels,
    ];
    let offsets = [dec!(-1), dec!(0), dec!(1), dec!(2)];
    for ((name, mode), offset) in TEXT_SERIES.into_iter().zip(modes).zip(offsets) {
        let y = x
            .iter()
            .map(|value| {
                value
                    .checked_sub(dec!(5))
                    .and_then(|centred| centred.checked_add(offset))
                    .and_then(|shifted| shifted.checked_mul(TEXT_SLOPE))
                    .ok_or(FixtureError::Overflow(CASE))
            })
            .collect::<Result<_, _>>()?;
        let (x, y) = if mode == TraceMode::TextLabels {
            // Two labels are enough to show centred point labels.
            let pick = |values: Vec<Decimal>| values.into_iter().step_by(5).take(2).collect();
            (pick(x.clone()), pick(y))
        } else {
            (x.clone(), y)
        };
        series.push(Series2D {
            x,
            y,
            name: name.to_string(),
            mode,
            line_color: None,
            line_width: None,
        });
    }
    Ok(Fixture {
        data: GraphData::MultiSeries(series),
        config: GraphConfig {
            title: TEXT_TITLE.to_string(),
            width: 960,
            height: 540,
            x_label: Some("Strike − spot ($)".to_string()),
            y_label: Some("Δ per $1 (Γ = 1.5e−7)".to_string()),
            z_label: None,
            line_style: LineStyle::Solid,
            color_scheme: ColorScheme::Default,
            legend: None,
            show_legend: true,
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_payoff_fixture_crosses_zero() -> Result<(), FixtureError> {
        let fixture = Case::Payoff.build()?;
        let GraphData::MultiSeries(series) = fixture.data else {
            panic!("the payoff chart is a multi-series chart");
        };
        let profit = series.iter().flat_map(|s| s.y.iter()).copied();
        let (low, high) = profit.fold((Decimal::MAX, Decimal::MIN), |(l, h), v| {
            (l.min(v), h.max(v))
        });
        assert!(low < Decimal::ZERO && high > Decimal::ZERO);
        Ok(())
    }

    #[test]
    fn test_curves_fixture_has_five_named_series() -> Result<(), FixtureError> {
        let fixture = Case::Curves.build()?;
        let GraphData::MultiSeries(series) = fixture.data else {
            panic!("the curves chart is a multi-series chart");
        };
        assert_eq!(series.len(), 5);
        assert_eq!(series.first().map(|s| s.name.as_str()), Some("σ = 10%"));
        assert!(series.iter().all(|s| s.x.len() == 61 && s.y.len() == 61));
        Ok(())
    }

    #[test]
    fn test_surface_fixture_is_a_complete_skewed_grid() -> Result<(), FixtureError> {
        let fixture = Case::Surface.build()?;
        let GraphData::GraphSurface(surface) = fixture.data else {
            panic!("the surface chart is a surface");
        };
        assert_eq!(surface.z.len(), 13 * 7);
        // Downside skew: the 70 strike is richer than the 130 strike at 7 days.
        let at = |strike: Decimal, days: Decimal| {
            surface
                .x
                .iter()
                .zip(&surface.y)
                .zip(&surface.z)
                .find(|((x, y), _)| **x == strike && **y == days)
                .map(|(_, z)| *z)
        };
        assert!(at(dec!(70), dec!(7)) > at(dec!(130), dec!(7)));
        assert_eq!(at(dec!(100), dec!(7)), Some(dec!(0.24)));
        Ok(())
    }

    #[test]
    fn test_case_slugs_round_trip() {
        for case in Case::EVERY {
            assert_eq!(Case::from_slug(case.slug()), Some(case));
        }
        assert_eq!(Case::from_slug("heatmap"), None);
        assert!(!Case::ALL.contains(&Case::Text));
    }

    #[test]
    fn test_text_fixture_names_every_awkward_label() -> Result<(), FixtureError> {
        let fixture = Case::Text.build()?;
        let GraphData::MultiSeries(series) = fixture.data else {
            panic!("the text chart is a multi-series chart");
        };
        let names: Vec<&str> = series.iter().map(|s| s.name.as_str()).collect();
        assert_eq!(names, TEXT_SERIES);
        assert!(series.iter().all(|s| s.x.len() == s.y.len()));
        assert!(
            series
                .iter()
                .any(|s| s.mode == TraceMode::TextLabels && s.x.len() == 2)
        );
        Ok(())
    }
}
