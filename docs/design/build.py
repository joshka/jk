"""Build the design atlas. No jj command runs; baseline PNGs stay outside this repo."""
import argparse
import base64
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--output', type=Path, default=ROOT / 'atlas.html')
parser.add_argument('--baselines', type=Path, help='Optional jk-screenshots/assets directory')
args = parser.parse_args()
if args.baselines and args.output.resolve().is_relative_to(ROOT.parents[1]):
    parser.error('Write embedded baseline images outside the main repository')
inventory = json.loads((ROOT / 'command-inventory.json').read_text())
fragment = (ROOT / 'atlas-shell.html').read_text()
fragment = fragment.replace('/* INVENTORY */', 'const inventory = ' + json.dumps(inventory) + ';')
baselines = {}
if args.baselines:
    for screen, filename in [('log', 'jk-log-v3.png'), ('diff', 'jk-diff-v3.png'),
                             ('help-log', 'jk-log-help-v3.png')]:
        source = args.baselines / filename
        baselines[screen] = {'name': filename, 'data': 'data:image/png;base64,' +
                            base64.b64encode(source.read_bytes()).decode()}
fragment = fragment.replace('/* BASELINES */', 'const baselines = ' + json.dumps(baselines) + ';')
fragment = fragment.replace('/* SCREENS */', (ROOT / 'screens.js').read_text())
fragment = fragment.replace('/* RUNTIME */', (ROOT / 'atlas-runtime.js').read_text())
args.output.write_text(fragment)
print(f'Built {args.output}: {len(fragment.encode()):,} bytes')
