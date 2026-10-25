# ADR-0005: Native PNG and SVG Export

- Status: Accepted (owner decisions of 2026-10-19; implemented in 0.23).
- Date: 2026-10-19
- Issues: #925, #926, #927, #928, #929, #930; experiments #920 and #923.
- Supersedes, for the static export only: the browser export recorded in the
  0.22 workspace and feature ADRs, which this record restates below so that
  it can be read on its own.

## Context

`optionstratlib-visualization` describes every chart once, as
backend-neutral data: `Graph::graph_data()` returns series or a surface and
`Graph::graph_config()` the title, size, labels and colours. Two backends
consume it:

- **Interactive HTML**, behind the `plotly` feature: `Graph::to_plot()`
  builds a Plotly figure, written by `write_html` or opened by `show()`.
- **Static images**, behind `static_export`: `Graph::write_png`,
  `Graph::write_svg` and `render(OutputType::Png | Svg)`.

Up to 0.22 the static images went through the same Plotly figure:
`plotly_static` started chromedriver and a headless Chrome, loaded the
figure and asked plotly.js for the image. That cost:

- **A browser on every machine that exports.** Chrome and a chromedriver of
  the same major version had to be installed (`WEBDRIVER_PATH`,
  `BROWSER_PATH`); the build script of `plotly_static` downloaded a driver,
  so building `static_export` needed network access and could not run on
  docs.rs.
- **Leaked processes and slow exports.** Exports took one to two seconds
  each, ran one at a time behind a process-wide lock, and left Chrome and
  chromedriver processes behind on failure.
- **Untestable in CI.** The export tests were `#[ignore]`d and ran weekly
  in a separate workflow with a Chrome and driver pair; pull requests never
  exercised an export.
- **Advisories.** The download stack brought `reqwest` 0.11, `h2` 0.3,
  `rustls-pemfile` and zip below 9, each waived in `.cargo/audit.toml` as a
  build-time-only path.
- **Silent repairs.** The old Plotly export path filled the missing points
  of an incomplete surface with zero and kept the last of repeated points
  (both in the crate's own `make_surface`), drew the shorter of two series
  arrays and accepted colour spellings CSS does not.

The experiments #920 (Plotters renderer, timings, leak checks) and #923
(SVG text as outlines, viewer and accessibility checks) measured a native
alternative in `prototypes/plotters-export`. Their reports
(`REPORT.md`, `REPORT-outlines.md`, `REPORT-text-path.md`) keep their
measurements and the commits they were taken at; they are evidence for this
decision, not a description of the released behaviour.

## Decision

The owner decided on 2026-10-19:

1. **PNG and SVG are drawn natively with Plotters; Plotly keeps HTML.**
   `write_png`, `write_svg` and `render` to PNG or SVG keep their
   signatures and are drawn by `optionstratlib-visualization` itself, from
   `graph_data()` and `graph_config()`. `to_plot()`, `write_html`, `show`
   and the Plotly helpers are unchanged.
2. **`to_plot()` overrides reach HTML only.** A downstream override of
   `Graph::to_plot` keeps changing the HTML outputs and no longer the
   images; the change is documented, with no browser fallback and no new
   customisation hook (`docs/migration-0.23.md`).
3. **SVG text is drawn as outlines.** Each label becomes glyph paths of the
   bundled font inside `<g role="img" aria-label="…">`, and the document
   carries `<title>` and `<desc>`, so the file renders identically without
   any font and keeps its strings as accessible names.
4. **No `ttf-parser` in a published crate** (RUSTSEC-2026-0192). The font is
   read with skrifa; Plotters is built with no font feature, and a text
   backend measures and draws every label; PNG glyphs are rasterised with
   `ab_glyph_rasterizer`, which parses no font. The font is Open Sans
   Regular, bundled under `assets/` with its SIL OFL 1.1 licence, separate
   from the crate's MIT licence.
5. **Data the export cannot draw faithfully is a typed error.** `GraphError`
   becomes `#[non_exhaustive]` and gains `InvalidDimensions`, `TooSmall`,
   `NoRoom`, `InvalidLineWidth`, `LengthMismatch`, `SurfaceLengthMismatch`,
   `DegenerateSurface`, `DuplicateSurfacePoint`, `IncompleteSurface`,
   `NotRepresentable`, `InvalidColor`, `MissingGlyph`, `NotXmlText` and
   `UnsupportedText`. `render` and `PlotBuilder::save` return them
   unchanged; file failures are `GraphError::Io`.
6. **The release is 0.23.** The new variants are breaking for an exhaustive
   match, so every crate moves to 0.23.0 (`docs/versioning-policy.md`).
   Publishing, tagging and the GitHub release remain separate owner
   decisions.

Approved dependencies, all optional behind `static_export`: plotters 0.3.7
(`bitmap_backend`, `bitmap_encoder`, `colormaps`, `full_palette`,
`line_series`, `svg_backend`; no font feature), plotters-backend 0.3.7,
skrifa 0.48 (`std` only) and ab_glyph_rasterizer 0.1.10.

### Feature matrix

| Build | Chart data, `Graph`, terminal tables | `to_plot`, HTML, `show`, Plotly helpers | `write_png`, `write_svg` |
|---|---|---|---|
| component, no feature / facade `visualization` | yes | | |
| `plotly` | yes | yes | |
| `static_export` | yes | yes (implies `plotly`) | yes |

`static_export` still implies `plotly` at both entry points, so the Plotly
Rust crate remains a dependency; no browser is involved in the images. The
facade's `async` keeps its own meaning (market wrappers on Tokio) and is not
implied by `static_export`.

### Verification

- `make check-graph` confines the native renderer to `static_export` and
  forbids font parsers, `plotly_static` and the WebDriver stack in every
  crate; `make check-feature-trees` pins every facade surface, including
  `static_export,async`, and where `tokio` may come from.
- `visualization.yml` runs each surface (neutral, `plotly`, `static_export`
  alone, `static_export` with `plotly`) with real PNG and SVG exports
  through the component, the facade and both `static_export` consumer
  fixtures; the examples workflow runs every example with its real exports,
  and an export error fails it.
- The publication dry run builds and tests every archive with all features,
  `static_export` included. The docs.rs metadata of both crates selects
  `static_export`; a local build with that selection, `--cfg docsrs` and
  warnings denied passes offline with network access denied (#930). The
  hosted docs.rs build happens only on publication.

## Consequences

- **No browser, driver, network or installed font** for PNG or SVG, in
  builds, tests, CI or docs.rs. Exports are independent across threads and
  processes, and are written next to their destination and renamed into
  place, so a failure leaves no partial file.
- **Visible differences from the Plotly images.** Text sizes, line widths
  and the series colourway follow Plotly's defaults, but the images are not
  pixel-identical.
  The 3D surface is drawn from one fixed viewpoint with flat-shaded cells,
  no lighting and a Viridis colour bar, and requires a complete grid; it
  shows the three axis titles the Plotly export lost. When the x and depth
  axes both have a tick at the front corner where they meet, the depth
  label there is left out so the two do not overprint (#937).
- **Text.** Labels are single lines with no kerning, ligatures or
  right-to-left shaping; a character outside the bundled font is
  `MissingGlyph`. SVG text cannot be selected, copied, found or edited. In
  Chrome's accessibility tree each label is an `image` node named by its
  text, and the chart is named and described by its `<title>` and `<desc>`
  (#923); how WebKit or a screen reader presents them was not checked, and
  the chart as a whole is not claimed to be accessible.
- **Stricter input.** Incomplete or repeated surface points, mismatched
  arrays, non-CSS colours, line widths outside 0 to 100 pixels or not
  finite, sizes outside 1 to 16384 pixels and text XML cannot carry are
  errors.
- **Advisories.** With the download stack gone, `.cargo/audit.toml` waives
  nothing (#928).
- **Maintenance.** The SVG outline converter parses the exact output of
  plotters-svg 0.3.7, and its tests fail on any change there; the font
  layout mirrors the earlier ab_glyph metrics and is locked by tests.

## Evidence

- Experiments: #920 / PR #921 (renderer, timings, leaks:
  `prototypes/plotters-export/REPORT.md`), #923 / PR #924 (outlines, three
  rendering engines, accessibility: `REPORT-outlines.md`), PR #931 (skrifa
  text path, no `ttf-parser`: `REPORT-text-path.md`). Their numbers belong
  to the commits recorded in those reports.
- Implementation: #925 (CSS colours, in the prototype), PR #933 (#927:
  cutover, typed errors, 0.23), PR #934 (#928: feature isolation, fresh
  dependency resolution, audit), PR #935 (#929: PR CI, browser workflow
  retired), and #930 (this record, documentation, docs.rs, package
  evidence).
- Owner decisions: ACS notes `static-export-migration-decisions-2026-10-19`,
  `static-export-dependency-approval-2026-10-19` and
  `static-export-contract-decisions-2026-10-19`.
