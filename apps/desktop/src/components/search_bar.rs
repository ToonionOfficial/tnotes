use crate::components::icons::*;
use gpui::*;

pub fn search_bar() -> impl IntoElement {
    div()
        .id("sidebar-search-bar")
        .flex()
        .items_center()
        .justify_between()
        .px_2()
        .py_1p5()
        .rounded_md()
        .hover(|s| s.bg(rgb(0x201f24)))
        .cursor_pointer()
        .child(
            div()
                .flex()
                .items_center()
                .gap_2()
                .child(
                    div()
                        .w_3p5()
                        .h_3p5()
                        .child(icon_search().size_3p5().text_color(rgb(0x938f99))),
                )
                .child(
                    div()
                        .text_xs()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(rgb(0x938f99))
                        .child("Search"),
                ),
        )
        .child(
            div()
                .px_1p5()
                .py_0p5()
                .rounded_sm()
                .bg(rgb(0x201f24))
                .text_xs()
                .text_color(rgb(0x938f99))
                .child("⌘P"),
        )
}
