use crate::components::icons::*;
use gpui::*;

pub fn sidebar_header(
    title: impl Into<SharedString>,
    on_new_note: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
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
            div()
                .flex()
                .items_center()
                .gap_2p5()
                .child(
                    div()
                        .w_6()
                        .h_6()
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(app_logo().size_6().rounded_md()),
                )
                .child(
                    div()
                        .text_base()
                        .font_weight(FontWeight::BOLD)
                        .text_color(rgb(0xe6e1e9))
                        .child(title),
                ),
        )
        .child(
            div()
                .flex()
                .items_center()
                .gap_1()
                .child(
                    div()
                        .id("sidebar-btn-new-note")
                        .p_1()
                        .rounded_md()
                        .hover(|s| s.bg(rgb(0x201f24)))
                        .cursor_pointer()
                        .on_click(on_new_note)
                        .child(icon_plus().size_4().text_color(rgb(0xc6c2cd))),
                )
                .child(
                    div()
                        .id("sidebar-btn-toggle")
                        .p_1()
                        .rounded_md()
                        .hover(|s| s.bg(rgb(0x201f24)))
                        .cursor_pointer()
                        .child(icon_panel_left().size_4().text_color(rgb(0xc6c2cd))),
                ),
        )
}
