# shellcheck shell=bash
# Shared by the scripts that work on the packaged crates (#555, #552, #559):
# packages the ten published crates together and unpacks their archives.
# Source it; it defines no side effect of its own.
#
#   OSL_VERSION               the lockstep version of every published crate
#   OSL_PACKAGES              the published packages, the facade last
#   osl_package_archives T    packages them into T/package
#   osl_unpack_archives T D   unpacks every T/package archive into D
#
# OSL_REUSE_PACKAGES=1 makes `osl_package_archives` reuse the archives already
# in T/package, but only when all ten are there; otherwise all ten are deleted
# and packaged again in one run. `make check-components` leaves the nine
# component archives and not the facade's, so the first run after it always
# repackages all ten.

OSL_VERSION="0.22.0"
# The facade last: it depends on every component.
OSL_PACKAGES=(
    optionstratlib-core optionstratlib-math optionstratlib-pricing optionstratlib-simulation
    optionstratlib-market optionstratlib-analytics optionstratlib-strategies optionstratlib-backtest
    optionstratlib-visualization optionstratlib
)

# Packages the facade and the components together: their path dependencies
# are not on crates.io yet, so one run resolves them from each other. The
# archives are rebuilt unless OSL_REUSE_PACKAGES=1 and all ten exist, so a
# stale copy of an earlier run cannot stand in for the working tree.
osl_package_archives() {
    local package_dir="$1/package" missing=0 package
    for package in "${OSL_PACKAGES[@]}"; do
        [ -f "$package_dir/$package-$OSL_VERSION.crate" ] || missing=1
    done
    if [ "${OSL_REUSE_PACKAGES:-0}" != "1" ] || [ "$missing" -eq 1 ]; then
        echo "packaging the facade and the component crates"
        rm -f "$package_dir"/optionstratlib-*.crate
        local args=()
        for package in "${OSL_PACKAGES[@]}"; do args+=(-p "$package"); done
        cargo package "${args[@]}" --allow-dirty --no-verify
    fi
}

# Unpacks every archive into the directory given; each one becomes
# `<dir>/<package>-<version>`.
osl_unpack_archives() {
    local package_dir="$1/package" package
    for package in "${OSL_PACKAGES[@]}"; do
        tar -xzf "$package_dir/$package-$OSL_VERSION.crate" -C "$2"
    done
}
