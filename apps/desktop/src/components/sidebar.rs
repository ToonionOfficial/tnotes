use gpui::*;
use crate::theme::ThemeExt;

#[allow(dead_code)]
#[derive(IntoElement)]
pub struct Sidebar {
    id: ElementId,
    width: Pixels,
    collapsed: bool,
    header: Option<AnyElement>,
    content: Option<AnyElement>,
    footer: Option<AnyElement>,
}

#[allow(dead_code)]
impl Sidebar {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            width: px(240.),
            collapsed: false,
            header: None,
            content: None,
            footer: None,
        }
    }

    pub fn width(mut self, width: impl Into<Pixels>) -> Self {
        self.width = width.into();
        self
    }

    pub fn collapsed(mut self, collapsed: bool) -> Self {
        self.collapsed = collapsed;
        self
    }

    pub fn header(mut self, header: impl IntoElement) -> Self {
        self.header = Some(header.into_any_element());
        self
    }

    pub fn content(mut self, content: impl IntoElement) -> Self {
        self.content = Some(content.into_any_element());
        self
    }

    pub fn footer(mut self, footer: impl IntoElement) -> Self {
        self.footer = Some(footer.into_any_element());
        self
    }
}

impl RenderOnce for Sidebar {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();

        if self.collapsed {
            return div().id(self.id).w(px(0.)).h_full().overflow_hidden();
        }

        div()
            .id(self.id)
            .w(self.width)
            .h_full()
            .bg(theme.background)
            .border_r_1()
            .border_color(theme.border)
            .flex()
            .flex_col()
            .children(self.header)
            .children(self.content)
            .children(self.footer)
    }
}

#[allow(dead_code)]
#[derive(IntoElement, Default)]
pub struct SidebarHeader {
    children: Vec<AnyElement>,
}

#[allow(dead_code)]
impl SidebarHeader {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn child(mut self, child: impl IntoElement) -> Self {
        self.children.push(child.into_any_element());
        self
    }

    pub fn children(mut self, children: impl IntoIterator<Item = impl IntoElement>) -> Self {
        self.children
            .extend(children.into_iter().map(|c| c.into_any_element()));
        self
    }
}

impl RenderOnce for SidebarHeader {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        div()
            .px_3()
            .py_2p5()
            .flex()
            .flex_col()
            .gap_2()
            .children(self.children)
    }
}

#[allow(dead_code)]
#[derive(IntoElement, Default)]
pub struct SidebarContent {
    children: Vec<AnyElement>,
}

#[allow(dead_code)]
impl SidebarContent {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn child(mut self, child: impl IntoElement) -> Self {
        self.children.push(child.into_any_element());
        self
    }

    pub fn children(mut self, children: impl IntoIterator<Item = impl IntoElement>) -> Self {
        self.children
            .extend(children.into_iter().map(|c| c.into_any_element()));
        self
    }
}

impl RenderOnce for SidebarContent {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        div()
            .id("sidebar-content-scroll")
            .flex_1()
            .overflow_y_scroll()
            .px_2()
            .py_1()
            .flex()
            .flex_col()
            .gap_3()
            .children(self.children)
    }
}

#[allow(dead_code)]
#[derive(IntoElement)]
pub struct SidebarGroup {
    label: Option<SharedString>,
    action: Option<AnyElement>,
    children: Vec<AnyElement>,
}

#[allow(dead_code)]
impl SidebarGroup {
    pub fn new() -> Self {
        Self {
            label: None,
            action: None,
            children: Vec::new(),
        }
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn action(mut self, action: impl IntoElement) -> Self {
        self.action = Some(action.into_any_element());
        self
    }

    pub fn child(mut self, child: impl IntoElement) -> Self {
        self.children.push(child.into_any_element());
        self
    }

    pub fn children(mut self, children: impl IntoIterator<Item = impl IntoElement>) -> Self {
        self.children
            .extend(children.into_iter().map(|c| c.into_any_element()));
        self
    }
}

impl RenderOnce for SidebarGroup {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();

        let header = self.label.map(|label| {
            div()
                .flex()
                .items_center()
                .justify_between()
                .px_2()
                .py_1()
                .text_size(px(10.5))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(theme.muted_foreground)
                .child(label)
                .children(self.action)
        });

        div()
            .flex()
            .flex_col()
            .gap_0p5()
            .children(header)
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_0p5()
                    .children(self.children),
            )
    }
}

#[allow(dead_code)]
#[derive(IntoElement, Default)]
pub struct SidebarFooter {
    children: Vec<AnyElement>,
}

#[allow(dead_code)]
impl SidebarFooter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn child(mut self, child: impl IntoElement) -> Self {
        self.children.push(child.into_any_element());
        self
    }

    pub fn children(mut self, children: impl IntoIterator<Item = impl IntoElement>) -> Self {
        self.children
            .extend(children.into_iter().map(|c| c.into_any_element()));
        self
    }
}

impl RenderOnce for SidebarFooter {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        div()
            .p_2()
            .border_t_1()
            .border_color(theme.border)
            .flex()
            .flex_col()
            .gap_1()
            .children(self.children)
    }
}
