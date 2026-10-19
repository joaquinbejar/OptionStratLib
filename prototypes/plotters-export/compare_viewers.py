#!/usr/bin/env python3
"""Compares the viewer renders of #923 label by label.

Method and tolerance, fixed before any result was looked at:

* Inputs: for every case of ``compare outlines``, the SVG text modes
  ``embedded-font`` (``<case>.svg``), ``viewer-font`` (``<case>.plain.svg``,
  the negative control) and ``outlines`` (``<case>.outlines.svg``), each
  rendered by Blink, WebKit and librsvg at the SVG's own size by
  ``render_svg_viewers.sh``, plus the native PNG. Every image is composited
  on white and reduced to luminance.
* Regions: each label's ink box from ``<case>.labels.json`` (the outline
  layout, which is where the chart's layout put the text), grown by
  ``MARGIN`` pixels on every side.
* Placement: the ink bounding box inside the region (pixels darker than
  ``INK``) must agree within ``EDGE_TOLERANCE`` pixels on all four edges.
* Shape: the regions of the images blurred with a ``BLUR`` pixel Gaussian,
  which absorbs the rasterisers' different antialiasing, must have a
  normalized cross-correlation of at least ``MIN_NCC``.
* A label passes for a pair of engines when both hold. A mode is portable
  when every label of every case passes for all three pairs. An output that
  is missing or has the wrong size fails every label it should show.
* The embedded-font SVG in librsvg, which ignores ``@font-face``, must fail:
  if it passed, the method could not tell fonts apart and the result would
  be inconclusive. That check is reported as ``discriminates``.

The same metrics are also reported, for information only, between each
engine's outlines render and the native PNG, and between the embedded-font
and outlines renders of each engine.

Usage: compare_viewers.py <artifact dir> <viewer dir> <output dir>

Writes ``comparison.json``, ``comparison.md`` and one crop sheet per case
(``<case>-crops.png``: title, rotated axis title and right-most label, per
engine and mode) into the output directory. Needs numpy and Pillow.
"""

from __future__ import annotations

import json
import math
import sys
from pathlib import Path

import numpy as np
from PIL import Image, ImageDraw, ImageFilter

MARGIN = 3
INK = 160
EDGE_TOLERANCE = 1
BLUR = 1.0
MIN_NCC = 0.90

ENGINES = ("chrome", "webkit", "rsvg")
PAIRS = (("chrome", "webkit"), ("chrome", "rsvg"), ("webkit", "rsvg"))
MODES = (("embedded-font", ""), ("viewer-font", ".plain"), ("outlines", ".outlines"))
CROP_SCALE = 3


def load(path: Path, size: tuple[int, int]) -> tuple[np.ndarray, np.ndarray] | str:
    """Luminance and blurred luminance of ``path``, or why it is unusable."""
    if not path.is_file() or path.stat().st_size == 0:
        return "missing"
    image = Image.open(path).convert("RGBA")
    if image.size != size:
        return f"size {image.size[0]}x{image.size[1]}, expected {size[0]}x{size[1]}"
    white = Image.new("RGBA", image.size, (255, 255, 255, 255))
    gray = Image.alpha_composite(white, image).convert("L")
    blurred = gray.filter(ImageFilter.GaussianBlur(BLUR))
    return np.asarray(gray, dtype=np.float64), np.asarray(blurred, dtype=np.float64)


def region(label: dict, size: tuple[int, int]) -> tuple[int, int, int, int]:
    width, height = size
    return (
        max(0, math.floor(label["left"]) - MARGIN),
        max(0, math.floor(label["top"]) - MARGIN),
        min(width, math.ceil(label["right"]) + MARGIN),
        min(height, math.ceil(label["bottom"]) + MARGIN),
    )


def ink_box(gray: np.ndarray, box: tuple[int, int, int, int]) -> tuple[int, int, int, int] | None:
    left, top, right, bottom = box
    ys, xs = np.nonzero(gray[top:bottom, left:right] < INK)
    if xs.size == 0:
        return None
    return int(xs.min()), int(ys.min()), int(xs.max()), int(ys.max())


def ncc(a: np.ndarray, b: np.ndarray) -> float:
    a = a - a.mean()
    b = b - b.mean()
    denominator = math.sqrt(float((a * a).sum()) * float((b * b).sum()))
    if denominator == 0.0:
        return 1.0 if not a.any() and not b.any() else 0.0
    return float((a * b).sum()) / denominator


def compare_label(first, second, label: dict, size: tuple[int, int]) -> dict:
    box = region(label, size)
    left, top, right, bottom = box
    (gray_a, blur_a), (gray_b, blur_b) = first, second
    ink_a, ink_b = ink_box(gray_a, box), ink_box(gray_b, box)
    if ink_a is None or ink_b is None:
        edge = math.inf
    else:
        edge = max(abs(p - q) for p, q in zip(ink_a, ink_b))
    correlation = ncc(blur_a[top:bottom, left:right], blur_b[top:bottom, left:right])
    return {
        "text": label["text"],
        "edge": edge,
        "ncc": round(correlation, 4),
        "passed": edge <= EDGE_TOLERANCE and correlation >= MIN_NCC,
    }


def compare_images(first, second, labels: list[dict], size) -> dict:
    """Every label of one image pair; an unusable image fails them all."""
    for image in (first, second):
        if isinstance(image, str):
            return {"passed": 0, "labels": len(labels), "unusable": image, "failures": []}
    results = [compare_label(first, second, label, size) for label in labels]
    failures = [result for result in results if not result["passed"]]
    finite = [result["edge"] for result in results if result["edge"] != math.inf]
    return {
        "passed": len(results) - len(failures),
        "labels": len(results),
        "worst_edge": max(finite) if len(finite) == len(results) else "no ink",
        "worst_ncc": min(result["ncc"] for result in results),
        "failures": failures,
    }


def sheet(case: str, labels: list[dict], images: dict, png, size, path: Path) -> None:
    """Crops of the title, the most rotated label and the right-most label."""
    picks = [labels[0]]
    rotated = max(labels, key=lambda l: (l["bottom"] - l["top"]) / max(1e-9, l["right"] - l["left"]))
    if rotated is not labels[0]:
        picks.append(rotated)
    rightmost = max(labels, key=lambda l: l["right"])
    if all(rightmost is not pick for pick in picks):
        picks.append(rightmost)
    rows = [("native png", None)] + [
        (f"{engine} {mode}", (engine, mode)) for mode, _ in MODES for engine in ENGINES
    ]
    boxes = [region(label, size) for label in picks]
    widths = [(box[2] - box[0]) * CROP_SCALE for box in boxes]
    heights = [(box[3] - box[1]) * CROP_SCALE for box in boxes]
    caption = 150
    canvas = Image.new("L", (caption + sum(widths) + 10 * len(boxes), (max(heights) + 6) * len(rows)), 255)
    draw = ImageDraw.Draw(canvas)
    for row, (name, key) in enumerate(rows):
        y = row * (max(heights) + 6)
        draw.text((4, y + 4), name, fill=0)
        source = png if key is None else images.get(key)
        x = caption
        for box, width, height in zip(boxes, widths, heights):
            if isinstance(source, tuple):
                crop = Image.fromarray(source[0].astype(np.uint8)).crop(box)
                canvas.paste(crop.resize((width, height), Image.NEAREST), (x, y))
            else:
                draw.text((x + 4, y + 4), str(source or "missing"), fill=0)
            x += width + 10
    canvas.save(path)


def main(artifacts: Path, viewers: Path, out: Path) -> int:
    manifest = json.loads((artifacts / "manifest.json").read_text())
    cases = list(dict.fromkeys(entry["case"] for entry in manifest))
    sizes = {entry["case"]: (entry["width"], entry["height"]) for entry in manifest}
    out.mkdir(parents=True, exist_ok=True)
    report: dict = {
        "method": {
            "margin_px": MARGIN,
            "ink_luminance_below": INK,
            "edge_tolerance_px": EDGE_TOLERANCE,
            "blur_px": BLUR,
            "min_ncc": MIN_NCC,
        },
        "cases": {},
    }
    portable = {mode: True for mode, _ in MODES}
    for case in cases:
        size = sizes[case]
        labels = json.loads((artifacts / f"{case}.labels.json").read_text())
        png = load(artifacts / f"{case}.png", size)
        images = {
            (engine, mode): load(viewers / engine / f"{case}{suffix}.png", size)
            for mode, suffix in MODES
            for engine in ENGINES
        }
        entry: dict = {"labels": len(labels), "modes": {}, "versus_png": {}, "embedded_vs_outlines": {}}
        for mode, _ in MODES:
            pairs = {}
            for a, b in PAIRS:
                result = compare_images(images[(a, mode)], images[(b, mode)], labels, size)
                pairs[f"{a}-{b}"] = result
                if result["passed"] != result["labels"]:
                    portable[mode] = False
            entry["modes"][mode] = pairs
        for engine in ENGINES:
            entry["versus_png"][engine] = compare_images(images[(engine, "outlines")], png, labels, size)
            entry["embedded_vs_outlines"][engine] = compare_images(
                images[(engine, "embedded-font")], images[(engine, "outlines")], labels, size
            )
        report["cases"][case] = entry
        sheet(case, labels, images, png, size, out / f"{case}-crops.png")

    discriminates = all(
        report["cases"][case]["modes"]["embedded-font"]["chrome-rsvg"]["passed"]
        < report["cases"][case]["modes"]["embedded-font"]["chrome-rsvg"]["labels"]
        for case in cases
    )
    report["portable"] = portable
    report["discriminates"] = discriminates
    (out / "comparison.json").write_text(json.dumps(report, indent=2, ensure_ascii=False) + "\n")

    lines = [
        "| case | mode | Blink-WebKit | Blink-librsvg | WebKit-librsvg |",
        "|---|---|---|---|---|",
    ]
    for case in cases:
        for mode, _ in MODES:
            cells = []
            for a, b in PAIRS:
                result = report["cases"][case]["modes"][mode][f"{a}-{b}"]
                if "unusable" in result:
                    cells.append(f"unusable: {result['unusable']}")
                else:
                    cells.append(
                        f"{result['passed']}/{result['labels']} (edge ≤ {result['worst_edge']} px, "
                        f"NCC ≥ {result['worst_ncc']:.3f})"
                    )
            lines.append(f"| {case} | {mode} | " + " | ".join(cells) + " |")
    lines += [
        "",
        f"Portable (every label, every pair): {json.dumps(portable)}",
        f"Method discriminates (embedded-font fails in librsvg on every case): {discriminates}",
        "",
        "| case | outlines vs native PNG: Blink | WebKit | librsvg | embedded vs outlines: Blink | WebKit | librsvg |",
        "|---|---|---|---|---|---|---|",
    ]
    for case in cases:
        cells = []
        for key in ("versus_png", "embedded_vs_outlines"):
            for engine in ENGINES:
                result = report["cases"][case][key][engine]
                cells.append(
                    f"unusable: {result['unusable']}"
                    if "unusable" in result
                    else f"{result['passed']}/{result['labels']} (NCC ≥ {result['worst_ncc']:.3f})"
                )
        lines.append(f"| {case} | " + " | ".join(cells) + " |")
    (out / "comparison.md").write_text("\n".join(lines) + "\n")
    print("\n".join(lines))
    return 0


if __name__ == "__main__":
    if len(sys.argv) != 4:
        print(__doc__.split("\n\n")[-2], file=sys.stderr)
        sys.exit(2)
    sys.exit(main(Path(sys.argv[1]), Path(sys.argv[2]), Path(sys.argv[3])))
