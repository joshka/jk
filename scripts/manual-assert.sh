#!/usr/bin/env bash
# Source after leaving jk. Preserve machine-readable evidence for each repository outcome.
# shellcheck disable=SC2154 # Variables come from sourced manual-fixture.sh.
set -euo pipefail
manual_assert_id=${1:?usage: source manual-assert.sh SCENARIO_OR_STAGE}
manual_scenario_id=${JK_MANUAL_SCENARIO:-${2:-$manual_assert_id}}
python3 - "$manual_assert_id" "$repo" "$manual_artifacts" "$manual_real_jj" \
    "$manual_initial_commit" "$manual_initial_change" "$manual_initial_graph" \
    "${fixture_root:-}" "${manual_remote_graph:-}" "${remote_repo:-}" "$manual_scenario_id" "$manual_initial_operation" <<'PY'
import json
from pathlib import Path
import subprocess
import sys

scenario, repository, artifacts, binary, initial_commit, initial_change, initial_graph, fixture_root, remote_graph, remote_repo, scenario_id, initial_operation = sys.argv[1:]
repo = Path(repository)
checks = []

def jj(*args, at=repo):
    return subprocess.check_output([binary, '-R', str(at), *args], text=True, stderr=subprocess.PIPE).strip()

def log(revision, template):
    return jj('log', '--ignore-working-copy', '--no-graph', '-r', revision, '-T', template)

def check(name, actual, expected=True):
    checks.append({'name': name, 'passed': actual == expected, 'actual': actual, 'expected': expected})

def unchanged():
    check('working-copy commit unchanged', log('@', 'commit_id'), initial_commit)
    graph = '\n'.join(sorted(log('all()', 'commit_id ++ "\n"').splitlines()))
    check('visible repository graph unchanged', graph, initial_graph)

def contents(path):
    return (repo / path).read_text() if (repo / path).is_file() else None

def revisions(description):
    return log(f'description(substring:"{description}")', 'change_id ++ "\n"').splitlines()

try:
    if scenario.startswith('manual-dialog-') or scenario in {'inspection', 'manual-run-options', 'manual-command-mode', 'manual-cancel-refresh', 'startup-container', 'change-edit-failure', 'abandon-probe-failure', 'manual-invalid-operands'} or scenario.endswith(('-cancelled', '-undone')):
        unchanged()
    elif scenario == 'change-lifecycle-new':
        check('new working copy is empty', log('@', 'empty'), 'true')
        check('new parent has multiline description', log('@-', 'description'), 'Tune forecast cache lifetime\n\nKeep cached forecasts fresh.')
        check('new child retains parent files', (repo / 'src/cache.js').is_file())
    elif scenario == 'change-lifecycle-edited':
        check('edit and redo select described revision', log('@', 'description'), 'Tune forecast cache lifetime\n\nKeep cached forecasts fresh.')
        check('edit retains cache addition', jj('diff', '--summary'), 'A src/cache.js')
    elif scenario == 'new-parents-merged':
        check('new merge is empty', log('@', 'empty'), 'true')
        check('parent selection order preserved', log('@', 'parents.map(|p| p.description().first_line()).join("\n")'), 'Experiment with forecast caching\nDocument cache policy')
        check('both parent trees retained', (repo / 'src/cache.js').is_file() and (repo / 'cache-policy.md').is_file())
    elif scenario == 'abandon-empty-replaced':
        check('empty abandoned working copy replaced', log('@', 'change_id') != initial_change)
        check('replacement working copy empty', log('@', 'empty'), 'true')
        check('abandoned empty description absent', revisions('Empty checkpoint'), [])
    elif scenario == 'abandon-review-abandoned':
        check('selected populated revision abandoned', revisions('Tune forecast caching'), [])
        check('descendants remain', len(revisions('Add cache metrics')) == 1 and len(revisions('Document cache metrics')) == 1)
        check('abandoned cache addition removed', not (repo / 'src/cache.js').exists())
        check('abandoned config modification removed', '"cacheTtlSeconds": 60' in contents('config.json'))
        check('abandoned README deletion removed', (repo / 'README.md').is_file())
    elif scenario == 'command-alternatives-committed':
        check('commit opens an empty working copy', log('@', 'empty'), 'true')
        check('commit names finalized parent', log('@-', 'description'), 'Finalize forecast caching')
        check('commit retains cache file and policy', (repo / 'src/cache.js').is_file() and '120' in contents('config.json'))
    elif scenario == 'command-alternatives-duplicated':
        duplicate_ids = revisions('Experiment with forecast caching')
        check('duplicate creates another change identity', len(set(duplicate_ids)), 2)
        check('duplicate tree equals original', jj('diff', '--from', duplicate_ids[0], '--to', duplicate_ids[1], '--summary'), '')
    elif scenario == 'command-alternatives-reverted':
        check('revert preserves current working copy', log('@', 'change_id'), initial_change)
        check('revert creates a reverse change', bool(revisions('Revert')))
        check('revert removes cache in new revision', 'D src/cache.js' in jj('diff', '-r', 'description(substring:"Revert")', '--summary'))
    elif scenario == 'command-filesets-split':
        check('split selected only config changes', jj('diff', '-r', 'description(substring:"Separate cache policy")', '--summary'), 'M config.json')
        check('split retains remaining file addition', jj('diff', '-r', 'description(substring:"Experiment with forecast caching")', '--summary'), 'A src/cache.js')
        check('split preserves combined working tree', (repo / 'src/cache.js').is_file() and '120' in contents('config.json'))
    elif scenario == 'command-filesets-restored':
        check('fileset restore keeps unselected cache addition', jj('diff', '--summary'), 'A src/cache.js')
        check('fileset restore recovers old config only', '60' in contents('config.json') and (repo / 'src/cache.js').is_file())
        check('undo removes split revision', revisions('Separate cache policy'), [])
    elif scenario == 'command-alternatives-absorbed':
        check('absorb leaves new file in current change', jj('diff', '--summary'), 'A src/cache.js')
        check('absorb moves config edit into parent', '120' in jj('file', 'show', '-r', '@-', 'config.json'))
    elif scenario == 'manual-run-options-ignore':
        check('ignore leaves source file unchanged', contents('source.txt'), 'source contents\n')
        check('ignore does not write destination file', not (repo / 'destination.txt').exists())
        check('ignore still rebases the recorded source', log('description(substring:"Rebase source")', 'parents.map(|p| p.description().first_line()).join(",")'), 'Rebase destination')
        check('ignore still records the rebase', any('rebase ' in line for line in jj('--ignore-working-copy', 'op', 'log', '--no-graph', '-T', 'description ++ "\\n"').splitlines()))
    elif scenario == 'manual-run-options-historical':
        check('historical rebase leaves source file unchanged', contents('source.txt'), 'source contents\n')
        check('historical rebase does not update working-copy destination file', not (repo / 'destination.txt').exists())
        check('later bookmark survives normal operation integration', bool(jj('--ignore-working-copy', 'bookmark', 'list', 'exact:after-baseline')))
        operation_rows = jj('--ignore-working-copy', 'op', 'log', '--no-graph', '-T', 'id ++ "\\t" ++ description ++ "\\t" ++ parents.map(|p| p.id()).join(",") ++ "\\n"').splitlines()
        historical = [row for row in operation_rows if '\trebase ' in row and initial_operation in row.split('\t')[-1]]
        check('rebase operation is a child of selected historical operation', bool(historical))
    elif scenario == 'manual-run-options-immutable-success':
        check('explicit immutable override rebases source', log('@', 'parents.map(|p| p.description().first_line()).join(",")'), 'Rebase destination')
        check('immutable override retains source content', contents('source.txt'), 'source contents\n')
    elif scenario.startswith('manual-rebase-'):
        role = scenario.removeprefix('manual-rebase-')
        check('source content retained', contents('source.txt'), 'source contents\n')
        check('source descendants remain', len(revisions('Verify source')), 1)
        check('destination descendants remain', len(revisions('Verify destination')), 1)
        if role.endswith('before'):
            expected = 'Rebase source' if role.startswith('revision') else 'Verify source'
            check('insert-before destination follows the moved source tip', log('description(substring:"Rebase destination")', 'parents.map(|p| p.description().first_line()).join(",")'), expected)
            check('insert-before source parent is base', log('description(substring:"Rebase source")', 'parents.map(|p| p.description().first_line()).join(",")'), 'Rebase base')
        else:
            check('source parent is selected destination', log('description(substring:"Rebase source")', 'parents.map(|p| p.description().first_line()).join(",")'), 'Rebase destination')
        if role.startswith('revision'):
            check('revision-only reconnects source descendant to old parent', log('description(substring:"Verify source")', 'parents.map(|p| p.description().first_line()).join(",")'), 'Rebase base')
        else:
            check('source or branch retains source descendant', log('description(substring:"Verify source")', 'parents.map(|p| p.description().first_line()).join(",")'), 'Rebase source')
        if role.endswith('after'):
            expected = 'Rebase source' if role.startswith('revision') else 'Verify source'
            check('insert-after relocates old destination descendant', log('description(substring:"Verify destination")', 'parents.map(|p| p.description().first_line()).join(",")'), expected)
    elif scenario in {'manual-squash-success', 'manual-squash-multiple-success'}:
        check('source patch moved into destination', jj('file', 'show', '-r', 'description(substring:"Squash destination")', 'source.txt'), 'source contents')
        check('source revision removed', revisions('Squash source'), [])
        if scenario == 'manual-squash-multiple-success':
            check('second source patch moved into destination', jj('file', 'show', '-r', 'description(substring:"Squash destination")', 'second-source.txt'), 'second source contents')
            check('second source revision removed', revisions('Squash second source'), [])
    elif scenario == 'manual-restore-success':
        check('working file restored from selected revision', contents('file.txt'), 'base content\n')
        check('restored working copy has no diff', jj('diff', '--summary'), '')
    elif scenario == 'refs-bookmarks':
        check('literal wildcard bookmark recreated at current change', log('bookmarks(exact:"review-*")', 'change_id'), log('@', 'change_id'))
        check('similarly named bookmark untouched', log('review-keep', 'change_id'), log('@-', 'change_id'))
        check('other bookmarks retained', all(jj('bookmark', 'list', f'exact:{name}') for name in ['demo', 'stable-a', 'stable-b', 'stable-c']))
        operations = jj('op', 'log', '--no-graph', '-T', 'description ++ "\n"')
        named = [line for line in operations.splitlines() if 'review-*' in line]
        check('literal bookmark created twice', sum('create bookmark' in line for line in named), 2)
        check('literal bookmark moved exactly once', sum('point bookmark' in line for line in named), 1)
        check('literal bookmark deleted exactly once', sum('delete bookmark' in line for line in named), 1)
    elif scenario.startswith('command-bookmarks-'):
        stage = scenario.removeprefix('command-bookmarks-')
        remote_id = log('incoming@fixture', 'change_id')
        check('remote incoming bookmark remains', bool(remote_id))
        tracking_rows = jj('bookmark', 'list', '--all-remotes', 'exact:incoming', '-T', 'if(remote, remote ++ ":" ++ tracked ++ "\\n")')
        check('tracking state matches requested operation', 'fixture:true' in tracking_rows, stage == 'tracked')
        if stage in {'tracked', 'untracked'}:
            check('local bookmark target preserved', log('bookmarks(exact:"incoming")', 'change_id'), remote_id)
        else:
            check('old local bookmark absent', jj('bookmark', 'list', 'exact:incoming'), '')
        if stage == 'renamed':
            check('renamed local bookmark target preserved', log('local-review', 'change_id'), remote_id)
        if stage == 'forgotten':
            check('forgotten local bookmark absent', jj('bookmark', 'list', 'exact:local-review'), '')
        actual_remote_graph = '\n'.join(sorted(jj('log', '--no-graph', '-r', 'all()', '-T', 'commit_id ++ "\\n"', at=remote_repo).splitlines()))
        check('local tracking operations leave remote repository unchanged', actual_remote_graph, remote_graph)
    elif scenario == 'refs-remotes':
        check('fetch imports incoming remote bookmark', bool(jj('bookmark', 'list', '--all-remotes', 'exact:incoming')))
        actual_remote_graph = '\n'.join(sorted(jj('log', '--no-graph', '-r', 'all()', '-T', 'commit_id ++ "\n"', at=remote_repo).splitlines()))
        check('push dry-run leaves remote graph unchanged', actual_remote_graph, remote_graph)
        check('push dry-run does not create remote demo', jj('bookmark', 'list', 'exact:demo', at=remote_repo), '')
    elif scenario.startswith('workspaces-'):
        names = jj('workspace', 'list', '-T', 'name ++ "\n"').splitlines()
        check('only original workspace registrations remain', sorted(names), ['default', 'scratch'])
        scratch = Path(fixture_root) / 'workspaces/scratch'
        check('scratch work retained', (scratch / 'KEEP-ME.txt').read_text(), 'workspace files remain after metadata changes\n')
        check('default tracked files retained', contents('README.md'), 'Workspace lifecycle demo files\n')
        if scenario == 'workspaces-lifecycle':
            added = Path(fixture_root) / 'workspaces/manual-added'
            check('forget retains added workspace directory and files', (added / 'README.md').read_text(), 'Workspace lifecycle demo files\n')
            check('rename does not rename filesystem directories', not added.with_name('manual-renamed').exists() and not added.with_name('manual-ready').exists())
        elif scenario == 'workspaces-stale':
            check('update-stale synchronizes scratch commit', jj('log', '--no-graph', '-r', '@', '-T', 'description', at=scratch), 'Refresh scratch description')
            check('update-stale applies the rewritten tree', not (scratch / 'README.md').exists())
            check('updated scratch status succeeds', bool(jj('status', at=scratch)))
    else:
        raise ValueError(f'No repository assertions defined for {scenario}')
except Exception as error:
    checks.append({'name': 'assertion execution', 'passed': False, 'error': str(error), 'stderr': getattr(error, 'stderr', None)})

passed = bool(checks) and all(item['passed'] for item in checks)
output = Path(artifacts) / f'{scenario}-assertions.json'
output.write_text(json.dumps({'scenario': scenario_id, 'stage': scenario, 'status': 'passed' if passed else 'failed', 'repository': repository, 'assertions': checks}, indent=2) + '\n')
if not passed:
    print(output.read_text(), file=sys.stderr)
    sys.exit(1)
print(f'MANUAL_ASSERT_OK {scenario}')
print('JK_MANUAL_ASSERTIONS_PASSED')
print('JK_ASSERTIONS_PASSED')
PY
