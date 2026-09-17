#!/usr/bin/env python3
"""
Bumps the version synchronously across all projects in the TNotes repository:
  - Root Cargo.toml ([workspace.package] version)
  - Cargo.lock (via cargo check)
  - apps/web/package.json (version)
  - apps/mobile/package.json (version)
  - scripts/desktop/installer.iss (#define AppVersion)

Usage:
  python3 scripts/bump-version.py patch
  python3 scripts/bump-version.py minor
  python3 scripts/bump-version.py major
  python3 scripts/bump-version.py beta
  python3 scripts/bump-version.py 0.2.0
  python3 scripts/bump-version.py 0.2.0 --dry-run
  python3 scripts/bump-version.py 0.2.0 --tag
"""

import argparse
import json
import re
import subprocess
import sys
from pathlib import Path

SEMVER_REGEX = re.compile(
    r"^v?(?P<major>0|[1-9]\d*)\.(?P<minor>0|[1-9]\d*)\.(?P<patch>0|[1-9]\d*)"
    r"(?:-(?P<prerelease>[0-9A-Za-z.-]+))?"
    r"(?:\+(?P<build>[0-9A-Za-z.-]+))?$"
)


def get_repo_root() -> Path:
    return Path(__file__).resolve().parent.parent


def get_current_version(repo_root: Path) -> str:
    cargo_toml = repo_root / "Cargo.toml"
    content = cargo_toml.read_text(encoding="utf-8")
    
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

    # Fallback to desktop Cargo.toml if not in root
    desktop_cargo = repo_root / "apps" / "desktop" / "Cargo.toml"
    if desktop_cargo.exists():
        match = re.search(r'version\s*=\s*"([^"]+)"', desktop_cargo.read_text(encoding="utf-8"))
        if match:
            return match.group(1)

    raise RuntimeError("Could not find current version in Cargo.toml")


def calculate_new_version(current: str, target: str) -> str:
    match = SEMVER_REGEX.match(current)
    if not match:
        raise ValueError(f"Current version '{current}' is not valid SemVer.")

    major = int(match.group("major"))
    minor = int(match.group("minor"))
    patch = int(match.group("patch"))
    prerelease = match.group("prerelease") or ""

    target_lower = target.lower()

    if target_lower == "patch":
        if prerelease:
            return f"{major}.{minor}.{patch}"
        return f"{major}.{minor}.{patch + 1}"
    elif target_lower == "minor":
        if prerelease and patch == 0:
            return f"{major}.{minor}.0"
        return f"{major}.{minor + 1}.0"
    elif target_lower == "major":
        if prerelease and minor == 0 and patch == 0:
            return f"{major}.0.0"
        return f"{major + 1}.0.0"
    elif target_lower == "alpha":
        if "alpha" in prerelease:
            num = re.search(r"alpha\.?(\d+)", prerelease)
            next_num = int(num.group(1)) + 1 if num is not None else 2
            return f"{major}.{minor}.{patch}-alpha.{next_num}"
        elif prerelease:
            return f"{major}.{minor}.{patch}-alpha.1"
        return f"{major}.{minor + 1}.0-alpha.1"
    elif target_lower == "beta":
        if "beta" in prerelease:
            num = re.search(r"beta\.?(\d+)", prerelease)
            next_num = int(num.group(1)) + 1 if num is not None else 2
            return f"{major}.{minor}.{patch}-beta.{next_num}"
        elif prerelease:
            return f"{major}.{minor}.{patch}-beta.1"
        return f"{major}.{minor + 1}.0-beta.1"
    elif target_lower == "rc":
        if "rc" in prerelease:
            num = re.search(r"rc\.?(\d+)", prerelease)
            next_num = int(num.group(1)) + 1 if num is not None else 2
            return f"{major}.{minor}.{patch}-rc.{next_num}"
        elif prerelease:
            return f"{major}.{minor}.{patch}-rc.1"
        return f"{major}.{minor + 1}.0-rc.1"
    elif target_lower in ("stable", "release"):
        return f"{major}.{minor}.{patch}"
    else:
        # Explicit version string
        explicit_match = SEMVER_REGEX.match(target)
        if not explicit_match:
            raise ValueError(
                f"Target '{target}' must be one of: major, minor, patch, alpha, beta, rc, or a valid SemVer string."
            )
        return target.lstrip("v")


def update_root_cargo_toml(repo_root: Path, new_version: str, dry_run: bool) -> None:
    cargo_path = repo_root / "Cargo.toml"
    content = cargo_path.read_text(encoding="utf-8")

    lines = content.splitlines()
    in_workspace_pkg = False
    updated = False

    for i, line in enumerate(lines):
        trimmed = line.strip()
        if trimmed == "[workspace.package]":
            in_workspace_pkg = True
            continue
        if in_workspace_pkg and trimmed.startswith("["):
            break
        if in_workspace_pkg and trimmed.startswith("version"):
            lines[i] = f'version = "{new_version}"'
            updated = True
            break

    if not updated:
        raise RuntimeError("Failed to update version under [workspace.package] in Cargo.toml")

    new_content = "\n".join(lines) + "\n"
    if not dry_run:
        cargo_path.write_text(new_content, encoding="utf-8")
    print(f"  [x] Cargo.toml (root) -> {new_version}")


def update_package_json(filepath: Path, new_version: str, dry_run: bool) -> None:
    if not filepath.exists():
        print(f"  [-] Skipped (not found): {filepath.name}")
        return

    content = filepath.read_text(encoding="utf-8")
    data = json.loads(content)
    data["version"] = new_version

    new_content = json.dumps(data, indent=2) + "\n"
    if not dry_run:
        filepath.write_text(new_content, encoding="utf-8")
    print(f"  [x] {filepath.relative_to(filepath.parents[2])} -> {new_version}")


def update_installer_iss(repo_root: Path, new_version: str, dry_run: bool) -> None:
    iss_path = repo_root / "scripts" / "desktop" / "installer.iss"
    if not iss_path.exists():
        return
    content = iss_path.read_text(encoding="utf-8")
    new_content = re.sub(r'#define AppVersion "[^"]+"', f'#define AppVersion "{new_version}"', content)
    if not dry_run:
        iss_path.write_text(new_content, encoding="utf-8")
    print(f"  [x] scripts/desktop/installer.iss -> {new_version}")


def run_cargo_check(repo_root: Path, dry_run: bool) -> None:
    if dry_run:
        print("  [dry-run] Would execute: cargo check --workspace")
        return
    print("  --> Updating Cargo.lock via 'cargo check --workspace'...")
    subprocess.run(["cargo", "check", "--workspace", "--quiet"], cwd=repo_root, check=True)
    print("  [x] Cargo.lock updated")


def check_clean_git(repo_root: Path) -> bool:
    res = subprocess.run(
        ["git", "status", "--porcelain"],
        cwd=repo_root,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )
    return len(res.stdout.strip()) == 0


def main():
    parser = argparse.ArgumentParser(description="Synchronously bump versions across all TNotes packages")
    parser.add_argument(
        "target",
        help="Version bump type (patch, minor, major, alpha, beta, rc) or explicit version string (e.g. 0.2.0)",
    )
    parser.add_argument("--dry-run", action="store_true", help="Preview version changes without writing files")
    parser.add_argument("--commit", action="store_true", help="Create a git commit with the version bump")
    parser.add_argument("--tag", action="store_true", help="Create an annotated git tag (implies --commit)")
    args = parser.parse_args()

    repo_root = get_repo_root()
    current_version = get_current_version(repo_root)
    new_version = calculate_new_version(current_version, args.target)

    print(f"==> Bumping version: {current_version} -> {new_version}")
    if args.dry_run:
        print("    (Running in dry-run mode, no files will be modified)")

    if (args.commit or args.tag) and not args.dry_run:
        if not check_clean_git(repo_root):
            print("Error: Git working tree contains uncommitted changes. Please commit or stash them first.", file=sys.stderr)
            sys.exit(1)

    print("--> Updating version files:")
    update_root_cargo_toml(repo_root, new_version, args.dry_run)
    update_package_json(repo_root / "apps" / "web" / "package.json", new_version, args.dry_run)
    update_package_json(repo_root / "apps" / "mobile" / "package.json", new_version, args.dry_run)
    update_installer_iss(repo_root, new_version, args.dry_run)

    run_cargo_check(repo_root, args.dry_run)

    if (args.commit or args.tag) and not args.dry_run:
        print("--> Creating Git commit:")
        files_to_stage = [
            "Cargo.toml",
            "Cargo.lock",
            "apps/web/package.json",
            "apps/mobile/package.json",
            "scripts/desktop/installer.iss",
        ]
        subprocess.run(["git", "add"] + files_to_stage, cwd=repo_root, check=True)
        commit_msg = f"chore(release): bump version to v{new_version}"
        subprocess.run(["git", "commit", "-m", commit_msg], cwd=repo_root, check=True)
        print(f"  [x] Committed: {commit_msg}")

        if args.tag:
            tag_name = f"v{new_version}"
            subprocess.run(["git", "tag", "-a", tag_name, "-m", f"Release {tag_name}"], cwd=repo_root, check=True)
            print(f"  [x] Created tag: {tag_name}")
            print(f"\n==> Done! Push release with: git push origin HEAD --tags")
            return

    print(f"\n==> Successfully bumped all applications to v{new_version}!")


if __name__ == "__main__":
    main()
