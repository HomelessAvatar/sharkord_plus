#!/usr/bin/env python3
"""
Generate formatted markdown release notes from git commit history.
Works automatically for both GitHub Pull Request workflows and direct-push tag releases.
"""

import os
import re
import subprocess
import sys


def run_command(cmd: list[str]) -> str:
    try:
        res = subprocess.run(cmd, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, check=True)
        return res.stdout.strip()
    except Exception:
        return ""


def get_repo_name() -> str:
    repo = os.environ.get("GITHUB_REPOSITORY", "").strip()
    if repo:
        return repo
    remote = run_command(["git", "config", "--get", "remote.origin.url"])
    if remote:
        match = re.search(r"github\.com[:/]([^/]+/[^/\.]+)", remote)
        if match:
            return match.group(1)
    return "HomelessAvatar/sharkord_plus"


def main():
    tag = sys.argv[1] if len(sys.argv) > 1 and sys.argv[1].strip() else ""
    out_file = sys.argv[2] if len(sys.argv) > 2 and sys.argv[2].strip() else "release-notes.md"
    repo = get_repo_name()

    if not tag:
        tag = os.environ.get("GITHUB_REF_NAME", "").strip()
    if not tag:
        tag = run_command(["git", "describe", "--tags", "--exact-match"])
    if not tag:
        tag = run_command(["git", "describe", "--tags", "--abbrev=0"])

    print(f"[release-notes] Target tag: '{tag}', Repository: '{repo}', Output: '{out_file}'")

    # If the release already exists on GitHub and has formatted release notes, skip to prevent duplicate notes
    existing_body = ""
    try:
        existing_body = run_command(["gh", "release", "view", tag, "--repo", repo, "--json", "body", "--jq", ".body"])
    except Exception:
        pass

    if existing_body and ("What's Changed" in existing_body or "Highlights" in existing_body or len(existing_body) > 150):
        print(f"[release-notes] Release {tag} already has notes on GitHub ({len(existing_body)} chars). Skipping.")
        with open(out_file, "w", encoding="utf-8") as f:
            f.write("")
        return

    # Determine previous tag to compute range
    prev_tag = ""
    if tag:
        all_tags = [t.strip() for t in run_command(["git", "tag", "-l", "v*", "--sort=-v:refname"]).splitlines() if t.strip()]
        if tag in all_tags:
            idx = all_tags.index(tag)
            if idx + 1 < len(all_tags):
                prev_tag = all_tags[idx + 1]
        if not prev_tag:
            prev_tag = run_command(["git", "describe", "--tags", "--abbrev=0", f"{tag}^"])

    if tag and prev_tag:
        git_range = f"{prev_tag}..{tag}"
        log_raw = run_command(["git", "log", git_range, "--pretty=format:%h%x09%s"])
    elif tag:
        log_raw = run_command(["git", "log", tag, "-n", "30", "--pretty=format:%h%x09%s"])
    else:
        log_raw = run_command(["git", "log", "-n", "30", "--pretty=format:%h%x09%s"])

    lines = [l.strip() for l in log_raw.splitlines() if l.strip()]

    features: list[str] = []
    fixes: list[str] = []
    perf: list[str] = []
    docs: list[str] = []
    maintenance: list[str] = []
    others: list[str] = []

    for line in lines:
        parts = line.split("\t", 1)
        if len(parts) != 2:
            continue
        h, s = parts[0], parts[1]
        item = f"- {s} ({h})"
        lower = s.lower()

        if re.match(r"^feat(\([^\)]+\))?:", lower):
            features.append(item)
        elif re.match(r"^fix(\([^\)]+\))?:", lower):
            fixes.append(item)
        elif re.match(r"^perf(\([^\)]+\))?:", lower):
            perf.append(item)
        elif re.match(r"^docs(\([^\)]+\))?:", lower):
            docs.append(item)
        elif re.match(r"^(chore|ci|refactor|test|build)(\([^\)]+\))?:", lower):
            maintenance.append(item)
        else:
            others.append(item)

    sections: list[str] = []
    display_title = tag if tag else "Latest Release"
    sections.append(f"## VibeShark {display_title}\n")
    sections.append("### 📋 What's Changed\n")

    has_categorized = bool(features or fixes or perf or docs or maintenance)

    if has_categorized:
        if features:
            sections.append("#### 🚀 Features\n" + "\n".join(features) + "\n")
        if fixes:
            sections.append("#### 🐛 Bug Fixes\n" + "\n".join(fixes) + "\n")
        if perf:
            sections.append("#### ⚡ Performance Improvements\n" + "\n".join(perf) + "\n")
        if docs:
            sections.append("#### 📚 Documentation\n" + "\n".join(docs) + "\n")
        if maintenance:
            sections.append("#### 🔧 Maintenance & Tooling\n" + "\n".join(maintenance) + "\n")
        if others:
            sections.append("#### 📝 Other Changes\n" + "\n".join(others) + "\n")
    elif others:
        sections.append("\n".join(others) + "\n")
    else:
        sections.append("*No commit changes recorded.*")

    if prev_tag and tag:
        sections.append(f"**Full Changelog**: https://github.com/{repo}/compare/{prev_tag}...{tag}\n")
    elif tag:
        sections.append(f"**Release**: https://github.com/{repo}/releases/tag/{tag}\n")

    content = "\n".join(sections).strip() + "\n"

    with open(out_file, "w", encoding="utf-8") as f:
        f.write(content)

    print(f"[release-notes] Successfully written {len(content)} bytes to '{out_file}'")


if __name__ == "__main__":
    main()
