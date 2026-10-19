## About the commit gated

The run above is of `fa277f1c`: `main` at `45255be9` plus this branch's
release notes, tooling changes and package fix. It is the evidence for the
release candidate as prepared on 2026-10-19; the commit actually tagged gets
a complete rerun of `make release-gates` if anything a gate reads changes
after this run. Nothing is published, tagged or released by these gates.

## Findings and fixes of this run

- The first run, on `b67de279`, failed `package-archives` and
  `publish-dry-run`: the facade's new doctest of the 0.23 release notes
  (`ReleaseNotes023Doctests`) reads `docs/release/0.23/RELEASE-NOTES.md`,
  which the facade's `include` list did not ship, so the packaged facade's
  doctests did not compile. `fa277f1c` adds the file to the list, beside the
  0.22 notes, and the complete rerun above passes every gate.
- `cargo package` notices about test and bench targets it does not ship are
  counted apart, as in 0.22 (#559).
- The `semver-*` rows compare the facade with `v0.22.2` and report no
  removal or incompatible reshaping on any surface. They do not see the
  breaking change of this release, which is in `optionstratlib-visualization`:
  the facade re-exports `GraphError` from it, and `cargo-semver-checks`
  compares the items a crate defines. `api-classification.md` adds a run on
  each component crate against its published version.
