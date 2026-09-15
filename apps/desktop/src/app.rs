use gpui::prelude::FluentBuilder;
use gpui::*;
use crate::assets::DesktopAssets;
use crate::components::{fps_monitor, Icon, IconName};
use crate::keymap::{
    CloseSettings, DeleteNote, FocusSearch, KeymapConfig, NewNote, OpenSettings, PinNote,
    ToggleFps, ToggleSidebar,
};
use crate::theme::{ActiveTheme, Theme, ThemeExt};
use crate::views::{SettingsView, SidebarView};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum AppScreen {
    #[default]
    Notes,
    Settings,
}

pub struct Tnotes {
    sidebar: Entity<SidebarView>,
    settings_view: Entity<SettingsView>,
    active_screen: AppScreen,
    #[allow(dead_code)]
    keymap: KeymapConfig,
    show_fps: bool,
    focus_handle: FocusHandle,
    _subscriptions: Vec<Subscription>,
}

impl Render for Tnotes {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();

        let right_pane = if let Some(note) = self.sidebar.read(cx).selected_note() {
            let folder_label = note
                .folder_name
                .clone()
                .unwrap_or_else(|| "Notes".to_string());

            div()
                .flex_1()
                .h_full()
                .flex()
                .flex_col()
                .bg(theme.background)
                .child(
                    div()
                        .w_full()
                        .h(px(48.))
                        .px(px(24.))
                        .flex()
                        .items_center()
                        .justify_between()
                        .border_b_1()
                        .border_color(theme.border)
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .text_size(px(12.))
                                .text_color(theme.muted_foreground)
                                .child("TNotes")
                                .child(div().text_size(px(10.)).child("/"))
                                .child(folder_label)
                                .child(div().text_size(px(10.)).child("/"))
                                .child(
                                    div()
                                        .text_color(theme.foreground)
                                        .font_weight(FontWeight::MEDIUM)
                                        .child(note.title.clone()),
                                ),
                        )
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .child(
                                    div()
                                        .w(px(28.))
                                        .h(px(28.))
                                        .rounded(px(5.))
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .cursor_pointer()
                                        .hover(|s| s.bg(theme.secondary).text_color(theme.foreground))
                                        .text_color(if note.is_pinned {
                                            theme.primary
                                        } else {
                                            theme.muted_foreground
                                        })
                                        .child(Icon::new(IconName::Star).size(px(14.))),
                                ),
                        ),
                )
                .child(
                    div()
                        .id("note-reader-scroll")
                        .flex_1()
                        .overflow_y_scroll()
                        .px(px(48.))
                        .py(px(36.))
                        .child(
                            div()
                                .max_w(px(760.))
                                .w_full()
                                .mx_auto()
                                .flex()
                                .flex_col()
                                .gap_4()
                                .child(
                                    div()
                                        .text_size(px(28.))
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(theme.foreground)
                                        .child(note.title.clone()),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap_2()
                                        .text_size(px(12.))
                                        .text_color(theme.muted_foreground)
                                        .child(format!("Last edited {}", note.updated_at)),
                                )
                                .child(div().w_full().h(px(1.)).bg(theme.border))
                                .child(
                                    div()
                                        .text_size(px(15.))
                                        .text_color(theme.foreground)
                                        .line_height(px(24.))
                                        .child(note.snippet.clone()),
                                ),
                        ),
                )
        } else {
            div()
                .flex_1()
                .h_full()
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .gap_3()
                .bg(theme.background)
                .child(
                    Icon::new(IconName::FileText)
                        .size(px(36.))
                        .color(theme.muted_foreground),
                )
                .child(
                    div()
                        .text_size(px(18.))
                        .font_weight(FontWeight::SEMIBOLD)
                        .child("No note selected"),
                )
                .child(
                    div()
                        .text_size(px(13.))
                        .text_color(theme.muted_foreground)
                        .child("Select a note from the sidebar or click 'New Note' to create one"),
                )
        };

        let main_content = div()
            .size_full()
            .flex()
            .child(self.sidebar.clone())
            .child(right_pane);

        div()
            .relative()
            .size_full()
            .flex()
            .bg(theme.background)
            .text_color(theme.foreground)
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(|this, _: &ToggleFps, _window, cx| {
                this.show_fps = !this.show_fps;
                cx.notify();
            }))
            .on_action(cx.listener(|this, _: &ToggleSidebar, _window, cx| {
                this.sidebar.update(cx, |sidebar, cx| {
                    sidebar.toggle_collapsed(cx);
                });
            }))
            .on_action(cx.listener(|this, _: &NewNote, _window, cx| {
                this.sidebar.update(cx, |sidebar, cx| {
                    sidebar.create_new_note(cx);
                });
            }))
            .on_action(cx.listener(|this, _: &FocusSearch, window, cx| {
                this.sidebar.update(cx, |sidebar, cx| {
                    sidebar.focus_search(window, cx);
                });
            }))
            .on_action(cx.listener(|this, _: &OpenSettings, window, cx| {
                this.open_settings(window, cx);
            }))
            .on_action(cx.listener(|this, _: &CloseSettings, window, cx| {
                this.close_settings(window, cx);
            }))
            .on_action(cx.listener(|this, _: &PinNote, _window, cx| {
                this.sidebar.update(cx, |sidebar, cx| {
                    if let Some(id) = sidebar.selected_note_id().map(|s| s.to_string()) {
                        sidebar.toggle_note_pin(&id, cx);
                    }
                });
            }))
            .on_action(cx.listener(|this, _: &DeleteNote, _window, cx| {
                this.sidebar.update(cx, |sidebar, cx| {
                    if let Some(id) = sidebar.selected_note_id().map(|s| s.to_string()) {
                        sidebar.delete_note(&id, cx);
                    }
                });
            }))
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, _window, cx| {
                if event.keystroke.key == "f12"
                    || ((event.keystroke.modifiers.control || event.keystroke.modifiers.platform)
                        && event.keystroke.modifiers.shift
                        && event.keystroke.key == "f")
                {
                    this.show_fps = !this.show_fps;
                    cx.notify();
                }
            }))
            .child(main_content)
            .when(self.active_screen == AppScreen::Settings, |this| {
                this.child(
                    div()
                        .absolute()
                        .inset_0()
                        .bg(theme.background)
                        .child(self.settings_view.clone()),
                )
            })
            .when(self.show_fps, |this| this.child(fps_monitor(window, cx)))
    }
}

impl Tnotes {
    pub fn open_settings(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.active_screen = AppScreen::Settings;
        self.settings_view.read(cx).focus(window);
        cx.notify();
    }

    pub fn close_settings(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.active_screen = AppScreen::Notes;
        window.focus(&self.focus_handle);
        cx.notify();
    }

    pub fn run_app() {
        Application::new()
            .with_assets(DesktopAssets)
            .run(|cx: &mut App| {
                cx.set_global(ActiveTheme(Theme::dark()));

                let keymap = KeymapConfig::load();
                keymap.bind_to_gpui(cx);

                let bounds = Bounds::centered(None, size(px(1080.), px(720.)), cx);

                cx.open_window(
                    WindowOptions {
                        window_bounds: Some(WindowBounds::Windowed(bounds)),
                        ..Default::default()
                    },
                    |window, cx| {
                        let sidebar = cx.new(|cx| SidebarView::new(cx));
                        let settings_view = cx.new(|cx| SettingsView::new(cx));
                        let focus_handle = cx.focus_handle();
                        window.focus(&focus_handle);

                        let app_keymap = keymap.clone();
                        cx.new(|cx| {
                            let mut subscriptions = Vec::new();

                            subscriptions.push(cx.observe(&sidebar, |_, _, cx| {
                                cx.notify();
                            }));

                            Tnotes {
                                sidebar,
                                settings_view,
                                active_screen: AppScreen::Notes,
                                keymap: app_keymap,
                                show_fps: false,
                                focus_handle,
                                _subscriptions: subscriptions,
                            }
                        })
                    },
                )
                .unwrap();

                cx.activate(true);
            });
    }
}
