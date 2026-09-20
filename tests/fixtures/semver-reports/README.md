# `cargo-semver-checks` report fixtures (#592)

`scripts/check_accepted_breaks.py --self-test` parses every file here. They
pin the report grammar of the pinned tool version
(`cargo-semver-checks 0.50.0`, recorded in `public-api/accepted-breaks.toml`)
and the parser's fail-closed rules: an unreadable or unexpected report must
fail the run, never be read as "zero breaks".

## Real reports

Produced by running the tool on a throwaway two-crate probe (a baseline copy
and a current copy of the same package); absolute paths were replaced by
`<root>`.

| File | Exit | What it pins |
| --- | ---: | --- |
| `breaks.txt` | 100 | seven lints, 69 items; two changes on one type reported under different lints; a path-level removal (`struct probe::TwiceAlias`, whose struct still exists at `inner::Twice`); 62 items in one list with no truncation marker; the three shapes of the source-location suffix |
| `clean.txt` | 0 | `Checked ... 196 pass, 58 skip` with no `fail` group and `Summary no semver update required` |
| `feature-gated.txt` | 100 | a function that leaves the default surface but stays behind a feature: reported under `--only-explicit-features`, invisible under `--all-features`, which is why the gate runs every surface |
| `compile-error.txt` | 101 | the current crate does not build: no `Checked`/`Summary` line at all |
| `missing-baseline.txt` | 101 | the baseline path does not resolve |
| `bad-flags.txt` | 2 | conflicting command-line flags |

## Derived reports

Edited copies of `breaks.txt`, each introducing one defect the parser must
refuse. They are synthetic on purpose: the tool does not produce them today,
and the rules exist so a future version cannot degrade silently.

| File | Defect |
| --- | --- |
| `truncated.txt` | an `and N more` marker inside a `Failed in:` list |
| `miscounted.txt` | the summary announces more failures than there are blocks |
| `unknown-lint.txt` | a lint name the pinned version does not have |
| `wrong-version.txt` | the `impl:` link points at another tool version |
| `no-items.txt` | a failure block whose `Failed in:` list is empty |
| `same-item-two-lints.txt` | one item reported under two lints, which must stay two findings |
| `blank-line-in-list.txt` | a blank line inside a `Failed in:` list, which must not truncate it (all 69 items still parse) |
| `wrapped-item.txt` | an item line split in two, whose continuation has no item text: refused rather than counted as a phantom finding |

## What the fixtures are for

`scripts/report_api_changes.py --self-test` parses every file here. They pin
the report grammar of the pinned tool version and the parser's fail-closed
rules: an unreadable or unexpected report must fail the job, never be read as
"this pull request changes no API".

`lints-0.50.0.txt` is the lint inventory of that version
(`cargo semver-checks --list`), used to reject an unknown lint. It is
deliberately not derived from the report under test, which by construction
contains only lints that report names.

The sequence fixture that exercised the accepted-breaks register was removed
with the register itself (#606): 0.22.0 is under development and
compatibility with the published 0.21.3 is no longer a requirement, so there
is no per-item authorisation left to test.
