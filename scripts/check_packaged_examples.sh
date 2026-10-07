#!/usr/bin/env bash
# Build each direct-component example (#555), or each 0.22 consumer fixture
# (#552), against the packaged OptionStratLib crates, the way a consumer gets
# them once they are published.
#
# For every `<source>/<scenario>` (`examples/direct` by default,
# `fixtures/consumers` with OSL_PACKAGED_SOURCE) this copies the manifest, the
# sources and the tests (with any data under them) out of the repository into
# a scratch directory, removes the `path = "..."` from the OptionStratLib
# dependencies (leaving `version = "0.22.0"`, the registry form), and adds a
# `[patch.crates-io]` that points the facade and every component at the
# unpacked `.crate` file `cargo package` produced in `<target>/package/`. It then tests
# the copy, and runs it when it is a binary. The copy sees no workspace, no
# path dependency and no unpackaged source, so a file missing from a package,
# a dependency that only resolved through the repository, or a manifest that
# is not self-contained fails here.
#
# Usage: scripts/check_packaged_examples.sh [scenario ...]   (default: every scenario)
# Environment: CARGO_TARGET_DIR (default: target) holds `package/` and the build;
# OSL_REUSE_PACKAGES=1 reuses the archives already there when all ten are;
# OSL_PACKAGED_SOURCE (default: examples/direct) is the directory of scenarios.

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TARGET="${CARGO_TARGET_DIR:-$ROOT/target}"
PACKAGE_DIR="$TARGET/package"
VERSION="0.22.0"
# The facade last: it depends on every component.
PACKAGES=(
    optionstratlib-core optionstratlib-math optionstratlib-pricing optionstratlib-simulation
    optionstratlib-market optionstratlib-analytics optionstratlib-strategies optionstratlib-backtest
    optionstratlib-visualization optionstratlib
)
SOURCE="${OSL_PACKAGED_SOURCE:-examples/direct}"

cd "$ROOT"

# Package the facade and the components together (their path dependencies are
# not on crates.io yet, so one run resolves them from each other). The archives
# are rebuilt every time, so a stale copy of an earlier run cannot stand in for
# the working tree. With OSL_REUSE_PACKAGES=1 the ten archives already in
# `<target>/package/` are used as they are, but only when all ten are there;
# if any is missing, all ten are deleted and packaged again in one run.
# `make check-components` leaves the nine component archives and not the
# facade's, so the first run after it always repackages all ten; a later run
# with OSL_REUSE_PACKAGES=1 reuses those.
reuse="${OSL_REUSE_PACKAGES:-0}"
missing=0
for package in "${PACKAGES[@]}"; do
    [ -f "$PACKAGE_DIR/$package-$VERSION.crate" ] || missing=1
done
if [ "$reuse" != "1" ] || [ "$missing" -eq 1 ]; then
    echo "packaging the facade and the component crates"
    rm -f "$PACKAGE_DIR"/optionstratlib-*.crate
    args=()
    for package in "${PACKAGES[@]}"; do args+=(-p "$package"); done
    cargo package "${args[@]}" --allow-dirty --no-verify
fi

work="$(mktemp -d "${TMPDIR:-/tmp}/osl-packaged-examples.XXXXXX")"
trap 'rm -rf "$work"' EXIT

# Unpack every archive once; the patches point at these directories.
for package in "${PACKAGES[@]}"; do
    tar -xzf "$PACKAGE_DIR/$package-$VERSION.crate" -C "$work"
done

scenarios=("$@")
if [ "${#scenarios[@]}" -eq 0 ]; then
    for dir in "$SOURCE"/*/; do
        [ -f "$dir/Cargo.toml" ] && scenarios+=("$(basename "$dir")")
    done
fi

for scenario in "${scenarios[@]}"; do
    source_dir="$SOURCE/$scenario"
    [ -f "$source_dir/Cargo.toml" ] || { echo "unknown scenario: $source_dir" >&2; exit 1; }
    copy="$work/scenario-$scenario"
    mkdir -p "$copy"
    cp -R "$source_dir/src" "$copy/src"
    [ -d "$source_dir/tests" ] && cp -R "$source_dir/tests" "$copy/tests"
    # Registry form of the OptionStratLib dependencies (the facade's line and
    # the components'): drop their `path`.
    sed -E '/^optionstratlib(-[a-z]+)? =/ s/path = "[^"]*", //' "$source_dir/Cargo.toml" > "$copy/Cargo.toml"
    {
        echo
        echo "[patch.crates-io]"
        for package in "${PACKAGES[@]}"; do
            echo "$package = { path = \"$work/$package-$VERSION\" }"
        done
    } >> "$copy/Cargo.toml"
    if grep -q 'path = "\.\./' "$copy/Cargo.toml"; then
        echo "$scenario: the copied manifest still has a relative path" >&2
        exit 1
    fi
    echo "=== $scenario (packaged)"
    build="$TARGET/packaged-$(basename "$SOURCE")"
    CARGO_TARGET_DIR="$build" cargo test --quiet --manifest-path "$copy/Cargo.toml"
    if [ -f "$copy/src/main.rs" ]; then
        CARGO_TARGET_DIR="$build" cargo run --quiet --manifest-path "$copy/Cargo.toml"
    fi
done
echo "OK: ${#scenarios[@]} scenario(s) of $SOURCE build and pass against the packaged crates"
