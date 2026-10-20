#!/usr/bin/env python3
"""Small dependency guards for the feature architecture; no extra packages needed."""

from pathlib import Path
import re
import sys
import tomllib

ROOT = Path(__file__).resolve().parents[1]
API = ROOT / "apps/rabovel-api/src"
errors = []


def reject(condition, message):
    if condition:
        errors.append(message)


def code(path):
    text = path.read_text(encoding="utf-8-sig")
    return re.sub(r"//[^\n]*|/\*[\s\S]*?\*/", "", text)


reject(
    {p.name for p in API.glob("*.rs")} - {"lib.rs", "main.rs"},
    "API root is for the facade and entry point; put workflows in their feature.",
)
reject(
    re.search(r"\b(?:async\s+)?fn\s+", code(API / "lib.rs")),
    "API lib.rs must remain a facade, without workflow implementations.",
)
for name in ("repositories", "providers", "adapters", "usecases"):
    reject((API / name).exists(), f"Global business bucket is forbidden: src/{name}")

for path in sorted((API / "features").rglob("usecases.rs")):
    source = code(path)
    rel = path.relative_to(ROOT)
    for forbidden in (
        r"\baxum::",
        r"\bGatewayState\b",
        r"\b(?:std::)?env::(?:var|vars)\b",
        r"\binfrastructure::",
        r"::repositories::",
        r"\b(?:sqlx|redis|solana_client)::",
    ):
        reject(re.search(forbidden, source), f"{rel}: effect/transport dependency {forbidden}")
    if any(part in path.parts for part in ("catalog", "settlement", "portfolio")):
        reject(
            re.search(r"\bstate\s*\.\s*(?:assets|setups)\(\)", source),
            f"{rel}: cross-feature reads must use asset/setup query facades.",
        )
    if "compliance" in path.parts:
        reject(".save_user(" in source, f"{rel}: compliance may update profiles, never principals.")

for manifest in sorted((ROOT / "crates").glob("*/Cargo.toml")):
    data = tomllib.loads(manifest.read_text(encoding="utf-8-sig"))
    dependencies = data.get("dependencies", {})
    reject("rabovel-api" in dependencies, f"{manifest}: libraries must not depend on an app.")
    for name, spec in dependencies.items():
        if isinstance(spec, dict):
            reject("apps/" in spec.get("path", ""), f"{manifest}: {name} imports an app.")
    if data["package"]["name"] == "infrastructure":
        reject(
            set(dependencies) & {"identity", "issuer-assets", "kyc", "domain"},
            "Infrastructure must contain technical transport/lifecycle, not business dependencies.",
        )

for path in (ROOT / "crates/matching-engine/src").rglob("*.rs"):
    reject(
        re.search(r"\b(?:axum|sqlx|redis|tokio|solana_client)::|std::(?:fs|net|env|time)::", code(path)),
        f"{path.relative_to(ROOT)}: matching rules must remain deterministic and I/O-free.",
    )

for path in (ROOT / "crates").rglob("*.rs"):
    reject(re.search(r"\btrait\s+Repository\b", code(path)), f"{path}: name persistence by its owner.")

for retired in ("domain", "tokenization-orchestrator", "reconciliation"):
    reject((ROOT / "crates" / retired).exists(), f"Retired package remains: {retired}")
reject((ROOT / "apps/tokenization-worker").exists(), "Retired tokenization worker remains.")
reject(not (ROOT.parent / ".github/workflows/backend.yml").is_file(), "Backend CI must live at the actual Git root.")

if errors:
    print("Architecture check failed:", file=sys.stderr)
    for error in errors:
        print(f"- {error}", file=sys.stderr)
    sys.exit(1)
print("Architecture boundaries passed.")
