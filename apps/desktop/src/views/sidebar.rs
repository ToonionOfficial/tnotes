use std::collections::HashSet;
use gpui::*;
use crate::components::{
    Button, FolderTreeItem, IconName, Input, Sidebar, SidebarContent, SidebarFooter,
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
        expanded_folders.insert("projects:architecture".to_string());

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
        let arch_expanded = self.expanded_folders.contains("projects:architecture");
        let personal_expanded = self.expanded_folders.contains("personal");

        if self.is_collapsed {
            return div()
                .id("main-sidebar-collapsed")
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
                        .gap_2()
                        .child(
                            Button::icon("sidebar-collapse-btn", IconName::PanelLeft)
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.toggle_collapsed(cx);
                                })),
                        )
                        .child(
                            Button::icon("new-note-collapsed-btn", IconName::Plus)
                                .size(crate::components::ButtonSize::Sm),
                        )
                        .child(
                            Button::icon("search-collapsed-btn", IconName::Search)
                                .size(crate::components::ButtonSize::Sm),
                        ),
                )
                .child(
                    Button::icon("settings-collapsed-btn", IconName::Settings)
                        .size(crate::components::ButtonSize::Sm)
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.select_section("settings", cx);
                        })),
                )
                .into_any_element();
        }

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
                            .gap_2()
                            .child(
                                div()
                                    .w(px(22.))
                                    .h(px(22.))
                                    .rounded(px(6.))
                                    .bg(theme.primary)
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .child(
                                        div()
                                            .text_size(px(12.))
                                            .font_weight(FontWeight::BOLD)
                                            .text_color(theme.primary_foreground)
                                            .child("T"),
                                    ),
                            )
                            .child(
                                div()
                                    .text_size(px(14.))
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(theme.foreground)
                                    .child("TNotes"),
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
                        let mut items = Vec::new();

                        items.push(
                            FolderTreeItem::new("folder-architecture", "Architecture")
                                .depth(1)
                                .icon(IconName::Code)
                                .expanded(arch_expanded)
                                .selected(selected == "folder:architecture")
                                .count(4)
                                .on_toggle(cx.listener(|this, _, _, cx| {
                                    this.toggle_folder("projects:architecture", cx);
                                }))
                                .on_select(cx.listener(|this, _, _, cx| {
                                    this.select_section("folder:architecture", cx);
                                }))
                                .into_any_element(),
                        );

                        if arch_expanded {
                            items.push(
                                FolderTreeItem::new("folder-arch-core", "Core Engine")
                                    .depth(2)
                                    .icon(IconName::Zap)
                                    .selected(selected == "folder:arch-core")
                                    .count(2)
                                    .on_select(cx.listener(|this, _, _, cx| {
                                        this.select_section("folder:arch-core", cx);
                                    }))
                                    .into_any_element(),
                            );
                            items.push(
                                FolderTreeItem::new("folder-arch-desktop", "Desktop Shell")
                                    .depth(2)
                                    .icon(IconName::Rocket)
                                    .selected(selected == "folder:arch-desktop")
                                    .count(2)
                                    .on_select(cx.listener(|this, _, _, cx| {
                                        this.select_section("folder:arch-desktop", cx);
                                    }))
                                    .into_any_element(),
                            );
                        }

                        items.push(
                            FolderTreeItem::new("folder-specs", "Specifications")
                                .depth(1)
                                .icon(IconName::FileText)
                                .selected(selected == "folder:specs")
                                .count(8)
                                .on_select(cx.listener(|this, _, _, cx| {
                                    this.select_section("folder:specs", cx);
                                }))
                                .into_any_element(),
                        );

                        items
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
                    )
                    .children(if personal_expanded {
                        vec![
                            FolderTreeItem::new("folder-journal", "Journal")
                                .depth(1)
                                .icon(IconName::Bookmark)
                                .selected(selected == "folder:journal")
                                .count(3)
                                .on_select(cx.listener(|this, _, _, cx| {
                                    this.select_section("folder:journal", cx);
                                }))
                                .into_any_element(),
                            FolderTreeItem::new("folder-ideas", "Ideas")
                                .depth(1)
                                .icon(IconName::Lightbulb)
                                .selected(selected == "folder:ideas")
                                .count(4)
                                .on_select(cx.listener(|this, _, _, cx| {
                                    this.select_section("folder:ideas", cx);
                                }))
                                .into_any_element(),
                        ]
                    } else {
                        vec![]
                    }),
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
            .collapsed(false)
            .header(header)
            .content(content)
            .footer(footer)
            .into_any_element()
    }
}
