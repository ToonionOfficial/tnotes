use std::collections::HashSet;
use gpui::*;
use crate::components::{
    Button, FolderTreeItem, Icon, IconName, Input, Sidebar, SidebarContent, SidebarFooter,
    SidebarGroup, SidebarHeader,
};
use crate::theme::ThemeExt;

pub struct SidebarView {
    is_collapsed: bool,
    selected_section: String,
    expanded_folders: HashSet<String>,
}

impl SidebarView {
    pub fn new() -> Self {
        let mut expanded_folders = HashSet::new();
        expanded_folders.insert("projects".to_string());

        Self {
            is_collapsed: false,
            selected_section: "all_notes".to_string(),
            expanded_folders,
        }
    }

    pub fn toggle_collapsed(&mut self, cx: &mut Context<Self>) {
        self.is_collapsed = !self.is_collapsed;
        cx.notify();
    }

    pub fn toggle_folder(&mut self, folder_id: &str, cx: &mut Context<Self>) {
        if self.expanded_folders.contains(folder_id) {
            self.expanded_folders.remove(folder_id);
        } else {
            self.expanded_folders.insert(folder_id.to_string());
        }
        cx.notify();
    }

    pub fn select_section(&mut self, section_id: &str, cx: &mut Context<Self>) {
        self.selected_section = section_id.to_string();
        cx.notify();
    }
}

impl Render for SidebarView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let selected = self.selected_section.clone();
        let projects_expanded = self.expanded_folders.contains("projects");
        let personal_expanded = self.expanded_folders.contains("personal");

        let header = SidebarHeader::new()
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .px_1()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_1p5()
                            .child(
                                Icon::new(IconName::Folder)
                                    .size(px(14.))
                                    .color(theme.primary),
                            )
                            .child(
                                div()
                                    .text_size(px(13.5))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(theme.foreground)
                                    .child("Personal Vault"),
                            ),
                    )
                    .child(
                        Button::icon("sidebar-collapse-btn", IconName::PanelLeft)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.toggle_collapsed(cx);
                            })),
                    ),
            )
            .child(
                Button::primary("new-note-btn", "New Note")
                    .leading_icon(IconName::Plus)
                    .full_width(true),
            )
            .child(Input::sidebar_search("sidebar-search-input"));

        let content = SidebarContent::new()
            .child(
                SidebarGroup::new()
                    .child(
                        Button::sidebar("all-notes", "All Notes")
                            .leading_icon(IconName::FileText)
                            .count(42)
                            .active(selected == "all_notes")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.select_section("all_notes", cx);
                            })),
                    )
                    .child(
                        Button::sidebar("starred", "Starred")
                            .leading_icon(IconName::Star)
                            .count(5)
                            .active(selected == "starred")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.select_section("starred", cx);
                            })),
                    )
                    .child(
                        Button::sidebar("trash", "Trash")
                            .leading_icon(IconName::Trash2)
                            .count(2)
                            .active(selected == "trash")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.select_section("trash", cx);
                            })),
                    ),
            )
            .child(
                SidebarGroup::new()
                    .label("Folders")
                    .action(Button::icon("add-folder-btn", IconName::Plus).size(crate::components::ButtonSize::Sm))
                    .child(
                        FolderTreeItem::new("folder-projects", "Projects")
                            .depth(0)
                            .expanded(projects_expanded)
                            .selected(selected == "folder:projects")
                            .count(12)
                            .on_toggle(cx.listener(|this, _, _, cx| {
                                this.toggle_folder("projects", cx);
                            }))
                            .on_select(cx.listener(|this, _, _, cx| {
                                this.select_section("folder:projects", cx);
                            })),
                    )
                    .children(if projects_expanded {
                        vec![
                            FolderTreeItem::new("folder-architecture", "Architecture")
                                .depth(1)
                                .selected(selected == "folder:architecture")
                                .count(4)
                                .on_select(cx.listener(|this, _, _, cx| {
                                    this.select_section("folder:architecture", cx);
                                }))
                                .into_any_element(),
                            FolderTreeItem::new("folder-specs", "Specifications")
                                .depth(1)
                                .selected(selected == "folder:specs")
                                .count(8)
                                .on_select(cx.listener(|this, _, _, cx| {
                                    this.select_section("folder:specs", cx);
                                }))
                                .into_any_element(),
                        ]
                    } else {
                        vec![]
                    })
                    .child(
                        FolderTreeItem::new("folder-personal", "Personal Notes")
                            .depth(0)
                            .expanded(personal_expanded)
                            .selected(selected == "folder:personal")
                            .count(7)
                            .on_toggle(cx.listener(|this, _, _, cx| {
                                this.toggle_folder("personal", cx);
                            }))
                            .on_select(cx.listener(|this, _, _, cx| {
                                this.select_section("folder:personal", cx);
                            })),
                    ),
            );

        let footer = SidebarFooter::new().child(
            Button::sidebar("settings-btn", "Settings")
                .leading_icon(IconName::Settings)
                .active(selected == "settings")
                .on_click(cx.listener(|this, _, _, cx| {
                    this.select_section("settings", cx);
                })),
        );

        Sidebar::new("main-sidebar")
            .width(px(250.))
            .collapsed(self.is_collapsed)
            .header(header)
            .content(content)
            .footer(footer)
    }
}
