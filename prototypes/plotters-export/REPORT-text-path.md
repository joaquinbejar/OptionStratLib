# A text path without ttf-parser (condition 4 of #920)

`ttf-parser` is unmaintained (RUSTSEC-2026-0192). The owner decided on
2026-10-19 that it must not enter a published crate, and that a text path
without it has to be found before the PNG/SVG export migrates. This report
is that investigation, run in the unpublished prototype. The baseline is
`cfa2ee6b` (PR #924).

**Recommendation.** Read the bundled font with
[skrifa](https://crates.io/crates/skrifa), the replacement the advisory
itself names. It is part of Google Fonts' fontations project, licensed MIT
or Apache-2.0, at version 0.48. Build Plotters with no font feature at all,
and let a backend wrapper measure and draw every label.

- **ttf-parser is gone.** `cargo tree --workspace --all-features -i
  ttf-parser` and `-i ab_glyph` both find nothing. The RUSTSEC-2026-0192
  waiver is removed from `.cargo/audit.toml`, and `cargo audit --deny
  warnings` is clean.
- **The layout does not move.** The viewer-font and embedded-font SVGs of
  all four charts are byte-identical to the committed #920 and #923
  artifacts. A test now fails if they ever differ.
- **The PNG text gets better.** Glyphs keep their side bearings and
  sub-pixel positions. PNG text and the outlines SVG now agree on 85 of 86
  labels, against 71 or 72 before.
- **The outlines SVG keeps the same geometry.** It also drops the empty
  subpaths ab_glyph left behind, about 1 KB less per chart.
- **Cost.** 8 more packages in the prototype's tree (113 instead of 105),
  and +0.66 MB (+25 %) on the release binary. Export times did not change.

## How it works

Plotters measures text only through `DrawingBackend::estimate_text_size`
(chart titles and the mesh's axis labels) and draws it only through
`DrawingBackend::draw_text`. The one exception is `MultiLineText`, which
the prototype does not use. Without a font feature, Plotters falls back to
its `naive` font: crude estimates, and a panic if it is ever asked to draw.
A wrapper backend therefore takes over both methods:

- **`src/text.rs`** reads the character map, advances, vertical metrics,
  glyph boxes and outlines with skrifa. It lays labels out exactly as
  Plotters' `ab_glyph` text did:
  - the Plotters size is ab_glyph's `PxScale` of `ascent - descent` font
    units, in `f32`;
  - the anchor box is the advance width truncated to whole pixels, with the
    size divided by `EM_RATIO` as its height;
  - the baseline is half the size below the box;
  - there is no kerning: the bundled Open Sans keeps its kerning in GPOS
    only, which ab_glyph never applied either.
- **`src/backend.rs`** (`TextBackend`) wraps the bitmap and SVG backends and
  delegates everything except text.
  - **Measuring** gives an unmapped character the `.notdef` advance, as
    ab_glyph did, so viewer-font SVGs with characters outside the font
    still lay out the same.
  - **PNG text** is filled from the outlines with `ab_glyph_rasterizer` (no
    font parsing; already in the tree), with Plotters' quarter-turn
    rotations.
  - **SVG text** still goes to plotters-svg as `<text>`, which needs no font
    data. The outline conversion (`src/outline.rs`) or the viewer draws it.
  - A character without a glyph in PNG or outlines is a typed
    `ExportError::MissingGlyph`. It is stashed by the wrapper because
    Plotters carries font errors only as strings. Before this change the PNG
    silently drew the `.notdef` box.

## Evidence

### skrifa reads the same font

A scratch comparison checked all 1010 mapped characters of the bundled Open
Sans against ab_glyph. Glyph ids, unscaled advances, ascent and descent are
identical. The outline command lists are identical once zero-length lines
are removed: ab_glyph closes each contour with a zero-length line, and
skrifa emits one zero-length line in `ę`.

### Layout and outputs

All against the committed artifacts:

| Output | Result |
|---|---|
| Viewer-font and embedded-font SVG, 4 cases | byte-identical (`test_layout_matches_the_920_artifacts_byte_for_byte` for the three charts) |
| Label ink boxes (`<case>.labels.json`), 4 cases | byte-identical |
| Outlines SVG, 4 cases | identical after removing ab_glyph's empty `M x yZ` subpaths; 0.9 to 1.6 KB smaller |
| PNG, non-text pixels | identical for curves, surface and text. In payoff, 106 pixels differ outside the label boxes: its title advances `\n\t` like spaces now (as the outlines do) rather than like `.notdef`. |

### Viewers

The SVGs were rendered again in Blink, WebKit and librsvg with the #923
harness and method; 36 of 36 renders passed. The results are in
`artifacts/text-path/`.

- **Outlines portability holds.** 86/86 labels match in every pair of
  engines, with NCC ≥ 0.995. The method still rejects the embedded-font SVG
  in librsvg.
- **PNG text against the outlines renders:**

  | Case | Blink | WebKit | librsvg | Before (#923, Blink) |
  |---|---|---|---|---|
  | payoff | 19/19 (NCC ≥ 0.982) | 19/19 | 19/19 | 14/19 (NCC ≥ 0.615) |
  | curves | 22/22 (NCC ≥ 0.996) | 22/22 | 22/22 | 19/22 |
  | surface | 21/22 (NCC ≥ 0.959) | 21/22 | 21/22 | 18/22 |
  | text | 23/23 (NCC ≥ 0.987) | 23/23 | 23/23 | 20/23 |

  The remaining surface label is the 3D z-axis title, where the surface's
  cells behind it shift the ink box by 3 px. Its NCC passes.
  `png-text-before-after.png` shows the change: no more `Un derlying` or
  `SP 500`, and `Δ per $1` instead of `Δper`.

### Cost

**Dependencies.** In the prototype's normal tree, ab_glyph,
owned_ttf_parser and ttf-parser leave. These arrive: skrifa, read-fonts,
font-types, foldhash, stable_deref_trait, yoke and zerofrom, plus four
proc-macro crates (bytemuck_derive, yoke-derive, zerofrom-derive,
synstructure).

**Binary.** The release `compare` binary grows from 2,649,728 to
3,311,600 bytes (+661,872, +25 %). Turning off skrifa's default
`autohint_shaping` feature does not change the size; it stays off because
nothing uses it. The size is skrifa's outline code. The alternative below
would avoid it.

**Time.** Measured under the sandbox with network, process creation and
font directories denied; 160 runs per binary, all succeeded. Results are in
`results/timing-text-path-{base,head}.json`. The head binary is the code of
this branch; the JSON's `git` field shows `547649db`, and only the skrifa
feature flag changed after it.

- **In process.** Medians differ by at most 0.44 ms:
  - PNG: 1.2 to 3.0 ms before, 1.3 to 3.0 ms after;
  - outlines SVG: +0.01 to +0.08 ms;
  - embedded-font SVG: unchanged.
- **Cold, interleaved base and head runs (40 each).** Medians within
  0.6 ms: PNG 18.9 and 19.3 ms; outlines SVG 17.4 ms both; embedded-font
  SVG 15.4 and 16.0 ms.

## Alternatives considered

| Option | Verdict |
|---|---|
| **skrifa + text backend** (this) | Recommended: maintained, named by the advisory, reads any glyph of the font, no layout change |
| Pre-baked glyph tables: generate the outlines and advances of a fixed character set into Rust source, with no font parser in the crate | Smallest binary. But the character set is fixed (anything else becomes `MissingGlyph`), a generator has to be maintained, and the generated data is a derivative of the font under the OFL. Worth revisiting only if the 0.66 MB matters. |
| Keep `ab_glyph` with a published-crate waiver | Rejected by the owner |
| fontdue, rusttype, stb_truetype | fontdue parses with ttf-parser; rusttype and stb_truetype carry their own RustSec advisories |
| Plotters' `ttf` feature | font-kit and fontconfig, and ttf-parser again |

## What the migration inherits

- `TextBackend`, `text.rs` and the outline conversion move into
  `optionstratlib-visualization` behind `static_export`. Plotters is built
  with no font feature, plus `skrifa` (without default features) and
  `ab_glyph_rasterizer`. Recheck `make check-feature-trees` and the audit
  there.
- **`MissingGlyph` becomes part of the PNG contract** as well as the
  outlines contract. Today a CJK title in a PNG is a typed error, not a
  `.notdef` box.
- **Plotters 0.3.7 is pinned tightly.** The layout is locked by the
  byte-identical test, and the outline conversion by its strict parser.
  Either fails loudly if a Plotters upgrade changes text handling.

## Reproduce

```sh
cargo tree --workspace --all-features -i ttf-parser   # nothing
cargo audit --deny warnings
cargo test -p osl-prototype-plotters-export
cargo build --release -p osl-prototype-plotters-export --bin compare
target/release/compare outlines /tmp/osl-text-path
prototypes/plotters-export/render_svg_viewers.sh /tmp/osl-text-path /tmp/osl-text-path-viewers
python3 prototypes/plotters-export/compare_viewers.py /tmp/osl-text-path /tmp/osl-text-path-viewers /tmp/osl-text-path-comparison
python3 -I prototypes/plotters-export/measure.py --native target/release/compare \
  --backends plotters --cases payoff,curves,surface,text --formats png,svg,svg-outlines \
  --cold 20 --warm 200 --sandbox --out /tmp/osl-text-path-timing --name timing-text-path
```
