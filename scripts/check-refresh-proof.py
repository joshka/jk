#!/usr/bin/env python3
"""Assert refresh outcomes from the same terminal checkpoints used for visual inspection."""
import json
from pathlib import Path
import sys

run = Path(sys.argv[1])
def screen(stage):
    return json.loads((run / f'cancellable-refresh-{stage}.json').read_text())['viewport_text']

assert 'Refreshing…' in screen('00-loading')
navigation = screen('01-navigation')
assert 'Refreshing…' in navigation
assert any(line.startswith('·') and 'Base change' in line for line in navigation.splitlines())
complete = screen('02-complete')
assert 'Refreshing…' not in complete, 'Replacement refresh did not finish'
assert 'Refresh demo' in complete and 'Base change' in complete
failure = screen('03-failure')
assert 'Refresh failed:' in failure
assert 'Refresh demo' in failure, 'Failed refresh discarded the last usable graph'
assert complete.split('\n\n', 1)[0] == failure.split('\n\n', 1)[0], (
    'Failed refresh changed the previously completed graph or its selection'
)
assert 'fixture refresh failed' in screen('04-failure-details'), 'Full failure reason was not retained'
print('Refresh proof: loading, responsive selection, replacement completion, and retained failure state.')
