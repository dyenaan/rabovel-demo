#!/usr/bin/env python3
"""Refresh the source inventory and baseline-to-current route ownership map.

Uses literal Axum registrations, not an HTTP probe. Keep the manual behavioral
traces in flow-map.md; this script accounts for source ownership and route moves.
"""

import json
import re
from datetime import date
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "docs/refactor"
BASELINE = json.loads((OUT / "flow-inventory.json").read_text())
SOURCES = sorted(p for top in ("apps", "crates") for p in (ROOT / top).rglob("*.rs"))


def flows(path):
    rel = str(path.relative_to(ROOT))
    parts = {
        "identity/accounts": ["F01", "F02", "F04"],
        "identity/sessions": ["F02"],
        "identity/access": ["F04"],
        "identity/wallets/linking": ["F06"],
        "identity/wallets/queries": ["F06"],
        "identity/wallets": ["F06", "F37"],
        "identity/security_state": ["F02", "F06", "F37"],
        "compliance/onboarding": ["F05"],
        "compliance/kyc": ["F07"],
        "issuer/organizations": ["F08"],
        "issuer/assets/drafts": ["F09"],
        "issuer/assets/media": ["F10"],
        "issuer/assets/review": ["F11"],
        "issuer/assets/backing": ["F11"],
        "issuer/assets/issuance/setup": ["F12"],
        "issuer/assets/issuance/inventory": ["F13"],
        "issuer/assets/issuance/providers": ["F12", "F13", "F27"],
        "issuer/assets/listings": ["F14"],
        "catalog/instruments": ["F17"],
        "market_data": ["F18"],
        "payments/cngn/broker_accounts": ["F15"],
        "payments/cngn/wallet_accounts": ["F16"],
        "payments": ["F15", "F16"],
        "settlement/direct_purchases/catalog": ["F19"],
        "settlement/direct_purchases/quotes": ["F20"],
        "settlement/direct_purchases/preparation": ["F21"],
        "settlement/direct_purchases/submission": ["F22"],
        "settlement": ["F19", "F20", "F21", "F22"],
        "trading/orders": ["F23", "F24", "F25"],
        "trading/execution": ["F23", "F24", "F25", "F26"],
        "trading/repricing": ["F26"],
        "trading/market_queries": ["F18", "F24"],
        "trading": ["F23", "F24", "F25", "F26"],
        "portfolio": ["F27"],
    }
    normalized = rel.replace("crates/identity/src/", "identity/")
    normalized = normalized.replace("crates/portfolio-trading/src/", "")
    for prefix, owners in parts.items():
        if normalized.startswith(prefix + "/") or f"/features/{prefix}/" in normalized:
            return owners
    if rel.startswith("crates/issuer-assets/"):
        for part, owner in (("organizations/", "F08"), ("drafts/", "F09"),
                            ("metadata/", "F10"), ("setup/", "F12"),
                            ("inventory/", "F13"), ("positions/", "F27")):
            if part in rel:
                return [owner]
        return ["F08", "F09", "F10", "F12", "F13", "F27"]
    if rel.startswith("crates/kyc/"):
        return ["F07"]
    if rel.startswith("crates/settlement-orchestrator/"):
        return ["F20", "F21", "F22"]
    if rel.startswith("crates/matching-engine/"):
        return ["F36"]
    if rel.startswith("crates/solana-adapter/"):
        return ["F34" if "/examples/" in rel else "F33"]
    if rel.startswith("crates/events/") or "/messaging/" in rel:
        return ["F32"]
    if rel.startswith("crates/infrastructure/"):
        return ["F35"]
    if rel.startswith("crates/identity/"):
        return ["F04", "F37"]
    if rel.startswith("crates/portfolio-trading/"):
        return ["F23", "F24", "F25", "F26", "F27"]
    if "-worker/" in rel:
        return ["F31"]
    if rel.startswith("apps/rabovel-api/"):
        if any(p in rel for p in ("health.rs", "documentation.rs", "/documentation/")):
            return ["F29"]
        return ["F30"]  # Module composition, ingress, errors and shared fixtures.
    raise ValueError(f"Unowned source: {rel}")


def route_expression(source, start):
    """Read until .route's closing parenthesis, respecting strings/nesting."""
    depth, quoted, escaped = 1, False, False
    for i in range(start, len(source)):
        ch = source[i]
        if quoted:
            if escaped:
                escaped = False
            elif ch == "\\":
                escaped = True
            elif ch == '"':
                quoted = False
        elif ch == '"':
            quoted = True
        elif ch == "(":
            depth += 1
        elif ch == ")":
            depth -= 1
            if depth == 0:
                return source[start:i]
    raise ValueError("Unbalanced route expression")


def inspect():
    definitions, sources, registrations = {}, [], {}
    for path in SOURCES:
        source = path.read_text(encoding="utf-8-sig")
        rel = str(path.relative_to(ROOT))
        for fn in re.finditer(r"\b(?:async\s+)?fn\s+(\w+)\s*\(", source):
            definitions.setdefault(fn[1], []).append(
                {"source": rel, "line": source.count("\n", 0, fn.start()) + 1}
            )
        tests = []
        for test in re.finditer(
            r"#\[(?:tokio::|actix_rt::)?test(?:\([^\]]*\))?\]"
            r"(?P<attrs>(?:\s*#\[[^\]]*\])*)\s*(?:async\s+)?fn\s+(?P<name>\w+)",
            source,
        ):
            tests.append({"name": test["name"],
                          "line": source.count("\n", 0, test.start()) + 1,
                          "ignored": "ignore" in test["attrs"]})
        sources.append({"source": rel, "lines": len(source.splitlines()),
                        "flows": flows(path), "tests": tests})
        if not rel.startswith("apps/rabovel-api/") or "/tests" in rel or "/test_support/" in rel:
            continue
        for route in re.finditer(r'\.route\(\s*"([^"\n]+)"\s*,', source):
            expression = route_expression(source, route.end())
            for method in re.finditer(r"\b(get|post|patch|delete|put)\(\s*([\w:]+)\s*\)", expression):
                key = (method[1].upper(), route[1])
                if key in registrations:
                    raise ValueError(f"Duplicate route: {key}")
                registrations[key] = {"method": key[0], "path": key[1],
                                      "handler": method[2].split("::")[-1],
                                      "source": rel,
                                      "line": source.count("\n", 0, route.start()) + 1}
    routes = []
    for old in BASELINE["routes"]:
        current = registrations.pop((old["method"], old["path"]), None)
        record = {k: old[k] for k in ("id", "method", "path", "flow", "availability")}
        record["status"] = "active" if current else "retired"
        if current:
            record.update(current)
            candidates = definitions.get(current["handler"], [])
            handler = next((d for d in candidates if d["source"].endswith("handlers.rs")), None)
            if handler is None:
                handler = next((d for d in candidates if d["source"] == current["source"]), None)
            if handler is None and len(candidates) == 1:
                handler = candidates[0]
            if not handler:
                raise ValueError(f"Unresolved handler: {current}")
            record["handler_definition"] = handler
        routes.append(record)
    if registrations:
        raise ValueError(f"New routes need manual flow IDs: {registrations}")
    retired = {r["id"] for r in routes if r["status"] == "retired"}
    if retired != {"H02", "H03", "H48", "H49"}:
        raise ValueError(f"Unexpected retirements: {retired}")
    active = [r for r in routes if r["status"] == "active"]
    return {
        "captured": str(date.today()),
        "scope": "Current apps/crates Rust source and literal API registrations; baseline IDs retained.",
        "method": "Static source scan; not runtime probes or proof of behavioral coverage. Implicit HEAD/OPTIONS excluded. Behavioral traces remain in baseline flow-map.md with changes in retirements.md.",
        "summary": {"explicit_route_methods": len(active),
                    "distinct_paths": len({r["path"] for r in active}),
                    "retired_route_methods": len(retired),
                    "rust_source_files": len(sources),
                    "test_declarations": sum(len(s["tests"]) for s in sources),
                    "ignored_test_declarations": sum(t["ignored"] for s in sources for t in s["tests"]),
                    "executable_applications": len(list((ROOT / "apps").glob("*/Cargo.toml"))),
                    "library_crates": len(list((ROOT / "crates").glob("*/Cargo.toml")))},
        "flow_owners": BASELINE["flow_owners"], "routes": routes, "rust_sources": sources,
    }


def main():
    inventory = inspect()
    (OUT / "current-inventory.json").write_text(json.dumps(inventory, indent=2) + "\n")
    rows = ["| ID / flow | Method / path | Current handler or retirement |",
            "| --- | --- | --- |"]
    for route in inventory["routes"]:
        if route["status"] == "retired":
            owner = "[Retired](retirements.md)"
        else:
            handler = route["handler_definition"]
            owner = f"[{route['handler']}](../../{handler['source']}#L{handler['line']})"
        rows.append(f"| {route['id']} / {route['flow']} | `{route['method']} {route['path']}` | {owner} |")
    (OUT / "current-routes.md").write_text(
        "# Current HTTP register\n\nGenerated by `python3 scripts/inspect_flows.py`. "
        "Baseline H IDs retained; availability conditions come from the baseline and remain unchanged for active routes. "
        "See [ownership and limits](current-map.md) and [machine-readable sources/availability](current-inventory.json).\n\n"
        + "\n".join(rows) + "\n"
    )
    print(json.dumps(inventory["summary"], indent=2))


if __name__ == "__main__":
    main()
