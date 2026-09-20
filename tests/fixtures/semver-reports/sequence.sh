#!/bin/bash
# Sequence test for the accepted-breaks gate (#592).
#
# Builds a throwaway git repository with a probe crate at 0.21.3 and walks it
# through the four states the gate must tell apart, running the three
# comparisons on each, twice: once as a pull request (on the synthetic merge
# commit CI checks out) and once as the push that integrates it.
#
#   usage: sequence.sh [workdir]        (default: a fresh mktemp -d)
#
# `sequence-results.txt` beside this script is the recorded output. Re-running
# produces the same findings with different SHAs.
# Four states: normal PR that ADDS, approved break that REMOVES a 0.21.3 item,
# normal PR, second break that REMOVES the item added in step 1.
# Runs C1 (published 0.21.3), C2 (controlled reference) and C3 (incremental)
# for every state, both as a PR (synthetic merge commit) and as the push that
# integrates it.
set -u
WORK=${1:-$(mktemp -d)}
SCRIPT=$(cd "$(dirname "$0")/../../.." && pwd)/scripts/check_accepted_breaks.py
PY=$(command -v python3.13 || command -v python3.12 || command -v python3.11 || command -v python3)
R=$WORK/repo
OUT=$WORK/results.txt
: > "$OUT"
log() { echo "$*" | tee -a "$OUT"; }

if [ ! -d "$R" ]; then
  mkdir -p "$R/src"
  cat > "$R/Cargo.toml" <<'EOF'
[package]
name = "probe"
version = "0.21.3"
edition = "2024"
[features]
default = []
EOF
  cat > "$R/src/lib.rs" <<'EOF'
//! Probe crate for the accepted-breaks gate sequence test.
pub fn old_fn() -> u32 { 1 }
pub struct S;
impl S { pub fn keep(&self) -> u32 { 2 } }
EOF
  git -C "$R" init -q -b main .
  git -C "$R" config user.email gate@example.invalid
  git -C "$R" config user.name "gate sequence test"
  git -C "$R" add -A
  git -C "$R" commit -q -m "v0.21.3 published surface"
  git -C "$R" tag v0.21.3
fi

# Newest ancestor of $1 (exclusive of $1 itself when $2 = exclusive) carrying
# the Accepted-Breaks trailer; falls back to the initial reference tag.
reference() {
  local start=$1 mode=${2:-inclusive} range
  [ "$mode" = exclusive ] && range="$start^" || range="$start"
  local sha
  sha=$(git -C "$R" log --format='%H' --grep='^Accepted-Breaks:' "$range" 2>/dev/null | head -1)
  [ -n "$sha" ] && echo "$sha" || git -C "$R" rev-parse v0.21.3
}

# The register the probe repository is checked against: step 2 and step 4
# land one authorised break each, declared per check exactly as the policy
# requires (C1 only sees the removal of an item that existed in 0.21.3).
write_register() {
  mkdir -p "$R/public-api"
  cat > "$R/public-api/accepted-breaks.toml" <<REG
[register]
# The probe was never published, so C1 falls back to `initial_reference`,
# which plays the published crate's role in the fixture.
initial_reference = "v0.21.3"
owner = "joaquinbejar"
repo = "joaquinbejar/OptionStratLib"
issue = 592
tool = "cargo-semver-checks 0.50.0"

[[register.surfaces]]
id = "none"
features = []
$1
REG
}

AB01='[[break]]
id = "AB-01"
surfaces = ["none"]
decision = "sequence fixture"
issue = 592
migration = "call the replacement"
status = "landed"
approval = { ref = "https://github.com/joaquinbejar/OptionStratLib/issues/592#issuecomment-1" }
landed = { pr = 1 }
findings = [
  { check = "C1", lint = "function_missing", item = "function probe::old_fn" },
  { check = "C2", lint = "function_missing", item = "function probe::old_fn" },
  { check = "C3", lint = "function_missing", item = "function probe::old_fn" },
]'
AB02='[[break]]
id = "AB-02"
surfaces = ["none"]
decision = "sequence fixture"
issue = 592
migration = "call the replacement"
status = "landed"
approval = { ref = "https://github.com/joaquinbejar/OptionStratLib/issues/592#issuecomment-2" }
landed = { pr = 2 }
findings = [
  { check = "C2", lint = "function_missing", item = "function probe::added_fn" },
  { check = "C3", lint = "function_missing", item = "function probe::added_fn" },
]'

# $1 = worktree, $2 = check, $3 = baseline, $4 = pr number (or empty)
policy() {
  local wt=$1 check=$2 base=$3 pr=$4 extra=""
  [ -n "$pr" ] && extra="--pr $pr"
  ( cd "$wt" && CARGO_TARGET_DIR=$WORK/target "$PY" "$SCRIPT"       --check "$check" --surface none --baseline "$base" --register "$wt/public-api/accepted-breaks.toml"       --root "$wt" $extra 2>&1 | tail -2 | sed 's/^/      /' )
}

# $1 = worktree to check, $2 = baseline rev, $3 = label
check() {
  local wt=$1 base=$2 label=$3 out
  out=$(cd "$wt" && CARGO_TARGET_DIR=$WORK/target cargo semver-checks --baseline-rev "$base" --only-explicit-features 2>&1)
  local rc=$?
  local items
  items=$(echo "$out" | awk '/^--- failure /{lint=$3; sub(":","",lint)} /^Failed in:/{f=1; next} f&&/^  /{print lint" "$0} /^$/{f=0}' | sed 's/, previously in file.*//; s/ in file .*//' | sed 's/^\([a-z_]*\)   */\1 /' | sort | tr '\n' ';')
  log "    $label (rc=$rc): ${items:-<none>}"
}

state() { # $1 = commit under test (synthetic merge or main tip), $2 = label, $3 = C3 baseline, $4 = C2 baseline
  local sha=$1 label=$2 c3base=$3 c2base=$4
  local wt=$WORK/wt
  rm -rf "$wt"; git -C "$R" worktree add -q --detach "$wt" "$sha" 2>/dev/null
  log "  $label  tree=$(git -C "$R" rev-parse --short "$sha")"
  check "$wt" "$(git -C "$R" rev-parse v0.21.3)" "C1 published-0.21.3"
  check "$wt" "$c2base" "C2 reference=$(git -C "$R" rev-parse --short "$c2base")"
  check "$wt" "$c3base" "C3 incremental=$(git -C "$R" rev-parse --short "$c3base")"
  if [ -f "$wt/public-api/accepted-breaks.toml" ]; then
    log "    register verdicts:"
    log "$(policy "$wt" C1 "$(git -C "$R" rev-parse v0.21.3)" "$PR_NUMBER")"
    log "$(policy "$wt" C2 "$c2base" "$PR_NUMBER")"
    log "$(policy "$wt" C3 "$c3base" "$PR_NUMBER")"
  fi
  git -C "$R" worktree remove --force "$wt" 2>/dev/null
}

pr() { # $1 = branch, $2 = commit message, $3 = trailer, $4 = register body, $5 = PR number, $6... = the change
  local branch=$1 msg=$2 trailer=$3 register=$4; PR_NUMBER=$5; shift 5
  git -C "$R" checkout -q -b "$branch" main
  "$@"
  write_register "$register"
  git -C "$R" add -A
  if [ -n "$trailer" ]; then
    git -C "$R" commit -q -m "$msg" -m "$trailer"
  else
    git -C "$R" commit -q -m "$msg"
  fi
  local head main_tip tree synth
  head=$(git -C "$R" rev-parse HEAD)
  main_tip=$(git -C "$R" rev-parse main)
  tree=$(git -C "$R" merge-tree --write-tree "$main_tip" "$head")
  synth=$(git -C "$R" commit-tree -p "$main_tip" -p "$head" -m "Merge $branch" "$tree")
  log ""
  log "== $msg"
  log "   base.sha=$(git -C "$R" rev-parse --short "$main_tip") head.sha=$(git -C "$R" rev-parse --short "$head") synthetic=$(git -C "$R" rev-parse --short "$synth")"
  # As a PR: C2 reference is the newest trailer ancestor of the BASE (the PR's
  # own commits are not yet on main and must not be the reference).
  state "$synth" "as PR" "$main_tip" "$(reference "$main_tip")"
  # Squash-merge into main.
  git -C "$R" checkout -q main
  git -C "$R" merge -q --squash "$branch" >/dev/null 2>&1
  if [ -n "$trailer" ]; then
    git -C "$R" commit -q -m "$msg (#PR)" -m "$trailer"
  else
    git -C "$R" commit -q -m "$msg (#PR)"
  fi
  local tip; tip=$(git -C "$R" rev-parse main)
  # As the push that integrates it: C3 baseline is HEAD^, and the C2 reference
  # must EXCLUDE the commit under test, otherwise a break PR would be compared
  # against itself.
  state "$tip" "as push to main" "$(git -C "$R" rev-parse main^)" "$(reference "$tip" exclusive)"
  log "   [wrong C2, inclusive] would be $(git -C "$R" rev-parse --short "$(reference "$tip")")"
}

# BSD and GNU sed disagree about `-i`; edit through a temporary file instead.
edit() { sed "$1" "$R/src/lib.rs" > "$R/src/lib.rs.tmp" && mv "$R/src/lib.rs.tmp" "$R/src/lib.rs"; }
add_fn()    { printf 'pub fn added_fn() -> u32 { 3 }\n' >> "$R/src/lib.rs"; }
rm_old()    { edit '/pub fn old_fn/d'; }
touch_body(){ edit 's/pub fn keep(&self) -> u32 { 2 }/pub fn keep(\&self) -> u32 { 2 + 0 }/'; }
rm_added()  { edit '/pub fn added_fn/d'; }

# The register grows with the run: nothing at step 1, AB-01 from step 2,
# AB-01 + AB-02 from step 4. The `landed.pr` numbers match the step.
pr step1-add   "Step 1: normal PR adds added_fn"          ""                       ""                     1 add_fn
pr step2-break "Step 2: approved break removes old_fn"    "Accepted-Breaks: AB-01" "$AB01"                1 rm_old
pr step3-noop  "Step 3: normal PR changes a body"         ""                       "$AB01"                3 touch_body
pr step4-break "Step 4: approved break removes added_fn"  "Accepted-Breaks: AB-02" "$AB01

$AB02"                2 rm_added
log ""
log "DONE"
