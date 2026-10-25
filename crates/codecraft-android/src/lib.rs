use android_activity::AndroidApp;
use jni::objects::JValue;
use std::path::{Path, PathBuf};

use codecraft_core::syntax::LanguageId;
use codecraft_core::workspace::{FileTreeNode, WorkspaceManager};
use codecraft_toolchain::pty::TerminalStreamCapture;
use codecraft_toolchain::runner::{RunnerManager, StreamType};
use codecraft_ui::icons::Codicon;
use codecraft_ui::layout::{ActivityBarTab, BottomDockTab, LayoutState};
use codecraft_ui::theme::Theme;

const WORKSPACE_DIR: &str = "/sdcard/Download/ArtCraft";

fn ensure_workspace_dir() -> PathBuf {
    let p = PathBuf::from(WORKSPACE_DIR);
    let _ = std::fs::create_dir_all(&p);
    let _ = std::fs::create_dir_all(p.join("Themes"));
    let _ = std::fs::create_dir_all(p.join("Projects"));
    p
}

fn apply_immersive_sticky(app: &AndroidApp) {
    let vm_ptr = app.vm_as_ptr();
    let activity_ptr = app.activity_as_ptr();
    if vm_ptr.is_null() || activity_ptr.is_null() {
        return;
    }

    unsafe {
        let jvm = match jni::JavaVM::from_raw(vm_ptr as *mut jni::sys::JavaVM) {
            Ok(jvm) => jvm,
            Err(_) => return,
        };
        let mut env = match jvm.attach_current_thread() {
            Ok(env) => env,
            Err(_) => return,
        };
        let activity = match jni::objects::JObject::from_raw(activity_ptr as jni::sys::jobject) {
            activity if !activity.as_raw().is_null() => activity,
            _ => return,
        };

        if let Ok(window) = env.call_method(&activity, "getWindow", "()Landroid/view/Window;", &[]) {
            if let Ok(window_obj) = window.l() {
                if let Ok(decor_view) = env.call_method(&window_obj, "getDecorView", "()Landroid/view/View;", &[]) {
                    if let Ok(decor_obj) = decor_view.l() {
                        let flags: i32 = 4096 | 4 | 2 | 1024 | 512 | 256;
                        let _ = env.call_method(&decor_obj, "setSystemUiVisibility", "(I)V", &[JValue::Int(flags)]);
                    }
                }
            }
        }
    }
}

fn to_color32(c: [f32; 4]) -> egui::Color32 {
    egui::Color32::from_rgba_premultiplied(
        (c[0] * 255.0) as u8,
        (c[1] * 255.0) as u8,
        (c[2] * 255.0) as u8,
        (c[3] * 255.0) as u8,
    )
}

fn ui_at_rect<R>(ui: &mut egui::Ui, rect: egui::Rect, add_contents: impl FnOnce(&mut egui::Ui) -> R) -> egui::InnerResponse<R> {
    ui.scope_builder(egui::UiBuilder::new().max_rect(rect), add_contents)
}

struct CodeCraftApp {
    workspace: WorkspaceManager,
    layout: LayoutState,
    theme: Theme,
    theme_names: Vec<String>,
    current_theme_idx: usize,
    status_msg: String,
    search_query: String,
    save_dialog_open: bool,
    new_filename: String,
    image_loader_installed: bool,
    terminal_capture: TerminalStreamCapture,
    terminal_logs: Vec<(StreamType, String)>,
    terminal_input: String,
    runner_manager: RunnerManager,
    palette_filter: String,
}

impl Default for CodeCraftApp {
    fn default() -> Self {
        let root = ensure_workspace_dir();
        let mut workspace = WorkspaceManager::new(&root);

        let default_code = r#"// CodeCraft Mobile IDE (Lapce Architecture)
// 1:1 Native Port with Lapce-Core, Xi-Rope, and Codicons

fn main() {
    println!("Hello from CodeCraft - Lapce Mobile!");
    let features = [
        "Authentic SVG Codicons",
        "Syntax Highlighting",
        "Line Numbers Gutter",
        "Vertical Activity Bar",
        "Integrated Terminal Dock",
    ];
    for f in features {
        println!(" - {}", f);
    }
}
"#;
        let main_path = root.join("main.rs");
        let initial_content = if main_path.exists() {
            std::fs::read_to_string(&main_path).unwrap_or_else(|_| default_code.to_string())
        } else {
            let _ = std::fs::write(&main_path, default_code);
            default_code.to_string()
        };

        workspace.open_file("main.rs".to_string(), Some(main_path.to_string_lossy().to_string()), &initial_content);

        let terminal_capture = TerminalStreamCapture::new();
        terminal_capture.emit_system("=== Lapce Integrated Terminal ===\nType 'help' or click 'Run' to execute active file.\n");

        Self {
            workspace,
            layout: LayoutState::default(),
            theme: Theme::lapce_dark(),
            theme_names: vec![
                "Lapce Dark Modern".to_string(),
                "Sublime Monokai Pro".to_string(),
                "ArtCraft Zinc (Emerald)".to_string(),
            ],
            current_theme_idx: 0,
            status_msg: "Ready".to_string(),
            search_query: String::new(),
            save_dialog_open: false,
            new_filename: "script.rs".to_string(),
            image_loader_installed: false,
            terminal_capture,
            terminal_logs: Vec::new(),
            terminal_input: String::new(),
            runner_manager: RunnerManager::new(),
            palette_filter: String::new(),
        }
    }
}

impl eframe::App for CodeCraftApp {
    fn raw_input_hook(&mut self, ctx: &egui::Context, _raw: &mut egui::RawInput) {
        if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::S)) {
            self.save_current_file();
        }
        if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::B)) {
            self.layout.toggle_sidebar();
        }
        if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::J)) {
            self.layout.toggle_terminal();
        }
        if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::P)) {
            self.layout.toggle_command_palette();
        }
    }

    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if !self.image_loader_installed {
            egui_extras::install_image_loaders(ctx);
            self.image_loader_installed = true;
        }

        let mut count = 0;
        while let Some(chunk) = self.terminal_capture.try_recv() {
            self.terminal_logs.push((chunk.stream_type, chunk.content));
            if self.terminal_logs.len() > 1000 {
                self.terminal_logs.remove(0);
            }
            count += 1;
            if count > 20 {
                break;
            }
        }
        if count > 0 {
            ctx.request_repaint();
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let bg_app = to_color32(self.theme.bg_app);
        let bg_editor = to_color32(self.theme.bg_editor);
        let bg_sidebar = to_color32(self.theme.bg_sidebar);
        let bg_terminal = to_color32(self.theme.bg_terminal);
        let bg_tabs = to_color32(self.theme.bg_tabs);
        let fg_primary = to_color32(self.theme.fg_primary);
        let fg_muted = to_color32(self.theme.fg_muted);
        let accent_lapce = to_color32(self.theme.accent_blue);
        let accent_em = to_color32(self.theme.accent_emerald);
        let border_subtle = to_color32(self.theme.border_subtle);

        let full_rect = ui.max_rect();
        ui.painter().rect_filled(full_rect, 0.0, bg_app);

        // TOP LAPCE TITLE BAR (Centered Search / Command Palette Bar + Actions)
        let title_bar_height = 36.0;
        let title_bar_rect = egui::Rect::from_min_size(full_rect.min, egui::vec2(full_rect.width(), title_bar_height));
        ui.painter().rect_filled(title_bar_rect, 0.0, bg_app);

        ui_at_rect(ui, title_bar_rect, |ui| {
            ui.horizontal_centered(|ui| {
                ui.add_space(8.0);
                let remote_rect = ui.allocate_space(egui::vec2(22.0, 22.0)).1;
                ui.painter().rect_filled(remote_rect, 3.0, accent_lapce);
                Codicon::Remote.paint(ui, remote_rect, 14.0, egui::Color32::WHITE);
                ui.add_space(4.0);

                ui.label(egui::RichText::new("CodeCraft").strong().size(13.5).color(fg_primary));
                ui.add_space(16.0);

                let palette_rect = ui.allocate_space(egui::vec2(320.0, 24.0)).1;
                ui.painter().rect_filled(palette_rect, 4.0, bg_sidebar);
                ui.painter().rect_stroke(palette_rect, 4.0, egui::Stroke::new(1.0, border_subtle), egui::StrokeKind::Inside);

                let search_icon_rect = egui::Rect::from_min_size(
                    egui::pos2(palette_rect.left() + 6.0, palette_rect.center().y - 7.0),
                    egui::vec2(14.0, 14.0),
                );
                Codicon::Search.paint(ui, search_icon_rect, 14.0, fg_muted);

                ui.painter().text(
                    egui::pos2(palette_rect.left() + 26.0, palette_rect.center().y),
                    egui::Align2::LEFT_CENTER,
                    "Search files or commands (Ctrl+P)",
                    egui::FontId::proportional(12.0),
                    fg_muted,
                );

                if ui.interact(palette_rect, ui.id().with("palette_btn"), egui::Sense::click()).clicked() {
                    self.layout.toggle_command_palette();
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(8.0);
                    let run_btn = ui.button(egui::RichText::new("▶ Run").color(accent_em).strong());
                    if run_btn.clicked() {
                        self.run_active_file();
                    }
                    ui.add_space(4.0);
                    let term_btn = ui.button(if self.layout.terminal_visible { "Terminal [ON]" } else { "Terminal" });
                    if term_btn.clicked() {
                        self.layout.toggle_terminal();
                    }
                    ui.add_space(4.0);
                    if ui.button("Save").clicked() {
                        self.save_current_file();
                    }
                });
            });
        });

        ui.painter().line_segment(
            [egui::pos2(full_rect.left(), title_bar_rect.bottom()), egui::pos2(full_rect.right(), title_bar_rect.bottom())],
            egui::Stroke::new(1.0, border_subtle),
        );

        let term_height = if self.layout.terminal_visible { self.layout.terminal_height } else { 0.0 };
        let accessory_height = if self.layout.accessory_bar_visible { 38.0 } else { 0.0 };
        let body_top = title_bar_rect.bottom();
        let body_height = (full_rect.height() - body_top - term_height - accessory_height - 24.0).max(100.0);

        let body_rect = egui::Rect::from_min_size(egui::pos2(full_rect.left(), body_top), egui::vec2(full_rect.width(), body_height));

        ui_at_rect(ui, body_rect, |ui| {
            ui.horizontal(|ui| {
                let act_width = 44.0;
                let act_rect = egui::Rect::from_min_size(ui.cursor().min, egui::vec2(act_width, body_height));
                ui.painter().rect_filled(act_rect, 0.0, bg_app);

                ui_at_rect(ui, act_rect, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.add_space(6.0);

                        let is_exp = self.layout.active_activity_tab == ActivityBarTab::Explorer && self.layout.sidebar_visible;
                        let exp_rect = ui.allocate_space(egui::vec2(32.0, 32.0)).1;
                        if is_exp {
                            ui.painter().rect_filled(exp_rect, 4.0, bg_sidebar);
                            ui.painter().rect_filled(
                                egui::Rect::from_min_size(egui::pos2(act_rect.left(), exp_rect.top() + 4.0), egui::vec2(2.5, 24.0)),
                                1.0,
                                accent_lapce,
                            );
                        }
                        Codicon::Files.paint(ui, exp_rect, 20.0, if is_exp { fg_primary } else { fg_muted });
                        if ui.interact(exp_rect, ui.id().with("act_files"), egui::Sense::click()).clicked() {
                            self.layout.set_activity_tab(ActivityBarTab::Explorer);
                        }
                        ui.add_space(6.0);

                        let is_search = self.layout.active_activity_tab == ActivityBarTab::Search && self.layout.sidebar_visible;
                        let search_rect = ui.allocate_space(egui::vec2(32.0, 32.0)).1;
                        if is_search {
                            ui.painter().rect_filled(search_rect, 4.0, bg_sidebar);
                            ui.painter().rect_filled(
                                egui::Rect::from_min_size(egui::pos2(act_rect.left(), search_rect.top() + 4.0), egui::vec2(2.5, 24.0)),
                                1.0,
                                accent_lapce,
                            );
                        }
                        Codicon::Search.paint(ui, search_rect, 20.0, if is_search { fg_primary } else { fg_muted });
                        if ui.interact(search_rect, ui.id().with("act_search"), egui::Sense::click()).clicked() {
                            self.layout.set_activity_tab(ActivityBarTab::Search);
                        }
                        ui.add_space(6.0);

                        let is_scm = self.layout.active_activity_tab == ActivityBarTab::SourceControl && self.layout.sidebar_visible;
                        let scm_rect = ui.allocate_space(egui::vec2(32.0, 32.0)).1;
                        if is_scm {
                            ui.painter().rect_filled(scm_rect, 4.0, bg_sidebar);
                            ui.painter().rect_filled(
                                egui::Rect::from_min_size(egui::pos2(act_rect.left(), scm_rect.top() + 4.0), egui::vec2(2.5, 24.0)),
                                1.0,
                                accent_lapce,
                            );
                        }
                        Codicon::SourceControl.paint(ui, scm_rect, 20.0, if is_scm { fg_primary } else { fg_muted });
                        if ui.interact(scm_rect, ui.id().with("act_scm"), egui::Sense::click()).clicked() {
                            self.layout.set_activity_tab(ActivityBarTab::SourceControl);
                        }
                        ui.add_space(6.0);

                        let is_ext = self.layout.active_activity_tab == ActivityBarTab::Extensions && self.layout.sidebar_visible;
                        let ext_rect = ui.allocate_space(egui::vec2(32.0, 32.0)).1;
                        if is_ext {
                            ui.painter().rect_filled(ext_rect, 4.0, bg_sidebar);
                            ui.painter().rect_filled(
                                egui::Rect::from_min_size(egui::pos2(act_rect.left(), ext_rect.top() + 4.0), egui::vec2(2.5, 24.0)),
                                1.0,
                                accent_lapce,
                            );
                        }
                        Codicon::Extensions.paint(ui, ext_rect, 20.0, if is_ext { fg_primary } else { fg_muted });
                        if ui.interact(ext_rect, ui.id().with("act_ext"), egui::Sense::click()).clicked() {
                            self.layout.set_activity_tab(ActivityBarTab::Extensions);
                        }

                        ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                            ui.add_space(6.0);
                            let set_rect = ui.allocate_space(egui::vec2(32.0, 32.0)).1;
                            Codicon::Settings.paint(ui, set_rect, 20.0, fg_muted);
                            if ui.interact(set_rect, ui.id().with("act_settings"), egui::Sense::click()).clicked() {
                                self.layout.toggle_settings();
                            }
                        });
                    });
                });

                ui.painter().line_segment(
                    [egui::pos2(act_rect.right(), act_rect.top()), egui::pos2(act_rect.right(), act_rect.bottom())],
                    egui::Stroke::new(1.0, border_subtle),
                );

                if self.layout.sidebar_visible {
                    let side_rect = egui::Rect::from_min_size(egui::pos2(act_rect.right() + 1.0, body_top), egui::vec2(self.layout.sidebar_width, body_height));
                    ui.painter().rect_filled(side_rect, 0.0, bg_sidebar);

                    ui_at_rect(ui, side_rect, |ui| {
                        ui.vertical(|ui| {
                            ui.add_space(6.0);
                            match self.layout.active_activity_tab {
                                ActivityBarTab::Explorer => {
                                    ui.horizontal(|ui| {
                                        ui.add_space(8.0);
                                        ui.label(egui::RichText::new("EXPLORER").strong().size(11.0).color(fg_muted));
                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                            ui.add_space(6.0);
                                            let r_rect = ui.allocate_space(egui::vec2(16.0, 16.0)).1;
                                            Codicon::Refresh.paint(ui, r_rect, 14.0, fg_muted);
                                            if ui.interact(r_rect, ui.id().with("side_ref"), egui::Sense::click()).clicked() {
                                                self.workspace.refresh_file_tree();
                                            }
                                            let nf_rect = ui.allocate_space(egui::vec2(16.0, 16.0)).1;
                                            Codicon::NewFile.paint(ui, nf_rect, 14.0, fg_muted);
                                            if ui.interact(nf_rect, ui.id().with("side_nf"), egui::Sense::click()).clicked() {
                                                self.save_dialog_open = true;
                                            }
                                        });
                                    });
                                    ui.add_space(2.0);
                                    ui.painter().line_segment(
                                        [egui::pos2(side_rect.left(), ui.cursor().top()), egui::pos2(side_rect.right(), ui.cursor().top())],
                                        egui::Stroke::new(1.0, border_subtle),
                                    );
                                    ui.add_space(4.0);

                                    egui::ScrollArea::vertical().id_salt("sidebar_scroll").show(ui, |ui| {
                                        let mut path_to_open = None;
                                        let mut path_to_toggle = None;

                                        Self::render_tree_nodes(
                                            ui,
                                            self.workspace.file_tree(),
                                            &mut path_to_open,
                                            &mut path_to_toggle,
                                            fg_primary,
                                            fg_muted,
                                            accent_lapce,
                                        );

                                        if let Some(toggle_p) = path_to_toggle {
                                            self.workspace.toggle_expand(&toggle_p);
                                        }
                                        if let Some(open_p) = path_to_open {
                                            self.open_file_from_path(&open_p);
                                        }
                                    });
                                }
                                ActivityBarTab::Search => {
                                    ui.horizontal(|ui| {
                                        ui.add_space(8.0);
                                        ui.label(egui::RichText::new("SEARCH").strong().size(11.0).color(fg_muted));
                                    });
                                    ui.add_space(4.0);
                                    ui.horizontal(|ui| {
                                        ui.add_space(6.0);
                                        ui.text_edit_singleline(&mut self.search_query);
                                    });
                                    ui.add_space(8.0);
                                    ui.label(egui::RichText::new("  Results in workspace:").small().color(fg_muted));
                                }
                                ActivityBarTab::SourceControl => {
                                    ui.horizontal(|ui| {
                                        ui.add_space(8.0);
                                        ui.label(egui::RichText::new("SOURCE CONTROL").strong().size(11.0).color(fg_muted));
                                    });
                                    ui.add_space(6.0);
                                    ui.horizontal(|ui| {
                                        ui.add_space(8.0);
                                        let b_rect = ui.allocate_space(egui::vec2(16.0, 16.0)).1;
                                        Codicon::GitBranch.paint(ui, b_rect, 14.0, accent_lapce);
                                        ui.label(egui::RichText::new("main").strong().size(12.0).color(accent_lapce));
                                    });
                                    ui.add_space(4.0);
                                    ui.label(egui::RichText::new("  Workspace: Clean (Up to date)").small().color(fg_muted));
                                }
                                ActivityBarTab::Extensions => {
                                    ui.horizontal(|ui| {
                                        ui.add_space(8.0);
                                        ui.label(egui::RichText::new("EXTENSIONS & RUNNERS").strong().size(11.0).color(fg_muted));
                                    });
                                    ui.add_space(6.0);
                                    ui.label(egui::RichText::new("  • Rust Toolchain (Native)").small().color(fg_primary));
                                    ui.label(egui::RichText::new("  • Java Dalvik (ECJ+D8)").small().color(fg_primary));
                                    ui.label(egui::RichText::new("  • Python 3 Local").small().color(fg_primary));
                                    ui.label(egui::RichText::new("  • Clang / C++ NDK").small().color(fg_primary));
                                }
                                ActivityBarTab::Settings => {}
                            }
                        });
                    });

                    ui.painter().line_segment(
                        [egui::pos2(side_rect.right(), side_rect.top()), egui::pos2(side_rect.right(), side_rect.bottom())],
                        egui::Stroke::new(1.0, border_subtle),
                    );
                }

                let editor_x = if self.layout.sidebar_visible {
                    act_width + self.layout.sidebar_width + 2.0
                } else {
                    act_width + 1.0
                };
                let editor_rect = egui::Rect::from_min_size(
                    egui::pos2(editor_x, body_top),
                    egui::vec2(full_rect.right() - editor_x, body_height),
                );

                ui_at_rect(ui, editor_rect, |ui| {
                    ui.painter().rect_filled(editor_rect, 0.0, bg_editor);

                    let tab_bar_height = 30.0;
                    let tab_bar_rect = egui::Rect::from_min_size(editor_rect.min, egui::vec2(editor_rect.width(), tab_bar_height));
                    ui.painter().rect_filled(tab_bar_rect, 0.0, bg_tabs);

                    ui_at_rect(ui, tab_bar_rect, |ui| {
                        ui.horizontal_centered(|ui| {
                            ui.add_space(2.0);
                            let tab_count = self.workspace.tabs().len();
                            let mut tab_to_select = None;
                            let mut tab_to_close = None;

                            for (idx, tab) in self.workspace.tabs().iter().enumerate() {
                                let is_active = idx == self.workspace.active_tab_idx();
                                let tab_w = 140.0f32;
                                let t_rect = ui.allocate_space(egui::vec2(tab_w, tab_bar_height)).1;

                                if is_active {
                                    ui.painter().rect_filled(t_rect, 0.0, bg_editor);
                                    ui.painter().rect_filled(
                                        egui::Rect::from_min_size(t_rect.min, egui::vec2(t_rect.width(), 2.0)),
                                        0.0,
                                        accent_lapce,
                                    );
                                }

                                let ic_rect = egui::Rect::from_min_size(
                                    egui::pos2(t_rect.left() + 6.0, t_rect.center().y - 7.0),
                                    egui::vec2(14.0, 14.0),
                                );
                                Codicon::FileCode.paint(ui, ic_rect, 13.0, if is_active { accent_lapce } else { fg_muted });

                                let title_text = if tab.is_dirty {
                                    format!("{} •", tab.title)
                                } else {
                                    tab.title.clone()
                                };
                                ui.painter().text(
                                    egui::pos2(t_rect.left() + 24.0, t_rect.center().y),
                                    egui::Align2::LEFT_CENTER,
                                    title_text,
                                    egui::FontId::proportional(12.5),
                                    if is_active { fg_primary } else { fg_muted },
                                );

                                if tab_count > 1 {
                                    let cl_rect = egui::Rect::from_min_size(
                                        egui::pos2(t_rect.right() - 20.0, t_rect.center().y - 7.0),
                                        egui::vec2(14.0, 14.0),
                                    );
                                    Codicon::Close.paint(ui, cl_rect, 11.0, fg_muted);
                                    if ui.interact(cl_rect, ui.id().with(("tab_cl", idx)), egui::Sense::click()).clicked() {
                                        tab_to_close = Some(idx);
                                    }
                                }

                                if ui.interact(t_rect, ui.id().with(("tab_sel", idx)), egui::Sense::click()).clicked() {
                                    tab_to_select = Some(idx);
                                }
                            }

                            if let Some(idx) = tab_to_select {
                                self.workspace.select_tab(idx);
                            }
                            if let Some(idx) = tab_to_close {
                                self.workspace.close_tab(idx);
                            }
                        });
                    });

                    ui.painter().line_segment(
                        [egui::pos2(editor_rect.left(), tab_bar_rect.bottom()), egui::pos2(editor_rect.right(), tab_bar_rect.bottom())],
                        egui::Stroke::new(1.0, border_subtle),
                    );

                    let breadcrumb_height = 24.0;
                    let breadcrumb_rect = egui::Rect::from_min_size(
                        egui::pos2(editor_rect.left(), tab_bar_rect.bottom()),
                        egui::vec2(editor_rect.width(), breadcrumb_height),
                    );
                    ui_at_rect(ui, breadcrumb_rect, |ui| {
                        ui.horizontal_centered(|ui| {
                            ui.add_space(8.0);
                            let f_rect = ui.allocate_space(egui::vec2(14.0, 14.0)).1;
                            Codicon::Folder.paint(ui, f_rect, 13.0, fg_muted);
                            ui.label(egui::RichText::new("ArtCraft").size(11.5).color(fg_muted));
                            ui.label(egui::RichText::new(">").size(11.5).color(fg_muted));

                            let fc_rect = ui.allocate_space(egui::vec2(14.0, 14.0)).1;
                            Codicon::FileCode.paint(ui, fc_rect, 13.0, accent_lapce);
                            let cur_name = self.workspace.active_tab().map(|t| t.title.as_str()).unwrap_or("Untitled");
                            ui.label(egui::RichText::new(cur_name).size(11.5).color(fg_primary));
                        });
                    });

                    ui.painter().line_segment(
                        [egui::pos2(editor_rect.left(), breadcrumb_rect.bottom()), egui::pos2(editor_rect.right(), breadcrumb_rect.bottom())],
                        egui::Stroke::new(1.0, border_subtle),
                    );

                    let editor_body_top = breadcrumb_rect.bottom();
                    let editor_body_rect = egui::Rect::from_min_size(
                        egui::pos2(editor_rect.left(), editor_body_top),
                        egui::vec2(editor_rect.width(), editor_rect.bottom() - editor_body_top),
                    );

                    let mut file_content = if let Some(tab) = self.workspace.active_tab() {
                        tab.buffer.to_string()
                    } else {
                        String::new()
                    };

                    let active_ext = self.workspace.active_tab()
                        .and_then(|t| t.path.as_ref())
                        .map(|p| Path::new(p).extension().and_then(|e| e.to_str()).unwrap_or("rs"))
                        .unwrap_or("rs");
                    let lang_id = LanguageId::from_extension(active_ext);
                    let theme_clone = self.theme.clone();

                    let line_count = file_content.lines().count().max(1);
                    let gutter_width = 46.0;

                    ui_at_rect(ui, editor_body_rect, |ui| {
                        egui::ScrollArea::both()
                            .id_salt("editor_scroll_area")
                            .auto_shrink([false, false])
                            .scroll_source(egui::scroll_area::ScrollSource {
                                mouse_wheel: true,
                                drag: egui::scroll_area::DragScroll::Always,
                                scroll_bar: true,
                            })
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    let row_h = ui.text_style_height(&egui::TextStyle::Monospace);
                                    let gutter_total_h = (line_count as f32) * row_h;
                                    let g_rect = ui.allocate_space(egui::vec2(gutter_width, gutter_total_h)).1;
                                    ui.painter().rect_filled(g_rect, 0.0, bg_app);
                                    ui.painter().line_segment(
                                        [egui::pos2(g_rect.right(), g_rect.top()), egui::pos2(g_rect.right(), g_rect.bottom())],
                                        egui::Stroke::new(1.0, border_subtle),
                                    );

                                    for i in 1..=line_count {
                                        let y = g_rect.top() + (i as f32 - 1.0) * row_h + row_h * 0.5;
                                        ui.painter().text(
                                            egui::pos2(g_rect.right() - 8.0, y),
                                            egui::Align2::RIGHT_CENTER,
                                            format!("{}", i),
                                            egui::FontId::monospace(13.0),
                                            fg_muted,
                                        );
                                    }

                                    let mut layouter = |ui: &egui::Ui, buf: &dyn egui::TextBuffer, _wrap: f32| {
                                        let job = codecraft_ui::render_syntax_job(buf.as_str(), lang_id, &theme_clone, 14.0, f32::INFINITY);
                                        ui.painter().layout_job(job)
                                    };

                                    let text_edit = egui::TextEdit::multiline(&mut file_content)
                                        .font(egui::TextStyle::Monospace)
                                        .code_editor()
                                        .layouter(&mut layouter)
                                        .desired_width(f32::INFINITY)
                                        .desired_rows(38);

                                    let resp = ui.add(text_edit);
                                    if resp.changed() {
                                        if let Some(tab) = self.workspace.active_tab_mut() {
                                            tab.buffer = codecraft_core::buffer::DocumentBuffer::from_str(&file_content);
                                            tab.is_dirty = true;
                                            self.status_msg = "Modified".to_string();
                                        }
                                    }
                                });
                            });
                    });
                });
            });
        });

        if self.layout.accessory_bar_visible {
            let acc_rect = egui::Rect::from_min_size(
                egui::pos2(full_rect.left(), full_rect.bottom() - term_height - 24.0 - accessory_height),
                egui::vec2(full_rect.width(), accessory_height),
            );
            ui.painter().rect_filled(acc_rect, 0.0, bg_sidebar);
            ui.painter().line_segment(
                [egui::pos2(acc_rect.left(), acc_rect.top()), egui::pos2(acc_rect.right(), acc_rect.top())],
                egui::Stroke::new(1.0, border_subtle),
            );

            ui_at_rect(ui, acc_rect, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.add_space(6.0);
                    ui.label(egui::RichText::new("Touch:").small().color(fg_muted));
                    let symbols = ["Tab", "{", "}", "(", ")", "[", "]", ";", ":", "\"", "'", "->", "=>", "=", "<", ">"];
                    for sym in symbols {
                        if ui.button(sym).clicked() {
                            self.insert_symbol(sym);
                        }
                    }
                });
            });
        }

        if self.layout.terminal_visible {
            let dock_top = full_rect.bottom() - 24.0 - term_height;
            let dock_rect = egui::Rect::from_min_size(egui::pos2(full_rect.left(), dock_top), egui::vec2(full_rect.width(), term_height));
            ui.painter().rect_filled(dock_rect, 0.0, bg_terminal);
            ui.painter().line_segment(
                [egui::pos2(dock_rect.left(), dock_rect.top()), egui::pos2(dock_rect.right(), dock_rect.top())],
                egui::Stroke::new(1.0, border_subtle),
            );

            ui_at_rect(ui, dock_rect, |ui| {
                ui.vertical(|ui| {
                    ui.horizontal_centered(|ui| {
                        ui.add_space(8.0);
                        let is_term = self.layout.active_dock_tab == BottomDockTab::Terminal;
                        let t_ic = ui.allocate_space(egui::vec2(14.0, 14.0)).1;
                        Codicon::Terminal.paint(ui, t_ic, 13.0, if is_term { accent_lapce } else { fg_muted });
                        if ui.selectable_label(is_term, "TERMINAL").clicked() {
                            self.layout.active_dock_tab = BottomDockTab::Terminal;
                        }

                        let is_out = self.layout.active_dock_tab == BottomDockTab::Output;
                        let o_ic = ui.allocate_space(egui::vec2(14.0, 14.0)).1;
                        Codicon::FileCode.paint(ui, o_ic, 13.0, if is_out { accent_lapce } else { fg_muted });
                        if ui.selectable_label(is_out, "OUTPUT").clicked() {
                            self.layout.active_dock_tab = BottomDockTab::Output;
                        }

                        let is_prob = self.layout.active_dock_tab == BottomDockTab::Problems;
                        let p_ic = ui.allocate_space(egui::vec2(14.0, 14.0)).1;
                        Codicon::Error.paint(ui, p_ic, 13.0, if is_prob { accent_lapce } else { fg_muted });
                        if ui.selectable_label(is_prob, "PROBLEMS").clicked() {
                            self.layout.active_dock_tab = BottomDockTab::Problems;
                        }

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.add_space(8.0);
                            let cl_btn = ui.allocate_space(egui::vec2(16.0, 16.0)).1;
                            Codicon::Close.paint(ui, cl_btn, 13.0, fg_muted);
                            if ui.interact(cl_btn, ui.id().with("dock_close"), egui::Sense::click()).clicked() {
                                self.layout.terminal_visible = false;
                            }
                            ui.add_space(8.0);
                            if ui.button("Clear").clicked() {
                                self.terminal_logs.clear();
                            }
                        });
                    });

                    ui.painter().line_segment(
                        [egui::pos2(dock_rect.left(), ui.cursor().top()), egui::pos2(dock_rect.right(), ui.cursor().top())],
                        egui::Stroke::new(1.0, border_subtle),
                    );

                    egui::ScrollArea::vertical()
                        .id_salt("dock_scroll")
                        .max_height(self.layout.terminal_height - 70.0)
                        .stick_to_bottom(true)
                        .show(ui, |ui| {
                            for (stype, text) in &self.terminal_logs {
                                let color = match stype {
                                    StreamType::Stdout => fg_primary,
                                    StreamType::Stderr => egui::Color32::from_rgb(244, 63, 94),
                                    StreamType::SystemNotice => accent_em,
                                };
                                ui.label(egui::RichText::new(text).monospace().small().color(color));
                            }
                        });

                    ui.horizontal(|ui| {
                        ui.add_space(6.0);
                        ui.label(egui::RichText::new("$").monospace().strong().color(accent_lapce));
                        let resp = ui.text_edit_singleline(&mut self.terminal_input);
                        if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                            let cmd = self.terminal_input.trim().to_string();
                            if !cmd.is_empty() {
                                self.execute_terminal_command(&cmd);
                                self.terminal_input.clear();
                            }
                        }
                        if ui.button("Send").clicked() {
                            let cmd = self.terminal_input.trim().to_string();
                            if !cmd.is_empty() {
                                self.execute_terminal_command(&cmd);
                                self.terminal_input.clear();
                            }
                        }
                    });
                });
            });
        }

        let status_rect = egui::Rect::from_min_size(
            egui::pos2(full_rect.left(), full_rect.bottom() - 24.0),
            egui::vec2(full_rect.width(), 24.0),
        );
        ui.painter().rect_filled(status_rect, 0.0, bg_app);
        ui.painter().line_segment(
            [egui::pos2(status_rect.left(), status_rect.top()), egui::pos2(status_rect.right(), status_rect.top())],
            egui::Stroke::new(1.0, border_subtle),
        );

        let ctx = ui.ctx().clone();
        ui_at_rect(ui, status_rect, |ui| {
            ui.horizontal_centered(|ui| {
                ui.add_space(6.0);
                let rm_rect = ui.allocate_space(egui::vec2(14.0, 14.0)).1;
                Codicon::Remote.paint(ui, rm_rect, 13.0, accent_lapce);
                ui.label(egui::RichText::new("Local").small().color(accent_lapce));
                ui.separator();

                let gb_rect = ui.allocate_space(egui::vec2(14.0, 14.0)).1;
                Codicon::GitBranch.paint(ui, gb_rect, 13.0, fg_muted);
                ui.label(egui::RichText::new("main").small().color(fg_primary));
                ui.separator();

                let err_rect = ui.allocate_space(egui::vec2(14.0, 14.0)).1;
                Codicon::Error.paint(ui, err_rect, 12.0, fg_muted);
                ui.label(egui::RichText::new("0").small().color(fg_muted));

                let warn_rect = ui.allocate_space(egui::vec2(14.0, 14.0)).1;
                Codicon::Warning.paint(ui, warn_rect, 12.0, fg_muted);
                ui.label(egui::RichText::new("0").small().color(fg_muted));

                ui.separator();
                ui.label(egui::RichText::new(&self.status_msg).small().color(fg_muted));

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(8.0);
                    ui.label(egui::RichText::new("Rust ARM64").small().color(fg_muted));
                    ui.separator();
                    ui.label(egui::RichText::new("UTF-8").small().color(fg_muted));
                    ui.separator();
                    let cur_lang = self.workspace.active_tab()
                        .and_then(|t| t.path.as_ref())
                        .map(|p| match Path::new(p).extension().and_then(|e| e.to_str()).unwrap_or("") {
                            "rs" => "Rust",
                            "java" => "Java",
                            "py" => "Python",
                            "c" | "cpp" => "C/C++",
                            _ => "Plain Text",
                        })
                        .unwrap_or("Rust");
                    ui.label(egui::RichText::new(cur_lang).small().color(fg_primary));
                    ui.separator();
                    ui.label(egui::RichText::new(&self.theme.name).small().color(fg_muted));
                });
            });
        });

        if self.layout.command_palette_open {
            egui::Window::new("Command Palette")
                .collapsible(false)
                .resizable(false)
                .title_bar(false)
                .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, 44.0))
                .fixed_size(egui::vec2(440.0, 320.0))
                .show(&ctx, |ui| {
                    ui.horizontal(|ui| {
                        let sc_rect = ui.allocate_space(egui::vec2(16.0, 16.0)).1;
                        Codicon::Search.paint(ui, sc_rect, 14.0, accent_lapce);
                        let text_resp = ui.add(
                            egui::TextEdit::singleline(&mut self.palette_filter)
                                .hint_text("Type a command or file name...")
                                .desired_width(f32::INFINITY),
                        );
                        if text_resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                            self.layout.command_palette_open = false;
                        }
                    });

                    ui.separator();

                    let commands = [
                        ("View: Toggle Primary Sidebar", "Ctrl+B"),
                        ("View: Toggle Integrated Terminal", "Ctrl+J"),
                        ("File: Save Current File", "Ctrl+S"),
                        ("File: Create New File", "Ctrl+N"),
                        ("Run: Execute Active Script", "F5"),
                        ("Preferences: Open Settings", "Ctrl+,"),
                    ];

                    let filter_lower = self.palette_filter.to_lowercase();
                    egui::ScrollArea::vertical().max_height(230.0).show(ui, |ui| {
                        for (cmd, shortcut) in commands {
                            if filter_lower.is_empty() || cmd.to_lowercase().contains(&filter_lower) {
                                ui.horizontal(|ui| {
                                    if ui.selectable_label(false, cmd).clicked() {
                                        self.handle_palette_command(cmd);
                                        self.layout.command_palette_open = false;
                                    }
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        ui.label(egui::RichText::new(shortcut).small().color(fg_muted));
                                    });
                                });
                            }
                        }
                    });

                    if ui.button("Close (Esc)").clicked() {
                        self.layout.command_palette_open = false;
                    }
                });
        }

        if self.layout.settings_open {
            egui::Window::new("Lapce Settings")
                .collapsible(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .show(&ctx, |ui| {
                    ui.heading("Touch & Keyboard");
                    ui.checkbox(&mut self.layout.accessory_bar_visible, "Show Virtual Touch Bar");
                    ui.label(egui::RichText::new("(Disable when using MatePad magnetic keyboard)").small().color(fg_muted));
                    ui.add_space(10.0);

                    ui.heading("Editor Theme");
                    for (i, tname) in self.theme_names.iter().enumerate() {
                        if ui.selectable_label(self.current_theme_idx == i, tname).clicked() {
                            self.current_theme_idx = i;
                            self.theme = match i {
                                1 => Theme::sublime_monokai(),
                                2 => Theme::artcraft_zinc(),
                                _ => Theme::lapce_dark(),
                            };
                        }
                    }
                    ui.add_space(6.0);
                    if ui.button("Load Custom theme.toml").clicked() {
                        self.load_custom_theme();
                    }

                    ui.add_space(12.0);
                    if ui.button("Done").clicked() {
                        self.layout.settings_open = false;
                    }
                });
        }

        if self.save_dialog_open {
            egui::Window::new("New File")
                .collapsible(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .show(&ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Filename:");
                        ui.text_edit_singleline(&mut self.new_filename);
                    });
                    ui.horizontal(|ui| {
                        if ui.button("Create").clicked() {
                            let fname = self.new_filename.trim().to_string();
                            if !fname.is_empty() {
                                let base_dir = ensure_workspace_dir();
                                let target_path = base_dir.join(&fname);
                                let _ = std::fs::write(&target_path, "");
                                self.workspace.open_file(fname.clone(), Some(target_path.to_string_lossy().to_string()), "");
                                self.workspace.refresh_file_tree();
                                self.save_dialog_open = false;
                            }
                        }
                        if ui.button("Cancel").clicked() {
                            self.save_dialog_open = false;
                        }
                    });
                });
        }
    }
}

impl CodeCraftApp {
    fn handle_palette_command(&mut self, cmd: &str) {
        if cmd.contains("Toggle Primary Sidebar") {
            self.layout.toggle_sidebar();
        } else if cmd.contains("Toggle Integrated Terminal") {
            self.layout.toggle_terminal();
        } else if cmd.contains("Save Current File") {
            self.save_current_file();
        } else if cmd.contains("Create New File") {
            self.save_dialog_open = true;
        } else if cmd.contains("Execute Active Script") {
            self.run_active_file();
        } else if cmd.contains("Open Settings") {
            self.layout.toggle_settings();
        }
    }

    fn render_tree_nodes(
        ui: &mut egui::Ui,
        nodes: &[FileTreeNode],
        path_to_open: &mut Option<PathBuf>,
        path_to_toggle: &mut Option<PathBuf>,
        fg_primary: egui::Color32,
        fg_muted: egui::Color32,
        accent: egui::Color32,
    ) {
        for node in nodes {
            if node.is_dir {
                ui.horizontal(|ui| {
                    let d_rect = ui.allocate_space(egui::vec2(12.0, 14.0)).1;
                    if node.is_expanded {
                        Codicon::ChevronDown.paint(ui, d_rect, 10.0, fg_muted);
                    } else {
                        Codicon::ChevronRight.paint(ui, d_rect, 10.0, fg_muted);
                    }

                    let f_rect = ui.allocate_space(egui::vec2(14.0, 14.0)).1;
                    if node.is_expanded {
                        Codicon::FolderOpened.paint(ui, f_rect, 13.0, accent);
                    } else {
                        Codicon::Folder.paint(ui, f_rect, 13.0, fg_muted);
                    }

                    if ui.selectable_label(false, egui::RichText::new(&node.name).size(12.0).color(fg_primary)).clicked() {
                        *path_to_toggle = Some(node.path.clone());
                    }
                });

                if node.is_expanded {
                    ui.indent(&node.name, |ui| {
                        Self::render_tree_nodes(ui, &node.children, path_to_open, path_to_toggle, fg_primary, fg_muted, accent);
                    });
                }
            } else {
                ui.horizontal(|ui| {
                    ui.add_space(14.0);
                    let fc_rect = ui.allocate_space(egui::vec2(14.0, 14.0)).1;
                    Codicon::FileCode.paint(ui, fc_rect, 12.0, fg_muted);
                    if ui.selectable_label(false, egui::RichText::new(&node.name).size(12.0).color(fg_primary)).clicked() {
                        *path_to_open = Some(node.path.clone());
                    }
                });
            }
        }
    }

    fn open_file_from_path(&mut self, path: &Path) {
        if let Ok(content) = std::fs::read_to_string(path) {
            let filename = path.file_name().unwrap_or_default().to_string_lossy().to_string();
            self.workspace.open_file(filename.clone(), Some(path.to_string_lossy().to_string()), &content);
            self.status_msg = format!("Opened: {}", filename);
        } else {
            self.status_msg = "Error reading file".to_string();
        }
    }

    fn save_current_file(&mut self) {
        if let Some(tab) = self.workspace.active_tab_mut() {
            let path_opt = tab.path.clone().map(PathBuf::from);
            let content = tab.buffer.to_string();
            let target = path_opt.unwrap_or_else(|| ensure_workspace_dir().join(&tab.title));

            if std::fs::write(&target, &content).is_ok() {
                tab.is_dirty = false;
                tab.path = Some(target.to_string_lossy().to_string());
                self.status_msg = format!("Saved: {}", tab.title);
                self.workspace.refresh_file_tree();
            } else {
                self.status_msg = "Error saving file!".to_string();
            }
        }
    }

    fn insert_symbol(&mut self, sym: &str) {
        if let Some(tab) = self.workspace.active_tab_mut() {
            let mut content = tab.buffer.to_string();
            content.push_str(if sym == "Tab" { "    " } else { sym });
            tab.buffer = codecraft_core::buffer::DocumentBuffer::from_str(&content);
            tab.is_dirty = true;
        }
    }

    fn load_custom_theme(&mut self) {
        let theme_file = ensure_workspace_dir().join("Themes").join("theme.toml");
        if let Ok(toml_content) = std::fs::read_to_string(&theme_file) {
            match Theme::from_toml(&toml_content) {
                Ok(new_theme) => {
                    self.theme = new_theme;
                    self.status_msg = "Custom theme loaded!".to_string();
                }
                Err(e) => {
                    self.status_msg = format!("Error parsing theme: {}", e);
                }
            }
        } else {
            let sample = Theme::lapce_dark();
            if let Ok(serialized) = sample.to_toml() {
                let _ = std::fs::write(&theme_file, serialized);
                self.status_msg = "Created sample theme in Themes/theme.toml".to_string();
            }
        }
    }

    fn run_active_file(&mut self) {
        self.layout.terminal_visible = true;
        self.layout.active_dock_tab = BottomDockTab::Terminal;
        let (tab_title, tab_path_opt) = match self.workspace.active_tab() {
            Some(t) => (t.title.clone(), t.path.clone()),
            None => {
                self.terminal_capture.emit_system("Error: No active file to run.\n");
                return;
            }
        };

        let file_path = match tab_path_opt {
            Some(p) => PathBuf::from(p),
            None => {
                self.save_current_file();
                ensure_workspace_dir().join(&tab_title)
            }
        };

        let runner_opt = self.runner_manager.get_runner_for_file(&file_path).cloned();
        let capture = self.terminal_capture.sender();

        if let Some(runner) = runner_opt {
            let name = runner.name.clone();
            let run_cmd = runner.run.clone();
            let compile_cmd = runner.compile.clone();

            self.terminal_capture.emit_system(format!("\n▶ Running [{}] via {}\n", tab_title, name));

            std::thread::spawn(move || {
                let build_dir = PathBuf::from(WORKSPACE_DIR).join(".build");
                let _ = std::fs::create_dir_all(&build_dir);

                if let Some(c_cmd) = compile_cmd {
                    let full_compile = RunnerManager::resolve_command(&c_cmd, &file_path, &build_dir, None);
                    let _ = capture.send(codecraft_toolchain::runner::ExecutionStreamChunk {
                        stream_type: StreamType::SystemNotice,
                        content: format!("Compiling: {}\n", full_compile),
                    });
                }

                let full_run = RunnerManager::resolve_command(&run_cmd, &file_path, &build_dir, None);
                let _ = capture.send(codecraft_toolchain::runner::ExecutionStreamChunk {
                    stream_type: StreamType::SystemNotice,
                    content: format!("Executing: {}\n", full_run),
                });

                let output = std::process::Command::new("sh")
                    .arg("-c")
                    .arg(&full_run)
                    .output();

                match output {
                    Ok(out) => {
                        let stdout = String::from_utf8_lossy(&out.stdout);
                        let stderr = String::from_utf8_lossy(&out.stderr);
                        if !stdout.is_empty() {
                            let _ = capture.send(codecraft_toolchain::runner::ExecutionStreamChunk {
                                stream_type: StreamType::Stdout,
                                content: stdout.to_string(),
                            });
                        }
                        if !stderr.is_empty() {
                            let _ = capture.send(codecraft_toolchain::runner::ExecutionStreamChunk {
                                stream_type: StreamType::Stderr,
                                content: stderr.to_string(),
                            });
                        }
                        let _ = capture.send(codecraft_toolchain::runner::ExecutionStreamChunk {
                            stream_type: StreamType::SystemNotice,
                            content: format!("Exit code: {:?}\n", out.status.code()),
                        });
                    }
                    Err(e) => {
                        let _ = capture.send(codecraft_toolchain::runner::ExecutionStreamChunk {
                            stream_type: StreamType::Stderr,
                            content: format!("Failed to execute command: {}\n", e),
                        });
                    }
                }
            });
        } else {
            self.terminal_capture.emit_system(format!("No runner configured for file: {}\n", tab_title));
        }
    }

    fn execute_terminal_command(&mut self, cmd: &str) {
        if cmd == "clear" {
            self.terminal_logs.clear();
            return;
        }

        let capture = self.terminal_capture.sender();
        let cmd_owned = cmd.to_string();

        self.terminal_capture.emit_system(format!("$ {}\n", cmd));

        std::thread::spawn(move || {
            let output = std::process::Command::new("sh")
                .arg("-c")
                .arg(&cmd_owned)
                .current_dir(WORKSPACE_DIR)
                .output();

            match output {
                Ok(out) => {
                    let stdout = String::from_utf8_lossy(&out.stdout);
                    let stderr = String::from_utf8_lossy(&out.stderr);
                    if !stdout.is_empty() {
                        let _ = capture.send(codecraft_toolchain::runner::ExecutionStreamChunk {
                            stream_type: StreamType::Stdout,
                            content: stdout.to_string(),
                        });
                    }
                    if !stderr.is_empty() {
                        let _ = capture.send(codecraft_toolchain::runner::ExecutionStreamChunk {
                            stream_type: StreamType::Stderr,
                            content: stderr.to_string(),
                        });
                    }
                }
                Err(e) => {
                    let _ = capture.send(codecraft_toolchain::runner::ExecutionStreamChunk {
                        stream_type: StreamType::Stderr,
                        content: format!("Failed to run: {}\n", e),
                    });
                }
            }
        });
    }
}

#[unsafe(no_mangle)]
fn android_main(app: AndroidApp) {
    android_logger::init_once(
        android_logger::Config::default()
            .with_max_level(log::LevelFilter::Info)
            .with_tag("CodeCraft"),
    );

    apply_immersive_sticky(&app);
    let _ = ensure_workspace_dir();

    let mut options = eframe::NativeOptions::default();
    options.android_app = Some(app);
    options.renderer = eframe::Renderer::Wgpu;

    let _ = eframe::run_native(
        "CodeCraft",
        options,
        Box::new(|_cc| Ok(Box::new(CodeCraftApp::default()))),
    );
}
