#!/usr/bin/env python3
"""Check that manual declarations still name the current source and recording collection.

This validates traceability, not what a human sees in a recording. Repository assertions and
terminal-state checks belong to manual-recordings.py; pixel review remains an explicit audit step.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import shlex
import sys
from pathlib import Path


def digest_source(path: Path) -> str:
    """Ignore the conventional trailing unit-test module when checking the production surface."""
    production = path.read_text().split("#[cfg(test)]\nmod tests", 1)[0]
    return hashlib.sha256(production.encode()).hexdigest()


def tape_media(path: Path) -> tuple[list[str], list[dict[str, str | None]]]:
    directives = []
    for line in path.read_text().splitlines():
        if re.match(r"^(Output|Screenshot|State)\s", line):
            directives.append(shlex.split(line, comments=True))
    outputs = [parts[1] for parts in directives if parts[0] == "Output"]
    screenshots = [parts[1] for parts in directives if parts[0] == "Screenshot"]
    states = [parts[1] for parts in directives if parts[0] == "State"]
    checkpoints = [
        {
            "id": Path(png).stem,
            "png": png,
            "state": next((state for state in states if Path(state).stem == Path(png).stem), None),
        }
        for png in screenshots
    ]
    return outputs, checkpoints


def sync_tapes(root: Path, manifest: dict) -> None:
    """Register authored tapes; never upgrade a recording's review status automatically."""
    previous = {scenario["id"]: scenario for scenario in manifest["scenarios"]}
    scenarios = []
    for tape in sorted((root / "tapes/manual").rglob("*.tape")):
        scenario_id = tape.stem
        scenario = previous.get(scenario_id, {})
        matching = [row for row in manifest["commands"] if scenario_id in row["scenario_ids"]]
        slugs = list(dict.fromkeys(row["manual_slug"] for row in matching))
        slugs = slugs or scenario.get("manual_slugs", [scenario.get("manual_slug", "inspect")])
        outputs, checkpoints = tape_media(tape)
        video = next((output for output in outputs if output.endswith((".mp4", ".webm", ".gif"))), None)
        tape_hash = hashlib.sha256(tape.read_bytes()).hexdigest()
        recording = scenario.get("recording", {"status": "planned"})
        if scenario.get("tape_sha256") != tape_hash:
            recording = {"status": "planned"}
        recording.update(video=video, checkpoints=checkpoints)
        scenario.update(
            id=scenario_id,
            title=scenario.get("title", scenario_id.replace("-", " ").capitalize()),
            group=tape.parent.name if tape.parent.name != "manual" else slugs[0],
            tape=str(tape.relative_to(root)),
            tape_sha256=tape_hash,
            manual_slug=slugs[0],
            manual_slugs=slugs,
            recording=recording,
        )
        scenarios.append(scenario)
    manifest["scenarios"] = scenarios


def refresh_source_lock(root: Path, manifest: dict) -> None:
    for item in manifest["source_lock"]:
        item["sha256"] = digest_source(root / item["path"])
    manifest["source_modules"] = sorted(
        re.findall(r"^pub mod (\w+);", (root / "crates/jk-cli/src/lib.rs").read_text(), re.MULTILINE)
    )
    for row in manifest["commands"]:
        for anchor in row["source_anchors"]:
            source = (root / anchor["path"]).read_text()
            if anchor["symbol"] in source:
                anchor["line"] = source[: source.index(anchor["symbol"])].count("\n") + 1


def validate(root: Path, manifest: dict, website: Path | None, incomplete: bool) -> list[str]:
    errors = []

    def require(condition: bool, message: str, *, unfinished: bool = False) -> None:
        if not condition and not (incomplete and unfinished):
            errors.append(message)

    require(manifest.get("schema_version") == 1, "unsupported schema_version")
    for collection in ["chapters", "commands", "scenarios", "findings"]:
        key = "slug" if collection == "chapters" else "id"
        ids = [row[key] for row in manifest[collection]]
        require(len(ids) == len(set(ids)), f"duplicate {collection} identifiers")

    chapters = {chapter["slug"]: chapter for chapter in manifest["chapters"]}
    scenarios = {scenario["id"]: scenario for scenario in manifest["scenarios"]}
    findings = {finding["id"]: finding for finding in manifest["findings"]}
    source_modules = sorted(
        re.findall(r"^pub mod (\w+);", (root / "crates/jk-cli/src/lib.rs").read_text(), re.MULTILINE)
    )
    require(source_modules == manifest["source_modules"], "jk-cli public modules changed; review command coverage")
    for item in manifest["source_lock"]:
        path = root / item["path"]
        require(path.is_file(), f"missing locked source: {item['path']}")
        if path.is_file():
            require(digest_source(path) == item["sha256"], f"source changed: {item['path']}; review forms, then --refresh-source-lock")

    for row in manifest["commands"]:
        prefix = row["id"]
        for field in ["title", "family", "jj_form", "intent", "starting_state", "result", "verification", "cancel", "failure", "recovery"]:
            require(bool(row.get(field)), f"{prefix}: missing {field}")
        require(row["support"] in {"native", "command-mode", "unavailable"}, f"{prefix}: invalid support")
        require(row["manual_slug"] in chapters, f"{prefix}: unknown manual chapter")
        require(bool(row["source_anchors"]), f"{prefix}: no source anchors")
        if row["support"] == "native":
            require(bool(row["scenario_ids"]), f"{prefix}: native form has no scenario", unfinished=True)
        for scenario_id in row["scenario_ids"]:
            require(scenario_id in scenarios, f"{prefix}: missing scenario {scenario_id}", unfinished=True)
        for finding_id in row["finding_ids"]:
            require(finding_id in findings, f"{prefix}: missing finding {finding_id}")
        for anchor in row["source_anchors"]:
            path = root / anchor["path"]
            require(path.is_file(), f"{prefix}: missing source {anchor['path']}")
            if path.is_file():
                source = path.read_text()
                require(anchor["symbol"] in source, f"{prefix}: source marker disappeared: {anchor['symbol']}")
                if anchor["symbol"] in source:
                    line = source[: source.index(anchor["symbol"])].count("\n") + 1
                    require(anchor["line"] == line, f"{prefix}: source anchor line changed; --refresh-source-lock after review")

    declared_tapes = {scenario["tape"] for scenario in scenarios.values()}
    authored_tapes = {str(path.relative_to(root)) for path in (root / "tapes/manual").rglob("*.tape")}
    require(declared_tapes == authored_tapes, "authored/declared tape collection differs; --sync-tapes")
    for scenario in scenarios.values():
        prefix = scenario["id"]
        tape = root / scenario["tape"]
        require(tape.is_file(), f"{prefix}: missing tape", unfinished=True)
        require(scenario["manual_slug"] in chapters, f"{prefix}: unknown manual chapter")
        require(all(slug in chapters for slug in scenario["manual_slugs"]), f"{prefix}: unknown manual_slugs entry")
        recording = scenario["recording"]
        require(recording["status"] in {"planned", "verified"}, f"{prefix}: invalid recording status")
        if tape.is_file():
            outputs, checkpoints = tape_media(tape)
            require(recording["video"] in outputs, f"{prefix}: video not declared by tape")
            require(recording["checkpoints"] == checkpoints, f"{prefix}: checkpoints differ from tape; --sync-tapes")
            require(bool(checkpoints), f"{prefix}: no PNG checkpoints")
            require(all(checkpoint["state"] for checkpoint in checkpoints), f"{prefix}: missing PNG/State pair")
            require(scenario.get("tape_sha256") == hashlib.sha256(tape.read_bytes()).hexdigest(), f"{prefix}: tape changed; --sync-tapes resets review status")
        if recording["status"] == "verified":
            for field in ["evidence_index", "pixel_review", "text_style_review", "source_commit"]:
                require(bool(recording.get(field)), f"{prefix}: verified recording lacks {field}")

    for finding in findings.values():
        path = root / finding["path"]
        require(path.is_file(), f"missing findings document: {finding['path']}", unfinished=True)
        if path.is_file():
            anchors = [re.sub(r"[^\w\- ]", "", heading.lower()).replace(" ", "-") for heading in re.findall(r"^#{1,6} (.+)$", path.read_text(), re.MULTILINE)]
            require(finding["anchor"] in anchors, f"{finding['id']}: missing heading anchor {finding['anchor']}", unfinished=True)
    if website:
        for chapter in chapters.values():
            require((website / chapter["path"]).is_file(), f"missing website chapter: {chapter['path']}")
    return errors


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest", type=Path, default=Path("docs/manual-coverage.json"))
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument("--website-root", type=Path)
    parser.add_argument("--sync-tapes", action="store_true", help="register authored tapes and reset changed recordings to planned")
    parser.add_argument("--refresh-source-lock", action="store_true", help="accept source fingerprints and line anchors AFTER reviewing affected coverage")
    parser.add_argument("--allow-incomplete", action="store_true", help="allow unfinished native mappings/findings while authoring; never use as the CI gate")
    args = parser.parse_args()
    root = args.root.resolve()
    path = args.manifest if args.manifest.is_absolute() else root / args.manifest
    manifest = json.loads(path.read_text())
    if args.sync_tapes:
        sync_tapes(root, manifest)
    if args.refresh_source_lock:
        refresh_source_lock(root, manifest)
    if args.sync_tapes or args.refresh_source_lock:
        path.write_text(json.dumps(manifest, indent=2) + "\n")
    errors = validate(root, manifest, args.website_root, args.allow_incomplete)
    for error in errors:
        print(f"ERROR: {error}", file=sys.stderr)
    native = sum(row["support"] == "native" for row in manifest["commands"])
    print(f"Manual coverage: {len(manifest['commands'])} entries, {native} native forms, {len(manifest['scenarios'])} scenarios, {len(errors)} errors")
    return bool(errors)


if __name__ == "__main__":
    sys.exit(main())
