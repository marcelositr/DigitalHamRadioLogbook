#!/usr/bin/env python3
"""Validate the repository's live Markdown documentation without network access."""

from __future__ import annotations

import re
import sys
from pathlib import Path
from urllib.parse import unquote

ROOT = Path(__file__).resolve().parents[1]

LIVE_MARKDOWN = [
    ROOT / "README.md",
    ROOT / "docs" / "README.md",
    *sorted((ROOT / "docs" / "architecture").glob("*.md")),
    *sorted((ROOT / "docs" / "data").glob("*.md")),
    *sorted((ROOT / "docs" / "operations").glob("*.md")),
    ROOT / "docs" / "quality" / "VISUAL-QA-v0.11.md",
    ROOT / "docs" / "releases" / "CHANGELOG.md",
    ROOT / "docs" / "releases" / "PRE-1.0-READINESS.md",
    ROOT / "docs" / "releases" / "RELEASE-CHECKLIST.md",
]

REQUIRED_DOC_DIRS = {
    "architecture",
    "data",
    "operations",
    "project",
    "quality",
    "releases",
}

INLINE_LINK = re.compile(r"!?\[[^\]]*\]\(([^)]+)\)")
FENCED_BLOCK = re.compile(r"```.*?```|~~~.*?~~~", re.DOTALL)


def strip_fenced_code(text: str) -> str:
    return FENCED_BLOCK.sub("", text)


def normalize_target(raw_target: str) -> str:
    target = raw_target.strip()
    if target.startswith("<") and target.endswith(">"):
        target = target[1:-1].strip()
    if " " in target and not target.startswith(("http://", "https://")):
        target = target.split(" ", 1)[0]
    return unquote(target)


def validate_structure(errors: list[str]) -> None:
    root_markdown = sorted(path.name for path in ROOT.glob("*.md"))
    if root_markdown != ["README.md"]:
        errors.append(
            "repository root must contain only README.md as Markdown; found: "
            + ", ".join(root_markdown)
        )

    docs_root = ROOT / "docs"
    actual_dirs = {path.name for path in docs_root.iterdir() if path.is_dir()}
    missing = REQUIRED_DOC_DIRS - actual_dirs
    if missing:
        errors.append("missing documentation directories: " + ", ".join(sorted(missing)))

    if not (docs_root / "README.md").is_file():
        errors.append("missing docs/README.md documentation index")


def validate_links(errors: list[str]) -> int:
    checked = 0

    for source in LIVE_MARKDOWN:
        if not source.is_file():
            errors.append(f"missing live documentation file: {source.relative_to(ROOT)}")
            continue

        text = strip_fenced_code(source.read_text(encoding="utf-8"))
        for match in INLINE_LINK.finditer(text):
            target = normalize_target(match.group(1))

            if not target or target.startswith("#"):
                continue
            if target.startswith(("http://", "https://", "mailto:", "tel:")):
                continue

            target_without_fragment = target.split("#", 1)[0]
            if not target_without_fragment:
                continue

            resolved = (source.parent / target_without_fragment).resolve()
            try:
                resolved.relative_to(ROOT.resolve())
            except ValueError:
                errors.append(
                    f"{source.relative_to(ROOT)} links outside repository: {target}"
                )
                continue

            checked += 1
            if not resolved.exists():
                errors.append(
                    f"{source.relative_to(ROOT)} has broken relative link: {target}"
                )

    return checked


def main() -> int:
    errors: list[str] = []
    validate_structure(errors)
    checked = validate_links(errors)

    if errors:
        print("Documentation validation failed:", file=sys.stderr)
        for error in errors:
            print(f"  - {error}", file=sys.stderr)
        return 1

    print(
        f"Documentation validation passed: {len(LIVE_MARKDOWN)} live Markdown files, "
        f"{checked} relative links checked."
    )
    print(
        "Historical snapshots are intentionally excluded from link freshness checks; "
        "their original context is preserved."
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
