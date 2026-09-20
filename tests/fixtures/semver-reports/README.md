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

## Sequence test

`sequence.sh [workdir]` builds a throwaway git repository with a probe crate
at 0.21.3 and walks it through the four states the gate must tell apart,
running C1, C2 and C3 on each, twice: as a pull request (on the synthetic
merge commit) and as the push that integrates it.

```
step 1  normal PR adds `added_fn`                     C1 -  C2 -  C3 -
step 2  authorised break removes `old_fn` (in 0.21.3) C1 x  C2 x  C3 x
step 3  normal, unlabelled PR changes a body          C1 x  C2 -  C3 -
step 4  authorised break removes `added_fn`           C1 -  C2 x  C3 x
```

Step 3 is why an ordinary pull request needs no label once a break has
landed: C1 keeps reporting it (and the register keeps accounting for it)
while C2 and C3 stay green. Step 4 is why C1 alone is not enough: an item
added after 0.21.3 and removed later is invisible to it.

Each state also runs the register comparison itself (`check_accepted_breaks.py`
against a register the fixture writes into the probe repository), so the
policy is exercised, not only the tool: 24 verdicts, all passing, including
step 3, where the landed AB-01 is still expected by C1 and no longer demanded
by C2 or C3, which is what keeps an ordinary pull request green without a
label.

`lints-0.50.0.txt` is the lint inventory of the pinned tool
(`cargo semver-checks --list`), used to reject an unknown lint. It is
deliberately not derived from the report under test, which by construction
contains only lints that the report names.

`sequence-results.txt` is the recorded output; re-running reproduces it with
different SHAs. It also shows, on the integrating push, the commit the wrong
(inclusive) reference rule would have selected: the commit under test itself.
