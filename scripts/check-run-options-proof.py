#!/usr/bin/env python3
"""Check run-option transitions from captured terminal text, alongside the tape's graph assertion."""
import json
from pathlib import Path
import re
import sys

run = Path(sys.argv[1])
initial_path = next(run.glob('run-options*-initial-preview.json'))
stem = initial_path.name.removesuffix('-initial-preview.json')

def viewport(stage):
    return json.loads((run / f'{stem}-{stage}.json').read_text())['viewport_text']

initial = viewport('initial-preview')
cancelled = viewport('cancelled-preview')
applied = viewport('applied-preview')
assert '--ignore-working-copy' not in initial
assert '--ignore-working-copy' not in cancelled, 'Cancelling must discard draft options'
assert '--ignore-working-copy' in applied, 'Applying must update the exact command'
assert 'Run options' not in cancelled, 'Cancelling must close the options drawer'
assert 'Run options' not in applied, 'Applying must return to the command preview'
for role in ('Source', 'Destination'):
    before = re.search(rf'{role}:\s*([0-9a-f]+)', initial)
    assert before and len(before.group(1)) == 40, f'{role} must contain the full commit ID'
    for stage, text in (('cancelled', cancelled), ('applied', applied)):
        after = re.search(rf'{role}:\s*([0-9a-f]+)', text)
        assert after and before.group(1) == after.group(1), f'{role} changed when {stage}'

def command_text(text):
    command = re.search(r'\n\s*Command\n(.*?)\n\s*enter\s+run', text, re.DOTALL)
    assert command, 'Exact command and preview controls must remain visible'
    # Ratatui wraps long fixture paths at the terminal edge; ignore only visual whitespace.
    return re.sub(r'\s+', '', command.group(1))

assert command_text(cancelled) == command_text(initial), 'Cancelling changed the exact command'
assert command_text(applied).replace('--ignore-working-copy', '', 1) == command_text(initial), (
    'Applying changed the repository or command operands beyond the selected working-copy policy'
)
assert 'snapshot/update' in viewport('drawer')
print('Run options: cancelled draft discarded; applied command preserves exact operands.')
