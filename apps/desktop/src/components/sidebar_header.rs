use crate::components::icons::*;
use gpui::*;

pub fn sidebar_header(
    title: impl Into<SharedString>,
    on_add_note: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    let title = title.into();

    div()
        .id("sidebar-header")
        .flex()
        .items_center()
        .justify_between()
        .px_2()
        .py_1p5()
        .child(
            // Left: Brand Icon + Title
            div()
                .id("sidebar-header-brand")
                .flex()
                .items_center()
                .gap_2()
                .cursor_pointer()
                .child(
                    div()
                        .w_6()
                        .h_6()
                        .rounded_md()
                        .overflow_hidden()
                        .child(app_logo().size_6()),
                )
                .child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(rgb(0xe6e1e9)) // Foreground
                        .child(title),
                ),
        )
        .child(
            // Right: Quick Action Buttons (+ and Sidebar toggle)
            div()
                .flex()
                .items_center()
                .gap_1()
                .child(
                    div()
                        .id("add-item-header")
                        .p_1()
                        .rounded_md()
                        .text_color(rgb(0xc6c2cd))
                        .hover(|s| s.bg(rgb(0x201f24)).text_color(rgb(0xe6e1e9)))
                        .cursor_pointer()
                        .on_click(on_add_note)
                        .child(icon_plus().size_3p5()),
                )
                .child(
                    div()
                        .id("toggle-sidebar-header")
                        .p_1()
                        .rounded_md()
                        .text_color(rgb(0xc6c2cd))
                        .hover(|s| s.bg(rgb(0x201f24)).text_color(rgb(0xe6e1e9)))
                        .cursor_pointer()
                        .child(icon_panel_left().size_3p5()),
                ),
        )
}
