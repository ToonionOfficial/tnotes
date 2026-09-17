#!/usr/bin/env python3
"""
Verifies that all application versions in the repository are in sync.
Exits 0 if all match, or 1 if any version drifts.

Usage:
  python3 scripts/check-version-sync.py
"""

import json
import re
import sys
from pathlib import Path


def get_repo_root() -> Path:
    return Path(__file__).resolve().parent.parent


def get_cargo_version(repo_root: Path) -> str:
    content = (repo_root / "Cargo.toml").read_text(encoding="utf-8")
    in_workspace_pkg = False
    for line in content.splitlines():
        trimmed = line.strip()
        if trimmed == "[workspace.package]":
            in_workspace_pkg = True
            continue
        if in_workspace_pkg and trimmed.startswith("["):
            break
        if in_workspace_pkg and trimmed.startswith("version"):
            match = re.search(r'version\s*=\s*"([^"]+)"', trimmed)
            if match:
                return match.group(1)
    raise ValueError("version not found under [workspace.package] in Cargo.toml")


def get_package_json_version(filepath: Path) -> str:
    data = json.loads(filepath.read_text(encoding="utf-8"))
    return data.get("version", "")


def main():
    repo_root = get_repo_root()

    versions = {}
    try:
        versions["Cargo Workspace"] = get_cargo_version(repo_root)
    except Exception as e:
        versions["Cargo Workspace"] = f"ERROR ({e})"

    web_pkg = repo_root / "apps" / "web" / "package.json"
    if web_pkg.exists():
        versions["Web Frontend"] = get_package_json_version(web_pkg)

    mobile_pkg = repo_root / "apps" / "mobile" / "package.json"
    if mobile_pkg.exists():
        versions["Mobile App"] = get_package_json_version(mobile_pkg)

    print("==> Checking version synchronization:")
    for name, ver in versions.items():
        print(f"  - {name:<18}: {ver}")

    unique_versions = set(versions.values())
    if len(unique_versions) == 1 and not any(v.startswith("ERROR") for v in unique_versions):
        synced_version = next(iter(unique_versions))
        print(f"\n[OK] All application versions are synchronized: v{synced_version}")
        sys.exit(0)
    else:
        print("\n[FAIL] Version drift detected across projects!", file=sys.stderr)
        print("To fix this, run: ./scripts/bump-version.sh <version>", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
