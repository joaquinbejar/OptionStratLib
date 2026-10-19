# OptionStratLib 0.23.0 release notes

OptionStratLib 0.23.0 draws PNG and SVG charts natively. `static_export` no
longer drives a headless Chrome through `plotly_static` and chromedriver: the
visualization crate draws the images itself with Plotters, from the chart's
`graph_data()` and `graph_config()`, with no browser, WebDriver, network
access or installed font. **It is a breaking release for code that matches
`GraphError` exhaustively, overrides `Graph::to_plot` to customise images,
or exports data the browser used to repair silently.** Nothing else changes:
pricing, analytics, strategies, simulation, market data and backtesting
give the same results as in 0.22.2, and eight of the ten crates change only
their version.

- [Migration guide](https://github.com/joaquinbejar/OptionStratLib/blob/main/docs/migration-0.23.md):
  what to change and why.
- [ADR-0005](https://github.com/joaquinbejar/OptionStratLib/blob/main/doc/adr/ADR-0005-native-static-export.md):
  the decision, its consequences and the evidence behind it.
- [`optionstratlib-visualization` README, "Static export"](https://github.com/joaquinbejar/OptionStratLib/blob/main/crates/optionstratlib-visualization/README.md#static-export):
  what the images are and how they differ from Plotly's.
- [CHANGELOG](https://github.com/joaquinbejar/OptionStratLib/blob/main/CHANGELOG.md):
  every change with its issue.

## Packages

Ten packages, all at **0.23.0** (`docs/versioning-policy.md`: a minor
release resets the family), in dependency order.

| Package | Changes in 0.23.0 | crates.io | docs.rs |
| --- | --- | --- | --- |
| `optionstratlib-core` | version only | [0.23.0](https://crates.io/crates/optionstratlib-core/0.23.0) | [docs](https://docs.rs/optionstratlib-core/0.23.0) |
| `optionstratlib-math` | version only | [0.23.0](https://crates.io/crates/optionstratlib-math/0.23.0) | [docs](https://docs.rs/optionstratlib-math/0.23.0) |
| `optionstratlib-pricing` | version only | [0.23.0](https://crates.io/crates/optionstratlib-pricing/0.23.0) | [docs](https://docs.rs/optionstratlib-pricing/0.23.0) |
| `optionstratlib-simulation` | version only | [0.23.0](https://crates.io/crates/optionstratlib-simulation/0.23.0) | [docs](https://docs.rs/optionstratlib-simulation/0.23.0) |
| `optionstratlib-market` | version only | [0.23.0](https://crates.io/crates/optionstratlib-market/0.23.0) | [docs](https://docs.rs/optionstratlib-market/0.23.0) |
| `optionstratlib-analytics` | version only | [0.23.0](https://crates.io/crates/optionstratlib-analytics/0.23.0) | [docs](https://docs.rs/optionstratlib-analytics/0.23.0) |
| `optionstratlib-strategies` | version only | [0.23.0](https://crates.io/crates/optionstratlib-strategies/0.23.0) | [docs](https://docs.rs/optionstratlib-strategies/0.23.0) |
| `optionstratlib-backtest` | version only | [0.23.0](https://crates.io/crates/optionstratlib-backtest/0.23.0) | [docs](https://docs.rs/optionstratlib-backtest/0.23.0) |
| `optionstratlib-visualization` | native PNG/SVG export, typed `GraphError` variants, bundled font | [0.23.0](https://crates.io/crates/optionstratlib-visualization/0.23.0) | [docs](https://docs.rs/optionstratlib-visualization/0.23.0) |
| `optionstratlib` | documentation, docs.rs feature selection | [0.23.0](https://crates.io/crates/optionstratlib/0.23.0) | [docs](https://docs.rs/optionstratlib/0.23.0) |

## Who has to change something

- **You export PNG or SVG** (`write_png`, `write_svg`, `render` to an image,
  `PlotBuilder::save`): remove any chromedriver, `WEBDRIVER_PATH` or
  `BROWSER_PATH` setup; nothing needs to be installed.
- **You match `GraphError` exhaustively**: it is `#[non_exhaustive]` now and
  has new variants; add a wildcard arm.
- **You override `Graph::to_plot`** to change an image: the override still
  changes the HTML outputs, but images come from `graph_data()` and
  `graph_config()`; move the customisation there.
- **Your charts relied on a repair**: a surface with missing or repeated grid
  points, a series whose x and y lengths differ, a colour spelling only
  plotly.js accepts, a label character outside the bundled font, or a figure
  side above 16384 pixels is now a typed `GraphError` instead of a quietly
  altered image.
- **You select SVG text**: labels are glyph outlines with their text as an
  accessible name; they can no longer be selected, copied, searched or
  edited as text. Use `write_html` for a text-searchable chart.

Everything else, including every `plotly` and HTML API, is unchanged.

## Minimal manifests

Each manifest below is followed by a complete program; `make
check-release-notes` builds and runs every pair as its own crate outside the
repository, against this checkout, and the facade's doctests compile the
same programs. A program that writes `dec!` depends on `rust_decimal` itself.

PNG and SVG through the facade, with `static_export` alone (it implies
`plotly`), and a typed refusal of data the export will not repair:

```toml
[dependencies]
optionstratlib = { version = "0.23.0", default-features = false, features = ["static_export"] }
rust_decimal = "1.43"
```

```rust
use optionstratlib::error::GraphError;
use optionstratlib::prelude::*;
use optionstratlib::visualization::{GraphData, Surface3D};

/// Three of the four points of a 2 x 2 strike/expiry grid.
struct GappySurface;

impl Graph for GappySurface {
    fn graph_data(&self) -> GraphData {
        GraphData::GraphSurface(Surface3D {
            x: vec![dec!(90), dec!(100), dec!(90)],
            y: vec![dec!(7), dec!(7), dec!(30)],
            z: vec![dec!(0.21), dec!(0.18), dec!(0.20)],
            name: "Implied volatility".to_string(),
        })
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let call = LongCall::new(
        "XYZ".to_string(),
        pos_or_panic!(100.0),
        ExpirationDate::Days(pos_or_panic!(30.0)),
        pos_or_panic!(0.2),
        Positive::ONE,
        pos_or_panic!(100.0),
        dec!(0.05),
        Positive::ZERO,
        pos_or_panic!(2.5),
        pos_or_panic!(0.5),
        pos_or_panic!(0.5),
    )?;
    let dir = std::env::temp_dir();
    let (png, svg) = (dir.join("osl-0.23-call.png"), dir.join("osl-0.23-call.svg"));
    call.write_png(&png)?;
    call.write_svg(&svg)?;
    assert!(std::fs::read(&png)?.starts_with(b"\x89PNG"));
    assert!(!std::fs::read_to_string(&svg)?.contains("<text"));

    // `GraphError` is `#[non_exhaustive]`: keep a wildcard arm.
    match GappySurface.write_png(&dir.join("osl-0.23-gappy.png")) {
        Err(GraphError::IncompleteSurface { expected, found, .. }) => {
            assert_eq!((expected, found), (4, 3));
        }
        Err(other) => return Err(other.into()),
        Ok(()) => return Err("the gappy surface was drawn".into()),
    }
    Ok(())
}
```

The same export on the visualization crate alone, without the facade:

```toml
[dependencies]
optionstratlib-visualization = { version = "0.23.0", features = ["static_export"] }
rust_decimal = "1.43"
rust_decimal_macros = "1.40"
```

```rust
use optionstratlib_visualization::visualization::{Graph, GraphData, Series2D, TraceMode};
use rust_decimal_macros::dec;

struct Payoff;

impl Graph for Payoff {
    fn graph_data(&self) -> GraphData {
        GraphData::Series(Series2D {
            x: vec![dec!(90), dec!(100), dec!(110)],
            y: vec![dec!(-2.5), dec!(-2.5), dec!(7.5)],
            name: "Long call".to_string(),
            mode: TraceMode::Lines,
            line_color: None,
            line_width: None,
        })
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::temp_dir().join("osl-0.23-direct.svg");
    Payoff.write_svg(&path)?;
    let svg = std::fs::read_to_string(&path)?;
    assert!(svg.starts_with("<svg") && !svg.contains("<text"));
    Ok(())
}
```

## Changes

### Static export

- PNG and SVG are drawn by `optionstratlib-visualization` with Plotters
  0.3.7 (no font feature), plotters-backend 0.3.7, skrifa 0.48 and
  ab_glyph_rasterizer 0.1.10, all behind `static_export`. The font is Open
  Sans Regular, bundled with its SIL OFL 1.1 licence.
- Exports are independent across threads and processes, need no lock, and
  write next to the destination before renaming into place, so a failure
  leaves no partial file.
- SVG labels are glyph outlines inside `<g role="img" aria-label="…">`, and
  the document carries a `<title>` and a `<desc>`.
- The 3D surface is drawn from a fixed viewpoint with flat-shaded cells and
  a Viridis colour bar, and shows its three axis titles.
- `GraphError` is `#[non_exhaustive]` with fourteen new variants; `render`
  and `PlotBuilder::save` return them unchanged, and file failures are
  `GraphError::Io`.

### Dependencies and security

- `static_export` no longer resolves `plotly_static`, `fantoccini`,
  `webdriver`, an HTTP client, an async runtime or `ttf-parser`:
  `ttf-parser` (RUSTSEC-2026-0192) is in no published path.
- `cargo audit --deny warnings` passes with no advisory waived.

### Verification and documentation

- Pull requests run real PNG and SVG exports through the component, the
  facade and two consumer fixtures, and every example with its real
  exports; the weekly browser workflow is retired.
- The publication dry run builds and tests every archive with all features,
  `static_export` included.
- The docs.rs metadata of the facade and the visualization crate selects
  `static_export`, so their documentation includes `write_png` and
  `write_svg`.

## Results that change

None. The only behaviour changes are the static export's, listed above and
in the migration guide.
