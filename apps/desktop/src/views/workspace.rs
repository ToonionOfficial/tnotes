use crate::state::{ActiveModal, ActiveScreen, AppState, WorkspaceViewMode};
use crate::views::editor::EditorView;
use crate::views::folder_dashboard::FolderDashboardView;
use crate::views::modals::command_palette_modal;
use crate::views::settings::SettingsView;
use crate::views::sidebar::SidebarView;
use gpui::prelude::FluentBuilder as _;
use gpui::*;
use gpui_fps::{FpsMonitor, FpsOverlay};

actions!(
    tnotes,
    [
        ToggleCommandPalette,
        ToggleSettings,
        CreateNewNote,
        CloseModal,
        ToggleFps
    ]
);

pub struct TNotesWorkspace {
    pub app_state: Entity<AppState>,
    pub sidebar: Entity<SidebarView>,
    pub dashboard: Entity<FolderDashboardView>,
    pub editor: Entity<EditorView>,
    pub settings: Entity<SettingsView>,
    pub fps_monitor: Entity<FpsMonitor>,
    pub show_fps: bool,
    pub focus_handle: FocusHandle,
}

impl TNotesWorkspace {
    pub fn new(app_state: Entity<AppState>, window: &Window, cx: &mut Context<Self>) -> Self {
        let sidebar = cx.new(|cx| SidebarView::new(app_state.clone(), cx));
        let dashboard = cx.new(|cx| FolderDashboardView::new(app_state.clone(), cx));
        let editor = cx.new(|cx| EditorView::new(app_state.clone(), cx));
        let settings = cx.new(|cx| SettingsView::new(app_state.clone(), cx));
        let fps_monitor = cx.new(|cx| FpsMonitor::new(window, cx).continuous(true));
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
            fps_monitor,
            show_fps: true,
            focus_handle,
        }
    }

    pub fn toggle_fps(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        self.show_fps = !self.show_fps;
        cx.notify();
    }
}

impl Render for TNotesWorkspace {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if !self.focus_handle.is_focused(window) {
            window.focus(&self.focus_handle, cx);
        }

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
            .when(self.show_fps, |this| {
                this.child(FpsOverlay::new(&self.fps_monitor))
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
