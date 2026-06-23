# Log Template Selection

Status: draft

Owner: implementation spike

Scope: first log template chooser and startup-template implementation chunk

Planning update: the [CLI surface addendum](cli-surface-addendum.md) keeps this startup-template
work, but changes the long-term in-app key shape. `T` is reserved for the tags screen, standalone
`v` is reserved for `evolog`, and template switching should become a log-specific branch of the
reusable `V` View Options overlay. The current implementation has a temporary standalone `T`
popup because View Options did not exist yet; migrate that popup under `V` before turning the slice
into review-ready product work.

## Problem

`jk` currently has two log display mechanisms that solve different problems:

- the rendered pass preserves `jj`'s graph, colors, configured revset, and configured template;
- the semantic JSON pass uses a narrow `-T` template so up/down navigation can target changes.

That split is correct, but users cannot start `jk log` with a chosen rendered template or switch
between useful log detail levels in the TUI. Inline expansion helps for one selected change, but it
does not make the whole log easier to scan when a user wants commit messages or bodies visible
across the list.

## Goals

- Add `jk log -T TEMPLATE` as startup behavior for the rendered log pass.
- Preserve the semantic JSON pass used for navigation, selection, and inline expansion.
- Add log template choices under the reusable `V` View Options overlay once that overlay exists.
- Provide configured/default plus the working built-in jj log templates that are useful as full log
  views.
- Use `jj` templates for rendered log output, not locally rendered summaries.
- Preserve selection, scroll, and up/down navigation across template switches.
- Keep bare `jk` compatible with `jj`'s configured default command unless the user explicitly
  chooses a template or explicit log command.
- Support rendered-line scrolling for reading long multi-line log messages without moving the
  selected change.

## Non-Goals

- Do not add command mode.
- Do not add persistent template config.
- Do not add user-editable template management in the first chunk.
- Do not parse or render commit messages locally for the log body.
- Do not make bare `jk` override `jj ui.default-command`.

## Proposed Model

Represent the active rendered log shape as a small source-level option owned near `JjLog`:

```rust
pub enum LogTemplateSelection {
    Configured,
    Comfortable,
    Compact,
    CompactFullDescription,
    Detailed,
    Oneline,
    Redacted,
    Custom(String),
}
```

Suggested rendered behavior:

- `Configured` passes no rendered `-T` argument, so `jj` owns the configured log template.
- built-in variants pass working jj log aliases discovered from `jj log -T` hints:
  `builtin_log_comfortable`, `builtin_log_compact`, `builtin_log_compact_full_description`,
  `builtin_log_detailed`, `builtin_log_oneline`, and `builtin_log_redacted`.
- `Custom(template)` passes the user-supplied string from `jk log -T TEMPLATE`.

The semantic pass must remain independent:

- always run the existing JSON `LOG_TEMPLATE`;
- keep using the same revset, limit, repository, and command context as the rendered pass;
- never let a user-supplied rendered template replace the semantic JSON template.

## Startup Behavior

Add `-T/--template TEMPLATE` to `jk log`.

```text
jk log -T 'builtin_log_compact'
jk log --template 'description'
```

Rules:

- `jk log -T TEMPLATE` starts in the explicit `jj log` view.
- the rendered command is `jj log -T TEMPLATE`, plus existing `-n`, repository, color, and pager
  adapter behavior;
- the semantic command remains `jj log -T LOG_TEMPLATE`, plus the same `-n` and repository data;
- the title should include a compact rendered template source, such as
  `jj log -T builtin_log_compact_full_description`, truncating only when a custom template is too
  long for a narrow title;
- `r` refreshes with the same rendered template selection;
- `H` should return to bare configured home without a rendered template unless the implementation
  deliberately preserves an explicit selected template in source state;
- `L` should switch to explicit `jj log` with the current chooser selection only if the user has
  already chosen one in-app.

Bare `jk` compatibility:

- bare `jk` must still run bare `jj` for the rendered pass so `jj ui.default-command` remains the
  source of truth;
- bare `jk` should not silently pass `-T`, because that can force the default command to be log-like
  and break users whose configured default command is not plain `log`;
- when a user chooses a template in-app from bare `jk`, switch the source to explicit `jj log` before
  applying a rendered `-T` template.

## In-App Chooser

Use the reusable View Options overlay, not a status-bar cycle or a permanent standalone log-only
key:

- `Configured`: no rendered `-T`, current default for `jk log`;
- `Comfortable`: `builtin_log_comfortable`;
- `Compact`: `builtin_log_compact`;
- `Full description`: `builtin_log_compact_full_description`;
- `Detailed`: `builtin_log_detailed`;
- `Oneline`: `builtin_log_oneline`;
- `Redacted`: `builtin_log_redacted`;
- `Custom`: only present when startup used `jk log -T TEMPLATE`.

Interaction:

- `V`: open View Options while the log is active;
- choose the Template row to enter the log-template selector;
- `j/k` or up/down arrows: move the highlighted template row;
- Enter: apply the highlighted template and close the selector;
- Esc, Backspace, or `q`: close the selector without changing the template.

Collision notes:

- `T` is reserved for the tags screen in the harmonized keymap;
- lowercase `v` is reserved for standalone `jj evolog`;
- keep templates under `V` so log templates, graph/list flags, and diff display flags share one
  View Options model.
- the current `T` popup is acceptable only as a temporary precursor to `V` while this remains a
  local spike.

Visible help/hotbar:

- add or reuse the common `V options` binding row in `crates/jk-tui/src/keymap.rs`;
- the overlay can show a log-only `Template` row when the active view supports log templates;
- do not expose the binding in diff/show/status contexts.

## Multi-Line Template Navigation

Multi-line templates change the relationship between selected changes and rendered lines. Selection
movement should remain change-oriented:

- `j/k` and up/down arrows move between semantic changes;
- selection uses rendered commit-row positions assigned by the semantic JSON pass;
- when a selected entry's rendered message fits in the viewport, keep the whole selected message in
  view rather than placing the commit row on the bottom content line;
- when a selected entry is taller than the viewport, keep the selected commit row visible and show as
  much of its message body below it as possible;
- `Ctrl-j` and `Ctrl-k` scroll by one rendered line without changing the selected change, so users
  can read long message bodies or surrounding context.

## State And Refresh

Template switching should reload the rendered log, then refresh `LogState` with the new snapshot.
Selection preservation should continue to use change ids:

- remember the selected change id before reload;
- parse semantic entries from the unchanged JSON pass;
- assign rendered lines from the new rendered output;
- call the existing refresh path so selection is restored when the change still exists;
- clamp scroll after rendered-line positions change.

Important alignment contract:

- rendered commit-row count must still match the semantic entry count;
- if a detailed template adds body lines under each commit, `assign_rendered_lines` must still point
  entries at the correct commit rows;
- up/down and page movement must use rendered-line positions after template switches, not stale row
  indexes from the previous template.

## Implementation Chunks

### Chunk 1: startup `jk log -T`

Files:

- `crates/jk-cli/src/log.rs`
- `crates/jk/src/main.rs`

Acceptance:

- `jk log -T TEMPLATE` affects only the rendered log command.
- the semantic pass still uses `LOG_TEMPLATE`.
- `-n`, `--repository`, color forcing, and no-pager behavior are preserved.
- bare `jk` behavior is unchanged.
- refresh keeps the startup template.

### Chunk 2: template selection source state

Files:

- `crates/jk-cli/src/log.rs`
- `crates/jk/src/main.rs`
- `crates/jk-tui/src/log_view.rs`

Acceptance:

- `JjLog` can carry `Configured`, `Detailed`, or `Custom` rendered template selection.
- switching templates reloads the log and refreshes the active `LogView`.
- selection and scroll are preserved by change id when possible.
- failures leave the previous rendered log visible with an error status.

### Chunk 3: View Options integration and visible help

Files:

- `crates/jk/src/key.rs`
- `crates/jk-tui/src/keymap.rs`
- `crates/jk-tui/src/log_view.rs`

Acceptance:

- `V` opens View Options while log is active.
- the log View Options overlay includes a Template row.
- diff, show, status, and operation views keep using `V` for their own view options.
- `T` remains available for the tags screen.
- existing `H` and `L` source switching remains predictable.

## Tests

Required unit tests:

- rendered `JjLog` command includes `-T TEMPLATE` for `Custom`.
- semantic `JjLog` command still includes `-T LOG_TEMPLATE`, not the custom template.
- `Detailed` builds a native jj template argument on the rendered pass.
- bare configured default uses no rendered `-T`.
- `jk log -T TEMPLATE` parses into explicit log startup state.
- `V` maps to View Options while log is active.
- log help and hotbar include `V options`.
- refresh after a template switch preserves selected change id.
- rendered-line alignment still works when a rendered template inserts body lines between commits.
- page and up/down movement use updated rendered-line positions after switching templates.
- `Ctrl-j/k` line scrolling changes the viewport without changing the selected change.
- selected multi-line entries keep body context visible near the bottom of the viewport.
- the popup selector renders template rows, supports arrow and `j/k` movement, applies with Enter,
  and cancels without changing source state.

Suggested validation:

```sh
cargo test -p jk-cli log
cargo test -p jk
cargo test -p jk-tui log
```

Run Markdown lint for this plan:

```sh
just lint-md
```

## Betamax Evidence Expectations

Implementation should collect assertion-first TUI evidence:

- `jk log -T <template>` starts with the expected rendered message/body detail visible;
- up/down navigation highlights the correct commit rows in the custom template view;
- `Ctrl-j/k` scrolls the rendered log by line without changing the selected change;
- pressing `V` opens View Options with built-in jj log templates and the startup custom
  template when present;
- applying a selector choice switches templates and keeps navigation usable;
- `r` refreshes the active template selection;
- bare `jk` still follows `jj`'s configured default command when no template is chosen.

Prefer short validation tapes under `tapes/validation/`. Generate README or website media only after
the key and template names are stable.

## Risks

- Applying user templates to the semantic pass would break navigation. Keep rendered and semantic
  template arguments separate.
- Passing `-T` to bare `jj` can break `ui.default-command` compatibility. Only do it after explicit
  `jk log` startup or an in-app user choice that moves to explicit log semantics.
- A detailed template that changes graph commit-row shape can break rendered-line alignment. Test a
  body-oriented template with multiple rendered lines per change.
- Dumping very long custom template strings into titles or hotbars will make narrow terminals
  unreadable. Show built-in alias names directly, but compact custom template strings before putting
  them in the title.
- Some aliases reported by `jj log -T` are components rather than full log templates. Only include
  aliases that work as a complete rendered log template in the first selector.

## Follow-Up Chunks

- Add a searchable selector if the built-in set grows or named user templates become persistent.
- Add a command-mode route such as `:log -T ...` after command mode exists.
- Add persistent named template config only after real usage shows which presets matter.
- Consider a `--template-alias` or `--template-file` shape later; do not include it in this slice.
