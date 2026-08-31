use crate::components::fps_overlay::{fps_overlay, FpsTracker, ToggleFps, ToggleStressTest};
use crate::state::{ActiveModal, AppState};
use crate::views::editor::EditorView;
use crate::views::modals::{command_palette_modal, settings_modal};
use crate::views::sidebar::SidebarView;
use gpui::*;
use std::time::Duration;

actions!(
    tnotes,
    [
        ToggleCommandPalette,
        ToggleSettingsModal,
        CreateNewNote,
        CloseModal
    ]
);

pub struct TNotesWorkspace {
    pub app_state: Entity<AppState>,
    pub sidebar: Entity<SidebarView>,
    pub editor: Entity<EditorView>,
    pub fps_tracker: FpsTracker,
    pub focus_handle: FocusHandle,
}

impl TNotesWorkspace {
    pub fn new(app_state: Entity<AppState>, cx: &mut Context<Self>) -> Self {
        let note_store = app_state.read(cx).note_store.clone();

        let sidebar = cx.new(|cx| SidebarView::new(app_state.clone(), cx));
        let editor = cx.new(|cx| EditorView::new(note_store, cx));
        let focus_handle = cx.focus_handle();

        // Re-render when modal or screen changes
        cx.observe(&app_state, |_this, _app_state, cx| {
            cx.notify();
        })
        .detach();

        Self {
            app_state,
            sidebar,
            editor,
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
        // Ensure root workspace retains focus for global keyboard shortcuts
        if !self.focus_handle.is_focused(window) {
            window.focus(&self.focus_handle, cx);
        }

        let metrics = self.fps_tracker.tick().clone();
        let is_visible = self.fps_tracker.is_visible;
        let is_stress_testing = self.fps_tracker.is_stress_testing;
        let active_modal = self.app_state.read(cx).active_modal;
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
            .on_action(cx.listener(|this, _: &ToggleSettingsModal, _window, cx| {
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
                    state.close_modal(cx);
                });
            }))
            // 1. Sidebar (Fixed width)
            .child(
                div()
                    .w_80()
                    .h_full()
                    .flex_shrink_0()
                    .child(self.sidebar.clone()),
            )
            // 2. Editor (Expanded)
            .child(
                div()
                    .flex_1()
                    .h_full()
                    .child(self.editor.clone()),
            )
            // 3. Floating FPS Overlay
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
            // 4. Modal Overlays (Command Palette / Settings)
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
                ActiveModal::Settings => Some(
                    settings_modal(
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
