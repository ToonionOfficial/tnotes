use gpui::*;
use crate::components::{Icon, IconName};
use crate::theme::ThemeExt;

#[allow(dead_code)]
#[derive(IntoElement)]
pub struct NoteCard {
    id: ElementId,
    title: SharedString,
    snippet: SharedString,
    updated_at: SharedString,
    folder_name: Option<SharedString>,
    is_pinned: bool,
    is_active: bool,
    on_select: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
}

#[allow(dead_code)]
impl NoteCard {
    pub fn new(id: impl Into<SharedString>, title: impl Into<SharedString>) -> Self {
        let id_str: SharedString = id.into();
        Self {
            id: ElementId::from(id_str),
            title: title.into(),
            snippet: "".into(),
            updated_at: "".into(),
            folder_name: None,
            is_pinned: false,
            is_active: false,
            on_select: None,
        }
    }

    pub fn snippet(mut self, snippet: impl Into<SharedString>) -> Self {
        self.snippet = snippet.into();
        self
    }

    pub fn updated_at(mut self, time: impl Into<SharedString>) -> Self {
        self.updated_at = time.into();
        self
    }

    pub fn folder_name(mut self, folder: impl Into<SharedString>) -> Self {
        self.folder_name = Some(folder.into());
        self
    }

    pub fn is_pinned(mut self, pinned: bool) -> Self {
        self.is_pinned = pinned;
        self
    }

    pub fn is_active(mut self, active: bool) -> Self {
        self.is_active = active;
        self
    }

    pub fn on_select(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_select = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for NoteCard {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();

        let bg = if self.is_active {
            theme.secondary
        } else {
            theme.card
        };

        let mut el = div()
            .id(self.id)
            .w_full()
            .flex()
            .flex_col()
            .gap(px(4.))
            .px(px(12.))
            .py(px(10.))
            .rounded(px(6.))
            .bg(bg)
            .cursor_pointer()
            .hover(|s| s.bg(theme.secondary))
            .border_1()
            .border_color(if self.is_active {
                theme.ring
            } else {
                gpui::transparent_black()
            })
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap_2()
                    .child(
                        div()
                            .flex_1()
                            .text_size(px(13.5))
                            .font_weight(if self.is_active {
                                FontWeight::SEMIBOLD
                            } else {
                                FontWeight::MEDIUM
                            })
                            .text_color(if self.is_active {
                                theme.foreground
                            } else {
                                theme.foreground
                            })
                            .line_clamp(1)
                            .child(if self.title.is_empty() {
                                "Untitled Note".into()
                            } else {
                                self.title.clone()
                            }),
                    )
                    .children(if self.is_pinned {
                        Some(
                            div()
                                .flex()
                                .items_center()
                                .justify_center()
                                .child(Icon::new(IconName::Star).size(px(12.)).color(theme.primary)),
                        )
                    } else {
                        None
                    }),
            );

        if !self.snippet.is_empty() {
            el = el.child(
                div()
                    .text_size(px(12.))
                    .text_color(theme.muted_foreground)
                    .line_clamp(2)
                    .child(self.snippet),
            );
        }

        let has_folder = self.folder_name.is_some();
        let has_time = !self.updated_at.is_empty();

        if has_folder || has_time {
            let mut footer = div()
                .flex()
                .items_center()
                .justify_between()
                .pt(px(2.))
                .text_size(px(11.))
                .text_color(theme.muted_foreground);

            if has_time {
                footer = footer.child(div().child(self.updated_at));
            } else {
                footer = footer.child(div());
            }

            if let Some(folder) = self.folder_name {
                footer = footer.child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(3.))
                        .child(Icon::new(IconName::Folder).size(px(10.)).color(theme.muted_foreground))
                        .child(div().line_clamp(1).child(folder)),
                );
            }

            el = el.child(footer);
        }

        if let Some(on_select) = self.on_select {
            el = el.on_click(move |ev, window, cx| on_select(ev, window, cx));
        }

        el
    }
}
