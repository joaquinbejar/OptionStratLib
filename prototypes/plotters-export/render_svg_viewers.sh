#!/usr/bin/env bash
# Renders every SVG artifact of #920 in three independent SVG engines, to
# check how portable the text is:
#
#   chrome  Blink, through a headless Chrome screenshot (BROWSER_PATH)
#   rsvg    librsvg, through rsvg-convert (Pango / fontconfig text)
#   webkit  WebKit, through a macOS Quick Look thumbnail (qlmanage)
#
# Run on a machine without Open Sans installed, so a viewer that ignores the
# embedded @font-face falls back to its own font and the difference shows.
#
# Usage: render_svg_viewers.sh <artifact dir> <output dir>
set -euo pipefail

src=${1:?artifact dir}
out=${2:?output dir}
browser=${BROWSER_PATH:-/Applications/Google Chrome.app/Contents/MacOS/Google Chrome}
mkdir -p "$out/chrome" "$out/rsvg" "$out/webkit"

{
  echo "chrome: $("$browser" --version)"
  echo "rsvg: $(rsvg-convert --version)"
  echo "webkit: $(sw_vers -productName) $(sw_vers -productVersion) Quick Look"
  echo "open sans installed: $(system_profiler SPFontsDataType 2>/dev/null | grep -c 'Open Sans' || true)"
} > "$out/viewers.txt"

for svg in "$src"/*.svg; do
  name=$(basename "$svg" .svg)
  size=$(grep -o 'width="[0-9]*" height="[0-9]*"' "$svg" | head -1)
  width=$(sed -E 's/width="([0-9]+)".*/\1/' <<< "$size")
  height=$(sed -E 's/.*height="([0-9]+)"/\1/' <<< "$size")
  "$browser" --headless=new --disable-gpu --hide-scrollbars \
    --user-data-dir="$(mktemp -d)" --window-size="$width,$height" \
    --screenshot="$out/chrome/$name.png" "file://$svg" > /dev/null 2>&1
  rsvg-convert -w "$width" -h "$height" -b white "$svg" -o "$out/rsvg/$name.png"
  qlmanage -t -s "$width" -o "$out/webkit" "$svg" > /dev/null 2>&1
  mv "$out/webkit/$name.svg.png" "$out/webkit/$name.png" 2> /dev/null || true
done
echo "rendered $(ls "$src"/*.svg | wc -l | tr -d ' ') SVGs into $out"
