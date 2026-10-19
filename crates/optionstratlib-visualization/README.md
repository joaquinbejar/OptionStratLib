# optionstratlib-visualization

Charts for [OptionStratLib](https://github.com/joaquinbejar/OptionStratLib):
backend-neutral chart models, the `Graph` contract, its implementations for
the library's types (options, positions, curves, surfaces, random walks,
simulators and every concrete strategy), and an optional Plotly backend with
static PNG/SVG export, plus the terminal tables of option chains and
simulation statistics. It is the leaf of the workspace: it depends on
`optionstratlib-core`, `optionstratlib-math`, `optionstratlib-pricing`,
`optionstratlib-simulation`, `optionstratlib-market`,
`optionstratlib-strategies` and `optionstratlib-backtest`, and no other
OptionStratLib crate depends on it.

| Module          | Contents                                                                 |
|-----------------|--------------------------------------------------------------------------|
| `visualization` | `Graph`, `GraphData`, `GraphConfig`, series, surfaces, styles, `PlotBuilder`, `Plottable`, the graph implementations; Plotly trace builders behind `plotly` |
| `visualization::terminal` | `ChainReport` and `SimulationReport`: option chains and simulation statistics as bordered terminal tables, rendered to a `String` or printed to stdout on request |
| `error`         | `GraphError`                                                             |

## Place in the workspace

- **Depends on** `optionstratlib-core`, `-math`, `-pricing`, `-simulation`,
  `-market`, `-strategies` and `-backtest`.
- **Must not be a dependency** of any other OptionStratLib crate: it is the
  leaf of the layering, and only the `optionstratlib` facade depends on it;
  `make check-graph` enforces the layering (ADR-0001 D9).
- **In the facade:** `optionstratlib::visualization`, `GraphError` and the
  aggregate `Error` in `optionstratlib::error` and the root
  `impl_graph_for_payoff_strategy!`, under the facade feature `visualization`;
  the facade's `plotly` and `static_export` forward to the features below. The
  facade paths are the same types as the paths here; the [ownership
  map](https://github.com/joaquinbejar/OptionStratLib/blob/main/docs/ownership.md)
  lists every one with its feature.

Moving from 0.21? The [0.22 architecture and adoption
guide](https://github.com/joaquinbejar/OptionStratLib/blob/main/docs/migration-0.22.md)
maps each redesign to its replacement workflow; 0.22 keeps no
compatibility with 0.21.

## Example

A runnable program that depends on this crate directly, with the smallest
dependency set and no facade, is [`osl-example-direct-visualization`](https://github.com/joaquinbejar/OptionStratLib/tree/main/examples/direct/visualization); `make tree-example-direct-visualization`
asserts its resolved graph.

## Features

| Feature         | Default | Effect                                                                 |
|-----------------|---------|------------------------------------------------------------------------|
| `plotly`        | no      | Plotly rendering: `to_plot`, `write_html`, `show`, `render`, trace builders. Resolves no image-export or async package |
| `static_export` | no      | PNG and SVG export (`write_png`, `write_svg`), drawn natively with Plotters: no browser, WebDriver or installed font; implies `plotly` |

Without either feature the crate builds the chart data (`graph_data`,
`graph_config`) and the terminal tables, and resolves no Plotly package.
It is the only OptionStratLib crate that depends on a terminal-table
package (`prettytable-rs`).

## Static export

`static_export` adds `Graph::write_png` and `Graph::write_svg`, and
`render(OutputType::Png | Svg)` through them. Since 0.23 they draw natively
with Plotters: no browser, WebDriver, network access or installed font is
involved, exports in threads or processes are independent, and the bundled
font and its licence ship in this crate. `static_export` still implies
`plotly`, so the Plotly Rust crate is a dependency for the HTML methods; it
plays no part in the images.

| Build | Chart data, `Graph`, terminal tables | `to_plot`, HTML, `show`, Plotly helpers | `write_png`, `write_svg` |
|---|---|---|---|
| no feature | yes | | |
| `plotly` | yes | yes | |
| `static_export` | yes | yes (implied `plotly`) | yes |

`show()` and `render(OutputType::Browser)` still hand an HTML chart to the
default browser; that is the interactive workflow, unchanged.

What the images are, and how they differ from the Plotly chart:

- **Source.** The images are drawn from `graph_data()` and
  `graph_config()`. An override of `Graph::to_plot` changes the HTML
  outputs, not the images; put image customisations in `GraphConfig`.
- **Text.** Every label uses the bundled Open Sans Regular (SIL OFL 1.1,
  `assets/OFL.txt`), read with skrifa, at Plotly's default sizes. A character
  the font has no glyph for (CJK, for example) is
  `GraphError::MissingGlyph`; tab, line feed and carriage return advance like
  a space. Labels are single lines with no kerning (the font's kerning
  table is not applied), no ligatures and no right-to-left shaping.
- **SVG text is outlines.** Each label is drawn as glyph paths, so the file
  looks the same in every viewer and needs no font. The label's text is
  kept as its accessible name (`<g role="img" aria-label>`), and the
  document carries a `<title>` and a `<desc>` naming the series and axes.
  The text cannot be selected, copied, found or edited as text. In Chrome's
  accessibility tree, checked when the design was chosen, each label is an
  `image` node named by its text, and the document is named by its title;
  how WebKit or a screen reader presents them was not checked. Use
  `write_html` for a text-searchable, interactive chart.
- **Surfaces.** A 3D surface is drawn from a fixed viewpoint with
  flat-shaded cells, no lighting and a Viridis colour bar (a `ColorScheme`
  does not apply to surfaces), and shows its three axis titles. It must be a
  complete grid: a missing point is `GraphError::IncompleteSurface`, a
  repeated one `GraphError::DuplicateSurfacePoint`.
- **Colours.** Colour strings follow CSS Color 4 (named colours, hex with
  or without alpha, `rgb()`, `rgba()`, `hsl()`, `hsla()`); a spelling only
  plotly.js tolerates is `GraphError::InvalidColor`.
- **Sizes.** Width and height must each be 1 to 16384 pixels.
- **Files.** The image is written next to its destination and renamed over
  it once complete, so a failed export leaves no partial file.

Data the export cannot draw faithfully is a typed `GraphError` rather than a
repaired chart. `GraphError` is `#[non_exhaustive]`: keep a wildcard arm.

```rust
use optionstratlib_visualization::error::GraphError;
use optionstratlib_visualization::visualization::{Graph, GraphData, Series2D, TraceMode};
use rust_decimal_macros::dec;

/// Three x values for two y values: a browser chart drew two points.
struct Lopsided;

impl Graph for Lopsided {
    fn graph_data(&self) -> GraphData {
        GraphData::Series(Series2D {
            x: vec![dec!(1), dec!(2), dec!(3)],
            y: vec![dec!(1), dec!(2)],
            name: "lopsided".to_string(),
            mode: TraceMode::Lines,
            line_color: None,
            line_width: None,
        })
    }
}

#[cfg(feature = "static_export")]
{
    let path = std::env::temp_dir().join("osl-readme-lopsided.png");
    let refused = match Lopsided.write_png(&path) {
        Err(GraphError::LengthMismatch { x, y, .. }) => Some((x, y)),
        Err(_) | Ok(()) => None,
    };
    assert_eq!(refused, Some((3, 2)));
    assert!(!path.exists());
}
```

## License

MIT
