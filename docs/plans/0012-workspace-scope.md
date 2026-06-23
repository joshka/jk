# Workspace Scope Slice

Status: draft/spec

Owner: implementation spike

Scope: first early workspace implementation slice

## Decision

Add `W` as a read-only Workspaces screen first. The smallest useful slice should list jj
workspaces, mark the current workspace, preserve workspace selection, and open status or diff for
the selected workspace through the existing rendered inspection stack.

Do not include `workspace update-stale` in the first implementation chunk. It belongs immediately
after the read-only screen, but it is still a mutating workspace operation. The first chunk should
prove the provider, list parsing, command specs, screen state, keymap context, view-stack
integration, fixture shape, and Betamax evidence before adding mutation refresh behavior.

The first chunk should therefore implement:

- `W` from the log and rendered inspection contexts opens Workspaces.
- Workspaces screen renders `jj workspace list` data with a current-workspace marker.
- `s` opens `jj status` scoped to the selected workspace.
- `d` opens `jj diff` scoped to the selected workspace.
- `r` refreshes the workspace list.
- `?` shows generated workspace help and searchable discovery rows.
- `Backspace`, `Esc`, `H`, or `L` returns to the previous view.

The next chunk should add `u` for `jj workspace update-stale` only after the read-only screen is
stable. That follow-up should reuse the same provider and selection model, run the mutating command
through `JjCommandSpec`, refresh the list after success, preserve output on failure, and keep the
selected workspace visible where possible.

## Context

The roadmap keeps workspaces in early core scope:

- 0.5 adds `W` workspace screen backed by `jj workspace list`.
- Acceptance includes listing workspaces, identifying the current workspace, inspecting
  status/diff, updating stale workspaces, adding and forgetting later.
- Tests include a `multi-workspace-repo` fixture and docs workspace media tape.

The product plan makes `jj workspace` a P0 dedicated screen with list, add, forget, root, and
update-stale. Step 10 says `W` opens `jj workspace list`, then adds status, diff, update-stale,
forget confirmation flows, fixtures, and tapes.

The CLI surface addendum assigns scoped workspace keys:

- `n`: add/new workspace;
- `r`: rename current workspace;
- `f`: forget workspace, confirmed;
- `u`: update stale;
- `s`: status for workspace;
- `d`: diff for workspace;
- `o`: operation log;
- `V`: view/sparse options.

This slice assumes the current implementation foundation:

1. `JjCommandSpec` and `GlobalOptions` exist in `jk-core`.
1. Read-only inspection providers exist for status, diff, show, and evolog.
1. `RenderedView` and `InspectionSnapshot` provide rendered output, search, refresh, retry, and
   error display.
1. `ViewStack` owns root and child views.
1. Help, hotbar, and searchable command discovery are data-backed.
1. `V` View Options exists as a placeholder where a context has no implemented options.
1. The adaptive hotbar can omit lower-priority labels without hiding commands from `?`.

## Why Update-Stale Is Next, Not First

`jj workspace update-stale` is important because stale workspaces are one of the reasons a
workspace screen exists. It should not be delayed behind command mode, broad mutation preview, or
workspace add/forget flows.

It should still be the second workspace chunk for four reasons:

1. The first chunk can deliver daily value without mutation by answering "what workspaces exist?"
   and "what is the status/diff for that workspace?"
1. The list screen must define stale-state display and selection preservation before any stale
   update can show a trustworthy result.
1. `update-stale` changes workspace state and needs success/failure output handling that should be
   informed by the first workspace provider and refresh contracts.
1. The first chunk can create the multi-workspace and stale-workspace fixture needed to validate
   both chunks.

The implementation order should be:

1. Read-only Workspaces screen, current marker, root/path metadata, status, diff, refresh.
1. `u update stale` for the selected stale workspace, with post-command refresh and failure output.
1. Forget confirmation.
1. Add/rename workspace flows.
1. Sparse options and workspace creation sparse-pattern controls.

## Goals

- Add a focused Workspaces screen reachable through `W`.
- Preserve command fidelity by constructing every jj command through `JjCommandSpec`.
- Reuse `GlobalOptions` for repository and output behavior instead of hand-building argv.
- Keep workspace data in a typed provider model, not parsed out of rendered table text.
- Show the current workspace distinctly.
- Show enough root/path/change/stale information to support follow-up actions.
- Open selected-workspace status and diff as ordinary rendered inspection child views.
- Preserve workspace selection across refreshes when the selected workspace still exists.
- Provide generated help, searchable discovery, and an adaptive hotbar for workspace actions.
- Add tests and Betamax evidence under `target/jk-artifacts`.

## Non-Goals

- Do not implement `jj workspace add`.
- Do not implement `jj workspace forget` or its confirmation flow.
- Do not implement `jj workspace rename`.
- Do not implement `jj workspace update-stale` in the first chunk.
- Do not implement sparse-pattern editing, sparse list/reset/set, or workspace creation sparse
  options.
- Do not add operation log from Workspaces yet.
- Do not add command mode or mutation preview.
- Do not switch the process working directory as an implicit cross-workspace mutation.
- Do not add website updates or regenerate public media in this slice.
- Do not replace jj-rendered status/diff output with native rendering.

## Command Shape

The workspace provider should build command specs first and execute those specs second.

List:

```text
jj workspace list --template TEMPLATE
```

Root for a named workspace:

```text
jj workspace root --name NAME
```

Status for a selected workspace:

```text
jj -R WORKSPACE_ROOT status
```

Diff for a selected workspace:

```text
jj -R WORKSPACE_ROOT diff
```

Rendered process argv still includes the existing adapter-level flags, for example:

```text
jj --no-pager --color always -R WORKSPACE_ROOT status
```

Titles should use command-family previews:

```text
jj workspace list
jj status -R WORKSPACE_ROOT
jj diff -R WORKSPACE_ROOT
```

If the display title format for existing inspection views always puts global options before the
command family, keep that current convention. The important contract is that titles come from the
same command spec used for execution.

## Workspace List Data

Use a deterministic `jj workspace list --template ...` form rather than parsing the user's default
human template. The template should emit one machine-readable record per workspace with fields that
the screen can render and tests can assert.

The exact template syntax should be verified during implementation against the installed `jj`
version. The target data model is:

```rust
pub struct WorkspaceListSnapshot {
    pub workspaces: Vec<WorkspaceSummary>,
    pub current: Option<String>,
    pub title: String,
}

pub struct WorkspaceSummary {
    pub name: String,
    pub root: Option<PathBuf>,
    pub current: bool,
    pub stale: WorkspaceStaleState,
    pub change_id: Option<String>,
    pub commit_id: Option<String>,
    pub raw_status: Option<String>,
}

pub enum WorkspaceStaleState {
    Current,
    Stale,
    Unknown,
}
```

Do not require every field to be perfect before the first screen can land. The first reviewable
implementation may ship with `Unknown` stale state or missing change/commit fields if the installed
template surface does not expose them directly. The provider should still keep typed optional fields
so later chunks do not need to replace the screen model.

Root paths can be loaded in one of two ways:

1. Prefer a list template field if `WorkspaceRef` exposes the root path in the installed jj.
1. Otherwise, run `jj workspace root --name NAME` per visible workspace and cache the result in the
   snapshot.

If roots are loaded by per-workspace commands, keep the first implementation simple and sequential.
Avoid background fan-out until there is a cancellable task model for provider refreshes.

## Provider Boundary

Add a workspace provider alongside the existing jj process integration types. Keep it in `jk-cli`
unless the implementation needs to share query types across crates immediately.

Suggested shape:

```rust
pub struct WorkspaceListQuery;

pub struct WorkspaceStatusQuery {
    pub name: String,
    pub root: PathBuf,
}

pub struct WorkspaceDiffQuery {
    pub name: String,
    pub root: PathBuf,
}

pub struct JjWorkspaces {
    repository: Option<PathBuf>,
}

impl JjWorkspaces {
    pub fn with_repository(self, repository: impl Into<PathBuf>) -> Self;
    pub fn load_list(&self) -> Result<WorkspaceListSnapshot, JjWorkspacesError>;
    pub fn load_status(
        &self,
        query: &WorkspaceStatusQuery,
    ) -> Result<InspectionSnapshot, JjWorkspacesError>;
    pub fn load_diff(
        &self,
        query: &WorkspaceDiffQuery,
    ) -> Result<InspectionSnapshot, JjWorkspacesError>;
    pub fn list_spec(&self) -> JjCommandSpec;
    pub fn status_spec(&self, query: &WorkspaceStatusQuery) -> JjCommandSpec;
    pub fn diff_spec(&self, query: &WorkspaceDiffQuery) -> JjCommandSpec;
}
```

The provider should:

- build specs before running commands;
- apply `GlobalOptions` consistently;
- pass `-R WORKSPACE_ROOT` for selected-workspace status and diff;
- return structured errors for spawn failures, command failures, parse failures, and missing roots;
- include stderr from failed jj commands in the rendered error surface;
- never infer workspace roots from string suffixes in the visible table.

## Screen State

Add a focused screen state rather than a permanent pane:

```rust
pub struct WorkspacesView {
    snapshot: WorkspaceListSnapshot,
    selected: usize,
    scroll: usize,
    last_error: Option<String>,
}
```

Selection rules:

- Initial selection should prefer the current workspace.
- If there is no current marker, select the first workspace.
- Refresh should preserve selection by workspace name when possible.
- If the selected workspace disappeared, select the current workspace.
- If there is no current workspace, clamp to the nearest valid row.
- Empty lists should not panic and should show an empty state.

Scrolling should match the log screen pattern:

- `j/k` or arrows move selection.
- `Ctrl-j/Ctrl-k` scroll without changing selection if the screen already has independent scroll
  helpers available.
- `g/G` or `Home/End` may be omitted until there is an obvious shared list-navigation helper.

## View-Stack Behavior

`W` should push a Workspaces screen on top of the current view. It should be available from:

- log;
- diff;
- show;
- status;
- evolog;
- future operation views.

Returning from Workspaces should reveal the previous view without reloading it. Returning from a
workspace status or diff child view should reveal Workspaces with the same selection and scroll.

Stack examples:

```text
Log
  -> Workspaces
      -> Workspace status
      -> Workspace diff
```

```text
Show
  -> Workspaces
      -> Workspace diff
```

`Backspace`, `Esc`, `H`, and `L` should follow the existing local conventions for returning to the
previous view. If `H`/`L` currently have special root navigation behavior, keep that behavior and
only add Workspaces-specific return handling where it is already consistent.

## Keymap, Hotbar, And Discovery

Add a `BindingContext::Workspaces` context.

First-slice bindings:

| Key             | Action                              |
| --------------- | ----------------------------------- |
| `j/k`           | Move workspace selection.           |
| `Ctrl-j/Ctrl-k` | Scroll workspace list.              |
| `Enter`         | Open status for selected workspace. |
| `s`             | Open status for selected workspace. |
| `d`             | Open diff for selected workspace.   |
| `r`             | Refresh workspace list.             |
| `V`             | Open View Options placeholder.      |
| `?`             | Open searchable help.               |
| `Backspace`     | Return to previous view.            |
| `Esc`           | Return to previous view.            |
| `q`             | Quit.                               |

Reserve but do not implement:

- `n add`;
- `r rename current workspace`;
- `f forget workspace`;
- `u update stale`;
- `o op log`.

The first slice has a conflict with the eventual `r rename current workspace` addendum key. Use
`r refresh` for now because every current screen uses `r` for refresh and because `rename` is out
of scope. When rename lands, the workspace screen should move refresh behind a consistent
refresh/help affordance or an action menu instead of keeping two meanings for `r`.

Suggested first hotbar:

```text
? help  s status  d diff  r refresh  V options  j/k move  q quit
```

`Enter` can be discoverable in help without appearing in the hotbar because `s status` names the
same action more clearly. `u update stale`, `f forget`, and `n add` should not appear in the hotbar
until implemented. Searchable discovery can include reserved future rows only if they are clearly
marked as unavailable; the safer first slice is to omit unimplemented actions from discovery.

Add command-family metadata for:

- `jj workspace list`;
- `jj workspace root`;
- `jj status`;
- `jj diff`;
- View Options;
- help;
- navigation;
- quit.

## View Options

`V` should open the existing placeholder in the first chunk:

```text
View Options

No workspace view options in this slice.
```

Do not add sparse controls here yet. The key is reserved because sparse options belong near
workspaces eventually, but first-slice `V` exists to keep the global screen model consistent.

## Rendering

The Workspaces screen should be a focused list with no permanent split pane. Rows should make these
facts scannable:

- workspace name;
- current marker;
- stale marker when known;
- root path when available;
- working-copy change or commit when available.

Use concise markers and avoid dense prose in row text. A possible row shape:

```text
> * feature    current   /path/to/repo/feature
    default    stale     /path/to/repo/default
```

The title should make the command source clear:

```text
jj workspace list
```

If the provider cannot determine roots in the first slice, render a clear placeholder such as
`root unknown` instead of hiding the missing field. Status and diff actions should be disabled for a
workspace whose root is unknown, with a short error message.

## Status And Diff Behavior

`s` and `Enter` should open a selected-workspace status view. `d` should open a selected-workspace
diff view.

These child views should use the existing rendered inspection behavior:

- search;
- scroll;
- refresh;
- retry on failed load;
- error rendering;
- `Backspace` return.

Refresh inside the child status or diff view should rerun the same selected-workspace command
against the same root path. It should not reselect a different workspace just because the workspace
list changed in the background. Returning to Workspaces and pressing `r` should refresh the list.

If the selected workspace root no longer exists, status/diff should open an error inspection view
with the failed command title and jj stderr or provider error text. Do not silently fall back to the
current process workspace.

## Error And Empty-State Behavior

Provider errors should be visible and recoverable:

- If `jj workspace list` fails on initial load, open Workspaces with an error body and a retry
  affordance.
- If refresh fails after a successful load, keep the previous list visible and show the error in a
  status/error area.
- If parsing fails, show the raw command title and a concise parse error; keep raw output available
  in logs or test artifacts if practical.
- If no workspaces are returned, show an empty state with `r refresh`, `? help`, and `Backspace`
  available.
- If roots cannot be resolved, disable status/diff for those rows and explain that the workspace
  root is unavailable.

Do not panic on:

- empty output;
- duplicate workspace names in malformed output;
- missing current marker;
- stale workspace root;
- deleted workspace directory;
- non-UTF-8 paths. Use lossy display only at the UI boundary.

## Acceptance

- Pressing `W` opens a focused Workspaces screen.
- The screen title is derived from the `jj workspace list` command spec.
- The current workspace is visibly marked.
- Selection starts on the current workspace when known.
- Refresh preserves selection by workspace name.
- `s` and `Enter` open selected-workspace status as a rendered inspection child view.
- `d` opens selected-workspace diff as a rendered inspection child view.
- Status and diff commands are scoped to the selected workspace root, not the process cwd by
  accident.
- `Backspace` from status/diff returns to Workspaces with selection preserved.
- `Backspace` or `Esc` from Workspaces returns to the previous view.
- Help, hotbar, and command discovery include implemented workspace actions.
- Unimplemented add, forget, rename, update-stale, op-log, and sparse actions are not advertised as
  available commands.
- Empty, failed, and root-unknown states are rendered without panics.
- No mutation is performed in the first chunk.

## Tests

Add focused unit tests before relying on Betamax evidence.

Provider tests:

- `workspace list` spec renders the expected command family and preserves global options.
- selected-workspace status spec uses `-R WORKSPACE_ROOT status`.
- selected-workspace diff spec uses `-R WORKSPACE_ROOT diff`.
- parser handles one current workspace.
- parser handles multiple workspaces.
- parser handles stale or unknown stale state.
- parser reports malformed records with a useful error.
- root resolution error disables status/diff for that row.

State tests:

- initial selection chooses the current workspace.
- refresh preserves selection by name.
- refresh clamps selection after deletion.
- empty list keeps navigation and return actions safe.
- status/diff child views return to Workspaces with selection preserved.
- failed refresh keeps the previous snapshot.

Keymap tests:

- `BindingContext::Workspaces` help contains implemented workspace actions.
- workspace hotbar fits at normal Betamax width through `adaptive_hotbar`.
- searchable discovery finds `workspace`, `jj workspace`, `status`, and `diff`.
- unimplemented `update stale`, `forget`, `rename`, and `add` are not advertised as available.

Regression tests:

- existing log, diff, show, status, evolog, View Options, and help flows still render their current
  hotbar/discovery rows.
- `W` can be opened from log and rendered inspection contexts.

## Multi-Workspace Fixture

Create a deterministic fixture script for workspace tests, likely under the existing fixture
pattern:

```text
multi-workspace-repo
  main workspace
  second workspace
  one committed file
  one workspace-local working-copy edit
  optional stale workspace state for the follow-up update-stale chunk
```

The fixture should create real jj workspaces rather than fake output. It should also write a small
README or fixture note explaining which workspace is current, which workspace has changes, and
which workspace is intended to become stale for the second chunk.

The first chunk should not require the stale state if making it deterministic is too expensive.
Instead, create the fixture layout so the next `update-stale` slice can extend it without replacing
the fixture.

## Betamax Evidence Plan

Store first-slice evidence under `target/jk-artifacts/` so the spike has durable local proof
without committing generated media to the repo.

Suggested artifacts:

```text
target/jk-artifacts/workspaces-list.txt
target/jk-artifacts/workspaces-status.txt
target/jk-artifacts/workspaces-diff.txt
target/jk-artifacts/workspaces-help.txt
target/jk-artifacts/workspaces-empty-or-error.txt
```

Suggested tapes:

- `workspace-list.tape`: open `jk`, press `W`, assert the Workspaces title, current marker, and at
  least two workspace rows.
- `workspace-status.tape`: from Workspaces, select a non-current workspace when present, press `s`,
  assert a `jj status` title scoped to that workspace.
- `workspace-diff.tape`: from Workspaces, press `d`, assert a `jj diff` title scoped to that
  workspace.
- `workspace-help.tape`: press `?` on Workspaces and assert implemented actions are discoverable.
- `docs-workspaces.tape`: later docs-facing media once behavior is stable enough for README or site
  copy.

The first implementation PR does not need website updates. It should still leave artifacts with
enough evidence that a later docs/media pass can reuse the workflow.

## Follow-Up Slices

### Slice 2: Update Stale

Add `u` for selected-workspace `jj workspace update-stale`.

Acceptance:

- `u` is visible only when implemented.
- stale workspaces are visually identifiable when jj exposes stale state.
- successful update refreshes the workspace list.
- failure preserves command output and keeps the user on Workspaces.
- selection remains on the updated workspace when it still exists.

### Slice 3: Forget Confirmation

Add `f` for `jj workspace forget` with an explicit confirmation message that forgetting does not
delete files on disk.

### Slice 4: Add And Rename

Add `n` and `r` flows after mutation confirmation and command preview patterns are mature enough
for path/name input.

### Slice 5: Sparse Options

Move `V` beyond the placeholder by adding sparse list/edit/reset/set visibility and eventually
workspace creation sparse-pattern choices.
