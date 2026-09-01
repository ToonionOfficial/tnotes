use crate::state::AppState;
use gpui::prelude::FluentBuilder as _;
use gpui::*;
use gpui_component::{
    breadcrumb::{Breadcrumb, BreadcrumbItem},
    button::{Button, ButtonVariants},
    h_flex, v_flex, Icon, IconName, Sizable,
};
use tnotes_core::models::folder::Folder;
use tnotes_core::models::note::Note;

pub struct FolderDashboardView {
    pub app_state: Entity<AppState>,
}

impl FolderDashboardView {
    pub fn new(app_state: Entity<AppState>, cx: &mut Context<Self>) -> Self {
        let note_store = app_state.read(cx).note_store.clone();
        let folder_store = app_state.read(cx).folder_store.clone();

        cx.observe(&app_state, |_this, _app_state, cx| cx.notify()).detach();
        cx.observe(&note_store, |_this, _note_store, cx| cx.notify()).detach();
        cx.observe(&folder_store, |_this, _folder_store, cx| cx.notify()).detach();

        Self { app_state }
    }
}

impl Render for FolderDashboardView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let app_state_read = self.app_state.read(cx);
        let folder_store_entity = app_state_read.folder_store.clone();
        let note_store_entity = app_state_read.note_store.clone();
        let app_state_entity = self.app_state.clone();

        let folder_store = folder_store_entity.read(cx);
        let note_store = note_store_entity.read(cx);

        let selected_folder_id = folder_store.selected_folder_id.clone();
        let current_folder = selected_folder_id
            .as_deref()
            .and_then(|id| folder_store.folder_by_id(id));

        let current_folder_title = current_folder
            .map(|f| f.name.clone())
            .unwrap_or_else(|| "All Notes".to_string());

        let ancestors = selected_folder_id
            .as_deref()
            .map(|id| folder_store.ancestors_of(id))
            .unwrap_or_default();

        let subfolders = folder_store.subfolders_of(selected_folder_id.as_deref());
        let notes = note_store.notes_in_folder(selected_folder_id.as_deref());

        let total_notes_count = notes.len();
        let subfolders_count = subfolders.len();

        let app_state_for_root = app_state_entity.clone();

        v_flex()
            .id("folder-dashboard")
            .size_full()
            .bg(rgb(0x141318))
            .overflow_y_scroll()
            .items_center()
            .p_8()
            .child(
                v_flex()
                    .w_full()
                    .max_w(px(840.0))
                    .gap_6()
            .child(
                h_flex()
                    .items_center()
                    .justify_between()
                    .w_full()
                    .child(
                        Breadcrumb::new()
                            .child(
                                BreadcrumbItem::new("All Notes").on_click(move |_, _, cx| {
                                    app_state_for_root.update(cx, |state, cx| {
                                        state.navigate_to_folder(None, cx);
                                    });
                                }),
                            )
                            .children(ancestors.into_iter().map(|ancestor| {
                                let anc_id = ancestor.id.clone();
                                let is_current = Some(&anc_id) == selected_folder_id.as_ref();
                                let app_state = app_state_entity.clone();

                                if is_current {
                                    BreadcrumbItem::new(ancestor.name)
                                } else {
                                    BreadcrumbItem::new(ancestor.name).on_click(
                                        move |_, _, cx| {
                                            let anc_id = anc_id.clone();
                                            app_state.update(cx, |state, cx| {
                                                state.navigate_to_folder(Some(anc_id), cx);
                                            });
                                        },
                                    )
                                }
                            })),
                    ),
            )
            .child(
                h_flex()
                    .items_center()
                    .justify_between()
                    .w_full()
                    .child(
                        v_flex()
                            .gap_1()
                            .child(
                                div()
                                    .text_3xl()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(rgb(0xe6e1e9))
                                    .child(current_folder_title),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(0x938f99))
                                    .child(format!("{total_notes_count} Notes • {subfolders_count} Subfolders")),
                            ),
                    )
                    .child(
                        h_flex()
                            .items_center()
                            .gap_2()
                            .child(
                                Button::new("dash-new-folder-btn")
                                    .ghost()
                                    .small()
                                    .icon(Icon::new(IconName::Folder).size_4())
                                    .label("New Folder")
                                    .on_click({
                                        let folder_store = folder_store_entity.clone();
                                        let parent_id = selected_folder_id.clone();
                                        move |_, _, cx| {
                                            let parent_id = parent_id.clone();
                                            folder_store.update(cx, |store, cx| {
                                                store.create_folder("New Folder", parent_id, cx);
                                            });
                                        }
                                    }),
                            )
                            .child(
                                Button::new("dash-new-note-btn")
                                    .primary()
                                    .small()
                                    .icon(Icon::new(IconName::Plus).size_4())
                                    .label("New Note")
                                    .on_click({
                                        let app_state = app_state_entity.clone();
                                        let folder_id = selected_folder_id.clone();
                                        move |_, _, cx| {
                                            let folder_id = folder_id.clone();
                                            app_state.update(cx, |state, cx| {
                                                let new_id = state.note_store.update(cx, |store, cx| {
                                                    store.create_note("Untitled Note", "<p></p>", folder_id, cx)
                                                });
                                                state.open_note(new_id, cx);
                                            });
                                        }
                                    }),
                            ),
                    ),
            )
            .child(div().h_px().bg(rgb(0x302e36)))
            .when(!subfolders.is_empty(), |this| {
                this.child(
                    v_flex()
                        .gap_3()
                        .child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::BOLD)
                                .text_color(rgb(0x79747e))
                                .child("SUBFOLDERS"),
                        )
                        .child(
                            v_flex()
                                .gap_3()
                                .children(subfolders.chunks(4).map(|chunk| {
                                    let mut row = h_flex().w_full().gap_3();
                                    for subfolder in chunk {
                                        let subfolder_id = subfolder.id.clone();
                                        let note_count = note_store.count_in_folder(Some(&subfolder.id));
                                        let app_state = app_state_entity.clone();

                                        row = row.child(render_subfolder_card(
                                            subfolder.clone(),
                                            note_count,
                                            move |_, _, cx| {
                                                let subfolder_id = subfolder_id.clone();
                                                app_state.update(cx, |state, cx| {
                                                    state.navigate_to_folder(Some(subfolder_id), cx);
                                                });
                                            },
                                        ));
                                    }
                                    for _ in chunk.len()..4 {
                                        row = row.child(div().flex_1());
                                    }
                                    row
                                })),
                        ),
                )
            })
            .child(
                v_flex()
                    .gap_3()
                    .flex_1()
                    .child(
                        div()
                            .text_xs()
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(0x79747e))
                            .child("NOTES"),
                    )
                    .when(notes.is_empty(), |this| {
                        this.child(
                            div()
                                .p_8()
                                .flex()
                                .flex_col()
                                .items_center()
                                .justify_center()
                                .gap_2()
                                .child(Icon::new(IconName::FileText).size_8().text_color(rgb(0x79747e)))
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(rgb(0x938f99))
                                        .child("No notes in this folder yet."),
                                ),
                        )
                    })
                    .when(!notes.is_empty(), |this| {
                        this.child(
                            v_flex().gap_2().children(notes.into_iter().enumerate().map(|(idx, note)| {
                                let note_id = note.id.clone();
                                let app_state = app_state_entity.clone();

                                render_note_card(
                                    idx as u64,
                                    note,
                                    move |_, _, cx| {
                                        let note_id = note_id.clone();
                                        app_state.update(cx, |state, cx| {
                                            state.open_note(note_id, cx);
                                        });
                                    },
                                )
                            })),
                        )
                    }),
            )
            )
    }
}

fn render_subfolder_card(
    folder: Folder,
    note_count: usize,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    div()
        .id(ElementId::NamedInteger("subfolder-card".into(), folder.sort_order as u64))
        .flex_1()
        .p_3()
        .rounded_lg()
        .bg(rgb(0x201f24))
        .border_1()
        .border_color(rgb(0x302e36))
        .hover(|s| s.bg(rgb(0x2a2930)).border_color(rgb(0xcabeff)))
        .cursor_pointer()
        .on_click(on_click)
        .child(
            h_flex()
                .items_center()
                .justify_between()
                .child(
                    h_flex()
                        .items_center()
                        .gap_2p5()
                        .child(Icon::new(IconName::Folder).size_4().text_color(rgb(0xcabeff)))
                        .child(
                            div()
                                .text_sm()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(rgb(0xe6e1e9))
                                .child(folder.name),
                        ),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(0x938f99))
                        .child(format!("{note_count}")),
                ),
        )
}

fn render_note_card(
    id_index: u64,
    note: Note,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    let snippet = strip_html_tags(&note.body);
    let preview = if snippet.trim().is_empty() {
        "Empty note".to_string()
    } else {
        snippet
    };

    div()
        .id(ElementId::NamedInteger("dash-note-card".into(), id_index))
        .w_full()
        .p_3()
        .rounded_lg()
        .bg(rgb(0x201f24))
        .border_1()
        .border_color(rgb(0x302e36))
        .hover(|s| s.bg(rgb(0x2a2930)).border_color(rgb(0x3f3d47)))
        .cursor_pointer()
        .on_click(on_click)
        .child(
            v_flex()
                .gap_1()
                .child(
                    h_flex()
                        .items_center()
                        .gap_2()
                        .child(Icon::new(IconName::FileText).size_4().text_color(rgb(0x938f99)))
                        .child(
                            div()
                                .text_sm()
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(rgb(0xe6e1e9))
                                .child(note.title),
                        ),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(0x938f99))
                        .line_clamp(1)
                        .child(preview),
                ),
        )
}

fn strip_html_tags(html: &str) -> String {
    let mut result = String::new();
    let mut inside_tag = false;
    for c in html.chars() {
        if c == '<' {
            inside_tag = true;
        } else if c == '>' {
            inside_tag = false;
        } else if !inside_tag {
            result.push(c);
        }
    }
    result
}
