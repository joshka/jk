# Community workflow signals

Archive research recorded 2026-09-20 for
[jk: Distill Ratatui Discord require…][requirements-task]. These historical reports inform validation
priorities; implemented scope is recorded in the [coverage audit](../command-coverage.md).

## Authority and scope

The [TUI design contract](../tui-design.md), established in
[Design a coherent TUI UI strategy][design-task], sets jk's direction: familiar jj output, calm
borderless content, clear scope, and consistent interaction. The requirements below apply that
contract to individual reports; they are planning conclusions, not community consensus.

This pass used the local Discrawl archive with automatic updates disabled. Its reported last sync
was 2026-09-20 07:41:09 UTC. Reads were restricted to Ratatui's known `#help` channel and threads
recorded as `thread_public` with `is_private_thread = 0`. No DMs, private/team-channel messages,
sync, archive export, or publication were used.

The cached guild is a Discord Desktop stub; several channels lack parent and permission metadata.
Public-thread classification is available, but current effective permissions and some forum parents
could not be verified. Sources therefore use recorded thread names; links require Discord access.
The selected examples span 2025-05-08 through 2026-05-19. No current Ratatui behavior was verified.

## Reports and implications

### Interaction tests need more than a screenshot

In the public thread channel *Approach for acceptance testing?*, on 2025-06-14 and
2025-06-15, a participant wanted to drive a sequence of user actions and inspect resulting state.
They identified output timing and coupling to the terminal backend as obstacles. Replies distinguished
rendering checks, input-to-action checks, application-state checks, and end-to-end flows.
[Initial question][acceptance-question] · [interaction requirement][acceptance-flow] ·
[testing layers][acceptance-layers].

For jk, use focused state/layout assertions alongside fixture-based Betamax journeys. PNGs reveal
appearance; semantic waits and terminal-state JSON check checkpoint content. Repository assertions
must verify mutation effects. Some testing advice came from jk's maintainer, so it is not independent
demand evidence.

### Pending work must leave the interface responsive

In *Async example*, on 2025-06-25, a participant implementing an HTTP-backed action
wanted input and rendering to continue while waiting, including immediate application exit. Replies
called out resize events and deliberate handling of keyboard events during the pending action.
[Resize/input discussion][async-input] · [participant's exit requirement][async-exit].

Slow jk refreshes and previews should leave navigation and cancellation responsive. The design
contract also requires ignoring superseded results; the discussion concerns input during a pending
request and provides no evidence about stale jj results or a preferred async architecture.

### Focus determines where input goes

In *Focus and event captures*, on 2026-01-02, a participant described the burden of
managing focus and routing events through components. Replies offered different approaches rather
than a single established solution. [Question][focus-question] · [discussion][focus-discussion].

jk's menus, selectors, text input, and graph need an explicit focus contract. Printable input must
not trigger graph actions, and closing a transient surface should restore the originating selection
and focus. Choosing a UI framework is outside this requirement.

### Captured output and terminal handoff are separate capabilities

In *Best way to display std::process::Command output in ratatui widgets*, on
2025-05-08, a participant reported command output obscuring surrounding chrome. A 2025-06-06 reply
distinguished displaying captured output from embedding a terminal. [Report][output-report] ·
[distinction][output-distinction]. In *Hey all, I’m working on running an*, on 2025-12-11,
participants discussed stopping event polling, leaving raw/alternate-screen mode for an editor, and
restoring the TUI. A reply questioned the example's assumptions; it was not a verified recipe.
[Handoff example][handoff-example] · [unresolved questions][handoff-questions].

Captured commands and interactive editor support need separate validation. Handoff tests must check
input ownership and terminal restoration after both success and failure; stdout capture alone cannot
exercise those conditions.

### Terminal geometry needs real-terminal proof

In *Resize messes up nerdfont icons on kitty?*, on 2026-05-19, a participant reported
icons changing apparent width after resize and a spacing workaround. This is one unverified report
about a particular terminal/font combination. [Report][resize-report].

Include that resize case in terminal inspection, and keep essential jk controls usable without a
patched font. The report leaves the cause unresolved and says nothing about a preferred narrow-screen
layout.

## Requirements and integration ownership

The [workspace integration][coherence] records implementation status and recovered changes from the
source workspaces below.

| Priority | Integration area and source workspace                                 | Acceptance criterion                                                                                                                             |
| -------- | --------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------ |
| P0       | Shared styling; `tui-design-guidelines`                               | At 80x24 and 60x16, scope and controls remain readable. Color-enabled selection preserves jj spans; PNG and state JSON agree.                    |
| P0       | Refresh; `work/cancellable-refresh`                                   | A delayed fixture remains navigable; cancellation returns to readable data; an older completion cannot overwrite the newest request.             |
| P0       | Selectors and menus; `work/shared-selectors`                          | Input reaches only the focused surface. Escape restores prior context. Moving a cursor never silently changes marked operands.                   |
| P0       | Mutations and refs; `work/workflows-integration`, `work/refs-remotes` | Preview names exact operands and scope. Cancel leaves fixture state unchanged. Success/failure records the result and preserves recovery access. |
| P0       | Proof across command families                                         | Each important journey uses a fresh repository, semantic waits, paired PNG/state checkpoints, and assertions on the resulting repository state.  |
| P1       | External tools; `work/external-command-mode`                          | Captured output is inspectable without losing context. Interactive handoff remains a separate claim requiring success/failure terminal tests.    |
| P1       | Run options; `work/run-options`                                       | Option editing preserves focus and operands; cancellation leaves the reviewed command unchanged; changed options require a current preview.      |

Cancellation closes an unexecuted preview or stops pending work; undo recovers from a completed local
mutation. A local undo cannot reverse a remote push. Keep these distinctions from jk's command and
recovery contract when applying the generic Ratatui examples.

## Verified local context and limits

Source inspection during this pass confirmed a `TerminalRestore` drop guard in
[`crates/jk/src/main.rs`](../../crates/jk/src/main.rs) and a recording command-runner wrapper in
[`crates/jk/src/runner.rs`](../../crates/jk/src/runner.rs). The release-smoke tape launches an absolute
binary against a disposable fixture supplied by `scripts/betamax-setup.sh`. These are source
observations, not newly executed behavioral validation or proof of interactive child-process handoff.

Initial channel-scoped FTS queries in `#help` returned no messages for layout, terminal handoff,
testing, or selection/cancellation terms. Discovery then used recorded public-thread names and
read-only FTS joins constrained to public threads. Searches covered:

- `narrow`, `resize`, `small terminal`, and `terminal size`;
- `external`, `editor`, `suspend`, and `restore` together with `terminal`;
- `snapshot`, `test`, `testing`, focus, and the acceptance-testing thread;
- `cancel`, `cancellation`, `stale`, `highlight`, and `color scheme`;
- `command palette`, `picker`, `rebase`, `bookmark`, and `command history`.

Within that restricted index, the last group returned zero matches. Selection-color phrases and
`stale OR cancellation` also returned zero. The one `narrow OR "small terminal"` match used
"narrow" in an unrelated sense. The Unicode-coloring thread had metadata but no retrievable messages.
No direct requirement for jj mutation safety, bookmark/remotes behavior, a palette, or preserving jj
foregrounds was established by this sample. Keyboard exit during pending work had direct evidence;
specific process-cancellation semantics did not.

Archive omissions, exact-word matching, and the public-only scope limit recall; missing results do
not mean nobody has raised a topic. Architecture preferences varied, and the editor example remained
unresolved. Color fidelity, narrow layouts, and safe jj workflows remain project requirements from
the accepted design contract.

[requirements-task]: codex://threads/019fdfb7-3346-7c43-b739-b272da276966
[design-task]: codex://threads/01a0b680-89a8-7ee2-8840-db4d9533e8d9
[coherence]: ../workspace-coherence.md
[acceptance-question]: https://discord.com/channels/1070692720437383208/1383430824464224436/1383430824464224436
[acceptance-flow]: https://discord.com/channels/1070692720437383208/1383430824464224436/1383520064161972265
[acceptance-layers]: https://discord.com/channels/1070692720437383208/1383430824464224436/1383587460474339460
[async-input]: https://discord.com/channels/1070692720437383208/1387573587686199308/1387575143307677808
[async-exit]: https://discord.com/channels/1070692720437383208/1387573587686199308/1387575251113607239
[focus-question]: https://discord.com/channels/1070692720437383208/1456593313103741020/1456593313103741020
[focus-discussion]: https://discord.com/channels/1070692720437383208/1456593313103741020/1456594941613572186
[output-report]: https://discord.com/channels/1070692720437383208/1370180951753359401/1370180951753359401
[output-distinction]: https://discord.com/channels/1070692720437383208/1370180951753359401/1380626262917382206
[handoff-example]: https://discord.com/channels/1070692720437383208/1448424343162978537/1448613614083637390
[handoff-questions]: https://discord.com/channels/1070692720437383208/1448424343162978537/1448800502102884513
[resize-report]: https://discord.com/channels/1070692720437383208/1506096382937923615/1506096382937923615
