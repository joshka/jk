"""Capture visible installed jj commands using read-only help invocations."""
import json
import re
import subprocess
from pathlib import Path


def help_text(words):
    return subprocess.check_output(['jj', *words, '-h'], text=True)


def visit(words):
    text = help_text(words)
    section = text.split('Commands:\n', 1)
    if len(section) == 2:
        rows = section[1].split('\n\n', 1)[0].splitlines()
        for row in rows:
            match = re.match(r'  ([a-z][a-z-]*)\s+(.*)', row)
            if match and match[1] != 'help':
                visit([*words, match[1]])
    else:
        entries.append({'command': ' '.join(words), 'help': text})


entries = []
visit([])
entries.append({'command': 'help', 'help': help_text(['help'])})
Path(__file__).with_name('command-inventory.json').write_text(json.dumps({
    'version': subprocess.check_output(['jj', '--version'], text=True).strip(),
    'scope': 'Visible command leaves; aliases share their canonical screen; hidden commands excluded.',
    'commands': entries,
}, indent=2) + '\n')
print('\n'.join(entry['command'] for entry in entries))
