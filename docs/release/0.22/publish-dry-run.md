# 0.22.0 package and publish dry-runs

Evidence for M8-02 (#558): every published crate packaged with cargo's
verification, its publication dry-run, and each archive built and tested on
its own. **Nothing was published**: every `cargo publish` below carries
`--dry-run`, and `scripts/publish_dry_run.py` refuses to run one without it.
No `cargo login`, tag or GitHub release was made.

## Reproduce

```sh
make publish-dry-run                    # scripts/publish_dry_run.py, on a committed tree
OSL_ALLOW_DIRTY=1 make publish-dry-run  # a working copy with uncommitted changes
```

`scripts/release_gates.py` runs it as the `publish-dry-run` gate. It needs
network access for the crates.io index and a committed tree: cargo refuses
to package uncommitted changes without `--allow-dirty`, which release evidence
does not use. Logs go to `target/publish-dry-run/logs/`, one per command.

## Run

- Commit: `95448fa6` (`origin/main` `5cc57634` plus the packaging fixes of
  this change; the commit that carries this document adds only it,
  `packages.md` and the changelog, none of which is in an archive).
- Toolchain: cargo 1.99.0 (`5f94df478 2026-08-27`), macOS (Apple silicon).
- Date: 2026-10-08.
- Result: every step passes; no warning other than the packaging notices
  below.

## 1. Package

```sh
cargo package -p optionstratlib-core -p optionstratlib-math -p optionstratlib-pricing \
  -p optionstratlib-simulation -p optionstratlib-market -p optionstratlib-analytics \
  -p optionstratlib-strategies -p optionstratlib-backtest -p optionstratlib-visualization \
  -p optionstratlib
```

One invocation packages the ten crates in dependency order, then verifies
each one (`Verifying ...`: a build of the archive). The components are not on
crates.io, so cargo resolves each crate's `^0.22.0` dependencies on its
siblings from the archives it has just written, not from the repository's
paths. Every package passes. The only warnings are 71 notices of the form
``ignoring test `x` as `tests/x.rs` is not included in the published
package``, which #559 accepts on purpose (`packages.md`, "Package notices").

| Archive | Bytes | SHA-256 |
| --- | ---: | --- |
| `optionstratlib-core-0.22.0.crate` | 155,040 | `efecdb0dcc2466188b66cd4ab142a5b400bdece23ef64c30af3e4f0871c8eddf` |
| `optionstratlib-math-0.22.0.crate` | 157,770 | `3773e906e862b60880cd2b280d9bf4bd6863a2ceb84f634646515ab6e5bb1440` |
| `optionstratlib-pricing-0.22.0.crate` | 283,994 | `c92b5f2f0566e8ea9b4a4a54bdba18ab8eccd63de61c6823c7ce7b6cc2439c16` |
| `optionstratlib-simulation-0.22.0.crate` | 90,677 | `8b12186fab292e349e3961ed6c9c2a8e6cf12aaeacd818287cb78d6de969e1ce` |
| `optionstratlib-market-0.22.0.crate` | 159,067 | `8e07d52e09d06a1cf13d8ce9ea7a480416fd46eaa8ca6a6dc6d2fbe7d74482b3` |
| `optionstratlib-analytics-0.22.0.crate` | 154,999 | `2613337114f1f8c4b38fc894b4eb2dbc69351c8a454908da0fb7c0bcd35371b6` |
| `optionstratlib-strategies-0.22.0.crate` | 393,643 | `cfb4a560ee61ff3606060c4defacb8bc31e1872af861122b24565f85745ffce8` |
| `optionstratlib-backtest-0.22.0.crate` | 46,542 | `a0019ce84387427ba6cdb67227674be6ffccf0232cdfa4534f61cfe6ea5977b8` |
| `optionstratlib-visualization-0.22.0.crate` | 80,648 | `f4a942520a1f9dfcfca82bbe7f7c491fb4e0c06ae7eac3c62c60e54ff711ff4a` |
| `optionstratlib-0.22.0.crate` | 103,363 | `41db05815812f041f92a7752ce0a56be425b62b8189f830f499e7c21276642ea` |

The archives are reproducible from a given commit: packaging `5cc57634`
twice gave the same ten checksums. They are not reproducible across commits,
because each archive's `.cargo_vcs_info.json` records the commit it was made
from, so the release commit's checksums will differ from these. The release
records its own by running `make publish-dry-run` on it.

## 2. Publish dry-run, workspace

```sh
cargo publish --dry-run --workspace
```

This publishes the workspace's ten publishable packages in dependency order,
short of the upload. For each one cargo prints `Uploading` and then
`warning: aborting upload due to dry run`, ten times in all. It passes. Before
this change it failed at once: ten example packages (`examples_chain`,
`examples_curves`, `examples_metrics`, `examples_simulation`,
`examples_strategies`, `examples_strategies_best`,
`examples_strategies_delta`, `examples_surfaces`, `examples_visualization`,
`examples_volatility`) did not declare `publish = false`, so the workspace
publication tried to include them and stopped on their unversioned path
dependency on `osl-example-support`. A real `cargo publish --workspace` would
have attempted to upload them. They now declare it, and
`scripts/publish_dry_run.py` checks that the publishable packages are exactly
the ten crates.

## 3. Publish dry-run, crate by crate

```sh
cargo publish --dry-run -p <crate>   # in dependency order
```

This is the dry-run a release repeats per crate once its dependencies are on
crates.io. Today only core resolves:

| Crate | Result |
| --- | --- |
| `optionstratlib-core` | passes against crates.io |
| `optionstratlib-math` | waiting on `optionstratlib-core` |
| `optionstratlib-pricing` | waiting on `optionstratlib-core` |
| `optionstratlib-simulation` | waiting on `optionstratlib-core` |
| `optionstratlib-market` | waiting on `optionstratlib-core` |
| `optionstratlib-analytics` | waiting on `optionstratlib-core` |
| `optionstratlib-strategies` | waiting on `optionstratlib-analytics` |
| `optionstratlib-backtest` | waiting on `optionstratlib-analytics` |
| `optionstratlib-visualization` | waiting on `optionstratlib-backtest` |
| `optionstratlib` | waiting on `optionstratlib-analytics` |

"Waiting on" is cargo's `no matching package named ... found, location
searched: crates.io index`: the first sibling the resolver reached that is
not published. The script accepts that failure and no other. During the
release, each crate's real `cargo publish` runs after the crates above it
are on crates.io, and its dry-run then resolves from the registry.

## 4. Each archive on its own

Each archive is unpacked outside the repository into a workspace that
contains only that crate. Every sibling it depends on, directly or
transitively, is resolved by a `[patch.crates-io]` entry pointing at that
sibling's unpacked archive, never at the repository: the staged equivalent
of the registry until the crates are published. Each crate then builds and
runs its unit and doc tests with three feature sets:

- the default features;
- `--no-default-features`;
- every feature except `static_export`, whose build script needs an installed
  browser. Only `optionstratlib-visualization` and the facade have it; the
  other crates use `--all-features`.

| Crate | Siblings from archives | Tests passed: default / none / all |
| --- | --- | --- |
| `optionstratlib-core` | none | 649 / 649 / 649 |
| `optionstratlib-math` | core | 473 / 473 / 473 |
| `optionstratlib-pricing` | core, math | 928 / 928 / 928 |
| `optionstratlib-simulation` | core, math, pricing | 189 / 189 / 189 |
| `optionstratlib-market` | core, math, pricing, simulation | 457 / 457 / 539 |
| `optionstratlib-analytics` | core to market | 538 / 538 / 538 |
| `optionstratlib-strategies` | core to analytics | 1,310 / 1,310 / 1,310 |
| `optionstratlib-backtest` | core to strategies | 67 / 67 / 67 |
| `optionstratlib-visualization` | core to backtest | 65 / 65 / 67 |
| `optionstratlib` | the nine components | 36 / 18 / 36 |

Every build and every test passes, with no failure and no warning. The counts
are unit and doc tests together. The ignored ones (`ignore` and `no_run`
doc blocks) are not counted.

## Findings and changes

1. **Ten example packages were publishable** (section 2). Each now declares
   `publish = false`, like the other example and fixture packages.
2. **The unit tests of three crates failed from their archives.** 24 call
   sites in `optionstratlib-market`, `optionstratlib-analytics` and
   `optionstratlib-strategies` loaded
   `../../examples/Chains/SP500-18-oct-2024-5781.88.json`, a path outside every
   package, so `cargo test` of the published crates failed: 38 tests, 19 in
   market (with all features), 13 in analytics and 6 in strategies. Each of the three
   crates now carries a copy under `testdata/` (21 KB, shipped through its
   `include` list), and its tests read it from there. `make check-packages`
   checks that every `testdata/` file is in its archive and is byte for byte
   the repository's copy under `examples/Chains/`.
3. **The archive checks could test stale code.** `cargo package` stamps
   every archived file with the same fixed mtime (24 July 2006). A workspace
   member's artifacts are named from its path relative to the workspace root,
   which is the same on every run. So a build directory kept from an earlier
   run took the archive's sources as unchanged, and cargo ran the old
   binaries. This was found when a re-run still failed on the fixed paths of
   finding 2. `scripts/check_package_archives.sh` (the #559 gate) keeps its
   build directories between runs and had the same exposure. Both scripts now
   stamp the unpacked files with the current time, which makes cargo rebuild
   the crates under test and keeps the dependency cache.

Not changed: the 71 packaging notices (`packages.md` records 53 at #559; the
difference is the integration tests and benches added since, all
repository-only by design).
