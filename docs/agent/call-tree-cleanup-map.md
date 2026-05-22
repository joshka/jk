# Call Tree Cleanup Map

This map turns the cleanup goal into a concrete traversal tree that follows the
runtime path from `main()` into app startup, dispatch, action flow, view
dispatch, and downstream feature owners.

The list is intentionally broader than only modules. It tracks the
module/function seams that determine ownership, behavior preservation, and
where future structural cleanup should land. The current pass counts 98 nodes.

## Entry And Startup

- `1: src/main.rs::main`
- `2: src/app/mod.rs::run`
- `3: src/app/navigation.rs::App::load`
- `4: src/app/navigation.rs::initial_view`
- `5: src/app/services.rs::AppServices::default`
- `6: src/app/services.rs::AppServices::load_view`
- `7: src/view_state/mod.rs::ViewState::load`
- `8: src/jj/view_spec/mod.rs::ViewSpec`
- `9: src/jj/command.rs::JjCommand`

## App Event Loop And Prefix Dispatch

- `10: src/app/mod.rs::App`
- `11: src/app/mod.rs::App::run`
- `12: src/app/mod.rs::App::handle_event`
- `13: src/app/mod.rs::App::handle_normal_key`
- `14: src/app/mod.rs::App::handle_normal_key_at`
- `15: src/app/mod.rs::App::handle_pending_command_key`
- `16: src/app/mod.rs::App::flush_expired_pending_command`
- `17: src/app/mod.rs::App::run_pending_fallback`
- `18: src/app/mod.rs::App::run_binding_with_status_refresh`
- `19: src/app/mod.rs::App::handle_key_after_prefix_fallback`
- `20: src/app/mod.rs::App::execute_binding`
- `21: src/app/mod.rs::App::execute_view`
- `22: src/app/mod.rs::App::apply_view_effect`
- `23: src/app/mod.rs::App::refresh`
- `24: src/command/mod.rs`
- `25: src/modes/mod.rs::InteractionMode`

## App Navigation And Services

- `26: src/app/navigation.rs::App::push_detail`
- `27: src/app/navigation.rs::App::detail_spec`
- `28: src/app/navigation.rs::App::push_view`
- `29: src/app/navigation.rs::App::pop_view`
- `30: src/app/navigation.rs::App::switch_to_log`
- `31: src/app/navigation.rs::App::switch_to_default`
- `32: src/app/navigation.rs::App::open_status`
- `33: src/app/navigation.rs::App::open_resolve`
- `34: src/app/navigation.rs::App::open_bookmarks`
- `35: src/app/navigation.rs::App::open_workspaces`
- `36: src/app/navigation.rs::App::open_operation_log`
- `37: src/app/navigation.rs::App::open_view_menu`
- `38: src/app/navigation.rs::App::apply_view_menu_action`
- `39: src/app/services.rs::AppServices`
- `40: src/app/services.rs::App::refresh_view_state`
- `41: src/app/services.rs::App::reveal_log_change`

## Modal Input And Reducers

- `42: src/app/input/mod.rs::App::handle_mode_key_event_with_terminal`
- `43: src/app/input/mod.rs::App::handle_active_mode_key`
- `44: src/app/input/mod.rs::App::open_copy_menu`
- `45: src/app/input/help.rs`
- `46: src/app/input/prompts.rs`
- `47: src/app/input/menus.rs`
- `48: src/app/input/abandon.rs`
- `49: src/app/actions/input.rs::common preview key handling`
- `50: src/app/reducers/mod.rs::text prompt reducers`
- `51: src/app/reducers/mod.rs::menu reducers`
- `52: src/app/reducers/mod.rs::confirmation reducers`

## Action Entry, Preview, And Completion

- `53: src/app/actions/entry/mod.rs`
- `54: src/app/actions/entry/menu.rs`
- `55: src/app/actions/entry/prompts.rs`
- `56: src/app/actions/entry/remote.rs`
- `57: src/app/actions/preview/mod.rs`
- `58: src/app/actions/completion/mod.rs`
- `59: folded into src/app/actions/completion/rewrite.rs`
- `60: src/app/actions/shared.rs`
- `61: src/app/actions/pane.rs`
- `62: src/actions/mod.rs`
- `63: src/actions/working_copy/mod.rs`
- `64: src/actions/rewrite.rs`
- `65: src/actions/describe/mod.rs`
- `66: src/actions/abandon/mod.rs`
- `67: src/actions/files/mod.rs`
- `68: src/actions/git_sync.rs`

## View Dispatch And Per-View Surfaces

- `69: src/view_state/mod.rs::ViewState::render`
- `70: src/view_state/mod.rs::ViewState::bindings`
- `71: src/view_state/mod.rs::ViewState::execute`
- `72: src/view_state/mod.rs::ViewState::refresh`
- `73: src/view_state/mod.rs::ViewState::clamp`
- `74: src/view_state/targets/mod.rs`

## Feature Roots And Shared Boundaries

- `75: src/log/mod.rs and src/log/view.rs`
- `76: src/show/mod.rs and src/diff/mod.rs`
- `77: src/status/mod.rs`
- `78: src/resolve/mod.rs and children`
- `79: src/files/mod.rs`
- `80: src/bookmarks/mod.rs`
- `81: src/workspaces/mod.rs`
- `82: src/operation_log/mod.rs`
- `83: src/documents/mod.rs`
- `84: src/documents/rendered/mod.rs`
- `85: src/documents/sticky/mod.rs`
- `86: src/jj/mod.rs`
- `87: src/jj/process/mod.rs`
- `88: src/jj/syntax.rs`
- `89: src/rendered_rows/mod.rs`
- `90: src/search/mod.rs`
- `91: src/selection.rs`
- `92: folded into src/menus/model/copy.rs`
- `93: src/clipboard.rs`
- `94: src/menus/mod.rs`
- `95: folded into src/view_state/targets/mod.rs`
- `96: src/tui/mod.rs`
- `97: src/tui/chrome.rs`
- `98: src/tui/overlays.rs`

## Current Use

Use the nodes in order when choosing the next maintainability packet:

1. start from the highest runtime node whose ownership is still hard to read
2. confirm the behavior contract from nearby tests and docs
3. make one structural move inside that owner
4. run focused validation plus `cargo check`
5. record what stayed behavior-identical before moving deeper in the tree
