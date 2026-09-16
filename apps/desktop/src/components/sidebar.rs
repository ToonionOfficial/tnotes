use crate::components::icon::{Icon, IconName};
use crate::theme::ThemeExt;
use gpui::prelude::FluentBuilder;
use gpui::*;

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SidebarCollapsible {
    #[default]
    Icon,
    Offcanvas,
    None,
}

impl From<bool> for SidebarCollapsible {
    fn from(collapsible: bool) -> Self {
        if collapsible { Self::Icon } else { Self::None }
    }
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SidebarSide {
    #[default]
    Left,
    Right,
}

#[allow(dead_code)]
#[derive(IntoElement)]
pub struct Sidebar {
    id: ElementId,
    width: Pixels,
    collapsed_width: Pixels,
    collapsed: bool,
    collapsible: SidebarCollapsible,
    side: SidebarSide,
    header: Option<AnyElement>,
    content: Option<AnyElement>,
    footer: Option<AnyElement>,
    rail: Option<AnyElement>,
}

#[allow(dead_code)]
impl Sidebar {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            width: px(260.),
            collapsed_width: px(48.),
            collapsed: false,
            collapsible: SidebarCollapsible::Icon,
            side: SidebarSide::Left,
            header: None,
            content: None,
            footer: None,
            rail: None,
        }
    }

    pub fn width(mut self, width: impl Into<Pixels>) -> Self {
        self.width = width.into();
        self
    }

    pub fn collapsed_width(mut self, width: impl Into<Pixels>) -> Self {
        self.collapsed_width = width.into();
        self
    }

    pub fn collapsed(mut self, collapsed: bool) -> Self {
        self.collapsed = collapsed;
        self
    }

    pub fn collapsible(mut self, collapsible: impl Into<SidebarCollapsible>) -> Self {
        self.collapsible = collapsible.into();
        self
    }

    pub fn side(mut self, side: SidebarSide) -> Self {
        self.side = side;
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

    pub fn rail(mut self, rail: impl IntoElement) -> Self {
        self.rail = Some(rail.into_any_element());
        self
    }
}

impl RenderOnce for Sidebar {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let is_collapsed = self.collapsed && self.collapsible != SidebarCollapsible::None;

        if is_collapsed {
            match self.collapsible {
                SidebarCollapsible::Offcanvas => {
                    return div()
                        .id(self.id)
                        .w(px(0.))
                        .h_full()
                        .overflow_hidden()
                        .into_any_element();
                }
                SidebarCollapsible::Icon => {
                    if let Some(rail) = self.rail {
                        return rail;
                    }
                    return div()
                        .id(self.id)
                        .w(self.collapsed_width)
                        .h_full()
                        .bg(theme.background)
                        .border_r_1()
                        .border_color(theme.border)
                        .flex()
                        .flex_col()
                        .items_center()
                        .justify_between()
                        .py_2p5()
                        .children(self.header)
                        .children(self.content)
                        .children(self.footer)
                        .into_any_element();
                }
                SidebarCollapsible::None => {}
            }
        }

        let mut sidebar = div()
            .id(self.id)
            .w(self.width)
            .h_full()
            .bg(theme.background)
            .flex()
            .flex_col();

        sidebar = match self.side {
            SidebarSide::Left => sidebar.border_r_1().border_color(theme.border),
            SidebarSide::Right => sidebar.border_l_1().border_color(theme.border),
        };

        sidebar
            .children(self.header)
            .children(self.content)
            .children(self.footer)
            .into_any_element()
    }
}

#[allow(dead_code)]
#[allow(clippy::type_complexity)]
#[derive(IntoElement)]
pub struct SidebarToggleButton {
    id: ElementId,
    collapsed: bool,
    side: SidebarSide,
    on_click: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
}

#[allow(dead_code)]
impl SidebarToggleButton {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            collapsed: false,
            side: SidebarSide::Left,
            on_click: None,
        }
    }

    pub fn collapsed(mut self, collapsed: bool) -> Self {
        self.collapsed = collapsed;
        self
    }

    pub fn side(mut self, side: SidebarSide) -> Self {
        self.side = side;
        self
    }

    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for SidebarToggleButton {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let mut btn = div()
            .id(self.id)
            .w(px(24.))
            .h(px(24.))
            .rounded(px(4.))
            .flex()
            .items_center()
            .justify_center()
            .cursor_pointer()
            .hover(|s| s.bg(theme.secondary).text_color(theme.foreground))
            .text_color(theme.muted_foreground)
            .child(Icon::new(IconName::PanelLeft).size(px(14.)));

        if let Some(handler) = self.on_click {
            btn = btn.on_click(move |ev, window, cx| handler(ev, window, cx));
        }

        btn
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
            .id("sidebar-content")
            .flex_1()
            .min_h_0()
            .overflow_hidden()
            .px_2()
            .py_1()
            .flex()
            .flex_col()
            .gap_2()
            .children(self.children)
    }
}

#[allow(dead_code)]
#[derive(IntoElement, Default)]
pub struct SidebarGroup {
    label: Option<SharedString>,
    action: Option<AnyElement>,
    collapsed: bool,
    collapsible: bool,
    fill: bool,
    children: Vec<AnyElement>,
}

#[allow(dead_code)]
impl SidebarGroup {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn fill(mut self, fill: bool) -> Self {
        self.fill = fill;
        self
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn action(mut self, action: impl IntoElement) -> Self {
        self.action = Some(action.into_any_element());
        self
    }

    pub fn collapsed(mut self, collapsed: bool) -> Self {
        self.collapsed = collapsed;
        self
    }

    pub fn collapsible(mut self, collapsible: bool) -> Self {
        self.collapsible = collapsible;
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
                .h(px(24.))
                .flex_none()
                .flex()
                .items_center()
                .justify_between()
                .pl(px(6.))
                .pr(px(2.))
                .text_size(px(11.))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(theme.muted_foreground)
                .child(label)
                .children(self.action)
        });

        let mut group = div().flex().flex_col().gap_0p5();

        if self.fill {
            group = group.flex_1().min_h_0().overflow_hidden();
        } else {
            group = group.flex_none();
        }

        group.children(header).when(!self.collapsed, |this| {
            if self.fill {
                this.child(
                    div()
                        .flex_1()
                        .min_h_0()
                        .overflow_hidden()
                        .flex()
                        .flex_col()
                        .children(self.children),
                )
            } else {
                this.child(div().flex().flex_col().gap_0p5().children(self.children))
            }
        })
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

#[allow(dead_code)]
#[derive(IntoElement)]
pub struct SidebarRail {
    id: ElementId,
    top: Vec<AnyElement>,
    bottom: Vec<AnyElement>,
}

#[allow(dead_code)]
impl SidebarRail {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            top: Vec::new(),
            bottom: Vec::new(),
        }
    }

    pub fn top_item(mut self, item: impl IntoElement) -> Self {
        self.top.push(item.into_any_element());
        self
    }

    pub fn bottom_item(mut self, item: impl IntoElement) -> Self {
        self.bottom.push(item.into_any_element());
        self
    }

    pub fn separator(mut self) -> Self {
        self.top.push(
            div()
                .w(px(24.))
                .h(px(1.))
                .bg(rgb(0x302e36))
                .into_any_element(),
        );
        self
    }
}

impl RenderOnce for SidebarRail {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        div()
            .id(self.id)
            .w(px(48.))
            .h_full()
            .bg(theme.background)
            .border_r_1()
            .border_color(theme.border)
            .flex()
            .flex_col()
            .items_center()
            .justify_between()
            .py_2p5()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap_3()
                    .children(self.top),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap_2()
                    .children(self.bottom),
            )
    }
}

#[allow(dead_code)]
#[allow(clippy::type_complexity)]
#[derive(IntoElement)]
pub struct SidebarRailItem {
    id: ElementId,
    icon: IconName,
    active: bool,
    on_click: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
}

#[allow(dead_code)]
impl SidebarRailItem {
    pub fn new(id: impl Into<ElementId>, icon: IconName) -> Self {
        Self {
            id: id.into(),
            icon,
            active: false,
            on_click: None,
        }
    }

    pub fn active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }

    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for SidebarRailItem {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let bg = if self.active {
            theme.secondary
        } else {
            gpui::transparent_black()
        };
        let text_color = if self.active {
            theme.foreground
        } else {
            theme.muted_foreground
        };

        let mut item = div()
            .id(self.id)
            .w(px(28.))
            .h(px(28.))
            .rounded(px(5.))
            .bg(bg)
            .flex()
            .items_center()
            .justify_center()
            .cursor_pointer()
            .hover(|s| s.bg(theme.secondary).text_color(theme.foreground))
            .text_color(text_color)
            .child(Icon::new(self.icon).size(px(16.)));

        if let Some(handler) = self.on_click {
            item = item.on_click(move |ev, window, cx| handler(ev, window, cx));
        }

        item
    }
}

#[allow(dead_code)]
#[derive(IntoElement, Default)]
pub struct SidebarMenu {
    children: Vec<AnyElement>,
}

#[allow(dead_code)]
impl SidebarMenu {
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

impl RenderOnce for SidebarMenu {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        div().flex().flex_col().gap_0p5().children(self.children)
    }
}

#[allow(dead_code)]
#[allow(clippy::type_complexity)]
#[derive(IntoElement)]
pub struct SidebarMenuItem {
    id: ElementId,
    label: SharedString,
    icon: Option<IconName>,
    active: bool,
    count: Option<usize>,
    badge: Option<SharedString>,
    action: Option<AnyElement>,
    depth: usize,
    is_expanded: bool,
    children: Vec<SidebarMenuItem>,
    on_click: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
    on_toggle: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
    on_right_click: Option<Box<dyn Fn(&MouseDownEvent, &mut Window, &mut App) + 'static>>,
}

#[allow(dead_code)]
impl SidebarMenuItem {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: None,
            active: false,
            count: None,
            badge: None,
            action: None,
            depth: 0,
            is_expanded: false,
            children: Vec::new(),
            on_click: None,
            on_toggle: None,
            on_right_click: None,
        }
    }

    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }

    pub fn count(mut self, count: usize) -> Self {
        self.count = Some(count);
        self
    }

    pub fn badge(mut self, badge: impl Into<SharedString>) -> Self {
        self.badge = Some(badge.into());
        self
    }

    pub fn action(mut self, action: impl IntoElement) -> Self {
        self.action = Some(action.into_any_element());
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

    pub fn child(mut self, child: SidebarMenuItem) -> Self {
        self.children.push(child);
        self
    }

    pub fn children(mut self, children: impl IntoIterator<Item = SidebarMenuItem>) -> Self {
        self.children.extend(children);
        self
    }

    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Box::new(handler));
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

impl RenderOnce for SidebarMenuItem {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let indent = px(6.0 + (self.depth as f32 * 12.0));

        let bg = if self.active {
            theme.secondary
        } else {
            gpui::transparent_black()
        };

        let text_color = if self.active {
            theme.foreground
        } else {
            theme.muted_foreground
        };

        let has_children = !self.children.is_empty() || self.on_toggle.is_some();
        let chevron = if has_children {
            let chevron_icon = if self.is_expanded {
                IconName::ChevronDown
            } else {
                IconName::ChevronRight
            };
            let on_toggle = self.on_toggle;
            let mut c = div()
                .id(ElementId::NamedInteger(
                    format!("{}-chevron", self.id).into(),
                    0,
                ))
                .w(px(16.))
                .h(px(16.))
                .flex()
                .items_center()
                .justify_center()
                .child(
                    Icon::new(chevron_icon)
                        .size(px(12.))
                        .color(theme.muted_foreground),
                );

            if let Some(on_toggle) = on_toggle {
                c = c.on_click(move |ev, window, cx| {
                    cx.stop_propagation();
                    on_toggle(ev, window, cx);
                });
            }
            c
        } else {
            div()
                .id(ElementId::NamedInteger(
                    format!("{}-chevron-spacer", self.id).into(),
                    0,
                ))
                .w(px(16.))
                .h(px(16.))
        };

        let mut row = div()
            .id(self.id)
            .w_full()
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
                    .flex_1()
                    .min_w_0()
                    .overflow_hidden()
                    .child(chevron)
                    .when_some(self.icon, |this, icon| {
                        this.child(
                            div()
                                .flex_shrink_0()
                                .flex()
                                .items_center()
                                .justify_center()
                                .child(Icon::new(icon).size(px(14.)).color(if self.active {
                                    theme.primary
                                } else {
                                    theme.muted_foreground
                                })),
                        )
                    })
                    .child(div().flex_1().min_w_0().truncate().child(self.label)),
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
        } else if let Some(badge) = self.badge {
            row = row.child(
                div()
                    .flex_shrink_0()
                    .ml_1()
                    .text_size(px(11.))
                    .text_color(theme.muted_foreground)
                    .child(badge),
            );
        } else if let Some(action) = self.action {
            row = row.child(div().flex_shrink_0().ml_1().child(action));
        }

        if let Some(on_click) = self.on_click {
            row = row.on_click(move |ev, window, cx| {
                if ev.is_right_click() {
                    return;
                }
                on_click(ev, window, cx);
            });
        }

        if let Some(on_right_click) = self.on_right_click {
            row = row.on_mouse_down(MouseButton::Right, move |ev, window, cx| {
                on_right_click(ev, window, cx)
            });
        }

        let is_expanded = self.is_expanded;
        let children = self.children;

        div()
            .w_full()
            .child(row)
            .when(is_expanded && !children.is_empty(), |this| {
                this.child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_0p5()
                        .border_l_1()
                        .border_color(theme.border)
                        .ml(indent + px(8.))
                        .children(children),
                )
            })
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Sidebar, SidebarCollapsible, SidebarContent, SidebarFooter, SidebarHeader, SidebarMenu,
        SidebarMenuItem, SidebarRail, SidebarRailItem,
    };
    use crate::components::IconName;
    use crate::theme::{ActiveTheme, Theme};
    use gpui::{
        AnyElement, Context, IntoElement, ParentElement, Render, TestAppContext, Window, div, px,
    };
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};

    struct TestWrapper(Box<dyn Fn() -> AnyElement>);

    impl Render for TestWrapper {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            (self.0)()
        }
    }

    #[test]
    fn sidebar_expanded_renders_full_width() {
        let mut cx = TestAppContext::single();
        cx.update(|cx| {
            cx.set_global(ActiveTheme(Theme::dark()));
        });

        let (_, cx) = cx.add_window_view(|_, _| {
            TestWrapper(Box::new(|| {
                Sidebar::new("test-sidebar")
                    .width(px(260.))
                    .header(SidebarHeader::new().child(div().child("Header")))
                    .content(SidebarContent::new().child(div().child("Content")))
                    .footer(SidebarFooter::new().child(div().child("Footer")))
                    .into_any_element()
            }))
        });
        cx.run_until_parked();
    }

    #[test]
    fn sidebar_collapsed_offcanvas_renders_zero_width() {
        let mut cx = TestAppContext::single();
        cx.update(|cx| {
            cx.set_global(ActiveTheme(Theme::dark()));
        });

        let (_, cx) = cx.add_window_view(|_, _| {
            TestWrapper(Box::new(|| {
                Sidebar::new("test-sidebar-offcanvas")
                    .collapsible(SidebarCollapsible::Offcanvas)
                    .collapsed(true)
                    .content(SidebarContent::new().child(div().child("Hidden")))
                    .into_any_element()
            }))
        });
        cx.run_until_parked();
    }

    #[test]
    fn sidebar_rail_renders_items() {
        let mut cx = TestAppContext::single();
        cx.update(|cx| {
            cx.set_global(ActiveTheme(Theme::dark()));
        });

        let clicked = Arc::new(AtomicBool::new(false));
        let clicked_clone = clicked.clone();

        let (_, cx) = cx.add_window_view(|_, _| {
            TestWrapper(Box::new(move || {
                let clicked_inner = clicked_clone.clone();
                SidebarRail::new("test-rail")
                    .top_item(
                        SidebarRailItem::new("rail-item", IconName::PanelLeft).on_click(
                            move |_, _, _| {
                                clicked_inner.store(true, Ordering::SeqCst);
                            },
                        ),
                    )
                    .separator()
                    .bottom_item(SidebarRailItem::new("rail-settings", IconName::Settings))
                    .into_any_element()
            }))
        });
        cx.run_until_parked();
    }

    #[test]
    fn sidebar_menu_item_renders() {
        let mut cx = TestAppContext::single();
        cx.update(|cx| {
            cx.set_global(ActiveTheme(Theme::dark()));
        });

        let (_, cx) = cx.add_window_view(|_, _| {
            TestWrapper(Box::new(|| {
                SidebarMenu::new()
                    .child(
                        SidebarMenuItem::new("menu-item-1", "Dashboard")
                            .icon(IconName::Home)
                            .active(true)
                            .count(3),
                    )
                    .child(
                        SidebarMenuItem::new("menu-item-2", "Projects")
                            .icon(IconName::Folder)
                            .expanded(true)
                            .child(SidebarMenuItem::new("menu-sub-1", "Subproject")),
                    )
                    .into_any_element()
            }))
        });
        cx.run_until_parked();
    }
}
