#!/usr/bin/env python3
"""
Generates SHA256SUMS.txt and latest.json update manifest from distribution artifacts.
Usage:
    python3 generate-manifest.py [--repo ToonionOfficial/tnotes] [--version 0.1.0] [--tag v0.1.0] [--dist-dir ./dist]
"""

import argparse
import datetime
import hashlib
import json
import re
import sys
from pathlib import Path


def get_version_from_cargo(repo_root: Path) -> str:
    root_cargo = repo_root / "Cargo.toml"
    if root_cargo.exists():
        content = root_cargo.read_text(encoding="utf-8")
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

    cargo_path = repo_root / "apps" / "desktop" / "Cargo.toml"
    if cargo_path.exists():
        content = cargo_path.read_text(encoding="utf-8")
        match = re.search(r'version\s*=\s*"([^"]+)"', content)
        if match:
            return match.group(1)
    return "0.1.0"


def sha256_file(filepath: Path) -> str:
    hasher = hashlib.sha256()
    with open(filepath, "rb") as f:
        for chunk in iter(lambda: f.read(65536), b""):
            hasher.update(chunk)
    return hasher.hexdigest()


def main():
    parser = argparse.ArgumentParser(description="Generate release manifest and checksums")
    parser.add_argument("--repo", default="ToonionOfficial/tnotes", help="GitHub repository (owner/repo)")
    parser.add_argument("--version", default="", help="Semantic version string (e.g. 0.1.0)")
    parser.add_argument("--tag", default="", help="Git tag name (e.g. v0.1.0)")
    parser.add_argument("--dist-dir", default="", help="Path to dist directory containing build artifacts")
    args = parser.parse_args()

    repo_root = Path(__file__).resolve().parent.parent.parent
    dist_dir = Path(args.dist_dir) if args.dist_dir else (repo_root / "dist")

    if not dist_dir.exists():
        print(f"Error: dist directory {dist_dir} does not exist.", file=sys.stderr)
        sys.exit(1)

    version = args.version
    if not version:
        if args.tag:
            version = args.tag.lstrip("v")
        else:
            version = get_version_from_cargo(repo_root)

    tag = args.tag if args.tag else f"v{version}"
    base_download_url = f"https://github.com/{args.repo}/releases/download/{tag}"

    print(f"Generating manifest for {args.repo} {tag} (v{version})...")

    # 1. Compute checksums for all files in dist_dir (excluding existing checksums and manifest)
    checksum_lines = []
    found_files = {}

    for item in sorted(dist_dir.iterdir()):
        if item.is_file() and item.name not in ("SHA256SUMS.txt", "latest.json"):
            h = sha256_file(item)
            checksum_lines.append(f"{h}  {item.name}")
            found_files[item.name] = {
                "sha256": h,
                "size": item.stat().st_size,
                "path": item,
            }

    checksums_file = dist_dir / "SHA256SUMS.txt"
    checksums_file.write_text("\n".join(checksum_lines) + "\n", encoding="utf-8")
    print(f"Wrote {len(checksum_lines)} entries to {checksums_file.name}")

    # 2. Build platform asset mapping
    platform_mappings = {
        "linux-x86_64": [
            "TNotes-Linux-x86_64.AppImage",
            f"tnotes-{version}-linux-x86_64.AppImage",
        ],
        "darwin-aarch64": [
            "TNotes-macOS-arm64.zip",
            f"tnotes-{version}-macos-arm64.zip",
        ],
        "darwin-x86_64": [
            "TNotes-macOS-x64.zip",
            f"tnotes-{version}-macos-x64.zip",
        ],
        "windows-x86_64": [
            "TNotes-Windows-x64-Setup.exe",
            f"tnotes-{version}-windows-x64-setup.exe",
        ],
    }

    platforms_json = {}
    for platform_key, candidate_filenames in platform_mappings.items():
        matched_filename = None
        for cand in candidate_filenames:
            if cand in found_files:
                matched_filename = cand
                break

        if matched_filename:
            file_info = found_files[matched_filename]
            platforms_json[platform_key] = {
                "name": matched_filename,
                "url": f"{base_download_url}/{matched_filename}",
                "sha256": file_info["sha256"],
                "size": file_info["size"],
            }
        else:
            print(f"Notice: No artifact found for platform key '{platform_key}' in {dist_dir.name}")

    manifest_data = {
        "version": version,
        "tag": tag,
        "pub_date": datetime.datetime.now(datetime.timezone.utc).isoformat(),
        "changelog_url": f"https://github.com/{args.repo}/releases/tag/{tag}",
        "platforms": platforms_json,
    }

    manifest_file = dist_dir / "latest.json"
    manifest_file.write_text(json.dumps(manifest_data, indent=2) + "\n", encoding="utf-8")
    print(f"Wrote manifest to {manifest_file.name}")
    print(json.dumps(manifest_data, indent=2))


if __name__ == "__main__":
    main()
