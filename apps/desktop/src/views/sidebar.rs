mod chrome;
mod context_menu;
mod folder_icons;
mod folder_tree;
mod model;
mod rename;

#[allow(unused_imports)]
pub use folder_icons::FOLDER_ICON_OPTIONS;
#[allow(unused_imports)]
pub(crate) use folder_icons::{folder_icon_for, folder_icon_from_name, folder_icon_name};
pub use model::{RenameKind, RenameState, SidebarContextMenu, SidebarContextTarget, SidebarView};

use crate::components::{InputState, Sidebar, SidebarCollapsible, UniformListScrollHandle};
use crate::store::NoteStore;
use gpui::*;
use std::collections::HashSet;

impl SidebarView {
    pub fn new(store: Entity<NoteStore>, cx: &mut Context<Self>) -> Self {
        let store_sub = cx.observe(&store, |this, _, cx| {
            this.tree_rows_dirty = true;
            cx.notify();
        });
        let expanded_folders = HashSet::new();

        Self {
            store,
            is_collapsed: false,
            search_state: InputState::new(""),
            search_focus: cx.focus_handle(),
            expanded_folders,
            context_menu: None,
            context_menu_focus: cx.focus_handle(),
            renaming: None,
            picking_icon_for: None,
            tree_scroll_handle: UniformListScrollHandle::new(),
            search_scroll_handle: UniformListScrollHandle::new(),
            tree_rows: Vec::new(),
            tree_rows_dirty: true,
            search_rows: Vec::new(),
            last_search_query: String::new(),
            scrollbar_drag_offset: None,
            updater: None,
            _store_subscription: store_sub,
            _updater_subscription: None,
        }
    }

    pub fn set_updater(&mut self, updater: Entity<crate::updater::UpdateManager>, cx: &mut Context<Self>) {
        let sub = cx.observe(&updater, |_, _, cx| cx.notify());
        self.updater = Some(updater);
        self._updater_subscription = Some(sub);
        cx.notify();
    }

    pub fn has_update_available(&self, cx: &App) -> bool {
        self.updater
            .as_ref()
            .map(|u| {
                matches!(
                    u.read(cx).status(),
                    crate::updater::UpdateStatus::Available { .. }
                        | crate::updater::UpdateStatus::ReadyToRestart { .. }
                )
            })
            .unwrap_or(false)
    }

    #[allow(dead_code)]
    pub fn search_query(&self) -> &str {
        self.search_state.value()
    }

    fn handle_search_key(
        &mut self,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.search_state.handle_key(event, window, cx) {
            cx.notify();
        }
    }

    pub fn focus_search(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.is_collapsed {
            self.is_collapsed = false;
        }
        window.focus(&self.search_focus);
        cx.notify();
    }

    pub fn toggle_collapsed(&mut self, cx: &mut Context<Self>) {
        self.is_collapsed = !self.is_collapsed;
        self.context_menu = None;
        cx.notify();
    }

    pub fn toggle_folder(&mut self, folder_id: &str, cx: &mut Context<Self>) {
        if self.expanded_folders.contains(folder_id) {
            self.expanded_folders.remove(folder_id);
        } else {
            self.expanded_folders.insert(folder_id.to_string());
        }
        self.tree_rows_dirty = true;
        cx.notify();
    }

    pub fn is_folder_expanded(&self, folder_id: &str) -> bool {
        self.expanded_folders.contains(folder_id)
    }

    #[cfg(test)]
    pub fn test_store(&self) -> Entity<NoteStore> {
        self.store.clone()
    }

    // ---- thin delegates over NoteStore ----

    pub fn selected_note_id(&self, cx: &App) -> Option<String> {
        self.store.read(cx).selected_note_id()
    }

    pub fn open_starred(&mut self, cx: &mut Context<Self>) {
        self.store.update(cx, |store, cx| store.open_starred(cx));
    }

    #[allow(dead_code)]
    pub fn toggle_starred(&mut self, cx: &mut Context<Self>) {
        self.open_starred(cx);
    }

    pub fn open_trash(&mut self, cx: &mut Context<Self>) {
        self.store.update(cx, |store, cx| store.open_trash(cx));
    }

    #[allow(dead_code)]
    pub fn select_section(&mut self, section_id: &str, cx: &mut Context<Self>) {
        match section_id {
            "starred" => self.open_starred(cx),
            "trash" => self.open_trash(cx),
            _ => cx.notify(),
        }
    }

    pub fn select_note(&mut self, note_id: &str, cx: &mut Context<Self>) {
        self.store
            .update(cx, |store, cx| store.select_note(note_id, cx));
    }

    pub fn navigate_back(&mut self, cx: &mut Context<Self>) -> bool {
        self.store.update(cx, |store, cx| store.navigate_back(cx))
    }

    pub fn navigate_forward(&mut self, cx: &mut Context<Self>) -> bool {
        self.store
            .update(cx, |store, cx| store.navigate_forward(cx))
    }

    pub fn create_new_note(&mut self, cx: &mut Context<Self>) {
        self.store.update(cx, |store, cx| store.create_new_note(cx));
        self.context_menu = None;
    }

    pub fn create_note_in_folder(&mut self, folder_id: Option<String>, cx: &mut Context<Self>) {
        self.store
            .update(cx, |store, cx| store.create_note_in_folder(folder_id, cx));
        self.context_menu = None;
    }

    pub fn toggle_note_pin(&mut self, note_id: &str, cx: &mut Context<Self>) {
        self.store
            .update(cx, |store, cx| store.toggle_note_pin(note_id, cx));
        self.context_menu = None;
    }

    pub fn duplicate_note(&mut self, note_id: &str, cx: &mut Context<Self>) {
        self.store
            .update(cx, |store, cx| store.duplicate_note(note_id, cx));
        self.context_menu = None;
    }

    pub fn delete_note(&mut self, note_id: &str, cx: &mut Context<Self>) {
        self.store
            .update(cx, |store, cx| store.delete_note(note_id, cx));
        self.context_menu = None;
    }

    pub fn create_top_level_folder(&mut self, cx: &mut Context<Self>) {
        self.store
            .update(cx, |store, cx| store.create_top_level_folder(cx));
        self.context_menu = None;
    }

    pub fn create_subfolder(&mut self, folder_id: &str, cx: &mut Context<Self>) {
        let folder_id = folder_id.to_string();
        self.store.update(cx, |store, cx| {
            store.create_folder("Untitled Folder", Some(folder_id), cx);
        });
        self.context_menu = None;
        cx.notify();
    }

    pub fn delete_folder(&mut self, folder_id: &str, cx: &mut Context<Self>) {
        let subtree = self.store.read(cx).folder_subtree_ids(folder_id);
        self.store
            .update(cx, |store, cx| store.delete_folder(folder_id, cx));
        for fid in subtree {
            self.expanded_folders.remove(&fid);
        }
        self.context_menu = None;
    }
}

impl Render for SidebarView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let sidebar = Sidebar::new("main-sidebar")
            .width(px(260.))
            .collapsed(self.is_collapsed)
            .collapsible(SidebarCollapsible::Icon)
            .header(self.render_header(cx))
            .content(self.render_content(cx))
            .footer(self.render_footer(cx))
            .rail(self.render_rail(cx))
            .into_any_element();

        if let Some(menu) = self.context_menu_element(cx) {
            div()
                .id("sidebar-with-context-menu")
                .h_full()
                .child(sidebar)
                .child(menu)
                .into_any_element()
        } else if let Some(picker) = self.icon_picker_element(cx) {
            div()
                .id("sidebar-with-icon-picker")
                .h_full()
                .child(sidebar)
                .child(picker)
                .into_any_element()
        } else {
            sidebar
        }
    }
}

#[cfg(test)]
mod tests {
    // NOTE: no glob imports here on purpose. `use gpui::*` would pull the
    // `gpui::test` attribute macro into scope, shadowing the builtin `#[test]`
    // and breaking compilation of this module.
    use super::{
        FOLDER_ICON_OPTIONS, RenameKind, RenameState, SidebarContextMenu, SidebarContextTarget,
        SidebarView, folder_icon_for, folder_icon_from_name, folder_icon_name,
    };
    use crate::components::{IconName, InputState};
    use crate::store::{NavigationLocation, NoteStore};
    use crate::theme::{ActiveTheme, Theme};
    use gpui::{AppContext, MouseButton, MouseDownEvent, TestAppContext, point, px};

    #[test]
    fn model_types_are_reexported() {
        // Compile-time proof the `model` split keeps the public surface.
        let _ = RenameKind::Folder;
        let _ = std::mem::size_of::<RenameState>();
        let _ = std::mem::size_of::<Option<SidebarContextMenu>>();
        let _ = InputState::new("");
    }

    #[test]
    fn folder_icon_table_roundtrips_and_falls_back() {
        assert_eq!(FOLDER_ICON_OPTIONS.len(), 17);
        assert_eq!(folder_icon_name(IconName::Briefcase), "briefcase");
        assert_eq!(folder_icon_from_name("rocket"), Some(IconName::Rocket));
        assert_eq!(folder_icon_from_name("  STAR  "), Some(IconName::Star));
        assert_eq!(folder_icon_from_name("nope"), None);
        // Legacy emoji + unknown values fall back to open/closed glyphs.
        assert_eq!(folder_icon_for("📁", true), IconName::FolderOpen);
        assert_eq!(folder_icon_for("???", false), IconName::Folder);
        assert_eq!(folder_icon_for("code", false), IconName::Code);
    }

    #[test]
    fn icon_picker_toggle_and_pick_flow() {
        let mut cx = TestAppContext::single();
        cx.update(|cx| {
            cx.set_global(ActiveTheme(Theme::dark()));
        });

        let (view, cx) = cx.add_window_view(|_, cx| {
            let store = cx.new(|_| NoteStore::new());
            SidebarView::new(store, cx)
        });
        let store = view.read_with(cx, |v, _| v.test_store());
        store.update(cx, |s, _| s.seed_test_data());
        cx.run_until_parked();

        assert!(view.read_with(cx, |v, _| v.picking_icon_for.is_none()));

        view.update(cx, |v, cx| v.toggle_icon_picker("projects", cx));
        cx.run_until_parked();
        assert_eq!(
            view.read_with(cx, |v, _| v.picking_icon_for.clone()),
            Some("projects".to_string())
        );

        view.update(cx, |v, cx| v.toggle_icon_picker("projects", cx));
        cx.run_until_parked();
        assert!(view.read_with(cx, |v, _| v.picking_icon_for.is_none()));

        view.update(cx, |v, cx| {
            v.toggle_icon_picker("projects", cx);
            v.pick_folder_icon("projects", IconName::Rocket, cx);
        });
        cx.run_until_parked();
        assert!(view.read_with(cx, |v, _| v.picking_icon_for.is_none()));
        assert_eq!(
            store.read_with(cx, |s, _| s
                .folder_tree()
                .iter()
                .find(|n| n.folder.id == "projects")
                .map(|n| n.folder.icon.clone())),
            Some("rocket".to_string())
        );
    }

    #[test]
    fn rename_commit_and_cancel_flow() {
        let mut cx = TestAppContext::single();
        cx.update(|cx| {
            cx.set_global(ActiveTheme(Theme::dark()));
        });

        let (view, cx) = cx.add_window_view(|_, cx| {
            let store = cx.new(|_| NoteStore::new());
            SidebarView::new(store, cx)
        });
        let store = view.read_with(cx, |v, _| v.test_store());
        store.update(cx, |s, _| s.seed_test_data());
        cx.run_until_parked();

        // Folder rename: begin → type → commit.
        cx.update(|window, cx| {
            view.update(cx, |v, cx| v.begin_rename_folder("projects", window, cx));
        });
        cx.run_until_parked();
        assert!(view.read_with(cx, |v, _| v.renaming.is_some()));
        view.update(cx, |v, cx| {
            if let Some(state) = v.renaming.as_mut() {
                state.input.set_value("Renamed");
            }
            v.commit_rename(cx);
        });
        cx.run_until_parked();
        assert!(view.read_with(cx, |v, _| v.renaming.is_none()));
        assert_eq!(
            store.read_with(cx, |s, _| s
                .folder_tree()
                .iter()
                .find(|n| n.folder.id == "projects")
                .map(|n| n.folder.name.clone())),
            Some("Renamed".to_string())
        );

        // Note rename: begin → cancel leaves the title untouched.
        cx.update(|window, cx| {
            view.update(cx, |v, cx| {
                v.begin_rename_note("note-db-schema", window, cx)
            });
        });
        cx.run_until_parked();
        assert!(view.read_with(cx, |v, _| v.renaming.is_some()));
        view.update(cx, |v, cx| v.cancel_rename(cx));
        cx.run_until_parked();
        assert!(view.read_with(cx, |v, _| v.renaming.is_none()));
        assert_eq!(
            store.read_with(cx, |s, _| s
                .active_notes()
                .into_iter()
                .find(|n| n.id == "note-db-schema")
                .map(|n| n.title)),
            Some("Database Schema & Index Design".to_string())
        );
    }

    /// Regression loop for "right-click in the sidebar does nothing".
    /// Drives the real event-dispatch path: synthetic right mouse-down events
    /// sweep down the sidebar until a folder row opens the context menu.
    #[test]
    fn right_click_folder_row_opens_context_menu() {
        let mut cx = TestAppContext::single();
        cx.update(|cx| {
            cx.set_global(ActiveTheme(Theme::dark()));
        });

        let (view, cx) = cx.add_window_view(|_, cx| {
            let store = cx.new(|_| NoteStore::new());
            SidebarView::new(store, cx)
        });
        let store = view.read_with(cx, |v, _| v.test_store());
        store.update(cx, |s, _| s.seed_test_data());
        cx.run_until_parked();
        cx.update(|window, cx| {
            _ = window.draw(cx);
        });

        let mut opened = None;
        let mut seen_folders = Vec::new();
        let mut y = 150.;
        while y < 700. {
            cx.simulate_event(MouseDownEvent {
                button: MouseButton::Right,
                position: point(px(130.), px(y)),
                modifiers: Default::default(),
                click_count: 1,
                first_mouse: false,
            });
            cx.run_until_parked();
            if let Some(state) = view.read_with(cx, |v, _| v.context_menu.clone())
                && let SidebarContextTarget::Folder { id, .. } = &state.target
            {
                seen_folders.push(id.clone());
                if id == "projects" {
                    opened = Some((y, state));
                    break;
                }
            }
            y += 4.;
        }

        let (y, state) = opened.expect("right-click should open a context menu on Projects");
        match state.target {
            SidebarContextTarget::Folder { id, .. } => {
                assert_eq!(id, "projects", "folder row hit while sweeping at y={y}")
            }
            other => panic!("expected folder target at y={y}, got {other:?}"),
        }
    }

    /// Full user flow: right-click a note row, left-click the first menu item
    /// ("Pin to Starred"), and assert the note flips to pinned. This covers
    /// dispatch → state → render → hit-test → action end to end.
    #[test]
    fn right_click_note_then_pin_item_pins_note() {
        let mut cx = TestAppContext::single();
        cx.update(|cx| {
            cx.set_global(ActiveTheme(Theme::dark()));
        });

        let (view, cx) = cx.add_window_view(|_, cx| {
            let store = cx.new(|_| NoteStore::new());
            SidebarView::new(store, cx)
        });
        let store = view.read_with(cx, |v, _| v.test_store());
        store.update(cx, |s, _| s.seed_test_data());
        view.update(cx, |v, cx| v.toggle_folder("projects", cx));
        cx.run_until_parked();
        cx.update(|window, cx| {
            _ = window.draw(cx);
        });

        // Sweep until the "Database Schema" note row opens its menu.
        let mut menu_position = None;
        let mut y = 150.;
        while y < 700. {
            cx.simulate_event(MouseDownEvent {
                button: MouseButton::Right,
                position: point(px(130.), px(y)),
                modifiers: Default::default(),
                click_count: 1,
                first_mouse: false,
            });
            cx.run_until_parked();
            if let Some(state) = view.read_with(cx, |v, _| v.context_menu.clone())
                && let SidebarContextTarget::Note { id, .. } = &state.target
                && id == "note-db-schema"
            {
                menu_position = Some(state.position);
                break;
            }
            y += 4.;
        }
        let menu_position =
            menu_position.expect("right-click should open a context menu on note-db-schema");

        assert!(
            !store.read_with(cx, |s, _| s
                .active_notes()
                .iter()
                .find(|n| n.id == "note-db-schema")
                .map(|n| n.pinned)
                .unwrap_or(true)),
            "note-db-schema should start unpinned"
        );

        // Redraw with the menu open so hitboxes exist, then press the first
        // item ("Pin to Starred"): label (~22px) + half the 28px row, past
        // the p_1 (4px) padding. Items activate on mouse-down.
        cx.update(|window, cx| {
            _ = window.draw(cx);
        });
        cx.simulate_event(MouseDownEvent {
            button: MouseButton::Left,
            position: point(menu_position.x + px(100.), menu_position.y + px(40.)),
            modifiers: Default::default(),
            click_count: 1,
            first_mouse: false,
        });
        cx.run_until_parked();

        assert!(
            store.read_with(cx, |s, _| s
                .active_notes()
                .iter()
                .find(|n| n.id == "note-db-schema")
                .map(|n| n.pinned)
                .unwrap_or(false)),
            "clicking Pin to Starred should pin note-db-schema"
        );
        assert!(
            view.read_with(cx, |v, _| v.context_menu.is_none()),
            "menu should close after selecting an item"
        );
    }

    #[test]
    fn toggle_collapsed_switches_between_expanded_and_rail() {
        let mut cx = TestAppContext::single();
        cx.update(|cx| {
            cx.set_global(ActiveTheme(Theme::dark()));
        });

        let (view, cx) = cx.add_window_view(|_, cx| {
            let store = cx.new(|_| NoteStore::new());
            SidebarView::new(store, cx)
        });
        cx.run_until_parked();

        assert!(!view.read_with(cx, |v, _| v.is_collapsed));

        view.update(cx, |v, cx| {
            v.toggle_collapsed(cx);
        });
        cx.run_until_parked();
        assert!(view.read_with(cx, |v, _| v.is_collapsed));

        view.update(cx, |v, cx| {
            v.toggle_collapsed(cx);
        });
        cx.run_until_parked();
        assert!(!view.read_with(cx, |v, _| v.is_collapsed));
    }

    #[test]
    fn sidebar_view_note_navigation_back_and_forward() {
        let mut cx = TestAppContext::single();
        cx.update(|cx| {
            cx.set_global(ActiveTheme(Theme::dark()));
        });

        let (view, cx) = cx.add_window_view(|_, cx| {
            let store = cx.new(|_| NoteStore::new());
            SidebarView::new(store, cx)
        });
        let store = view.read_with(cx, |v, _| v.test_store());
        store.update(cx, |s, _| s.seed_test_data());
        cx.run_until_parked();

        assert_eq!(
            store.read_with(cx, |s, _| s.selected_note_id()),
            Some("note-arch-spec".to_string())
        );
        assert!(!store.read_with(cx, |s, _| s.can_navigate_back()));
        assert!(!store.read_with(cx, |s, _| s.can_navigate_forward()));

        view.update(cx, |v, cx| {
            v.select_note("note-desktop-gpui", cx);
        });
        cx.run_until_parked();
        assert_eq!(
            store.read_with(cx, |s, _| s.selected_note_id()),
            Some("note-desktop-gpui".to_string())
        );
        assert!(store.read_with(cx, |s, _| s.can_navigate_back()));
        assert!(!store.read_with(cx, |s, _| s.can_navigate_forward()));

        view.update(cx, |v, cx| {
            v.select_note("note-db-schema", cx);
        });
        cx.run_until_parked();

        let went_back = view.update(cx, |v, cx| v.navigate_back(cx));
        assert!(went_back);
        assert_eq!(
            store.read_with(cx, |s, _| s.selected_note_id()),
            Some("note-desktop-gpui".to_string())
        );
        assert!(store.read_with(cx, |s, _| s.can_navigate_back()));
        assert!(store.read_with(cx, |s, _| s.can_navigate_forward()));

        let went_back = view.update(cx, |v, cx| v.navigate_back(cx));
        assert!(went_back);
        assert_eq!(
            store.read_with(cx, |s, _| s.selected_note_id()),
            Some("note-arch-spec".to_string())
        );
        assert!(!store.read_with(cx, |s, _| s.can_navigate_back()));
        assert!(store.read_with(cx, |s, _| s.can_navigate_forward()));

        let went_forward = view.update(cx, |v, cx| v.navigate_forward(cx));
        assert!(went_forward);
        assert_eq!(
            store.read_with(cx, |s, _| s.selected_note_id()),
            Some("note-desktop-gpui".to_string())
        );
    }

    #[test]
    fn folder_toggle_does_not_select_folder_or_affect_note_selection() {
        let mut cx = TestAppContext::single();
        cx.update(|cx| {
            cx.set_global(ActiveTheme(Theme::dark()));
        });

        let (view, cx) = cx.add_window_view(|_, cx| {
            let store = cx.new(|_| NoteStore::new());
            SidebarView::new(store, cx)
        });
        let store = view.read_with(cx, |v, _| v.test_store());
        store.update(cx, |s, _| s.seed_test_data());
        cx.run_until_parked();

        assert_eq!(
            store.read_with(cx, |s, _| s.selected_note_id()),
            Some("note-arch-spec".to_string())
        );

        view.update(cx, |v, cx| {
            v.toggle_folder("projects", cx);
        });
        cx.run_until_parked();

        assert_eq!(
            store.read_with(cx, |s, _| s.selected_note_id()),
            Some("note-arch-spec".to_string())
        );

        view.update(cx, |v, cx| {
            v.open_starred(cx);
        });
        cx.run_until_parked();

        assert_eq!(
            store.read_with(cx, |s, _| s.active_location().clone()),
            NavigationLocation::Starred
        );

        view.update(cx, |v, cx| {
            v.open_trash(cx);
        });
        cx.run_until_parked();
        assert_eq!(
            store.read_with(cx, |s, _| s.active_location().clone()),
            NavigationLocation::Trash
        );

        view.update(cx, |v, cx| {
            v.select_note("note-desktop-gpui", cx);
        });
        cx.run_until_parked();
        assert_eq!(
            store.read_with(cx, |s, _| s.active_location().clone()),
            NavigationLocation::Note("note-desktop-gpui".to_string())
        );
    }

    #[test]
    fn test_starred_and_trash_navigation_and_item_actions() {
        let mut cx = TestAppContext::single();
        cx.update(|cx| {
            cx.set_global(ActiveTheme(Theme::dark()));
        });

        let (view, cx) = cx.add_window_view(|_, cx| {
            let store = cx.new(|_| NoteStore::new());
            SidebarView::new(store, cx)
        });
        let store = view.read_with(cx, |v, _| v.test_store());
        store.update(cx, |s, _| s.seed_test_data());
        cx.run_until_parked();

        assert_eq!(
            store.read_with(cx, |s, _| s.active_location().clone()),
            NavigationLocation::Note("note-arch-spec".to_string())
        );

        view.update(cx, |v, cx| v.open_starred(cx));
        assert_eq!(
            store.read_with(cx, |s, _| s.active_location().clone()),
            NavigationLocation::Starred
        );
        assert!(store.read_with(cx, |s, _| s.can_navigate_back()));
        assert!(!store.read_with(cx, |s, _| s.starred_notes().is_empty()));

        view.update(cx, |v, cx| v.open_trash(cx));
        assert_eq!(
            store.read_with(cx, |s, _| s.active_location().clone()),
            NavigationLocation::Trash
        );
        assert!(store.read_with(cx, |s, _| s.can_navigate_back()));

        view.update(cx, |v, cx| {
            assert!(v.navigate_back(cx));
        });
        assert_eq!(
            store.read_with(cx, |s, _| s.active_location().clone()),
            NavigationLocation::Starred
        );

        view.update(cx, |v, cx| {
            assert!(v.navigate_back(cx));
        });
        assert_eq!(
            store.read_with(cx, |s, _| s.active_location().clone()),
            NavigationLocation::Note("note-arch-spec".to_string())
        );

        view.update(cx, |v, cx| {
            v.delete_note("note-arch-spec", cx);
        });
        assert!(store.read_with(cx, |s, _| {
            s.trash_notes().iter().any(|n| n.id == "note-arch-spec")
        }));

        view.update(cx, |v, cx| {
            v.open_trash(cx);
        });
        store.update(cx, |s, cx| {
            s.restore_note("note-arch-spec", cx);
        });
        assert!(!store.read_with(cx, |s, _| {
            s.trash_notes().iter().any(|n| n.id == "note-arch-spec")
        }));
        assert_eq!(
            store.read_with(cx, |s, _| s.active_location().clone()),
            NavigationLocation::Trash
        );
        assert!(store.read_with(cx, |s, _| {
            s.active_notes().iter().any(|n| n.id == "note-arch-spec")
        }));

        view.update(cx, |v, cx| {
            v.delete_note("note-arch-spec", cx);
        });
        store.update(cx, |s, cx| {
            s.empty_trash(cx);
        });
        assert!(store.read_with(cx, |s, _| s.trash_notes().is_empty()),);
    }

    #[test]
    fn sidebar_renders_with_scrollbar_when_overflowing() {
        let mut cx = TestAppContext::single();
        cx.update(|cx| {
            cx.set_global(ActiveTheme(Theme::dark()));
        });

        let (view, cx) = cx.add_window_view(|_, cx| {
            let store = cx.new(|_| NoteStore::new());
            SidebarView::new(store, cx)
        });
        let store = view.read_with(cx, |v, _| v.test_store());
        store.update(cx, |s, cx| {
            for _ in 0..50 {
                s.create_note_in_folder(None, cx);
            }
        });
        cx.run_until_parked();
        cx.update(|window, cx| {
            _ = window.draw(cx);
        });

        view.update(cx, |v, _| {
            assert!(v.tree_rows.len() >= 50);
        });
    }
}
