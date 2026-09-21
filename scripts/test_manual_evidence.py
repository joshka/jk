"""Check readiness against archived bytes and independent reviews, without repositories."""

import copy
import importlib.util
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest


SCRIPT = Path(__file__).with_name("prepare-manual-evidence.py")
SPEC = importlib.util.spec_from_file_location("manual_evidence", SCRIPT)
evidence = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(evidence)


def write_json(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2) + "\n")


def reviewed():
    return {
        "status": "reviewed",
        "reviewer": "Evidence reviewer",
        "reviewed_at": "2026-09-21T03:00:00+00:00",
        "summary": "Inspected the exact archived bytes and their visible outcome.",
        "findings": [],
    }


class EvidenceReadiness(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.archive = self.root / "target/evidence/sample"
        self.archive.mkdir(parents=True)
        self.reviews = self.root / "target/reviews"
        self.reviews.mkdir(parents=True)
        self.review_path = self.reviews / "review.json"
        self.metadata_path = self.archive / "capture-metadata.json"
        self.tape = self.root / "tapes/manual/sample.tape"
        self.tape.parent.mkdir(parents=True)
        self.tape.write_text("Output sample.mp4\nScreenshot checkpoint.png\nState checkpoint.json\n")
        source_contents = {
            "Cargo.toml": "[workspace]\nmembers = ['crates/sample']\n",
            "Cargo.lock": "version = 4\n",
            "crates/sample/Cargo.toml": "[package]\nname = 'sample'\nversion = '0.1.0'\n",
            "crates/sample/src/lib.rs": "pub fn example() {}\n",
            "scripts/fixture.sh": "#!/bin/sh\nprintf 'fixture ready\\n'\n",
        }
        for name, contents in source_contents.items():
            path = self.root / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(contents)
        write_json(self.root / "docs/manual-coverage.json", {"scenarios": [{
            "id": "sample", "title": "Review one change", "tape": "tapes/manual/sample.tape",
            "recording": {"video": "target/manual/sample.mp4", "checkpoints": [{"id": "checkpoint"}]},
        }]})
        # This suite tests integrity decisions, not PNG or MP4 decoding.
        for name, contents in {
            "source.tape": self.tape.read_bytes(),
            "capture.tape": self.tape.read_bytes(),
            "fixture.json": b'{"repository":"disposable-fixture"}\n',
            "checkpoint.png": b"archived PNG bytes",
            "checkpoint.json": b'{"viewport_text":"Selected change","styles":[{}]}\n',
            "sample.mp4": b"archived MP4 bytes",
            "render.log": b"renderer finished\n",
            "source-scripts/fixture.sh": (self.root / "scripts/fixture.sh").read_bytes(),
        }.items():
            path = self.archive / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(contents)
        write_json(self.archive / "success-assertions.json", {
            "scenario": "sample", "status": "passed",
            "assertions": [{"name": "selected parent matches outcome", "passed": True}],
        })
        tool = {"sha256": "1" * 64, "version": "test tool 1.0"}
        self.metadata = {
            "schema": 1,
            "scenario": {"id": "sample", "tape": "tapes/manual/sample.tape",
                         "video": "sample.mp4", "checkpoints": ["checkpoint"]},
            "started_at": "2026-09-21T02:00:00+00:00",
            "status": "passed", "renderer_passed": True, "problems": [],
            "source_tape_sha256": evidence.digest(self.tape),
            "source_snapshot_status": "preserved",
            "source_snapshots": {"scripts/fixture.sh": "source-scripts/fixture.sh"},
            "source_consistency": {"passed": True, "changed_scripts": []},
            "provenance": {
                "sources": {name: evidence.digest(self.root / name) for name in source_contents},
                "jk": dict(tool), "jj": dict(tool), "betamax": dict(tool),
                "github": {"GITHUB_SHA": "a" * 40, "JK_PR_HEAD_SHA": "b" * 40,
                           "GITHUB_REPOSITORY": "example/project", "GITHUB_RUN_ID": "123",
                           "GITHUB_RUN_ATTEMPT": "1"},
            },
            "files": {
                str(path.relative_to(self.archive)): {
                    "sha256": evidence.digest(path), "bytes": path.stat().st_size,
                }
                for path in self.archive.rglob("*") if path.is_file()
            },
        }
        write_json(self.metadata_path, self.metadata)
        self.review = {
            "scenario_id": "sample",
            "source_tape_sha256": evidence.digest(self.tape),
            "capture_metadata_sha256": evidence.digest(self.metadata_path),
            "checkpoints": [{
                "id": "checkpoint",
                "png_sha256": evidence.digest(self.archive / "checkpoint.png"),
                "state_sha256": evidence.digest(self.archive / "checkpoint.json"),
                "pixel_review": reviewed(), "text_style_review": reviewed(),
            }],
            "video_review": {**reviewed(), "sha256": evidence.digest(self.archive / "sample.mp4")},
        }
        self.save_review()

    def save_review(self):
        write_json(self.review_path, {"schema_version": 1, "reviews": [self.review]})

    def save_metadata(self):
        """Bind a fresh review so an integrity test cannot pass just by invalidating reviews."""
        write_json(self.metadata_path, self.metadata)
        self.review["capture_metadata_sha256"] = evidence.digest(self.metadata_path)
        self.save_review()

    def result(self, reviews=None):
        return evidence.report(
            self.root, [self.archive], reviews or [self.review_path], "reviewed-v1"
        )["scenarios"][0]

    def assert_blocked(self, fragment):
        result = self.result()
        self.assertEqual(result["status"], "blocked", result)
        self.assertTrue(any(fragment in problem for problem in result["problems"]), result["problems"])

    def test_complete_exact_capture_and_reviews_are_ready(self):
        result = self.result()
        self.assertEqual(result["status"], "ready", result["problems"])
        self.assertTrue(result["source_matches_checkout"])
        self.assertTrue(result["video_review_complete"])
        self.assertEqual(result["website"]["path"], "assets/manual/reviewed-v1/sample")
        self.assertEqual(result["metadata_path"], "target/evidence/sample/capture-metadata.json")

    def test_changed_authored_tape_blocks_old_capture(self):
        self.tape.write_text(self.tape.read_text() + "Sleep 3s\n")
        self.assert_blocked("captured tape differs")

    def test_review_for_other_capture_metadata_is_not_accepted(self):
        self.review["capture_metadata_sha256"] = "f" * 64
        self.save_review()
        result = self.result()
        self.assertEqual(result["status"], "awaiting-review")
        self.assertFalse(result["pixel_review_complete"])
        self.assertFalse(result["video_review_complete"])

    def test_pair_review_requires_both_exact_hashes(self):
        for field in ["png_sha256", "state_sha256"]:
            with self.subTest(field=field):
                original = self.review["checkpoints"][0][field]
                self.review["checkpoints"][0][field] = "f" * 64
                self.save_review()
                result = self.result()
                self.assertEqual(result["status"], "awaiting-review")
                self.assertFalse(result["pixel_review_complete"])
                self.assertFalse(result["text_style_review_complete"])
                self.review["checkpoints"][0][field] = original

    def test_video_review_requires_exact_video_hash(self):
        self.review["video_review"]["sha256"] = "f" * 64
        self.save_review()
        result = self.result()
        self.assertEqual(result["status"], "checkpoint-reviewed")
        self.assertFalse(result["video_review_complete"])

    def test_missing_pair_and_null_review_hashes_never_become_ready(self):
        for name in ["checkpoint.png", "checkpoint.json"]:
            (self.archive / name).unlink()
            self.metadata["files"].pop(name)
        self.review["checkpoints"][0]["png_sha256"] = None
        self.review["checkpoints"][0]["state_sha256"] = None
        self.save_metadata()
        self.assert_blocked("Required artifacts lack hashes")
        self.assertFalse(self.result()["pixel_review_complete"])

    def test_missing_video_file_blocks_even_with_matching_review(self):
        (self.archive / "sample.mp4").unlink()
        self.assert_blocked("Archived file hashes differ: sample.mp4")

    def test_unmodified_media_does_not_excuse_altered_diagnostic(self):
        (self.archive / "render.log").write_text("different run\n")
        self.assert_blocked("Archived file hashes differ: render.log")

    def test_missing_source_inventory_entry_blocks(self):
        self.metadata["provenance"]["sources"].pop("crates/sample/src/lib.rs")
        self.save_metadata()
        self.assert_blocked("Compiled source inventory differs")

    def test_modified_current_source_blocks(self):
        (self.root / "crates/sample/src/lib.rs").write_text("pub fn changed() {}\n")
        self.assert_blocked("Compiled source differs")

    def test_missing_helper_snapshot_inventory_blocks(self):
        self.metadata["source_snapshots"] = {}
        self.save_metadata()
        self.assert_blocked("helper snapshot inventory is incomplete")

    def test_snapshot_cannot_be_replaced_and_rehashed_after_capture(self):
        snapshot = self.archive / "source-scripts/fixture.sh"
        snapshot.write_text("#!/bin/sh\nprintf 'different fixture\\n'\n")
        self.metadata["files"]["source-scripts/fixture.sh"]["sha256"] = evidence.digest(snapshot)
        self.save_metadata()
        self.assert_blocked("Original helper snapshot hash differs")

    def test_passed_label_does_not_override_renderer_failure(self):
        self.metadata["renderer_passed"] = False
        self.save_metadata()
        self.assert_blocked("Capture/automated verification failed")

    def test_rehashed_failed_repository_assertions_still_block(self):
        path = self.archive / "success-assertions.json"
        write_json(path, {"scenario": "sample", "status": "passed",
                          "assertions": [{"name": "parent", "passed": False}]})
        self.metadata["files"][path.name]["sha256"] = evidence.digest(path)
        self.save_metadata()
        self.assert_blocked("Repository outcome assertions failed")

    def test_legacy_review_schema_is_normalized_without_importing_indexes(self):
        legacy = copy.deepcopy(self.review)
        legacy["id"] = legacy.pop("scenario_id")
        legacy["video_review"]["video_sha256"] = legacy["video_review"].pop("sha256")
        write_json(self.review_path, {"scenarios": [legacy]})
        write_json(self.reviews / "candidate-index.json", {
            "coverage_manifest_sha256": "a" * 64, "scenarios": [legacy],
        })
        self.assertEqual(len(evidence.read_reviews([self.reviews])), 1)
        self.assertEqual(self.result([self.reviews])["status"], "ready")

    def test_positive_review_cannot_erase_rerecording_finding_in_either_order(self):
        negative = copy.deepcopy(self.review)
        negative["checkpoints"][0]["pixel_review"]["status"] = "needs-rerecording"
        negative["video_review"]["status"] = "needs-rerecording"
        negative_path = self.reviews / "negative.json"
        write_json(negative_path, {"reviews": [negative]})
        for paths in [[negative_path, self.review_path], [self.review_path, negative_path]]:
            with self.subTest(paths=[path.name for path in paths]):
                result = self.result(paths)
                self.assertEqual(result["status"], "blocked")
                self.assertEqual(result["video_review"]["status"], "needs-rerecording")
                self.assertEqual(result["checkpoints"][0]["pixel_review"]["status"], "needs-rerecording")

    def test_older_ready_capture_survives_newer_unreviewed_or_invalid_capture(self):
        newer = self.archive.with_name("newer")
        shutil.copytree(self.archive, newer)
        metadata = copy.deepcopy(self.metadata)
        metadata["started_at"] = "2026-09-21T04:00:00+00:00"
        write_json(newer / "capture-metadata.json", metadata)
        for invalid in [False, True]:
            with self.subTest(newer_invalid=invalid):
                if invalid:
                    (newer / "render.log").write_text("corrupted diagnostic\n")
                result = evidence.report(
                    self.root, [self.archive.parent], [self.review_path], "reviewed-v1"
                )["scenarios"][0]
                self.assertEqual(result["status"], "ready", result["problems"])
                self.assertEqual(result["metadata_path"], "target/evidence/sample/capture-metadata.json")

    def test_require_ready_preserves_existing_final_index_when_reviews_incomplete(self):
        self.review["video_review"]["status"] = "pending"
        self.save_review()
        output = self.root / "final-index.json"
        output.write_text("previous reviewed index\n")
        result = subprocess.run([
            sys.executable, str(SCRIPT), "--root", str(self.root),
            "--artifacts", str(self.archive), "--reviews", str(self.review_path),
            "--website-version", "reviewed-v1", "--require-ready", "--output", str(output),
        ], capture_output=True, text=True, check=False)
        self.assertEqual(result.returncode, 1, result.stderr)
        self.assertIn("Final index not written", result.stderr)
        self.assertEqual(output.read_text(), "previous reviewed index\n")


if __name__ == "__main__":
    unittest.main()
