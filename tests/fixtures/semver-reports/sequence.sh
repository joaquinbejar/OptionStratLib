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
  git -C "$R" worktree remove --force "$wt" 2>/dev/null
}

pr() { # $1 = branch, $2 = commit message, $3 = trailer or empty, $4... = sed script applied to src/lib.rs
  local branch=$1 msg=$2 trailer=$3; shift 3
  git -C "$R" checkout -q -b "$branch" main
  "$@"
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

add_fn()    { printf 'pub fn added_fn() -> u32 { 3 }\n' >> "$R/src/lib.rs"; }
rm_old()    { sed -i '' '/pub fn old_fn/d' "$R/src/lib.rs"; }
touch_body(){ sed -i '' 's/pub fn keep(&self) -> u32 { 2 }/pub fn keep(\&self) -> u32 { 2 + 0 }/' "$R/src/lib.rs"; }
rm_added()  { sed -i '' '/pub fn added_fn/d' "$R/src/lib.rs"; }

pr step1-add   "Step 1: normal PR adds added_fn"              ""                              add_fn
pr step2-break "Step 2: approved break removes old_fn"        "Accepted-Breaks: AB-01"        rm_old
pr step3-noop  "Step 3: normal PR changes a body"             ""                              touch_body
pr step4-break "Step 4: approved break removes added_fn"      "Accepted-Breaks: AB-02"        rm_added
log ""
log "DONE"
