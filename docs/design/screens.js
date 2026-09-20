// Design fixtures only. No command in this gallery is executed.
const screens = [];
const add = (id, title, group, command, lines, extra = {}) => screens.push({id,title,group,command,lines,...extra});
const graph = [
'@  pvsouytl joshka 2026-09-19 10:42:08 default@ 283e5c68',
'│  Add contextual action menu',
'○  qpvuntsm joshka 2026-09-19 10:31:42 review-ui 0f71aeb2',
'│  Keep focus when closing a preview',
'○  mzrqpvnu joshka 2026-09-19 09:58:16 8ab16cc0',
'│  Preserve jj colors in selected revisions',
'│ ○  vtqsknru joshka 2026-09-18 17:04:21 docs 70d290bf',
'│ │  Explain operation recovery',
'├─╯',
'◆  zuktvkvo joshka 2026-09-18 16:22:03 main 5c3bcaf5',
'│  Improve contextual help overlays',
'~'
];
const patch = [
'Modified regular file crates/jk-tui/src/selected_row.rs:',
'    8    8: use ratatui::prelude::Color;',
'    9    9:',
'   10     : const SELECTED_ROW_BG: Color = Color::Rgb(82, 196, 192);',
'        10: const SELECTED_ROW_BG: Color = Color::Reset;',
'   11   11:',
'   12   12: /// Keep jj foreground styles when indicating selection.',
'   13   13: pub fn paint_selected_row(frame: &mut Frame, area: Rect) {',
'   14     :     cell.set_fg(SELECTED_ROW_FG);',
'        14:     paint_selection_marker(frame, area);',
'   15   15: }',
'',
'Modified regular file docs/tui-design.md:',
'    1    1: # TUI design guidelines',
'         2: Preserve jj colors. Keep cursor, marks, and @ distinct.',
'         3: Use a quiet selection surface and a dedicated gutter.'
];
const files = ['M crates/jk-tui/src/selected_row.rs','M crates/jk-tui/src/chrome.rs','M docs/tui-design.md'];
const ops = ['@  82fda090 joshka 2026-09-19 10:42:08','│  describe commit 283e5c68','○  734bb86a joshka 2026-09-19 10:31:42','│  rebase commit 0f71aeb2 and descendants','○  6687b1c0 joshka 2026-09-19 09:58:16','│  new empty commit','○  412fe900 joshka 2026-09-19 09:41:02','│  snapshot working copy'];
const btn=(key,label,to)=>({key,label,to});
const inspection=[btn('d','Diff','diff'),btn('s','Status','status'),btn('a','Actions','actions'),btn('?','Help','help-log')];
add('log','Revision graph','P0 · Inspect','log',graph,{selected:2,actions:[btn('Enter','Show','show'),...inspection],note:'Configured jj graph, template, and colors; jk adds only the interaction gutter.'});
add('expanded','Inline inspection','P0 · Inspect','log', [...graph.slice(0,4),'│  M crates/jk-tui/src/chrome.rs','│  M docs/tui-design.md','│  2 files changed, 18 insertions(+), 6 deletions(-)',...graph.slice(4)],{selected:2,actions:[btn('Enter','Collapse','log'),...inspection]});
add('marks','Ordered revision marks','P0 · Inspect','log',graph,{selected:6,marks:{2:'1',4:'2'},status:'2 marked · first mark is the comparison source',actions:[btn('d','Compare','compare'),btn('a','Actions','actions'),btn('Space','Mark / unmark','marks'),btn('Esc','Clear marks','log')]});
add('show','Change details','P0 · Inspect','show qpvuntsm',[
'Commit ID: 0f71aeb2d4ef76a9013b823b11a3c49e21d659ab',
'Change ID: qpvuntsmrvqpxwqvtnksymrolntqqyss',
'Bookmarks: review-ui',
'Author   : Joshka <joshka@users.noreply.github.com> (2026-09-19)',
'Committer: Joshka <joshka@users.noreply.github.com> (2026-09-19)',
'', '    Keep focus when closing a preview','',...patch.slice(0,11)],{actions:[btn('d','Diff','diff'),btn('v','Evolution','evolog'),btn('V','View options','view-options'),btn('Esc','Back','log')]});
add('diff','Diff · configured color-words','P0 · Inspect','diff -r mzrqpvnu',patch,{kinds:{3:'del',4:'add',8:'del',9:'add',14:'add',15:'add'},actions:[btn('f','Files','files'),btn('/','Find','search'),btn('-','Fold hunk','folded'),btn('V','View options','view-options'),btn('?','Help','help-diff')]});
add('folded','Diff · folded context','P0 · Inspect','diff -r mzrqpvnu',[
patch[0],'  ▸ Hunk at line 8 · 8 lines hidden','',patch[12],'    1    1: # TUI design guidelines',patch[14],patch[15]],{kinds:{1:'muted',5:'add',6:'add'},actions:[btn('+','Unfold hunk','diff'),btn('f','Files','files'),btn('Esc','Back','log')]});
add('search','Diff · search','P0 · Inspect','diff -r mzrqpvnu',patch,{query:'selection',status:'/ selection   1 of 3 matches',actions:[btn('n','Next match','search'),btn('Esc','Close search','diff')],input:{label:'Find in diff',value:'selection'}});
add('files','Diff · file picker','P0 · Inspect','diff -r mzrqpvnu',files,{modal:true,selected:0,base:'diff',actions:[btn('Enter','Jump to file','diff'),btn('Space','Mark','content-picker'),btn('Esc','Cancel','diff')]});
add('compare','Compare two revisions','P0 · Inspect','diff --from mzrqpvnu --to qpvuntsm',['From  mzrqpvnu  Preserve jj colors in selected revisions','To    qpvuntsm  Keep focus when closing a preview','',...patch],{actions:[btn('V','View options','view-options'),btn('Esc','Back','marks')]});
add('status','Working-copy status','P0 · Inspect','status',['Working copy changes:',...files,'','Working copy  (@) : pvsouytl 283e5c68 Add contextual action menu','Parent commit (@-): qpvuntsm 0f71aeb2 review-ui | Keep focus', ''],{actions:[btn('d','Diff','diff'),btn('c','Commit','commit-form'),btn('a','File actions','file-actions'),btn('Esc','Back','log')]});
add('revset','Revset filter and completion','P0 · Inspect','log -r "mine() & mutable()"',['Filter revisions','', 'mine() & mutable()','', '  mutable()    Revisions outside the immutable set','  mine()       Revisions authored by you','', 'Resolved: 4 revisions · existing marks stay attached to their changes'],{modal:true,input:{label:'Revset',value:'mine() & mutable()'},actions:[btn('Enter','Apply filter','filtered'),btn('Esc','Cancel','log')]});
add('filtered','Filtered graph','P0 · Inspect','log -r "mine() & mutable()"',graph.slice(0,9),{selected:2,status:'Filter: mine() & mutable() · 4 changes',actions:[btn('/','Edit filter','revset'),btn('Esc','Previous filter','log'),...inspection]});
add('actions','Contextual action menu','P0 · Controls','log',[
'Change actions · qpvuntsm', '',
'  n  New change           e  Edit change',
'  m  Describe             c  Commit working copy',
'  R  Rebase               a  Abandon', '',
'Move content',
'  s  Squash               S  Split',
'  r  Restore              b  Absorb',
'  d  Diff editor          Resolve conflicts…', '',
'History and recovery',
'  o  Operation log        u  Undo',
'  C  Command history      :  All jj commands'],{modal:true,actions:[btn('R','Rebase','rebase-target'),btn('s','Squash','squash-files'),btn('S','Split','split-files'),btn('m','Describe','describe-form'),btn('Esc','Close','log')],note:'Menu groups are the design direction. Unresolved shortcut collisions are not a final keymap.'});
add('help-log','Contextual help · graph','P0 · Controls','log',[
'Open and inspect',
'  Enter             Show selected change',
'  d                 Diff selected change or ordered marks',
'  s                 Working-copy status',
'  v                 Evolution of selected change', '',
'Change actions',
'  a                 Open contextual action menu',
'  n / e             New change / edit selected change',
'  m                 Describe selected change', '',
'History and recovery',
'  o / C             Operation log / command history',
'  u / U             Preview undo / redo', '',
'Move and find',
'  j/k, ↑/↓          Move between changes',
'  Ctrl-j/Ctrl-k     Scroll without moving selection',
'  Space             Mark or unmark selected change',
'  V                 View options',
'  W                 Workspaces',
'  :                 jj command mode'],{modal:true,actions:[btn('Esc','Close help','log')],note:'One document, shared column stops, no selection cursor in help.'});
add('help-diff','Contextual help · diff','P0 · Controls','diff -r mzrqpvnu',[
'Move and find','  j/k, ↑/↓          Scroll rendered lines','  [ / ]             Previous / next file','  { / }             Previous / next hunk','  f                 Open file list','  /                 Search diff text','  n / N             Next / previous match','',
'Fold and inspect','  h / l             Fold / unfold file','  - / +             Fold / unfold hunk','  Space             Mark current file','  V                 View options','',
'Change actions','  a                 Actions for marked files','  O                 Open working-copy file in editor','',
'Session','  Esc / Backspace   Return to previous view'],{base:'diff',modal:true,actions:[btn('Esc','Close help','diff')]});
add('view-options','View options · diff','P0 · Controls','diff -r mzrqpvnu',[
'Format', '  (*) Configured         color-words', '  ( ) Git patch', '  ( ) Summary', '  ( ) Stat', '  ( ) File names', '',
'Context                  3 lines', '[ ] Ignore whitespace changes', '', 'Applied value: Configured · cursor does not apply an option'],{base:'diff',modal:true,selected:2,actions:[btn('Enter','Apply Git patch','git-diff'),btn('Esc','Cancel','diff')]});
add('git-diff','Diff · Git patch format','P0 · Inspect','diff -r mzrqpvnu --git',[
'diff --git a/crates/jk-tui/src/selected_row.rs b/crates/jk-tui/src/selected_row.rs',
'index 16c944b..e007c2a 100644','--- a/crates/jk-tui/src/selected_row.rs','+++ b/crates/jk-tui/src/selected_row.rs','@@ -10,3 +10,3 @@',
'-const SELECTED_ROW_BG: Color = Color::Rgb(82, 196, 192);','+const SELECTED_ROW_BG: Color = Color::Reset;',
' ', '-    cell.set_fg(SELECTED_ROW_FG);','+    paint_selection_marker(frame, area);'],{actions:[btn('V','View options','view-options'),btn('Esc','Back','log')]});
add('run-options','Run options · shared scope','P0 · Controls','rebase -s qpvuntsm -o main',[
'Repository              /work/jk', 'Workspace               default', 'Operation               current', '',
'[ ] Ignore working copy', '[ ] Do not integrate operation', '[ ] Allow immutable rewrites', '',
'Config overlays         none', 'Output                  configured colors · normal diagnostics', '',
'Advanced flags change command semantics.', 'Not integrating an operation does not prevent remote side effects.'],{modal:true,actions:[btn('Enter','Apply options','rebase-preview'),btn('Esc','Cancel','rebase-preview')]});
add('command','All jj commands','P0 · Controls','',[],{mode:'catalog',actions:[btn('Esc','Back','log')],note:'Search is part of command discovery, not the contextual help sheet.'});
add('describe-form','Describe · edit text','P0 · Change','describe qpvuntsm',['Change       qpvuntsm 0f71aeb2','', 'Description', 'Keep focus when closing a preview','', 'Restore the selected revision and scroll position on return.', '', 'Changes are local to this draft until reviewed and run.'],{modal:true,input:{label:'Description',value:'Keep focus when closing a preview'},actions:[btn('Enter','Review','cmd-describe'),btn('Esc','Cancel','log')]});
add('commit-form','Commit · working-copy message','P0 · Change','commit',['Working copy  pvsouytl 283e5c68','',...files,'','Description','Add contextual action menu','','Updates this description, then creates a new working-copy change.'],{modal:true,input:{label:'Description',value:'Add contextual action menu'},actions:[btn('Enter','Review','cmd-commit'),btn('Esc','Cancel','status')]});
add('rebase-target','Rebase · choose destination','P0 · Change','rebase -s qpvuntsm -o main',[
'Source       qpvuntsm + descendants', 'Scope        Source and descendants (-s)', 'Destination  main', '',
'◆  zuktvkvo  main         Improve contextual help overlays',
'○  vtqsknru  docs         Explain operation recovery', '',
'2 revisions will move. Existing source parents are replaced.', 'Destination candidates exclude the selected source and descendants.'],{modal:true,selected:4,input:{label:'Destination revset',value:'main'},actions:[btn('Enter','Review rebase','rebase-preview'),btn('Esc','Cancel','log')]});
add('rebase-preview','Rebase · review graph change','P0 · Change','rebase -s qpvuntsm -o main',[
'Source       qpvuntsm + pvsouytl', 'Destination  zuktvkvo main', '',
'Before                         After · illustrative',
'@ pvsouytl                     @ pvsouytl',
'○ qpvuntsm                     ○ qpvuntsm',
'○ mzrqpvnu                     │ ○ mzrqpvnu',
'◆ main                         ├─╯',
'                               ◆ main', '',
'2 revisions rewritten; working-copy identity remains pvsouytl.',
'Conflict prediction is not available in this mockup.'],{modal:true,actions:[btn('Enter','Run rebase','running'),btn('O','Run options','run-options'),btn('Esc','Change target','rebase-target')],note:'The graph is a design illustration, not a verified simulation of repository changes.'});
add('content-picker','Fileset selector','P0 · Content','split -r pvsouytl',['Revision   pvsouytl','Fileset    all()','',...files,'','2 marked files · no hunks selected'],{modal:true,selected:3,marks:{3:'✓',5:'✓'},actions:[btn('Space','Toggle file','content-picker'),btn('Enter','Review','cmd-split'),btn('Esc','Cancel','diff')]});
add('squash-files','Squash · source and content','P0 · Content','squash --from pvsouytl --into qpvuntsm',[
'From         pvsouytl Add contextual action menu', 'Into         qpvuntsm Keep focus when closing a preview', '',
'[x] crates/jk-tui/src/chrome.rs','[ ] docs/tui-design.md','',
'Message      Keep destination message', '[ ] Keep emptied source', '',
'Only the selected file changes move; remaining work stays in source.'],{modal:true,actions:[btn('Enter','Review squash','cmd-squash'),btn('i','Choose hunks externally','external-diff'),btn('Esc','Cancel','log')]});
add('split-files','Split · selected and remaining content','P0 · Content','split -r pvsouytl',[
'Selected changes → first revision','[x] crates/jk-tui/src/chrome.rs','',
'Remaining changes → child revision','[ ] docs/tui-design.md','',
'First description  Add contextual action menu','[ ] Make revisions parallel','',
'For partial files, continue in the configured diff editor.'],{modal:true,actions:[btn('Enter','Review split','cmd-split'),btn('i','Choose hunks externally','external-diff'),btn('Esc','Cancel','log')]});
add('restore-files','Restore · explicit direction','P0 · Content','restore --from @- --into @ docs/tui-design.md',[
'Copy content from  @-  qpvuntsm', 'Replace content in @   pvsouytl', '',
'[x] docs/tui-design.md','[ ] crates/jk-tui/src/chrome.rs','',
'Current changes in the selected path will be replaced.', 'Other files stay unchanged. Review the affected path before running.'],{modal:true,actions:[btn('Enter','Review restore','cmd-restore'),btn('d','Inspect content','diff'),btn('Esc','Cancel','status')]});
add('running','Command running','P0 · Recovery','rebase -s qpvuntsm -o main',['Running rebase','', 'Workspace default · 2 revisions in scope', '', 'Waiting for jj…', '', 'A stop request cannot undo already completed work.'],{modal:true,actions:[btn('Enter','Show sample completion','success'),btn('Esc','Show sample interruption','interrupted')]});
add('success','Mutation complete','P0 · Recovery','log',graph,{selected:2,status:'Rebased 2 commits · operation 91ab32cd · undo available',actions:[btn('u','Undo','cmd-undo'),btn('o','Operation log','op-log'),btn('C','Command history','history'),btn('Enter','Show','show')]});
add('interrupted','Interrupted command','P0 · Recovery','rebase -s qpvuntsm -o main',['Command interrupted','', 'Repository state is being refreshed.', 'Inspect operation history before retrying.', '', 'Partial effects, if any, are not rolled back by closing this view.'],{actions:[btn('o','Operation log','op-log'),btn('C','Command history','history'),btn('Esc','Back','log')]});
add('op-log','Operation history','P0 · Recovery','operation log',ops,{selected:2,actions:[btn('Enter','Show operation','op-show'),btn('d','Compare operations','op-diff'),btn('l','Graph at operation','time-travel'),btn('r','Restore','cmd-operation-restore'),btn('Esc','Back','log')]});
add('op-show','Operation details','P0 · Recovery','operation show 734bb86a',[
'Operation: 734bb86a', 'User: joshka', 'Description: rebase commit 0f71aeb2 and descendants', '',
'Changed commits:', '+ qpvuntsm 0f71aeb2 Keep focus when closing a preview','- qpvuntsm 591ef10c Keep focus when closing a preview','',
'Changed working copies:', '  default@ pvsouytl 283e5c68','',
'Changed local bookmarks:', '  review-ui: 591ef10c → 0f71aeb2'],{actions:[btn('d','Diff operations','op-diff'),btn('r','Restore state','cmd-operation-restore'),btn('Esc','Back','op-log')]});
add('op-diff','Operation comparison','P0 · Recovery','operation diff --from 6687b1c0 --to 734bb86a',[
'From operation 6687b1c0', 'To operation   734bb86a', '',
'Changed commits:', '+ qpvuntsm 0f71aeb2 Keep focus when closing a preview', '- qpvuntsm 591ef10c Keep focus when closing a preview','',
'Changed local bookmarks:', '  review-ui: 591ef10c → 0f71aeb2'],{actions:[btn('V','View options','view-options'),btn('Esc','Back','op-log')]});
add('time-travel','Graph at a prior operation','P0 · Recovery','--at-operation 6687b1c0 log',graph.slice(2),{status:'Historical operation 6687b1c0 · working copy ignored',actions:[btn('Enter','Inspect change','show'),btn('Esc','Return to current','op-log')],note:'Historical inspection keeps its scope visible; mutations require explicit advanced intent.'});
add('history','Command history','P0 · Recovery','',[
'Outcome  Duration  Workspace  Command', 'ok       182 ms    default    jj rebase -s qpvuntsm -o main',
'error    45 ms     default    jj git push --bookmark review-ui',
'ok       16 ms     default    jj describe qpvuntsm -m "Keep focus"',
'ok       10 ms     default    jj diff -r mzrqpvnu'],{selected:1,actions:[btn('Enter','Command details','history-detail'),btn('Esc','Back','log')]});
add('history-detail','Retained command output','P0 · Recovery','rebase -s qpvuntsm -o main',[
'Outcome      ok · exit 0', 'Duration     182 ms', 'Workspace    default', 'Operation    91ab32cd','',
'Rebased 2 commits onto destination', 'Working copy now at: pvsouytl 283e5c68 Add contextual action menu', '',
'Replay opens a fresh preview against current state.'],{actions:[btn('Enter','Review replay','rebase-preview'),btn('o','Operation','op-show'),btn('Esc','Back','history')]});
add('bookmarks','Bookmarks and tracking','P0 · Share','bookmark list',[
'review-ui: qpvuntsm 0f71aeb2 Keep focus when closing a preview','  @origin (behind by 2 commits): mzrqpvnu 8ab16cc0',
'main: zuktvkvo 5c3bcaf5 Improve contextual help overlays','  @origin: zuktvkvo 5c3bcaf5','docs: vtqsknru 70d290bf Explain operation recovery','',
'Untracked remote bookmarks:', 'release@origin: vynspqro 2b2e7109 Prepare release'],{selected:0,actions:[btn('Enter','Inspect target','show'),btn('m','Move','cmd-bookmark-move'),btn('P','Push','push-review'),btn('Esc','Back','log')]});
add('push-review','Push · dry-run review','P0 · Share','git push --remote origin --bookmark review-ui --dry-run',[
'Remote       origin', 'Bookmark     review-ui', '',
'Changes to push to origin:', '  Move forward bookmark review-ui from 8ab16cc0 to 0f71aeb2', '',
'Dry-run: no remote changes made.', '', 'Next command', 'jj git push --remote origin --bookmark review-ui', '',
'The remote changes only after Run push.'],{modal:true,actions:[btn('Enter','Run push','push-result'),btn('Esc','Cancel','bookmarks')]});
add('push-result','Push · result','P0 · Share','git push --remote origin --bookmark review-ui',[
'Push completed', '', 'review-ui@origin now points to qpvuntsm 0f71aeb2', '',
'Remote changes are not undone by local operation undo.'],{actions:[btn('Enter','Bookmarks','bookmarks'),btn('C','Command history','history')]});
add('workspaces','Workspace overview','P0 · Workspaces','workspace list',[
'default: pvsouytl 283e5c68 Add contextual action menu',
'docs: vtqsknru 70d290bf Explain operation recovery',
'review: ntwquvxs a531df11 Review release notes', '',
'Inspected workspace  review', 'Path                 /work/jk/review', 'Working copy         stale',
'Current app scope    default', '', 'Opening status inspects review; it does not switch app scope.'],{selected:2,actions:[btn('s','Status','workspace-status'),btn('u','Update stale','cmd-workspace-update-stale'),btn('n','Add','cmd-workspace-add'),btn('Esc','Back','log')]});
add('workspace-status','Status in another workspace','P0 · Workspaces','-R /work/jk/review status',[
'Working copy changes:', 'M docs/release-notes.md', '', 'Working copy  (@) : ntwquvxs a531df11 Review release notes', 'Parent commit (@-): zuktvkvo 5c3bcaf5 main', '',
'Inspection scope: review · app home: default'],{scope:'review',actions:[btn('Esc','Back to workspaces','workspaces')]});
add('evolog','Evolution of a change','P1 · Inspect','evolog -r qpvuntsm',[
'○  qpvuntsm 0f71aeb2 joshka 2026-09-19 10:31:42', '│  Keep focus when closing a preview',
'○  qpvuntsm 591ef10c joshka 2026-09-19 10:21:09', '│  Keep focus when closing a preview',
'○  qpvuntsm 271a07f0 joshka 2026-09-19 10:08:33', '   Fix preview focus'],{selected:2,actions:[btn('d','Compare versions','interdiff'),btn('Enter','Show','show'),btn('Esc','Back','log')]});
add('interdiff','Compare change versions','P1 · Inspect','interdiff --from 591ef10c --to 0f71aeb2',[
'Earlier  qpvuntsm 591ef10c', 'Later    qpvuntsm 0f71aeb2', '',...patch.slice(0,11)],{actions:[btn('Esc','Back','evolog')]});
add('file-list','Files at a revision','P1 · Inspect','file list -r qpvuntsm',[
'Cargo.toml','README.md','crates/jk/src/main.rs','crates/jk-tui/src/chrome.rs',
'crates/jk-tui/src/selected_row.rs','docs/tui-design.md'],{selected:4,actions:[btn('Enter','Open file','file-show'),btn('a','Annotate','annotate'),btn('/','Search files','file-search'),btn('Esc','Back','log')]});
add('file-show','File content','P1 · Inspect','file show -r qpvuntsm crates/jk-tui/src/selected_row.rs',[
'//! Selected-row highlighting for rendered log output.','',
'use ratatui::Frame;', 'use ratatui::layout::Rect;', '',
'/// Keep the user\'s jj colors while indicating selection.',
'pub fn paint_selected_row(frame: &mut Frame, area: Rect) {',
'    paint_selection_marker(frame, area);', '}'],{actions:[btn('a','Annotate','annotate'),btn('Esc','Back','file-list')]});
add('annotate','File annotation','P1 · Inspect','file annotate -r qpvuntsm crates/jk-tui/src/selected_row.rs',[
'mzrqpvnu joshka 2026-09-19  1: //! Selected-row highlighting.',
'zuktvkvo joshka 2026-09-18  2:', 'zuktvkvo joshka 2026-09-18  3: use ratatui::Frame;',
'mzrqpvnu joshka 2026-09-19  4: paint_selection_marker(frame, area);'],{selected:3,actions:[btn('Enter','Inspect change','show'),btn('Esc','Back','file-show')]});
add('file-search','Search repository content','P1 · Inspect','file search -r qpvuntsm --pattern selection',[
'crates/jk-tui/src/selected_row.rs:1: //! Selected-row highlighting.',
'crates/jk-tui/src/selected_row.rs:14: paint_selection_marker(frame, area);',
'docs/tui-design.md:48: Preserve semantic foregrounds during selection.'],{query:'selection',input:{label:'Search pattern',value:'selection'},actions:[btn('Enter','Open match','file-show'),btn('Esc','Back','file-list')]});
add('conflicts','Conflicted files','P1 · Content','resolve --list',[
'crates/jk-tui/src/chrome.rs    2-sided conflict', '',
'Change       pvsouytl', 'Merge tool   configured jj merge tool', '',
'Open the selected file in the configured tool.', 'On return, refresh this list and review the resolved diff.'],{selected:0,actions:[btn('Enter','Review tool launch','cmd-resolve'),btn('d','Inspect','diff'),btn('Esc','Back','status')]});
add('external-diff','External editor handoff','P1 · Content','split -r pvsouytl --interactive',[
'Choose partial changes in your configured diff editor', '',
'Change      pvsouytl', 'Task        Split selected changes from this revision', '',
'jk releases the terminal to jj and the configured editor.', 'After the editor exits, jk refreshes and restores this context.'],{modal:true,actions:[btn('Enter','Show return state','success'),btn('Esc','Cancel before launch','split-files')]});
add('file-actions','File actions','P1 · Content','',[
'Selected path  docs/tui-design.md','', 'Inspect','  Open content','  Annotate','', 'Change','  Restore from revision…','  Stop tracking…','  Set executable bit…'],{modal:true,actions:[btn('r','Restore','restore-files'),btn('u','Untrack','cmd-file-untrack'),btn('Esc','Close','status')]});
add('tags','Tags','P1 · Share','tag list',[
'v0.2.7: zuktvkvo 5c3bcaf5 Improve contextual help overlays','  @origin: zuktvkvo 5c3bcaf5', 'v0.2.6: plmvnqsr 764a2ce1 Stabilize diff navigation'],{selected:0,actions:[btn('Enter','Inspect','show'),btn('s','Set','cmd-tag-set'),btn('d','Delete','cmd-tag-delete'),btn('Esc','Back','log')]});
add('remotes','Git remotes','P1 · Share','git remote list',[
'origin https://github.com/joshka/jk.git', 'upstream https://github.com/example/jk.git'],{selected:0,actions:[btn('F','Fetch','cmd-git-fetch'),btn('u','Set URL','cmd-git-remote-set-url'),btn('n','Add','cmd-git-remote-add'),btn('Esc','Back','bookmarks')]});
add('welcome','Open or create a repository','P1 · Setup','',[
'No jj repository in this directory', '',
'  Open an existing workspace', '  Clone a Git repository', '  Initialize a Git-backed jj repository', '',
'Current directory  /work'],{selected:3,actions:[btn('c','Clone','cmd-git-clone'),btn('i','Initialize','cmd-git-init')]});
add('sparse','Sparse working-copy paths','P2 · Advanced','sparse list',[
'crates/jk-tui','docs','', 'Workspace  review', 'Included paths are present in the working copy.', 'Other tracked paths remain in revision history.'],{scope:'review',actions:[btn('s','Set patterns','cmd-sparse-set'),btn('e','Edit externally','cmd-sparse-edit'),btn('r','Reset','cmd-sparse-reset'),btn('Esc','Back','workspaces')]});
add('config','Configuration and provenance','P2 · Advanced','config list',[
'user.name = "Joshka"', 'ui.editor = "hx"', 'ui.diff-formatter = "color-words"',
'ui.graph.style = "curved"','revsets.log = "present(@) | ancestors(immutable_heads().., 2)"','',
'Selected setting: ui.diff-formatter', 'Effective value: color-words', 'Scope for edits: repository · user config stays unchanged'],{selected:2,actions:[btn('Enter','Inspect value','cmd-config-get'),btn('s','Set value','cmd-config-set'),btn('Esc','Back','log')]});
add('immutable','Blocked immutable target','States · Edge cases','edit main',[
'Cannot edit immutable revision zuktvkvo', '', 'Target   main 5c3bcaf5', '',
'Create a new change on top of this revision to continue work.', 'Overriding immutability is an explicit advanced option.'],{modal:true,actions:[btn('n','Review new change','cmd-new'),btn('Esc','Choose another revision','log')]});
add('empty','No matching revisions','States · Edge cases','log -r "description(no-such-change)"',[
'No revisions match this filter.', '', 'Filter  description(no-such-change)', '', 'The repository is not empty.'],{actions:[btn('/','Edit filter','revset'),btn('Esc','Previous filter','log')]});
add('error','Command error · preserved context','States · Edge cases','git push --remote origin --bookmark review-ui',[
'Error: Cannot push commits with empty descriptions.', '', 'Affected change  pvsouytl 283e5c68', 'Remote           origin', '',
'Add a description, then review the push again.', 'The failed command and diagnostic are retained in history.'],{actions:[btn('m','Describe','describe-form'),btn('C','History','history'),btn('Esc','Back','bookmarks')]});
add('refresh','Refresh while inspecting','States · Edge cases','log',graph,{selected:2,status:'Refreshing… previous output retained · selection qpvuntsm',actions:[btn('Enter','Show refreshed state','log'),btn('Esc','Stop refresh','log')]});
add('narrow','Compact terminal · graph','States · Edge cases','log',[
'@ pvsouytl joshka default@ 283e5c68','│ Add contextual action menu',
'○ qpvuntsm joshka review-ui 0f71aeb2','│ Keep focus when closing a preview',
'○ mzrqpvnu joshka 8ab16cc0','│ Preserve jj colors in selected revisions',
'◆ zuktvkvo joshka main 5c3bcaf5','│ Improve contextual help overlays'],{selected:2,compact:true,actions:[btn('Enter','Show','show'),btn('a','Actions','actions'),btn('?','Help','help-log')],note:'Compact jj template is an explicit user choice here, not a silent replacement of configured output.'});
// Each command gets its own review state. Shared layout never hides command-specific effects.
const commandDesigns = `
abandon|abandon qpvuntsm|Revision: qpvuntsm;Descendants: pvsouytl|Abandon the selected revision; descendants are rebased onto its parents.|P0 · Change
absorb|absorb --from @|Source: @ pvsouytl;Destinations: eligible mutable ancestors|Let jj distribute changes to mutable ancestors. Inspect operation output afterward.|P1 · Content
arrange|arrange|Input: configured interactive arrange tool|Hand the terminal to jj's graph arranger; return to the refreshed graph.|P2 · Advanced
bisect run|bisect run --range 'main..@' cargo test|Range: main..@;Test command: cargo test|Run the test across candidate revisions to locate the first bad revision.|P3 · Tools
bookmark advance|bookmark advance review-ui --to @|Bookmark: review-ui;Target: @ pvsouytl|Advance the closest matching bookmark to the selected target.|P0 · Share
bookmark create|bookmark create review-v2 -r @|New name: review-v2;Target: @ pvsouytl|Create a local bookmark. Nothing is pushed.|P0 · Share
bookmark delete|bookmark delete review-ui|Bookmark: review-ui;Remote: origin tracking relationship|Delete locally; the deletion can be propagated by a later push.|P0 · Share
bookmark forget|bookmark forget review-ui|Bookmark: review-ui|Forget local bookmark state without scheduling a remote deletion.|P0 · Share
bookmark move|bookmark move review-ui --to @|Bookmark: review-ui;From: qpvuntsm;To: @ pvsouytl|Move the existing local bookmark; pushing remains a separate action.|P0 · Share
bookmark rename|bookmark rename review-ui preview-focus|Old name: review-ui;New name: preview-focus|Rename the local bookmark; inspect remote tracking before the next push.|P0 · Share
bookmark set|bookmark set review-ui -r @|Name: review-ui;Target: @ pvsouytl|Create or update the local bookmark at this revision.|P0 · Share
bookmark track|bookmark track release@origin|Remote bookmark: release@origin|Start tracking the selected remote bookmark.|P0 · Share
bookmark untrack|bookmark untrack release@origin|Remote bookmark: release@origin|Stop tracking this remote bookmark; no remote deletion.|P0 · Share
commit|commit -m 'Add contextual action menu'|Working copy: @ pvsouytl;Files: all working-copy changes|Set this description, then create a new working-copy change on top.|P0 · Change
config edit|config edit --repo|Scope: repository;Editor: configured editor|Edit repository configuration externally, then reload effective settings.|P3 · Tools
config gc|config gc|Scope: repo-level config directories|Find orphaned repo-level config directories and optionally delete them.|P3 · Tools
config get|config get ui.diff-formatter|Setting: ui.diff-formatter|Read the effective value without changing configuration.|P2 · Advanced
config list|config list|Scope: effective configuration|Inspect jj's configured values and preserve its output formatting.|P2 · Advanced
config path|config path --repo|Scope: repository|Show the path to the repository configuration file.|P2 · Advanced
config set|config set --repo ui.diff-formatter color-words|Scope: repository;Key: ui.diff-formatter;Value: color-words|Write this setting to repository config; user-level config is unchanged.|P3 · Tools
config unset|config unset --repo ui.diff-formatter|Scope: repository;Key: ui.diff-formatter|Remove this override; a lower-precedence value may become effective.|P3 · Tools
converge|converge -r 'mutable()'|Search space: mutable();Mode: interactive jj resolution|Converge divergent versions of changes. jj may ask for resolution choices.|P2 · Advanced
describe|describe qpvuntsm -m 'Keep focus when closing a preview'|Change: qpvuntsm;Description: Keep focus when closing a preview|Rewrite the selected description; preserve file content.|P0 · Change
diffedit|diffedit -r @|Revision: @ pvsouytl;Tool: configured diff editor|Review and edit the revision's content in the configured external tool.|P1 · Content
duplicate|duplicate qpvuntsm|Source: qpvuntsm|Create a new change with the same content; keep the source revision.|P2 · Advanced
edit|edit qpvuntsm|Current: @ pvsouytl;New working copy: qpvuntsm|Update working-copy files to the selected revision.|P0 · Change
file chmod|file chmod +x scripts/check.sh|Path: scripts/check.sh;Mode: executable|Set the executable bit in the working-copy revision.|P1 · Content
file track|file track docs/notes.md|Path: docs/notes.md|Start tracking this path in the working copy.|P1 · Content
file untrack|file untrack docs/tui-design.md|Path: docs/tui-design.md|Stop tracking the path. Check ignore rules to prevent automatic retracking.|P1 · Content
fix|fix -s qpvuntsm|Source: qpvuntsm;Tools: configured fix tools|Run configured fix tools over selected revisions and descendants.|P1 · Content
gerrit upload|gerrit upload|Destination: configured Gerrit remote;Revisions: jj configured/default selection|Upload for code review. Confirm the resolved revisions before sending.|P3 · Tools
git clone|git clone https://github.com/joshka/jk.git /work/jk-new|Source URL: https://github.com/joshka/jk.git;Destination: /work/jk-new|Create a new Git-backed jj repository, then open its workspace.|P1 · Setup
git colocation disable|git colocation disable|Repository: /work/jk;Requested mode: non-colocated|Disable Git colocation for this repository.|P2 · Advanced
git colocation enable|git colocation enable|Repository: /work/jk;Requested mode: colocated|Enable Git colocation for this repository.|P2 · Advanced
git colocation status|git colocation status|Repository: /work/jk|Inspect whether this jj repository is colocated with Git.|P2 · Advanced
git export|git export|Repository: /work/jk|Export jj state to the underlying local Git repository; no remote push.|P1 · Share
git fetch|git fetch --remote origin|Remote: origin;Selection: configured fetch selection|Fetch remote state and refresh bookmarks; no push is performed.|P0 · Share
git import|git import|Repository: /work/jk|Import changes made in the underlying Git repository.|P1 · Share
git init|git init /work/jk-new|Destination: /work/jk-new|Create a new Git-backed jj repository at this location.|P1 · Setup
git push|git push --remote origin --bookmark review-ui|Remote: origin;Bookmark: review-ui|Review a dry-run before executing the push. Local undo does not undo a push.|P0 · Share
git remote add|git remote add upstream https://github.com/example/jk.git|Name: upstream;URL: https://github.com/example/jk.git|Save a remote URL locally. Fetch is a separate action.|P1 · Share
git remote remove|git remote remove upstream|Remote: upstream|Remove the local remote configuration and associated remote state.|P1 · Share
git remote rename|git remote rename upstream source|Old name: upstream;New name: source|Rename the configured remote locally.|P1 · Share
git remote set-url|git remote set-url origin https://github.com/joshka/jk.git|Remote: origin;URL: https://github.com/joshka/jk.git|Change the URL used for future fetches and pushes.|P1 · Share
git root|git root|Repository: /work/jk|Print the underlying Git directory path.|P2 · Advanced
metaedit|metaedit qpvuntsm --author 'Joshka <joshka@users.noreply.github.com>'|Change: qpvuntsm;Author: Joshka <joshka@users.noreply.github.com>|Rewrite author metadata while preserving file content.|P2 · Advanced
new|new qpvuntsm -m 'Continue preview work'|Parent: qpvuntsm;Description: Continue preview work|Create an empty change on this parent and edit it in the working copy.|P0 · Change
next|next|Current: @ pvsouytl;Direction: child revision|Move the working-copy revision, not the cursor in this graph.|P2 · Advanced
operation abandon|operation abandon 412fe900|Operation: 412fe900|Discard operation history at this point; inspect recovery consequences first.|P2 · Advanced
operation integrate|operation integrate 91ab32cd|Operation: 91ab32cd;State: previously unintegrated|Integrate this operation into the current operation history.|P2 · Advanced
operation restore|operation restore 6687b1c0|Restore state at: 6687b1c0;Portions: repo and remote-tracking|Restore local repository state to this operation. Remote servers are unchanged.|P0 · Recovery
operation revert|operation revert 734bb86a|Operation to reverse: 734bb86a|Reverse this operation's changes while retaining subsequent unrelated work.|P0 · Recovery
parallelize|parallelize 'mzrqpvnu::qpvuntsm'|Revisions: mzrqpvnu::qpvuntsm|Make these revisions siblings instead of an ordered stack.|P2 · Advanced
prev|prev|Current: @ pvsouytl;Direction: parent revision|Move the working-copy revision, not the viewport or cursor.|P2 · Advanced
rebase|rebase -s qpvuntsm -o main|Source: qpvuntsm and descendants;Onto: main zuktvkvo|Rebase the source subtree onto main; review source scope before running.|P0 · Change
redo|redo|Operation: most recently undone operation|Redo the operation currently eligible in jj's undo history.|P0 · Recovery
resolve|resolve crates/jk-tui/src/chrome.rs|Path: crates/jk-tui/src/chrome.rs;Tool: configured merge tool|Hand the conflicted file to jj's merge tool; inspect results on return.|P1 · Content
restore|restore --from @- --into @ docs/tui-design.md|From: @- qpvuntsm;Into: @ pvsouytl;Path: docs/tui-design.md|Replace this path's current content with its content from the parent.|P0 · Content
revert|revert -r qpvuntsm -o @|Revision to reverse: qpvuntsm;Onto: @ pvsouytl|Create a new change that applies the reverse of the selected change.|P2 · Advanced
root|root|Workspace: default|Print this workspace's root directory.|P2 · Advanced
run|run -r 'main..@' --ignore-changes cargo test|Revisions: main..@;Command: cargo test;Policy: ignore changes|Run tests across the revision range without rewriting commits.|P2 · Advanced
sign|sign -r qpvuntsm|Revision: qpvuntsm;Key: configured signing key|Sign this revision. The signing provider may require user interaction.|P3 · Tools
simplify-parents|simplify-parents -r @|Revision: @ pvsouytl|Remove redundant parent edges without discarding independent parents.|P2 · Advanced
sparse edit|sparse edit|Workspace: default;Editor: configured editor|Edit which paths are materialized in this workspace.|P2 · Advanced
sparse reset|sparse reset|Workspace: default|Materialize all tracked paths in this workspace.|P2 · Advanced
sparse set|sparse set --clear --add crates/jk-tui --add docs|Workspace: default;Included: crates/jk-tui, docs|Replace sparse patterns; other tracked content remains in history.|P2 · Advanced
split|split -r pvsouytl -m 'Add contextual action menu' crates/jk-tui/src/chrome.rs|Source: pvsouytl;Selected: crates/jk-tui/src/chrome.rs;Remaining: docs/tui-design.md|Put selected content into the first revision; remaining content stays in its child.|P0 · Content
squash|squash --from pvsouytl --into qpvuntsm --use-destination-message crates/jk-tui/src/chrome.rs|From: pvsouytl;Into: qpvuntsm;Path: crates/jk-tui/src/chrome.rs|Move selected file changes into the destination; keep its message.|P0 · Content
tag delete|tag delete v0.2.7|Tag: v0.2.7|Delete the local tag; review remote effects separately.|P1 · Share
tag set|tag set v0.2.8 -r @|Tag: v0.2.8;Target: @ pvsouytl|Create or set the local tag at this revision.|P1 · Share
tag track|tag track v0.2.7@origin|Remote tag: v0.2.7@origin|Start tracking this remote tag.|P1 · Share
tag untrack|tag untrack v0.2.7@origin|Remote tag: v0.2.7@origin|Stop tracking this remote tag.|P1 · Share
undo|undo|Operation: most recent undoable operation|Undo the most recent operation; review its scope before execution.|P0 · Recovery
unsign|unsign -r qpvuntsm|Revision: qpvuntsm|Drop the cryptographic signature from this revision.|P3 · Tools
util backend name|util backend name|Repository: /work/jk|Print the repository backend name.|P3 · Tools
util completion|util completion bash|Shell: bash|Generate shell completion text; do not run it automatically.|P3 · Tools
util config-schema|util config-schema|Format: JSON schema|Show the jj configuration schema as inspectable command output.|P3 · Tools
util exec|util exec -- cargo test|Program: cargo;Arguments: test|Delegate to jj's utility runner; show output and retain the exit status.|P3 · Tools
util gc|util gc|Repository: /work/jk|Garbage-collect repository data; inspect jj help before maintenance.|P3 · Tools
util install-man-pages|util install-man-pages /work/man|Destination: /work/man|Install generated manual pages into the chosen directory.|P3 · Tools
util markdown-help|util markdown-help|Output: Markdown|Generate CLI reference output for inspection or explicit copying.|P3 · Tools
util snapshot|util snapshot|Workspace: default|Snapshot working-copy changes and record the resulting operation.|P3 · Tools
version|version|Installed version: jj 0.45.1|Show jj version details.|P3 · Tools
workspace add|workspace add --name review-v2 -r main --sparse-patterns full /work/jk/review-v2|Destination: /work/jk/review-v2;Name: review-v2;Parent: main;Sparse: full|Create a sibling jj workspace with a new working-copy change.|P0 · Workspaces
workspace forget|workspace forget review|Workspace: review;Directory: /work/jk/review|Forget the workspace registration. Its directory is not deleted.|P0 · Workspaces
workspace rename|workspace rename review-v2|Current workspace: default;New name: review-v2|Rename the current workspace registration.|P0 · Workspaces
workspace root|workspace root|Workspace: default|Print the workspace root directory.|P0 · Workspaces
workspace update-stale|workspace update-stale|Workspace: default;Status: stale|Update this workspace to the repository's current operation.|P0 · Workspaces
help|help rebase|Command: rebase|Show the installed jj help, including supported arguments and options.|P3 · Tools
`.trim().split('\n').map(line=>{const [command,example,roles,effect,group]=line.split('|');return {command,example,roles:roles.split(';'),effect,group};});
const direct = {
log:'log',diff:'diff',show:'show',status:'status',evolog:'evolog',interdiff:'interdiff',
'file list':'file-list','file show':'file-show','file annotate':'annotate','file search':'file-search',
'operation log':'op-log','operation show':'op-show','operation diff':'op-diff',
'bookmark list':'bookmarks','tag list':'tags','git remote list':'remotes',
'workspace list':'workspaces','sparse list':'sparse'
};
const readOnly = new Set(['config get','config list','config path','git colocation status','git root','root','workspace root','util backend name','util completion','util config-schema','util markdown-help','version','help']);
const externalCommands = new Set(['arrange','bisect run','config edit','converge','diffedit','gerrit upload','resolve','sparse edit','sign','util exec','run']);
const commandTargets = {};
for (const item of inventory.commands) {
  const cmd=item.command;
  if(direct[cmd]) { commandTargets[cmd]=direct[cmd]; continue; }
  const d=commandDesigns.find(d=>d.command===cmd);
  if(!d) throw new Error('Missing command design: '+cmd);
  const id='cmd-'+cmd.replaceAll(' ','-');
  const read=readOnly.has(cmd);
  const external=externalCommands.has(cmd);
  const related=cmd.startsWith('bookmark')?'bookmarks':cmd.startsWith('tag')?'tags':cmd.startsWith('workspace')?'workspaces':cmd.startsWith('operation')?'op-log':cmd.startsWith('config')?'config':'log';
  const label=read?'Inspect':external?'Continue in tool':'Run '+cmd.split(' ').at(-1);
  add(id,cmd+' · '+(read?'command output':'review'),d.group,d.example,
    [...d.roles,'','Effect',d.effect,'',...(external?['Interactive tool','jk restores the terminal and resumes here when the tool exits.']:[])],
    {modal:!read,mode:read?'read-command':'preview',help:item.help,read,external,
     actions:[btn('Enter',label,read?'command-result':external?'tool-session':'command-result'),btn('O','Run options','run-options'),btn('Esc','Cancel',related)],
     note:'Illustrative operands. Command syntax is checked against the installed jj help; outcomes are not executed.'});
  commandTargets[cmd]=id;
}
add('command-result','Command output · reusable result','States · Execution','',[],{mode:'result',actions:[btn('Esc','Return','log'),btn('C','History','history')]});
add('tool-session','Interactive tool · return contract','States · Execution','',[
'Terminal handed to jj / configured tool', '',
'The external program owns its native controls while it runs.',
'jk does not draw over an editor, credential prompt, or jj arranger.', '',
'When it exits: retain output, refresh repository state, restore context.'],{actions:[btn('Enter','Show return to jk','command-result'),btn('Esc','Back to preview','log')]});
add('run-progress','Run across revisions · progress','P2 · Advanced',"run -r 'main..@' --ignore-changes cargo test",[
'Revision   Result      Command', 'mzrqpvnu   passed      cargo test', 'qpvuntsm   running     cargo test', 'pvsouytl   queued      cargo test', '',
'Policy: ignore changes · repository revisions are not rewritten', '',
'Output for qpvuntsm', 'running 12 tests', 'test keeps_focus_on_return ... ok'],{actions:[btn('Enter','Inspect output','history-detail'),btn('Esc','Show stop state','interrupted')]});
add('bisect-progress','Bisect · test feedback','P3 · Tools',"bisect run --range 'main..@' cargo test",[
'Range       main..@', 'Test        cargo test', '',
'Revision    Outcome', 'mzrqpvnu    good', 'qpvuntsm    bad', '',
'First bad revision: qpvuntsm 0f71aeb2', 'Keep focus when closing a preview'],{actions:[btn('Enter','Inspect revision','show'),btn('Esc','Back','log')]});
const journeys={
'Inspect a change':['log','expanded','show','diff','files','search','folded','help-diff'],
'Rebase a stack':['log','actions','rebase-target','rebase-preview','run-options','running','success','cmd-undo'],
'Split and squash':['status','content-picker','split-files','cmd-split','external-diff','squash-files','cmd-squash'],
'Restore and resolve':['status','restore-files','cmd-restore','conflicts','cmd-resolve','tool-session'],
'Share work':['bookmarks','cmd-bookmark-move','cmd-git-fetch','push-review','push-result','error'],
'Recover a mistake':['history','history-detail','op-log','op-show','op-diff','time-travel','cmd-operation-restore'],
'Work across workspaces':['workspaces','workspace-status','cmd-workspace-update-stale','cmd-workspace-add','sparse'],
'Inspect history deeply':['evolog','interdiff','file-list','file-show','annotate','file-search'],
'Configure and maintain':['config','cmd-config-get','cmd-config-set','remotes','cmd-git-remote-add','welcome','cmd-git-clone'],
'Advanced workflows':['cmd-converge','cmd-parallelize','cmd-arrange','cmd-run','run-progress','cmd-bisect-run','bisect-progress'],
'Adapt and recover':['narrow','help-log','empty','refresh','immutable','error','interrupted']
};
add('log-options','View options · graph','P0 · Controls','log',[
'Template       Configured jj template','Graph          Configured jj graph style','Limit          50','[ ] Reverse order','[ ] Show patch','[ ] Flat output (no graph)','',
'Changing the template is explicit; custom jj output remains available.'],{modal:true,actions:[btn('Enter','Choose template','template-picker'),btn('Esc','Cancel','log')]});
add('template-picker','Configured and custom templates','P0 · Controls','log',[
'(*) Configured template','( ) builtin_log_compact','( ) builtin_log_compact_full_description','( ) Custom expression…','',
'Current source  jj configuration','No template override is applied.'],{modal:true,selected:0,actions:[btn('Enter','Keep configured template','log'),btn('Esc','Back','log-options')]});
add('new-merge','New change · multiple parents','P0 · Change',"new qpvuntsm vtqsknru -m 'Combine UI and recovery docs'",[
'Parents · ordered marks','1  qpvuntsm  Keep focus when closing a preview','2  vtqsknru  Explain operation recovery','',
'Description  Combine UI and recovery docs','',
'Create a merge change with these two parents and edit it.','Inspect any content conflicts after creation.'],{modal:true,actions:[btn('Enter','Show sample result','command-result'),btn('Esc','Cancel','marks')]});
add('rebase-scope','Rebase · choose source semantics','P0 · Change','rebase -s qpvuntsm -o main',[
'(*) Source and descendants   -s qpvuntsm','( ) Selected revisions      -r qpvuntsm','( ) Whole branch            -b qpvuntsm','',
'Placement','(*) Onto destination        -o main','( ) Insert after            -A main','( ) Insert before           -B main','',
'Source includes pvsouytl. Selected revisions would relocate descendants.'],{modal:true,actions:[btn('Enter','Choose destination','rebase-target'),btn('Esc','Cancel','log')]});
add('external-command','External command mode','P1 · Tools','',[
'Program   cargo','Arguments test -p jk-tui','',
'Workspace default','Execution argv, without a shell','',
'External programs may change files independently of jj operations.',
'Use an explicit shell program when shell syntax is intended.'],{modal:true,input:{label:'External command',value:'cargo test -p jk-tui'},actions:[btn('Enter','Continue in terminal','tool-session'),btn('Esc','Cancel','log')]});
add('bookmark-conflict','Bookmark conflict · inspect targets','P0 · Share','bookmark list',[
'review-ui (conflicted):','  - qpvuntsm 591ef10c Keep focus when closing a preview','  + qpvuntsm 0f71aeb2 Keep focus when closing a preview','  + vtqsknru 70d290bf Explain operation recovery','',
'Inspect both added targets before setting an explicit resolution.'],{selected:2,actions:[btn('Enter','Inspect target','show'),btn('s','Set resolved target','cmd-bookmark-set'),btn('Esc','Back','bookmarks')]});
add('help-workspaces','Contextual help · workspaces','P0 · Controls','workspace list',[
'Open and inspect','  Enter             Inspect workspace','  s / d             Status / diff in selected workspace','',
'Workspace actions','  n                 Add workspace','  u                 Update stale workspace','  f                 Forget registration','',
'Move and return','  j/k, ↑/↓          Select workspace','  Esc               Return without changing app scope'],{base:'workspaces',modal:true,actions:[btn('Esc','Close help','workspaces')]});
add('help-operations','Contextual help · operation log','P0 · Controls','operation log',[
'Open and inspect','  Enter             Show selected operation','  d                 Compare operations','  l                 Graph at selected operation','  s                 Status at selected operation','',
'History and recovery','  r                 Preview restore to operation','  v                 Preview reversal of operation','',
'Move and return','  j/k, ↑/↓          Select operation','  Esc               Return to previous view'],{base:'op-log',modal:true,actions:[btn('Esc','Close help','op-log')]});
add('help-history','Contextual help · command history','P0 · Controls','',[
'Open and inspect','  Enter             Open command details','',
'Within command details','  Enter             Review replay against current state','  o                 Inspect resulting operation','',
'Move and return','  j/k, ↑/↓          Select history entry','  Esc               Return to previous view'],{base:'history',modal:true,actions:[btn('Esc','Close help','history')]});
add('description-review','Describe · before and after','P0 · Change',"describe qpvuntsm -m 'Keep focus when closing a preview'",[
'Change  qpvuntsm 0f71aeb2','', 'Before','  Fix preview focus','', 'After','  Keep focus when closing a preview','',
'File content is unchanged. The revision and descendants may be rewritten.'],{modal:true,actions:[btn('Enter','Show sample result','command-result'),btn('Esc','Edit description','describe-form')]});
add('hunk-picker','Partial content · external selector path','P1 · Content','squash --from pvsouytl --into qpvuntsm --interactive',[
'From pvsouytl → into qpvuntsm','', 'crates/jk-tui/src/chrome.rs','  Hunk 1   Restore selected revision after closing preview','  Hunk 2   Preserve scroll offset','',
'Partial-hunk selection takes place in the configured jj diff editor.',
'File-level selection stays available in jk.'],{actions:[btn('Enter','Continue in configured tool','tool-session'),btn('Esc','Back to files','squash-files')]});
// Routes remain explicit so later screenshots and tapes can refer to stable screen IDs.
journeys['Inspect a change'].push('log-options','template-picker');
journeys['Rebase a stack'].splice(2,0,'rebase-scope');
journeys['Split and squash'].push('hunk-picker');
journeys['Work across workspaces'].push('help-workspaces');
add('help-current','Contextual help · current view','P0 · Controls','',[],{modal:true,actions:[btn('Esc','Close help','log')]});

const stalePreview=screens.find(s=>s.id==='cmd-workspace-update-stale');
stalePreview.command='-R /work/jk/review workspace update-stale';
stalePreview.scope='review';
stalePreview.lines[0]='Workspace: review';
// Borderless confirmation inspired by the user's terminal reference.
const abandonDialog=screens.find(s=>s.id==='cmd-abandon');
abandonDialog.title='Abandon this change?';
abandonDialog.lines=[
'qpvuntsm  Keep focus when closing a preview', '',
'Changes to abandon (3 files)', '',
'D  README.md                                      +0  -3',
'M  crates/jk-tui/src/chrome.rs                      +1  -1',
'A  docs/tui-design.md                              +5  -0', '',
'Abandon removes this change and its edits from history.',
'1 descendant change will be rebased onto its parents.',
'Recover with Undo in the action menu.'
];
abandonDialog.buttons=[{label:'View diff',role:'secondary'},
 {label:'Cancel',role:'cancel',focused:true},{label:'Abandon change',role:'danger'}];
abandonDialog.actions=[btn('d','View diff','diff'),btn('Enter','Cancel','log'),
 btn('y','Abandon change','command-result'),btn('Esc','Cancel','log')];

abandonDialog.lines=abandonDialog.lines.map(line=>{
 const match=line.match(/^([DMA]) +(.+?) +(\+\d+) +(-\d+)$/);
 return match?`${match[1]}  ${match[2].padEnd(44)} ${match[3].padStart(3)} ${match[4].padStart(3)}`:line;
});
