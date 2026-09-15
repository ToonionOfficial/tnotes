use gpui::*;
use crate::components::icon::{Icon, IconName};
use crate::theme::ThemeExt;

#[allow(dead_code)]
#[derive(IntoElement)]
pub struct FolderTreeItem {
    id: ElementId,
    name: SharedString,
    icon: Option<IconName>,
    depth: usize,
    is_expanded: bool,
    count: Option<usize>,
    on_toggle: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
    on_right_click: Option<Box<dyn Fn(&MouseDownEvent, &mut Window, &mut App) + 'static>>,
}

#[allow(dead_code)]
impl FolderTreeItem {
    pub fn new(id: impl Into<ElementId>, name: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            icon: None,
            depth: 0,
            is_expanded: false,
            count: None,
            on_toggle: None,
            on_right_click: None,
        }
    }

    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn depth(mut self, depth: usize) -> Self {
        self.depth = depth;
        self
    }

    pub fn expanded(mut self, expanded: bool) -> Self {
        self.is_expanded = expanded;
        self
    }

    pub fn count(mut self, count: usize) -> Self {
        self.count = Some(count);
        self
    }

    pub fn on_toggle(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_toggle = Some(Box::new(handler));
        self
    }

    pub fn on_right_click(
        mut self,
        handler: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_right_click = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for FolderTreeItem {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let indent = px(6.0 + (self.depth as f32 * 12.0));

        let item_icon = self.icon.unwrap_or(if self.is_expanded {
            IconName::FolderOpen
        } else {
            IconName::Folder
        });

        let chevron = if self.on_toggle.is_some() {
            let chevron_icon = if self.is_expanded {
                IconName::ChevronDown
            } else {
                IconName::ChevronRight
            };
            div()
                .flex_shrink_0()
                .w(px(16.))
                .h(px(16.))
                .flex()
                .items_center()
                .justify_center()
                .child(Icon::new(chevron_icon).size(px(12.)).color(theme.muted_foreground))
        } else {
            div().flex_shrink_0().w(px(16.)).h(px(16.))
        };

        let on_toggle = self.on_toggle;
        let on_right_click = self.on_right_click;

        let mut row = div()
            .id(self.id)
            .w_full()
            .h(px(28.))
            .pl(indent)
            .pr_2()
            .rounded(px(6.))
            .bg(gpui::transparent_black())
            .flex()
            .items_center()
            .justify_between()
            .cursor_pointer()
            .hover(|s| s.bg(theme.secondary).text_color(theme.foreground))
            .text_color(theme.muted_foreground)
            .text_size(px(13.))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_1p5()
                    .flex_1()
                    .min_w_0()
                    .overflow_hidden()
                    .child(chevron)
                    .child(
                        div()
                            .flex_shrink_0()
                            .flex()
                            .items_center()
                            .justify_center()
                            .child(
                                Icon::new(item_icon)
                                    .size(px(14.))
                                    .color(theme.muted_foreground),
                            ),
                    )
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .truncate()
                            .child(self.name),
                    ),
            );

        if let Some(count) = self.count {
            row = row.child(
                div()
                    .flex_shrink_0()
                    .ml_1()
                    .text_size(px(11.))
                    .text_color(theme.muted_foreground)
                    .child(count.to_string()),
            );
        }

        if let Some(on_toggle) = on_toggle {
            row = row.on_click(move |ev, window, cx| {
                if ev.is_right_click() {
                    return;
                }
                on_toggle(ev, window, cx);
            });
        }

        if let Some(on_right_click) = on_right_click {
            row = row.on_mouse_down(
                MouseButton::Right,
                move |ev, window, cx| on_right_click(ev, window, cx),
            );
        }

        row
    }
}

#[allow(dead_code)]
#[derive(IntoElement)]
pub struct NoteTreeItem {
    id: ElementId,
    title: SharedString,
    depth: usize,
    is_active: bool,
    is_pinned: bool,
    on_click: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
    on_right_click: Option<Box<dyn Fn(&MouseDownEvent, &mut Window, &mut App) + 'static>>,
}

#[allow(dead_code)]
impl NoteTreeItem {
    pub fn new(id: impl Into<SharedString>, title: impl Into<SharedString>) -> Self {
        let id_str: SharedString = id.into();
        Self {
            id: ElementId::from(id_str),
            title: title.into(),
            depth: 0,
            is_active: false,
            is_pinned: false,
            on_click: None,
            on_right_click: None,
        }
    }

    pub fn depth(mut self, depth: usize) -> Self {
        self.depth = depth;
        self
    }

    pub fn active(mut self, active: bool) -> Self {
        self.is_active = active;
        self
    }

    pub fn pinned(mut self, pinned: bool) -> Self {
        self.is_pinned = pinned;
        self
    }

    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }

    pub fn on_right_click(
        mut self,
        handler: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_right_click = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for NoteTreeItem {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let indent = px(12. + (self.depth as f32) * 16.);

        let (bg, text_color, icon_color) = if self.is_active {
            (theme.secondary, theme.foreground, theme.primary)
        } else {
            (gpui::transparent_black(), theme.foreground, theme.muted_foreground)
        };

        let mut row = div()
            .id(self.id)
            .w_full()
            .h(px(28.))
            .pl(indent)
            .pr(px(8.))
            .rounded(px(5.))
            .bg(bg)
            .flex()
            .items_center()
            .justify_between()
            .cursor_pointer()
            .hover(|s| s.bg(theme.secondary))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .flex_1()
                    .min_w_0()
                    .overflow_hidden()
                    .child(
                        div()
                            .flex_shrink_0()
                            .flex()
                            .items_center()
                            .justify_center()
                            .child(
                                Icon::new(IconName::FileText)
                                    .size(px(14.))
                                    .color(icon_color),
                            ),
                    )
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .text_size(px(13.))
                            .font_weight(if self.is_active {
                                FontWeight::MEDIUM
                            } else {
                                FontWeight::NORMAL
                            })
                            .text_color(text_color)
                            .truncate()
                            .child(self.title),
                    ),
            );

        if self.is_pinned {
            row = row.child(
                div()
                    .flex_shrink_0()
                    .ml_1()
                    .child(
                        Icon::new(IconName::Star)
                            .size(px(11.))
                            .color(theme.primary),
                    ),
            );
        }

        if let Some(on_click) = self.on_click {
            row = row.on_click(move |ev, window, cx| {
                // Right-clicks open the context menu instead of selecting.
                if ev.is_right_click() {
                    return;
                }
                on_click(ev, window, cx)
            });
        }

        if let Some(on_right_click) = self.on_right_click {
            row = row.on_mouse_down(
                MouseButton::Right,
                move |ev, window, cx| on_right_click(ev, window, cx),
            );
        }

        row
    }
}

#[cfg(test)]
mod tests {
    use super::{FolderTreeItem, NoteTreeItem};
    use crate::theme::{ActiveTheme, Theme};
    use gpui::{
        div, AnyElement, Context, IntoElement, ParentElement, Render, TestAppContext, Window,
    };

    struct TestWrapper(Box<dyn Fn() -> AnyElement>);

    impl Render for TestWrapper {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            (self.0)()
        }
    }

    #[test]
    fn folder_tree_item_and_note_tree_item_render_with_long_title() {
        let mut cx = TestAppContext::single();
        cx.update(|cx| {
            cx.set_global(ActiveTheme(Theme::dark()));
        });

        let (_, cx) = cx.add_window_view(|_, _| {
            TestWrapper(Box::new(|| {
                div()
                    .child(
                        FolderTreeItem::new(
                            "test-folder",
                            "Very long folder title that should truncate properly with ellipsis and not overflow",
                        )
                        .count(42)
                        .expanded(true),
                    )
                    .child(
                        NoteTreeItem::new(
                            "test-note",
                            "Very long note title that should truncate properly with ellipsis and not overflow",
                        )
                        .pinned(true)
                        .active(true),
                    )
                    .into_any_element()
            }))
        });
        cx.run_until_parked();
    }
}
