# Ordered Revision Marks

Status: draft

Owner: implementation spike

Scope: first ordered revision mark implementation chunk

## Problem

`jk` now has the foundations needed for revision marks:

- `Ctrl-j` and `Ctrl-k` already scroll the log viewport independently from selected change state;
- searchable command discovery can expose keymap metadata by action and `jj` command family;
- pushed diff, show, and status views exist;
- the next inspection slice needs a stable way to resolve cursor plus marks into `jj` command
  shapes.

The product plan says object screens use `Space` to mark or unmark selected objects. The current log
implementation still treats `Space` as page down through the binary key adapter and visible keymap
metadata. That blocks ordered revision marks and would make later command resolution ambiguous.

This slice should add only the durable ordered mark model and visible log behavior. It should not
start graph search, revset filtering, mutation previews, or mark-aware diff/show/status resolution.

## Goals

- Add ordered revision marks to the log screen.
- Store marks by stable change id, not row index or rendered line.
- Preserve mark insertion order so later resolvers can infer source/from and destination/to roles.
- Prevent duplicate marks.
- Toggle the selected revision mark with `Space`.
- Add clear-marks behavior that is discoverable and does not steal future commit/create behavior
  when no marks exist.
- Preserve marks across refresh when marked change ids still exist.
- Drop marks for changes that disappear after refresh.
- Preserve the existing selection and scroll semantics, including independent `Ctrl-j/Ctrl-k`
  scrolling.
- Show marks in the log body with a small, stable visual affordance.
- Update hotbar, help, and searchable discovery metadata so `Space` and clear marks are findable.
- Keep the first slice implementation small enough for one medium worker.

## Non-Goals

- Do not implement mark-aware `d`, `S`, `Enter`, or `s` command resolution in this slice.
- Do not add mutation previews, command confirmation screens, or safe mutation role pickers.
- Do not add graph search, revset filter backtracking, or `/` search on the log.
- Do not add file, hunk, operation, bookmark, tag, or workspace marks.
- Do not add persistent marks across process restarts.
- Do not add mouse selection or mouse marking.
- Do not change `Space` paging behavior in diff/show/status text-reading screens.
- Do not replace jj-rendered graph output with native graph rendering.

## Current State

The current code shape is useful and should be extended rather than replaced:

- `crates/jk-tui/src/log_state.rs` owns selected entry, expanded change id, scroll offset,
  viewport height, and `follow_selection`.
- `LogState::refresh` already preserves selection by `LogEntry::change_id`.
- `LogState::scroll_previous_line` and `LogState::scroll_next_line` already implement
  independent `Ctrl-k` and `Ctrl-j` scrolling without changing selection.
- `crates/jk-tui/src/log_view.rs` exposes `selected_change_id` for follow-up inspection commands
  and paints the selected row after rendering the jj body.
- `crates/jk/src/key.rs` currently maps unmodified `Space` to `LogAction::PageNext`.
- `crates/jk-tui/src/keymap.rs` currently advertises `space / b, Ctrl-f/b` as page movement in
  log, diff, and inspection contexts.
- `crates/jk/src/main.rs` still maps selected-change `d` to `DiffQuery::Revision`, and
  `Enter`/`s` to selected show and repository status without consulting marks.

The first mark implementation should focus on log state, log rendering, visible key metadata, and
binary key dispatch. Later inspection work can then build resolver tests on a stable mark API.

## Data Model

Use change ids as the mark identity:

```rust
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct OrderedRevisionMarks {
    change_ids: Vec<String>,
}
```

The exact type name can change. The behavior should not:

- `change_ids` is ordered by first mark time.
- A change id appears at most once.
- Toggling an unmarked selected change appends its change id to the end.
- Toggling an already marked selected change removes that change id and closes the gap.
- Clearing marks empties the vector.
- Refresh retention filters the vector against the new visible `LogEntry` change ids while
  preserving the relative order of the survivors.

Keep the first type local to `crates/jk-tui/src/log_state.rs` unless a current caller needs it. Move
or re-export it from `jk-core` only when later command resolvers need to share the type across
crates. If it does move, `jk-core` is the right home because marks become part of the durable
cursor-plus-marks command model described in the product plan.

Recommended `LogState` additions:

```rust
marks: OrderedRevisionMarks,
```

Recommended methods:

```rust
pub fn toggle_selected_mark(&mut self);
pub fn clear_marks(&mut self) -> bool;
pub fn has_marks(&self) -> bool;
pub fn marked_change_ids(&self) -> &[String];
pub fn selected_mark_index(&self) -> Option<usize>;
pub fn mark_index_for_change_id(&self, change_id: &str) -> Option<usize>;
```

`clear_marks` should return whether anything changed so callers can decide whether `c` was consumed
as clear-marks or should remain available for a future screen-specific action.

Do not store mark row indexes. Row indexes and rendered lines change when revsets, templates,
hidden revisions, elision, or refresh output changes.

## UI Behavior

### Mark Toggle

On the log screen, unmodified `Space` toggles the selected revision mark:

- if no revision is selected, `Space` is a no-op;
- if the selected change is unmarked, append it to ordered marks;
- if the selected change is already marked, remove it;
- toggling a mark must not move selection;
- toggling a mark must not change `scroll_offset`;
- toggling a mark must not expand or collapse inline details.

This replaces log-screen `Space` page down. Keep `b`, `Ctrl-f`, `Ctrl-b`, `PageDown`, and `PageUp`
available for page movement. Diff, show, and status screens can keep `Space` as page down in this
slice because they are text-reading screens.

### Clear Marks

Use `c` to clear marks only when marks exist:

- with one or more marks, `c` clears all log marks and does not run any future commit/create action;
- with no marks, `c` should be ignored in this slice so future commit/create behavior remains
  available;
- clearing marks must not move selection or scroll;
- clearing marks should clear only revision marks in the active log, not future file/hunk marks.

`Esc` should close active overlays or prompt modes first, as it does today. Outside overlays,
`Esc` should clear marks before any future back/quit behavior only if that can be implemented
without regressing current `q`/`Esc` behavior. If the implementation would be noisy, keep `Esc`
unchanged in this slice and rely on `c` for explicit mark clearing.

`Backspace` remains view-stack back. It should not clear marks while a child view is active because
returning to the log should preserve the log state stored under the child view.

### Hotbar, Help, And Discovery

Update log-context visible key metadata:

- add `Space` as `mark/unmark revision`;
- add `c` as `clear marks` and make the help text clear that it applies when marks exist;
- remove `Space` from the log page hotbar/help row;
- keep page movement discoverable through `b`, `Ctrl-f`, `Ctrl-b`, `PageDown`, and `PageUp`;
- tag mark rows with a mark/navigation command family or alias so searchable discovery matches
  queries such as `mark`, `space`, `selected`, and `clear`.

The hotbar should remain compact. A likely log hotbar shape:

```text
? help  H home  L log  r refresh  space mark  c clear  enter show  d diff  s status  j/k move
```

Exact ordering can change during implementation. Prefer showing `space mark` when there is room
over advertising every page key.

Discovery rows should be generated from the same keymap metadata as current searchable help. Do not
add a second ad hoc discovery table.

### Visual Rendering

Add a log-body affordance that is visible but does not rewrite jj graph semantics. Requirements:

- marked rows show an ordered number, starting at `1`;
- selected row highlighting still wins for the selected row background;
- jj graph glyphs and rendered text remain intact;
- narrow terminals still render without panics or layout shifts;
- unmarked rows do not gain distracting filler.

Two acceptable first-slice render approaches:

- paint a tiny prefix or gutter in the content area before the jj-rendered line, such as `1` or
  `2`, similar to selected-row painting; or
- overlay a short suffix near the right edge for marked visible rows, such as `[1]`.

Prefer the approach that is least invasive to opaque jj output and easiest to test with the
existing `ratatui::backend::TestBackend`. Do not parse or rewrite the full rendered log body just
to insert marks into text. A paint overlay using `LogEntry::rendered_line` plus
`LogState::scroll_offset` is a good fit because selected-row painting already uses that pattern.

If a mark index exceeds one digit, render the full number only when it fits. Clipping a large mark
number is acceptable on narrow terminals; panicking or shifting the jj graph is not.

## Refresh Behavior

Refresh should preserve marks using change ids:

1. Capture the current ordered marks before replacing entries.
1. Load the new `LogSnapshot`.
1. Preserve selected change by the existing selected-change-id logic.
1. Filter ordered marks to change ids still present in the new entries.
1. Preserve survivor order exactly.
1. Drop marks for disappeared changes without status spam.
1. Preserve existing scroll behavior:
   - if the user was following selection, keep selection visible;
   - if the user independently scrolled with `Ctrl-j/Ctrl-k`, clamp `scroll_offset` but do not
     force it back to selected row;
   - if the selected change disappeared, use the existing fallback selection behavior.

Refreshing must not re-add duplicate marks, reorder survivors by graph order, or convert marks to
visible row indexes.

Changing log templates or switching home/log source should go through the same `LogState::refresh`
path so marks follow change ids when possible. If a template switch changes rendered-line positions,
mark overlays must use the new `LogEntry::rendered_line` values.

## Later Command-Resolution Implications

This slice should expose enough read-only mark state for later resolvers, but it should not change
`d`, `S`, `Enter`, or `s` command behavior yet.

The later resolver should treat cursor plus ordered marks as follows:

- no marks plus cursor `A`: `jj diff -r A`, `jj show A`;
- one mark `A` plus cursor `B`: `jj diff --from A --to B`; show can use cursor or
  prompt;
- two marks `A`, `B`: `jj diff --from A --to B`; `jj show A B` if requested;
- more than two marks: no silent diff guess; use action menu, prompt, or show multi-rev;
- contiguous marked range: later may offer a range revset such as `A::D` with visible command;
- non-contiguous marks: later must prompt or choose a command family that naturally accepts many
  revs.

Specific notes for later work:

- ordered mark roles are insertion-order roles, not graph-order roles;
- the cursor remains an implicit fallback destination when exactly one mark exists;
- if the cursor is also the only mark, the resolver should avoid building `--from A --to A` unless
  a command intentionally supports that shape;
- any mutation resolver must show a preview before running, per the safe mutation loop;
- read-only inspection may open directly, but title/status must show the resolved `jj` command.

Do not implement the resolver in this slice. The only command-facing requirement here is that the
mark API is not shaped in a way that prevents the table above.

## Implementation Chunks

### Chunk 1: Ordered mark state

Files:

- `crates/jk-tui/src/log_state.rs`
- `crates/jk-core/src/lib.rs` only if the mark type must be shared immediately

Acceptance:

- `LogState` can toggle the selected change mark.
- duplicate marks are impossible through the public state API.
- unmarking removes only the selected change id and preserves order of remaining marks.
- `clear_marks` empties marks and reports whether marks were present.
- refresh filters marks by new visible change ids while preserving survivor order.
- selection, expansion, and independent scroll tests still pass.

### Chunk 2: Log action contract and key dispatch

Files:

- `crates/jk-tui/src/log_view.rs`
- `crates/jk/src/key.rs`
- `crates/jk/src/main.rs`

Acceptance:

- add log actions for `ToggleMark` and `ClearMarks`;
- unmodified `Space` toggles marks while the active view is the log;
- `Space` still pages down in diff/show/status contexts;
- `c` clears marks when the active view is the log and marks exist;
- `c` is otherwise ignored in this slice;
- `Backspace` keeps existing view-stack behavior;
- overlay modes keep consuming printable keys according to their own input-mode handlers.

The current binary maps one `AppKey::Action(LogAction)` through every active view. If `Space`
needs different semantics by view, either:

- make `AppKey` represent page movement and mark toggling separately, then map by active view in
  `main.rs`; or
- add a log-only action and translate it to page movement only for diff/inspection views.

Choose the smaller change that keeps dispatch readable and does not make text-reading screens lose
space-as-page-down.

### Chunk 3: Rendering affordance

Files:

- `crates/jk-tui/src/log_view.rs`
- `crates/jk-tui/src/log_state.rs`
- possibly a small helper near `crates/jk-tui/src/selected_row.rs`

Acceptance:

- marked visible rows show ordered mark numbers.
- selected-row highlighting remains visually intact.
- hidden/offscreen marks do not render until their row is visible.
- changing selection does not change mark numbers.
- terminal widths too narrow for the mark overlay do not panic.

### Chunk 4: Help, hotbar, and discovery metadata

Files:

- `crates/jk-tui/src/keymap.rs`
- `crates/jk/src/key.rs` tests, if key labels and dispatch tests live there

Acceptance:

- log hotbar advertises `Space` as mark/unmark and no longer advertises `Space` as page down.
- log help includes mark and clear-marks rows.
- log discovery can find mark behavior by `mark`, `space`, and `clear`.
- diff and inspection help still show `Space` as page down.
- keymap snapshot tests are updated intentionally.

## Tests

Add state tests in `crates/jk-tui/src/log_state.rs`:

- `toggle_selected_mark_adds_change_id_in_order`;
- `toggle_selected_mark_removes_existing_mark`;
- `toggle_selected_mark_does_not_duplicate_change_id`;
- `clear_marks_empties_marks_and_reports_change`;
- `refresh_preserves_marks_when_change_ids_still_exist`;
- `refresh_drops_marks_for_disappeared_changes`;
- `line_scroll_and_mark_toggle_do_not_change_selection_or_scroll`.

Add view/render tests in `crates/jk-tui/src/log_view.rs`:

- marked rows render ordered affordances;
- selected marked row still has the selected background;
- clear marks removes visible affordances;
- narrow terminal rendering does not panic or wrap status text unexpectedly.

Add key and metadata tests:

- `Space` maps to mark behavior for log context;
- `Space` still pages text-reading views if dispatch is context-sensitive;
- `c` clears marks only when marks exist;
- log hotbar/help/discovery rows mention mark and clear;
- diff and inspection hotbar/help rows keep page behavior.

Suggested validation:

```sh
cargo test -p jk-tui log_state
cargo test -p jk-tui log_view
cargo test -p jk-tui keymap
cargo test -p jk key
```

Run broader tests if dispatch or public API changes are larger than expected:

```sh
cargo test -p jk-tui
cargo test -p jk
```

## Betamax Evidence Expectations

Add or update a log-mark validation tape after the implementation lands. The tape should use a
small deterministic repository with at least three visible revisions.

Minimum assertions:

- start in log with no marks;
- move to one revision and press `Space`;
- assert mark `1` is visible on that row;
- move to another revision and press `Space`;
- assert marks `1` and `2` preserve selection order;
- press `Ctrl-j` or `Ctrl-k`;
- assert viewport scroll changes while selection and mark order remain stable;
- press `Space` on a marked revision;
- assert that mark disappears and later marks close the numbering gap;
- press `c`;
- assert all mark affordances disappear;
- refresh with `r`;
- assert surviving marks remain if the tape keeps marks through refresh.

If a fixture can cheaply hide or remove a marked change before refresh, add one assertion that the
disappeared mark is dropped. Otherwise keep that as a state-test-only case.

Suggested tape name from the product plan:

```text
tapes/validation/log-marks.tape
```

Run the normal evidence command used by this repo, likely `just betamax-diff`, after adding or
updating the tape.

## Markdown And Validation

Before handing off the implementation PR:

```sh
just lint-md
cargo test -p jk-tui log_state
cargo test -p jk-tui log_view
cargo test -p jk-tui keymap
cargo test -p jk key
```

If this plan is edited without Rust changes, `just lint-md` is sufficient validation.

## Risks

- Reusing `Space` for both log marking and text paging can regress diff/show/status reading if
  dispatch is not context-aware.
- Overlaying mark affordances on opaque jj output can obscure graph glyphs or ANSI styles if the
  implementation rewrites text instead of painting cells carefully.
- Storing row indexes will break on refresh, template switches, hidden revisions, and graph
  elision.
- Reordering marks by graph order would break the source/from then destination/to mental model.
- Clearing marks on `Esc` may surprise users if it conflicts with overlay close or current quit
  behavior. Prefer explicit `c` unless the interaction is clear and tested.
- Moving mark types into `jk-core` too early can over-publicize a model that is still log-only.
  Keep it local until command resolution needs a shared type.
- Hotbar growth can clip more useful actions on narrow terminals. Keep labels short and let full
  help/discovery carry detail.

## Follow-Up Chunks

- Implement mark-aware diff/stat resolution from `0004-inspection-foundation.md`:
  `d`/`S` with no marks, one mark plus cursor, and two ordered marks.
- Extend show/details resolution so `Enter` can use cursor or ordered multi-revision marks
  deliberately.
- Add graph search and filter backtracking with mark preservation by change id.
- Add mark summaries to command titles/status lines once command specs own the resolver.
- Add file and hunk marks inside diff views after file/hunk selection semantics are stable.
- Add mutation preview role pickers for rebase, squash, split, restore, abandon, and absorb.
- Move ordered mark types to `jk-core` when they become shared by revision, file, operation,
  bookmark, or workspace command resolvers.
