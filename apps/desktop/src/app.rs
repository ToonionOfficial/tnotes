use gpui::prelude::FluentBuilder;
use gpui::*;
use crate::assets::DesktopAssets;
use crate::components::{fps_monitor, Icon, IconName};
use crate::keymap::{
    CloseSettings, DeleteNote, FocusSearch, KeymapConfig, NavigateBack, NavigateForward, NewNote,
    OpenSettings, PinNote, ToggleFps, ToggleSidebar,
};
use crate::theme::{ActiveTheme, Theme, ThemeExt};
use crate::views::{NavigationLocation, SettingsView, SidebarView};

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
        let theme = cx.theme().clone();

        let (active_location, can_back, can_forward, selected_note, starred_notes, trash_notes) = {
            let sidebar_view = self.sidebar.read(cx);
            (
                sidebar_view.active_location().clone(),
                sidebar_view.can_navigate_back(),
                sidebar_view.can_navigate_forward(),
                sidebar_view.selected_note().cloned(),
                sidebar_view.starred_notes().into_iter().cloned().collect::<Vec<_>>(),
                sidebar_view.trash_notes().into_iter().cloned().collect::<Vec<_>>(),
            )
        };

        let right_pane = match active_location {
            NavigationLocation::Note(_) => {
                self.render_note_pane(selected_note.as_ref(), can_back, can_forward, cx)
            }
            NavigationLocation::Starred => {
                self.render_starred_pane(&starred_notes, can_back, can_forward, cx)
            }
            NavigationLocation::Trash => {
                self.render_trash_pane(&trash_notes, can_back, can_forward, cx)
            }
        };

        let is_settings = self.active_screen == AppScreen::Settings;

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
    fn render_nav_buttons(
        &self,
        can_back: bool,
        can_forward: bool,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let theme = cx.theme();
        div()
            .flex()
            .items_center()
            .gap_1()
            .child(
                div()
                    .id("note-nav-back-btn")
                    .w(px(24.))
                    .h(px(24.))
                    .rounded(px(4.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_color(theme.muted_foreground)
                    .when(can_back, |this| {
                        this.cursor_pointer()
                            .hover(|s| s.bg(theme.secondary).text_color(theme.foreground))
                            .on_click(cx.listener(|this, _, _window, cx| {
                                this.sidebar.update(cx, |sidebar, cx| {
                                    sidebar.navigate_back(cx);
                                });
                            }))
                    })
                    .when(!can_back, |this| this.opacity(0.35).cursor_default())
                    .child(Icon::new(IconName::ArrowLeft).size(px(13.))),
            )
            .child(
                div()
                    .id("note-nav-forward-btn")
                    .w(px(24.))
                    .h(px(24.))
                    .rounded(px(4.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_color(theme.muted_foreground)
                    .when(can_forward, |this| {
                        this.cursor_pointer()
                            .hover(|s| s.bg(theme.secondary).text_color(theme.foreground))
                            .on_click(cx.listener(|this, _, _window, cx| {
                                this.sidebar.update(cx, |sidebar, cx| {
                                    sidebar.navigate_forward(cx);
                                });
                            }))
                    })
                    .when(!can_forward, |this| this.opacity(0.35).cursor_default())
                    .child(Icon::new(IconName::ArrowRight).size(px(13.))),
            )
    }

    fn render_note_pane(
        &self,
        note: Option<&crate::views::NoteItem>,
        can_back: bool,
        can_forward: bool,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let theme = cx.theme().clone();

        if let Some(note) = note {
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
                                .gap_3()
                                .child(self.render_nav_buttons(can_back, can_forward, cx))
                                .child(div().w(px(1.)).h(px(14.)).bg(theme.border))
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
                                ),
                        )
                        .child(
                            div()
                                .id("note-pin-btn")
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
                                .child(Icon::new(IconName::Star).size(px(14.)))
                                .on_click(cx.listener({
                                    let note_id = note.id.clone();
                                    move |this, _, _window, cx| {
                                        this.sidebar.update(cx, |s, cx| {
                                            s.toggle_note_pin(&note_id, cx);
                                        });
                                    }
                                })),
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
                .into_any_element()
        } else {
            div()
                .flex_1()
                .h_full()
                .flex()
                .flex_col()
                .child(
                    div()
                        .w_full()
                        .h(px(48.))
                        .px(px(24.))
                        .flex()
                        .items_center()
                        .border_b_1()
                        .border_color(theme.border)
                        .child(self.render_nav_buttons(can_back, can_forward, cx)),
                )
                .child(
                    div()
                        .flex_1()
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
                        ),
                )
                .into_any_element()
        }
    }

    fn render_starred_pane(
        &self,
        starred_notes: &[crate::views::NoteItem],
        can_back: bool,
        can_forward: bool,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let theme = cx.theme().clone();
        let count = starred_notes.len();

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
                            .gap_3()
                            .child(self.render_nav_buttons(can_back, can_forward, cx))
                            .child(div().w(px(1.)).h(px(14.)).bg(theme.border))
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .text_size(px(12.))
                                    .text_color(theme.muted_foreground)
                                    .child("TNotes")
                                    .child(div().text_size(px(10.)).child("/"))
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .gap_1p5()
                                            .text_color(theme.foreground)
                                            .font_weight(FontWeight::MEDIUM)
                                            .child(Icon::new(IconName::Star).size(px(13.)).color(theme.primary))
                                            .child("Starred"),
                                    ),
                            ),
                    )
                    .child(
                        div()
                            .px_2()
                            .py(px(2.))
                            .rounded(px(10.))
                            .bg(theme.secondary)
                            .text_size(px(11.))
                            .text_color(theme.muted_foreground)
                            .child(format!("{count} note{}", if count == 1 { "" } else { "s" })),
                    ),
            )
            .child(
                div()
                    .id("starred-pane-scroll")
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
                            .gap_5()
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_3()
                                    .child(
                                        div()
                                            .w(px(36.))
                                            .h(px(36.))
                                            .rounded(px(8.))
                                            .bg(theme.secondary)
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .child(Icon::new(IconName::Star).size(px(18.)).color(theme.primary)),
                                    )
                                    .child(
                                        div()
                                            .flex()
                                            .flex_col()
                                            .child(
                                                div()
                                                    .text_size(px(24.))
                                                    .font_weight(FontWeight::BOLD)
                                                    .text_color(theme.foreground)
                                                    .child("Starred Notes"),
                                            )
                                            .child(
                                                div()
                                                    .text_size(px(13.))
                                                    .text_color(theme.muted_foreground)
                                                    .child("Quick access to your important and favorite notes"),
                                            ),
                                    ),
                            )
                            .child(div().w_full().h(px(1.)).bg(theme.border))
                            .when(starred_notes.is_empty(), |this| {
                                this.child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .items_center()
                                        .justify_center()
                                        .py(px(64.))
                                        .gap_3()
                                        .child(
                                            Icon::new(IconName::Star)
                                                .size(px(40.))
                                                .color(theme.muted_foreground),
                                        )
                                        .child(
                                            div()
                                                .text_size(px(16.))
                                                .font_weight(FontWeight::SEMIBOLD)
                                                .text_color(theme.foreground)
                                                .child("No starred notes yet"),
                                        )
                                        .child(
                                            div()
                                                .text_size(px(13.))
                                                .text_color(theme.muted_foreground)
                                                .child("Star any note to quickly find it here."),
                                        ),
                                )
                            })
                            .when(!starred_notes.is_empty(), |this| {
                                this.child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap_2p5()
                                        .children(starred_notes.iter().map(|note| {
                                            let note_id = note.id.clone();
                                            div()
                                                .id(SharedString::from(format!("starred-card-{}", note.id)))
                                                .w_full()
                                                .p(px(14.))
                                                .rounded(px(8.))
                                                .bg(theme.card)
                                                .border_1()
                                                .border_color(theme.border)
                                                .cursor_pointer()
                                                .hover(|s| s.bg(theme.secondary).border_color(theme.ring))
                                                .on_click(cx.listener({
                                                    let note_id = note_id.clone();
                                                    move |this, _, _window, cx| {
                                                        this.sidebar.update(cx, |s, cx| {
                                                            s.select_note(&note_id, cx);
                                                        });
                                                    }
                                                }))
                                                .child(
                                                    div()
                                                        .flex()
                                                        .flex_col()
                                                        .gap_1p5()
                                                        .child(
                                                            div()
                                                                .flex()
                                                                .items_center()
                                                                .justify_between()
                                                                .child(
                                                                    div()
                                                                        .flex()
                                                                        .items_center()
                                                                        .gap_2()
                                                                        .child(Icon::new(IconName::Star).size(px(13.)).color(theme.primary))
                                                                        .child(
                                                                            div()
                                                                                .text_size(px(14.))
                                                                                .font_weight(FontWeight::SEMIBOLD)
                                                                                .text_color(theme.foreground)
                                                                                .child(note.title.clone()),
                                                                        ),
                                                                )
                                                                .children(note.folder_name.as_ref().map(|f| {
                                                                    div()
                                                                        .px_2()
                                                                        .py(px(1.5))
                                                                        .rounded(px(4.))
                                                                        .bg(theme.muted)
                                                                        .text_size(px(11.))
                                                                        .text_color(theme.muted_foreground)
                                                                        .child(f.clone())
                                                                })),
                                                        )
                                                        .child(
                                                            div()
                                                                .text_size(px(12.5))
                                                                .text_color(theme.muted_foreground)
                                                                .line_clamp(2)
                                                                .child(note.snippet.clone()),
                                                        )
                                                        .child(
                                                            div()
                                                                .flex()
                                                                .items_center()
                                                                .justify_between()
                                                                .pt_1()
                                                                .child(
                                                                    div()
                                                                        .text_size(px(11.))
                                                                        .text_color(theme.muted_foreground)
                                                                        .child(format!("Last edited {}", note.updated_at)),
                                                                )
                                                                .child(
                                                                    div()
                                                                        .text_size(px(11.))
                                                                        .text_color(theme.primary)
                                                                        .child("Open note →"),
                                                                ),
                                                        ),
                                                )
                                        })),
                                )
                            }),
                    ),
            )
            .into_any_element()
    }

    fn render_trash_pane(
        &self,
        trash_notes: &[crate::views::NoteItem],
        can_back: bool,
        can_forward: bool,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let theme = cx.theme().clone();
        let count = trash_notes.len();

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
                            .gap_3()
                            .child(self.render_nav_buttons(can_back, can_forward, cx))
                            .child(div().w(px(1.)).h(px(14.)).bg(theme.border))
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .text_size(px(12.))
                                    .text_color(theme.muted_foreground)
                                    .child("TNotes")
                                    .child(div().text_size(px(10.)).child("/"))
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .gap_1p5()
                                            .text_color(theme.foreground)
                                            .font_weight(FontWeight::MEDIUM)
                                            .child(Icon::new(IconName::Trash2).size(px(13.)).color(theme.muted_foreground))
                                            .child("Trash"),
                                    ),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .when(!trash_notes.is_empty(), |this| {
                                this.child(
                                    div()
                                        .id("empty-trash-header-btn")
                                        .px_2p5()
                                        .py_1()
                                        .rounded(px(5.))
                                        .bg(gpui::transparent_black())
                                        .border_1()
                                        .border_color(theme.border)
                                        .cursor_pointer()
                                        .hover(|s| s.bg(theme.destructive).border_color(theme.destructive).text_color(theme.destructive_foreground))
                                        .text_size(px(11.5))
                                        .text_color(theme.muted_foreground)
                                        .child("Empty Trash")
                                        .on_click(cx.listener(|this, _, _window, cx| {
                                            this.sidebar.update(cx, |s, cx| {
                                                s.empty_trash(cx);
                                            });
                                        })),
                                )
                            })
                            .child(
                                div()
                                    .px_2()
                                    .py(px(2.))
                                    .rounded(px(10.))
                                    .bg(theme.secondary)
                                    .text_size(px(11.))
                                    .text_color(theme.muted_foreground)
                                    .child(format!("{count} item{}", if count == 1 { "" } else { "s" })),
                            ),
                    ),
            )
            .child(
                div()
                    .id("trash-pane-scroll")
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
                            .gap_5()
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_3()
                                    .child(
                                        div()
                                            .w(px(36.))
                                            .h(px(36.))
                                            .rounded(px(8.))
                                            .bg(theme.secondary)
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .child(Icon::new(IconName::Trash2).size(px(18.)).color(theme.muted_foreground)),
                                    )
                                    .child(
                                        div()
                                            .flex()
                                            .flex_col()
                                            .child(
                                                div()
                                                    .text_size(px(24.))
                                                    .font_weight(FontWeight::BOLD)
                                                    .text_color(theme.foreground)
                                                    .child("Trash"),
                                            )
                                            .child(
                                                div()
                                                    .text_size(px(13.))
                                                    .text_color(theme.muted_foreground)
                                                    .child("Deleted notes remain here until permanently removed"),
                                            ),
                                    ),
                            )
                            .child(div().w_full().h(px(1.)).bg(theme.border))
                            .when(trash_notes.is_empty(), |this| {
                                this.child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .items_center()
                                        .justify_center()
                                        .py(px(64.))
                                        .gap_3()
                                        .child(
                                            Icon::new(IconName::Trash2)
                                                .size(px(40.))
                                                .color(theme.muted_foreground),
                                        )
                                        .child(
                                            div()
                                                .text_size(px(16.))
                                                .font_weight(FontWeight::SEMIBOLD)
                                                .text_color(theme.foreground)
                                                .child("Trash is empty"),
                                        )
                                        .child(
                                            div()
                                                .text_size(px(13.))
                                                .text_color(theme.muted_foreground)
                                                .child("Notes you delete will appear here."),
                                        ),
                                )
                            })
                            .when(!trash_notes.is_empty(), |this| {
                                this.child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap_2p5()
                                        .children(trash_notes.iter().map(|note| {
                                            let note_id = note.id.clone();
                                            div()
                                                .id(SharedString::from(format!("trash-card-{}", note.id)))
                                                .w_full()
                                                .p(px(14.))
                                                .rounded(px(8.))
                                                .bg(theme.card)
                                                .border_1()
                                                .border_color(theme.border)
                                                .child(
                                                    div()
                                                        .flex()
                                                        .flex_col()
                                                        .gap_1p5()
                                                        .child(
                                                            div()
                                                                .flex()
                                                                .items_center()
                                                                .justify_between()
                                                                .child(
                                                                    div()
                                                                        .flex()
                                                                        .items_center()
                                                                        .gap_2()
                                                                        .child(Icon::new(IconName::FileText).size(px(13.)).color(theme.muted_foreground))
                                                                        .child(
                                                                            div()
                                                                                .text_size(px(14.))
                                                                                .font_weight(FontWeight::SEMIBOLD)
                                                                                .text_color(theme.foreground)
                                                                                .child(note.title.clone()),
                                                                        ),
                                                                )
                                                                .children(note.folder_name.as_ref().map(|f| {
                                                                    div()
                                                                        .px_2()
                                                                        .py(px(1.5))
                                                                        .rounded(px(4.))
                                                                        .bg(theme.muted)
                                                                        .text_size(px(11.))
                                                                        .text_color(theme.muted_foreground)
                                                                        .child(f.clone())
                                                                })),
                                                        )
                                                        .child(
                                                            div()
                                                                .text_size(px(12.5))
                                                                .text_color(theme.muted_foreground)
                                                                .line_clamp(2)
                                                                .child(note.snippet.clone()),
                                                        )
                                                        .child(
                                                            div()
                                                                .flex()
                                                                .items_center()
                                                                .justify_between()
                                                                .pt_1()
                                                                .child(
                                                                    div()
                                                                        .text_size(px(11.))
                                                                        .text_color(theme.muted_foreground)
                                                                        .child(format!("Deleted • {}", note.updated_at)),
                                                                )
                                                                .child(
                                                                    div()
                                                                        .flex()
                                                                        .items_center()
                                                                        .gap_3()
                                                                        .child(
                                                                            div()
                                                                                .id(SharedString::from(format!("restore-btn-{}", note.id)))
                                                                                .cursor_pointer()
                                                                                .hover(|s| s.text_color(theme.foreground))
                                                                                .text_size(px(12.))
                                                                                .text_color(theme.primary)
                                                                                .child("Restore")
                                                                                .on_click(cx.listener({
                                                                                    let note_id = note_id.clone();
                                                                                    move |this, _, _window, cx| {
                                                                                        this.sidebar.update(cx, |s, cx| {
                                                                                            s.restore_note(&note_id, cx);
                                                                                        });
                                                                                    }
                                                                                })),
                                                                        )
                                                                        .child(
                                                                            div()
                                                                                .id(SharedString::from(format!("delete-perm-btn-{}", note.id)))
                                                                                .cursor_pointer()
                                                                                .hover(|s| s.text_color(theme.destructive))
                                                                                .text_size(px(12.))
                                                                                .text_color(theme.muted_foreground)
                                                                                .child("Delete Permanently")
                                                                                .on_click(cx.listener({
                                                                                    let note_id = note_id.clone();
                                                                                    move |this, _, _window, cx| {
                                                                                        this.sidebar.update(cx, |s, cx| {
                                                                                            s.permanently_delete_note(&note_id, cx);
                                                                                        });
                                                                                    }
                                                                                })),
                                                                        ),
                                                                ),
                                                        ),
                                                )
                                        })),
                                )
                            }),
                    ),
            )
            .into_any_element()
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
                        let sidebar = cx.new(|cx| SidebarView::new(cx));
                        let settings_view = cx.new(|cx| SettingsView::new(cx));
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
