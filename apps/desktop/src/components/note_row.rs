use crate::components::icons::*;
use gpui::prelude::FluentBuilder;
use gpui::*;

pub fn note_row(
    id_index: u64,
    title: impl Into<SharedString>,
    is_selected: bool,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    let title = title.into();

    div()
        .id(ElementId::NamedInteger("note-row".into(), id_index))
        .flex()
        .items_center()
        .justify_between()
        .px_2()
        .py_1p5()
        .rounded_md()
        .when(is_selected, |s| {
            s.bg(rgb(0x201f24))
                .border_1()
                .border_color(rgb(0xcabeff))
        })
        .when(!is_selected, |s| {
            s.hover(|s| s.bg(rgb(0x201f24)))
        })
        .cursor_pointer()
        .on_click(on_click)
        .child(
            div()
                .flex()
                .items_center()
                .gap_2p5()
                .child(
                    div()
                        .w_4()
                        .h_4()
                        .child(icon_file_text().size_4().text_color(rgb(0x938f99))),
                )
                .child(
                    div()
                        .text_xs()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(rgb(0xe6e1e9))
                        .line_clamp(1)
                        .child(title),
                ),
        )
}
