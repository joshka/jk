#!/usr/bin/env python3
"""Preserve observed scope and recovery outcomes for the disposable UX audit fixture."""

import json
from pathlib import Path
import subprocess
import sys


stage, binary, repository, sibling, artifacts = sys.argv[1:]
repo = Path(repository)
scratch = Path(sibling)
output = Path(artifacts)
baseline_path = output / "audit-baseline.json"
scenario = "audit-scope-context"


def jj(at, *args):
    return subprocess.check_output(
        [binary, "-R", str(at), "--ignore-working-copy", *args],
        text=True,
        stderr=subprocess.PIPE,
    ).strip()


def snapshot(at):
    return {
        "description": jj(at, "log", "--no-graph", "-r", "@", "-T", "description"),
        "commit": jj(at, "log", "--no-graph", "-r", "@", "-T", "commit_id"),
    }


if stage == "prepare":
    baseline_path.write_text(
        json.dumps({"default": snapshot(repo), "scratch": snapshot(scratch)}, indent=2) + "\n"
    )
    print("MANUAL_AUDIT_BASELINE_READY")
    raise SystemExit(0)

baseline = json.loads(baseline_path.read_text())
checks = []


def check(name, actual, expected):
    checks.append({"name": name, "passed": actual == expected, "actual": actual, "expected": expected})


check("sibling working-copy identity and description stay unchanged", snapshot(scratch), baseline["scratch"])
check(
    "both original workspace registrations remain",
    sorted(jj(repo, "workspace", "list", "-T", 'name ++ "\n"').splitlines()),
    ["default", "scratch"],
)
if stage == "scope":
    check(
        "command from sibling inspection changes startup working copy",
        snapshot(repo)["description"],
        "Scope audit changed startup workspace",
    )
    before = output / "audit-scope-context-bookmarks.json"
    after = output / "audit-scope-context-hidden-help.json"
    check(
        "action mode leaves bookmark viewport text unchanged",
        json.loads(after.read_text())["viewport_text"],
        json.loads(before.read_text())["viewport_text"],
    )
elif stage == "recovered":
    check("Undo restores startup working-copy identity and description", snapshot(repo), baseline["default"])
else:
    raise SystemExit(f"Unknown audit assertion stage: {stage}")

passed = all(item["passed"] for item in checks)
(output / f"audit-{stage}-assertions.json").write_text(
    json.dumps(
        {"scenario": scenario, "stage": stage, "status": "passed" if passed else "failed", "assertions": checks},
        indent=2,
    ) + "\n"
)
if not passed:
    for item in checks:
        if not item["passed"]:
            print(json.dumps(item))
    raise SystemExit(1)
print(f"MANUAL_AUDIT_{stage.upper()}_CONFIRMED")
