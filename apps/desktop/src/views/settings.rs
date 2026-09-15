mod about;
mod account;
mod appearance;
mod components;
mod developer;
mod keybinding_capture;
mod keybindings;
mod storage;
mod sync;

pub use components::SettingsSectionId;

use gpui::*;
use crate::components::{Icon, IconName};
use crate::keymap::{CloseSettings, KeymapConfig, ALL_ACTIONS};
use crate::store::NoteStore;
use crate::theme::ThemeExt;

pub struct SettingsView {
    store: Entity<NoteStore>,
    active_section: SettingsSectionId,
    keymap: KeymapConfig,
    capturing: Option<keybinding_capture::KeybindingCapture>,
    focus_handle: FocusHandle,
    _store_subscription: Subscription,
}

impl SettingsView {
    pub fn new(store: Entity<NoteStore>, cx: &mut Context<Self>) -> Self {
        let store_sub = cx.observe(&store, |_, _, cx| cx.notify());
        Self {
            store,
            active_section: SettingsSectionId::Account,
            keymap: KeymapConfig::load(),
            capturing: None,
            focus_handle: cx.focus_handle(),
            _store_subscription: store_sub,
        }
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
        self.capturing = Some(keybinding_capture::KeybindingCapture::new(
            action_id, label,
        ));
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

    fn render_nav(&self, cx: &mut Context<Self>) -> AnyElement {
        let theme = cx.theme().clone();
        div()
            .flex_1()
            .flex()
            .flex_col()
            .gap(px(2.))
            .children(SettingsSectionId::ALL.iter().map(|section| {
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
                    .child(Icon::new(section.icon()).size(px(14.)))
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
            .into_any_element()
    }

    fn render_content(&self, cx: &mut Context<Self>) -> AnyElement {
        let theme = cx.theme().clone();
        let body = match self.active_section {
            SettingsSectionId::Account => account::render(self, cx),
            SettingsSectionId::Sync => sync::render(self, cx),
            SettingsSectionId::Appearance => appearance::render(self, cx),
            SettingsSectionId::Storage => storage::render(self, cx),
            SettingsSectionId::Keybindings => keybindings::render(self, cx),
            SettingsSectionId::Developer => developer::render(self, cx),
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
                    .gap_4()
                    .child(
                        div()
                            .text_size(px(20.))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(theme.foreground)
                            .child(self.active_section.title()),
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
                                div()
                                    .id("settings-back-btn")
                                    .w_full()
                                    .h(px(34.))
                                    .px_2()
                                    .rounded(px(6.))
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .cursor_pointer()
                                    .hover(|s| s.bg(theme.secondary))
                                    .on_click(cx.listener(|_this, _, window, cx| {
                                        window.dispatch_action(Box::new(CloseSettings), cx);
                                    }))
                                    .child(
                                        Icon::new(IconName::ArrowLeft)
                                            .size(px(14.))
                                            .color(theme.muted_foreground),
                                    )
                                    .child(
                                        div()
                                            .text_size(px(13.))
                                            .font_weight(FontWeight::MEDIUM)
                                            .text_color(theme.foreground)
                                            .child("Back to Notes"),
                                    ),
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
    use gpui::{AppContext, Entity, TestAppContext};

    fn add_settings(cx: &mut TestAppContext) -> Entity<SettingsView> {
        let (view, _) = cx.add_window_view(|_, cx| {
            let store = cx.new(|_| NoteStore::new());
            SettingsView::new(store, cx)
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
            let settings = SettingsView::new(store, cx);
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
            SettingsView::new(store, cx)
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
}
