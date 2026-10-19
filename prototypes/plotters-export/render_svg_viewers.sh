#!/usr/bin/env bash
# Renders every SVG of a directory in three independent SVG engines, to
# check how portable the text is (#920, #923):
#
#   chrome  Blink: a headless Chrome screenshot (BROWSER_PATH) at device
#           scale 1, with a fresh profile per render that is deleted
#           afterwards, every host name resolving to nothing and an
#           unreachable proxy, so no request can leave the machine.
#   rsvg    librsvg: rsvg-convert (Pango / fontconfig text) under
#           sandbox-exec with all network access denied.
#   webkit  WebKit: a WKWebView snapshot (webkit_snapshot.swift) with a
#           non-persistent data store and every network URL blocked.
#           #920 used a Quick Look thumbnail instead, which renders a
#           square, rescaled crop of the SVG; #923 replaced it.
#
# Run on a machine without Open Sans installed, so a viewer that ignores the
# embedded @font-face falls back to its own font and the difference shows;
# viewers.txt records the check.
#
# Every expected output must exist, be non-empty and have the SVG's size.
# results.tsv lists each render with its status, and the script exits
# non-zero when any render failed. It removes only what it created.
#
# Usage: render_svg_viewers.sh <svg dir> <output dir>
set -euo pipefail

src=$(cd "${1:?svg dir}" && pwd)
out=${2:?output dir}
here=$(cd "$(dirname "$0")" && pwd)
browser=${BROWSER_PATH:-/Applications/Google Chrome.app/Contents/MacOS/Google Chrome}
font="$here/assets/OpenSans-Regular.ttf"
mkdir -p "$out/chrome" "$out/rsvg" "$out/webkit"
out=$(cd "$out" && pwd)

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
swiftc -O "$here/webkit_snapshot.swift" -o "$work/webkit_snapshot"

{
  echo "date: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "machine: $(sw_vers -productName) $(sw_vers -productVersion) $(uname -m)"
  echo "chrome: $("$browser" --version)"
  echo "rsvg: $(rsvg-convert --version | head -1)"
  echo "webkit: WKWebView, $(defaults read /System/Library/Frameworks/WebKit.framework/Resources/Info.plist CFBundleVersion 2>/dev/null || echo unknown) (Safari $(defaults read /Applications/Safari.app/Contents/Info.plist CFBundleShortVersionString 2>/dev/null || echo unknown))"
  echo "swiftc: $(swiftc --version 2>&1 | head -1)"
  echo "bundled font sha256: $(shasum -a 256 "$font" | cut -d' ' -f1)"
  echo "open sans in system_profiler: $(system_profiler SPFontsDataType 2>/dev/null | grep -c 'Open Sans' || true)"
  echo "open sans in fontconfig: $(fc-list 2>/dev/null | grep -Eci 'open ?sans' || true)"
  echo "chrome command: <browser> --headless=new --disable-gpu --hide-scrollbars --force-device-scale-factor=1 --no-first-run --no-default-browser-check --disable-background-networking --disable-component-update --disable-sync --host-resolver-rules='MAP * ~NOTFOUND' --proxy-server=127.0.0.1:9 --user-data-dir=<fresh dir> --window-size=<w>,<h> --screenshot=<png> file://<svg>"
  echo "rsvg command: sandbox-exec -p '(version 1)(allow default)(deny network*)' rsvg-convert -w <w> -h <h> -b white <svg> -o <png>"
  echo "webkit command: webkit_snapshot <svg> <w> <h> <png>"
} > "$out/viewers.txt"

printf 'svg\tengine\tstatus\twidth\theight\tbytes\n' > "$out/results.tsv"
failures=0

# Records one render: ok only when the PNG exists, is non-empty and has the
# expected size.
check() {
  local name=$1 engine=$2 png=$3 width=$4 height=$5 ran=$6
  local status=ok got_width=- got_height=- bytes=0
  if [[ -s $png ]]; then
    bytes=$(stat -f %z "$png")
    got_width=$(sips -g pixelWidth "$png" | awk '/pixelWidth/ {print $2}')
    got_height=$(sips -g pixelHeight "$png" | awk '/pixelHeight/ {print $2}')
    [[ $got_width == "$width" && $got_height == "$height" ]] || status=wrong-size
  else
    status=missing
  fi
  [[ $ran == 0 ]] || status="failed($ran):$status"
  [[ $status == ok ]] || failures=$((failures + 1))
  printf '%s\t%s\t%s\t%s\t%s\t%s\n' "$name" "$engine" "$status" "$got_width" "$got_height" "$bytes" >> "$out/results.tsv"
}

for svg in "$src"/*.svg; do
  name=$(basename "$svg" .svg)
  size=$(grep -o 'width="[0-9]*" height="[0-9]*"' "$svg" | head -1)
  width=$(sed -E 's/width="([0-9]+)".*/\1/' <<< "$size")
  height=$(sed -E 's/.*height="([0-9]+)"/\1/' <<< "$size")

  profile=$(mktemp -d "$work/chrome.XXXXXX")
  ran=0
  "$browser" --headless=new --disable-gpu --hide-scrollbars --force-device-scale-factor=1 \
    --no-first-run --no-default-browser-check --disable-background-networking \
    --disable-component-update --disable-sync \
    --host-resolver-rules='MAP * ~NOTFOUND' --proxy-server=127.0.0.1:9 \
    --user-data-dir="$profile" --window-size="$width,$height" \
    --screenshot="$out/chrome/$name.png" "file://$svg" > /dev/null 2>&1 || ran=$?
  rm -rf "$profile"
  check "$name" chrome "$out/chrome/$name.png" "$width" "$height" "$ran"

  ran=0
  sandbox-exec -p '(version 1)(allow default)(deny network*)' \
    rsvg-convert -w "$width" -h "$height" -b white "$svg" -o "$out/rsvg/$name.png" || ran=$?
  check "$name" rsvg "$out/rsvg/$name.png" "$width" "$height" "$ran"

  ran=0
  "$work/webkit_snapshot" "$svg" "$width" "$height" "$out/webkit/$name.png" || ran=$?
  check "$name" webkit "$out/webkit/$name.png" "$width" "$height" "$ran"
done

total=$(($(wc -l < "$out/results.tsv") - 1))
echo "rendered $total outputs into $out; $failures failed"
[[ $failures == 0 ]]
