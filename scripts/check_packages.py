#!/usr/bin/env python3
"""Check the contents and the metadata of the ten published packages (#559).

The facade `optionstratlib` and its nine `optionstratlib-*` components are
separate crates.io products. For each one this checks:

* Contents (`cargo package --list`): the source, `Cargo.toml`, `README.md`
  and `LICENSE` are in the archive, and no local, build or planning artifact
  is (`Draws/`, `target/`, `doc/`, `.issues/`, `.github/`, `scripts/`,
  `fixtures/`, `public-api/`, tests, benches, examples, the Makefile, the
  toolchain file, the changelog, the README template).
* Metadata (`cargo metadata --no-deps`): the lockstep version, edition,
  `rust-version`, license, readme, repository and homepage are the workspace
  values; the description names OptionStratLib and, for a component, its
  place in the facade; categories are valid crates.io slugs (at most five);
  keywords follow the crates.io rules (at most five, at most 20 characters,
  ASCII starting with a letter).
* Dependencies: every path dependency also carries a version requirement, and
  one on a sibling package requires the lockstep version.
* Features: every `dep:` names an optional dependency, every `x/feature` a
  dependency and every `x?/feature` an optional one, every plain entry a
  feature or an optional dependency, every optional dependency is enabled by
  some feature, and no feature is named to subtract (`no-*`, `no_*`,
  `without-*`), which the additive model forbids.
* The ownership map: each row of `docs/ownership.md` section 2 names a facade
  feature that enables the component feature in its last column.
* Documentation links: every `github.com/joaquinbejar/OptionStratLib/blob/main/`
  link in a published README names a tracked file.
* Test data: a crate whose unit tests read a fixture ships it under
  `testdata/`, so `cargo test` works from the archive (#558); every such file
  is in the archive and is byte for byte the repository's copy under
  `examples/Chains/`.

Usage:
    check_packages.py              check every package
    check_packages.py --report     also print the evidence tables (Markdown)
    check_packages.py --self-test  check the validators
"""

from __future__ import annotations

import json
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

# The release line every package is on (docs/versioning-policy.md, #834):
# the ten crates share `0.MINOR` and each may be at its own patch.
LINE = "0.22"
LINE_VERSION_RE = re.compile(r"^0\.22\.(0|[1-9][0-9]*)$")
EDITION = "2024"
RUST_VERSION = "1.89"
LICENSE = "MIT"
URL = "https://github.com/joaquinbejar/OptionStratLib"
FACADE = "optionstratlib"
COMPONENTS = [
    "optionstratlib-core",
    "optionstratlib-math",
    "optionstratlib-pricing",
    "optionstratlib-simulation",
    "optionstratlib-market",
    "optionstratlib-analytics",
    "optionstratlib-strategies",
    "optionstratlib-backtest",
    "optionstratlib-visualization",
]
PACKAGES = [FACADE, *COMPONENTS]

# The crates.io category slugs (https://crates.io/category_slugs), top level.
VALID_CATEGORIES = {
    "accessibility", "aerospace", "algorithms", "api-bindings", "asynchronous",
    "authentication", "caching", "command-line-interface", "command-line-utilities",
    "compilers", "compression", "computer-vision", "concurrency", "config",
    "cryptography", "data-structures", "database", "database-implementations",
    "date-and-time", "development-tools", "email", "embedded", "emulators",
    "encoding", "external-ffi-bindings", "filesystem", "finance",
    "game-development", "game-engines", "games", "graphics", "gui",
    "hardware-support", "internationalization", "localization", "mathematics",
    "memory-management", "multimedia", "network-programming", "no-std", "os",
    "parser-implementations", "parsing", "rendering", "rust-patterns", "science",
    "simulation", "template-engine", "text-editors", "text-processing",
    "value-formatting", "visualization", "wasm", "web-programming",
}
KEYWORD_RE = re.compile(r"^[A-Za-z][A-Za-z0-9_+-]{0,19}$")
MAX_KEYWORDS = 5
MAX_CATEGORIES = 5

REQUIRED_FILES = {"Cargo.toml", "Cargo.toml.orig", "README.md", "LICENSE", "src/lib.rs"}
# Paths a published archive must not carry.
FORBIDDEN_RE = re.compile(
    r"^(Draws|target|doc|\.issues|\.github|\.agents|scripts|fixtures|public-api|"
    r"tests|benches|examples|Docker)/"
    r"|^(Makefile|rust-toolchain\.toml|CHANGELOG\.md|README\.tpl|codecov\.yml|"
    r"CLAUDE\.md|AGENTS\.md)$"
)
SUBTRACTIVE_FEATURE_RE = re.compile(r"^(no[-_]|without[-_])")
BLOB_LINK_RE = re.compile(re.escape(URL) + r"/blob/main/([^)\s#]+)")


def run(*args: str) -> str:
    return subprocess.run(args, cwd=ROOT, check=True, capture_output=True, text=True).stdout


def valid_keyword(keyword: str) -> bool:
    return bool(KEYWORD_RE.match(keyword))


def feature_problems(name: str, features: dict[str, list[str]], deps: list[dict]) -> list[str]:
    """Problems with one package's feature table, per the additive model."""
    by_name = {dep.get("rename") or dep["name"]: dep for dep in deps if dep["kind"] is None}
    optional = {key for key, dep in by_name.items() if dep["optional"]}
    problems = []
    enabled_optional: set[str] = set()
    for feature, entries in features.items():
        if SUBTRACTIVE_FEATURE_RE.match(feature):
            problems.append(f"{name}: feature `{feature}` is named to subtract")
        for entry in entries:
            if entry.startswith("dep:"):
                dep = entry[4:]
                if dep not in optional:
                    problems.append(f"{name}: `{feature}` enables `{entry}`, not an optional dependency")
                enabled_optional.add(dep)
            elif "/" in entry:
                dep, _ = entry.split("/", 1)
                weak = dep.endswith("?")
                dep = dep.rstrip("?")
                if dep not in by_name:
                    problems.append(f"{name}: `{feature}` enables `{entry}`, not a dependency")
                elif weak and dep not in optional:
                    problems.append(f"{name}: `{feature}` enables `{entry}` weakly on a required dependency")
                elif not weak and dep in optional:
                    enabled_optional.add(dep)
            elif entry not in features and entry not in optional:
                problems.append(f"{name}: `{feature}` enables `{entry}`, neither a feature nor an optional dependency")
            elif entry in optional:
                enabled_optional.add(entry)
    for dep in sorted(optional - enabled_optional):
        problems.append(f"{name}: optional dependency `{dep}` is enabled by no feature")
    return problems


def metadata() -> dict[str, dict]:
    data = json.loads(run("cargo", "metadata", "--format-version", "1", "--no-deps"))
    packages = {package["name"]: package for package in data["packages"]}
    missing = [name for name in PACKAGES if name not in packages]
    if missing:
        raise SystemExit(f"not in the workspace: {', '.join(missing)}")
    return packages


def metadata_problems(package: dict) -> list[str]:
    name = package["name"]
    problems = []
    if not LINE_VERSION_RE.match(package.get("version") or ""):
        problems.append(f"{name}: version is {package.get('version')!r}, expected a {LINE}.x version")
    expected = {
        "edition": EDITION,
        "rust_version": RUST_VERSION,
        "license": LICENSE,
        "repository": URL,
        "homepage": URL,
    }
    for field, value in expected.items():
        if package.get(field) != value:
            problems.append(f"{name}: {field} is {package.get(field)!r}, expected {value!r}")
    if package.get("license_file"):
        problems.append(f"{name}: license_file is set alongside license")
    readme = package.get("readme")
    manifest_dir = Path(package["manifest_path"]).parent
    if readme is None or Path(readme).name != "README.md" or not (manifest_dir / "README.md").is_file():
        problems.append(f"{name}: readme is {readme!r}, expected README.md beside the manifest")
    if not (manifest_dir / "LICENSE").is_file():
        problems.append(f"{name}: no LICENSE beside the manifest")
    if package.get("publish") == []:
        problems.append(f"{name}: publish = false")
    description = package.get("description") or ""
    if "OptionStratLib" not in description and "optionstratlib" not in description:
        problems.append(f"{name}: the description does not name OptionStratLib")
    if name != FACADE and "optionstratlib facade" not in description:
        problems.append(f"{name}: the description does not place the crate in the facade")
    if name == FACADE and "component crates" not in description:
        problems.append(f"{name}: the description does not name the component crates")
    categories = package.get("categories") or []
    if not categories or len(categories) > MAX_CATEGORIES:
        problems.append(f"{name}: {len(categories)} categories (1 to {MAX_CATEGORIES} expected)")
    for category in categories:
        if category.split("::")[0] not in VALID_CATEGORIES:
            problems.append(f"{name}: `{category}` is not a crates.io category slug")
    keywords = package.get("keywords") or []
    if not keywords or len(keywords) > MAX_KEYWORDS:
        problems.append(f"{name}: {len(keywords)} keywords (1 to {MAX_KEYWORDS} expected)")
    for keyword in keywords:
        if not valid_keyword(keyword):
            problems.append(f"{name}: keyword `{keyword}` breaks the crates.io rules")
    for dep in package["dependencies"]:
        if dep.get("path") is not None and dep["req"] in ("*", ""):
            problems.append(f"{name}: path dependency `{dep['name']}` has no version requirement")
    problems += feature_problems(name, package["features"], package["dependencies"])
    return problems


def patch_of(version: str) -> int | None:
    """The patch number of a `0.22.N` version, or `None` off the line."""
    match = LINE_VERSION_RE.match(version)
    return int(match.group(1)) if match else None


def sibling_requirement_problems(name: str, deps: list[dict], versions: dict[str, str]) -> list[str]:
    """Sibling requirements under the versioning policy (#834).

    A sibling is required as `^0.22` or `^0.22.N` with `N` no newer than the
    sibling's own patch, so a crate can raise its requirement to a fixed
    patch published in the same release but never past what exists.
    """
    problems = []
    for dep in deps:
        sibling = dep["name"]
        if dep.get("path") is None or sibling not in versions:
            continue
        req = dep["req"]
        sibling_patch = patch_of(versions[sibling])
        if req == f"^{LINE}":
            continue
        required_patch = patch_of(req[1:]) if req.startswith("^") else None
        if required_patch is None:
            problems.append(f"{name}: `{sibling}` requires {req}, expected ^{LINE} or ^{LINE}.N")
        elif sibling_patch is None or required_patch > sibling_patch:
            problems.append(f"{name}: `{sibling}` requires {req}, newer than its {versions[sibling]}")
    return problems


def contents(name: str) -> list[str]:
    return run("cargo", "package", "--list", "--allow-dirty", "-p", name).split()


def contents_problems(name: str, files: list[str]) -> list[str]:
    problems = [f"{name}: the archive lacks {path}" for path in sorted(REQUIRED_FILES - set(files))]
    problems += [f"{name}: the archive carries {path}" for path in files if FORBIDDEN_RE.search(path)]
    return problems


def ownership_problems(packages: dict[str, dict]) -> list[str]:
    """Section 2 of the ownership map against the facade's features."""
    text = (ROOT / "docs" / "ownership.md").read_text()
    section = text.split("## 2.", 1)[1].split("\n## ", 1)[0]
    facade = packages[FACADE]["features"]
    problems = []
    rows = 0
    for line in section.splitlines():
        cells = [cell.strip() for cell in line.strip().strip("|").split("|")]
        if len(cells) != 5 or not cells[4].startswith("`optionstratlib-"):
            continue
        rows += 1
        feature, component_feature = cells[3], cells[4].strip("`")
        component, wanted = component_feature.split("/", 1)
        if feature not in facade:
            problems.append(f"ownership map: facade feature `{feature}` does not exist")
            continue
        if component_feature not in facade[feature] and f"{component}?/{wanted}" not in facade[feature]:
            problems.append(f"ownership map: facade `{feature}` does not enable `{component_feature}`")
        if wanted not in packages[component]["features"]:
            problems.append(f"ownership map: `{component}` has no feature `{wanted}`")
    if rows == 0:
        problems.append("ownership map: section 2 has no feature row")
    return problems


def testdata_problems(packages: dict[str, dict], listings: dict[str, list[str]]) -> list[str]:
    """Each package's `testdata/` files are archived and match `examples/Chains/`."""
    problems: list[str] = []
    for name in PACKAGES:
        testdata = Path(packages[name]["manifest_path"]).parent / "testdata"
        if not testdata.is_dir():
            continue
        for path in sorted(testdata.iterdir()):
            relative = f"testdata/{path.name}"
            if relative not in listings[name]:
                problems.append(f"{name}: {relative} is not in the archive")
            original = ROOT / "examples" / "Chains" / path.name
            if not original.is_file():
                problems.append(f"{name}: {relative} has no original under examples/Chains/")
            elif original.read_bytes() != path.read_bytes():
                problems.append(f"{name}: {relative} differs from examples/Chains/{path.name}")
    return problems


def link_problems(packages: dict[str, dict]) -> list[str]:
    tracked = set(run("git", "ls-files").split())
    problems = []
    for name in PACKAGES:
        readme = Path(packages[name]["manifest_path"]).parent / "README.md"
        for path in BLOB_LINK_RE.findall(readme.read_text()):
            if path not in tracked:
                problems.append(f"{name}: README links {path}, which is not a tracked file")
    return problems


def report(packages: dict[str, dict], listings: dict[str, list[str]], sizes: dict[str, str]) -> None:
    print("| Package | Files | Archive | Categories | Keywords |")
    print("| --- | ---: | --- | --- | --- |")
    for name in PACKAGES:
        package = packages[name]
        print(
            f"| `{name}` | {len(listings[name])} | {sizes.get(name, 'n/a')} | "
            f"{', '.join(package['categories'])} | {', '.join(package['keywords'])} |"
        )
    print()
    for name in PACKAGES:
        non_source = [path for path in listings[name] if not path.startswith("src/")]
        sources = len(listings[name]) - len(non_source)
        print(f"- `{name}`: {sources} files under `src/`, plus {', '.join(f'`{p}`' for p in non_source)}")


def sizes_from_dry_run() -> dict[str, str]:
    """Archive sizes from `cargo package` itself (its `Packaged ...` lines)."""
    out = subprocess.run(
        ["cargo", "package", "--allow-dirty", "--no-verify", *sum((["-p", n] for n in PACKAGES), [])],
        cwd=ROOT, check=True, capture_output=True, text=True,
    ).stderr
    sizes: dict[str, str] = {}
    current = None
    for line in out.splitlines():
        packaging = re.search(r"Packaging (\S+) v", line)
        if packaging:
            current = packaging.group(1)
        packaged = re.search(r"Packaged (\d+) files?, (.+)$", line)
        if packaged and current:
            sizes[current] = packaged.group(2)
    return sizes


def self_test() -> int:
    checks = [
        ("keyword ok", valid_keyword("option-chain")),
        ("keyword too long", not valid_keyword("a" * 21)),
        ("keyword digit first", not valid_keyword("3d")),
        ("keyword space", not valid_keyword("market data")),
        (
            "dep: on optional",
            feature_problems("p", {"io": ["dep:csv"]}, [{"name": "csv", "kind": None, "optional": True}]) == [],
        ),
        (
            "dep: on required",
            len(feature_problems("p", {"io": ["dep:csv"]}, [{"name": "csv", "kind": None, "optional": False}])) == 1,
        ),
        (
            "unused optional",
            len(feature_problems("p", {}, [{"name": "csv", "kind": None, "optional": True}])) == 1,
        ),
        (
            "weak on optional",
            feature_problems(
                "p",
                {"schema": ["sim?/schema"], "synthetic": ["dep:sim"]},
                [{"name": "sim", "kind": None, "optional": True}],
            ) == [],
        ),
        ("subtractive name", len(feature_problems("p", {"no-std": []}, [])) == 1),
        ("forbidden Draws", bool(FORBIDDEN_RE.search("Draws/chart.png"))),
        ("forbidden nested README", contents_problems("p", ["examples/direct/README.md", *REQUIRED_FILES]) != []),
        ("source allowed", not FORBIDDEN_RE.search("src/tests.rs")),
        ("test data allowed", not FORBIDDEN_RE.search("testdata/SP500-18-oct-2024-5781.88.json")),
        ("0.22.0 on the line", patch_of("0.22.0") == 0),
        ("0.22.3 on the line", patch_of("0.22.3") == 3),
        ("0.23.0 off the line", patch_of("0.23.0") is None),
        ("0.22.01 off the line", patch_of("0.22.01") is None),
        (
            "mixed patches pass",
            sibling_requirement_problems(
                "optionstratlib",
                [
                    {"name": "optionstratlib-pricing", "path": "p", "req": "^0.22.1"},
                    {"name": "optionstratlib-core", "path": "c", "req": "^0.22.0"},
                    {"name": "optionstratlib-math", "path": "m", "req": "^0.22"},
                ],
                {"optionstratlib-pricing": "0.22.1", "optionstratlib-core": "0.22.0", "optionstratlib-math": "0.22.0"},
            ) == [],
        ),
        (
            "a sibling on 0.23 fails",
            len(sibling_requirement_problems(
                "optionstratlib",
                [{"name": "optionstratlib-core", "path": "c", "req": "^0.22.0"}],
                {"optionstratlib-core": "0.23.0"},
            )) == 1,
        ),
        (
            "a requirement past the sibling fails",
            len(sibling_requirement_problems(
                "optionstratlib",
                [{"name": "optionstratlib-core", "path": "c", "req": "^0.22.2"}],
                {"optionstratlib-core": "0.22.1"},
            )) == 1,
        ),
        (
            "a requirement on another line fails",
            len(sibling_requirement_problems(
                "optionstratlib",
                [{"name": "optionstratlib-core", "path": "c", "req": "^0.21.3"}],
                {"optionstratlib-core": "0.22.0"},
            )) == 1,
        ),
    ]
    failures = 0
    for label, ok in checks:
        failures += not ok
        print(f"self-test {'ok' if ok else 'FAIL'}: {label}")
    return 1 if failures else 0


def main() -> int:
    if "--self-test" in sys.argv:
        return self_test()
    packages = metadata()
    listings = {name: contents(name) for name in PACKAGES}
    problems: list[str] = []
    for name in PACKAGES:
        problems += metadata_problems(packages[name])
        problems += contents_problems(name, listings[name])
    problems += ownership_problems(packages)
    problems += link_problems(packages)
    problems += testdata_problems(packages, listings)
    versions = {name: packages[name]["version"] for name in PACKAGES}
    for name in PACKAGES:
        problems += sibling_requirement_problems(name, packages[name]["dependencies"], versions)
    if "--report" in sys.argv:
        report(packages, listings, sizes_from_dry_run())
    if problems:
        print("\n".join(problems), file=sys.stderr)
        return 1
    print(f"OK: the {len(PACKAGES)} packages carry their sources, README and LICENSE, no local artifact, "
          f"{LINE}.x metadata with sibling requirements no newer than the siblings, valid categories and keywords, versioned path dependencies, "
          f"additive features matching the ownership map, tracked documentation links, and archived test data "
          f"matching examples/Chains")
    return 0


if __name__ == "__main__":
    sys.exit(main())
