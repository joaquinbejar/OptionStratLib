# 0.22.1 release gates: notes

## About the commit gated

The full run in `gates.md` is of `ec2904bc` (the 0.22.1 version, dependency
and README commit on top of `main` at `564a74ab`). Every gate passed there
with no warning except `publish-dry-run`. That gate failed for a reason in the
gate itself: `scripts/publish_dry_run.py` recognised an unpublished sibling
only as cargo's "no matching package named" message, which is what the first
release of the family (0.22.0) produced. For a patch, the sibling exists on
crates.io at 0.22.0 and cargo reports "failed to select a version for the
requirement `optionstratlib-core = "^0.22.1"`" instead.

`a69edd7c` teaches the script that message, with a self-test case. It changes
only `scripts/publish_dry_run.py`, which no other gate reads and no package
contains. The `publish-dry-run` row is from a rerun on `a69edd7c`
(`release_gates.py run --release 0.22.1 --only publish-dry-run`); every other
row is from `ec2904bc`.

## Findings

- Dependencies: `positive` 0.8.0, `expiration_date` 0.5.0 and `option_type`
  0.5.0, one version of each in the graph. No code change was needed.
- Semver reports against 0.21.3 are informational, as for 0.22.0.
