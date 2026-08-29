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
	LOGLEVEL=WARN cargo test --features plotly
	LOGLEVEL=WARN cargo test --features static_export,plotly

# Run the tests that need a real browser: PNG/SVG export through a WebDriver,
# and the one that hands a chart to the default browser. They are `#[ignore]`d
# so `make test` never spawns a browser; run this target explicitly, with
# WEBDRIVER_PATH pointing at a chromedriver whose major version matches the
# installed Chrome.
.PHONY: test-visual
test-visual:
	LOGLEVEL=WARN cargo test --features static_export,plotly -- --ignored

# Format the code
.PHONY: fmt
fmt:
	cargo +stable fmt --all

# Check formatting
.PHONY: fmt-check
fmt-check:
	cargo +stable fmt --check

# Run Clippy for linting
.PHONY: lint
lint:
	cargo clippy --all-targets --all-features --workspace -- -D warnings

.PHONY: lint-fix
lint-fix: 
	cargo clippy --fix --all-targets --all-features --allow-staged --allow-dirty --workspace -- -D warnings

# Clean the project
.PHONY: clean
clean:
	cargo clean

# Pre-push checks
.PHONY: check
check: test fmt-check lint scan-banned

# Fails when a panicking construct reappears in production code. A bare
# `#[cfg(test)]` never truncates the scan; only the braced body of the item
# it actually gates is skipped, brace counted (files may carry several):
# `mod`/`fn`/`impl`/`trait`/`struct`/`enum`/`union` (any `pub(..)` visibility,
# `async`/`unsafe`, generics or return type before the `{`, spanning multiple
# signature lines if needed), and a bare `#[cfg(test)] { ... }` block. A
# body-less item behind `#[cfg(test)]` (`use`, `const`, `static`, `type`
# alias, or a `;`-terminated declaration such as a tuple struct or a trait
# method signature) skips nothing; scanning resumes right after it as usual.
# Doc comments are skipped too. A reviewed exception carries a trailing
# `// scan-banned: allow -- <reason>` marker on the same line.
.PHONY: scan-banned
scan-banned:
	@found=$$(for f in $$(find src -name '*.rs'); do \
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
		| grep -E '\.unwrap\(\)|\.expect\(' \
		| grep -v -E ':[0-9]+:[[:space:]]*(///|//!|//|\*)' \
		| grep -v 'scan-banned: allow' || true); \
	if [ -n "$$found" ]; then \
		echo "Banned patterns found in production code:"; \
		echo "$$found"; \
		exit 1; \
	fi; \
	echo "OK: no .unwrap()/.expect() in production code"

# Installs the tooling `public-api-check`/`public-api-update` need: the
# `cargo public-api` subcommand, plus a nightly toolchain to build rustdoc
# JSON from (it does not need to be the active/default toolchain, so it
# coexists with the `stable` channel pinned in rust-toolchain.toml).
.PHONY: check-cargo-public-api
check-cargo-public-api:
	@command -v cargo-public-api > /dev/null || (echo "Installing cargo-public-api..."; cargo install cargo-public-api --locked)
	@rustup toolchain list | grep -q '^nightly' || (echo "Installing a nightly toolchain for rustdoc JSON..."; rustup toolchain install nightly --profile minimal)

# Regenerates public-api/optionstratlib.txt, the checked-in snapshot of the
# crate's full public API (built with --all-features, matching semver.yml's
# feature-group). Run this and commit the result in the same PR whenever a
# change to a `pub` item is intentional. `-sss` omits Blanket/Auto
# Trait/Auto Derived impls so the snapshot and its diffs stay reviewable;
# losing one of those (e.g. a dropped `Send`/`Debug`) is already caught by
# the `semver` CI job's auto_trait_impl_removed / derive_trait_impl_removed
# lints, so nothing is lost by omitting them here.
.PHONY: public-api-update
public-api-update: check-cargo-public-api
	@mkdir -p public-api
	cargo public-api -sss --all-features > public-api/optionstratlib.txt

# Fails when the crate's public API has drifted from public-api/optionstratlib.txt
# without the snapshot being updated to match, i.e. an *unacknowledged* API
# change slipped in. This catches classes of breakage cargo-semver-checks
# does not lint for on its own, e.g. a function's return type changing from
# `T` to `Result<T, E>` (see
# https://github.com/obi1kenobi/cargo-semver-checks/issues/1613, confirmed a
# known gap by the maintainer). Run `make public-api-update` and commit the
# resulting diff to acknowledge a deliberate change.
.PHONY: public-api-check
public-api-check: check-cargo-public-api
	@mkdir -p target/public-api
	@cargo public-api -sss --all-features > target/public-api/optionstratlib.txt
	@if ! diff -u public-api/optionstratlib.txt target/public-api/optionstratlib.txt; then \
		echo; \
		echo "Public API drifted from public-api/optionstratlib.txt (see diff above)."; \
		echo "If this change is intentional, run 'make public-api-update', review the"; \
		echo "resulting diff, and commit it together with this change."; \
		exit 1; \
	fi
	@echo "OK: public API matches public-api/optionstratlib.txt"

# Run the project
.PHONY: run
run:
	cargo run

.PHONY: fix
fix:
	cargo fix --allow-staged --allow-dirty

.PHONY: pre-push
pre-push: fix fmt lint-fix test readme doc

.PHONY: doc
doc:
	cargo clippy -- -W missing-docs

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
	cargo install cargo-tarpaulin
	mkdir -p coverage
	cargo tarpaulin --verbose --all-features --workspace --timeout 0 --out Xml --output-dir coverage

.PHONY: coverage-html
coverage-html:
	export LOGLEVEL=WARN
	export RUST_lOG=WARN
	cargo install cargo-tarpaulin
	mkdir -p coverage
	cargo tarpaulin --color Always --engine llvm --tests --all-targets --all-features --workspace --timeout 0 --out Html --output-dir coverage

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

.PHONY: check-cargo-readme
check-cargo-readme:
	@command -v cargo-readme > /dev/null || (echo "Installing cargo-readme..."; cargo install cargo-readme)

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
