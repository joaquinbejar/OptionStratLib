# Makefile for common tasks in a Rust project
# Detect current branch
CURRENT_BRANCH := $(shell git rev-parse --abbrev-ref HEAD)
ZIP_NAME = OptionStratLib.zip


# Default target
.PHONY: all
all: test fmt lint build

# Build the project
.PHONY: build
build:
	cargo build

.PHONY: release
release:
	cargo build --release

# Run tests
.PHONY: test
test:
	LOGLEVEL=WARN cargo test
	LOGLEVEL=WARN cargo test -p optionstratlib-core
	LOGLEVEL=WARN cargo test -p optionstratlib-core --all-features
	LOGLEVEL=WARN cargo test -p optionstratlib-math
	LOGLEVEL=WARN cargo test -p optionstratlib-math --all-features
	LOGLEVEL=WARN cargo test -p optionstratlib-pricing
	LOGLEVEL=WARN cargo test -p optionstratlib-pricing --all-features
	LOGLEVEL=WARN cargo test -p optionstratlib-simulation
	LOGLEVEL=WARN cargo test -p optionstratlib-simulation --all-features
	LOGLEVEL=WARN cargo test -p optionstratlib-market
	LOGLEVEL=WARN cargo test -p optionstratlib-market --all-features
	LOGLEVEL=WARN cargo test -p optionstratlib-analytics
	LOGLEVEL=WARN cargo test -p optionstratlib-analytics --all-features
	LOGLEVEL=WARN cargo test -p optionstratlib-strategies
	LOGLEVEL=WARN cargo test -p optionstratlib-strategies --all-features
	LOGLEVEL=WARN cargo test -p optionstratlib-backtest
	LOGLEVEL=WARN cargo test -p optionstratlib-backtest --all-features
	LOGLEVEL=WARN cargo test -p optionstratlib-visualization
	LOGLEVEL=WARN cargo test -p optionstratlib-visualization --features plotly
	LOGLEVEL=WARN cargo test -p optionstratlib-visualization --all-features
	@$(MAKE) --no-print-directory test-workspace-integration
	cargo build --no-default-features
	LOGLEVEL=WARN cargo test --features plotly
	LOGLEVEL=WARN cargo test --features static_export,plotly

# Cross-component integration tests (ADR-0004 sections 7 and 8, #534): the
# `osl-workspace-tests` member under tests/workspace, which declares every
# component it uses directly and does not build the facade. Called by `test`
# and by the Components workflow.
.PHONY: test-workspace-integration
test-workspace-integration:
	LOGLEVEL=WARN cargo test -p osl-workspace-tests

# Run the tests that need a real browser: PNG/SVG export through a WebDriver,
# and the one that hands a chart to the default browser. They are `#[ignore]`d
# so `make test` never spawns a browser (`check-browser-tests` enforces it);
# run this target explicitly, with WEBDRIVER_PATH pointing at a chromedriver
# whose major version matches the installed Chrome. Exports run one at a time
# per process (`STATIC_EXPORT_LOCK` in graph.rs), so parallel tests in one
# binary never share a chromedriver; do not run two of these targets at once.
.PHONY: test-visual
test-visual:
	LOGLEVEL=WARN cargo test -p optionstratlib-visualization --features static_export -- --ignored

# The PNG and SVG half of `test-visual`: the `static_export` acceptance run of
# #544, without the test that opens the default browser. It writes real image
# files through `plotly_static`, so it needs the same matching chromedriver
# (`WEBDRIVER_PATH`) and runs outside `make test` and the PR CI for that
# reason. The scheduled `static_export.yml` workflow (weekly, plus manual
# dispatch; it never blocks a PR) runs it with a Chrome and chromedriver pair
# installed together.
#
# Locally, install Chrome and a chromedriver of the same major version
# (`google-chrome --version` and `chromedriver --version` must agree; a
# mismatched driver fails all six tests), then:
#
#   WEBDRIVER_PATH=/path/to/chromedriver make test-export
#
# Platform requirements (ADR-0004 section 5, exhaustive tier): Linux or macOS
# with Chrome, the matching chromedriver, a headless session and outbound
# network access; Windows is untested. Retention: the workflow uploads no
# artifact. The tests write their images to a temporary directory and assert
# them in place (PNG signature, `<svg`), so a failure is read from the run log
# (GitHub keeps run logs for the repository's retention setting).
.PHONY: test-export
test-export:
	LOGLEVEL=WARN cargo test -p optionstratlib-visualization --features static_export -- --ignored png svg

# The compilation surfaces of the visualization layer (#547, ADR-0004 section
# 1 "Default-only tests" and section 2 "Plotly" / "Static export"). An
# all-features run compiles only the `plotly` + `static_export` code and never
# the backend-neutral `Graph`, so each surface is built on its own, for the
# direct crate and for the facade route to it:
#   neutral        no features         facade `visualization`
#   plotly         `plotly`            facade `plotly`
#   static_export  `static_export,plotly` facade `static_export,plotly`
# `make check-visualization-surface SURFACE=<surface>` runs, for one surface:
# check and Clippy (`-D warnings`) of both packages, their tests and doctests,
# their docs with warnings denied, the consumer fixtures of that surface, and
# the dependency assertions (`check-graph`, `check-feature-trees` and the
# fixtures' `present`/`absent` lists), so headless surfaces provably exclude
# Plotly and the export stack. The `#[ignore]`d export tests stay out: they
# need a browser, see `test-export`. `visualization.yml` runs one job per
# surface; `check-visualization` runs all three.
VIS_CRATE_FLAGS_neutral := --no-default-features
VIS_CRATE_FLAGS_plotly := --no-default-features --features plotly
VIS_CRATE_FLAGS_static_export := --no-default-features --features static_export,plotly
VIS_FACADE_FLAGS_neutral := --no-default-features --features visualization
VIS_FACADE_FLAGS_plotly := --no-default-features --features plotly
VIS_FACADE_FLAGS_static_export := --no-default-features --features static_export,plotly
VIS_FIXTURES_neutral := facade-visualization headless-full
VIS_FIXTURES_plotly := facade-plotly
VIS_FIXTURES_static_export := facade-static-export
VIS_SURFACES := neutral plotly static_export

.PHONY: check-visualization-surface
check-visualization-surface:
	@test -n "$(VIS_CRATE_FLAGS_$(SURFACE))" || { echo "SURFACE must be one of: $(VIS_SURFACES)"; exit 1; }
	cargo check -p optionstratlib-visualization --all-targets $(VIS_CRATE_FLAGS_$(SURFACE))
	cargo clippy -p optionstratlib-visualization --all-targets $(VIS_CRATE_FLAGS_$(SURFACE)) -- -D warnings
	LOGLEVEL=WARN cargo test -p optionstratlib-visualization $(VIS_CRATE_FLAGS_$(SURFACE))
	RUSTDOCFLAGS="-D warnings" cargo doc -p optionstratlib-visualization --no-deps $(VIS_CRATE_FLAGS_$(SURFACE))
	cargo check -p optionstratlib --all-targets $(VIS_FACADE_FLAGS_$(SURFACE))
	cargo clippy -p optionstratlib --all-targets $(VIS_FACADE_FLAGS_$(SURFACE)) -- -D warnings
	LOGLEVEL=WARN cargo test -p optionstratlib $(VIS_FACADE_FLAGS_$(SURFACE))
	RUSTDOCFLAGS="-D warnings" cargo doc -p optionstratlib --no-deps $(VIS_FACADE_FLAGS_$(SURFACE))
	@for fixture in $(VIS_FIXTURES_$(SURFACE)); do \
		CARGO_TARGET_DIR=$(FIXTURE_TARGET_DIR)/$$fixture cargo test --manifest-path fixtures/consumers/$$fixture/Cargo.toml || exit 1; \
	done
	@python3 scripts/check_fixtures.py $(VIS_FIXTURES_$(SURFACE))
	@$(MAKE) --no-print-directory check-graph check-feature-trees

.PHONY: check-visualization
check-visualization:
	@for surface in $(VIS_SURFACES); do \
		echo "=== visualization surface $$surface"; \
		$(MAKE) --no-print-directory check-visualization-surface SURFACE=$$surface || exit 1; \
	done

# Format the code
.PHONY: fmt
fmt:
	cargo +stable fmt --all

# Check formatting
.PHONY: fmt-check
fmt-check:
	cargo +stable fmt --all --check
	@for manifest in fixtures/consumers/*/Cargo.toml; do \
		cargo +stable fmt --manifest-path $$manifest --check || exit 1; \
	done

# Run Clippy for linting
# Each facade capability on its own (ADR-0002 Decision 2): `pricing` without
# market, `market` without I/O or simulation, `analytics` and `strategies`
# without simulation, `strategies,simulation` without backtesting,
# `backtest` without charts, the smallest set the property tests and benches
# compile against, and `visualization`, the smallest set the unit suite
# compiles against (both leave out `io` and `synthetic`, so their gates are
# linted too). Each set is also documented
# with warnings denied, so no doc link names a gated item, and its library
# and doc tests run, so every doc example compiles under the capability it
# needs (the integration suites declare `required-features` and are skipped).
# The final facade matrix (ADR-0002 section 2, ADR-0004 section 2, #549):
#   no features        `lint` (`cargo test -p optionstratlib --no-default-features`)
#   each capability    this loop: math, schema, pricing, market, io, async, synthetic,
#                      analytics, strategies, simulation, backtest,
#                      visualization, and the documented pairs
#   plotly, static_export  `check-visualization` (one job per surface, #547)
#   default            `test` (`cargo test`) and `build`
#   all features       `lint` (Clippy) and `test` (`--all-features`)
# `check-feature-trees` pins the resolved graph of the no-capability facade,
# of each capability alone and of the default, and asserts which components
# and which backends each one may resolve.
FACADE_FEATURE_SETS := math schema pricing market io async synthetic analytics strategies simulation market,simulation analytics,simulation strategies,simulation backtest visualization market,synthetic

# Every test source must be reachable from its target root through `mod`
# declarations, or it never compiles and its tests never run (#633).
.PHONY: check-test-modules
check-test-modules:
	@python3 scripts/check_test_modules.py --self-test > /dev/null || (python3 scripts/check_test_modules.py --self-test; exit 1)
	@python3 scripts/check_test_modules.py

# Fails on a `#[test]` that exports PNG/SVG or opens a browser without being
# `#[ignore]`d (#724): such a test spawns chromedriver and headless Chrome on
# every `make test` and CI run. Those belong to `test-visual` / `test-export`.
.PHONY: check-browser-tests
check-browser-tests:
	@python3 scripts/check_browser_tests.py --self-test > /dev/null || (python3 scripts/check_browser_tests.py --self-test; exit 1)
	@python3 scripts/check_browser_tests.py

.PHONY: lint
lint: check-test-modules check-browser-tests
	cargo clippy --all-targets --all-features --workspace -- -D warnings
	cargo clippy --all-targets --no-default-features --workspace -- -D warnings
	LOGLEVEL=WARN cargo test -q -p optionstratlib --no-default-features
	@for features in $(FACADE_FEATURE_SETS); do \
		echo "clippy optionstratlib --no-default-features --features $$features"; \
		cargo clippy -p optionstratlib --all-targets --no-default-features --features $$features -- -D warnings || exit 1; \
		RUSTDOCFLAGS="-D warnings" cargo doc -q -p optionstratlib --no-deps --no-default-features --features $$features || exit 1; \
		LOGLEVEL=WARN cargo test -q -p optionstratlib --no-default-features --features $$features || exit 1; \
	done

.PHONY: lint-fix
lint-fix: 
	cargo clippy --fix --all-targets --all-features --allow-staged --allow-dirty --workspace -- -D warnings

# Clean the project
.PHONY: clean
clean:
	cargo clean

# Pre-push checks
.PHONY: check
check: test fmt-check lint scan-banned check-graph check-api-report

# Fails on any production `crate::<module>` reference that crosses a
# forbidden layer boundary of the multi-crate target graph (ADR-0001 D9,
# doc/DEPENDENCY-MATRIX.md). Known reverse edges whose removal is a breaking
# change wait in the script's DEFERRED list with the issue that removes
# them (none since #658); the self-test proves the scanner catches what it
# must.
# Consumer fixtures (ADR-0004 section 3): real crates under
# fixtures/consumers/<scenario>/, excluded from the workspace and built with
# their own manifest and a target dir of their own, so nothing the workspace
# enables can widen their graph. `check-fixtures` asserts every fixture's
# expect.toml (present / absent packages) and prints its package count.
FIXTURE_TARGET_DIR := target/fixtures

.PHONY: check-fixtures
check-fixtures:
	@python3 scripts/check_fixtures.py --self-test > /dev/null || (python3 scripts/check_fixtures.py --self-test; exit 1)
	@python3 scripts/check_fixtures.py

# The 0.22 executable consumers (#552): every fixture under
# fixtures/consumers/ is a downstream crate with the manifest a 0.22 user
# writes, on the facade (one capability each, the defaults, `async`,
# `plotly`, `static_export`, `schema` off) or on component crates directly
# (pricing, market minimal / `io` / `synthetic`, analytics, simulation, a
# full backtest). `test-022-consumers` asserts every fixture's graph
# (`check-fixtures`), then lints (`-D warnings`) and tests each one with a
# target dir of its own, under its defaults and, when it declares features of
# its own, with none and with all (scripts/test_consumers.py), and finally
# tests a copy of each, outside the repository, against the packaged facade
# and component archives (`check-022-consumers-packaged`). A new fixture
# directory is picked up without editing this file. It replaces the
# per-fixture `check-consumer-*` / `test-consumer-*` targets.
.PHONY: test-022-consumers
test-022-consumers: check-fixtures
	@python3 scripts/test_consumers.py --self-test > /dev/null || (python3 scripts/test_consumers.py --self-test; exit 1)
	@python3 scripts/test_consumers.py
	@$(MAKE) --no-print-directory check-022-consumers-packaged

# The consumer fixtures built outside the repository against the packaged
# crates, through the `[patch.crates-io]` mechanism of the direct-component
# examples (#555), with the facade packaged alongside the components. With
# OSL_REUSE_PACKAGES=1 the ten archives in `<target>/package/` are reused when
# all are there, and all ten are packaged again otherwise.
.PHONY: check-022-consumers-packaged
check-022-consumers-packaged:
	OSL_PACKAGED_SOURCE=fixtures/consumers scripts/check_packaged_examples.sh

# `make tree-consumer FIXTURE=<scenario>` prints one fixture's resolved
# normal graph and asserts its expect.toml, and for a fixture with features of
# its own also counts the all-features graph; it replaces the per-fixture
# `tree-consumer-*` targets.
.PHONY: tree-consumer
tree-consumer:
	@test -n "$(FIXTURE)" || { echo "FIXTURE must name a directory under fixtures/consumers"; exit 1; }
	cargo tree --manifest-path fixtures/consumers/$(FIXTURE)/Cargo.toml -e normal --prefix none | sed 's/ (\*)$$//' | sort -u
	@python3 scripts/check_fixtures.py $(FIXTURE)
	@if grep -q '^\[features\]' fixtures/consumers/$(FIXTURE)/Cargo.toml; then \
		echo "all features: $$(cargo tree --manifest-path fixtures/consumers/$(FIXTURE)/Cargo.toml -e normal --prefix none --all-features | sed 's/ (\*)$$//' | sort -u | wc -l | tr -d ' ') resolved package entries"; \
	fi

# Direct-component examples (#555, ADR-0004 section 8): runnable programs under
# `examples/direct/<scenario>/`, workspace members named
# `osl-example-direct-<scenario>`, each depending on the component crates it
# uses and on no facade feature. They show the smallest dependency set of each
# capability (see examples/direct/README.md); the fixtures above prove the
# same graphs without a program. `test-direct-component-examples` lints,
# tests and runs every one; `tree-example-direct-<scenario>` prints the
# resolved graph and asserts its `expect.toml` (`check-fixtures` asserts all).
DIRECT_EXAMPLES := math pricing market analytics strategies simulation backtest visualization

.PHONY: test-direct-component-examples
test-direct-component-examples:
	@set -e; for scenario in $(DIRECT_EXAMPLES); do \
		package=osl-example-direct-$$scenario; \
		echo "=== $$package"; \
		cargo clippy -p $$package --all-targets -- -D warnings; \
		LOGLEVEL=WARN cargo test -p $$package; \
		cargo run -q -p $$package; \
	done

# Builds and runs every manifest example of the 0.22 release notes (#562) as
# a standalone crate with exactly the `[dependencies]` shown, patched to this
# checkout; the facade's doctests compile the same programs with every feature.
# The self-test first proves that a facade-only manifest using the prelude's
# `dec!` needs `rust_decimal` (#777), the rule every example is held to.
.PHONY: check-release-notes
check-release-notes:
	python3 scripts/check_release_notes.py --self-test
	python3 scripts/check_release_notes.py

# Builds a copy of each example, outside the repository, against the packaged
# facade and component crates through `[patch.crates-io]` (#555): the "compile against
# packaged crates" check that the path dependencies of the in-tree manifests
# cannot give. It packages the facade and the components itself; with
# `OSL_REUSE_PACKAGES=1` it reuses the ten archives in `<target>/package/` when
# all are there. `check-components` leaves no facade archive, so the
# Components workflow's run right after it packages all ten again.
.PHONY: check-direct-examples-packaged
check-direct-examples-packaged:
	scripts/check_packaged_examples.sh

.PHONY: $(addprefix tree-example-direct-,$(DIRECT_EXAMPLES))
$(addprefix tree-example-direct-,$(DIRECT_EXAMPLES)): tree-example-direct-%:
	cargo tree --manifest-path examples/direct/$*/Cargo.toml -e normal --prefix none | sed 's/ (\*)$$//' | sort -u
	@python3 scripts/check_fixtures.py direct-$*

# Measures what a per-family feature split of optionstratlib-strategies could
# save: packages, the crate's own check and build time, rlib sizes (#532).
# Informational, not run in CI; the crate docs record the numbers.
.PHONY: measure-strategies
measure-strategies:
	@python3 scripts/measure_strategies.py

.PHONY: check-graph
check-graph:
	@python3 scripts/check_module_boundaries.py --self-test > /dev/null || (python3 scripts/check_module_boundaries.py --self-test; exit 1)
	@python3 scripts/check_module_boundaries.py
	@python3 scripts/check_ownership_map.py --self-test > /dev/null || (python3 scripts/check_ownership_map.py --self-test; exit 1)
	@python3 scripts/check_ownership_map.py

# Verifies every extracted component crate on its own, without the facade in
# the build (#519): tests with default, no and all features, Clippy, docs
# with broken links and missing docs denied, and the packaged archive. The
# crates are packaged together because a component's path dependencies are
# not on crates.io yet; `cargo package` resolves them from the same run.
COMPONENT_CRATES := optionstratlib-core optionstratlib-math optionstratlib-pricing optionstratlib-simulation optionstratlib-market optionstratlib-analytics optionstratlib-strategies optionstratlib-backtest optionstratlib-visualization
# Named feature sets each component must also build, lint and test alone
# (`crate:feature`), besides no, default and all features (ADR-0003, #525).
# Visualization's `plotly` alone is the Plotly surface without static export
# (#542); `static_export` implies it, so all features covers that one.
COMPONENT_FEATURE_SETS := optionstratlib-market:io optionstratlib-market:async optionstratlib-market:synthetic optionstratlib-visualization:plotly

# `cargo package` verifies each crate against the others through a temporary
# local registry. Cargo treats a crate from a registry as immutable: it never
# re-reads its source, so an `optionstratlib-*` 0.22.0 compiled from that
# registry by an earlier run (and restored by the CI cache of `target/`) is
# reused as is, and a crate is verified against a stale neighbour (seen on
# main after #672). The version stays 0.22.0 while the code changes, so drop
# every copy first: the unpacked and cached sources under `$CARGO_HOME`
# (local registries only, the directories whose name starts with `-`;
# crates.io's never does), the packaged `.crate` files and temporary registry
# in `<target>/package/`, and the compiled `optionstratlib` artifacts and
# fingerprints in `<target>/debug/`. Nothing from crates.io is touched; the
# workspace crates are simply rebuilt.
CARGO_HOME_DIR := $(or $(CARGO_HOME),$(HOME)/.cargo)
TARGET_DIR := $(or $(CARGO_TARGET_DIR),target)
PACKAGE_DIR := $(TARGET_DIR)/package

.PHONY: check-components
check-components:
	@rm -rf $(CARGO_HOME_DIR)/registry/src/-*/optionstratlib-* $(CARGO_HOME_DIR)/registry/cache/-*/optionstratlib-*
	@rm -rf $(PACKAGE_DIR)
	@set -e; for crate in $(COMPONENT_CRATES); do \
		echo "=== $$crate"; \
		LOGLEVEL=WARN cargo test -p $$crate; \
		LOGLEVEL=WARN cargo test -p $$crate --no-default-features; \
		LOGLEVEL=WARN cargo test -p $$crate --all-features; \
		cargo clippy -p $$crate --all-targets --all-features -- -D warnings; \
		RUSTDOCFLAGS="-D warnings" cargo doc -p $$crate --all-features --no-deps; \
		RUSTDOCFLAGS="-D warnings" cargo doc -p $$crate --no-default-features --no-deps; \
		lib=crates/$$crate/src/lib.rs; \
		grep -q '^#!\[deny(missing_docs, rustdoc::broken_intra_doc_links)\]' $$lib \
			|| { echo "$$lib must deny missing_docs and broken intra-doc links"; exit 1; }; \
		files=$$(cargo package -p $$crate --list --allow-dirty); \
		for required in Cargo.toml README.md LICENSE src/lib.rs; do \
			echo "$$files" | grep -qx "$$required" \
				|| { echo "$$crate package is missing $$required"; exit 1; }; \
		done; \
	done
	@set -e; for spec in $(COMPONENT_FEATURE_SETS); do \
		crate=$${spec%%:*}; features=$${spec#*:}; \
		echo "=== $$crate --features $$features"; \
		LOGLEVEL=WARN cargo test -p $$crate --no-default-features --features $$features; \
		cargo clippy -p $$crate --all-targets --no-default-features --features $$features -- -D warnings; \
	done
	@rm -rf $(PACKAGE_DIR)
	@for dir in $(TARGET_DIR)/debug/.fingerprint $(TARGET_DIR)/debug/deps; do \
		[ -d $$dir ] || continue; \
		find $$dir -maxdepth 1 \( -name 'optionstratlib-*' -o -name 'optionstratlib_*' \
			-o -name 'liboptionstratlib_*' \) -exec rm -rf {} +; \
	done
	cargo package $(addprefix -p ,$(COMPONENT_CRATES)) --allow-dirty
	@echo "OK: $(COMPONENT_CRATES) verified standalone"

# Pins the dependency graph of the market surface without `synthetic` and with
# it (roadmap M1-15), one fixture each, as a `parent -> child` edge list
# resolved with `cargo tree --target all` so it is host-independent. The
# difference between the two is derived and printed: since #536 it is the
# edge to `optionstratlib-simulation` and that crate's own graph, which is
# the point. The `plotly` and `static_export` surfaces are pinned too, and
# every surface carries dependency assertions that `feature-trees-update`
# cannot record over (#544): the Plotly, image-export and runtime packages
# and the visualization crate appear only where a surface asks for them.
# Run `make feature-trees-update` to record an intended change.
.PHONY: check-feature-trees
check-feature-trees:
	@python3 scripts/check_feature_trees.py --self-test > /dev/null || (python3 scripts/check_feature_trees.py --self-test; exit 1)
	@python3 scripts/check_feature_trees.py

.PHONY: feature-trees-update
feature-trees-update:
	@python3 scripts/check_feature_trees.py --update

# Reports the public API changes of a pull request per feature surface
# (#606). `check-api-report` runs the parser's self-test only; the comparison
# needs a baseline and runs in CI (`api_changes.yml`) or with
# `scripts/report_api_changes.py --surface default --baseline <rev>`.
PYTHON311 ?= $(shell command -v python3.13 || command -v python3.12 || command -v python3.11 || echo python3)
.PHONY: check-api-report
check-api-report:
	@$(PYTHON311) scripts/report_api_changes.py --self-test > /dev/null || ($(PYTHON311) scripts/report_api_changes.py --self-test; exit 1)
	@echo "api-changes: parser self-test OK"

# Fails when a panicking construct reappears in production code.
#
# The banned set, and why each entry is there:
#   * `.unwrap()` / `.expect(` — abort instead of reporting.
#   * `panic!` / `unreachable!` / `todo!` / `unimplemented!` — the same,
#     spelled out. The pattern requires a non-word, non-`_` character before
#     the macro name so `pos_or_panic!` is not swept up with `panic!`.
#   * `println!` / `eprintln!` / `print!` / `eprint!` / `dbg!` and
#     `tracing_subscriber` — library code logs through `tracing` and never
#     writes to stdout or installs a global subscriber (rules/global_rules.md,
#     "Logging & Observability"; #522).
#   * `.exp()` / `.ln()` / `.powd(` — `rust_decimal`'s `MathematicalOps`
#     aborts on all three (`Exp overflowed`, `Unable to calculate ln for
#     zero`, `Pow overflowed`); `d_exp` / `d_ln` / `d_powd` in
#     `src/model/decimal.rs` are the checked forms. `f64` has the same three
#     method names and does *not* abort, so those call sites carry a marker
#     saying so — grep cannot tell the receiver types apart.
#   * `.sqrt().unwrap()` — `Decimal::sqrt` returns `Option` for negative
#     input; unwrapping it puts that panic back.
#   * `.sqrt()` / `.checked_sqrt()` on `Decimal` / `Positive` — upstream
#     `Decimal::sqrt` aborts with `geo mean circuit breaker` when its Newton
#     iteration oscillates (#588); `d_sqrt` / `p_sqrt` in
#     `src/model/decimal.rs` are the total forms. `f64::sqrt` call sites
#     carry a marker saying so, like `exp`/`ln`/`powd`.
#
# A bare `#[cfg(test)]` never truncates the scan; only the braced body of the
# item it actually gates is skipped, brace counted (files may carry several):
# `mod`/`fn`/`impl`/`trait`/`struct`/`enum`/`union` (any `pub(..)` visibility,
# `async`/`unsafe`, generics or return type before the `{`, spanning multiple
# signature lines if needed), and a bare `#[cfg(test)] { ... }` block. A
# body-less item behind `#[cfg(test)]` (`use`, `const`, `static`, `type`
# alias, or a `;`-terminated declaration such as a tuple struct or a trait
# method signature) skips nothing; scanning resumes right after it as usual.
#
# Line comments (`///`, `//!`, `//`) are skipped, and so is the closing line
# of a `/*** … ***/` banner (`*`-run followed by `/`). The filter used to skip
# every line starting with `*`, which also skipped the continuation lines of a
# multi-line product — `        * asr.exp()` in `crates/optionstratlib-pricing/src/pricing/compound.rs` is
# five such lines — so a banned construct could hide there.
#
# A reviewed exception carries a trailing
# `// scan-banned: allow -- <reason>` marker on the same line.
.PHONY: scan-banned
scan-banned:
	@found=$$(for f in $$(find src crates/*/src -name '*.rs'); do \
		awk -v file="$$f" ' \
			BEGIN { skip = 0; depth = 0; pending = 0; awaiting = 0 } \
			function braces(line,   tmp, o, c) { \
				tmp = line; o = gsub(/{/, "{", tmp); \
				tmp = line; c = gsub(/}/, "}", tmp); \
				return o - c; \
			} \
			{ \
				raw = $$0; \
				if (skip) { \
					depth += braces(raw); \
					if (depth <= 0) { skip = 0; depth = 0 } \
					next; \
				} \
				if (awaiting) { \
					depth += braces(raw); \
					if (depth > 0) { awaiting = 0; skip = 1; next } \
					if (raw ~ /;[[:space:]]*$$/) { awaiting = 0; depth = 0 } \
					next; \
				} \
				if (pending) { \
					if (raw ~ /^[[:space:]]*#\[/) { next } \
					pending = 0; \
					trimmed = raw; sub(/^[[:space:]]+/, "", trimmed); \
					if (trimmed ~ /^\{/) { \
						depth = braces(raw); \
						if (depth > 0) { skip = 1 } \
						next; \
					} \
					if (trimmed ~ /^(pub([[:space:]]*\([^)]*\))?[[:space:]]+)?(async[[:space:]]+)?(unsafe[[:space:]]+)?(mod|fn|impl|trait|struct|enum|union)([[:space:]<(]|$$)/) { \
						depth = braces(raw); \
						if (depth > 0) { skip = 1 } \
						else if (raw ~ /;[[:space:]]*$$/) { depth = 0 } \
						else { awaiting = 1; depth = 0 } \
						next; \
					} \
				} \
				if (raw ~ /^[[:space:]]*#\[cfg\(test\)\][[:space:]]*$$/) { pending = 1; next } \
				print file ":" NR ":" raw; \
			} \
		' "$$f"; \
	done \
		| grep -E '\.unwrap\(\)|\.expect\(|\.exp\(\)|\.ln\(\)|\.powd\(|\.sqrt\(\)|\.checked_sqrt\(\)|[^_[:alnum:]](panic|unreachable|todo|unimplemented|println|eprintln|print|eprint|dbg)!|tracing_subscriber' \
		| grep -v -E ':[0-9]+:[[:space:]]*(///|//!|//|\*+/)' \
		| grep -v -E 'scan-banned: allow -- [^[:space:]]' || true); \
	malformed=$$(grep -rn 'scan-banned: allow' src crates/*/src \
		| grep -v -E 'scan-banned: allow -- [^[:space:]]' || true); \
	if [ -n "$$malformed" ]; then \
		echo "Exemption markers without a reason (use 'scan-banned: allow -- <reason>'):"; \
		echo "$$malformed"; \
		exit 1; \
	fi; \
	if [ -n "$$found" ]; then \
		echo "Banned patterns found in production code:"; \
		echo "$$found"; \
		exit 1; \
	fi; \
	echo "OK: no unwrap/expect, no panic/unreachable/todo/unimplemented, no print/dbg macros or tracing_subscriber, no unchecked exp/ln/powd/sqrt in production code"

# Pinned producers of public-api/optionstratlib.txt. Both anchors are needed
# and they only work as a pair: `cargo public-api` does not read the source,
# it reads rustdoc JSON, whose schema is unstable and versioned
# (`format_version`). A floating nightly can therefore start emitting a schema
# the installed CLI cannot parse, and a floating CLI can render the same
# rustdoc JSON differently. Pinning both is what keeps an unchanged commit
# from producing a different snapshot in a different environment.
#
# Bumping either pin is a deliberate act: change the value here, run
# `make public-api-update`, and commit the resulting snapshot diff.
# `.github/workflows/public_api.yml` reads both values from this file (see
# `print-public-api-pins`) so CI cannot drift from a local run.
CARGO_PUBLIC_API_VERSION := 0.52.0
PUBLIC_API_NIGHTLY := nightly-2026-08-28

# Installs the tooling `public-api-check`/`public-api-update` need: the
# `cargo public-api` subcommand at the pinned version, plus the pinned nightly
# toolchain to build rustdoc JSON from (it does not need to be the
# active/default toolchain, so it coexists with the `stable` channel pinned in
# rust-toolchain.toml, and it is never used to build the crate itself).
#
# The version comparison is not just a first-run install guard: CI restores
# `~/.cargo/bin` from a cache whose `restore-keys` can hand back an older
# binary, so an already-installed CLI at the wrong version is reinstalled
# rather than silently accepted.
.PHONY: check-cargo-public-api
check-cargo-public-api:
	@installed=$$(cargo-public-api --version 2>/dev/null | awk '{print $$2}'); \
	if [ "$$installed" != "$(CARGO_PUBLIC_API_VERSION)" ]; then \
		echo "Installing cargo-public-api $(CARGO_PUBLIC_API_VERSION) (found: $${installed:-none})..."; \
		cargo install cargo-public-api --locked --version $(CARGO_PUBLIC_API_VERSION); \
	fi
	@rustup toolchain list | grep -q '^$(PUBLIC_API_NIGHTLY)' || (echo "Installing $(PUBLIC_API_NIGHTLY) for rustdoc JSON..."; rustup toolchain install $(PUBLIC_API_NIGHTLY) --profile minimal)

# Prints the pinned tooling versions as `key=value` lines, one per line, for
# .github/workflows/public_api.yml to append to $$GITHUB_OUTPUT. Keeps the
# pins defined in exactly one place.
.PHONY: print-public-api-pins
print-public-api-pins:
	@echo "cargo_public_api_version=$(CARGO_PUBLIC_API_VERSION)"
	@echo "public_api_nightly=$(PUBLIC_API_NIGHTLY)"

# Regenerates public-api/optionstratlib.txt, the checked-in snapshot of the
# crate's full public API (built with --all-features, matching semver.yml's
# feature-group). Run this and commit the result in the same PR whenever a
# change to a `pub` item is intentional. `-sss` omits Blanket/Auto
# Trait/Auto Derived impls so the snapshot and its diffs stay reviewable;
# losing one of those (e.g. a dropped `Send`/`Debug`) is already caught by
# the `semver` CI job's auto_trait_impl_removed / derive_trait_impl_removed
# lints, so nothing is lost by omitting them here.
# Workspace component crates with their own snapshot, `public-api/<crate>.txt`.
# The facade re-exports their modules, and `cargo public-api` does not inline
# another crate's items, so each component is tracked on its own.
PUBLIC_API_CRATES := optionstratlib-core optionstratlib-math optionstratlib-pricing optionstratlib-simulation optionstratlib-market optionstratlib-analytics optionstratlib-strategies optionstratlib-backtest optionstratlib-visualization

.PHONY: public-api-update
public-api-update: check-cargo-public-api
	@mkdir -p public-api
	cargo +$(PUBLIC_API_NIGHTLY) public-api -sss --all-features > public-api/optionstratlib.txt
	@for crate in $(PUBLIC_API_CRATES); do \
		cargo +$(PUBLIC_API_NIGHTLY) public-api -p $$crate -sss --all-features > public-api/$$crate.txt || exit 1; \
	done

# Fails on any public `f64` in a component crate's API snapshot outside error
# diagnostics and the reviewed public-api/float-boundary-allowlist.txt (#522).
.PHONY: check-float-boundary
check-float-boundary:
	@python3 scripts/check_float_boundary.py --self-test > /dev/null || (python3 scripts/check_float_boundary.py --self-test; exit 1)
	@python3 scripts/check_float_boundary.py

# Fails when the crate's public API has drifted from public-api/optionstratlib.txt
# without the snapshot being updated to match, i.e. an *unacknowledged* API
# change slipped in. This catches classes of breakage cargo-semver-checks
# does not lint for on its own, e.g. a function's return type changing from
# `T` to `Result<T, E>` (see
# https://github.com/obi1kenobi/cargo-semver-checks/issues/1613, confirmed a
# known gap by the maintainer). Run `make public-api-update` and commit the
# resulting diff to acknowledge a deliberate change.
.PHONY: public-api-check
public-api-check: check-cargo-public-api check-float-boundary
	@mkdir -p target/public-api
	@cargo +$(PUBLIC_API_NIGHTLY) public-api -sss --all-features > target/public-api/optionstratlib.txt
	@if ! diff -u public-api/optionstratlib.txt target/public-api/optionstratlib.txt; then \
		echo; \
		echo "Public API drifted from public-api/optionstratlib.txt (see diff above)."; \
		echo "If this change is intentional, run 'make public-api-update', review the"; \
		echo "resulting diff, and commit it together with this change."; \
		exit 1; \
	fi
	@for crate in $(PUBLIC_API_CRATES); do \
		cargo +$(PUBLIC_API_NIGHTLY) public-api -p $$crate -sss --all-features > target/public-api/$$crate.txt || exit 1; \
		if ! diff -u public-api/$$crate.txt target/public-api/$$crate.txt; then \
			echo; \
			echo "Public API of $$crate drifted from public-api/$$crate.txt (see diff above)."; \
			echo "If this change is intentional, run 'make public-api-update' and commit the diff."; \
			exit 1; \
		fi; \
	done
	@echo "OK: public API matches public-api/optionstratlib.txt and the component snapshots ($(PUBLIC_API_CRATES))"

# Run the project
.PHONY: run
run:
	cargo run

.PHONY: fix
fix:
	cargo fix --allow-staged --allow-dirty

.PHONY: pre-push
pre-push: fix fmt lint-fix check-browser-tests test readme doc

# Builds the crate's documentation with every feature on. It used to run
# `cargo clippy -- -W missing-docs`, which builds no documentation at all and
# so never resolved an intra-doc link. `--all-features` is what makes the
# difference: `create-doc` below omits it, so a link inside `plotly` /
# `static_export` / `async` code was checked by nothing.
#
# The target starts green: two `private_intra_doc_links` warnings stand
# (`RNDStatistics::new` in src/chains/rnd.rs and `lower_break_even` in
# src/strategies/base.rs), and warnings do not fail it. `src/lib.rs` denies
# `rustdoc::broken_intra_doc_links`, so a broken link is an error and exits 101.
.PHONY: doc
doc:
	cargo doc --all-features --no-deps -p optionstratlib -p optionstratlib-core -p optionstratlib-math -p optionstratlib-pricing -p optionstratlib-simulation -p optionstratlib-market -p optionstratlib-analytics -p optionstratlib-strategies -p optionstratlib-backtest -p optionstratlib-visualization

.PHONY: doc-open
doc-open:
	cargo doc --open

.PHONY: publish
publish: readme
	cargo login ${CARGO_REGISTRY_TOKEN}
	cargo package
	cargo publish

.PHONY: coverage
coverage:
	export LOGLEVEL=WARN
	export RUST_lOG=WARN
	cargo install cargo-tarpaulin --locked --version '>=0.37.5'
	mkdir -p coverage
	# `--timeout` budgets one whole test binary's run under the LLVM engine,
	# not one test; tarpaulin enforces it from 0.37.4 on, so it carries a real
	# value. Raise it when a binary grows, not when a single test gets slower.
	cargo tarpaulin --verbose --all-features --workspace --timeout 1200 --out Xml --output-dir coverage

.PHONY: coverage-html
coverage-html:
	export LOGLEVEL=WARN
	export RUST_lOG=WARN
	cargo install cargo-tarpaulin --locked --version '>=0.37.5'
	mkdir -p coverage
	cargo tarpaulin --color Always --engine llvm --tests --all-targets --all-features --workspace --timeout 1200 --out Html --output-dir coverage

.PHONY: open-coverage
open-coverage:
	open coverage/tarpaulin-report.html

# Rule to show git log
git-log:
	@if [ "$(CURRENT_BRANCH)" = "HEAD" ]; then \
		echo "You are in a detached HEAD state. Please check out a branch."; \
		exit 1; \
	fi; \
	echo "Showing git log for branch $(CURRENT_BRANCH) against main:"; \
	git log main..$(CURRENT_BRANCH) --pretty=full

.PHONY: create-doc
create-doc:
	cargo doc --no-deps --document-private-items

.PHONY: readme
readme: check-cargo-readme create-doc
	cargo readme > README.md

# 3.4.0 is the first release that resolves `version.workspace = true` and the
# other `[workspace.package]` fields the root manifest inherits; 3.3.x fails
# with "invalid type: map, expected a string".
CARGO_README_VERSION := 3.4.0

.PHONY: check-cargo-readme
check-cargo-readme:
	@installed=$$(cargo-readme --version 2>/dev/null | awk '{print $$NF}'); \
	if [ "$$installed" != "$(CARGO_README_VERSION)" ]; then \
		echo "Installing cargo-readme $(CARGO_README_VERSION) (found: $${installed:-none})..."; \
		cargo install cargo-readme --locked --version $(CARGO_README_VERSION); \
	fi

.PHONY: check-spanish
check-spanish:
	@rg -n --pcre2 -e '^\s*(//|///|//!|#|/\*|\*).*?[áéíóúÁÉÍÓÚñÑ¿¡]' \
    	    --glob '!target/*' \
    	    --glob '!**/*.png' \
    	    . || (echo "❌  Spanish comments found"; exit 1)

.PHONY: zip
zip:
	@echo "Creating $(ZIP_NAME) without any 'target' directories, 'Cargo.lock', and hidden files..."
	@find . -type f \
		! -path "*/target/*" \
		! -path "./.*" \
		! -name "Cargo.lock" \
		! -name ".*" \
		| zip -@ $(ZIP_NAME)
	@echo "$(ZIP_NAME) created successfully."


.PHONY: check-cargo-criterion
check-cargo-criterion:
	@command -v cargo-criterion > /dev/null || (echo "Installing cargo-criterion..."; cargo install cargo-criterion)

.PHONY: bench
bench: check-cargo-criterion
	cargo criterion --output-format=quiet

.PHONY: bench-show
bench-show:
	open target/criterion/report/index.html

.PHONY: bench-save
bench-save: check-cargo-criterion
	cargo criterion --output-format quiet --history-id v0.3.2 --history-description "Version 0.3.2 baseline"

.PHONY: bench-compare
bench-compare: check-cargo-criterion
	cargo criterion --output-format verbose

.PHONY: bench-json
bench-json: check-cargo-criterion
	cargo criterion --message-format json

.PHONY: bench-clean
bench-clean:
	rm -rf target/criterion


.PHONY: workflow-coverage
workflow-coverage:
	DOCKER_HOST="$${DOCKER_HOST}" act push --job code_coverage_report \
       -P ubuntu-latest=catthehacker/ubuntu:latest \
       --privileged

.PHONY: workflow-build
workflow-build:
	DOCKER_HOST="$${DOCKER_HOST}" act push --job build \
       -P ubuntu-latest=catthehacker/ubuntu:latest

.PHONY: workflow-lint
workflow-lint:
	DOCKER_HOST="$${DOCKER_HOST}" act push --job lint

.PHONY: workflow-test
workflow-test:
	DOCKER_HOST="$${DOCKER_HOST}" act push --job run_tests

.PHONY: workflow
workflow: workflow-build workflow-lint workflow-test workflow-coverage

.PHONY: generate_markdown
generate_markdown:
	./doc/generate_md_docs.sh
