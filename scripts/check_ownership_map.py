#!/usr/bin/env python3
"""Check that `docs/ownership.md` matches the code (#556).

The ownership map is a published contract: one defining owner per concept,
its direct import, its facade path and the feature it needs. This check fails
when the map and the workspace disagree:

1. Every path a table names exists in the public API: a defining path in its
   component's snapshot under `public-api/`, a facade path in
   `public-api/optionstratlib.txt` (snapshots are kept current by
   `make public-api-check`).
2. Every facade path is proved to be its defining item by a compiled test:
   the pair appears in `tests/unit/canonical_paths_test.rs` (`same!` rows
   compare `TypeId`s) or, for macros and functions, as a facade re-export of
   the defining item.
3. Every feature is a feature of the facade manifest, gates the facade path
   in `src/lib.rs` / `src/error/mod.rs` exactly as the map says, and forwards
   to the component feature the map names, which the component declares.
4. Coverage: every facade module, every root item and every concrete error
   (`pub enum <crate>::error::*Error`) in the snapshots has exactly one row,
   and every foundational crate is a dependency of `optionstratlib-core`.

Usage: scripts/check_ownership_map.py [--self-test]
"""

from __future__ import annotations

import re
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
MAP = Path("docs/ownership.md")
FACADE_SNAPSHOT = Path("public-api/optionstratlib.txt")
CANONICAL_TEST = Path("tests/unit/canonical_paths_test.rs")
PATH_TOKEN = re.compile(r"[A-Za-z_][A-Za-z0-9_]*(?:::[A-Za-z_][A-Za-z0-9_]*)+")


def strip_ticks(cell: str) -> str:
    """The first backticked token of a cell, or the bare cell."""
    ticked = re.search(r"`([^`]+)`", cell)
    return (ticked.group(1) if ticked else cell).strip()


def tables(text: str) -> dict[str, list[dict[str, str]]]:
    """Every markdown table under a `## <n>.` heading, keyed by `<n>`."""
    result: dict[str, list[dict[str, str]]] = {}
    section = None
    header: list[str] | None = None
    for line in text.splitlines():
        heading = re.match(r"^##\s+(\d+)\.", line)
        if heading:
            section, header = heading.group(1), None
            continue
        if section is None or not line.startswith("|"):
            header = None if not line.startswith("|") else header
            continue
        cells = [c.strip() for c in line.strip().strip("|").split("|")]
        if header is None:
            header = cells
            continue
        if set("".join(cells)) <= set("-: "):
            continue
        result.setdefault(section, []).append(dict(zip(header, cells)))
    return result


def manifest_features(path: Path) -> dict[str, list[str]]:
    """The `[features]` table of a Cargo manifest, with no TOML dependency."""
    text = path.read_text()
    match = re.search(r"^\[features\]\s*$(.*?)(?=^\[)", text, re.M | re.S)
    if not match:
        return {}
    body = "\n".join(line.split("#", 1)[0] for line in match.group(1).splitlines())
    features: dict[str, list[str]] = {}
    for name, value in re.findall(r"^\s*([A-Za-z0-9_-]+)\s*=\s*\[(.*?)\]", body, re.M | re.S):
        features[name] = re.findall(r'"([^"]+)"', value)
    return features


def manifest_dependencies(path: Path) -> set[str]:
    text = path.read_text()
    match = re.search(r"^\[dependencies\]\s*$(.*?)(?=^\[|\Z)", text, re.M | re.S)
    if not match:
        return set()
    return set(re.findall(r"^\s*([A-Za-z0-9_-]+)\s*=", match.group(1), re.M))


def snapshot_paths(text: str) -> set[str]:
    """Every path token of a public-API snapshot, plus each one with a single
    defining submodule removed, so `a::b::c::Item` also answers `a::b::Item`
    (the flat re-export of an item defined in a submodule)."""
    tokens = set(PATH_TOKEN.findall(text))
    widened = set(tokens)
    for token in tokens:
        parts = token.split("::")
        for i in range(1, len(parts) - 1):
            widened.add("::".join(parts[:i] + parts[i + 1:]))
    return widened


def normalize(path: str) -> str:
    """A type as written in a test: no `dyn`, generics or whitespace."""
    path = re.sub(r"\bdyn\b", "", path)
    path = re.sub(r"<.*>", "", path)
    return re.sub(r"\s+", "", path)


def proved_pairs(test_text: str) -> set[tuple[str, str]]:
    """`(facade, defining)` pairs a compiled test compares."""
    flat = re.sub(r"\s+", " ", test_text)
    pairs = set()
    for facade, defining in re.findall(r"same!\(\s*([^,]+?)\s*,\s*([^;]+?)\s*,?\s*\);", flat):
        facade = normalize(facade)
        if facade.startswith("facade::"):
            facade = "optionstratlib::error::" + facade[len("facade::"):]
        pairs.add((facade, normalize(defining)))
    for facade, defining in re.findall(
        r"assert_eq!\(\s*(optionstratlib::[\w:]+)\s*,\s*(optionstratlib_\w+::[\w:]+)\s*\)", flat
    ):
        pairs.add((facade, defining))
    return pairs


def facade_gates(source: str) -> dict[str, str]:
    """Facade item -> the feature gating its `pub use` (`always` if none)."""
    gates: dict[str, str] = {}
    for statement in re.split(r";", re.sub(r"//[^\n]*", "", source)):
        cfg = re.findall(r'#\[cfg\(feature\s*=\s*"([^"]+)"\)\]', statement)
        use = re.search(r"pub\s+(?:use|mod)\s+([^;]+)$", statement.strip(), re.S)
        if not use:
            continue
        feature = cfg[-1] if cfg else "always"
        body = re.sub(r"\s+", "", use.group(1))
        prefix, _, group = body.partition("{")
        names = group.rstrip("}").split(",") if group else [body]
        for name in filter(None, names):
            leaf = name.split("::")[-1]
            gates[leaf] = feature
    return gates


DECLARATION = r"\b(?:fn|mod|struct|enum|trait|type|const|static)\s+{leaf}\b|^\s*pub\s+use\s+[^;]*\b{leaf}\b"


def cfg_gated(sources: dict[str, str], leaf: str, feature: str) -> bool:
    """Whether some declaration of `leaf` (the item itself, its `pub use` or
    the `pub mod` naming it) carries `feature = "<feature>"` in a `cfg` or
    `cfg_attr` attribute directly above it."""
    pattern = re.compile(DECLARATION.format(leaf=re.escape(leaf)), re.M)
    wanted = f'feature = "{feature}"'
    for text in sources.values():
        lines = text.splitlines()
        for number, line in enumerate(lines):
            if not pattern.search(line):
                continue
            above = number - 1
            while above >= 0 and lines[above].strip().startswith(("#[", "//")):
                if lines[above].strip().startswith("#[") and wanted in lines[above]:
                    return True
                above -= 1
    return False


def reexported_macros(lib_source: str) -> dict[str, str]:
    """Macro (or other root item) name -> the crate `src/lib.rs` re-exports it
    from, for `pub use <crate>::{..}` / `pub use <crate>::<name>` at the root."""
    found: dict[str, str] = {}
    for crate, body in re.findall(r"^pub use (optionstratlib_\w+)::(\{[^}]*\}|\w+);", lib_source, re.M):
        for name in re.findall(r"\w+", body):
            found[name] = crate
    return found


def check(root: Path) -> list[str]:
    errors: list[str] = []
    text = (root / MAP).read_text()
    rows = tables(text)
    facade_features = manifest_features(root / "Cargo.toml")
    facade_snapshot = (root / FACADE_SNAPSHOT).read_text()
    snapshots = {
        path.stem: path.read_text()
        for path in (root / "public-api").glob("optionstratlib-*.txt")
    }
    all_paths = set()
    for body in snapshots.values():
        all_paths |= snapshot_paths(body)
    proved = proved_pairs((root / CANONICAL_TEST).read_text())
    required = {
        "1": {"Defining module", "Facade path", "Example item", "Facade feature", "0.22 status"},
        "2": {"Capability", "Defining item", "Facade path", "Facade feature", "Component feature"},
        "3": {"Owner package", "Direct import", "Facade path", "Facade feature", "Kind module"},
        "4": {"Item", "Defining path", "Facade path", "Facade feature"},
        "5": {"Owning crate", "Core re-export", "Facade path"},
    }
    for section, columns in required.items():
        if not rows.get(section):
            errors.append(f"section {section}: no table")
        malformed = [row for row in rows.get(section, []) if not columns <= set(row)]
        if malformed:
            errors.append(
                f"section {section}: {len(malformed)} row(s) outside the table's header "
                f"(a blank line splits the table, or a header changed)"
            )
        rows[section] = [row for row in rows.get(section, []) if columns <= set(row)]
    lib_gates = facade_gates((root / "src" / "lib.rs").read_text())
    lib_reexports = reexported_macros((root / "src" / "lib.rs").read_text())
    error_gates = facade_gates((root / "src" / "error" / "mod.rs").read_text())

    def feature_ok(where: str, feature: str) -> None:
        if feature != "always" and feature not in facade_features:
            errors.append(f"{where}: feature `{feature}` is not a feature of Cargo.toml")

    def exists(where: str, path: str) -> None:
        if path not in all_paths:
            errors.append(f"{where}: `{path}` is not in any public-API snapshot")

    # 1. Capability modules.
    modules = {}
    for row in rows.get("1", []):
        defining = strip_ticks(row["Defining module"])
        facade = strip_ticks(row["Facade path"])
        example = strip_ticks(row["Example item"])
        feature = row["Facade feature"].strip()
        where = f"section 1, {facade}"
        modules[facade] = defining
        feature_ok(where, feature)
        exists(where, f"{defining}::{example}")
        if (f"{facade}::{example}", f"{defining}::{example}") not in proved:
            errors.append(f"{where}: no compiled test compares `{facade}::{example}` with `{defining}::{example}`")
        leaf = facade.split("::")[-1]
        if lib_gates.get(leaf) != feature:
            errors.append(f"{where}: src/lib.rs gates it by `{lib_gates.get(leaf)}`, the map says `{feature}`")
        if row["0.22 status"].strip() != "public":
            errors.append(f"{where}: a listed module must be public")
    component_modules = {
        path.split("::")[-1]
        for path in re.findall(r"^pub mod (optionstratlib_\w+::\w+)$", "\n".join(snapshots.values()), re.M)
    }
    facade_modules = {
        path
        for path in re.findall(r"^pub use (optionstratlib::[a-z_]+)$", facade_snapshot, re.M)
        if path.split("::")[-1] in component_modules
    }
    for missing in sorted(facade_modules - set(modules)):
        errors.append(f"section 1: facade module `{missing}` has no row")
    for stale in sorted(set(modules) - facade_modules):
        errors.append(f"section 1: `{stale}` is not a facade module")

    # 2. Feature-gated capabilities.
    for row in rows.get("2", []):
        defining = strip_ticks(row["Defining item"])
        facade = strip_ticks(row["Facade path"])
        feature = row["Facade feature"].strip()
        forward = strip_ticks(row["Component feature"])
        where = f"section 2, {row['Capability']}"
        feature_ok(where, feature)
        exists(where, defining)
        if feature == "schema":
            if not re.search(rf"^impl utoipa::ToSchema for {re.escape(defining.rsplit('::', 1)[0])}::(\w+::)?{re.escape(defining.rsplit('::', 1)[1])}$", "\n".join(snapshots.values()), re.M):
                errors.append(f"{where}: `{defining}` derives no `ToSchema` in the snapshots")
        module = next((m for m in modules if facade.startswith(m + "::")), None)
        if module is None or not defining.startswith(modules[module] + "::"):
            errors.append(f"{where}: `{facade}` is not under the facade module of `{defining}`")
        elif facade.split("::")[-1] != defining.split("::")[-1]:
            errors.append(f"{where}: `{facade}` and `{defining}` name different items")
        if forward not in facade_features.get(feature, []):
            errors.append(f"{where}: facade feature `{feature}` does not forward to `{forward}`")
        package, _, component_feature = forward.partition("/")
        component_manifest = root / "crates" / package / "Cargo.toml"
        if not component_manifest.is_file() or component_feature not in manifest_features(component_manifest):
            errors.append(f"{where}: `{package}` declares no feature `{component_feature}`")
        else:
            sources = {
                str(path): path.read_text()
                for path in (root / "crates" / package / "src").rglob("*.rs")
            }
            if not cfg_gated(sources, defining.split("::")[-1], component_feature):
                errors.append(
                    f"{where}: no declaration of `{defining}` in `{package}` is gated by "
                    f'`#[cfg(feature = "{component_feature}")]`'
                )

    # 3. Errors.
    concrete = set(re.findall(r"^pub enum (optionstratlib_\w+::[\w:]*Error)$", "\n".join(snapshots.values()), re.M))
    listed: dict[str, str] = {}
    kind_modules_listed: set[str] = set()
    for row in rows.get("3", []):
        direct = strip_ticks(row["Direct import"])
        facade = strip_ticks(row["Facade path"])
        feature = row["Facade feature"].strip()
        owner = strip_ticks(row["Owner package"])
        name = facade.split("::")[-1]
        where = f"section 3, {name}"
        feature_ok(where, feature)
        if name in listed:
            errors.append(f"{where}: listed twice")
        listed[name] = owner
        if owner != "optionstratlib":
            crate = owner.replace("-", "_")
            if not direct.startswith(crate + "::error::"):
                errors.append(f"{where}: `{direct}` is not in the error module of `{owner}`")
            exists(where, direct)
            if (facade, direct) not in proved:
                errors.append(f"{where}: no compiled test compares `{facade}` with `{direct}`")
        elif ("optionstratlib::error::Error", "optionstratlib::error::Error") not in proved:
            errors.append(f"{where}: no compiled test pins the facade aggregate")
        if error_gates.get(name) != feature:
            errors.append(f"{where}: src/error/mod.rs gates it by `{error_gates.get(name)}`, the map says `{feature}`")
        kind = strip_ticks(row["Kind module"])
        if kind not in ("—", "-", ""):
            module = f"{direct.rsplit('::', 1)[0]}::{kind}"
            kind_modules_listed.add(module)
            if not re.search(rf"^pub mod {re.escape(module)}$", "\n".join(snapshots.values()), re.M):
                errors.append(f"{where}: kind module `{module}` is not public")
    public_kind_modules = set(
        re.findall(r"^pub mod (optionstratlib_\w+::error::\w+)$", "\n".join(snapshots.values()), re.M)
    )
    for module in sorted(public_kind_modules - kind_modules_listed):
        errors.append(f"section 3: public error module `{module}` is not the kind module of any row")
    directs = {strip_ticks(row["Direct import"]) for row in rows.get("3", [])}
    for path in sorted(concrete):
        name = path.split("::")[-1]
        owner = path.split("::")[0].replace("_", "-")
        parts = path.split("::")
        # An error defined in a public kind module (`error::chains::ChainError`)
        # is also listed at its defining path; its row names the flat one.
        flat = "::".join(parts[:-2] + parts[-1:]) if len(parts) == 4 and parts[1] == "error" else path
        if not ({path, flat} & directs) or listed.get(name) != owner:
            errors.append(
                f"section 3: concrete error `{path}` has no row with that direct import owned by "
                f"`{owner}` (an error belongs in its crate's `error` module)"
            )

    # 4. Root items.
    root_items = set(re.findall(r"^pub use optionstratlib::([A-Za-z_0-9]+)$", facade_snapshot, re.M))
    root_items -= {m.split("::")[-1] for m in facade_modules}
    covered = set()
    for row in rows.get("4", []):
        facade = strip_ticks(row["Facade path"])
        defining = strip_ticks(row["Defining path"])
        feature = row["Facade feature"].strip()
        name = facade.split("::")[-1]
        where = f"section 4, {name}"
        feature_ok(where, feature)
        covered.add(name)
        if name == "VERSION":
            covered.add("version")
            for needed in ("pub const optionstratlib::VERSION", "pub fn optionstratlib::version"):
                if needed not in facade_snapshot:
                    errors.append(f"{where}: `{needed}` is not in the facade snapshot")
            continue
        if lib_gates.get(name) != feature:
            errors.append(f"{where}: src/lib.rs gates it by `{lib_gates.get(name)}`, the map says `{feature}`")
        if strip_ticks(row["Item"]).endswith("!"):
            if f"pub macro {defining}!" not in "\n".join(snapshots.values()):
                errors.append(f"{where}: `{defining}!` is not a macro in the snapshots")
            if defining.split("::")[-1] != name or strip_ticks(row["Item"]).rstrip("!") != name:
                errors.append(f"{where}: the item, `{defining}` and `{facade}` name different macros")
            if lib_reexports.get(name) != defining.split("::")[0]:
                errors.append(f"{where}: src/lib.rs does not re-export `{name}` from `{defining.split('::')[0]}`")
        elif (facade, defining) not in proved:
            errors.append(f"{where}: no compiled test compares `{facade}` with `{defining}`")
    for missing in sorted(root_items - covered):
        errors.append(f"section 4: facade root item `{missing}` has no row")

    # 5. Foundational crates.
    core_deps = manifest_dependencies(root / "crates" / "optionstratlib-core" / "Cargo.toml")
    core_snapshot = snapshots.get("optionstratlib-core", "")
    reexported = set()
    for source in (root / "crates" / "optionstratlib-core" / "src").rglob("*.rs"):
        reexported |= set(re.findall(r"^\s*pub use ([a-z_]+)::", source.read_text(), re.M))
    foundational = {crate for crate in reexported if crate.replace("_", "-") in core_deps or crate in core_deps}
    listed_crates = {strip_ticks(row["Owning crate"]) for row in rows.get("5", [])}
    for missing in sorted(foundational - listed_crates):
        errors.append(f"section 5: core re-exports the external crate `{missing}`, which has no row")
    for row in rows.get("5", []):
        crate = strip_ticks(row["Owning crate"])
        reexport = strip_ticks(row["Core re-export"])
        facade = strip_ticks(row["Facade path"])
        where = f"section 5, {crate}"
        if crate not in core_deps:
            errors.append(f"{where}: `{crate}` is not a dependency of optionstratlib-core")
        if f"pub use {reexport}\n" not in core_snapshot + "\n":
            errors.append(f"{where}: core does not re-export `{reexport}`")
        if facade != "optionstratlib::" + reexport.split("::", 1)[1]:
            errors.append(f"{where}: `{facade}` is not the facade path of `{reexport}`")
        for manifest in [root / "Cargo.toml", *(root / "crates").glob("*/Cargo.toml")]:
            if manifest.parent.name != "optionstratlib-core" and crate in manifest_dependencies(manifest):
                owner = "optionstratlib" if manifest.parent == root else manifest.parent.name
                errors.append(f"{where}: `{owner}` depends on `{crate}` directly (ADR-0001 D8)")
    return errors


def self_test() -> int:
    failures = 0
    sample = "## 1. A\n\n| X | Y |\n| --- | --- |\n| `a` | b |\n\n## 2. B\n| P |\n| - |\n| q |\n"
    ok = tables(sample) == {"1": [{"X": "`a`", "Y": "b"}], "2": [{"P": "q"}]}
    failures += not ok
    print(f"self-test {'ok' if ok else 'FAIL'}: table parsing")
    with tempfile.TemporaryDirectory() as tmp:
        manifest = Path(tmp) / "Cargo.toml"
        manifest.write_text('[features]\na = ["x", "y"] # c\nb = [\n  "z",\n]\n[dependencies]\np = "1"\nq = { workspace = true }\n')
        ok = manifest_features(manifest) == {"a": ["x", "y"], "b": ["z"]} and manifest_dependencies(manifest) == {"p", "q"}
    failures += not ok
    print(f"self-test {'ok' if ok else 'FAIL'}: manifest parsing")
    ok = "a::b::Item" in snapshot_paths("pub struct a::b::c::Item")
    failures += not ok
    print(f"self-test {'ok' if ok else 'FAIL'}: flat re-export of a submodule item")
    pairs = proved_pairs("same!(\n facade::E,\n x::error::E\n);\nsame!(dyn a::T, dyn b::T);\nsame!(a::G<'static>, b::G<'static>);")
    ok = pairs == {("optionstratlib::error::E", "x::error::E"), ("a::T", "b::T"), ("a::G", "b::G")}
    failures += not ok
    print(f"self-test {'ok' if ok else 'FAIL'}: identity pairs")
    sources = {
        "mod.rs": '#[cfg(feature = "synthetic")]\nmod generators;\n#[cfg(feature = "synthetic")]\npub use generators::generator_optionchain;\n',
        "generators.rs": "/// Builds a chain.\npub fn generator_optionchain() {}\n",
        "chain.rs": '    /// Loads.\n    #[cfg(feature = "async")]\n    pub async fn load_from_csv_async() {}\n',
    }
    ok = (
        cfg_gated(sources, "generator_optionchain", "synthetic")
        and not cfg_gated(sources, "generator_optionchain", "io")
        and cfg_gated(sources, "load_from_csv_async", "async")
        and not cfg_gated(sources, "load_from_csv_async", "io")
    )
    failures += not ok
    print(f"self-test {'ok' if ok else 'FAIL'}: component cfg gates (synthetic->io and async->io are caught)")
    macros = reexported_macros("pub use optionstratlib_core::{assert_decimal_eq, d2f, nz};\npub use optionstratlib_visualization::impl_graph_for_payoff_strategy;\n")
    ok = macros.get("nz") == "optionstratlib_core" and macros.get("impl_graph_for_payoff_strategy") == "optionstratlib_visualization"
    failures += not ok
    print(f"self-test {'ok' if ok else 'FAIL'}: root macro re-exports")
    gates = facade_gates('pub use c::model;\n#[cfg(feature = "math")]\npub use m::{A, b};\n// pub use x::Y;\n')
    ok = gates == {"model": "always", "A": "math", "b": "math"}
    failures += not ok
    print(f"self-test {'ok' if ok else 'FAIL'}: feature gates")
    return failures


def main() -> int:
    if "--self-test" in sys.argv[1:]:
        return 1 if self_test() else 0
    errors = check(ROOT)
    if errors:
        print("docs/ownership.md is out of date with the code (#556):")
        for error in errors:
            print(f"  {error}")
        return 1
    print("OK: docs/ownership.md matches the snapshots, the identity tests and the manifests")
    return 0


if __name__ == "__main__":
    sys.exit(main())
