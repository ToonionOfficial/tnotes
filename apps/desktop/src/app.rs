use gpui::prelude::FluentBuilder;
use gpui::*;
use crate::assets::DesktopAssets;
use crate::components::fps_monitor;
use crate::keymap::{
    CloseSettings, DeleteNote, FocusSearch, KeymapConfig, NavigateBack, NavigateForward, NewNote,
    OpenSettings, PinNote, ToggleFps, ToggleSidebar,
};
use crate::theme::{ActiveTheme, Theme, ThemeExt};
use crate::views::{
    NavigationLocation, NoteView, SettingsView, SidebarView, StarredView, TrashView,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum AppScreen {
    #[default]
    Notes,
    Settings,
}

pub struct Tnotes {
    sidebar: Entity<SidebarView>,
    note_view: Entity<NoteView>,
    starred_view: Entity<StarredView>,
    trash_view: Entity<TrashView>,
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

        let active_location = self.sidebar.read(cx).active_location().clone();
        let is_settings = self.active_screen == AppScreen::Settings;

        let is_note = matches!(active_location, NavigationLocation::Note(_));
        let is_starred = active_location == NavigationLocation::Starred;
        let is_trash = active_location == NavigationLocation::Trash;

        let right_pane = div()
            .flex_1()
            .h_full()
            .relative()
            .child(
                div()
                    .size_full()
                    .when(!is_note, |this| this.hidden())
                    .child(self.note_view.clone()),
            )
            .child(
                div()
                    .size_full()
                    .when(!is_starred, |this| this.hidden())
                    .child(self.starred_view.clone()),
            )
            .child(
                div()
                    .size_full()
                    .when(!is_trash, |this| this.hidden())
                    .child(self.trash_view.clone()),
            );

        let main_content = div()
            .size_full()
            .flex()
            .when(is_settings, |this| this.hidden())
            .child(self.sidebar.clone())
            .child(right_pane);

        let settings_content = div()
            .size_full()
            .flex()
            .when(!is_settings, |this| this.hidden())
            .child(self.settings_view.clone());

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
            .on_action(cx.listener(|this, _: &NavigateBack, _window, cx| {
                this.sidebar.update(cx, |sidebar, cx| {
                    sidebar.navigate_back(cx);
                });
            }))
            .on_action(cx.listener(|this, _: &NavigateForward, _window, cx| {
                this.sidebar.update(cx, |sidebar, cx| {
                    sidebar.navigate_forward(cx);
                });
            }))
            .on_mouse_down(
                MouseButton::Navigate(NavigationDirection::Back),
                cx.listener(|this, _event, window, cx| {
                    if this.active_screen == AppScreen::Settings {
                        this.close_settings(window, cx);
                    } else {
                        this.sidebar.update(cx, |sidebar, cx| {
                            sidebar.navigate_back(cx);
                        });
                    }
                }),
            )
            .on_mouse_down(
                MouseButton::Navigate(NavigationDirection::Forward),
                cx.listener(|this, _event, _window, cx| {
                    if this.active_screen == AppScreen::Notes {
                        this.sidebar.update(cx, |sidebar, cx| {
                            sidebar.navigate_forward(cx);
                        });
                    }
                }),
            )
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
            .child(settings_content)
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
                        let note_view = cx.new(|cx| NoteView::new(sidebar.clone(), cx));
                        let starred_view = cx.new(|cx| StarredView::new(sidebar.clone(), cx));
                        let trash_view = cx.new(|cx| TrashView::new(sidebar.clone(), cx));
                        let focus_handle = cx.focus_handle();
                        window.focus(&focus_handle);

                        let app_keymap = keymap.clone();
                        let app = cx.new(|cx| {
                            let mut subscriptions = Vec::new();

                            subscriptions.push(cx.observe(&sidebar, |_, _, cx| {
                                cx.notify();
                            }));

                            Tnotes {
                                sidebar,
                                note_view,
                                starred_view,
                                trash_view,
                                settings_view: settings_view.clone(),
                                active_screen: AppScreen::Notes,
                                keymap: app_keymap,
                                show_fps: false,
                                focus_handle,
                                _subscriptions: subscriptions,
                            }
                        });

                        let app_handle = app.clone();
                        settings_view.update(cx, |view, _| {
                            view.set_on_back(move |window, cx| {
                                app_handle.update(cx, |tnotes, cx| {
                                    tnotes.close_settings(window, cx);
                                });
                            });
                        });

                        app
                    },
                )
                .unwrap();

                cx.activate(true);
            });
    }
}
