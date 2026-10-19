## About the commit gated

The run above is of `e5084f33`: `main` at `45255be9` plus this branch's
release notes, tooling changes, package fix and README update. The files
changed after the run are this evidence (`gates.md`, these notes), the
CHANGELOG, which moves the `[Unreleased]` entries under `[0.23.0]`, and one
sentence of the release notes (`3c4c2bbe`).

- No gate reads the evidence or the CHANGELOG.
- The release notes sentence is prose in "Dependencies and security": it
  names `ttf-parser` instead of claiming that no font parser is resolved,
  since `skrifa` reads fonts. No code block or program changed.
- The `release-notes`, `package-archives` and `publish-dry-run` gates read the
  notes. On `3c4c2bbe`, `make check-release-notes RELEASE=0.23.0` builds and
  runs both manifest examples, and
  `cargo test --doc --features static_export -p optionstratlib ReleaseNotes0`
  passes both `ReleaseNotes023Doctests`.

Apart from that sentence, the crates are published from the merge of this
branch, whose packaged sources are those of the gated commit.

## Findings and fixes on the way to this run

- **Packaged notes.** The first complete run, on `b67de279`, failed
  `package-archives` and `publish-dry-run`. The facade's new doctest of the
  0.23 release notes (`ReleaseNotes023Doctests`) reads
  `docs/release/0.23/RELEASE-NOTES.md`, which the facade's `include` list
  did not ship, so the packaged facade's doctests did not compile.
  `fa277f1c` adds the file to the list, beside the 0.22 notes.
- **Evidence of a rerun.** A complete rerun kept the first run's metadata,
  so `gates.md` named the earlier commit. `f2491c96` makes a complete run
  start afresh, records each result's commit, and lists in `gates.md` any
  gate run on a commit other than the gated one; this run has none.
- **README.** The run on `f2491c96` was stopped to bring the README and crate
  docs to 0.23 (`e5084f33`); this complete run followed.
- `cargo package` notices about test and bench targets it does not ship are
  counted apart, as in 0.22 (#559).

## Semver rows

The `semver-*` rows compare the facade with `v0.22.2` and report no removal
or incompatible reshaping on any surface. They do not see this release's
breaking change, which is in `optionstratlib-visualization`: the facade
re-exports `GraphError` from it, and `cargo-semver-checks` compares the items
a crate defines. `api-classification.md` adds a run on each component crate
against its published version and classifies its one finding.
