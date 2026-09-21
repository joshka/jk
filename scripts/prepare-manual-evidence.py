#!/usr/bin/env python3
"""Build a hash-bound evidence/review candidate index; never mutate coverage status."""
import argparse
from collections import Counter
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import re
import sys


def digest(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def object_digest(value):
    return hashlib.sha256(json.dumps(value, sort_keys=True, separators=(',', ':')).encode()).hexdigest()


def pending():
    return {'status': 'pending', 'reviewer': None, 'reviewed_at': None, 'summary': None, 'findings': []}


def valid_hash(value):
    return isinstance(value, str) and re.fullmatch(r'[0-9a-f]{64}', value) is not None


def accepted(review):
    return review.get('status') == 'reviewed' and all(
        isinstance(review.get(field), str) and review[field].strip()
        for field in ['reviewer', 'reviewed_at', 'summary']
    )


def reference(path, root):
    """Keep repository-local review references portable in the durable index."""
    try:
        return path.resolve().relative_to(root.resolve()).as_posix()
    except ValueError:
        return path.name


def evidence_path(directory, name):
    path = (directory / name).resolve()
    if not path.is_relative_to(directory.resolve()):
        raise ValueError(f'Artifact path escapes its evidence directory: {name}')
    return path


def read_reviews(paths):
    """Accept review ledgers, including the first audit's scenarios/id spelling."""
    ledger = []
    for review_path in paths:
        if not review_path.exists():
            raise ValueError(f'Review input does not exist: {review_path}')
        files = sorted(review_path.glob('*.json')) if review_path.is_dir() else [review_path]
        for path in files:
            value = json.loads(path.read_text())
            # A generated index is a report, never another independent review.
            if 'coverage_manifest_sha256' in value:
                continue
            for original in value.get('reviews', value.get('scenarios', [])):
                entry = dict(original)
                entry['scenario_id'] = entry.get('scenario_id', entry.get('id'))
                video = dict(entry.get('video_review', {}))
                video['sha256'] = video.get('sha256', video.get('video_sha256'))
                entry['video_review'] = video
                ledger.append((path, entry))
    return ledger


def check_capture(root, directory, metadata):
    """Recheck the archive, source set and outcome reports before using reviews."""
    problems = []
    sources = metadata.get('provenance', {}).get('sources', {})
    if any(not valid_hash(sha) for sha in sources.values()):
        problems.append('Source inventory contains an invalid SHA-256.')
    compiled = {name: sha for name, sha in sources.items()
                if name.startswith('crates/') or name in {'Cargo.toml', 'Cargo.lock'}}
    expected_sources = {str(path.relative_to(root))
                        for pattern in ['Cargo.toml', 'Cargo.lock', 'crates/**/*.rs',
                                        'crates/**/Cargo.toml']
                        for path in root.glob(pattern)}
    if not compiled or set(compiled) != expected_sources:
        problems.append('Compiled source inventory differs from checkout.')
    drift = [name for name, sha in compiled.items()
             if not evidence_path(root, name).is_file() or digest(root / name) != sha]
    if drift:
        problems.append('Compiled source differs from checkout: ' + ', '.join(drift))
    files = metadata.get('files', {})
    if any(not valid_hash(spec.get('sha256')) for spec in files.values()):
        problems.append('Artifact inventory contains an invalid SHA-256.')
    mismatched = [name for name, spec in files.items()
                  if not evidence_path(directory, name).is_file()
                  or digest(directory / name) != spec.get('sha256')]
    if mismatched:
        problems.append('Archived file hashes differ: ' + ', '.join(mismatched))
    scenario = metadata['scenario']
    required = {'source.tape', 'capture.tape', 'fixture.json', scenario['video']}
    required.update(f'{checkpoint}.{extension}' for checkpoint in scenario['checkpoints']
                    for extension in ['png', 'json'])
    if not required.issubset(files):
        problems.append('Required artifacts lack hashes: ' + ', '.join(sorted(required - files.keys())))
    if files.get('source.tape', {}).get('sha256') != metadata.get('source_tape_sha256'):
        problems.append('Preserved source tape hash differs from capture metadata.')
    if (metadata.get('status') != 'passed' or metadata.get('renderer_passed') is not True
            or metadata.get('problems')):
        problems.append('Capture/automated verification failed.')
        problems.extend(metadata.get('problems', []))
    for tool in ['jk', 'jj', 'betamax']:
        info = metadata.get('provenance', {}).get(tool, {})
        if not valid_hash(info.get('sha256')) or not info.get('version'):
            problems.append(f'Exact {tool} binary provenance is absent.')
    snapshots = metadata.get('source_snapshots', {})
    expected_helpers = {name for name in sources if name.startswith('scripts/')}
    if (metadata.get('source_snapshot_status') != 'preserved' or not snapshots
            or set(snapshots) != expected_helpers):
        problems.append('Original helper snapshot inventory is incomplete.')
    for name, snapshot in snapshots.items():
        if files.get(snapshot, {}).get('sha256') != sources.get(name):
            problems.append(f'Original helper snapshot hash differs: {snapshot}')
    if metadata.get('source_consistency', {}).get('passed') is not True:
        problems.append('Helper consistency during capture is unverified or failed.')
    assertions = [name for name in files if name.endswith('-assertions.json')]
    if not assertions:
        problems.append('Repository outcome assertions are absent.')
    for name in assertions:
        path = evidence_path(directory, name)
        if not path.is_file():
            continue  # The archive hash check reports the missing file.
        value = json.loads(path.read_text())
        checks = value.get('assertions', [])
        if (value.get('scenario') != scenario['id'] or value.get('status') != 'passed'
                or not checks or any(check.get('passed') is not True for check in checks)):
            problems.append(f'Repository outcome assertions failed: {name}')
    return compiled, not drift and set(compiled) == expected_sources, problems


def assess_capture(root, scenario, chosen, ledger, website_version):
    """Bind one capture to its exact source, artifacts and independent reviews."""
    scenario_id = scenario['id']
    tape_hash = digest(root / scenario['tape'])
    item = {'id': scenario_id, 'title': scenario['title'], 'status': 'missing',
            'source_tape': scenario['tape'], 'source_tape_sha256': tape_hash,
            'website': {'path': f'assets/manual/{website_version}/{scenario_id}' if website_version else None},
            'problems': [], 'checkpoints': []}
    if chosen is None:
        item['problems'].append('No downloaded evidence metadata for this scenario.')
        return item
    path, metadata = chosen
    directory = path.parent
    provenance = metadata.get('provenance', {})
    github = provenance.get('github', {})
    compiled_sources, source_matches, problems = check_capture(root, directory, metadata)
    item['problems'].extend(problems)
    video = metadata['scenario']['video']
    item.update(status='awaiting-review', metadata_path=reference(path, root), metadata_sha256=digest(path),
                evidence_directory=reference(directory, root), evidence_status=metadata.get('status'),
                source_commit=github.get('GITHUB_SHA'), pr_head_commit=github.get('JK_PR_HEAD_SHA'),
                source_manifest_sha256=object_digest(compiled_sources),
                source_matches_checkout=source_matches,
                binary=provenance.get('jk'), jj=provenance.get('jj'), betamax=provenance.get('betamax'),
                ci_run={'id': github.get('GITHUB_RUN_ID'), 'attempt': github.get('GITHUB_RUN_ATTEMPT'),
                        'url': f"https://github.com/{github.get('GITHUB_REPOSITORY')}/actions/runs/{github.get('GITHUB_RUN_ID')}"},
                assertions=[{'path': name, **spec} for name, spec in metadata.get('files', {}).items()
                            if name.endswith('-assertions.json')],
                video={'path': video, **metadata.get('files', {}).get(video, {})},
                video_review=pending(),
                source_snapshot_status=metadata.get('source_snapshot_status'))
    if metadata.get('source_tape_sha256') != tape_hash:
        item['problems'].append('The captured tape differs from the current authored tape.')
    if metadata['scenario'].get('tape') != scenario['tape']:
        item['problems'].append('Captured tape path differs from the current manifest.')
    if video != Path(scenario['recording']['video']).name:
        item['problems'].append('Captured video differs from the current manifest.')
    if not item['source_commit'] or not item['pr_head_commit']:
        item['problems'].append('Exact CI build/PR commit provenance is absent.')
    expected_checkpoints = [checkpoint['id'] for checkpoint in scenario['recording']['checkpoints']]
    if metadata['scenario']['checkpoints'] != expected_checkpoints:
        item['problems'].append('Captured checkpoints differ from the current manifest.')
    matching_reviews = [(review_path, review) for review_path, review in ledger
                        if review.get('scenario_id') == scenario_id
                        and review.get('source_tape_sha256') == tape_hash
                        and review.get('capture_metadata_sha256') == item['metadata_sha256']]
    for checkpoint in expected_checkpoints:
        png, state = f'{checkpoint}.png', f'{checkpoint}.json'
        png_path, state_path = evidence_path(directory, png), evidence_path(directory, state)
        png_hash = digest(png_path) if png_path.is_file() else None
        state_hash = digest(state_path) if state_path.is_file() else None
        row = {'id': checkpoint, 'png': png, 'state': state, 'png_sha256': png_hash,
               'state_sha256': state_hash, 'pixel_review': pending(), 'text_style_review': pending()}
        for review_path, review in matching_reviews:
            for candidate in review.get('checkpoints', []):
                if candidate.get('id') != checkpoint:
                    continue
                if (not valid_hash(png_hash) or not valid_hash(state_hash)
                        or candidate.get('png_sha256') != png_hash
                        or candidate.get('state_sha256') != state_hash):
                    continue
                for kind in ['pixel_review', 'text_style_review']:
                    if (candidate.get(kind, {}).get('status') in {'reviewed', 'needs-rerecording'}
                            and row[kind].get('status') != 'needs-rerecording'):
                        row[kind] = candidate[kind]
                        row[kind + '_source'] = reference(review_path, root)
        item['checkpoints'].append(row)
    for review_path, review in matching_reviews:
        video_review = review.get('video_review', {})
        if (valid_hash(item['video'].get('sha256'))
                and video_review.get('sha256') == item['video']['sha256']
                and video_review.get('status') in {'reviewed', 'needs-rerecording'}
                and item['video_review'].get('status') != 'needs-rerecording'):
            item['video_review'] = video_review
            item['video_review_source'] = reference(review_path, root)
    item['pixel_review_complete'] = bool(item['checkpoints']) and all(accepted(c['pixel_review']) for c in item['checkpoints'])
    item['text_style_review_complete'] = bool(item['checkpoints']) and all(accepted(c['text_style_review']) for c in item['checkpoints'])
    item['video_review_complete'] = accepted(item['video_review'])
    evidence_issues = any(c[k].get('status') == 'needs-rerecording'
                          for c in item['checkpoints'] for k in ['pixel_review', 'text_style_review'])
    if evidence_issues or item['video_review'].get('status') == 'needs-rerecording':
        item['problems'].append('Reviewer identified an evidence defect requiring a new recording.')
    if item['problems']:
        item['status'] = 'blocked'
    elif item['pixel_review_complete'] and item['text_style_review_complete']:
        item['status'] = 'checkpoint-reviewed'
        if item['video_review_complete']:
            item['status'] = 'ready'
    return item


def report(root, artifacts, reviews, website_version):
    coverage = json.loads((root / 'docs/manual-coverage.json').read_text())
    if not coverage.get('scenarios'):
        raise ValueError('Coverage manifest has no scenarios.')
    available = {}
    for artifact_root in artifacts:
        if not artifact_root.exists():
            raise ValueError(f'Artifact input does not exist: {artifact_root}')
        captures = [(path, json.loads(path.read_text()))
                    for path in artifact_root.rglob('capture-metadata.json')]
        for path, value in sorted(captures, key=lambda capture: capture[1].get('started_at', '')):
            scenario_id = value.get('scenario', {}).get('id')
            if scenario_id:
                available.setdefault(scenario_id, []).append((path, value))
    ledger = read_reviews(reviews)
    entries = []
    for scenario in coverage['scenarios']:
        candidates = [assess_capture(root, scenario, candidate, ledger, website_version)
                      for candidate in available.get(scenario['id'], [])]
        # Reuse accepted bytes when the source and tape still match. New CI output
        # does not invalidate an existing review, but it cannot borrow that review.
        ready = next((item for item in reversed(candidates) if item['status'] == 'ready'), None)
        valid = next((item for item in reversed(candidates)
                      if item['status'] in {'checkpoint-reviewed', 'awaiting-review'}), None)
        fallback = candidates[-1] if candidates else assess_capture(
            root, scenario, None, ledger, website_version)
        entries.append(ready or valid or fallback)
    return {'schema_version': 1, 'generated_at': datetime.now(timezone.utc).isoformat(),
            'coverage_manifest_sha256': digest(root / 'docs/manual-coverage.json'),
            'scope': 'Candidate evidence index. Ready means exact recorded bytes and supplied reviews match; this tool never upgrades coverage recording status.',
            'summary': dict(Counter(row['status'] for row in entries)), 'scenarios': entries}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--root', required=True, type=Path)
    parser.add_argument('--artifacts', required=True, type=Path, action='append')
    parser.add_argument('--reviews', type=Path, action='append', default=[])
    parser.add_argument('--website-version')
    parser.add_argument('--output', required=True, type=Path)
    parser.add_argument('--require-ready', action='store_true',
                        help='Write the final index only when every scenario is ready.')
    args = parser.parse_args()
    if args.require_ready and not args.website_version:
        parser.error('--require-ready needs --website-version for permanent media paths')
    if args.website_version and not re.fullmatch(r'[A-Za-z0-9][A-Za-z0-9._-]*', args.website_version):
        parser.error('--website-version must be one path component')
    result = report(args.root, args.artifacts, args.reviews, args.website_version)
    print(json.dumps(result['summary'], sort_keys=True))
    for item in result['scenarios']:
        if item['problems']:
            print(item['id']+': '+item['problems'][0])
    if args.require_ready and any(item['status'] != 'ready' for item in result['scenarios']):
        print('Final index not written: every scenario must be ready.', file=sys.stderr)
        return 1
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(result, indent=2)+'\n')
    return 0


if __name__ == '__main__':
    sys.exit(main())
