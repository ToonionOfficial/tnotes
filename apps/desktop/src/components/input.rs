use crate::components::badge::KbdBadge;
use crate::components::icon::{Icon, IconName};
use crate::theme::ThemeExt;
use gpui::prelude::FluentBuilder;
use gpui::*;
use std::ops::Range;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InputState {
    value: String,
    cursor: usize,
    selection: Option<Range<usize>>,
    undo_stack: Vec<(String, usize)>,
    redo_stack: Vec<(String, usize)>,
}

impl Default for InputState {
    fn default() -> Self {
        Self::new("")
    }
}

#[allow(dead_code)]
impl InputState {
    pub fn new(initial: impl Into<String>) -> Self {
        let value = initial.into();
        let cursor = value.len();
        Self {
            value,
            cursor,
            selection: None,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }

    pub fn selection(&self) -> Option<Range<usize>> {
        self.selection.clone()
    }

    pub fn is_empty(&self) -> bool {
        self.value.is_empty()
    }

    pub fn len(&self) -> usize {
        self.value.len()
    }

    pub fn set_value(&mut self, new_val: impl Into<String>) {
        self.push_undo();
        self.value = new_val.into();
        self.cursor = self.clamp_cursor(self.value.len());
        self.selection = None;
    }

    pub fn set_cursor(&mut self, cursor: usize) {
        self.cursor = self.clamp_cursor(cursor);
        self.selection = None;
    }

    fn clamp_cursor(&self, offset: usize) -> usize {
        let offset = offset.min(self.value.len());
        if self.value.is_char_boundary(offset) {
            offset
        } else {
            self.value
                .char_indices()
                .map(|(i, _)| i)
                .chain(std::iter::once(self.value.len()))
                .rfind(|&i| i <= offset)
                .unwrap_or(0)
        }
    }

    fn push_undo(&mut self) {
        self.undo_stack.push((self.value.clone(), self.cursor));
        if self.undo_stack.len() > 100 {
            self.undo_stack.remove(0);
        }
        self.redo_stack.clear();
    }

    pub fn undo(&mut self) -> bool {
        if let Some((prev_val, prev_cursor)) = self.undo_stack.pop() {
            self.redo_stack.push((self.value.clone(), self.cursor));
            self.value = prev_val;
            self.cursor = self.clamp_cursor(prev_cursor);
            self.selection = None;
            true
        } else {
            false
        }
    }

    pub fn redo(&mut self) -> bool {
        if let Some((next_val, next_cursor)) = self.redo_stack.pop() {
            self.undo_stack.push((self.value.clone(), self.cursor));
            self.value = next_val;
            self.cursor = self.clamp_cursor(next_cursor);
            self.selection = None;
            true
        } else {
            false
        }
    }

    pub fn clear(&mut self) {
        if self.value.is_empty() && self.selection.is_none() {
            return;
        }
        self.push_undo();
        self.value.clear();
        self.cursor = 0;
        self.selection = None;
    }

    pub fn previous_char_boundary(&self, offset: usize) -> usize {
        let offset = self.clamp_cursor(offset);
        if offset == 0 {
            return 0;
        }
        self.value[..offset]
            .char_indices()
            .last()
            .map(|(idx, _)| idx)
            .unwrap_or(0)
    }

    pub fn next_char_boundary(&self, offset: usize) -> usize {
        let offset = self.clamp_cursor(offset);
        if offset >= self.value.len() {
            return self.value.len();
        }
        self.value[offset..]
            .char_indices()
            .nth(1)
            .map(|(idx, _)| offset + idx)
            .unwrap_or(self.value.len())
    }

    pub fn previous_word_boundary(&self, offset: usize) -> usize {
        let offset = self.clamp_cursor(offset);
        if offset == 0 {
            return 0;
        }

        let slice = &self.value[..offset];
        let mut chars = slice.char_indices().rev().peekable();

        while let Some(&(_, c)) = chars.peek() {
            if c.is_whitespace() {
                chars.next();
            } else {
                break;
            }
        }

        if let Some(&(_, first_c)) = chars.peek() {
            let is_word = first_c.is_alphanumeric() || first_c == '_';
            while let Some(&(_, c)) = chars.peek() {
                if c.is_whitespace() {
                    break;
                }
                let curr_is_word = c.is_alphanumeric() || c == '_';
                if curr_is_word == is_word {
                    chars.next();
                } else {
                    break;
                }
            }
        }

        chars.next().map(|(idx, c)| idx + c.len_utf8()).unwrap_or(0)
    }

    pub fn next_word_boundary(&self, offset: usize) -> usize {
        let offset = self.clamp_cursor(offset);
        if offset >= self.value.len() {
            return self.value.len();
        }

        let slice = &self.value[offset..];
        let mut chars = slice.char_indices().peekable();

        while let Some(&(_, c)) = chars.peek() {
            if c.is_whitespace() {
                chars.next();
            } else {
                break;
            }
        }

        if let Some(&(_, first_c)) = chars.peek() {
            let is_word = first_c.is_alphanumeric() || first_c == '_';
            while let Some(&(_, c)) = chars.peek() {
                if c.is_whitespace() {
                    break;
                }
                let curr_is_word = c.is_alphanumeric() || c == '_';
                if curr_is_word == is_word {
                    chars.next();
                } else {
                    break;
                }
            }
        }

        chars
            .peek()
            .map(|&(idx, _)| offset + idx)
            .unwrap_or(self.value.len())
    }

    pub fn select_all(&mut self) {
        if self.value.is_empty() {
            self.selection = None;
            self.cursor = 0;
            return;
        }
        self.selection = Some(0..self.value.len());
        self.cursor = self.value.len();
    }

    fn delete_selection(&mut self) -> bool {
        if let Some(r) = self.selection.take()
            && !r.is_empty()
            && r.start < self.value.len()
        {
            self.push_undo();
            let start = self.clamp_cursor(r.start);
            let end = self.clamp_cursor(r.end);
            self.value.drain(start..end);
            self.cursor = start;
            return true;
        }
        false
    }

    pub fn insert(&mut self, text: &str) {
        self.push_undo();
        if let Some(r) = self.selection.take()
            && !r.is_empty()
            && r.start < self.value.len()
        {
            let start = self.clamp_cursor(r.start);
            let end = self.clamp_cursor(r.end);
            self.value.drain(start..end);
            self.cursor = start;
        }
        let cursor = self.clamp_cursor(self.cursor);
        self.value.insert_str(cursor, text);
        self.cursor = cursor + text.len();
        self.selection = None;
    }

    pub fn backspace(&mut self) {
        if self.delete_selection() {
            return;
        }
        if self.cursor > 0 {
            self.push_undo();
            let prev = self.previous_char_boundary(self.cursor);
            self.value.drain(prev..self.cursor);
            self.cursor = prev;
        }
    }

    pub fn delete(&mut self) {
        if self.delete_selection() {
            return;
        }
        if self.cursor < self.value.len() {
            self.push_undo();
            let next = self.next_char_boundary(self.cursor);
            self.value.drain(self.cursor..next);
        }
    }

    pub fn delete_to_previous_word(&mut self) {
        if self.delete_selection() {
            return;
        }
        if self.cursor > 0 {
            self.push_undo();
            let prev = self.previous_word_boundary(self.cursor);
            self.value.drain(prev..self.cursor);
            self.cursor = prev;
        }
    }

    pub fn delete_to_next_word(&mut self) {
        if self.delete_selection() {
            return;
        }
        if self.cursor < self.value.len() {
            self.push_undo();
            let next = self.next_word_boundary(self.cursor);
            self.value.drain(self.cursor..next);
        }
    }

    pub fn delete_to_start_of_line(&mut self) {
        if self.delete_selection() {
            return;
        }
        if self.cursor > 0 {
            self.push_undo();
            self.value.drain(0..self.cursor);
            self.cursor = 0;
        }
    }

    pub fn delete_to_end_of_line(&mut self) {
        if self.delete_selection() {
            return;
        }
        if self.cursor < self.value.len() {
            self.push_undo();
            let len = self.value.len();
            self.value.drain(self.cursor..len);
        }
    }

    pub fn move_left(&mut self, by_word: bool, select: bool) {
        let target = if by_word {
            self.previous_word_boundary(self.cursor)
        } else {
            self.previous_char_boundary(self.cursor)
        };

        if select {
            let anchor = match self.selection {
                Some(ref r) => {
                    if self.cursor == r.start {
                        r.end
                    } else {
                        r.start
                    }
                }
                None => self.cursor,
            };
            self.cursor = target;
            let start = anchor.min(target);
            let end = anchor.max(target);
            self.selection = if start == end { None } else { Some(start..end) };
        } else {
            if let Some(ref r) = self.selection
                && !by_word
            {
                self.cursor = r.start;
                self.selection = None;
                return;
            }
            self.cursor = target;
            self.selection = None;
        }
    }

    pub fn move_right(&mut self, by_word: bool, select: bool) {
        let target = if by_word {
            self.next_word_boundary(self.cursor)
        } else {
            self.next_char_boundary(self.cursor)
        };

        if select {
            let anchor = match self.selection {
                Some(ref r) => {
                    if self.cursor == r.start {
                        r.end
                    } else {
                        r.start
                    }
                }
                None => self.cursor,
            };
            self.cursor = target;
            let start = anchor.min(target);
            let end = anchor.max(target);
            self.selection = if start == end { None } else { Some(start..end) };
        } else {
            if let Some(ref r) = self.selection
                && !by_word
            {
                self.cursor = r.end;
                self.selection = None;
                return;
            }
            self.cursor = target;
            self.selection = None;
        }
    }

    pub fn move_home(&mut self, select: bool) {
        let target = 0;
        if select {
            let anchor = match self.selection {
                Some(ref r) => {
                    if self.cursor == r.start {
                        r.end
                    } else {
                        r.start
                    }
                }
                None => self.cursor,
            };
            self.cursor = target;
            let start = anchor.min(target);
            let end = anchor.max(target);
            self.selection = if start == end { None } else { Some(start..end) };
        } else {
            self.cursor = target;
            self.selection = None;
        }
    }

    pub fn move_end(&mut self, select: bool) {
        let target = self.value.len();
        if select {
            let anchor = match self.selection {
                Some(ref r) => {
                    if self.cursor == r.start {
                        r.end
                    } else {
                        r.start
                    }
                }
                None => self.cursor,
            };
            self.cursor = target;
            let start = anchor.min(target);
            let end = anchor.max(target);
            self.selection = if start == end { None } else { Some(start..end) };
        } else {
            self.cursor = target;
            self.selection = None;
        }
    }

    pub fn copy(&self, cx: &mut App) {
        if let Some(ref r) = self.selection
            && !r.is_empty()
            && r.end <= self.value.len()
        {
            let text = self.value[r.clone()].to_string();
            cx.write_to_clipboard(ClipboardItem::new_string(text));
        }
    }

    pub fn cut(&mut self, cx: &mut App) {
        if let Some(ref r) = self.selection
            && !r.is_empty()
            && r.end <= self.value.len()
        {
            let text = self.value[r.clone()].to_string();
            cx.write_to_clipboard(ClipboardItem::new_string(text));
            self.delete_selection();
        }
    }

    pub fn paste(&mut self, cx: &mut App) {
        if let Some(item) = cx.read_from_clipboard()
            && let Some(text) = item.text()
        {
            let single_line = text.replace(['\r', '\n'], " ");
            self.insert(&single_line);
        }
    }

    pub fn handle_key(&mut self, event: &KeyDownEvent, window: &mut Window, cx: &mut App) -> bool {
        let key = &event.keystroke.key;
        let ctrl = event.keystroke.modifiers.control;
        let alt = event.keystroke.modifiers.alt;
        let shift = event.keystroke.modifiers.shift;
        let platform = event.keystroke.modifiers.platform;
        let word_nav = ctrl || alt;
        let cmd_or_ctrl = ctrl || platform;

        if key == "left" {
            if platform && cfg!(target_os = "macos") {
                self.move_home(shift);
            } else {
                self.move_left(word_nav, shift);
            }
            return true;
        }

        if key == "right" {
            if platform && cfg!(target_os = "macos") {
                self.move_end(shift);
            } else {
                self.move_right(word_nav, shift);
            }
            return true;
        }

        if key == "home" {
            self.move_home(shift);
            return true;
        }

        if key == "end" {
            self.move_end(shift);
            return true;
        }

        if key == "backspace" {
            if word_nav {
                self.delete_to_previous_word();
            } else if platform && cfg!(target_os = "macos") {
                self.delete_to_start_of_line();
            } else {
                self.backspace();
            }
            return true;
        }

        if key == "delete" {
            if word_nav {
                self.delete_to_next_word();
            } else {
                self.delete();
            }
            return true;
        }

        if key == "a" && cmd_or_ctrl {
            self.select_all();
            return true;
        }

        if key == "c" && cmd_or_ctrl {
            self.copy(cx);
            return true;
        }

        if key == "x" && cmd_or_ctrl {
            self.cut(cx);
            return true;
        }

        if key == "v" && cmd_or_ctrl {
            self.paste(cx);
            return true;
        }

        if key == "z" && cmd_or_ctrl && !shift {
            return self.undo();
        }

        if (key == "y" && cmd_or_ctrl) || (key == "z" && cmd_or_ctrl && shift) {
            return self.redo();
        }

        if key == "escape" {
            window.blur();
            return true;
        }

        if key == "enter" {
            window.blur();
            return true;
        }

        if !platform && !ctrl && !alt {
            if let Some(ref text) = event.keystroke.key_char {
                if !text.is_empty() && !text.chars().all(|c| c.is_control()) {
                    self.insert(text);
                    return true;
                }
            } else if key == "space" {
                self.insert(" ");
                return true;
            } else if key.chars().count() == 1 && !key.chars().any(|c| c.is_control()) {
                self.insert(key);
                return true;
            }
        }

        false
    }
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum InputVariant {
    #[default]
    Default,
    Sidebar,
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum InputSize {
    Sm,
    #[default]
    Default,
    Lg,
}

#[allow(dead_code)]
#[allow(clippy::type_complexity)]
#[derive(IntoElement)]
pub struct Input {
    id: ElementId,
    variant: InputVariant,
    size: InputSize,
    placeholder: SharedString,
    value: SharedString,
    cursor: Option<usize>,
    selection: Option<Range<usize>>,
    focus_handle: Option<FocusHandle>,
    leading_icon: Option<IconName>,
    trailing_kbd: Option<SharedString>,
    disabled: bool,
    on_click: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
    on_key_down: Option<Box<dyn Fn(&KeyDownEvent, &mut Window, &mut App) + 'static>>,
    on_clear: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
}

#[allow(dead_code)]
impl Input {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            variant: InputVariant::Default,
            size: InputSize::Default,
            placeholder: "".into(),
            value: "".into(),
            cursor: None,
            selection: None,
            focus_handle: None,
            leading_icon: None,
            trailing_kbd: None,
            disabled: false,
            on_click: None,
            on_key_down: None,
            on_clear: None,
        }
    }

    pub fn sidebar_search(id: impl Into<ElementId>) -> Self {
        Self::new(id)
            .variant(InputVariant::Sidebar)
            .size(InputSize::Sm)
            .placeholder("Search notes...")
            .leading_icon(IconName::Search)
            .trailing_kbd("⌘K")
    }

    pub fn state(mut self, state: &InputState) -> Self {
        self.value = state.value().to_string().into();
        self.cursor = Some(state.cursor());
        self.selection = state.selection();
        self
    }

    pub fn variant(mut self, variant: InputVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn size(mut self, size: InputSize) -> Self {
        self.size = size;
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    pub fn value(mut self, value: impl Into<SharedString>) -> Self {
        self.value = value.into();
        self
    }

    pub fn cursor(mut self, cursor: usize) -> Self {
        self.cursor = Some(cursor);
        self
    }

    pub fn selection(mut self, selection: Range<usize>) -> Self {
        self.selection = Some(selection);
        self
    }

    pub fn focus_handle(mut self, handle: FocusHandle) -> Self {
        self.focus_handle = Some(handle);
        self
    }

    pub fn leading_icon(mut self, icon: IconName) -> Self {
        self.leading_icon = Some(icon);
        self
    }

    pub fn trailing_kbd(mut self, kbd: impl Into<SharedString>) -> Self {
        self.trailing_kbd = Some(kbd.into());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }

    pub fn on_key_down(
        mut self,
        handler: impl Fn(&KeyDownEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_key_down = Some(Box::new(handler));
        self
    }

    pub fn on_clear(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_clear = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for Input {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();

        let (height, padding_x, font_size, icon_size, radius) = match self.size {
            InputSize::Sm => (px(28.), px(8.), px(12.5), px(14.), px(6.)),
            InputSize::Default => (px(32.), px(10.), px(13.5), px(15.), px(6.)),
            InputSize::Lg => (px(40.), px(12.), px(15.0), px(18.), px(8.)),
        };

        let is_focused = self
            .focus_handle
            .as_ref()
            .map(|h| h.is_focused(window))
            .unwrap_or(false);

        let (bg, border_color) = match self.variant {
            InputVariant::Default => (
                theme.card,
                if is_focused { theme.ring } else { theme.border },
            ),
            InputVariant::Sidebar => (
                theme.card,
                if is_focused { theme.ring } else { theme.border },
            ),
        };

        let is_empty = self.value.is_empty();

        let mut el = div()
            .id(self.id)
            .w_full()
            .h(height)
            .px(padding_x)
            .rounded(radius)
            .bg(bg)
            .border_1()
            .border_color(border_color)
            .flex()
            .items_center()
            .justify_between()
            .text_size(font_size);

        if self.disabled {
            el = el.opacity(0.5).cursor_not_allowed();
        } else {
            el = el
                .cursor_text()
                .hover(|s| s.border_color(theme.ring))
                .focus(|s| s.border_color(theme.ring));
        }

        if let Some(ref focus_handle) = self.focus_handle {
            el = el.track_focus(focus_handle);
        }

        if !self.disabled {
            let focus_handle = self.focus_handle.clone();
            let on_click = self.on_click;
            if focus_handle.is_some() || on_click.is_some() {
                el = el.on_click(move |ev, window, cx| {
                    if let Some(ref h) = focus_handle {
                        h.focus(window);
                    }
                    if let Some(ref click) = on_click {
                        click(ev, window, cx);
                    }
                });
            }
        }

        if let Some(on_key_down) = self.on_key_down {
            el = el.on_key_down(move |event, window, cx| {
                on_key_down(event, window, cx);
            });
        }

        if is_focused {
            el = el.on_mouse_down_out(move |_event, window, _| {
                window.blur();
            });
        }

        let caret_bar = div()
            .w(px(1.5))
            .h(px(14.))
            .bg(theme.primary)
            .flex_shrink_0();

        let text_container = if is_empty {
            let mut container = div().flex().items_center().overflow_hidden();
            if is_focused {
                container = container.child(caret_bar.mr(px(2.)));
            }
            container.child(
                div()
                    .text_color(theme.muted_foreground)
                    .whitespace_nowrap()
                    .child(self.placeholder.clone()),
            )
        } else {
            let val_str = self.value.as_ref();
            let clamp_boundary = |offset: usize| -> usize {
                let offset = offset.min(val_str.len());
                if val_str.is_char_boundary(offset) {
                    offset
                } else {
                    val_str
                        .char_indices()
                        .map(|(i, _)| i)
                        .chain(std::iter::once(val_str.len()))
                        .rfind(|&i| i <= offset)
                        .unwrap_or(0)
                }
            };

            let valid_selection = self
                .selection
                .filter(|r| !r.is_empty() && r.start < val_str.len());

            if let Some(range) = valid_selection {
                let start = clamp_boundary(range.start);
                let end = clamp_boundary(range.end);
                let (start, end) = if start <= end {
                    (start, end)
                } else {
                    (end, start)
                };

                let before = &val_str[..start];
                let selected = &val_str[start..end];
                let after = &val_str[end..];

                div()
                    .flex()
                    .items_center()
                    .overflow_hidden()
                    .when(!before.is_empty(), |this| {
                        this.child(
                            div()
                                .text_color(theme.foreground)
                                .whitespace_nowrap()
                                .child(before.to_string()),
                        )
                    })
                    .child(
                        div()
                            .bg(theme.accent)
                            .text_color(theme.foreground)
                            .rounded(px(2.))
                            .whitespace_nowrap()
                            .child(selected.to_string()),
                    )
                    .when(!after.is_empty(), |this| {
                        this.child(
                            div()
                                .text_color(theme.foreground)
                                .whitespace_nowrap()
                                .child(after.to_string()),
                        )
                    })
            } else {
                let cursor_pos = clamp_boundary(self.cursor.unwrap_or(val_str.len()));
                let before = &val_str[..cursor_pos];
                let after = &val_str[cursor_pos..];

                div()
                    .flex()
                    .items_center()
                    .overflow_hidden()
                    .when(!before.is_empty(), |this| {
                        this.child(
                            div()
                                .text_color(theme.foreground)
                                .whitespace_nowrap()
                                .child(before.to_string()),
                        )
                    })
                    .when(is_focused, |this| this.child(caret_bar))
                    .when(!after.is_empty(), |this| {
                        this.child(
                            div()
                                .text_color(theme.foreground)
                                .whitespace_nowrap()
                                .child(after.to_string()),
                        )
                    })
            }
        };

        let leading = div()
            .flex()
            .items_center()
            .gap_2()
            .flex_1()
            .overflow_hidden()
            .children(self.leading_icon.map(|name| {
                Icon::new(name).size(icon_size).color(if is_focused {
                    theme.primary
                } else {
                    theme.muted_foreground
                })
            }))
            .child(text_container);

        el = el.child(leading);

        if !is_empty && let Some(on_clear) = self.on_clear {
            el = el.child(
                div()
                    .id("input-clear-btn")
                    .w(px(16.))
                    .h(px(16.))
                    .rounded(px(3.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .cursor_pointer()
                    .hover(|s| s.bg(theme.secondary).text_color(theme.foreground))
                    .text_color(theme.muted_foreground)
                    .on_click(move |ev, window, cx| {
                        cx.stop_propagation();
                        on_clear(ev, window, cx);
                    })
                    .child(Icon::new(IconName::X).size(px(11.))),
            );
        } else if let Some(kbd) = self.trailing_kbd {
            el = el.child(KbdBadge::new(kbd));
        }

        el
    }
}

#[cfg(test)]
mod tests {
    use super::InputState;

    #[test]
    fn input_state_insert_and_backspace() {
        let mut state = InputState::new("");
        assert_eq!(state.value(), "");
        assert_eq!(state.cursor(), 0);

        state.insert("hello");
        assert_eq!(state.value(), "hello");
        assert_eq!(state.cursor(), 5);

        state.backspace();
        assert_eq!(state.value(), "hell");
        assert_eq!(state.cursor(), 4);
    }

    #[test]
    fn input_state_cursor_middle_insertion() {
        let mut state = InputState::new("helo");
        state.set_cursor(3);
        state.insert("l");
        assert_eq!(state.value(), "hello");
        assert_eq!(state.cursor(), 4);
    }

    #[test]
    fn input_state_word_navigation() {
        let mut state = InputState::new("hello world test");
        assert_eq!(state.cursor(), 16);

        state.move_left(true, false);
        assert_eq!(state.cursor(), 12);

        state.move_left(true, false);
        assert_eq!(state.cursor(), 6);

        state.move_left(true, false);
        assert_eq!(state.cursor(), 0);

        state.move_right(true, false);
        assert_eq!(state.cursor(), 5);

        state.move_right(true, false);
        assert_eq!(state.cursor(), 11);
    }

    #[test]
    fn input_state_word_delete() {
        let mut state = InputState::new("hello world");
        state.delete_to_previous_word();
        assert_eq!(state.value(), "hello ");
        assert_eq!(state.cursor(), 6);

        state.delete_to_previous_word();
        assert_eq!(state.value(), "");
        assert_eq!(state.cursor(), 0);

        state.insert("first second third");
        state.set_cursor(6);
        state.delete_to_next_word();
        assert_eq!(state.value(), "first  third");
    }

    #[test]
    fn input_state_home_and_end() {
        let mut state = InputState::new("hello world");
        state.move_home(false);
        assert_eq!(state.cursor(), 0);

        state.move_end(false);
        assert_eq!(state.cursor(), 11);
    }

    #[test]
    fn input_state_selection_and_replace() {
        let mut state = InputState::new("hello world");
        state.select_all();
        assert_eq!(state.selection(), Some(0..11));

        state.insert("new text");
        assert_eq!(state.value(), "new text");
        assert_eq!(state.cursor(), 8);
        assert_eq!(state.selection(), None);
    }

    #[test]
    fn input_state_undo_and_redo() {
        let mut state = InputState::new("first");
        state.insert(" second");
        assert_eq!(state.value(), "first second");

        assert!(state.undo());
        assert_eq!(state.value(), "first");

        assert!(state.redo());
        assert_eq!(state.value(), "first second");
    }
}
