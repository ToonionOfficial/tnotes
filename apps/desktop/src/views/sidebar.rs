use crate::components::*;
use crate::state::AppState;
use gpui::*;

pub struct SidebarView {
    pub app_state: Entity<AppState>,
}

impl SidebarView {
    pub fn new(app_state: Entity<AppState>, cx: &mut Context<Self>) -> Self {
        // Observe changes to app_state, note_store, and folder_store
        let note_store = app_state.read(cx).note_store.clone();
        let folder_store = app_state.read(cx).folder_store.clone();

        cx.observe(&app_state, |_this, _app_state, cx| cx.notify()).detach();
        cx.observe(&note_store, |_this, _note_store, cx| cx.notify()).detach();
        cx.observe(&folder_store, |_this, _folder_store, cx| cx.notify()).detach();

        Self { app_state }
    }
}

impl Render for SidebarView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let app_state_read = self.app_state.read(cx);
        let note_store_entity = app_state_read.note_store.clone();
        let folder_store_entity = app_state_read.folder_store.clone();
        let app_state_entity = self.app_state.clone();

        let note_store = note_store_entity.read(cx);
        let folder_store = folder_store_entity.read(cx);

        let notes = note_store.notes.clone();
        let selected_id = note_store.selected_note_id.clone();
        let folders = folder_store.folders.clone();

        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(rgb(0x141318)) // Theme background
            .border_r_1()
            .border_color(rgb(0x302e36)) // Theme border
            .p_2p5()
            .gap_2p5()
            // 1. Header
            .child(sidebar_header(
                "TNotes",
                cx.listener({
                    let note_store = note_store_entity.clone();
                    move |_this, _, _window, cx| {
                        note_store.update(cx, |store, cx| {
                            store.create_note("Untitled Note", "<p></p>", None, cx);
                        });
                    }
                }),
            ))
            // 2. Search Bar
            .child(search_bar())
            // 3. Unified Scrollable Tree: Folders and Notes together
            .child(
                div()
                    .id("sidebar-tree-scroll")
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_h_0()
                    .w_full()
                    .overflow_y_scroll()
                    .gap_3()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_0p5()
                            // FOLDERS Section
                            .child(section_header("FOLDERS"))
                            .children(folders.iter().enumerate().map(|(idx, folder)| {
                                folder_row(idx as u64, folder.name.clone(), 0)
                            }))
                            // NOTES Section (unified in same tree)
                            .child(section_header("NOTES"))
                            .children(notes.iter().enumerate().map(|(idx, note)| {
                                let is_selected = selected_id.as_deref() == Some(&note.id);
                                let note_id = note.id.clone();
                                let store = note_store_entity.clone();

                                note_row(
                                    idx as u64,
                                    note.title.clone(),
                                    is_selected,
                                    move |_event, _window, cx| {
                                        let note_id = note_id.clone();
                                        store.update(cx, |store, cx| {
                                            store.select_note(note_id, cx);
                                        });
                                    },
                                )
                            })),
                    ),
            )
            // 4. Pinned SYSTEM Section at Bottom
            .child(system_section())
            // 5. Pinned Profile & Settings Footer
            .child(sidebar_footer(
                "Local Account",
                "Synced with local SQLite",
                cx.listener({
                    let app_state = app_state_entity.clone();
                    move |_this, _, _window, cx| {
                        app_state.update(cx, |state, cx| {
                            state.toggle_settings(cx);
                        });
                    }
                }),
            ))
    }
}

fn section_header(label: &'static str) -> impl IntoElement {
    div()
        .px_2()
        .pt_2()
        .pb_1()
        .text_xs()
        .font_weight(FontWeight::BOLD)
        .text_color(rgb(0x79747e)) // Subtle uppercase category label
        .child(label)
}
