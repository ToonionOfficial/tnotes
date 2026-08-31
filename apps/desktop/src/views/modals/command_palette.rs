use crate::components::icons::{icon_plus, icon_search, icon_settings};
use crate::state::AppState;
use gpui::*;

pub fn command_palette_modal(
    app_state: Entity<AppState>,
    on_close: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    div()
        .id("command-palette-backdrop")
        .absolute()
        .inset_0()
        .bg(rgba(0x00000088))
        .flex()
        .items_start()
        .justify_center()
        .pt_24()
        .on_click(on_close)
        .child(
            div()
                .id("command-palette-container")
                .w(px(540.0))
                .max_h(px(400.0))
                .flex()
                .flex_col()
                .bg(rgb(0x201f24))
                .border_1()
                .border_color(rgb(0x302e36))
                .rounded_xl()
                .shadow_xl()
                .overflow_hidden()
                .on_click(|_event, _window, cx| {
                    // Prevent closing when clicking inside the palette box
                    cx.stop_propagation();
                })
                .child(
                    // Search Input header
                    div()
                        .flex()
                        .items_center()
                        .gap_3()
                        .px_4()
                        .py_3p5()
                        .border_b_1()
                        .border_color(rgb(0x302e36))
                        .child(icon_search().size_4().text_color(rgb(0xcabeff)))
                        .child(
                            div()
                                .text_sm()
                                .text_color(rgb(0xe6e1e9))
                                .child("Type a command or search notes..."),
                        )
                        .child(
                            div()
                                .ml_auto()
                                .px_1p5()
                                .py_0p5()
                                .rounded_sm()
                                .bg(rgb(0x2a2930))
                                .text_xs()
                                .text_color(rgb(0x79747e))
                                .child("ESC to close"),
                        ),
                )
                .child(
                    // Command items list
                    div()
                        .id("command-palette-actions-list")
                        .flex()
                        .flex_col()
                        .p_2()
                        .gap_1()
                        .overflow_y_scroll()
                        .child(palette_action_row(
                            "action-new-note",
                            icon_plus().size_4().text_color(rgb(0xa6e3a1)),
                            "Create New Note",
                            "⌘N",
                            {
                                let app_state = app_state.clone();
                                move |_event, _window, cx| {
                                    app_state.update(cx, |state, cx| {
                                        state.note_store.update(cx, |store, cx| {
                                            store.create_note("Untitled Note", "<p></p>", None, cx);
                                        });
                                        state.close_modal(cx);
                                    });
                                }
                            },
                        ))
                        .child(palette_action_row(
                            "action-open-settings",
                            icon_settings().size_4().text_color(rgb(0xcabeff)),
                            "Open Settings & Preferences",
                            "⌘,",
                            {
                                let app_state = app_state.clone();
                                move |_event, _window, cx| {
                                    app_state.update(cx, |state, cx| {
                                        state.set_modal(crate::state::ActiveModal::Settings, cx);
                                    });
                                }
                            },
                        )),
                ),
        )
}

fn palette_action_row(
    id: &'static str,
    icon: impl IntoElement,
    label: impl Into<SharedString>,
    shortcut: &'static str,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    div()
        .id(id)
        .flex()
        .items_center()
        .gap_3()
        .px_3()
        .py_2()
        .rounded_md()
        .cursor_pointer()
        .hover(|s| s.bg(rgb(0x2a2930)))
        .on_click(on_click)
        .child(icon)
        .child(
            div()
                .flex_1()
                .text_xs()
                .font_weight(FontWeight::MEDIUM)
                .text_color(rgb(0xe6e1e9))
                .child(label.into()),
        )
        .child(
            div()
                .text_xs()
                .text_color(rgb(0x79747e))
                .child(shortcut),
        )
}
