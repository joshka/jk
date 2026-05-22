use super::*;

use crate::bookmarks;
use crate::log;

#[test]
fn push_target_from_log_uses_exact_revision() {
    let view = ViewState::Log(log::LogView::test_new(vec![crate::log::LogItem::new(
        Vec::new(),
        Some("abcdefg".to_owned()),
        None,
    )]));

    assert_eq!(
        view.push_target().unwrap(),
        Some(JjGitPushTarget::Revision("abcdefg".to_owned()))
    );
}

#[test]
fn push_target_from_log_requires_exact_revision() {
    let view = ViewState::Log(log::LogView::test_new(vec![crate::log::LogItem::new(
        Vec::new(),
        None,
        None,
    )]));

    assert_eq!(
        view.push_target().unwrap_err().to_string(),
        "push from log requires a selected row with an exact revision"
    );
}

#[test]
fn push_target_from_bookmarks_uses_selected_name() {
    let view = ViewState::Bookmarks(bookmarks::BookmarksView::test_new(vec![
        crate::bookmarks::BookmarkItem::new(Vec::new(), "main".to_owned(), None, None),
    ]));

    assert_eq!(
        view.push_target().unwrap(),
        Some(JjGitPushTarget::Bookmark("main".to_owned()))
    );
}

#[test]
fn bookmark_target_from_log_and_status_is_exact() {
    let view = ViewState::Log(log::LogView::test_new(vec![crate::log::LogItem::new(
        Vec::new(),
        Some("abcdefg".to_owned()),
        None,
    )]));

    assert_eq!(
        view.bookmark_target().unwrap(),
        Some(JjBookmarkTarget::exact_change("abcdefg"))
    );

    let view = ViewState::Status(crate::status::StatusView::test_new(&[]));

    assert_eq!(
        view.bookmark_target().unwrap(),
        Some(JjBookmarkTarget::current_working_copy())
    );
}

#[test]
fn bookmark_target_from_log_requires_exact_revision() {
    let view = ViewState::Log(log::LogView::test_new(vec![crate::log::LogItem::new(
        Vec::new(),
        None,
        None,
    )]));

    assert_eq!(
        view.bookmark_target().unwrap_err().to_string(),
        "bookmark mutation from log requires a selected row with an exact revision"
    );
}

#[test]
fn selected_local_bookmark_name_rejects_nonlocal_rows() {
    let view = ViewState::Bookmarks(bookmarks::BookmarksView::test_new(vec![
        crate::bookmarks::BookmarkItem::new(Vec::new(), "@origin".to_owned(), None, None)
            .with_local(false),
    ]));

    assert_eq!(
        view.selected_local_bookmark_name().unwrap_err().to_string(),
        "delete requires a selected exact local bookmark"
    );
}

#[test]
fn exact_restore_revert_context_uses_log_derived_detail_target_and_path() {
    let show = ViewState::Show(crate::show::ShowView::test_new(ViewSpec::show(
        "abcdefg".to_owned(),
        crate::jj::DiffFormat::Default,
    )));
    let file_list = ViewState::FileList(crate::files::list::FileListView::test_with_spec(
        ViewSpec::file_list(Some("abcdefg".to_owned()), Some("src/main.rs".to_owned()))
            .with_exact_change_target(),
        vec![crate::files::list::FileListItem::new(
            Vec::new(),
            "src/main.rs".to_owned(),
        )],
    ));
    let file_show = ViewState::FileShow(crate::files::show::FileShowView::new(
        ViewSpec::file_show(Some("abcdefg".to_owned()), "src/main.rs".to_owned())
            .with_exact_change_target(),
        "src/main.rs",
        crate::documents::DocumentLines::new(Vec::new()),
    ));

    assert_eq!(
        show.exact_restore_revert_context().unwrap(),
        Some(ExactActionContext::detail("abcdefg"))
    );
    assert_eq!(
        file_list.exact_restore_revert_context().unwrap(),
        Some(ExactActionContext::detail("abcdefg").with_selected_path("src/main.rs"))
    );
    assert_eq!(
        file_show.exact_restore_revert_context().unwrap(),
        Some(ExactActionContext::detail("abcdefg").with_selected_path("src/main.rs"))
    );
}

#[test]
fn exact_restore_revert_context_uses_status_selected_path_at_working_copy() {
    let mut status =
        crate::status::StatusView::test_new(&["Working copy changes:", "M src/status.rs"]);
    status.scroll_down(4, 1);
    let view = ViewState::Status(status);

    assert_eq!(
        view.exact_restore_revert_context().unwrap(),
        Some(ExactActionContext::status_path("src/status.rs"))
    );
}

#[test]
fn exact_restore_revert_context_rejects_ambiguous_status_row() {
    let view = ViewState::Status(crate::status::StatusView::test_new(&[
        "R {old.rs => new.rs}",
    ]));

    assert_eq!(
        view.exact_restore_revert_context().unwrap_err().to_string(),
        "status file action unavailable: renamed status rows contain multiple paths"
    );
}

#[test]
fn exact_restore_revert_context_rejects_direct_startup_detail_revsets() {
    let show = ViewState::Show(crate::show::ShowView::test_new(ViewSpec::new(
        JjCommand::Show,
        vec!["main".to_owned()],
    )));
    let diff = ViewState::Diff(crate::diff::DiffView::test_new(ViewSpec::new(
        JjCommand::Diff,
        vec!["-r".to_owned(), "main".to_owned()],
    )));
    let file_list = ViewState::FileList(crate::files::list::FileListView::test_with_spec(
        ViewSpec::file_list(Some("main".to_owned()), Some("src/main.rs".to_owned())),
        vec![crate::files::list::FileListItem::new(
            Vec::new(),
            "src/main.rs".to_owned(),
        )],
    ));
    let file_show = ViewState::FileShow(crate::files::show::FileShowView::new(
        ViewSpec::file_show(Some("main".to_owned()), "src/main.rs".to_owned()),
        "src/main.rs",
        crate::documents::DocumentLines::new(Vec::new()),
    ));

    assert_eq!(
        show.exact_restore_revert_context().unwrap_err().to_string(),
        "restore/revert from jk show main requires an exact log-derived revision target"
    );
    assert_eq!(
        diff.exact_restore_revert_context().unwrap_err().to_string(),
        "restore/revert from jk diff -r main requires an exact log-derived revision target"
    );
    assert_eq!(
        file_list
            .exact_restore_revert_context()
            .unwrap_err()
            .to_string(),
        "file actions from jk file list -r main require a working-copy file list or exact log-derived revision target"
    );
    assert_eq!(
        file_show
            .exact_restore_revert_context()
            .unwrap_err()
            .to_string(),
        "file actions from jk file show -r main src/main.rs require a working-copy file show or exact log-derived revision target"
    );
}
