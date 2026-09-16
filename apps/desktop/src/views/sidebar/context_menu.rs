use super::{SidebarContextTarget, SidebarView};
use crate::components::{
    ContextMenu, ContextMenuContent, ContextMenuItem, ContextMenuLabel, ContextMenuSeparator,
    IconName,
};
use gpui::*;

impl SidebarView {
    pub fn open_context_menu(
        &mut self,
        position: Point<Pixels>,
        target: SidebarContextTarget,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // A new menu takes focus; any inline rename or icon picking ends.
        self.renaming = None;
        self.picking_icon_for = None;
        let viewport = window.viewport_size();
        let max_x = (viewport.width - px(300.)).max(px(0.));
        let max_y = (viewport.height - px(260.)).max(px(0.));
        let clamped = point(position.x.min(max_x), position.y.min(max_y));

        self.context_menu = Some(super::SidebarContextMenu {
            position: clamped,
            target,
        });
        window.focus(&self.context_menu_focus);
        cx.notify();
    }

    pub fn close_context_menu(&mut self, cx: &mut Context<Self>) {
        self.context_menu = None;
        cx.notify();
    }

    pub(super) fn folder_menu_handler(
        folder_id: String,
        folder_name: String,
    ) -> impl Fn(&mut Self, &MouseDownEvent, &mut Window, &mut Context<Self>) + 'static {
        move |this, event, window, cx| {
            this.open_context_menu(
                event.position,
                SidebarContextTarget::Folder {
                    id: folder_id.clone(),
                    name: folder_name.clone(),
                },
                window,
                cx,
            );
        }
    }

    pub(super) fn note_menu_handler(
        note_id: String,
        title: String,
        is_pinned: bool,
    ) -> impl Fn(&mut Self, &MouseDownEvent, &mut Window, &mut Context<Self>) + 'static {
        move |this, event, window, cx| {
            this.open_context_menu(
                event.position,
                SidebarContextTarget::Note {
                    id: note_id.clone(),
                    title: title.clone(),
                    is_pinned,
                },
                window,
                cx,
            );
        }
    }

    pub(super) fn context_menu_element(&self, cx: &mut Context<Self>) -> Option<AnyElement> {
        let state = self.context_menu.clone()?;

        let content = match &state.target {
            SidebarContextTarget::Folder { id, name } => {
                let entity = cx.entity();
                let new_note_id = id.clone();
                let new_note_entity = entity.clone();
                let subfolder_id = id.clone();
                let subfolder_entity = entity.clone();
                let icon_id = id.clone();
                let icon_entity = entity.clone();
                let rename_id = id.clone();
                let rename_entity = entity.clone();
                let delete_id = id.clone();

                ContextMenuContent::new()
                    .child(ContextMenuLabel::new(name.clone()))
                    .child(
                        ContextMenuItem::new("ctx-folder-new-note", "New Note")
                            .icon(IconName::FilePlus)
                            .on_select(move |_, _, cx| {
                                new_note_entity.update(cx, |this, cx| {
                                    this.create_note_in_folder(Some(new_note_id.clone()), cx);
                                });
                            }),
                    )
                    .child(
                        ContextMenuItem::new("ctx-folder-new-subfolder", "New Subfolder")
                            .icon(IconName::FolderPlus)
                            .on_select(move |_, _, cx| {
                                subfolder_entity.update(cx, |this, cx| {
                                    this.create_subfolder(&subfolder_id, cx);
                                });
                            }),
                    )
                    .child(ContextMenuSeparator)
                    .child(
                        ContextMenuItem::new("ctx-folder-icon", "Change Icon")
                            .icon(IconName::Palette)
                            .on_select(move |_, _, cx| {
                                icon_entity.update(cx, |this, cx| {
                                    this.toggle_icon_picker(&icon_id, cx);
                                });
                            }),
                    )
                    .child(
                        ContextMenuItem::new("ctx-folder-rename", "Rename")
                            .icon(IconName::Pencil)
                            .on_select(move |_, window, cx| {
                                rename_entity.update(cx, |this, cx| {
                                    this.begin_rename_folder(&rename_id, window, cx);
                                });
                            }),
                    )
                    .child(
                        ContextMenuItem::new("ctx-folder-delete", "Delete")
                            .icon(IconName::Trash2)
                            .destructive(true)
                            .on_select(move |_, _, cx| {
                                entity.update(cx, |this, cx| {
                                    this.delete_folder(&delete_id, cx);
                                });
                            }),
                    )
            }
            SidebarContextTarget::Note {
                id,
                title,
                is_pinned,
            } => {
                let entity = cx.entity();
                let pin_id = id.clone();
                let pin_entity = entity.clone();
                let rename_id = id.clone();
                let rename_entity = entity.clone();
                let duplicate_id = id.clone();
                let duplicate_entity = entity.clone();
                let delete_id = id.clone();
                let pin_label: SharedString = if *is_pinned {
                    "Unpin from Starred"
                } else {
                    "Pin to Starred"
                }
                .into();

                ContextMenuContent::new()
                    .child(ContextMenuLabel::new(title.clone()))
                    .child(
                        ContextMenuItem::new("ctx-note-pin", pin_label)
                            .icon(IconName::Star)
                            .on_select(move |_, _, cx| {
                                pin_entity.update(cx, |this, cx| {
                                    this.toggle_note_pin(&pin_id, cx);
                                });
                            }),
                    )
                    .child(ContextMenuSeparator)
                    .child(
                        ContextMenuItem::new("ctx-note-rename", "Rename")
                            .icon(IconName::Pencil)
                            .on_select(move |_, window, cx| {
                                rename_entity.update(cx, |this, cx| {
                                    this.begin_rename_note(&rename_id, window, cx);
                                });
                            }),
                    )
                    .child(
                        ContextMenuItem::new("ctx-note-duplicate", "Duplicate")
                            .icon(IconName::Copy)
                            .on_select(move |_, _, cx| {
                                duplicate_entity.update(cx, |this, cx| {
                                    this.duplicate_note(&duplicate_id, cx);
                                });
                            }),
                    )
                    .child(
                        ContextMenuItem::new("ctx-note-delete", "Delete")
                            .icon(IconName::Trash2)
                            .destructive(true)
                            .on_select(move |_, _, cx| {
                                entity.update(cx, |this, cx| {
                                    this.delete_note(&delete_id, cx);
                                });
                            }),
                    )
            }
        };

        let dismiss = cx.entity();
        Some(
            ContextMenu::new("sidebar-context-menu", state.position)
                .focus_handle(self.context_menu_focus.clone())
                .on_dismiss(move |_, cx| {
                    dismiss.update(cx, |this, cx| this.close_context_menu(cx));
                })
                .content(content)
                .into_any_element(),
        )
    }
}
