# Community workflow signals

Status: bounded requirements evidence, recorded 2026-09-20. This note completes the unfinished
[jk: Distill Ratatui Discord require…][requirements-task] documentation pass. It introduces no new
implemented feature claims.

## Authority and scope

The accepted [TUI design contract](../tui-design.md), established in
[Design a coherent TUI UI strategy][design-task], owns jk's direction: familiar jj output, calm
borderless content, clear scope, and polish through alignment, spacing, hierarchy, and continuity.
Community examples below help choose validation cases. Isolated preferences do not override that
contract or establish community consensus.

This pass used the local Discrawl archive with automatic updates disabled. Its reported last sync
was 2026-09-20 07:41:09 UTC. Reads were restricted to Ratatui's known `#help` channel and threads
recorded as `thread_public` with `is_private_thread = 0`. No DMs, private/team-channel messages,
sync, archive export, or publication were used.

The cached guild is a Discord Desktop stub; several channels lack parent and permission metadata.
Public-thread classification is available, but this pass cannot independently verify current
effective Discord permissions or reconstruct every forum parent. Accordingly, sources name the
recorded thread channels without inventing a `#help` parent. Links require appropriate Discord access.
The selected examples span 2025-05-08 through 2026-05-19; they are historical reports, not claims
about current Ratatui behavior.

## Observed reports and planning implications

### Interaction tests need more than a screenshot

**Observed:** In the public thread channel *Approach for acceptance testing?*, on 2025-06-14 and
2025-06-15, a participant wanted to drive a sequence of user actions and inspect resulting state.
They identified output timing and coupling to the terminal backend as obstacles. Replies distinguished
rendering checks, input-to-action checks, application-state checks, and end-to-end flows.
[Initial question][acceptance-question] · [interaction requirement][acceptance-flow] ·
[testing layers][acceptance-layers].

**Inference for jk:** Keep focused state/layout assertions and fixture-based Betamax journeys
complementary. A PNG demonstrates appearance; semantic waits and terminal-state JSON demonstrate
what appeared at the checkpoint. Neither alone proves that a repository mutation had the intended
effect. Some testing advice came from jk's maintainer and is not independent demand evidence.

### Pending work must leave the interface responsive

**Observed:** In *Async example*, on 2025-06-25, a participant implementing an HTTP-backed action
wanted input and rendering to continue while waiting, including immediate application exit. Replies
called out resize events and deliberate handling of keyboard events during the pending action.
[Resize/input discussion][async-input] · [participant's exit requirement][async-exit].

**Inference for jk:** Slow refresh and preview work should preserve responsive navigation and a
visible cancellation path. Superseded results must not replace the current target's content. The
stale-result rule follows jk's design contract; this discussion does not itself document stale jj
results or prescribe a particular async architecture.

### Focus and input routing need one understandable owner

**Observed:** In *Focus and event captures*, on 2026-01-02, a participant described the burden of
managing focus and routing events through components. Replies offered different approaches rather
than a single established solution. [Question][focus-question] · [discussion][focus-discussion].

**Inference for jk:** Menus, selectors, text input, and the underlying graph need an explicit focus
contract. Printable input must not trigger graph actions; closing a transient surface should restore
the originating selection and focus. This evidence does not justify adopting a new UI framework.

### Captured output and terminal handoff are separate capabilities

**Observed:** In *Best way to display std::process::Command output in ratatui widgets*, on
2025-05-08, a participant reported command output obscuring surrounding chrome. A 2025-06-06 reply
distinguished displaying captured output from embedding a terminal. [Report][output-report] ·
[distinction][output-distinction]. In *Hey all, I’m working on running an*, on 2025-12-11,
participants discussed stopping event polling, leaving raw/alternate-screen mode for an editor, and
restoring the TUI. A reply questioned the example's assumptions; it was not a verified recipe.
[Handoff example][handoff-example] · [unresolved questions][handoff-questions].

**Inference for jk:** Review shell-free captured commands independently of interactive editor or
diff-editor support. Future handoff tests must exercise input ownership and terminal restoration on
both success and failure. Capturing stdout is insufficient evidence that an interactive tool works.

### Terminal geometry needs real-terminal proof

**Observed:** In *Resize messes up nerdfont icons on kitty?*, on 2026-05-19, a participant reported
icons changing apparent width after resize and a spacing workaround. This is one unverified report
about a particular terminal/font combination. [Report][resize-report].

**Inference for jk:** Use ordinary terminal-cell controls with no required patched font, and inspect
resize and constrained-width states in a real terminal. This supports a concrete edge case; it does
not establish user demand for a particular narrow-screen layout or diagnose a current library bug.

## Requirements and integration ownership

These priorities apply the existing design contract to the [workspace integration][coherence].
Source workspaces identify provenance; implementation status belongs in that integration ledger.

| Priority | Integration area and source workspace                                 | Acceptance criterion                                                                                                                             |
| -------- | --------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------ |
| P0       | Shared styling; `tui-design-guidelines`                               | At 80x24 and 60x16, scope and controls remain readable. Color-enabled selection preserves jj spans; PNG and state JSON agree.                    |
| P0       | Refresh; `work/cancellable-refresh`                                   | A delayed fixture remains navigable; cancellation returns to readable data; an older completion cannot overwrite the newest request.             |
| P0       | Selectors and menus; `work/shared-selectors`                          | Input reaches only the focused surface. Escape restores prior context. Moving a cursor never silently changes marked operands.                   |
| P0       | Mutations and refs; `work/workflows-integration`, `work/refs-remotes` | Preview names exact operands and scope. Cancel leaves fixture state unchanged. Success/failure records the result and preserves recovery access. |
| P0       | Proof across command families                                         | Each important journey uses a fresh repository, semantic waits, paired PNG/state checkpoints, and assertions on the resulting repository state.  |
| P1       | External tools; `work/external-command-mode`                          | Captured output is inspectable without losing context. Interactive handoff remains a separate claim requiring success/failure terminal tests.    |
| P1       | Run options; `work/run-options`                                       | Option editing preserves focus and operands; cancellation leaves the reviewed command unchanged; changed options require a current preview.      |

For cancellation, distinguish closing an unexecuted preview, stopping pending read work, and undoing
a completed mutation. These are different effects. A local undo must not be advertised as reversing
a remote push. These requirements come from jk's accepted command and recovery semantics, not from
the generic Ratatui examples.

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

Archive omissions, exact-word matching, and the public-only scope limit recall. Different preferences
about application architecture and the unresolved editor example remain visible uncertainties.
Do not convert missing search results into a claim that nobody has raised a topic. The accepted
design contract remains the authority for color fidelity, narrow layouts, and safe jj workflows.

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
