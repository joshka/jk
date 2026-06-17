#!/usr/bin/env python3
"""Reflow plain Rust comment prose to the repo comment width."""

from __future__ import annotations

import argparse
import re
import sys
import textwrap
from pathlib import Path

COMMENT_RE = re.compile(r"^(\s*)(//[/!]?) ?(.*)$")
NUMBERED_LIST_RE = re.compile(r"\d+[.)]\s")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "paths",
        nargs="*",
        type=Path,
        default=[Path("crates")],
        help="Rust files or directories to reflow",
    )
    parser.add_argument(
        "--width",
        type=int,
        default=100,
        help="target total line width, including comment prefix",
    )
    parser.add_argument(
        "--check",
        action="store_true",
        help="report files that would change without writing them",
    )
    args = parser.parse_args()

    changed = []
    for path in rust_files(args.paths):
        original = path.read_text()
        reflowed = reflow_text(original, args.width)
        if original == reflowed:
            continue
        changed.append(path)
        if not args.check:
            path.write_text(reflowed)

    if args.check and changed:
        for path in changed:
            print(path)
        return 1
    return 0


def rust_files(paths: list[Path]) -> list[Path]:
    files = []
    for path in paths:
        if path.is_dir():
            files.extend(sorted(path.rglob("*.rs")))
        elif path.suffix == ".rs":
            files.append(path)
    return files


def reflow_text(text: str, width: int) -> str:
    has_trailing_newline = text.endswith("\n")
    lines = text.splitlines()
    reflowed: list[str] = []
    index = 0

    while index < len(lines):
        block = read_comment_block(lines, index)
        if block is None:
            reflowed.append(lines[index])
            index += 1
            continue

        next_index, indent, prefix, comments = block
        reflowed.extend(reflow_comment_block(indent, prefix, comments, width))
        index = next_index

    result = "\n".join(reflowed)
    if has_trailing_newline:
        result += "\n"
    return result


def read_comment_block(
    lines: list[str], start: int
) -> tuple[int, str, str, list[str]] | None:
    match = COMMENT_RE.match(lines[start])
    if match is None:
        return None

    indent, prefix = match.group(1), match.group(2)
    comments = []
    index = start
    while index < len(lines):
        match = COMMENT_RE.match(lines[index])
        if match is None or match.group(1) != indent or match.group(2) != prefix:
            break
        comments.append(match.group(3))
        index += 1

    return index, indent, prefix, comments


def reflow_comment_block(
    indent: str, prefix: str, comments: list[str], width: int
) -> list[str]:
    lines: list[str] = []
    paragraph: list[str] = []
    in_fence = False

    def flush_paragraph() -> None:
        if not paragraph:
            return
        lines.extend(format_paragraph(indent, prefix, paragraph, width))
        paragraph.clear()

    for comment in comments:
        stripped = comment.strip()
        if stripped.startswith("```"):
            flush_paragraph()
            lines.append(format_comment(indent, prefix, comment))
            in_fence = not in_fence
            continue

        if in_fence or is_boundary_comment(comment):
            flush_paragraph()
            lines.append(format_comment(indent, prefix, comment))
            continue

        paragraph.append(comment)

    flush_paragraph()
    return lines


def is_boundary_comment(comment: str) -> bool:
    stripped = comment.strip()
    return (
        stripped == ""
        or comment.startswith(" ")
        or stripped.startswith(("- ", "* ", "+ ", "# ", "```"))
        or NUMBERED_LIST_RE.match(stripped) is not None
        or "|" in stripped
        or stripped.startswith(("http://", "https://"))
    )


def format_paragraph(
    indent: str, prefix: str, paragraph: list[str], width: int
) -> list[str]:
    body = " ".join(line.strip() for line in paragraph)
    comment_width = max(20, width - len(indent) - len(prefix) - 1)
    wrapped = textwrap.wrap(
        body,
        width=comment_width,
        break_long_words=False,
        break_on_hyphens=False,
    )
    if not wrapped:
        return [format_comment(indent, prefix, "")]
    return [format_comment(indent, prefix, line) for line in wrapped]


def format_comment(indent: str, prefix: str, body: str) -> str:
    if body == "":
        return f"{indent}{prefix}"
    return f"{indent}{prefix} {body}"


if __name__ == "__main__":
    sys.exit(main())
