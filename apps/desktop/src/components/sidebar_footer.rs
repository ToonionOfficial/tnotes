use crate::components::icons::*;
use gpui::*;

pub fn sidebar_footer(
    username: impl Into<SharedString>,
    sync_status: impl Into<SharedString>,
) -> impl IntoElement {
    let username = username.into();
    let sync_status = sync_status.into();

    div()
        .id("sidebar-profile-footer")
        .flex()
        .items_center()
        .justify_between()
        .px_2()
        .py_2()
        .rounded_lg()
        .hover(|s| s.bg(rgb(0x201f24)))
        .cursor_pointer()
        .child(
            // User Account Info
            div()
                .flex()
                .items_center()
                .gap_2p5()
                .child(
                    div()
                        .w_7()
                        .h_7()
                        .rounded_full()
                        .bg(rgb(0x2a2930))
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(icon_user().size_4().text_color(rgb(0xcabeff))),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(rgb(0xe6e1e9))
                                .child(username),
                        )
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_1()
                                .child(div().w_1p5().h_1p5().rounded_full().bg(rgb(0xa6e3a1))) // Green online dot
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(rgb(0x938f99))
                                        .child(sync_status),
                                ),
                        ),
                ),
        )
        .child(
            // Settings Gear icon
            div()
                .p_1()
                .child(icon_settings().size_4().text_color(rgb(0xc6c2cd))),
        )
}
