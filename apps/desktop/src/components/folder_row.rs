use crate::components::icons::*;
use gpui::*;

pub fn folder_row(
    id_index: u64,
    name: impl Into<SharedString>,
    count: usize,
) -> impl IntoElement {
    let name = name.into();

    div()
        .id(ElementId::NamedInteger("folder-row".into(), id_index))
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
                .gap_2p5()
                .child(
                    div()
                        .w_4()
                        .h_4()
                        .child(icon_folder().size_4().text_color(rgb(0xcabeff))),
                )
                .child(
                    div()
                        .text_xs()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(rgb(0xe6e1e9))
                        .child(name),
                ),
        )
        .child(
            div()
                .text_xs()
                .text_color(rgb(0x938f99))
                .child(format!("{count}")),
        )
}
