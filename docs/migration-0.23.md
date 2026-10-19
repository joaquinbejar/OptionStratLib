# Moving to 0.23: native PNG and SVG export

0.23 changes how `static_export` draws images. The `Graph` methods keep their
signatures; what changes is what draws them and which charts they accept.
The full list is in `CHANGELOG.md`.

## Nothing to install

PNG and SVG are drawn by the library itself, with Plotters. Remove any
chromedriver, `WEBDRIVER_PATH` or `BROWSER_PATH` setup you kept for
`static_export`; a container or CI job needs neither a browser nor fonts.

## `to_plot` overrides reach HTML only

`write_png` and `write_svg` now draw from `graph_data()` and
`graph_config()`. If you override `Graph::to_plot` to customise the chart,
the override still applies to `write_html`, `show` and `render` to HTML, but
not to the images. Move image customisations into the `GraphData` and
`GraphConfig` you return:

```rust,ignore
impl Graph for MyChart {
    fn graph_data(&self) -> GraphData { /* series, surfaces */ }

    fn graph_config(&self) -> GraphConfig {
        GraphConfig {
            title: "Custom title".to_string(),
            width: 1600,
            color_scheme: ColorScheme::Custom(vec!["tomato".into(), "#1e90ff".into()]),
            ..GraphConfig::default()
        }
    }

    // Still used for HTML and `show()`, no longer for PNG/SVG.
    fn to_plot(&self) -> Plot { /* ... */ }
}
```

## Bad data is an error, not a repaired chart

The Plotly export quietly repaired some data; the native export reports it as
a `GraphError` variant you can match:

| Data | Plotly export | 0.23 |
|---|---|---|
| Surface with missing grid points | filled with 0 | `IncompleteSurface` |
| Surface with a repeated grid point | last value kept | `DuplicateSurfacePoint` |
| Series with more x than y values | shorter length drawn | `LengthMismatch` |
| Colour plotly.js tolerates but CSS does not | drawn | `InvalidColor` |
| Label character outside the bundled font | browser's fallback font | `MissingGlyph` |
| Width or height above 16384 pixels | drawn by the browser | `InvalidDimensions` |

`GraphError` is `#[non_exhaustive]`: keep a wildcard arm.

```rust,ignore
match chart.write_png(path) {
    Ok(()) => {}
    Err(GraphError::IncompleteSurface { expected, found, .. }) => { /* fill the grid */ }
    Err(other) => return Err(other.into()),
}
```

## SVG text is outlines

Every label in an SVG is drawn as glyph outlines, so the file looks the same
in any viewer and needs no font. Each label keeps its text as an accessible
name and the document has a `<title>` and `<desc>`, but the text cannot be
selected, copied or searched as text. Use `write_html` for an interactive,
text-searchable chart.
