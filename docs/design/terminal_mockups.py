"""Draw mockups with ANSI terminal cells for Betamax PNG capture; never run jj commands."""
import argparse
import json
import os
from pathlib import Path
import re
import sys
import termios
import textwrap
import tty
import unicodedata

DARK = dict(bg='#10151d', fg='#d0d5df', surface='#202833', selected='#2c3d3d',
            muted='#9aa5b3', accent='#83bdb2', green='#91bb87', red='#d68c90',
            purple='#c395d5', blue='#91b2dd', yellow='#c8af75', button='#354352', danger='#70363e')
LIGHT = dict(bg='#faf9f6', fg='#282d34', surface='#eaece8', selected='#dce7e3',
             muted='#5e6975', accent='#236b63', green='#326939', red='#9d353c',
             purple='#7e468d', blue='#365f99', yellow='#806021', button='#d4dce0', danger='#e8bdc1')


def cell_width(char):
    if unicodedata.combining(char):
        return 0
    return 2 if unicodedata.east_asian_width(char) in ('W', 'F') else 1


def cell_len(text):
    return sum(cell_width(c) for c in text)


class Canvas:
    def __init__(self, cols, rows, palette):
        self.cols, self.rows, self.p = cols, rows, palette
        self.grid = [[[' ', 'fg', 'bg'] for _ in range(cols)] for _ in range(rows)]

    def rect(self, x, y, width, height, bg):
        for row in range(max(0, y), min(self.rows, y + height)):
            for col in range(max(0, x), min(self.cols, x + width)):
                self.grid[row][col] = [' ', 'fg', bg]

    def text(self, x, y, text, fg='fg', bg=None, limit=None):
        if not 0 <= y < self.rows:
            return
        end = min(self.cols, x + (limit if limit is not None else self.cols))
        for char in text:
            width = cell_width(char)
            if width == 0:
                if x > 0:
                    self.grid[y][x - 1][0] += char
                continue
            if x + width > end:
                break
            if x >= 0:
                previous_bg = self.grid[y][x][2]
                self.grid[y][x] = [char, fg, bg or previous_bg]
                if width == 2:
                    self.grid[y][x + 1] = ['', fg, bg or previous_bg]
            x += width

    def semantic(self, x, y, text, fg='fg', bg=None, limit=None):
        self.text(x, y, text, fg, bg, limit)
        if fg in ('red', 'green'):
            return
        rules = [
            (r'\b(?:pvsouytl|qpvuntsm|mzrqpvnu|vtqsknru|zuktvkvo|ntwquvxs)\b', 'purple'),
            (r'\b[a-f0-9]{8}\b', 'blue'),
            (r'\bjoshka\b', 'yellow'),
            (r'2026-\d\d-\d\d(?: \d\d:\d\d:\d\d)?', 'accent'),
            (r'\b(?:main|review-ui)\b|default@', 'green'),
            (r'\+\d+', 'green'), (r'(?<!\S)-\d+(?!\d)', 'red'),
            (r'^D(?=  )', 'red'), (r'^A(?=  )', 'green'), (r'^M(?=  )', 'yellow'),
        ]
        for pattern, color in rules:
            for match in re.finditer(pattern, text):
                offset = cell_len(text[:match.start()])
                remaining = None if limit is None else limit - offset
                if remaining is None or remaining > 0:
                    self.text(x + offset, y, match[0], color, bg, remaining)

    def ansi(self):
        out = ['\x1b[?25l\x1b[?7l\x1b[2J']
        for y, row in enumerate(self.grid):
            out.append(f'\x1b[{y + 1};1H')
            previous = None
            for char, fg, bg in row:
                if (fg, bg) != previous:
                    rgb = lambda role: ';'.join(str(int(self.p[role][i:i+2], 16)) for i in (1, 3, 5))
                    out.append(f'\x1b[38;2;{rgb(fg)}m\x1b[48;2;{rgb(bg)}m')
                    previous = fg, bg
                out.append(char)
        out.append('\x1b[0m\x1b[H')
        return ''.join(out)


def footer(canvas, screen, more=0):
    cols, rows = canvas.cols, canvas.rows
    status = screen.get('status', '')
    if more:
        status = f'↓ {more} more lines · scroll to read'
    canvas.text(1, rows - 2, status, 'muted', limit=cols - 2)
    canvas.rect(0, rows - 1, cols, 1, 'surface')
    actions = screen.get('actions', [{'key': 'Esc', 'label': 'Back'}])
    x = 1
    for action in actions:
        key, label = action['key'], action['label']
        width = cell_len(key + ' ' + label) + 3
        if x + width > cols:
            canvas.text(cols - 9, rows - 1, '? More', 'accent')
            break
        canvas.text(x, rows - 1, key, 'accent')
        canvas.text(x + cell_len(key) + 1, rows - 1, label)
        x += width


def draw(screen, all_screens, cols, rows, palette):
    c = Canvas(cols, rows, palette)
    modal = screen.get('modal', False)
    base = all_screens.get(screen.get('base', 'log'), screen) if modal else screen
    c.rect(0, 0, cols, 1, 'surface')
    c.text(1, 0, 'jk', 'accent')
    command = 'jj ' + base['command'] if base.get('command') else base['title']
    scope = screen.get('scope', 'default')
    c.text(5, 0, command, limit=max(12, cols - cell_len(scope) - 9))
    c.text(cols - cell_len(scope) - 2, 0, scope, 'muted')
    if modal:
        for i, line in enumerate(base.get('lines', [])[:rows - 5]):
            c.semantic(4, i + 2, line, 'muted', limit=cols - 5)
    lines = list(screen.get('lines', []))
    if screen.get('mode') == 'catalog':
        lines = ['Find a jj command', ': rebase', '', '› rebase      Move revisions to different parents',
                 '  restore     Restore paths from another revision',
                 '  revert      Apply the reverse of revisions', '',
                 'Choose a command to set operands and review its effects.']
    elif screen.get('mode') == 'result':
        lines = ['Rebased 2 commits onto destination', '',
                 'Working copy now at: pvsouytl 283e5c68 Add contextual action menu',
                 'Operation: 91ab32cd', '', 'u Undo   o Operation log   C Command history']
    if not lines:
        lines = [screen['title'], '', 'Esc returns to the previous view.']
    kinds = screen.get('kinds', {})
    if modal:
        width = min(cols - 6, max(48, min(78, max(cell_len(l) for l in lines) + 6)))
        content_width = width - 4
        wrapped = []
        for i, line in enumerate(lines):
            parts = textwrap.wrap(line, content_width, replace_whitespace=False,
                                  drop_whitespace=False) or ['']
            wrapped.extend((part, i) for part in parts)
        is_preview = screen.get('mode') == 'preview' or screen['id'] in {
            'rebase-preview', 'push-review', 'new-merge', 'description-review',
            'cmd-split', 'cmd-squash', 'cmd-restore'}
        if is_preview and screen.get('command'):
            wrapped += [('', -1), ('Command', -2)]
            wrapped += [(line, -1) for line in textwrap.wrap('jj ' + screen['command'], content_width)]
        buttons = screen.get('buttons')
        if not buttons and is_preview:
            buttons = [{'label': 'Cancel', 'role': 'cancel', 'focused': True},
                       {'label': 'Run command', 'role': 'primary'}]
        controls_height = 6 if buttons else 0
        height = min(rows - 6, len(wrapped) + 4 + controls_height)
        x, y = (cols - width) // 2, max(2, (rows - 2 - height) // 2)
        c.rect(x, y, width, height, 'surface')
        c.text(x + 2, y + 1, screen['title'].replace('Contextual help · ', 'Help · '),
               'accent', limit=content_width)
        available = height - 4 - controls_height
        visible = wrapped[:available]
        for n, (line, original) in enumerate(visible):
            row = y + 3 + n
            selected = original == screen.get('selected')
            if selected:
                c.rect(x, row, width, 1, 'selected')
                c.text(x, row, '›', 'accent')
            fg = 'muted' if original == -2 else 'fg'
            c.semantic(x + 2, row, line, fg, limit=content_width)
        if buttons:
            widths = [max(12, cell_len(b['label']) + 4) for b in buttons]
            gap = 2
            bx = x + 2
            by = y + height - 6
            for b, bw in zip(buttons, widths):
                if bx + bw > x + width - 1:
                    break
                bg = 'accent' if b.get('focused') else ('danger' if b['role'] == 'danger' else 'button')
                fg = 'bg' if b.get('focused') else 'fg'
                c.rect(bx, by, bw, 3, bg)
                label = '› ' + b['label'] + ' ‹' if b.get('focused') else b['label']
                c.text(bx + (bw - cell_len(label)) // 2, by + 1, label, fg)
                bx += bw + gap
            c.text(x + 2, by + 4, 'Tab/←/→ choose · Enter activate · Esc cancel', 'muted', limit=content_width)
        more = max(0, len(wrapped) - available)
    else:
        available = rows - 5
        for i, line in enumerate(lines[:available]):
            y = i + 2
            selected = i == screen.get('selected')
            if selected:
                c.rect(0, y, cols, 1, 'selected')
                c.text(0, y, '›', 'accent')
            mark = screen.get('marks', {}).get(str(i), '')
            c.text(2, y, str(mark), 'accent')
            fg = {'add': 'green', 'del': 'red', 'muted': 'muted'}.get(kinds.get(str(i)), 'fg')
            if line.startswith('+'):
                fg = 'green'
            elif line.startswith('-') or line.startswith('Error:'):
                fg = 'red'
            if line.startswith(('Modified regular file ', 'Added regular file ')):
                c.rect(0, y, cols, 1, 'surface')
                fg = 'yellow'
            c.semantic(4, y, line, fg, limit=cols - 5)
        more = max(0, len(lines) - available)
    footer(c, screen, more)
    if modal and buttons:
        c.rect(0, rows - 1, cols, 1, 'surface')
        c.text(1, rows - 1, 'Preview · default workspace · no changes until confirmed', 'muted', limit=cols - 2)
    return c


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('fixtures', type=Path)
    parser.add_argument('--screen', default='log')
    parser.add_argument('--theme', choices=['dark', 'light'], default='dark')
    parser.add_argument('--all', action='store_true', help='Advance alphabetically with n for capture')
    parser.add_argument('--metadata', type=Path)
    args = parser.parse_args()
    data = json.loads(args.fixtures.read_text())
    all_screens = {s['id']: s for s in data['screens']}
    screen_ids = sorted(all_screens) if args.all else [args.screen]
    cols, rows = os.get_terminal_size()
    palette = DARK if args.theme == 'dark' else LIGHT
    if args.metadata:
        args.metadata.write_text(json.dumps({'columns': cols, 'rows': rows,
                                             'theme': args.theme, 'screens': screen_ids}, indent=2))
    old = termios.tcgetattr(sys.stdin)
    try:
        tty.setraw(sys.stdin.fileno())
        for sid in screen_ids:
            sys.stdout.write(draw(all_screens[sid], all_screens, cols, rows, palette).ansi())
            sys.stdout.flush()
            while True:
                key = sys.stdin.read(1)
                if key in ('q', '\x03'):
                    return
                if key in ('n', '\r', ' '):
                    break
    finally:
        termios.tcsetattr(sys.stdin, termios.TCSADRAIN, old)
        sys.stdout.write('\x1b[0m\x1b[?7h\x1b[?25h')


if __name__ == '__main__':
    main()
