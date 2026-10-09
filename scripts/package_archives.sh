# shellcheck shell=bash
# Shared by the scripts that work on the packaged crates (#555, #552, #559):
# packages the ten published crates together and unpacks their archives.
# Source it; it defines no side effect of its own.
#
#   OSL_PACKAGES              the published packages, the facade last
#   osl_package_archives T    packages them into T/package
#   osl_unpack_archives T D   unpacks every T/package archive into D
#   osl_archive T P           the one archive of package P in T/package
#   osl_unpacked D P          the directory package P unpacked to in D
#
# Each crate is on the 0.22 line at its own patch (docs/versioning-policy.md,
# #834), so an archive is found by name and version digits, not by one shared
# version.
#
# OSL_REUSE_PACKAGES=1 makes `osl_package_archives` reuse the archives already
# in T/package, but only when all ten are there; otherwise all ten are deleted
# and packaged again in one run. `make check-components` leaves the nine
# component archives and not the facade's, so the first run after it always
# repackages all ten.

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
# Prints the one `<package>-<version>.crate` of a package in `$1/package`;
# fails when there is none or more than one. `-[0-9]` keeps `optionstratlib`
# from matching the components' archives.
osl_archive() {
    local found=() path
    for path in "$1/package/$2"-[0-9]*.crate; do
        [ -f "$path" ] && found+=("$path")
    done
    if [ "${#found[@]}" -ne 1 ]; then
        echo "expected one archive of $2 in $1/package, found ${#found[@]}" >&2
        return 1
    fi
    echo "${found[0]}"
}

# Prints the name of the `<package>-<version>` directory a package unpacked
# to in `$1`; fails when there is none or more than one.
osl_unpacked() {
    local found=() path
    for path in "$1/$2"-[0-9]*; do
        [ -d "$path" ] && found+=("$(basename "$path")")
    done
    if [ "${#found[@]}" -ne 1 ]; then
        echo "expected one unpacked $2 in $1, found ${#found[@]}" >&2
        return 1
    fi
    echo "${found[0]}"
}

osl_package_archives() {
    local missing=0 package
    for package in "${OSL_PACKAGES[@]}"; do
        osl_archive "$1" "$package" > /dev/null 2>&1 || missing=1
    done
    if [ "${OSL_REUSE_PACKAGES:-0}" != "1" ] || [ "$missing" -eq 1 ]; then
        echo "packaging the facade and the component crates"
        rm -f "$1"/package/optionstratlib-*.crate
        local args=()
        for package in "${OSL_PACKAGES[@]}"; do args+=(-p "$package"); done
        cargo package "${args[@]}" --allow-dirty --no-verify
    fi
}

# Unpacks every archive into the directory given; each one becomes
# `<dir>/<package>-<version>`.
osl_unpack_archives() {
    local package archive
    for package in "${OSL_PACKAGES[@]}"; do
        archive="$(osl_archive "$1" "$package")" || return 1
        tar -xzf "$archive" -C "$2"
    done
}
