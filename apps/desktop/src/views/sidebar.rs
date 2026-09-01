use crate::components::app_logo;
use crate::state::AppState;
use gpui::*;
use gpui_component::{
    button::{Button, ButtonVariants},
    h_flex,
    sidebar::{
        Sidebar, SidebarFooter, SidebarGroup, SidebarHeader, SidebarItem, SidebarMenu,
        SidebarMenuItem,
    },
    v_flex, Icon, IconName, Sizable,
};

pub struct SidebarView {
    pub app_state: Entity<AppState>,
}

impl SidebarView {
    pub fn new(app_state: Entity<AppState>, cx: &mut Context<Self>) -> Self {
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

        Sidebar::new("tnotes-sidebar")
            .w(px(280.))
            .header(
                SidebarHeader::new()
                    .child(
                        h_flex()
                            .items_center()
                            .justify_between()
                            .w_full()
                            .child(
                                h_flex()
                                    .items_center()
                                    .gap_2()
                                    .child(app_logo().size_6().rounded_md())
                                    .child(
                                        div()
                                            .text_base()
                                            .font_weight(FontWeight::BOLD)
                                            .text_color(rgb(0xe6e1e9))
                                            .child("TNotes"),
                                    ),
                            )
                            .child(
                                Button::new("sidebar-btn-new-note")
                                    .ghost()
                                    .small()
                                    .icon(Icon::new(IconName::Plus).size_4())
                                    .on_click(cx.listener({
                                        let note_store = note_store_entity.clone();
                                        move |_this, _, _window, cx| {
                                            note_store.update(cx, |store, cx| {
                                                store.create_note("Untitled Note", "<p></p>", None, cx);
                                            });
                                        }
                                    })),
                            ),
                    ),
            )
            .child(
                SidebarGroup::new("NAVIGATION").child(
                    SidebarMenu::new().child(
                        SidebarMenuItem::new("Search")
                            .icon(IconName::Search)
                            .suffix(|_, _| {
                                div()
                                    .px_1p5()
                                    .py_0p5()
                                    .rounded_sm()
                                    .bg(rgb(0x201f24))
                                    .text_xs()
                                    .text_color(rgb(0x938f99))
                                    .child("⌘P")
                            })
                            .on_click({
                                let app_state = app_state_entity.clone();
                                move |_, _, cx| {
                                    app_state.update(cx, |state, cx| {
                                        state.toggle_command_palette(cx);
                                    });
                                }
                            }),
                    ),
                ),
            )
            .child(
                SidebarGroup::new("FOLDERS").child(
                    SidebarMenu::new().children(folders.iter().enumerate().map(|(_idx, folder)| {
                        SidebarMenuItem::new(folder.name.clone())
                            .icon(IconName::Folder)
                            .suffix(|_, _| div().text_xs().text_color(rgb(0x938f99)).child("0"))
                    })),
                ),
            )
            .child(
                SidebarGroup::new("NOTES").child(
                    SidebarMenu::new().children(notes.iter().enumerate().map(|(_idx, note)| {
                        let note_id = note.id.clone();
                        let store = note_store_entity.clone();
                        let is_selected = selected_id.as_deref() == Some(&note.id);

                        SidebarMenuItem::new(note.title.clone())
                            .icon(IconName::FileText)
                            .active(is_selected)
                            .on_click(move |_, _, cx| {
                                let note_id = note_id.clone();
                                store.update(cx, |store, cx| {
                                    store.select_note(note_id, cx);
                                });
                            })
                    })),
                ),
            )
            .footer(
                v_flex()
                    .w_full()
                    .gap_1()
                    .child(
                        SidebarGroup::new("SYSTEM")
                            .child(
                                SidebarMenu::new()
                                    .child(SidebarMenuItem::new("Archive").icon(IconName::Inbox))
                                    .child(
                                        SidebarMenuItem::new("Trash")
                                            .icon(IconName::Delete)
                                            .suffix(|_, _| {
                                                div().text_xs().text_color(rgb(0x938f99)).child("0")
                                            }),
                                    )
                                    .child(SidebarMenuItem::new("Assets").icon(IconName::File)),
                            )
                            .render("system-group", _window, cx),
                    )
                    .child(
                        SidebarFooter::new().child(
                            h_flex()
                                .id("sidebar-profile-footer")
                                .items_center()
                                .justify_between()
                                .w_full()
                                .cursor_pointer()
                                .on_click(cx.listener({
                                    let app_state = app_state_entity.clone();
                                    move |_this, _, _window, cx| {
                                        app_state.update(cx, |state, cx| {
                                            state.toggle_settings(cx);
                                        });
                                    }
                                }))
                                    .child(
                                        h_flex()
                                            .items_center()
                                            .gap_2p5()
                                            .child(
                                                div()
                                                    .w_7()
                                                    .h_7()
                                                    .rounded_full()
                                                    .bg(rgb(0x2a2930))
                                                    .flex()
                                                    .items_center()
                                                    .justify_center()
                                                    .child(
                                                        Icon::new(IconName::User)
                                                            .size_4()
                                                            .text_color(rgb(0xcabeff)),
                                                    ),
                                            )
                                            .child(
                                                v_flex()
                                                    .child(
                                                        div()
                                                            .text_xs()
                                                            .font_weight(FontWeight::SEMIBOLD)
                                                            .child("Local Account"),
                                                    )
                                                    .child(
                                                        h_flex()
                                                            .items_center()
                                                            .gap_1()
                                                            .child(
                                                                div()
                                                                    .w_1p5()
                                                                    .h_1p5()
                                                                    .rounded_full()
                                                                    .bg(rgb(0xa6e3a1)),
                                                            )
                                                            .child(
                                                                div()
                                                                    .text_xs()
                                                                    .text_color(rgb(0x938f99))
                                                                    .child("Synced with local SQLite"),
                                                            ),
                                                    ),
                                            ),
                                    )
                                    .child(
                                        div()
                                            .p_1()
                                            .child(
                                                Icon::new(IconName::Settings)
                                                    .size_4()
                                                    .text_color(rgb(0xc6c2cd)),
                                            ),
                                    ),
                            ),
                    ),
            )
    }
}
