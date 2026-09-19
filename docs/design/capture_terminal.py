"""Export the design fixtures and create Betamax tapes for fixed-cell image mockups."""
import argparse
import html
import json
from pathlib import Path
import shlex
import subprocess

ROOT = Path(__file__).resolve().parent
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('output', type=Path, help='Image directory outside the source repository')
parser.add_argument('--theme', choices=['dark', 'light'], default='dark')
parser.add_argument('--run', action='store_true')
args = parser.parse_args()
out = args.output.resolve()
if out.is_relative_to(ROOT.parents[1]):
    parser.error('Keep generated PNGs outside the main repository')
out.mkdir(parents=True, exist_ok=True)
source = "const fs=require('fs'),vm=require('vm');const p=process.argv[1];console.log(vm.runInNewContext('const inventory='+fs.readFileSync(p+'/command-inventory.json','utf8')+';'+fs.readFileSync(p+'/screens.js','utf8')+';JSON.stringify({screens,graph})'));"
fixtures = subprocess.check_output(['node', '-e', source, str(ROOT)], text=True)
(out / 'fixtures.json').write_text(fixtures)
screens = sorted(json.loads(fixtures)['screens'], key=lambda s: s['id'])
command = shlex.join(['python3', str(ROOT / 'terminal_mockups.py'), str(out / 'fixtures.json'),
                      '--all', '--theme', args.theme, '--metadata', str(out / 'dimensions.json')])
tape = [f'Output {out / "final.png"}', 'Set Shell "bash"', 'Set FontSize 20',
        'Set Width 1200', 'Set Height 768', 'Set Padding 0', 'Set Margin 0',
        'Set BorderRadius 0', 'Set CursorBlink false', 'Set KeyboardOverlay Off',
        'Set TypingSpeed 1ms', 'Hide', 'Type ' + json.dumps(command), 'Enter',
        'Wait+Screen "Change actions"', 'Show']
for i, screen in enumerate(screens):
    if i:
        tape += ['Type "n"']
    tape += ['Sleep 100ms', f'Screenshot {out / (screen["id"] + ".png")}']
    if screen['id'] in ('log', 'diff', 'files', 'help-log', 'rebase-preview', 'cmd-split'):
        tape += [f'State {out / (screen["id"] + ".json")}']
(out / 'capture.tape').write_text('\n'.join(tape) + '\n')
# The gallery contains images only; its navigation is not part of the proposed terminal UI.
options = ''.join(f'<option value="{html.escape(s["id"])}">{html.escape(s["group"] + " / " + s["title"])}</option>' for s in screens)
page = '''<!doctype html><html lang="en"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>jk terminal cell mockups</title><style>body{margin:24px;background:#13171d;color:#d0d5df;font:16px system-ui}nav{display:flex;gap:12px;margin-bottom:16px;align-items:center;flex-wrap:wrap}select,button{font:inherit;padding:6px;background:#242e39;color:inherit;border:1px solid #52606d}img{display:block;max-width:100%;height:auto}p{color:#a7b1bd}select{max-width:100%}</style><nav><label>Screen <select id="screen">''' + options + '''</select></label><button id="prev">Previous</button><button id="next">Next</button><span>Actual ANSI cells · 92 × 38 · one font size · no borders</span></nav><img id="image" alt="Terminal mockup"><p id="caption"></p><script>const select=document.getElementById('screen'),image=document.getElementById('image');function render(){image.src=select.value+'.png';image.alt=select.selectedOptions[0].textContent;document.getElementById('caption').textContent=image.alt;}select.value='files';select.onchange=render;document.getElementById('prev').onclick=()=>{select.selectedIndex=(select.selectedIndex+select.length-1)%select.length;render();};document.getElementById('next').onclick=()=>{select.selectedIndex=(select.selectedIndex+1)%select.length;render();};render();</script></html>'''
(out / 'index.html').write_text(page)
print(f'{len(screens)} captures planned: {out}')
if args.run:
    subprocess.run(['betamax', 'run', '--quiet', str(out / 'capture.tape')], check=True)
    missing = [s['id'] for s in screens if not (out / (s['id'] + '.png')).exists()]
    if missing:
        raise RuntimeError(f'Missing PNGs: {missing}')
    print(f'Captured all {len(screens)} terminal screens')
