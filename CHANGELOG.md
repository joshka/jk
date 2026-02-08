# Changelog

## [0.1.0] - 2026-02-08

### 🚀 Features

- *(tui)* Add runtime shell and jj runner
- *(config)* Add keybind and alias foundations
- *(alias)* Expand jj alias normalization
- *(tui)* Add guided command flow planner
- *(core)* Add top-level command registry
- *(flows)* Expand recovery and bookmark prompts
- *(log)* Stabilize revision selection on detail rows
- *(ux)* Add in-app command registry view
- *(confirm)* Add in-app danger preview scaffolding
- *(log)* Add metadata-backed row revision mapping
- *(help)* Add filtered command catalog lookups
- *(operation)* Add guided operation subcommand flows
- *(workspace)* Add guided workspace subcommand flows
- *(core)* Align guided flow coverage
- *(alias)* Support rebase destination overrides
- *(ux)* Expand safe shortcuts and command parsing
- *(view)* Improve status/show/diff presentation
- *(ux)* Add normal-mode help shortcut
- *(command)* Add command-mode history navigation
- *(ux)* Add repeat-last-command shortcut
- *(view)* Add native root output view
- *(ux)* Add nav keys and list wrappers
- *(ux)* Add quick read-mode shortcuts
- *(view)* Unify workspace-root presentation
- *(tui)* Improve discoverability and scanability
- *(view)* Refine status and operation summaries
- *(help)* Add local views to command registry
- *(flow)* Default file and tag list views
- *(flow)* Default list-first command groups
- *(ux)* Expand read-mode wrappers and shortcuts
- *(view)* Wrap mutation outputs
- *(view)* Add mutation summary heuristics
- *(alias)* Surface core jj default shorthands
- *(commands)* Annotate default aliases in registry
- *(view)* Prefer signal summaries in mutation wrappers
- *(view)* Add signal summaries for remote wrappers
- *(view)* Add native version output wrapper
- *(flow)* Lift absorb duplicate parallelize
- *(flow)* Lift interdiff evolog metaedit
- *(flow)* Lift simplify-parents flow
- *(flow)* Lift fix into guided rewrite flow
- *(view)* Lift resolve wrapper and guided mode
- *(flow)* Lift diffedit into guided defaults
- *(ux)* Add one-key return to log home view

### 🐛 Bug Fixes

- *(startup)* Route cli commands through flow planner
- *(alias)* Preserve explicit destinations and parity
- *(alias)* Canonicalize core jj shorthands
- *(view)* Align color output with jj styling
- *(view)* Reset ansi state per rendered row

### 🚜 Refactor

- *(app)* Split app into directory modules
- *(modules)* Split flow alias commands
- *(app)* Split runtime and view modules
- *(config)* Split config schema module
- *(app)* Slice input handling by mode

### 📚 Documentation

- *(plan)* Define implementation and quality workflow
- *(plan)* Track scope and execution status
- *(status)* Record startup regression coverage
- *(plan)* Refresh coverage trackers and ADRs
- *(project)* Tighten contributor and progress docs
- *(readme)* Add practical onboarding guide
- Complete greenfield documentation pass

### 🧪 Testing

- *(safety)* Harden command gating and alias fidelity
- *(startup)* Verify prompt and confirm entry paths
- *(ux)* Expand alias and wrapper coverage
- *(view)* Snapshot status wrapper output
- *(view)* Broaden mutation wrapper coverage
- *(view)* Add gold wrapper matrix
- *(view)* Snapshot mutation wrapper variants
- *(view)* Snapshot remaining bookmark mutations
- *(view)* Snapshot top-level mutation variants
- *(view)* Snapshot remaining top-level mutations
- *(view)* Add mutation wrapper routing matrix
- *(startup)* Cover core jj alias routing
- *(startup)* Cover high-frequency OMZ aliases

### ⚙️ Miscellaneous Tasks

- *(init)* Scaffold jk repository
- *(ci)* Add github actions and release audit
