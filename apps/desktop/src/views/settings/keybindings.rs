use gpui::*;
use crate::components::KbdBadge;
use crate::theme::ThemeExt;
use super::components::SettingsSection;
use super::keybinding_capture::{actions_by_category, display_keystroke};
use super::SettingsView;

pub(super) fn render(view: &SettingsView, cx: &mut Context<SettingsView>) -> AnyElement {
    let theme = cx.theme().clone();
    // Entity first: `render_element` needs `&mut App` while rows borrow `cx`.
    let entity = cx.entity();

    let mut content: Vec<AnyElement> = Vec::new();

    if let Some(capture) = view.capturing() {
        let hint = match (&capture.preview, &capture.conflict) {
            (Some(keys), Some(conflict)) => format!(
                "Bound to {keys} — also used by {conflict}. Press Esc to dismiss."
            ),
            _ => format!("Press keys for “{}” — Esc to cancel", capture.label),
        };
        content.push(
            div()
                .w_full()
                .rounded(px(10.))
                .bg(theme.secondary)
                .border_1()
                .border_color(theme.ring)
                .px(px(16.))
                .py(px(12.))
                .flex()
                .items_center()
                .gap_3()
                .child(
                    div()
                        .text_size(px(13.))
                        .text_color(theme.foreground)
                        .child(hint),
                )
                .into_any_element(),
        );
    }

    for (category, actions) in actions_by_category() {
        let mut section = SettingsSection::new(category);
        for action in actions {
            let bound = view
                .keymap()
                .get_key_for_action(action.id)
                .map(|(key, ctx)| {
                    let label = display_keystroke(&key);
                    match ctx {
                        Some(ctx) => format!("{label} · {ctx}"),
                        None => label,
                    }
                })
                .unwrap_or_else(|| "Unbound".to_string());
            let is_capturing = view
                .capturing()
                .is_some_and(|c| c.action_id == action.id);
            let row_id =
                SharedString::from(format!("settings-key-{}", action.id.replace("tnotes::", "")));
            let action_id = action.id.to_string();
            let label = action.label.to_string();
            // `on_press` gets (&mut Window, &mut App) but capture needs the
            // view entity: captured above before any `&mut App` reborrow.
            let row_entity = entity.clone();
            let row = super::components::SettingsRow::new(row_id, action.label)
                .subtitle(action.description)
                .value(if is_capturing {
                    "Press keys…".to_string()
                } else {
                    bound
                })
                .on_press(move |window, cx| {
                    row_entity.update(cx, |this, cx| {
                        this.begin_capture(&action_id, &label, cx);
                    });
                    let _ = window;
                });
            section = section.child(row);
        }
        let cx_app: &mut App = cx;
        content.push(section.render_element(cx_app));
    }

    // Footer: reset + conflict-aware note, then KbdBadge legend.
    content.push(
        div()
            .w_full()
            .flex()
            .items_center()
            .justify_between()
            .pt(px(4.))
            .child(
                div()
                    .id("settings-keymap-reset")
                    .px(px(12.))
                    .py(px(6.))
                    .rounded(px(6.))
                    .border_1()
                    .border_color(theme.border)
                    .cursor_pointer()
                    .hover(|s| s.bg(theme.secondary))
                    .text_size(px(12.5))
                    .text_color(theme.muted_foreground)
                    .child("Reset to defaults")
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.reset_keymap(cx);
                    })),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(KbdBadge::new("Click a row, press keys, Esc cancels")),
            )
            .into_any_element(),
    );

    div()
        .flex()
        .flex_col()
        .gap_4()
        .children(content)
        .into_any_element()
}
