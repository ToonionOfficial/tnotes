use std::rc::Rc;

use crate::components::icon::{Icon, IconName};
use crate::theme::ThemeExt;
use gpui::*;

pub type ContextMenuItemSelectHandler =
    Box<dyn Fn(&MouseDownEvent, &mut Window, &mut App) + 'static>;
pub type ContextMenuDismissHandler = Rc<dyn Fn(&mut Window, &mut App) + 'static>;

/// A single selectable row inside a [`ContextMenuContent`].
///
/// Mirrors `shadcn`'s `ContextMenuItem`: optional leading icon, optional
/// trailing shortcut hint, destructive + disabled variants.
#[allow(dead_code)]
#[derive(IntoElement)]
pub struct ContextMenuItem {
    id: ElementId,
    label: SharedString,
    icon: Option<IconName>,
    shortcut: Option<SharedString>,
    destructive: bool,
    disabled: bool,
    on_select: Option<ContextMenuItemSelectHandler>,
}

#[allow(dead_code)]
impl ContextMenuItem {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: None,
            shortcut: None,
            destructive: false,
            disabled: false,
            on_select: None,
        }
    }

    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn shortcut(mut self, shortcut: impl Into<SharedString>) -> Self {
        self.shortcut = Some(shortcut.into());
        self
    }

    pub fn destructive(mut self, destructive: bool) -> Self {
        self.destructive = destructive;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_select(
        mut self,
        handler: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_select = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for ContextMenuItem {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();

        let (text_color, icon_color) = if self.destructive {
            (theme.destructive, theme.destructive)
        } else {
            (theme.popover_foreground, theme.muted_foreground)
        };

        let mut row = div()
            .id(self.id)
            .h(px(28.))
            .px_2()
            .rounded(px(6.))
            .flex()
            .items_center()
            .gap_2()
            .text_size(px(12.5))
            .text_color(text_color);

        if self.disabled {
            row = row.opacity(0.5).cursor_not_allowed();
        } else {
            let (hover_bg, hover_text) = if self.destructive {
                (theme.destructive.opacity(0.12), theme.destructive)
            } else {
                (theme.secondary, theme.foreground)
            };
            row = row
                .cursor_pointer()
                .hover(move |s| s.bg(hover_bg).text_color(hover_text));
        }

        row = row.children(self.icon.map(|name| {
            Icon::new(name).size(px(14.)).color(if self.disabled {
                theme.muted_foreground
            } else {
                icon_color
            })
        }));

        row = row.child(div().flex_1().line_clamp(1).child(self.label));

        row = row.children(self.shortcut.map(|shortcut| {
            div()
                .ml_auto()
                .text_size(px(11.))
                .text_color(theme.muted_foreground)
                .child(shortcut)
        }));

        if let Some(on_select) = self.on_select.filter(|_| !self.disabled) {
            // Activate on mouse-down (not click) so the action runs before
            // the menu closes, and stop propagation so rows painted behind
            // the menu never see the press.
            row = row.on_mouse_down(MouseButton::Left, move |ev, window, cx| {
                cx.stop_propagation();
                on_select(ev, window, cx);
            });
        }

        row
    }
}

/// Thin divider between menu items. Mirrors `shadcn`'s `ContextMenuSeparator`.
#[allow(dead_code)]
#[derive(IntoElement, Default)]
pub struct ContextMenuSeparator;

impl RenderOnce for ContextMenuSeparator {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        div().h(px(1.)).mx_1().my_1().bg(theme.border)
    }
}

/// Small heading shown above a section of items.
/// Mirrors `shadcn`'s `ContextMenuLabel`.
#[allow(dead_code)]
#[derive(IntoElement)]
pub struct ContextMenuLabel {
    label: SharedString,
}

#[allow(dead_code)]
impl ContextMenuLabel {
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
        }
    }
}

impl RenderOnce for ContextMenuLabel {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        div()
            .px_2()
            .py_1()
            .text_size(px(11.))
            .font_weight(FontWeight::SEMIBOLD)
            .text_color(theme.muted_foreground)
            .line_clamp(1)
            .child(self.label)
    }
}

/// Popover panel that hosts items, separators and labels.
/// Mirrors `shadcn`'s `ContextMenuContent`. Compose with `.child(...)`.
#[allow(dead_code)]
#[derive(IntoElement, Default)]
pub struct ContextMenuContent {
    children: Vec<AnyElement>,
}

#[allow(dead_code)]
impl ContextMenuContent {
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

impl RenderOnce for ContextMenuContent {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        div()
            .min_w(px(200.))
            .max_w(px(280.))
            .bg(theme.popover)
            .border_1()
            .border_color(theme.border)
            .rounded(px(8.))
            .shadow_lg()
            .p_1()
            .flex()
            .flex_col()
            .gap_0p5()
            .children(self.children)
    }
}

/// Floating overlay that positions a [`ContextMenuContent`] at a window
/// position (e.g. from a right-click `MouseDownEvent::position`).
///
/// Mirrors `shadcn`'s `ContextMenu` root: renders deferred so it paints above
/// surrounding content, occludes underlying hitboxes to avoid hover leaks,
/// clamps within viewport bounds, and dismisses on outside click, right-click,
/// or `Escape`.
#[allow(dead_code)]
#[derive(IntoElement)]
pub struct ContextMenu {
    id: ElementId,
    position: Point<Pixels>,
    focus_handle: Option<FocusHandle>,
    on_dismiss: Option<ContextMenuDismissHandler>,
    content: Option<AnyElement>,
}

#[allow(dead_code)]
impl ContextMenu {
    pub fn new(id: impl Into<ElementId>, position: Point<Pixels>) -> Self {
        Self {
            id: id.into(),
            position,
            focus_handle: None,
            on_dismiss: None,
            content: None,
        }
    }

    pub fn focus_handle(mut self, handle: FocusHandle) -> Self {
        self.focus_handle = Some(handle);
        self
    }

    pub fn on_dismiss(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_dismiss = Some(Rc::new(handler));
        self
    }

    pub fn content(mut self, content: impl IntoElement) -> Self {
        self.content = Some(content.into_any_element());
        self
    }

    pub fn child(self, child: impl IntoElement) -> Self {
        self.content(child)
    }
}

impl RenderOnce for ContextMenu {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let viewport = window.viewport_size();
        let position = if viewport.width > px(0.) && viewport.height > px(0.) {
            let max_x = (viewport.width - px(288.)).max(px(0.));
            let max_y = (viewport.height - px(260.)).max(px(0.));
            point(
                self.position.x.min(max_x).max(px(0.)),
                self.position.y.min(max_y).max(px(0.)),
            )
        } else {
            self.position
        };

        // If no focus handle was provided, allocate one so keyboard dismiss (Escape) works.
        let focus_handle = self.focus_handle.unwrap_or_else(|| cx.focus_handle());

        // NOTE: the menu is returned directly with NO intermediate wrapper
        // div. An in-flow wrapper would lay out after preceding siblings and
        // shift the absolute offsets by its own origin.
        //
        // `.occlude()` sets HitboxBehavior::BlockMouse, ensuring elements
        // painted behind the menu are excluded from GPUI mouse hit-testing,
        // which prevents unwanted hover styling on background items.
        let mut menu = div()
            .id(self.id)
            .occlude()
            .track_focus(&focus_handle)
            .absolute()
            .left(position.x)
            .top(position.y)
            .children(self.content)
            // Swallow left presses so elements painted behind the menu
            // (e.g. sidebar rows under the popup) never toggle/select.
            .on_mouse_down(MouseButton::Left, |_, _, cx| {
                cx.stop_propagation();
            });

        if let Some(on_dismiss) = self.on_dismiss {
            // Right-clicking the context menu itself dismisses it.
            let on_right_click = Rc::clone(&on_dismiss);
            menu = menu.on_mouse_down(MouseButton::Right, move |_, window, cx| {
                cx.stop_propagation();
                on_right_click(window, cx);
            });

            // Clicking outside the menu dismisses it.
            let on_outside = Rc::clone(&on_dismiss);
            menu = menu.on_mouse_down_out(move |event: &MouseDownEvent, window, cx| {
                if event.button == MouseButton::Left || event.button == MouseButton::Middle {
                    on_outside(window, cx);
                }
            });

            menu = menu.on_key_down(move |event, window, cx| {
                if event.keystroke.key == "escape" {
                    on_dismiss(window, cx);
                }
            });
        }

        deferred(menu)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ContextMenu, ContextMenuContent, ContextMenuItem, ContextMenuLabel, ContextMenuSeparator,
    };
    use crate::theme::{ActiveTheme, Theme};
    use gpui::{
        Context, FocusHandle, InteractiveElement, IntoElement, Modifiers, MouseMoveEvent,
        ParentElement, Render, StatefulInteractiveElement, Styled, TestAppContext, Window, div,
        point, px,
    };
    use std::cell::Cell;
    use std::rc::Rc;

    struct TestOverlayView {
        underlying_hovered: Rc<Cell<bool>>,
        show_menu: bool,
    }

    impl Render for TestOverlayView {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            let hovered = Rc::clone(&self.underlying_hovered);
            let underlying =
                div()
                    .id("underlying-box")
                    .w(px(300.))
                    .h(px(300.))
                    .on_hover(move |is_h, _, _| {
                        hovered.set(*is_h);
                    });

            let mut root = div().size(px(500.)).child(underlying);

            if self.show_menu {
                root = root.child(
                    ContextMenu::new("test-menu", point(px(50.), px(50.))).content(
                        ContextMenuContent::new()
                            .child(ContextMenuItem::new("item-1", "Test Item")),
                    ),
                );
            }

            root
        }
    }

    #[test]
    fn context_menu_blocks_hover_for_underlying_elements() {
        let mut cx = TestAppContext::single();
        cx.update(|cx| {
            cx.set_global(ActiveTheme(Theme::dark()));
        });

        let hovered = Rc::new(Cell::new(false));
        let hovered_clone = Rc::clone(&hovered);

        let (view, cx) = cx.add_window_view(|_, _| TestOverlayView {
            underlying_hovered: hovered_clone,
            show_menu: false,
        });
        cx.run_until_parked();
        cx.update(|window, cx| {
            _ = window.draw(cx);
        });

        // 1. Hover over underlying box when menu is NOT open.
        cx.simulate_event(MouseMoveEvent {
            position: point(px(100.), px(100.)),
            modifiers: Modifiers::default(),
            pressed_button: None,
        });
        cx.run_until_parked();
        assert!(
            hovered.get(),
            "underlying element should be hovered when menu is closed"
        );

        // 2. Open context menu on top of the underlying box at (50, 50).
        view.update(cx, |this, cx| {
            this.show_menu = true;
            cx.notify();
        });
        cx.run_until_parked();
        cx.update(|window, cx| {
            _ = window.draw(cx);
        });

        // 3. Move mouse to (100, 65), which is inside the context menu item.
        cx.simulate_event(MouseMoveEvent {
            position: point(px(100.), px(65.)),
            modifiers: Modifiers::default(),
            pressed_button: None,
        });
        cx.run_until_parked();

        // When the context menu is open, the underlying item must NOT be hovered!
        assert!(
            !hovered.get(),
            "underlying element MUST NOT be hovered when the mouse is over the context menu!"
        );
    }

    struct TestMenuInteractionView {
        menu_open: bool,
        selected_item: Rc<Cell<Option<&'static str>>>,
        dismissed: Rc<Cell<bool>>,
        focus_handle: FocusHandle,
    }

    impl Render for TestMenuInteractionView {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            let mut root = div().id("test-root").size(px(600.));

            if self.menu_open {
                let sel1 = Rc::clone(&self.selected_item);
                let sel2 = Rc::clone(&self.selected_item);
                let dis = Rc::clone(&self.dismissed);

                let content = ContextMenuContent::new()
                    .child(ContextMenuLabel::new("Header Label"))
                    .child(
                        ContextMenuItem::new("item-normal", "Normal Item").on_select(
                            move |_, _, _| {
                                sel1.set(Some("normal"));
                            },
                        ),
                    )
                    .child(ContextMenuSeparator)
                    .child(
                        ContextMenuItem::new("item-disabled", "Disabled Item")
                            .disabled(true)
                            .on_select(move |_, _, _| {
                                sel2.set(Some("disabled"));
                            }),
                    );

                root = root.child(
                    ContextMenu::new("test-ctx-menu", point(px(100.), px(100.)))
                        .focus_handle(self.focus_handle.clone())
                        .on_dismiss(move |_, _| {
                            dis.set(true);
                        })
                        .content(content),
                );
            }

            root
        }
    }

    #[test]
    fn context_menu_item_selection_triggers_on_select() {
        let mut cx = TestAppContext::single();
        cx.update(|cx| {
            cx.set_global(ActiveTheme(Theme::dark()));
        });

        let selected = Rc::new(Cell::new(None));
        let dismissed = Rc::new(Cell::new(false));

        let (_view, cx) = cx.add_window_view(|window, cx| {
            let focus_handle = cx.focus_handle();
            window.focus(&focus_handle);
            TestMenuInteractionView {
                menu_open: true,
                selected_item: Rc::clone(&selected),
                dismissed: Rc::clone(&dismissed),
                focus_handle,
            }
        });
        cx.run_until_parked();
        cx.update(|window, cx| {
            _ = window.draw(cx);
        });

        // Click first item (Normal Item) around (150, 138)
        cx.simulate_event(gpui::MouseDownEvent {
            button: gpui::MouseButton::Left,
            position: point(px(150.), px(138.)),
            modifiers: Modifiers::default(),
            click_count: 1,
            first_mouse: false,
        });
        cx.run_until_parked();

        assert_eq!(
            selected.get(),
            Some("normal"),
            "clicking enabled item should trigger on_select"
        );
    }

    #[test]
    fn context_menu_disabled_item_does_not_trigger_on_select() {
        let mut cx = TestAppContext::single();
        cx.update(|cx| {
            cx.set_global(ActiveTheme(Theme::dark()));
        });

        let selected = Rc::new(Cell::new(None));
        let dismissed = Rc::new(Cell::new(false));

        let (_view, cx) = cx.add_window_view(|window, cx| {
            let focus_handle = cx.focus_handle();
            window.focus(&focus_handle);
            TestMenuInteractionView {
                menu_open: true,
                selected_item: Rc::clone(&selected),
                dismissed: Rc::clone(&dismissed),
                focus_handle,
            }
        });
        cx.run_until_parked();
        cx.update(|window, cx| {
            _ = window.draw(cx);
        });

        // Click disabled item around (150, 175)
        cx.simulate_event(gpui::MouseDownEvent {
            button: gpui::MouseButton::Left,
            position: point(px(150.), px(175.)),
            modifiers: Modifiers::default(),
            click_count: 1,
            first_mouse: false,
        });
        cx.run_until_parked();

        assert_eq!(
            selected.get(),
            None,
            "clicking disabled item should NOT trigger on_select"
        );
    }

    #[test]
    fn context_menu_dismiss_on_outside_click() {
        let mut cx = TestAppContext::single();
        cx.update(|cx| {
            cx.set_global(ActiveTheme(Theme::dark()));
        });

        let selected = Rc::new(Cell::new(None));
        let dismissed = Rc::new(Cell::new(false));

        let (_view, cx) = cx.add_window_view(|window, cx| {
            let focus_handle = cx.focus_handle();
            window.focus(&focus_handle);
            TestMenuInteractionView {
                menu_open: true,
                selected_item: Rc::clone(&selected),
                dismissed: Rc::clone(&dismissed),
                focus_handle,
            }
        });
        cx.run_until_parked();
        cx.update(|window, cx| {
            _ = window.draw(cx);
        });

        // Click outside the menu at (450, 450)
        cx.simulate_event(gpui::MouseDownEvent {
            button: gpui::MouseButton::Left,
            position: point(px(450.), px(450.)),
            modifiers: Modifiers::default(),
            click_count: 1,
            first_mouse: false,
        });
        cx.run_until_parked();

        assert!(
            dismissed.get(),
            "clicking outside menu should trigger on_dismiss"
        );
    }

    #[test]
    fn context_menu_dismiss_on_right_click_menu() {
        let mut cx = TestAppContext::single();
        cx.update(|cx| {
            cx.set_global(ActiveTheme(Theme::dark()));
        });

        let selected = Rc::new(Cell::new(None));
        let dismissed = Rc::new(Cell::new(false));

        let (_view, cx) = cx.add_window_view(|window, cx| {
            let focus_handle = cx.focus_handle();
            window.focus(&focus_handle);
            TestMenuInteractionView {
                menu_open: true,
                selected_item: Rc::clone(&selected),
                dismissed: Rc::clone(&dismissed),
                focus_handle,
            }
        });
        cx.run_until_parked();
        cx.update(|window, cx| {
            _ = window.draw(cx);
        });

        // Right-click inside the menu at (150, 115)
        cx.simulate_event(gpui::MouseDownEvent {
            button: gpui::MouseButton::Right,
            position: point(px(150.), px(115.)),
            modifiers: Modifiers::default(),
            click_count: 1,
            first_mouse: false,
        });
        cx.run_until_parked();

        assert!(
            dismissed.get(),
            "right-clicking the context menu should dismiss it"
        );
    }

    #[test]
    fn context_menu_dismiss_on_escape() {
        let mut cx = TestAppContext::single();
        cx.update(|cx| {
            cx.set_global(ActiveTheme(Theme::dark()));
        });

        let selected = Rc::new(Cell::new(None));
        let dismissed = Rc::new(Cell::new(false));

        let (_view, cx) = cx.add_window_view(|window, cx| {
            let focus_handle = cx.focus_handle();
            window.focus(&focus_handle);
            TestMenuInteractionView {
                menu_open: true,
                selected_item: Rc::clone(&selected),
                dismissed: Rc::clone(&dismissed),
                focus_handle,
            }
        });
        cx.run_until_parked();
        cx.update(|window, cx| {
            _ = window.draw(cx);
        });

        cx.simulate_keystrokes("escape");
        cx.run_until_parked();

        assert!(dismissed.get(), "pressing escape should trigger on_dismiss");
    }
}
