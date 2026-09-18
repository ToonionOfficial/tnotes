mod about;
mod account;
mod components;
mod developer;
mod keybinding_capture;
mod keybindings;
mod storage;
mod sync;
mod updates;

pub use components::SettingsSectionId;

use crate::components::{
    Button, Icon, IconName, Input, InputSize, InputState, InputVariant, KbdBadge,
};
use crate::keymap::{ALL_ACTIONS, CloseSettings, KeymapConfig};
use crate::store::NoteStore;
use crate::theme::ThemeExt;
use crate::updater::UpdateManager;
use gpui::prelude::FluentBuilder;
use gpui::*;

pub struct SettingsView {
    store: Entity<NoteStore>,
    updater: Entity<UpdateManager>,
    active_section: SettingsSectionId,
    keymap: KeymapConfig,
    capturing: Option<keybinding_capture::KeybindingCapture>,
    focus_handle: FocusHandle,
    search_state: InputState,
    search_focus: FocusHandle,
    _store_subscription: Subscription,
    _updater_subscription: Subscription,
}

impl SettingsView {
    pub fn new(
        store: Entity<NoteStore>,
        updater: Entity<UpdateManager>,
        cx: &mut Context<Self>,
    ) -> Self {
        let store_sub = cx.observe(&store, |_, _, cx| cx.notify());
        let updater_sub = cx.observe(&updater, |_, _, cx| cx.notify());
        Self {
            store,
            updater,
            active_section: SettingsSectionId::Account,
            keymap: KeymapConfig::load(),
            capturing: None,
            focus_handle: cx.focus_handle(),
            search_state: InputState::new(""),
            search_focus: cx.focus_handle(),
            _store_subscription: store_sub,
            _updater_subscription: updater_sub,
        }
    }

    pub(crate) fn updater(&self) -> &Entity<UpdateManager> {
        &self.updater
    }

    pub fn focus(&self, window: &mut Window) {
        window.focus(&self.focus_handle);
    }

    #[cfg(test)]
    pub fn active_section(&self) -> SettingsSectionId {
        self.active_section
    }

    pub fn select_section(&mut self, section: SettingsSectionId, cx: &mut Context<Self>) {
        self.active_section = section;
        self.capturing = None;
        cx.notify();
    }

    pub(crate) fn keymap(&self) -> &KeymapConfig {
        &self.keymap
    }

    #[cfg(test)]
    pub(crate) fn keymap_mut_for_test(
        &mut self,
        action_id: &str,
        key: &str,
        context: Option<&str>,
    ) {
        self.keymap.set_key_for_action(action_id, key, context);
    }

    pub(crate) fn capturing(&self) -> Option<&keybinding_capture::KeybindingCapture> {
        self.capturing.as_ref()
    }

    pub(crate) fn begin_capture(&mut self, action_id: &str, label: &str, cx: &mut Context<Self>) {
        self.capturing = Some(keybinding_capture::KeybindingCapture::new(action_id, label));
        cx.notify();
    }

    pub(crate) fn cancel_capture(&mut self, cx: &mut Context<Self>) {
        if self.capturing.take().is_some() {
            cx.notify();
        }
    }

    /// Feed a key event into an in-progress capture. Returns true when the
    /// event was consumed (Escape cancels, anything else commits).
    /// Must run before the view-level Escape-to-close handler.
    fn handle_capture_key(&mut self, event: &KeyDownEvent, cx: &mut Context<Self>) -> bool {
        if self.capturing.is_none() {
            return false;
        }
        if event.keystroke.key.eq_ignore_ascii_case("escape") {
            self.cancel_capture(cx);
            return true;
        }
        let Some(keystroke) = keybinding_capture::keystroke_from_event(event) else {
            return true;
        };
        // Single-keystroke bindings commit on press, like VS Code.
        let capture = self.capturing.take().unwrap();
        let context = ALL_ACTIONS
            .iter()
            .find(|a| a.id == capture.action_id)
            .and_then(|a| a.default_context);
        let conflicts = keybinding_capture::apply_captured_keystroke(
            &mut self.keymap,
            &capture.action_id,
            context,
            &keystroke,
            cx,
        );
        if !conflicts.is_empty() {
            self.capturing = Some(keybinding_capture::KeybindingCapture {
                conflict: Some(conflicts.join(", ")),
                preview: Some(keystroke),
                ..capture
            });
        }
        cx.notify();
        true
    }

    pub(crate) fn reset_keymap(&mut self, cx: &mut Context<Self>) {
        self.keymap = KeymapConfig::default_config();
        if let Err(e) = self.keymap.save() {
            eprintln!("[keymap] failed to persist keymap: {e}");
        }
        cx.clear_key_bindings();
        self.keymap.bind_to_gpui(cx);
        self.capturing = None;
        cx.notify();
    }

    #[cfg(test)]
    pub fn test_store(&self) -> Entity<NoteStore> {
        self.store()
    }

    pub(crate) fn store(&self) -> Entity<NoteStore> {
        self.store.clone()
    }

    fn section_matches_query(&self, section: SettingsSectionId, query: &str) -> bool {
        if query.is_empty() {
            return true;
        }
        let q = query.to_lowercase();
        match section {
            SettingsSectionId::Account => "account profile user local offline vault".contains(&q),
            SettingsSectionId::Sync => {
                "sync server connect websocket cloud backend url disconnect auto".contains(&q)
            }
            SettingsSectionId::Storage => {
                "data storage database sqlite export backup markdown json vacuum trash".contains(&q)
            }
            SettingsSectionId::Keybindings => {
                "keybindings shortcuts keyboard capture actions keys".contains(&q)
            }
            SettingsSectionId::Developer => {
                "flags benchmark dev developer performance test tools hud fps".contains(&q)
            }
            SettingsSectionId::Updates => {
                "update updates upgrade channel stable beta alpha version download install check".contains(&q)
            }
            SettingsSectionId::About => {
                "about version tnotes github repo license source".contains(&q)
            }
        }
    }

    fn handle_search_key(
        &mut self,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.search_state.handle_key(event, window, cx) {
            let query = self.search_state.value().trim().to_string();
            if !query.is_empty()
                && !self.section_matches_query(self.active_section, &query)
                && let Some(&first) = SettingsSectionId::ALL
                    .iter()
                    .find(|&&s| self.section_matches_query(s, &query))
            {
                self.select_section(first, cx);
            }
            cx.notify();
        }
    }

    fn render_nav_group(
        &self,
        group_label: &'static str,
        sections: &[SettingsSectionId],
        cx: &mut Context<Self>,
    ) -> Div {
        let theme = cx.theme().clone();
        div()
            .flex()
            .flex_col()
            .gap(px(2.))
            .child(
                div()
                    .px_2()
                    .pt(px(8.))
                    .pb(px(4.))
                    .text_size(px(10.5))
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme.muted_foreground)
                    .child(group_label),
            )
            .children(sections.iter().map(|section| {
                let active = *section == self.active_section;
                let section = *section;
                div()
                    .id(SharedString::from(format!(
                        "settings-nav-{}",
                        section.title().to_lowercase()
                    )))
                    .w_full()
                    .h(px(34.))
                    .px_2()
                    .rounded(px(6.))
                    .flex()
                    .items_center()
                    .gap_2()
                    .cursor_pointer()
                    .bg(if active {
                        theme.secondary
                    } else {
                        gpui::transparent_black()
                    })
                    .hover(|s| s.bg(theme.secondary))
                    .text_color(if active {
                        theme.foreground
                    } else {
                        theme.muted_foreground
                    })
                    .child(Icon::new(section.icon()).size(px(14.)).color(if active {
                        theme.primary
                    } else {
                        theme.muted_foreground
                    }))
                    .child(
                        div()
                            .text_size(px(13.))
                            .font_weight(if active {
                                FontWeight::MEDIUM
                            } else {
                                FontWeight::NORMAL
                            })
                            .child(section.title()),
                    )
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.select_section(section, cx);
                    }))
            }))
    }

    fn render_nav(&self, cx: &mut Context<Self>) -> AnyElement {
        let query = self.search_state.value().trim().to_string();
        let ws_matching: Vec<SettingsSectionId> = SettingsSectionId::WORKSPACE
            .iter()
            .copied()
            .filter(|&s| self.section_matches_query(s, &query))
            .collect();
        let sys_matching: Vec<SettingsSectionId> = SettingsSectionId::SYSTEM
            .iter()
            .copied()
            .filter(|&s| self.section_matches_query(s, &query))
            .collect();

        div()
            .flex_1()
            .flex()
            .flex_col()
            .gap_3()
            .when(!ws_matching.is_empty(), |this| {
                this.child(self.render_nav_group("WORKSPACE", &ws_matching, cx))
            })
            .when(!sys_matching.is_empty(), |this| {
                this.child(self.render_nav_group("SYSTEM & ENGINE", &sys_matching, cx))
            })
            .into_any_element()
    }

    fn render_content(&self, cx: &mut Context<Self>) -> AnyElement {
        let theme = cx.theme().clone();
        let query = self.search_state.value().trim().to_string();
        let has_any_match = query.is_empty()
            || SettingsSectionId::ALL
                .iter()
                .any(|&s| self.section_matches_query(s, &query));

        if !has_any_match {
            return div()
                .flex_1()
                .h_full()
                .bg(theme.background)
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .gap_2()
                .child(
                    div()
                        .text_size(px(17.))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(theme.foreground)
                        .child("No Results"),
                )
                .child(
                    div()
                        .text_size(px(14.))
                        .text_color(theme.muted_foreground)
                        .child(format!("No settings match “{}”", query)),
                )
                .into_any_element();
        }

        let body = match self.active_section {
            SettingsSectionId::Account => account::render(self, cx),
            SettingsSectionId::Sync => sync::render(self, cx),
            SettingsSectionId::Storage => storage::render(self, cx),
            SettingsSectionId::Keybindings => keybindings::render(self, cx),
            SettingsSectionId::Developer => developer::render(self, cx),
            SettingsSectionId::Updates => updates::render(self, cx),
            SettingsSectionId::About => about::render(self, cx),
        };
        div()
            .flex_1()
            .h_full()
            .id("settings-content-scroll")
            .overflow_y_scroll()
            .bg(theme.background)
            .child(
                div()
                    .max_w(px(880.))
                    .w_full()
                    .mx_auto()
                    .px(px(32.))
                    .py(px(28.))
                    .flex()
                    .flex_col()
                    .gap_5()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(3.))
                            .child(
                                div()
                                    .text_size(px(22.))
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(theme.foreground)
                                    .child(self.active_section.title()),
                            )
                            .child(
                                div()
                                    .text_size(px(12.5))
                                    .text_color(theme.muted_foreground)
                                    .child(self.active_section.description()),
                            ),
                    )
                    .child(body),
            )
            .into_any_element()
    }
}

impl Render for SettingsView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();

        div()
            .id("settings-screen")
            .size_full()
            .flex()
            .bg(theme.background)
            .text_color(theme.foreground)
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, window, cx| {
                if this.handle_capture_key(event, cx) {
                    return;
                }
                if event.keystroke.key.eq_ignore_ascii_case("escape") {
                    window.dispatch_action(Box::new(CloseSettings), cx);
                }
            }))
            .on_mouse_down(
                MouseButton::Navigate(NavigationDirection::Back),
                cx.listener(|_this, _event, window, cx| {
                    window.dispatch_action(Box::new(CloseSettings), cx);
                }),
            )
            .child(
                div()
                    .id("settings-sidebar")
                    .w(px(240.))
                    .h_full()
                    .flex()
                    .flex_col()
                    .justify_between()
                    .bg(theme.card)
                    .border_r_1()
                    .border_color(theme.border)
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .child(
                                div()
                                    .id("settings-sidebar-header")
                                    .px_3()
                                    .py(px(12.))
                                    .border_b_1()
                                    .border_color(theme.border)
                                    .flex()
                                    .items_center()
                                    .justify_between()
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .gap_2()
                                            .child(
                                                div()
                                                    .w(px(22.))
                                                    .h(px(22.))
                                                    .rounded(px(6.))
                                                    .bg(theme.primary)
                                                    .flex()
                                                    .items_center()
                                                    .justify_center()
                                                    .child(
                                                        Icon::new(IconName::Settings)
                                                            .size(px(12.))
                                                            .color(theme.primary_foreground),
                                                    ),
                                            )
                                            .child(
                                                div()
                                                    .text_size(px(13.5))
                                                    .font_weight(FontWeight::BOLD)
                                                    .text_color(theme.foreground)
                                                    .child("Settings"),
                                            ),
                                    )
                                    .child(KbdBadge::new("Esc")),
                            )
                            .child(
                                div()
                                    .id("settings-sidebar-search")
                                    .px_2()
                                    .pt(px(6.))
                                    .pb(px(4.))
                                    .child(
                                        Input::new("settings-search-bar")
                                            .placeholder("Search settings...")
                                            .leading_icon(IconName::Search)
                                            .variant(InputVariant::Sidebar)
                                            .size(InputSize::Sm)
                                            .state(&self.search_state)
                                            .focus_handle(self.search_focus.clone())
                                            .on_key_down(cx.listener(|this, event, window, cx| {
                                                this.handle_search_key(event, window, cx);
                                            }))
                                            .on_clear(cx.listener(|this, _, _, cx| {
                                                this.search_state.clear();
                                                cx.notify();
                                            })),
                                    ),
                            ),
                    )
                    .child(
                        div()
                            .id("settings-sidebar-nav")
                            .flex_1()
                            .p_2()
                            .child(self.render_nav(cx)),
                    )
                    .child(
                        div()
                            .id("settings-sidebar-footer")
                            .p_2()
                            .border_t_1()
                            .border_color(theme.border)
                            .child(
                                Button::ghost("settings-back-btn", "Back to Notes")
                                    .leading_icon(IconName::ArrowLeft)
                                    .full_width(true)
                                    .trailing_element(crate::components::KbdBadge::new("Esc"))
                                    .on_click(cx.listener(|_this, _, window, cx| {
                                        window.dispatch_action(Box::new(CloseSettings), cx);
                                    })),
                            ),
                    ),
            )
            .child(
                div()
                    .id("settings-content-area")
                    .flex_1()
                    .h_full()
                    .flex()
                    .flex_col()
                    .child(self.render_content(cx)),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::{SettingsSectionId, SettingsView};
    use crate::keymap::KeymapConfig;
    use crate::store::NoteStore;
    use crate::theme::{ActiveTheme, Theme};
    use crate::updater::UpdateManager;
    use gpui::{AppContext, Entity, TestAppContext};

    fn add_settings(cx: &mut TestAppContext) -> Entity<SettingsView> {
        let (view, _) = cx.add_window_view(|_, cx| {
            let store = cx.new(|_| NoteStore::new());
            let updater = cx.new(UpdateManager::new);
            SettingsView::new(store, updater, cx)
        });
        view
    }

    #[test]
    fn settings_nav_lists_all_sections() {
        let mut cx = TestAppContext::single();
        cx.update(|cx| {
            cx.set_global(ActiveTheme(Theme::dark()));
        });

        let view = add_settings(&mut cx);
        let cx = &mut cx;
        cx.run_until_parked();

        assert_eq!(
            view.read_with(cx, |v, _| v.active_section()),
            SettingsSectionId::Account
        );
        view.update(cx, |v, cx| {
            v.select_section(SettingsSectionId::About, cx);
        });
        cx.run_until_parked();
        assert_eq!(
            view.read_with(cx, |v, _| v.active_section()),
            SettingsSectionId::About
        );
    }

    #[test]
    fn settings_account_renders_rows() {
        let mut cx = TestAppContext::single();
        cx.update(|cx| {
            cx.set_global(ActiveTheme(Theme::dark()));
        });

        let view = add_settings(&mut cx);
        let cx = &mut cx;
        cx.run_until_parked();

        let store = view.read_with(cx, |v, _| v.test_store());

        assert_eq!(
            store.read_with(cx, |s, _| s.user_id().to_string()),
            crate::store::LOCAL_USER_ID
        );
        assert!(store.read_with(cx, |s, _| s.active_notes().is_empty()));
        assert!(store.read_with(cx, |s, _| s.folder_tree().is_empty()));
    }

    #[test]
    fn settings_keybinding_capture_and_reset_flow() {
        let mut cx = TestAppContext::single();
        cx.update(|cx| {
            cx.set_global(ActiveTheme(Theme::dark()));
        });

        let view = add_settings(&mut cx);
        let cx = &mut cx;
        cx.run_until_parked();

        // Begin capture on one action.
        view.update(cx, |v, cx| {
            v.begin_capture("tnotes::ToggleFps", "Toggle Performance HUD", cx);
        });
        cx.run_until_parked();
        assert!(
            view.read_with(cx, |v, _| v.capturing().is_some()),
            "capture should be active"
        );

        // Escape cancels without touching the keymap.
        let before = view.read_with(cx, |v, _| {
            v.keymap()
                .get_key_for_action("tnotes::ToggleFps")
                .map(|(k, _)| k)
        });
        view.update(cx, |v, cx| v.cancel_capture(cx));
        cx.run_until_parked();
        assert!(view.read_with(cx, |v, _| v.capturing().is_none()));
        assert_eq!(
            view.read_with(cx, |v, _| v
                .keymap()
                .get_key_for_action("tnotes::ToggleFps")
                .map(|(k, _)| k)),
            before
        );

        // Reset restores defaults from a mutated config.
        view.update(cx, |v, cx| {
            v.keymap_mut_for_test("tnotes::ToggleFps", "f9", None);
            v.reset_keymap(cx);
        });
        cx.run_until_parked();
        assert_eq!(
            view.read_with(cx, |v, _| v
                .keymap()
                .get_key_for_action("tnotes::ToggleFps")
                .map(|(k, _)| k)),
            Some("f3".to_string())
        );
    }

    #[test]
    fn settings_view_escape_dispatches_close_settings() {
        let mut cx = TestAppContext::single();
        cx.update(|cx| {
            cx.set_global(ActiveTheme(Theme::dark()));
            KeymapConfig::default_config().bind_to_gpui(cx);
        });

        let (_view, cx) = cx.add_window_view(|window, cx| {
            let store = cx.new(|_| NoteStore::new());
            let updater = cx.new(UpdateManager::new);
            let settings = SettingsView::new(store, updater, cx);
            settings.focus(window);
            settings
        });
        cx.run_until_parked();

        cx.simulate_keystrokes("escape");
        cx.run_until_parked();
    }

    #[test]
    fn settings_view_mouse_back_dispatches_close_settings() {
        let mut cx = TestAppContext::single();
        cx.update(|cx| {
            cx.set_global(ActiveTheme(Theme::dark()));
            KeymapConfig::default_config().bind_to_gpui(cx);
        });

        let (_view, cx) = cx.add_window_view(|_, cx| {
            let store = cx.new(|_| NoteStore::new());
            let updater = cx.new(UpdateManager::new);
            SettingsView::new(store, updater, cx)
        });
        cx.run_until_parked();

        cx.simulate_event(gpui::MouseDownEvent {
            button: gpui::MouseButton::Navigate(gpui::NavigationDirection::Back),
            position: gpui::point(gpui::px(50.), gpui::px(50.)),
            modifiers: Default::default(),
            click_count: 1,
            first_mouse: false,
        });
        cx.run_until_parked();
    }

    #[test]
    fn settings_nav_supports_all_sections_and_workspace_groups() {
        let mut cx = TestAppContext::single();
        cx.update(|cx| {
            cx.set_global(ActiveTheme(Theme::dark()));
        });

        let view = add_settings(&mut cx);
        let cx = &mut cx;
        cx.run_until_parked();

        for section in SettingsSectionId::ALL {
            view.update(cx, |v, cx| {
                v.select_section(section, cx);
            });
            cx.run_until_parked();
            assert_eq!(view.read_with(cx, |v, _| v.active_section()), section);
        }

        assert_eq!(SettingsSectionId::ALL.len(), 7);
        assert_eq!(SettingsSectionId::WORKSPACE.len(), 2);
        assert_eq!(SettingsSectionId::SYSTEM.len(), 5);
    }

    #[test]
    fn settings_theme_toggle_switches_mode() {
        let dark = Theme::dark();
        assert!(dark.is_dark());

        let light = Theme::light();
        assert!(!light.is_dark());
    }

    #[test]
    fn settings_search_matches_query_filters_sections() {
        let mut cx = TestAppContext::single();
        cx.update(|cx| {
            cx.set_global(ActiveTheme(Theme::dark()));
        });

        let view = add_settings(&mut cx);
        let cx = &mut cx;
        cx.run_until_parked();

        assert!(view.read_with(cx, |v, _| {
            v.section_matches_query(SettingsSectionId::Sync, "sync")
        }));
        assert!(view.read_with(cx, |v, _| {
            v.section_matches_query(SettingsSectionId::Storage, "sqlite")
        }));
        assert!(view.read_with(cx, |v, _| {
            v.section_matches_query(SettingsSectionId::Developer, "benchmark")
        }));
        assert!(view.read_with(cx, |v, _| {
            v.section_matches_query(SettingsSectionId::Updates, "channel")
        }));
        assert!(!view.read_with(cx, |v, _| {
            v.section_matches_query(SettingsSectionId::Account, "xyznonexistent")
        }));
    }

    #[test]
    fn settings_benchmark_creation_and_deletion() {
        let mut cx = TestAppContext::single();
        cx.update(|cx| {
            cx.set_global(ActiveTheme(Theme::dark()));
        });

        let view = add_settings(&mut cx);
        let cx = &mut cx;
        cx.run_until_parked();

        let store = view.read_with(cx, |v, _| v.test_store());

        store.update(cx, |s, cx| {
            let (notes, folders) = s.create_benchmark_notes(25, cx);
            assert_eq!(notes, 25);
            assert_eq!(folders, 3);
        });
        cx.run_until_parked();

        assert_eq!(store.read_with(cx, |s, _| s.active_notes().len()), 25);

        store.update(cx, |s, cx| {
            let deleted = s.delete_benchmark_notes(cx);
            assert_eq!(deleted, 25);
        });
        cx.run_until_parked();

        assert_eq!(store.read_with(cx, |s, _| s.active_notes().len()), 0);
    }

    #[test]
    fn settings_benchmark_large_batch_creation() {
        let mut cx = TestAppContext::single();
        cx.update(|cx| {
            cx.set_global(ActiveTheme(Theme::dark()));
        });

        let view = add_settings(&mut cx);
        let cx = &mut cx;
        cx.run_until_parked();

        let store = view.read_with(cx, |v, _| v.test_store());

        store.update(cx, |s, cx| {
            let (notes, folders) = s.create_benchmark_notes(5000, cx);
            assert_eq!(notes, 5000);
            assert_eq!(folders, 500);
        });
        cx.run_until_parked();

        assert_eq!(store.read_with(cx, |s, _| s.active_note_count()), 5000);

        store.update(cx, |s, cx| {
            let deleted = s.delete_benchmark_notes(cx);
            assert_eq!(deleted, 5000);
        });
        cx.run_until_parked();

        assert_eq!(store.read_with(cx, |s, _| s.active_note_count()), 0);
    }
}
