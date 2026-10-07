#!/usr/bin/env bash
# Build each direct-component example (#555) against the packaged component
# crates, the way a consumer gets them once they are published.
#
# For every `examples/direct/<scenario>` this copies the manifest and sources
# out of the repository into a scratch directory, removes the `path = "..."`
# from the OptionStratLib dependencies (leaving `version = "0.22.0"`, the
# registry form), and adds a `[patch.crates-io]` that points every component
# at the unpacked `.crate` file `cargo package` produced (the ones
# `make check-components` leaves in `<target>/package/`). It then tests and
# runs the copy. The copy sees no workspace, no path dependency and no
# unpackaged source, so a file missing from a package, a dependency that only
# resolved through the repository, or a manifest that is not self-contained
# fails here.
#
# Usage: scripts/check_packaged_examples.sh [scenario ...]   (default: every example)
# Environment: CARGO_TARGET_DIR (default: target) holds `package/` and the build;
# OSL_REUSE_PACKAGES=1 reuses the archives already there.

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TARGET="${CARGO_TARGET_DIR:-$ROOT/target}"
PACKAGE_DIR="$TARGET/package"
VERSION="0.22.0"
CRATES=(core math pricing simulation market analytics strategies backtest visualization)

cd "$ROOT"

# Package the components together (their path dependencies are not on
# crates.io yet, so one run resolves them from each other). The archives are
# rebuilt every time, so a stale copy of an earlier run cannot stand in for the
# working tree; set OSL_REUSE_PACKAGES=1 to use the ones `make check-components`
# has just left in `<target>/package/` (CI does, right after that target).
reuse="${OSL_REUSE_PACKAGES:-0}"
missing=0
for crate in "${CRATES[@]}"; do
    [ -f "$PACKAGE_DIR/optionstratlib-$crate-$VERSION.crate" ] || missing=1
done
if [ "$reuse" != "1" ] || [ "$missing" -eq 1 ]; then
    echo "packaging the component crates"
    rm -f "$PACKAGE_DIR"/optionstratlib-*.crate
    args=()
    for crate in "${CRATES[@]}"; do args+=(-p "optionstratlib-$crate"); done
    cargo package "${args[@]}" --allow-dirty --no-verify
fi

work="$(mktemp -d "${TMPDIR:-/tmp}/osl-packaged-examples.XXXXXX")"
trap 'rm -rf "$work"' EXIT

# Unpack every archive once; the patches point at these directories.
for crate in "${CRATES[@]}"; do
    tar -xzf "$PACKAGE_DIR/optionstratlib-$crate-$VERSION.crate" -C "$work"
done

scenarios=("$@")
if [ "${#scenarios[@]}" -eq 0 ]; then
    for dir in examples/direct/*/; do scenarios+=("$(basename "$dir")"); done
fi

for scenario in "${scenarios[@]}"; do
    source_dir="examples/direct/$scenario"
    [ -f "$source_dir/Cargo.toml" ] || { echo "unknown example: $scenario" >&2; exit 1; }
    copy="$work/example-$scenario"
    mkdir -p "$copy"
    cp -R "$source_dir/src" "$copy/src"
    # Registry form of the OptionStratLib dependencies: drop their `path`.
    sed -E '/^optionstratlib-/ s/path = "[^"]*", //' "$source_dir/Cargo.toml" > "$copy/Cargo.toml"
    {
        echo
        echo "[patch.crates-io]"
        for crate in "${CRATES[@]}"; do
            echo "optionstratlib-$crate = { path = \"$work/optionstratlib-$crate-$VERSION\" }"
        done
    } >> "$copy/Cargo.toml"
    if grep -q 'path = "\.\./' "$copy/Cargo.toml"; then
        echo "$scenario: the copied manifest still has a relative path" >&2
        exit 1
    fi
    echo "=== $scenario (packaged)"
    CARGO_TARGET_DIR="$TARGET/packaged-examples" cargo test --quiet --manifest-path "$copy/Cargo.toml"
    CARGO_TARGET_DIR="$TARGET/packaged-examples" cargo run --quiet --manifest-path "$copy/Cargo.toml"
done
echo "OK: ${#scenarios[@]} example(s) build, pass and run against the packaged component crates"
