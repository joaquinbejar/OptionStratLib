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
# OSL_REUSE_PACKAGES=1 reuses the archives already there when all ten are
# (scripts/package_archives.sh);
# OSL_PACKAGED_SOURCE (default: examples/direct) is the directory of scenarios.

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TARGET="${CARGO_TARGET_DIR:-$ROOT/target}"
# shellcheck source=scripts/package_archives.sh
source "$ROOT/scripts/package_archives.sh"
SOURCE="${OSL_PACKAGED_SOURCE:-examples/direct}"

cd "$ROOT"

osl_package_archives "$TARGET"

work="$(mktemp -d "${TMPDIR:-/tmp}/osl-packaged-examples.XXXXXX")"
trap 'rm -rf "$work"' EXIT

# Unpack every archive once; the patches point at these directories.
osl_unpack_archives "$TARGET" "$work"

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
        for package in "${OSL_PACKAGES[@]}"; do
            echo "$package = { path = \"$work/$(osl_unpacked "$work" "$package")\" }"
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
