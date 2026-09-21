"""Contract checks for complete evidence and failure preservation, without creating repositories."""

import importlib.util
import io
import json
import os
from pathlib import Path
import tempfile
import unittest
from contextlib import redirect_stderr
from unittest import mock


spec = importlib.util.spec_from_file_location(
    "manual_recordings", Path(__file__).with_name("manual-recordings.py")
)
recordings = importlib.util.module_from_spec(spec)
spec.loader.exec_module(recordings)


class EvidenceContract(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)
        self.patch = mock.patch.object(recordings, "ROOT", self.root)
        self.patch.start()
        self.addCleanup(self.patch.stop)
        self.addCleanup(self.temp.cleanup)

    def tape(self, name="sample", count=1, family="changes"):
        path = self.root / "tapes/manual" / family / f"{name}.tape"
        path.parent.mkdir(parents=True, exist_ok=True)
        lines = [f"Output target/{name}.mp4"]
        for index in range(count):
            lines += [f"Screenshot target/{name}-{index}.png", f"State target/{name}-{index}.json"]
        path.write_text("\n".join(lines) + "\n")
        return path

    def evidence(self):
        item = recordings.scenario(self.tape())
        directory = self.root / "evidence"
        directory.mkdir()
        recordings.write_json(directory / "capture-metadata.json", {
            "scenario": item,
            "renderer_passed": True,
            "provenance": {"betamax": {"sha256": "example"}},
        })
        (directory / "sample.mp4").write_bytes(b"video")
        (directory / "sample-0.png").write_bytes(b"\x89PNG\r\n\x1a\n")
        recordings.write_json(directory / "sample-0.json", {
            "viewport_text": "Revision selected",
            "styles": [{"fg": "#123456"}],
            "viewport": [["Revision selected"]],
            "size": [80, 24],
        })
        recordings.write_json(directory / "fixture.json", {
            "repository": str(self.root / "disposable"),
            "initial_commit": "example-commit", "initial_operation": "example-operation",
            "binary": "/example/jk", "isolated_jj_config": "/dev/null",
        })
        recordings.write_json(directory / "success-assertions.json", {
            "scenario": "sample", "stage": "success", "status": "passed",
            "assertions": [{"name": "working-copy parent", "passed": True}],
        })
        return directory

    def test_gallery_limit_counts_video_and_every_checkpoint(self):
        self.assertEqual(recordings.scenario(self.tape(count=19))["media_count"], 20)
        with self.assertRaisesRegex(ValueError, "exceeds 20"):
            recordings.scenario(self.tape(count=20))

    def test_unpaired_state_fails_before_capture(self):
        tape = self.tape()
        tape.write_text(tape.read_text().replace("State target/sample-0.json\n", ""))
        with self.assertRaisesRegex(ValueError, "same-stem"):
            recordings.scenario(tape)

    def test_recursive_discovery_and_manifest_are_complete(self):
        self.tape()
        self.tape("remote", family="refs")
        manifest = self.root / "docs/manual-coverage.json"
        manifest.parent.mkdir()
        recordings.write_json(manifest, {"scenarios": [{
            "tape": "tapes/manual/changes/sample.tape",
        }]})
        with self.assertRaisesRegex(ValueError, "unregistered tapes=.*remote"):
            recordings.collection()

    def test_earlier_failed_assertion_cannot_be_hidden_by_later_success(self):
        directory = self.evidence()
        recordings.write_json(directory / "cancel-assertions.json", {
            "scenario": "sample", "stage": "cancel", "status": "failed",
            "assertions": [{"name": "unchanged after cancel", "passed": False}],
        })
        self.assertFalse(recordings.verify(directory))
        metadata = json.loads((directory / "capture-metadata.json").read_text())
        self.assertEqual(metadata["status"], "failed")
        self.assertTrue(any("cancel-assertions" in problem for problem in metadata["problems"]))
        self.assertTrue((directory / "review.html").exists())
        self.assertTrue((directory / "sample-0.png").exists())

    def test_missing_state_keeps_failure_metadata_and_available_pixels(self):
        directory = self.evidence()
        (directory / "sample-0.json").unlink()
        self.assertFalse(recordings.verify(directory))
        self.assertTrue((directory / "sample-0.png").exists())
        self.assertIn("failed", (directory / "capture-metadata.json").read_text())
        page = (directory / "review.html").read_text()
        self.assertIn('<img src="sample-0.png"', page)
        self.assertIn("Terminal state missing or unreadable", page)

    def test_evidence_checks_do_not_claim_visual_approval(self):
        directory = self.evidence()
        self.assertTrue(recordings.verify(directory))
        metadata = json.loads((directory / "capture-metadata.json").read_text())
        self.assertTrue(metadata["visual_review"].startswith("pending"))
        self.assertIn("sample-0.json", metadata["files"])

    def test_repository_assertions_are_required(self):
        directory = self.evidence()
        (directory / "success-assertions.json").unlink()
        self.assertFalse(recordings.verify(directory))

    def test_renderer_failure_cannot_pass_with_complete_checkpoints(self):
        directory = self.evidence()
        path = directory / "capture-metadata.json"
        data = json.loads(path.read_text())
        data["renderer_passed"] = False
        recordings.write_json(path, data)
        self.assertFalse(recordings.verify(directory))

    def test_reusing_evidence_directory_is_rejected(self):
        item = recordings.scenario(self.tape())
        directory = self.root / "existing"
        directory.mkdir()
        (directory / "previous.png").write_text("previous capture")
        with self.assertRaisesRegex(ValueError, "not empty"):
            recordings.prepare(item, directory, Path("missing-binary"))

    def test_mise_shim_identifies_selected_program_not_manager(self):
        actual = self.root / "tools/betamax"
        actual.parent.mkdir()
        actual.write_text("#!/bin/sh\necho 'betamax test-version'\n")
        actual.chmod(0o755)
        manager = self.root / "mise"
        manager.write_text(f"#!/bin/sh\nprintf '%s\\n' '{actual}'\n")
        manager.chmod(0o755)
        shim = self.root / "shims/betamax"
        shim.parent.mkdir()
        shim.symlink_to(manager)
        info = recordings.tool_info(shim)
        self.assertEqual(info["version"], "betamax test-version")
        self.assertEqual(info["sha256"], recordings.digest(actual))
        self.assertEqual(info["invoked_path"], str(shim))

    def test_inactive_mise_shim_identifies_path_fallback(self):
        actual = self.root / "tools/betamax"
        actual.parent.mkdir()
        actual.write_text("#!/bin/sh\necho 'betamax fallback-version'\n")
        actual.chmod(0o755)
        manager = self.root / "mise"
        manager.write_text("#!/bin/sh\nexit 1\n")
        manager.chmod(0o755)
        shim = self.root / "shims/betamax"
        shim.parent.mkdir()
        shim.symlink_to(manager)
        with mock.patch.dict(os.environ, {"PATH": f"{shim.parent}{os.pathsep}{actual.parent}"}):
            info = recordings.tool_info("betamax")
        self.assertEqual(info["version"], "betamax fallback-version")
        self.assertEqual(info["sha256"], recordings.digest(actual))

    def test_action_failure_import_keeps_exact_video_logs_and_failed_result(self):
        directory = self.evidence()
        action = self.root / "action"
        (action / "tape-1").mkdir(parents=True)
        (action / "tape-1/preview.mp4").write_bytes(b"action capture")
        (action / "tape-1.log").write_text("actual capture log")
        recordings.write_json(action / "manifest.json", {
            "results": [{"tape": "capture.tape", "status": "failed"}], "problems": [],
        })
        recordings.import_action(directory, action)
        self.assertEqual((directory / "sample.mp4").read_bytes(), b"action capture")
        self.assertEqual(
            (directory / "action-diagnostics/tape-1.log").read_text(), "actual capture log"
        )
        self.assertFalse(recordings.verify(directory))

    def test_setup_failure_finalizes_evidence_and_keeps_collection_running(self):
        self.tape()
        self.tape("second")
        binary = self.root / "jk"
        binary.write_text("#!/bin/sh\necho jk\n")
        binary.chmod(0o755)
        helper = self.root / "scripts/manual-fixture.sh"
        helper.parent.mkdir()
        helper.write_text("# helper fixture\n")
        provenance = {"sources": {"scripts/manual-fixture.sh": recordings.digest(helper)}}
        cases = {
            "missing-renderer": {"path": "/missing/betamax", "error": "not found"},
            "launch-exception": {"path": "/missing/betamax", "sha256": "example"},
        }
        for label, renderer in cases.items():
            with self.subTest(label=label):
                directory = self.root / label
                arguments = [
                    "manual-recordings.py", "run", "--directory", str(directory),
                    "--binary", str(binary), "--scenario", "sample", "--scenario", "second",
                ]
                with (
                    mock.patch("sys.argv", arguments),
                    mock.patch.object(recordings, "provenance", return_value=provenance),
                    mock.patch.object(recordings, "tool_info", return_value=renderer),
                    redirect_stderr(io.StringIO()),
                ):
                    self.assertEqual(recordings.main(), 1)
                for identifier in ("sample", "second"):
                    metadata = json.loads(
                        (directory / identifier / "capture-metadata.json").read_text()
                    )
                    self.assertEqual(metadata["status"], "failed")
                    self.assertTrue(metadata["setup_error"])
                    self.assertTrue((directory / identifier / "review.html").exists())
                    self.assertIn(
                        "Capture could not complete",
                        (directory / identifier / "render.log").read_text(),
                    )
                self.assertTrue((directory / "index.html").exists())

    def test_snapshots_keep_original_helpers_and_detect_mid_capture_edits(self):
        helper = self.root / "scripts/manual-fixture.sh"
        helper.parent.mkdir()
        helper.write_text("# original fixture\n")
        metadata = {"provenance": {"sources": {
            "scripts/manual-fixture.sh": recordings.digest(helper),
        }}}
        directory = self.root / "evidence"
        recordings.snapshot_scripts(directory, metadata)
        helper.write_text("# changed fixture\n")
        snapshot = directory / "source-scripts/manual-fixture.sh"
        self.assertEqual(snapshot.read_text(), "# original fixture\n")
        self.assertEqual(recordings.source_consistency(metadata)["changed_scripts"], [
            "scripts/manual-fixture.sh",
        ])
        # Already-preserved bytes remain usable after legitimate later checkout edits.
        recordings.snapshot_scripts(directory, metadata)
        self.assertEqual(snapshot.read_text(), "# original fixture\n")

    def test_backfill_rejects_newer_helpers_without_writing_partial_snapshot(self):
        helper = self.root / "scripts/manual-assert.sh"
        helper.parent.mkdir()
        helper.write_text("# original assertions\n")
        metadata = {"provenance": {"sources": {
            "scripts/manual-assert.sh": recordings.digest(helper),
        }}}
        helper.write_text("# changed assertions\n")
        directory = self.root / "evidence"
        directory.mkdir()
        with self.assertRaisesRegex(ValueError, "differ from recorded source hash"):
            recordings.snapshot_scripts(directory, metadata)
        self.assertFalse((directory / "source-scripts").exists())

    def test_prepared_fixture_types_real_operation_and_preserves_setup_log(self):
        helper = self.root / "scripts/manual-fixture.sh"
        helper.parent.mkdir()
        operation = "abcdef0123456789"
        helper.write_text(
            'printf "fixture kind: %s\\n" "$1"\n'
            f"printf '%s\\n' '{{\"initial_operation\":\"{operation}\"}}' "
            '> "$JK_MANUAL_ARTIFACT_DIR/fixture.json"\n'
            'printf "# Explicit prepared fixture variables\\n" '
            '> "$JK_MANUAL_ARTIFACT_DIR/fixture-env.sh"\n'
        )
        directory = self.root / "evidence"
        directory.mkdir()
        text, environment = recordings.prepare_fixture(
            '# PreparedFixture run-options\nType "{{INITIAL_OPERATION}}"\n',
            directory, Path("/source/jk"), "historical",
        )
        self.assertIn(f'Type "{operation}"', text)
        self.assertEqual(
            environment["JK_MANUAL_PREPARED_FIXTURE"], directory / "fixture-env.sh"
        )
        self.assertEqual(
            (directory / "fixture-preparation.log").read_text(), "fixture kind: run-options\n"
        )

    def test_operation_placeholder_requires_explicit_prepared_fixture(self):
        with self.assertRaisesRegex(ValueError, "requires a '# PreparedFixture"):
            recordings.prepare_fixture(
                'Type "{{INITIAL_OPERATION}}"', self.root, Path("jk"), "sample"
            )


if __name__ == "__main__":
    unittest.main()
