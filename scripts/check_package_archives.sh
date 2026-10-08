#!/usr/bin/env bash
# Build, document and doc-test the ten published crates from their unpacked
# archives (#559), the files crates.io would serve and nothing else.
#
# The archives come from scripts/package_archives.sh (shared with
# check_packaged_examples.sh). They are unpacked into a scratch directory
# outside the repository and joined there by a throwaway workspace whose
# `[patch.crates-io]` resolves each crate's `version = "0.22.0"` dependencies
# on its siblings to the unpacked copies. In that workspace this runs:
#
#   cargo build  --workspace --all-features
#   cargo check  -p <crate> --no-default-features     (each crate)
#   cargo doc    --workspace --all-features --no-deps (RUSTDOCFLAGS=-D warnings)
#   cargo test   --workspace --all-features --doc     (the documented examples)
#   cargo +<rust-version> check --workspace --all-features and
#   cargo +<rust-version> check --workspace --no-default-features
#
# The last two prove the declared minimum supported Rust version; they are
# skipped with a notice when that toolchain is not installed.
#
# Usage: scripts/check_package_archives.sh
# Environment: CARGO_TARGET_DIR (default: target) holds `package/` and the
# build; OSL_REUSE_PACKAGES=1 reuses the archives already there.

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TARGET="${CARGO_TARGET_DIR:-$ROOT/target}"
# shellcheck source=scripts/package_archives.sh
source "$ROOT/scripts/package_archives.sh"

cd "$ROOT"
osl_package_archives "$TARGET"

work="$(mktemp -d "${TMPDIR:-/tmp}/osl-package-archives.XXXXXX")"
trap 'rm -rf "$work"' EXIT
osl_unpack_archives "$TARGET" "$work"
# `cargo package` stamps every archived file with the same fixed mtime (2006),
# and the members below keep the same relative paths on every run, so the
# artifacts of an earlier run in the persistent target directories would pass
# for up to date and the checks would build stale code (#558). A fresh mtime
# makes cargo rebuild the ten crates; the dependencies stay cached.
find "$work" -type f -exec touch {} +

{
    echo "[workspace]"
    echo 'resolver = "3"'
    echo "members = ["
    for package in "${OSL_PACKAGES[@]}"; do echo "    \"$package-$OSL_VERSION\","; done
    echo "]"
    echo
    echo "[patch.crates-io]"
    for package in "${OSL_PACKAGES[@]}"; do
        echo "$package = { path = \"$package-$OSL_VERSION\" }"
    done
} > "$work/Cargo.toml"

export CARGO_TARGET_DIR="$TARGET/package-archives"
manifest=(--manifest-path "$work/Cargo.toml")

echo "=== build (all features)"
cargo build "${manifest[@]}" --workspace --all-features
for package in "${OSL_PACKAGES[@]}"; do
    echo "=== check $package (no default features)"
    cargo check "${manifest[@]}" -p "$package" --no-default-features
done
echo "=== doc (all features, warnings denied)"
RUSTDOCFLAGS="-D warnings" cargo doc "${manifest[@]}" --workspace --all-features --no-deps
echo "=== doc tests (all features)"
cargo test "${manifest[@]}" --workspace --all-features --doc

rust_version="$(sed -nE 's/^rust-version = "([^"]+)"$/\1/p' "$work/optionstratlib-$OSL_VERSION/Cargo.toml")"
if [ -z "$rust_version" ]; then
    echo "the facade archive declares no rust-version" >&2
    exit 1
fi
if rustup run "$rust_version" cargo --version > /dev/null 2>&1; then
    export CARGO_TARGET_DIR="$TARGET/package-archives-msrv"
    echo "=== Rust $rust_version: check (all features)"
    cargo "+$rust_version" check "${manifest[@]}" --workspace --all-features
    echo "=== Rust $rust_version: check (no default features)"
    cargo "+$rust_version" check "${manifest[@]}" --workspace --no-default-features
else
    echo "NOTICE: Rust $rust_version is not installed; the minimum supported version was not checked"
fi
echo "OK: the ${#OSL_PACKAGES[@]} unpacked archives build, document and pass their doc tests"
