use std::time::Instant;
use gpui::prelude::FluentBuilder;
use gpui::*;
use crate::assets::DesktopAssets;
use crate::components::fps::fps_monitor_with_duration;
use crate::keymap::{
    CloseSettings, DeleteNote, FocusSearch, KeymapConfig, NavigateBack, NavigateForward, NewNote,
    OpenSettings, PinNote, ToggleFps, ToggleSidebar,
};
use crate::store::{NavigationLocation, NoteStore, LOCAL_USER_ID};
use crate::theme::{ActiveTheme, Theme, ThemeExt};
use crate::views::{
    NoteView, SettingsView, SidebarView, StarredView, TrashView,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum AppScreen {
    #[default]
    Notes,
    Settings,
}

pub struct Tnotes {
    store: Entity<NoteStore>,
    sidebar: Entity<SidebarView>,
    note_view: Entity<NoteView>,
    starred_view: Entity<StarredView>,
    trash_view: Entity<TrashView>,
    settings_view: Entity<SettingsView>,
    active_screen: AppScreen,
    show_fps: bool,
    focus_handle: FocusHandle,
    _subscriptions: Vec<Subscription>,
}

impl Render for Tnotes {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let frame_start = Instant::now();
        let theme = cx.theme();

        let root = div()
            .relative()
            .size_full()
            .flex()
            .bg(theme.background)
            .text_color(theme.foreground)
            .track_focus(&self.focus_handle);

        let content = self.with_actions(root, cx);
        let content = if self.active_screen == AppScreen::Settings {
            content.child(self.render_settings(cx))
        } else {
            content.child(self.render_notes(cx))
        };

        if self.show_fps {
            window.request_animation_frame();
            let draw_duration = frame_start.elapsed();
            content.child(fps_monitor_with_duration(window, cx, draw_duration))
        } else {
            content
        }
    }
}

impl Tnotes {
    fn render_notes(&self, cx: &Context<Self>) -> impl IntoElement {
        let active_location = self.store.read(cx).active_location().clone();

        let right_pane = div()
            .flex_1()
            .h_full()
            .relative()
            .map(|this| match active_location {
                NavigationLocation::Note(_) => this.child(self.note_view.clone()),
                NavigationLocation::Starred => this.child(self.starred_view.clone()),
                NavigationLocation::Trash => this.child(self.trash_view.clone()),
            });

        div()
            .size_full()
            .flex()
            .child(self.sidebar.clone())
            .child(right_pane)
    }

    fn render_settings(&self, _cx: &Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .flex()
            .child(self.settings_view.clone())
    }

    fn with_actions(&self, el: Div, cx: &mut Context<Self>) -> Div {
        el.on_action(cx.listener(|this, _: &ToggleFps, _window, cx| {
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
                if let Some(id) = sidebar.selected_note_id(cx) {
                    sidebar.toggle_note_pin(&id, cx);
                }
            });
        }))
        .on_action(cx.listener(|this, _: &DeleteNote, _window, cx| {
            this.sidebar.update(cx, |sidebar, cx| {
                if let Some(id) = sidebar.selected_note_id(cx) {
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
            cx.listener(|this, _event, window, cx| match this.active_screen {
                AppScreen::Settings => this.close_settings(window, cx),
                AppScreen::Notes => {
                    this.sidebar.update(cx, |sidebar, cx| {
                        sidebar.navigate_back(cx);
                    });
                }
            }),
        )
        .on_mouse_down(
            MouseButton::Navigate(NavigationDirection::Forward),
            cx.listener(|this, _event, _window, cx| match this.active_screen {
                AppScreen::Notes => {
                    this.sidebar.update(cx, |sidebar, cx| {
                        sidebar.navigate_forward(cx);
                    });
                }
                AppScreen::Settings => {}
            }),
        )
    }

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
                        let _ = crate::paths::ensure_dirs();
                        let db_path = crate::paths::database_file();
                        let store = cx.new(|_| match NoteStore::open(&db_path, LOCAL_USER_ID) {
                            Ok(store) => store,
                            Err(err) => {
                                eprintln!(
                                    "tnotes: cannot open database at {} ({err}); starting with an empty store",
                                    db_path.display()
                                );
                                NoteStore::new()
                            }
                        });
                        let sidebar = cx.new(|cx| SidebarView::new(store.clone(), cx));
                        let settings_view = cx.new(|cx| SettingsView::new(store.clone(), cx));
                        let note_view = cx.new(|cx| NoteView::new(store.clone(), cx));
                        let starred_view = cx.new(|cx| StarredView::new(store.clone(), cx));
                        let trash_view = cx.new(|cx| TrashView::new(store.clone(), cx));
                        let focus_handle = cx.focus_handle();
                        window.focus(&focus_handle);

                        cx.new(|cx| {
                            let mut subscriptions = Vec::new();

                            subscriptions.push(cx.observe(&store, |_, _, cx| {
                                cx.notify();
                            }));
                            subscriptions.push(cx.observe(&sidebar, |_, _, cx| {
                                cx.notify();
                            }));

                            Tnotes {
                                store,
                                sidebar,
                                note_view,
                                starred_view,
                                trash_view,
                                settings_view,
                                active_screen: AppScreen::Notes,
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
