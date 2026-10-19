# SVG text as outlines: follow-up report (#923)

Follow-up to the [#920 decision report](REPORT.md), whose original
measurements and provenance stay as they were. The baseline is `851899e8`
(PR #921).

**Conclusions, kept separate as #923 asks:**

1. **The experiment is complete.** The prototype has a fourth SVG text mode,
   `SvgText::Outlines`. Every artifact, render, comparison and measurement
   below was produced from the implementation commit `ba77d074` and can be
   reproduced with the commands at the end of this report.
2. **The SVG portability condition of #920 is met by the outlines mode.**
   All 86 labels of the three charts and the text fixture match in Blink,
   WebKit and librsvg for every pair of engines. The comparison method and
   tolerance were declared before the run, and the same method rejects the
   embedded-font SVG in librsvg. The outlines SVG names no font, embeds none
   and loads nothing.
3. **The remaining tradeoffs justify taking the next step.** That step is to
   work through the other migration conditions of REPORT.md (conditions 2
   to 8), not to migrate. What the outlines cost is listed under
   [Tradeoffs](#tradeoffs): text can no longer be selected, searched or
   edited, and each in-process SVG export takes about 0.6 ms longer.
   Nothing in a published crate changed. The migration decision, and every
   other condition, remain open and belong to the owner.

## What was built

All of it lives in `prototypes/plotters-export` (`publish = false`).

- `src/outline.rs`: `outline_text` replaces each `<text>` element with the
  outlines of the bundled Open Sans, as
  `<g role="img" aria-label="…"><path d="…"/></g>`. It keeps the element's
  fill, opacity and rotation, and adds a document `<title>` and `<desc>`.
  `render_svg(…, SvgText::Outlines)` and `render_svg_outlined` (which also
  returns each label's ink box) call it.
- Typed failures (`src/error.rs`):
  - `ExportError::MissingGlyph`: a character the bundled font has no glyph
    for.
  - `ExportError::UnsupportedSvgText`: anything outside the subset
    plotters-svg 0.3.7 writes. That covers another family, bold or italic,
    a transform other than a quarter turn about the anchor, markup or an
    unknown entity in the text, and a missing or duplicate attribute.
  - `ExportError::NotXmlText`: a label, title or description containing a
    character XML 1.0 cannot carry (a C0 control other than tab, line feed
    and carriage return, U+FFFE or U+FFFF). The description names series
    that may never be drawn, such as a hidden legend's, so it is checked
    before anything is written rather than relying on the glyph check.
    Found in review: before the check, a NUL in a hidden series name gave
    an unparsable SVG with `Ok`.

  Nothing falls back to a viewer font or a raster, and no label is
  dropped. The one exception is a label that is empty or only spaces: it
  has nothing to paint or announce, so it is omitted.
- `fixtures::Case::Text`: a compact chart of awkward text. It is not one of
  the three compared charts (`Case::ALL` is unchanged; `Case::EVERY` adds
  it) and contains:
  - kerning pairs `AV` and `To`, the ligature candidate `fi`, and leading
    and trailing spaces;
  - accents, Greek, `€ £ ¥ $`, U+2212, scientific ticks, and `& < > " '`;
  - left, centred and right anchors, and a rotated axis title.
- `compare outlines <dir>`: PNG plus the three SVG text modes for every case.
  `manifest.json` records each file's mode (`text`: `embedded-font`,
  `viewer-font` or `outlines`), and `<case>.labels.json` holds the label ink
  boxes. `compare export … svg-outlines` exports a single file, and the
  Criterion bench has a `svg-outlines` row per case.
- `render_svg_viewers.sh` (rewritten), `webkit_snapshot.swift`,
  `compare_viewers.py` and `accessibility_tree.mjs`: the evidence harness
  described below. `measure.py` gained `--cases`, `--formats` and the
  `svg-outlines` format, and its sandbox now also denies reads of the font
  directories.

### Implementation note: where the paths are made

The two options in #923 were to generate paths at the text drawing boundary
or to convert the SVG subset this prototype emits. The conversion was
chosen. Plotters' `DrawingBackend` has no path primitive with holes or a
fill rule (`fill_polygon` takes a single contour), and `SVGBackend` does not
expose its buffer. A wrapper backend would therefore still have to
post-process the document.

The converter accepts exactly the element form of plotters-svg 0.3.7 and
rejects anything else. A plotters-svg upgrade that changes that form fails
the tests instead of producing wrong text.

## Layout and typography

### Metrics

The outlines use the metrics Plotters lays the chart out with. Those are
also the metrics of the PNG and of `estimate_text_size`, which sizes the
legend column.

- **Size.** plotters-svg writes `font-size` as the Plotters size divided by
  1.24 (`EM_RATIO`). The converter multiplies it back once and scales the
  font the way ab_glyph does: Plotters size over ascent minus descent,
  `(2189 + 600) / 2048` em for Open Sans.
- **Anchor box.** Horizontal advances plus ab_glyph's kerning, truncated to
  whole pixels, with the height set to the CSS size. The anchors are those
  of `plotters-backend`'s default `draw_text`, and the baseline sits half
  the Plotters size below the top of the box.
- **Glyphs.** Each glyph keeps its left side bearing and its sub-pixel pen
  position. Path coordinates are written to 1/100 px.

Because every label is laid out with the same metrics that reserved its
room, the painted text fits that room. `test_outlines_fit_the_reserved_legend_column`
checks this, and `test_outlines_labels_never_overlap` finds no overlapping
label boxes in any case.

### Kerning and shaping

The outlines kern nothing. ab_glyph reads only the legacy `kern` table, and
the bundled Open Sans 3.003 has none: its tables are `GDEF GPOS GSUB OS/2
cmap … glyf …`, so its kerning lives in GPOS.
`test_bundled_font_kerns_nothing_through_ab_glyph` measures a zero
adjustment for `AV`, `To`, `Te`, `Yo`, `LT`, `P5` and `VA` at the size
Plotters uses. The PNG therefore has no kerning either.

There is also no shaping: one glyph per character and no ligatures, so `fi`
stays two glyphs. Browsers shape `<text>` with GPOS kerning and GSUB.

### Whitespace

A tab, line feed or carriage return advances like one space, on the same
line, and the accessible name reads it as a space. Plotters has no line
breaks, and the PNG and every viewer also draw these characters as
horizontal space.

This policy is needed for real data: the payoff fixture's title, built by
`BullCallSpread::graph_config`, contains `\n\t`, meant as line breaks that
no backend draws. Before the policy, the outlines mode rejected that title
with `MissingGlyph('\n')`. Any other character without a glyph is still an
error, for example U+0007 or `中`.

### Differences from the PNG

These comparisons are for information; the PNG is not the reference.

| Source of the difference | Effect |
|---|---|
| Plotters' ab_glyph rasteriser drops each glyph's left side bearing and truncates pen positions to whole pixels | Uneven gaps in the PNG: `Un derlying`, `SP 500`, `Europ ean`. The outlines use the same advances without kerning and are evenly spaced. |
| The PNG rotates text pixel by pixel | Rotated titles land 1 to 2 px off |
| The PNG advances `\n` and `\t` by the `.notdef` advance, which is wider than a space | The payoff title differs after its first `\n` |
| Other geometry crossing a label | Lines are rasterised differently. The worst "edge" in the table, 5 px for `-52.66`, is the current-price line crossing that label, not the text. |

Label by label, the outlines renders and the PNG agree on 14/19, 19/22 and
18/22 labels for the three charts in every engine, and on 20/23 (Blink) or
19/23 (WebKit, librsvg) for the text fixture. The other labels are the longer titles and the
rotated ones, explained by the table above.

### Differences from the embedded-font SVG

- **Size.** Browsers draw `<text>` at `font-size` CSS pixels per em. The
  layout measured an em of `1.24 × 2048 / 2789 ≈ 0.91` of that, so the
  embedded-font text is about 9.8 % larger than the room the layout gave
  it. This is a new finding, not noted in #920. The outlines do not have
  this problem.
- **Vertical position.** The `<text>` baseline comes from `dy` in em and ex
  units, not from the layout box.
- **Shaping and whitespace.** Browsers apply GPOS kerning and GSUB, and
  collapse whitespace, so leading spaces disappear.
- **Rasterisation.** WebKit visibly darkens stems.

Almost no label of the embedded-font SVG matches its outlines render: none
in any case or engine, except one of the text fixture's 23 labels in Blink
(see the second table below). This is the expected size and metric
difference, not a defect of either mode.

### Corrections to the #920 report

- *"Text in PNG: ab_glyph rasterisation: readable, but no kerning, so some
  glyph pairs (`52.66`, `SP 500`) look loosely spaced."* The cause is wrong.
  The PNG does have no kerning, but the loose pairs come from the
  rasteriser dropping side bearings and truncating pen positions: outlines
  with the same advances and no kerning are evenly spaced. REPORT.md now
  says so.
- *"Plotters, embedded font: Open Sans in Blink and WebKit."* This
  still holds with the exact-size WebKit renderer described below; the crop
  sheets show Open Sans in both. The label comparison adds one detail: the
  two engines do not draw that same font identically (WebKit's stem
  darkening, and their own shaping). Only 3 to 10 labels per case match
  between them, while the outlines match 100 %.
- *WebKit via Quick Look.* The #920 renders came from a Quick Look
  thumbnail, which is a square, rescaled crop of the SVG; the #923 text
  fixture's legend fell outside it. #923 replaces it with a
  `WKWebView` snapshot at the SVG's own size.

## Evidence

### Viewer environment

Recorded in `artifacts/outlines/viewers/viewers.txt`. Rendered on
2026-10-18 on macOS 27.0.1 arm64.

| Item | Value |
|---|---|
| Blink | Chrome 155.0.8059.85, headless, device scale 1 |
| WebKit | `WKWebView` 22625.1.29.11.28 (Safari 27.0.1) |
| librsvg | `rsvg-convert` 2.63.3 (cairo 1.18.6, pango 1.58.2, harfbuzz 14.6.0, fontconfig 2.18.3) |
| Bundled font | SHA-256 `c53aceea…f8eb59` |
| Open Sans installed | absent from the system (`system_profiler`: 0) and from fontconfig (`fc-list`: 0) |

Each engine ran with its own isolation:

- **Chrome:** a fresh profile per render, deleted afterwards; every host
  name maps to nothing; an unreachable proxy.
- **librsvg:** `sandbox-exec` with all network access denied.
- **WebKit:** a non-persistent data store, and a content rule list that
  blocks every `http`, `https`, `ws`, `wss` and `ftp` URL.

The harness removes only what it created. It requires every output to
exist, be non-empty and have the SVG's size. All 36 renders passed
(`results.tsv`): 4 cases × 3 modes × 3 engines, which includes the 18
outputs #923 requires.

### Method and tolerance

Declared in `compare_viewers.py` before any result was looked at.

- **Regions.** Each label's ink box from `<case>.labels.json`, grown by
  3 px.
- **Placement.** The ink bounding box inside the region, taken as pixels
  darker than luminance 160, must agree within 1 px on all four edges.
- **Shape.** After a 1 px Gaussian blur, which absorbs the different
  antialiasing, the normalized cross-correlation must be at least 0.90.
- **Pass rule.** A label passes for a pair of engines when both hold. A
  mode is portable when every label of every case passes for all three
  pairs.
- **Negative control.** The embedded-font SVG in librsvg must fail;
  otherwise the method could not tell fonts apart and the result would be
  inconclusive.

### Results

Labels passing, out of the case's labels:

| Case | Mode | Blink and WebKit | Blink and librsvg | WebKit and librsvg |
|---|---|---|---|---|
| payoff | embedded font | 3/19 | 13/19 (NCC ≥ 0.49) | 12/19 |
| payoff | viewer font (control) | 3/19 | 19/19 | 18/19 |
| payoff | **outlines** | **19/19** (NCC ≥ 0.997) | **19/19** (≥ 0.996) | **19/19** (≥ 0.999) |
| curves | embedded font | 3/22 | 17/22 (NCC ≥ 0.52) | 11/22 |
| curves | viewer font (control) | 3/22 | 22/22 | 20/22 |
| curves | **outlines** | **22/22** (≥ 0.996) | **22/22** (≥ 0.996) | **22/22** (≥ 0.999) |
| surface | embedded font | 5/22 | 17/22 (NCC ≥ 0.64) | 11/22 |
| surface | viewer font (control) | 6/22 | 22/22 | 18/22 |
| surface | **outlines** | **22/22** (≥ 0.996) | **22/22** (≥ 0.997) | **22/22** (≥ 0.999) |
| text | embedded font | 10/23 | 13/23 (NCC ≥ 0.57) | 16/23 |
| text | viewer font (control) | 8/23 | 22/23 | 20/23 |
| text | **outlines** | **23/23** (≥ 0.996) | **23/23** (≥ 0.995) | **23/23** (≥ 0.999) |

- **Outlines are portable.** Every label passes in every pair, with ink
  edges within 1 px and a correlation of at least 0.995.
- **The method discriminates.** The embedded-font SVG fails in librsvg in
  every case, and the viewer-font control fails too.

Full tables, including the comparisons against the PNG and against the
embedded-font mode, are in `comparison.md` and `comparison.json`. Full-chart
renders of the embedded-font and outlines SVGs for every case and engine
are under `artifacts/outlines/viewers/<engine>/`. The crop sheets
`<case>-crops.png` show the title, the rotated axis title and the
right-most label (legend or colour bar) for each engine and mode, next to
the PNG.

### Values and geometry

`test_outlines_leave_every_non_text_element_unchanged` checks that the
outlines SVG, with its labels and title/desc removed, is byte-identical to
the viewer-font SVG with its `<text>` elements removed, for every case. So
the plotted values, units, series identity and surface geometry are the
same.

### Native export independence

The cold-export timing runs export each case to embedded-font and outlines
SVG 20 times each under `sandbox-exec`. The sandbox denies network access,
process creation, and reads of `/System/Library/Fonts`, `/Library/Fonts`
and `~/Library/Fonts`. All 160 runs succeeded, so the export needs no
browser, driver, network or installed font. Browsers are used only to view
the finished SVGs.

## Accessibility

Chrome's accessibility tree was read with `accessibility_tree.mjs`
(DevTools `Accessibility.getFullAXTree`, headless Chrome 155). The results
are in `results/accessibility/`. The script owns its browser as a process
group of its own: every step has a deadline, and it ends with
`Browser.close`, then SIGTERM and SIGKILL to that group, each with a bounded
wait, and removes the profile. `--self-test` checks this without a browser,
against a stand-in whose group ignores SIGTERM and a browser path that does
not exist.

| | Embedded-font SVG | Outlines SVG |
|---|---|---|
| Document name | empty | the chart title, from `<title>` |
| Document description | empty | the series and axes, from `<desc>` |
| Each label | `StaticText` with the label | `image` named by the label (`aria-label`) |

- **Kept.** All 19 payoff and 23 text-fixture labels are exposed with their
  text in both modes. The outlines add a chart name and description.
- **Name normalisation.** Chrome collapses whitespace in names:
  `"  AV To fi  "` is announced as `" AV To fi "`.
- **Lost.** Labels cannot be selected or copied, find-in-page does not find
  them, and they cannot be edited, because there are no `<text>` elements.
  A screen reader announces each label as an image rather than as text.
- **Not checked.** The WebKit tree and any screen reader were not checked;
  librsvg has no accessibility tree.
- The metadata keeps the strings readable in the source for inspection, but
  it does not make the chart as a whole accessible.

## Measurements

All on the same machine and commit (`ba77d074`), with identical inputs and
dimensions. Timings are in `results/timing-outlines.json`.

### SVG size

| Case | Embedded font: raw / gzip | Outlines: raw / gzip | Viewer font: raw / gzip |
|---|---|---|---|
| payoff | 205,891 / 105,699 | 100,938 / 34,068 | 9,073 / 1,859 |
| curves | 212,255 / 105,994 | 68,997 / 20,493 | 15,437 / 2,152 |
| surface | 234,533 / 108,266 | 93,739 / 23,479 | 37,715 / 4,256 |
| text | 206,029 / 105,052 | 71,495 / 24,232 | 9,211 / 1,308 |

- The outlines SVG is 2.0 to 3.1 times smaller than the embedded-font SVG
  raw, and 3.1 to 5.2 times smaller gzipped. It is larger than an SVG with
  no font.
- No glyph is reused through `<defs>`/`<use>`; each label is one inline
  path. Reuse or font subsetting would shrink it further, but neither is
  needed to decide this condition.

### Export time

Times in ms. Cold exports are a fresh process under the sandbox above, with
20 runs per row. In-process exports are 200 repeats in one process; the
statistics cover runs 2 to 200, and the first export is listed separately.

| Case | Cold, embedded: median / p90 | Cold, outlines: median / p90 | In-process, embedded: first / median / p90 | In-process, outlines: first / median / p90 |
|---|---|---|---|---|
| payoff | 21.5 / 22.7 | 22.7 / 28.8 | 0.50 / 0.21 / 0.37 | 1.60 / 1.20 / 1.57 |
| curves | 21.8 / 22.7 | 22.0 / 33.1 | 0.56 / 0.20 / 0.35 | 1.21 / 0.78 / 0.94 |
| surface | 21.6 / 22.2 | 25.6 / 45.8 | 0.50 / 0.33 / 0.45 | 1.12 / 0.89 / 1.13 |
| text | 22.6 / 28.7 | 22.0 / 28.9 | 0.41 / 0.23 / 0.45 | 1.99 / 0.85 / 1.09 |

- **Regressions, stated explicitly.** In-process, outlines take 0.55 to
  1.0 ms more per export (2.7 to 5.7 times the embedded-font time). Cold
  medians are within 4 ms, with a wider tail at p90.
- **For scale.** The Plotly export this would replace takes 1.1 to 2.0 s
  (REPORT.md).
- **Caching.** The bundled font is parsed once per process (`OnceLock`).
  Glyph outlines are rebuilt for every label and every export; nothing is
  cached.
- **Sandbox overhead.** These cold runs are slower than #920's 11 ms
  because `sandbox-exec` adds its own start-up. Both modes ran under the
  same conditions.

### Dependencies, binary and audit

- **Dependency tree.** `cargo tree -p osl-prototype-plotters-export -e
  normal` has the same 105 packages at `851899e8` and at `ba77d074`.
  `ab_glyph 0.2.32` is now a direct dependency, but it was already in the
  tree through `plotters/ab_glyph`. No asset or licence changed.
- **Binary size.** The release `compare` binary grows from 2,591,008 to
  2,648,944 bytes (+57,936, +2.2 %), built with the same toolchain
  (`rustc 1.99.1`).
- **Audit.** `cargo audit --deny warnings` is clean.
  `cargo tree --workspace --all-features -i ttf-parser` still reaches only
  the prototype, now by two paths to the same `ab_glyph`. The comment on the
  RUSTSEC-2026-0192 waiver in `.cargo/audit.toml` now lists both paths.
  The waiver stays justified only for the unpublished prototype and its
  bundled font; it does not cover user-supplied fonts or `ab_glyph` in a
  published crate.

## Tradeoffs

- **Text semantics.** Selection, copying, find-in-page and editing are
  lost, and labels are announced as images. The strings survive as
  accessible names and in the source.
- **Speed and size.** About 0.6 ms more per in-process SVG export. The file
  is larger than an SVG with no font, and smaller than one with the font
  embedded.
- **Typography.** No GPOS kerning and no ligatures, the same as the PNG.
  This is visible on pairs such as `AV` or `To` when compared with a
  browser's shaping.
- **Coverage.** The bundled font's coverage is the limit: a character
  outside it is an error in this mode only.
- **Maintenance.** The converter is tied to the exact output of
  plotters-svg 0.3.7, and any change there fails the tests. About 670
  lines with documentation, tests excluded.

## Acceptance criteria (#923)

| Criterion | Status | Evidence |
|---|---|---|
| Reproducible outputs or explicit failures for the three charts and the text fixture, with commands and provenance | **met** | `artifacts/outlines/`, `manifest.json`, commands below. The three charts' PNG, embedded-font SVG and viewer-font SVG are byte-identical to the #920 artifacts (`artifacts/<case>-plotters.*`), so they are not duplicated. |
| Outlines SVG independent of viewer fonts in all three engines, with consistent shapes and placement, no new overlap or clipping, and values unchanged | **met** | Results table; no font in the document; Open Sans absent and network blocked; `test_outlines_labels_never_overlap`; labels inside the figure; `test_outlines_leave_every_non_text_element_unchanged` |
| Typography differences explained; original report corrected where disproved | **met** | [Layout and typography](#layout-and-typography); REPORT.md's kerning sentence corrected |
| Strings and semantics retained; accessibility exposure checked; losses stated | **met** | [Accessibility](#accessibility). Checked in Chrome only; WebKit and screen readers were not checked. |
| Native export independence, size and latency, typed failures, regression evidence; no hidden substitution | **met** | Sandbox runs; measurement tables; `MissingGlyph` and `UnsupportedSvgText` tests; the outlines contain no `<text>`, `<image>`, `href`, `data:` or font reference (`test_outlines_every_case_paints_text_without_any_font`) |
| Dependency trees and audit compared with `851899e8`; the waiver is not approval for new paths | **met** | [Dependencies, binary and audit](#dependencies-binary-and-audit) |
| Rules and local gates; focused tests; viewer-font numeric-label assertions kept, outlines assertions separate | **met** | The PR's test plan. The `numeric_labels` assertions on `ViewerFont` are untouched, with separate `test_outlines_keep_distinct_small_tick_labels` and `test_outlines_keep_a_tiny_series_scale`. |
| Report concludes completion, portability and next step separately | **met** | [Conclusions](#svg-text-as-outlines-follow-up-report-923) |

## Reproduce

From the repository root, on macOS with Chrome, `rsvg-convert` and Xcode's
`swiftc`. `compare_viewers.py` needs numpy and Pillow, and
`accessibility_tree.mjs` needs Node 22+.

```sh
cargo build --release -p osl-prototype-plotters-export --bin compare
target/release/compare outlines /tmp/osl-923
prototypes/plotters-export/render_svg_viewers.sh /tmp/osl-923 /tmp/osl-923-viewers
python3 prototypes/plotters-export/compare_viewers.py /tmp/osl-923 /tmp/osl-923-viewers /tmp/osl-923-comparison
node prototypes/plotters-export/accessibility_tree.mjs /tmp/osl-923/text.outlines.svg
node prototypes/plotters-export/accessibility_tree.mjs --self-test   # no browser needed
python3 -I prototypes/plotters-export/measure.py --native target/release/compare \
  --backends plotters --cases payoff,curves,surface,text --formats svg,svg-outlines \
  --cold 20 --warm 200 --sandbox --out /tmp/osl-923-timing --name timing-outlines
cargo bench -p osl-prototype-plotters-export --bench native_export
```
