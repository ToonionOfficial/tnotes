use crate::components::fps_overlay::{fps_overlay, FpsTracker, ToggleFps, ToggleStressTest};
use crate::state::{ActiveModal, ActiveScreen, AppState, WorkspaceViewMode};
use crate::views::editor::EditorView;
use crate::views::folder_dashboard::FolderDashboardView;
use crate::views::modals::command_palette_modal;
use crate::views::settings::SettingsView;
use crate::views::sidebar::SidebarView;
use gpui::*;
use std::time::Duration;

actions!(
    tnotes,
    [
        ToggleCommandPalette,
        ToggleSettings,
        CreateNewNote,
        CloseModal
    ]
);

pub struct TNotesWorkspace {
    pub app_state: Entity<AppState>,
    pub sidebar: Entity<SidebarView>,
    pub dashboard: Entity<FolderDashboardView>,
    pub editor: Entity<EditorView>,
    pub settings: Entity<SettingsView>,
    pub fps_tracker: FpsTracker,
    pub focus_handle: FocusHandle,
}

impl TNotesWorkspace {
    pub fn new(app_state: Entity<AppState>, cx: &mut Context<Self>) -> Self {
        let sidebar = cx.new(|cx| SidebarView::new(app_state.clone(), cx));
        let dashboard = cx.new(|cx| FolderDashboardView::new(app_state.clone(), cx));
        let editor = cx.new(|cx| EditorView::new(app_state.clone(), cx));
        let settings = cx.new(|cx| SettingsView::new(app_state.clone(), cx));
        let focus_handle = cx.focus_handle();

        cx.observe(&app_state, |_this, _app_state, cx| {
            cx.notify();
        })
        .detach();

        Self {
            app_state,
            sidebar,
            dashboard,
            editor,
            settings,
            fps_tracker: FpsTracker::new(),
            focus_handle,
        }
    }

    pub fn toggle_fps(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        self.fps_tracker.toggle_visibility();
        cx.notify();
    }

    pub fn toggle_stress_test(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        self.fps_tracker.toggle_stress_test();
        if self.fps_tracker.is_stress_testing {
            let view = cx.entity().clone();
            cx.spawn(async move |_, cx| {
                while cx.update(|cx| view.read(cx).fps_tracker.is_stress_testing) {
                    cx.background_executor().timer(Duration::from_micros(6500)).await;
                    cx.update(|cx| {
                        view.update(cx, |_this, cx| {
                            cx.notify();
                        });
                    });
                }
            })
            .detach();
        }
        cx.notify();
    }
}

impl Render for TNotesWorkspace {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if !self.focus_handle.is_focused(window) {
            window.focus(&self.focus_handle, cx);
        }

        let metrics = self.fps_tracker.tick().clone();
        let is_visible = self.fps_tracker.is_visible;
        let is_stress_testing = self.fps_tracker.is_stress_testing;
        let app_state_read = self.app_state.read(cx);
        let active_screen = app_state_read.active_screen;
        let active_modal = app_state_read.active_modal;
        let app_state_entity = self.app_state.clone();

        div()
            .id("tnotes-workspace-root")
            .track_focus(&self.focus_handle)
            .key_context("Workspace")
            .relative()
            .flex()
            .flex_row()
            .size_full()
            .bg(rgb(0x141318))
            .text_color(rgb(0xe6e1e9))
            .on_action(cx.listener(|this, _: &ToggleFps, window, cx| {
                this.toggle_fps(window, cx);
            }))
            .on_action(cx.listener(|this, _: &ToggleStressTest, window, cx| {
                this.toggle_stress_test(window, cx);
            }))
            .on_action(cx.listener(|this, _: &ToggleCommandPalette, _window, cx| {
                this.app_state.update(cx, |state, cx| {
                    state.toggle_command_palette(cx);
                });
            }))
            .on_action(cx.listener(|this, _: &ToggleSettings, _window, cx| {
                this.app_state.update(cx, |state, cx| {
                    state.toggle_settings(cx);
                });
            }))
            .on_action(cx.listener(|this, _: &CreateNewNote, _window, cx| {
                this.app_state.update(cx, |state, cx| {
                    state.note_store.update(cx, |store, cx| {
                        store.create_note("Untitled Note", "<p></p>", None, cx);
                    });
                });
            }))
            .on_action(cx.listener(|this, _: &CloseModal, _window, cx| {
                this.app_state.update(cx, |state, cx| {
                    if state.active_modal != ActiveModal::None {
                        state.close_modal(cx);
                    } else if state.active_screen == ActiveScreen::Settings {
                        state.set_screen(ActiveScreen::Workspace, cx);
                    }
                });
            }))
            .child(match active_screen {
                ActiveScreen::Settings => self.settings.clone().into_any_element(),
                ActiveScreen::Workspace | ActiveScreen::Auth => {
                    let workspace_mode = app_state_read.workspace_mode;
                    div()
                        .flex()
                        .flex_row()
                        .size_full()
                        .child(self.sidebar.clone())
                        .child(
                            div()
                                .flex_1()
                                .h_full()
                                .child(match workspace_mode {
                                    WorkspaceViewMode::Dashboard => {
                                        self.dashboard.clone().into_any_element()
                                    }
                                    WorkspaceViewMode::Editor => {
                                        self.editor.clone().into_any_element()
                                    }
                                }),
                        )
                        .into_any_element()
                }
            })
            .children(if is_visible {
                Some(fps_overlay(
                    &self.fps_tracker,
                    &metrics,
                    is_stress_testing,
                    cx.listener(|this, _, window, cx| {
                        this.toggle_stress_test(window, cx);
                    }),
                ))
            } else {
                None
            })
            .children(match active_modal {
                ActiveModal::CommandPalette => Some(
                    command_palette_modal(
                        app_state_entity.clone(),
                        cx.listener(|this, _, _window, cx| {
                            this.app_state.update(cx, |state, cx| state.close_modal(cx));
                        }),
                    )
                    .into_any_element(),
                ),
                ActiveModal::None => None,
            })
    }
}
