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
    is_selected: bool,
    count: Option<usize>,
    on_toggle: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
    on_select: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
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
            is_selected: false,
            count: None,
            on_toggle: None,
            on_select: None,
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

    pub fn selected(mut self, selected: bool) -> Self {
        self.is_selected = selected;
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

    pub fn on_select(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_select = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for FolderTreeItem {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let indent = px(6.0 + (self.depth as f32 * 12.0));

        let bg = if self.is_selected {
            theme.secondary
        } else {
            gpui::transparent_black()
        };

        let text_color = if self.is_selected {
            theme.foreground
        } else {
            theme.muted_foreground
        };

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
                .w(px(16.))
                .h(px(16.))
                .flex()
                .items_center()
                .justify_center()
                .child(Icon::new(chevron_icon).size(px(12.)).color(theme.muted_foreground))
        } else {
            div().w(px(16.)).h(px(16.))
        };

        let on_toggle = self.on_toggle;
        let on_select = self.on_select;

        let mut row = div()
            .id(self.id)
            .h(px(28.))
            .pl(indent)
            .pr_2()
            .rounded(px(6.))
            .bg(bg)
            .flex()
            .items_center()
            .justify_between()
            .cursor_pointer()
            .hover(|s| s.bg(theme.secondary).text_color(theme.foreground))
            .text_color(text_color)
            .text_size(px(13.))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_1p5()
                    .child(chevron)
                    .child(
                        Icon::new(item_icon)
                            .size(px(14.))
                            .color(if self.is_selected { theme.primary } else { theme.muted_foreground }),
                    )
                    .child(div().line_clamp(1).child(self.name)),
            );

        if let Some(count) = self.count {
            row = row.child(
                div()
                    .text_size(px(11.))
                    .text_color(theme.muted_foreground)
                    .child(count.to_string()),
            );
        }

        if on_toggle.is_some() || on_select.is_some() {
            row = row.on_click(move |ev, window, cx| {
                if let Some(ref toggle) = on_toggle {
                    toggle(ev, window, cx);
                }
                if let Some(ref select) = on_select {
                    select(ev, window, cx);
                }
            });
        }

        row
    }
}
