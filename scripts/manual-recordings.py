#!/usr/bin/env python3
"""Run the entire manual collection and retain reviewable evidence, including failures."""

import argparse
from datetime import datetime, timezone
import hashlib
import html
import json
import os
from pathlib import Path
import re
import shlex
import shutil
import signal
import subprocess
import sys
import tempfile


ROOT = Path(__file__).resolve().parent.parent
RUNNER_SHA256 = hashlib.sha256(Path(__file__).read_bytes()).hexdigest()
MEDIA_LIMIT = 20
FILE_LIMIT = 10 * 1024 * 1024
TOTAL_LIMIT = 40 * 1024 * 1024
IDENTIFIER = re.compile(r"^[a-z0-9][a-z0-9-]{0,39}$")


def now():
    return datetime.now(timezone.utc).isoformat()


def write_json(path, data):
    path.write_text(json.dumps(data, indent=2) + "\n")


def digest(path):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def tool_info(executable):
    path = shutil.which(str(executable))
    if not path:
        return {"path": str(executable), "error": "not found"}
    invoked = Path(path)
    path = invoked.resolve()
    # Tool-manager shims dispatch by their original basename. Resolve the selected
    # tool explicitly so hashes identify the program, not mise/rustup itself.
    if path.name == "mise" and invoked.name != "mise":
        selected = subprocess.run(
            [str(path), "which", invoked.name], cwd=ROOT,
            capture_output=True, text=True, check=False, timeout=15,
        )
        if selected.returncode == 0:
            path = Path(selected.stdout.strip()).resolve()
        else:
            fallback_path = os.pathsep.join(
                entry for entry in os.environ.get("PATH", "").split(os.pathsep)
                if Path(entry).resolve() != invoked.parent.resolve()
            )
            fallback = shutil.which(invoked.name, path=fallback_path)
            if not fallback:
                return {"path": str(invoked), "error": "could not resolve mise shim"}
            path = Path(fallback).resolve()
    if path.name == "rustup" and invoked.name != "rustup":
        selected = subprocess.run(
            [str(path), "which", invoked.name], cwd=ROOT,
            capture_output=True, text=True, check=False, timeout=15,
        )
        if selected.returncode:
            return {"path": str(invoked), "error": "could not resolve rustup proxy"}
        path = Path(selected.stdout.strip()).resolve()
    result = subprocess.run(
        [str(path), "--version"], capture_output=True, text=True, check=False, timeout=15
    )
    return {
        "invoked_path": str(invoked),
        "path": str(path),
        "sha256": digest(path),
        "version": (result.stdout + result.stderr).strip(),
        "version_exit_code": result.returncode,
    }


def provenance(binary):
    return {
        "recorded_at": now(),
        "source_repo": str(ROOT),
        "github": {
            key: os.environ[key]
            for key in (
                "GITHUB_SHA", "JK_PR_HEAD_SHA", "GITHUB_REPOSITORY", "GITHUB_RUN_ID",
                "GITHUB_RUN_ATTEMPT", "GITHUB_WORKFLOW", "GITHUB_EVENT_NAME",
            )
            if key in os.environ
        },
        "jk": tool_info(binary),
        "jj": tool_info("jj"),
        "rustc": tool_info("rustc"),
        "sources": {
            str(path.relative_to(ROOT)): digest(path)
            for pattern in (
                "Cargo.toml", "Cargo.lock", "crates/**/*.rs", "crates/**/Cargo.toml",
                "scripts/*.sh", "scripts/*.py",
            )
            for path in sorted(ROOT.glob(pattern))
        },
    }


def directives(tape):
    result = []
    for number, line in enumerate(tape.read_text().splitlines(), 1):
        if re.match(r"^\s*(Output|Screenshot|State)\s", line):
            words = shlex.split(line, comments=True)
            if len(words) != 2:
                raise ValueError(f"{tape}:{number}: expected one output path")
            result.append((words[0], Path(words[1])))
    return result


def scenario(tape):
    identifier = tape.stem
    if not IDENTIFIER.fullmatch(identifier):
        raise ValueError(f"Scenario basename must be a gallery identifier: {tape}")
    outputs = directives(tape)
    videos = [path for command, path in outputs if command == "Output"]
    screenshots = [path for command, path in outputs if command == "Screenshot"]
    states = [path for command, path in outputs if command == "State"]
    if len(videos) != 1 or videos[0].suffix != ".mp4":
        raise ValueError(f"{tape}: declare exactly one MP4 Output")
    if not screenshots:
        raise ValueError(f"{tape}: include at least one PNG/State checkpoint")
    names = [path.name for _, path in outputs]
    if len(names) != len(set(names)):
        raise ValueError(f"{tape}: output basenames must be unique")
    if any(path.suffix != ".png" for path in screenshots):
        raise ValueError(f"{tape}: screenshots must be PNG files")
    if {path.stem for path in screenshots} != {path.stem for path in states}:
        raise ValueError(f"{tape}: every screenshot must have a same-stem State JSON")
    if any(path.suffix != ".json" for path in states):
        raise ValueError(f"{tape}: terminal states must be JSON files")
    if len(screenshots) + 1 > MEDIA_LIMIT:
        raise ValueError(f"{tape}: split the scenario; gallery exceeds {MEDIA_LIMIT} media files")
    relative = tape.relative_to(ROOT)
    group = relative.parent.name if relative.parent.name != "manual" else identifier
    variant = (
        identifier if identifier == group or identifier.startswith(f"{group}-")
        else f"{group}-{identifier}"
    )
    if not IDENTIFIER.fullmatch(variant):
        raise ValueError(f"{tape}: family-prefixed gallery name must fit 40 lowercase characters")
    return {
        "id": identifier,
        "tape": str(relative),
        "group": group,
        "variant": variant,
        "video": videos[0].name,
        "checkpoints": [path.stem for path in screenshots],
        "media_count": len(screenshots) + 1,
    }


def collection(validate_manifest=True):
    scenarios = [scenario(tape) for tape in sorted((ROOT / "tapes/manual").rglob("*.tape"))]
    if not scenarios:
        raise ValueError("No manual tapes found under tapes/manual/")
    identifiers = [item["id"] for item in scenarios]
    if len(set(identifiers)) != len(identifiers):
        raise ValueError("Manual tape basenames must be globally unique")
    if len(scenarios) > 256:
        raise ValueError("The collection exceeds GitHub's 256-job matrix; add grouped sharding")
    manifest = ROOT / "docs/manual-coverage.json"
    if manifest.exists() and validate_manifest:
        recorded = {item["tape"] for item in json.loads(manifest.read_text())["scenarios"]}
        discovered = {item["tape"] for item in scenarios}
        if recorded != discovered:
            raise ValueError(
                "Coverage/scenario mismatch: "
                f"missing tapes={sorted(recorded - discovered)}, "
                f"unregistered tapes={sorted(discovered - recorded)}"
            )
    return scenarios


def select(identifier, validate_manifest=True):
    for item in collection(validate_manifest):
        if item["id"] == identifier:
            return item
    raise ValueError(f"Unknown scenario: {identifier}")


def snapshot_scripts(directory, metadata):
    """Keep reproducible helper bytes; never substitute newer files for recorded hashes."""
    scripts = {
        name: checksum for name, checksum in metadata.get("provenance", {}).get("sources", {}).items()
        if Path(name).parts[0] == "scripts" and Path(name).suffix in (".sh", ".py")
    }
    if not scripts:
        raise ValueError("No helper-script hashes were recorded; source snapshot cannot be recovered")
    snapshots = {}
    contents = {}
    for name, checksum in scripts.items():
        relative = Path(name)
        if relative.is_absolute() or ".." in relative.parts:
            raise ValueError(f"Invalid recorded script path: {name}")
        destination = Path("source-scripts") / relative.relative_to("scripts")
        existing = directory / destination
        content = existing.read_bytes() if existing.exists() else (ROOT / relative).read_bytes()
        if hashlib.sha256(content).hexdigest() != checksum:
            raise ValueError(f"{name}: bytes differ from recorded source hash; rerun this scenario")
        snapshots[name] = str(destination)
        contents[destination] = content
    # Validate the complete set before writing, so an unsuccessful backfill stays untouched.
    for relative, content in contents.items():
        destination = directory / relative
        destination.parent.mkdir(parents=True, exist_ok=True)
        destination.write_bytes(content)
    metadata["source_snapshots"] = snapshots
    metadata["source_snapshots_recorded_at"] = now()
    metadata["source_snapshot_status"] = "preserved"


def source_consistency(metadata):
    """Check helper files at capture completion, before later checkout edits are legitimate."""
    changed = []
    for name in metadata.get("source_snapshots", {}):
        try:
            if digest(ROOT / name) != metadata["provenance"]["sources"][name]:
                changed.append(name)
        except OSError:
            changed.append(name)
    return {"checked_at": now(), "passed": not changed, "changed_scripts": changed}


def prepare_fixture(text, directory, binary, identifier):
    """Supply a real operation ID to tapes that teach entering historical Run Options."""
    kinds = re.findall(r"^# PreparedFixture ([a-z0-9-]+)$", text, flags=re.MULTILINE)
    if not kinds:
        if "{{INITIAL_OPERATION}}" in text:
            raise ValueError("INITIAL_OPERATION requires a '# PreparedFixture KIND' declaration")
        return text, {}
    if len(kinds) != 1:
        raise ValueError("Declare exactly one prepared fixture per tape")
    environment = {
        **os.environ,
        "JK_SOURCE_REPO": str(ROOT), "JK_BIN": str(binary),
        "JK_MANUAL_ARTIFACT_DIR": str(directory), "JK_MANUAL_SCENARIO": identifier,
        "JK_MANUAL_PREPARE_FIXTURE": "1",
    }
    environment.pop("JK_MANUAL_PREPARED_FIXTURE", None)
    with (directory / "fixture-preparation.log").open("w") as log:
        result = subprocess.run(
            ["bash", "-c", 'source "$1" "$2"', "manual-fixture",
             str(ROOT / "scripts/manual-fixture.sh"), kinds[0]],
            cwd=ROOT, env=environment, stdout=log, stderr=subprocess.STDOUT,
            check=False, timeout=120,
        )
    if result.returncode:
        raise ValueError("Fixture preparation failed; inspect fixture-preparation.log")
    fixture = json.loads((directory / "fixture.json").read_text())
    operation = fixture["initial_operation"]
    if not isinstance(operation, str) or not re.fullmatch(r"[0-9a-f]{12,}", operation):
        raise ValueError("Prepared fixture did not produce a valid initial operation ID")
    environment_file = directory / "fixture-env.sh"
    if not environment_file.is_file():
        raise ValueError("Prepared fixture did not retain fixture-env.sh")
    return text.replace("{{INITIAL_OPERATION}}", operation), {
        "JK_MANUAL_PREPARED_FIXTURE": environment_file,
    }


def prepare(item, directory, binary, for_action=False, build_provenance=None):
    directory = directory.resolve()
    if directory.exists() and any(directory.iterdir()):
        raise ValueError(f"Evidence directory is not empty: {directory}; choose a fresh path")
    directory.mkdir(parents=True, exist_ok=True)
    binary = binary.resolve()
    if not binary.is_file() or not os.access(binary, os.X_OK):
        raise ValueError(f"Missing executable: {binary}; build this checkout first")
    if digest(Path(__file__)) != RUNNER_SHA256:
        raise ValueError("Runner source changed after launch; restart the recording collection")
    source = ROOT / item["tape"]
    source_bytes = source.read_bytes()
    metadata = {
        "schema": 1,
        "scenario": item,
        "status": "prepared",
        "started_at": now(),
        "source_tape_sha256": hashlib.sha256(source_bytes).hexdigest(),
        "provenance": provenance(binary),
    }
    if build_provenance:
        built = json.loads(build_provenance.read_text())
        if built["jk"]["sha256"] != metadata["provenance"]["jk"]["sha256"]:
            raise ValueError("Downloaded jk binary does not match build provenance")
        if built["sources"] != metadata["provenance"]["sources"]:
            raise ValueError("Build provenance does not match checked-out sources")
        shutil.copyfile(build_provenance, directory / "build-provenance.json")
    snapshot_scripts(directory, metadata)
    write_json(directory / "capture-metadata.json", metadata)
    (directory / "source.tape").write_bytes(source_bytes)
    capture_text, fixture_environment = prepare_fixture(
        source_bytes.decode(), directory, binary, item["id"]
    )
    lines = [
        f"Env {name} {json.dumps(str(value))}"
        for name, value in {
            "JK_SOURCE_REPO": ROOT,
            "JK_BIN": binary,
            "JK_MANUAL_ARTIFACT_DIR": directory,
            "JK_MANUAL_SCENARIO": item["id"],
            **fixture_environment,
        }.items()
    ]
    for line in capture_text.splitlines():
        if re.match(r"^\s*(Output|Screenshot|State)\s", line):
            command, destination = shlex.split(line, comments=True)
            if for_action and command == "Output":
                continue  # The action adds its video output; verify imports that exact file.
            line = f"{command} {json.dumps(str(directory / Path(destination).name))}"
        lines.append(line)
    (directory / "capture.tape").write_text("\n".join(lines) + "\n")
    if for_action:
        # The action derives gallery captions from the tape's basename.
        shutil.copyfile(directory / "capture.tape", directory / f"{item['id']}.tape")
    return directory


def import_action(directory, action_directory):
    """Retain the action's exact movie, logs and renderer provenance in the evidence archive."""
    metadata = json.loads((directory / "capture-metadata.json").read_text())
    video = action_directory / "tape-1/preview.mp4"
    if video.exists():
        shutil.copyfile(video, directory / metadata["scenario"]["video"])
    diagnostics = directory / "action-diagnostics"
    diagnostics.mkdir(exist_ok=True)
    for path in action_directory.rglob("*"):
        if path.is_file() and (path.suffix == ".log" or path.name == "manifest.json"):
            destination = diagnostics / path.relative_to(action_directory)
            destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(path, destination)
    renderer = action_directory / "bin/betamax"
    if renderer.exists():
        metadata["provenance"]["betamax"] = tool_info(renderer)
    manifest = action_directory / "manifest.json"
    if manifest.exists():
        result = json.loads(manifest.read_text())
        metadata["renderer_passed"] = (
            len(result.get("results", [])) == 1
            and result["results"][0]["status"] == "passed"
            and not result.get("problems")
        )
    else:
        metadata["renderer_passed"] = False
    metadata["source_consistency"] = source_consistency(metadata)
    write_json(directory / "capture-metadata.json", metadata)


def verify(directory):
    metadata_path = directory / "capture-metadata.json"
    metadata = json.loads(metadata_path.read_text())
    item = metadata["scenario"]
    problems = []
    if not metadata.get("renderer_passed"):
        problems.append("Renderer did not complete successfully; inspect render/action logs")
    if not metadata["provenance"].get("betamax", {}).get("sha256"):
        problems.append("Exact Betamax executable provenance is missing")
    snapshots = metadata.get("source_snapshots", {})
    for name, snapshot in snapshots.items():
        try:
            if digest(directory / snapshot) != metadata["provenance"]["sources"][name]:
                problems.append(f"{snapshot}: helper snapshot does not match recorded source hash")
        except (OSError, KeyError) as error:
            problems.append(f"{snapshot}: {error}")
    if metadata.get("source_consistency", {}).get("passed") is False:
        changed = ", ".join(metadata["source_consistency"]["changed_scripts"])
        problems.append(f"Helper sources changed during capture; rerun this scenario: {changed}")
    try:
        fixture = json.loads((directory / "fixture.json").read_text())
        for field in ("repository", "initial_commit", "initial_operation", "binary"):
            if not fixture.get(field):
                problems.append(f"Fixture provenance is missing {field}")
        if fixture.get("isolated_jj_config") != "/dev/null":
            problems.append("Fixture did not record isolated jj configuration")
    except (OSError, ValueError, AttributeError) as error:
        problems.append(f"Fixture provenance: {error}")
    media = [directory / item["video"]]
    states = []
    for checkpoint in item["checkpoints"]:
        png, state_path = directory / f"{checkpoint}.png", directory / f"{checkpoint}.json"
        media.append(png)
        row = {"checkpoint": checkpoint, "viewport_text": "", "styles": [], "size": None}
        try:
            if png.read_bytes()[:8] != b"\x89PNG\r\n\x1a\n":
                problems.append(f"{png.name}: invalid PNG signature")
        except OSError as error:
            problems.append(f"{png.name}: {error}")
        try:
            state = json.loads(state_path.read_text())
            if not state.get("viewport_text", "").strip():
                problems.append(f"{state_path.name}: empty terminal text")
            if not isinstance(state.get("styles"), list) or not state.get("viewport"):
                problems.append(f"{state_path.name}: missing styled terminal spans")
            row.update({
                "viewport_text": state.get("viewport_text", ""),
                "styles": state.get("styles", []),
                "size": state.get("size"),
            })
        except (OSError, ValueError, AttributeError) as error:
            problems.append(f"{checkpoint}: {error}")
            row["state_error"] = str(error)
        states.append(row)
    total = 0
    for path in media:
        if not path.is_file() or path.stat().st_size == 0:
            problems.append(f"Missing or empty media: {path.name}")
            continue
        size = path.stat().st_size
        total += size
        if size > FILE_LIMIT:
            problems.append(f"{path.name}: exceeds the action's 10 MiB/file limit")
    if total > TOTAL_LIMIT:
        problems.append("Scenario exceeds the action's 40 MiB total gallery limit")
    assertion_paths = sorted(directory.glob("*-assertions.json"))
    if not assertion_paths:
        problems.append("Repository outcome assertions are missing")
    for assertion_path in assertion_paths:
        try:
            assertions = json.loads(assertion_path.read_text())
            if assertions.get("scenario") != item["id"]:
                problems.append(f"{assertion_path.name}: assertion scenario does not match recording")
            checks = assertions.get("assertions", [])
            if (
                assertions.get("status") != "passed"
                or not checks
                or any(check.get("passed") is not True for check in checks)
            ):
                problems.append(f"{assertion_path.name}: outcome assertions are empty or failed")
        except (OSError, ValueError, AttributeError) as error:
            problems.append(f"{assertion_path.name}: {error}")
    write_json(directory / "checkpoint-summary.json", states)
    metadata.update({
        "status": "failed" if problems else "passed",
        "finished_at": now(),
        "problems": problems,
        "visual_review": "pending: inspect PNG pixels, terminal text and styled spans",
        "source_snapshot_status": (
            "preserved" if snapshots else "unavailable: original capture retained hashes only"
        ),
        "files": {
            str(path.relative_to(directory)): {"sha256": digest(path), "bytes": path.stat().st_size}
            for path in sorted(directory.rglob("*"))
            if path.is_file() and path != metadata_path and path.name != "review.html"
        },
    })
    write_json(metadata_path, metadata)
    gallery(directory, metadata, states)
    for problem in problems:
        print(f"{item['id']}: {problem}", file=sys.stderr)
    return not problems


def gallery(directory, metadata, states):
    esc = html.escape
    item = metadata["scenario"]
    cards = []
    for state in states:
        name = state["checkpoint"]
        state_link = (
            f'<a href="{esc(name)}.json">Full terminal state</a>'
            if (directory / f"{name}.json").exists() else ""
        )
        state_error = (
            f'<p>Terminal state missing or unreadable: {esc(state["state_error"])}</p>'
            if state.get("state_error") else ""
        )
        screenshot = (
            f'<img src="{esc(name)}.png" alt="{esc(name)}">'
            if (directory / f"{name}.png").exists() else '<p>Screenshot missing.</p>'
        )
        cards.append(
            f'<section><h2>{esc(name)}</h2>{screenshot}{state_error}'
            f'<details><summary>Terminal text and styles</summary><pre>{esc(state["viewport_text"])}'
            f'</pre><pre>{esc(json.dumps(state["styles"], indent=2))}</pre></details>'
            f'{state_link}</section>'
        )
    (directory / "review.html").write_text(
        '<!doctype html><html lang="en"><meta charset="utf-8">'
        '<meta name="viewport" content="width=device-width,initial-scale=1">'
        f'<title>{esc(item["id"])} manual evidence</title>'
        '<style>body{font:16px/1.5 system-ui;max-width:1100px;margin:2rem auto;padding:1rem;'
        'background:#111923;color:#e2e6ed}a{color:#74d6d0}img,video{max-width:100%}'
        'pre{overflow:auto;background:#202b3b;padding:1rem}section{margin:3rem 0}</style>'
        f'<h1>{esc(item["id"])}</h1><p>Automated checks: {esc(metadata["status"])}. '
        'Visual review remains a human judgment; inspect pixels and text together.</p>'
        '<p><a href="capture-metadata.json">Provenance and file hashes</a></p>'
        + "".join(
            f'<p><a href="{esc(path.name)}">Repository assertions: {esc(path.stem)}</a></p>'
            for path in sorted(directory.glob("*-assertions.json"))
        )
        + f'<video controls preload="metadata" src="{esc(item["video"])}"></video>'
        + "".join(f"<p>{esc(problem)}</p>" for problem in metadata["problems"])
        + "".join(cards) + "</html>\n"
    )


def run_one(item, directory, binary, renderer, timeout):
    prepare(item, directory, binary)
    metadata_path = directory / "capture-metadata.json"
    metadata = json.loads(metadata_path.read_text())
    renderer_info = tool_info(renderer)
    if "error" in renderer_info:
        raise ValueError(f"Cannot identify exact Betamax executable: {renderer_info['error']}")
    metadata["provenance"]["betamax"] = renderer_info
    metadata["status"] = "running"
    write_json(metadata_path, metadata)
    with (directory / "render.log").open("w") as log:
        process = subprocess.Popen(
            [renderer_info["path"], "run", "--quiet", str(directory / "capture.tape")],
            cwd=ROOT, stdout=log, stderr=subprocess.STDOUT, start_new_session=True,
            env={**os.environ, "BETAMAX_WORKING_DIRECTORY": str(ROOT)},
        )
        try:
            code = process.wait(timeout=timeout)
        except subprocess.TimeoutExpired:
            os.killpg(process.pid, signal.SIGTERM)
            try:
                process.wait(timeout=10)
            except subprocess.TimeoutExpired:
                os.killpg(process.pid, signal.SIGKILL)
                process.wait()
            code = 124
            log.write(f"\nScenario exceeded {timeout}s timeout.\n")
        metadata["renderer_exit_code"] = code
        metadata["renderer_passed"] = code == 0
        metadata["source_consistency"] = source_consistency(metadata)
        write_json(metadata_path, metadata)
    return verify(directory)


def finalize_error(item, directory, error):
    """Setup and launch failures need the same durable review page as a failed tape."""
    directory.mkdir(parents=True, exist_ok=True)
    path = directory / "capture-metadata.json"
    metadata = json.loads(path.read_text()) if path.exists() else {
        "schema": 1, "scenario": item, "started_at": now(), "provenance": {},
    }
    metadata.update({"renderer_passed": False, "setup_error": str(error)})
    write_json(path, metadata)
    with (directory / "render.log").open("a") as log:
        log.write(f"\nCapture could not complete: {error}\n")
    verify(directory)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    commands.add_parser("plan", help="Validate coverage and emit the complete GitHub matrix")
    build = commands.add_parser("provenance", help="Record exact build binary and source hashes")
    build.add_argument("--binary", type=Path, required=True)
    build.add_argument("--output", type=Path, required=True)
    prepare_parser = commands.add_parser("prepare", help="Prepare an isolated CI capture")
    prepare_parser.add_argument("--scenario", required=True)
    prepare_parser.add_argument("--directory", type=Path, required=True)
    prepare_parser.add_argument("--binary", type=Path, required=True)
    prepare_parser.add_argument("--build-provenance", type=Path)
    prepare_parser.add_argument("--for-action", action="store_true")
    verify_parser = commands.add_parser("verify", help="Retain renderer results and check evidence")
    verify_parser.add_argument("--directory", type=Path, required=True)
    verify_parser.add_argument("--action-directory", type=Path)
    snapshot_parser = commands.add_parser(
        "snapshot-sources", help="Recover script bytes only when they match existing recorded hashes"
    )
    snapshot_parser.add_argument("--directory", type=Path, required=True)
    run = commands.add_parser("run", help="Build and exercise all scenarios; keep failures")
    run.add_argument("--scenario", action="append", help="Limit local run; CI always runs all")
    run.add_argument("--binary", type=Path, help="Already-built absolute source binary")
    run.add_argument("--betamax", default=os.environ.get("BETAMAX", "betamax"))
    run.add_argument("--directory", type=Path)
    run.add_argument("--timeout", type=int, default=600)
    args = parser.parse_args()
    if args.command == "plan":
        print(json.dumps({"include": collection()}, separators=(",", ":")))
    elif args.command == "provenance":
        args.output.parent.mkdir(parents=True, exist_ok=True)
        write_json(args.output, provenance(args.binary.resolve()))
    elif args.command == "prepare":
        item = select(args.scenario)
        fresh = not args.directory.exists() or not any(args.directory.iterdir())
        try:
            prepare(item, args.directory, args.binary, args.for_action, args.build_provenance)
        except (OSError, ValueError, subprocess.SubprocessError) as error:
            if fresh:
                finalize_error(item, args.directory, error)
            raise
    elif args.command == "verify":
        if args.action_directory and args.action_directory.is_dir():
            import_action(args.directory, args.action_directory)
        return 0 if verify(args.directory) else 1
    elif args.command == "snapshot-sources":
        path = args.directory / "capture-metadata.json"
        metadata = json.loads(path.read_text())
        snapshot_scripts(args.directory, metadata)
        for snapshot in metadata["source_snapshots"].values():
            source = args.directory / snapshot
            metadata.setdefault("files", {})[snapshot] = {
                "sha256": digest(source), "bytes": source.stat().st_size,
            }
        write_json(path, metadata)
    elif args.command == "run":
        items = (
            [select(identifier, validate_manifest=False) for identifier in args.scenario]
            if args.scenario else collection()
        )
        if len({item["id"] for item in items}) != len(items):
            raise ValueError("Select each scenario at most once; each capture needs a fresh directory")
        parent = ROOT / "target/manual-recordings"
        parent.mkdir(parents=True, exist_ok=True)
        directory = args.directory or Path(tempfile.mkdtemp(prefix="run-", dir=parent))
        directory = directory.resolve()
        if directory.exists() and any(directory.iterdir()):
            raise ValueError(f"Run directory is not empty: {directory}")
        directory.mkdir(parents=True, exist_ok=True)
        print(f"Manual evidence: {directory}", flush=True)
        binary = args.binary
        if binary is None:
            with (directory / "build.log").open("w") as log:
                result = subprocess.run(
                    ["cargo", "build", "--locked", "-p", "jk"], cwd=ROOT,
                    stdout=log, stderr=subprocess.STDOUT, check=False,
                )
            if result.returncode:
                write_json(directory / "collection.json", {"status": "build failed", "scenarios": items})
                return result.returncode
            cargo = subprocess.check_output(
                ["cargo", "metadata", "--no-deps", "--format-version", "1"], cwd=ROOT, text=True
            )
            binary = Path(json.loads(cargo)["target_directory"]) / "debug/jk"
        results = []
        for item in items:
            try:
                passed = run_one(item, directory / item["id"], binary, args.betamax, args.timeout)
                result = {"id": item["id"], "passed": passed}
            except (OSError, ValueError, subprocess.SubprocessError) as error:
                finalize_error(item, directory / item["id"], error)
                result = {"id": item["id"], "passed": False, "error": str(error)}
            results.append(result)
            write_json(directory / "collection.json", {"finished_at": now(), "scenarios": results})
            print(f"{item['id']}: {'passed' if result['passed'] else 'FAILED'}", flush=True)
        links = "".join(
            f'<li><a href="{html.escape(row["id"])}/review.html">{html.escape(row["id"])}</a>: '
            f'{"passed" if row["passed"] else "FAILED"}</li>' for row in results
        )
        (directory / "index.html").write_text(
            '<!doctype html><html lang="en"><meta charset="utf-8"><title>jk manual recordings</title>'
            f'<h1>jk manual recordings</h1><ul>{links}</ul></html>\n'
        )
        return 0 if all(row["passed"] for row in results) else 1
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except (OSError, ValueError, KeyError, subprocess.SubprocessError) as error:
        print(f"manual recordings: {error}", file=sys.stderr)
        sys.exit(1)
