use android_activity::AndroidApp;
use jni::objects::JValue;
use std::path::{Path, PathBuf};

use codecraft_core::workspace::{FileTreeNode, WorkspaceManager};
use codecraft_toolchain::pty::TerminalStreamCapture;
use codecraft_toolchain::runner::{RunnerManager, StreamType};
use codecraft_ui::layout::LayoutState;
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

struct CodeCraftApp {
    workspace: WorkspaceManager,
    layout: LayoutState,
    theme: Theme,
    theme_names: Vec<String>,
    current_theme_idx: usize,
    status_msg: String,
    save_dialog_open: bool,
    open_dialog_open: bool,
    new_filename: String,
    // Terminal & Toolchain Runner
    terminal_capture: TerminalStreamCapture,
    terminal_logs: Vec<(StreamType, String)>,
    terminal_input: String,
    runner_manager: RunnerManager,
    is_running: bool,
}

impl Default for CodeCraftApp {
    fn default() -> Self {
        let root = ensure_workspace_dir();
        let mut workspace = WorkspaceManager::new(&root);

        // Template awal berkas main.rs
        let default_code = r#"// CodeCraft Mobile IDE (Lapce UI + Sublime Speed)
// Touch-optimized for Huawei MatePad 12X (ARM64 Native)

fn main() {
    println!("Hello from CodeCraft Mobile IDE!");
    let nums = vec![1, 2, 3, 4, 5];
    let sum: i32 = nums.iter().sum();
    println!("Sum calculated: {}", sum);
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
        terminal_capture.emit_system("=== CodeCraft Integrated Terminal ===\nType 'help' or click 'Run' to execute active file.\n");

        Self {
            workspace,
            layout: LayoutState::default(),
            theme: Theme::sublime_monokai(),
            theme_names: vec![
                "Sublime Monokai Pro".to_string(),
                "Lapce Dark Modern".to_string(),
                "ArtCraft Zinc (Emerald)".to_string(),
            ],
            current_theme_idx: 0,
            status_msg: "Siap (Ready)".to_string(),
            save_dialog_open: false,
            open_dialog_open: false,
            new_filename: "script.rs".to_string(),
            terminal_capture,
            terminal_logs: Vec::new(),
            terminal_input: String::new(),
            runner_manager: RunnerManager::new(),
            is_running: false,
        }
    }
}

impl eframe::App for CodeCraftApp {
    fn raw_input_hook(&mut self, ctx: &egui::Context, _raw: &mut egui::RawInput) {
        if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::S)) {
            self.save_current_file();
        }
        if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::O)) {
            self.open_dialog_open = true;
        }
        if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::B)) {
            self.layout.toggle_sidebar();
        }
        if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::J)) {
            self.layout.toggle_terminal();
        }
    }

    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Poll terminal output chunks
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
        let _bg_tabs = to_color32(self.theme.bg_tabs);
        let bg_terminal = to_color32(self.theme.bg_terminal);
        let accent_em = to_color32(self.theme.accent_emerald);
        let border_subtle = to_color32(self.theme.border_subtle);

        // Apply background styling to full frame
        let full_rect = ui.max_rect();
        ui.painter().rect_filled(full_rect, 0.0, bg_app);

        // TOP ACTIVITY BAR (Lapce / Sublime Header)
        ui.horizontal(|ui| {
            ui.add_space(8.0);
            ui.label(egui::RichText::new("⚡ CodeCraft").strong().size(16.0).color(accent_em));
            ui.separator();

            if ui.button(if self.layout.sidebar_visible { "📂 Explorer [ON]" } else { "📁 Explorer" }).clicked() {
                self.layout.toggle_sidebar();
            }

            if ui.button("💾 Simpan").clicked() {
                self.save_current_file();
            }

            if ui.button("➕ File Baru").clicked() {
                self.save_dialog_open = true;
            }

            if ui.button(if self.layout.terminal_visible { "💻 Terminal [ON]" } else { "💻 Terminal" }).clicked() {
                self.layout.toggle_terminal();
            }

            if ui.button(egui::RichText::new("▶ Run").color(accent_em).strong()).clicked() {
                self.run_active_file();
            }

            if ui.button("⚙ Settings").clicked() {
                self.layout.toggle_settings();
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add_space(8.0);
                ui.label(egui::RichText::new(&self.status_msg).small().color(egui::Color32::LIGHT_GRAY));
            });
        });

        ui.painter().line_segment(
            [egui::pos2(full_rect.left(), ui.cursor().top()), egui::pos2(full_rect.right(), ui.cursor().top())],
            egui::Stroke::new(1.0, border_subtle),
        );
        ui.add_space(2.0);

        // MULTI-TAB BAR (Sublime / Lapce Editor Tabs)
        ui.horizontal(|ui| {
            ui.add_space(6.0);
            let tab_count = self.workspace.tabs().len();
            let mut tab_to_select = None;
            let mut tab_to_close = None;

            for (idx, tab) in self.workspace.tabs().iter().enumerate() {
                let is_active = idx == self.workspace.active_tab_idx();
                let title_text = if tab.is_dirty {
                    format!("● {}", tab.title)
                } else {
                    tab.title.clone()
                };

                let tab_resp = ui.selectable_label(is_active, title_text);
                if tab_resp.clicked() {
                    tab_to_select = Some(idx);
                }

                // Close button per tab
                if tab_count > 1 {
                    if ui.small_button("×").clicked() {
                        tab_to_close = Some(idx);
                    }
                }
                ui.add_space(4.0);
            }

            if let Some(idx) = tab_to_select {
                self.workspace.select_tab(idx);
            }
            if let Some(idx) = tab_to_close {
                self.workspace.close_tab(idx);
            }
        });

        ui.painter().line_segment(
            [egui::pos2(full_rect.left(), ui.cursor().top()), egui::pos2(full_rect.right(), ui.cursor().top())],
            egui::Stroke::new(1.0, border_subtle),
        );

        // MAIN WORKSPACE LAYOUT: SIDEBAR + EDITOR + TERMINAL
        let term_height = if self.layout.terminal_visible { self.layout.terminal_height } else { 0.0 };
        let accessory_height = if self.layout.accessory_bar_visible { 42.0 } else { 0.0 };
        let body_height = (full_rect.height() - ui.cursor().top() - term_height - accessory_height - 30.0).max(100.0);

        ui.horizontal(|ui| {
            // SIDEBAR (EXPLORER TREEVIEW)
            if self.layout.sidebar_visible {
                ui.allocate_ui_with_layout(
                    egui::vec2(self.layout.sidebar_width, body_height),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        ui.painter().rect_filled(ui.max_rect(), 0.0, bg_sidebar);
                        ui.add_space(6.0);
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("PROJECT EXPLORER").strong().small());
                            if ui.button("🔄").clicked() {
                                self.workspace.refresh_file_tree();
                            }
                        });
                        ui.label(egui::RichText::new(WORKSPACE_DIR).monospace().size(10.0).color(egui::Color32::GRAY));
                        ui.separator();

                        egui::ScrollArea::vertical().id_salt("sidebar_scroll").show(ui, |ui| {
                            let mut path_to_open = None;
                            let mut path_to_toggle = None;

                            Self::render_tree_nodes(
                                ui,
                                self.workspace.file_tree(),
                                &mut path_to_open,
                                &mut path_to_toggle,
                            );

                            if let Some(toggle_p) = path_to_toggle {
                                self.workspace.toggle_expand(&toggle_p);
                            }
                            if let Some(open_p) = path_to_open {
                                self.open_file_from_path(&open_p);
                            }
                        });
                    },
                );

                ui.painter().line_segment(
                    [egui::pos2(ui.cursor().left(), ui.cursor().top()), egui::pos2(ui.cursor().left(), ui.cursor().top() + body_height)],
                    egui::Stroke::new(1.0, border_subtle),
                );
            }

            // EDITOR PANE (GUTTER + TEXT AREA)
            ui.allocate_ui_with_layout(
                egui::vec2(ui.available_width(), body_height),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    ui.painter().rect_filled(ui.max_rect(), 0.0, bg_editor);

                    let mut file_content = if let Some(tab) = self.workspace.active_tab() {
                        tab.buffer.to_string()
                    } else {
                        String::new()
                    };

                    egui::ScrollArea::both()
                        .id_salt("editor_scroll")
                        .auto_shrink([false, false])
                        .scroll_source(egui::scroll_area::ScrollSource {
                            mouse_wheel: true,
                            drag: egui::scroll_area::DragScroll::Always,
                            scroll_bar: true,
                        })
                        .show(ui, |ui| {
                            let text_edit = egui::TextEdit::multiline(&mut file_content)
                                .font(egui::TextStyle::Monospace)
                                .code_editor()
                                .desired_width(f32::INFINITY)
                                .desired_rows(35);

                            let resp = ui.add(text_edit);
                            if resp.changed() {
                                if let Some(tab) = self.workspace.active_tab_mut() {
                                    tab.buffer = codecraft_core::buffer::DocumentBuffer::from_str(&file_content);
                                    tab.is_dirty = true;
                                    self.status_msg = "Perubahan belum disimpan".to_string();
                                }
                            }
                        });
                },
            );
        });

        // VIRTUAL ACCESSORY TOUCH BAR (Opsional / Toggleable)
        if self.layout.accessory_bar_visible {
            ui.painter().line_segment(
                [egui::pos2(full_rect.left(), ui.cursor().top()), egui::pos2(full_rect.right(), ui.cursor().top())],
                egui::Stroke::new(1.0, border_subtle),
            );
            ui.horizontal(|ui| {
                ui.add_space(4.0);
                ui.label(egui::RichText::new("⌨ Quick Touch:").small().color(egui::Color32::GRAY));
                let symbols = ["Tab", "{", "}", "(", ")", "[", "]", ";", ":", "\"", "'", "->", "=>", "=", "<", ">"];
                for sym in symbols {
                    if ui.button(sym).clicked() {
                        self.insert_symbol(sym);
                    }
                }
            });
        }

        // TERMINAL / RUNNER SPLIT PANEL
        if self.layout.terminal_visible {
            ui.painter().line_segment(
                [egui::pos2(full_rect.left(), ui.cursor().top()), egui::pos2(full_rect.right(), ui.cursor().top())],
                egui::Stroke::new(1.0, border_subtle),
            );

            ui.allocate_ui_with_layout(
                egui::vec2(full_rect.width(), self.layout.terminal_height),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    ui.painter().rect_filled(ui.max_rect(), 0.0, bg_terminal);
                    ui.horizontal(|ui| {
                        ui.add_space(8.0);
                        ui.label(egui::RichText::new("TERMINAL / RUNNER OUTPUT").strong().small());
                        if ui.button("Clear Log").clicked() {
                            self.terminal_logs.clear();
                        }
                        if ui.button("Close Dock").clicked() {
                            self.layout.terminal_visible = false;
                        }
                    });

                    egui::ScrollArea::vertical()
                        .id_salt("terminal_scroll")
                        .max_height(self.layout.terminal_height - 65.0)
                        .stick_to_bottom(true)
                        .show(ui, |ui| {
                            for (stype, text) in &self.terminal_logs {
                                let color = match stype {
                                    StreamType::Stdout => egui::Color32::from_rgb(220, 220, 230),
                                    StreamType::Stderr => egui::Color32::from_rgb(244, 63, 94),
                                    StreamType::SystemNotice => accent_em,
                                };
                                ui.label(egui::RichText::new(text).monospace().small().color(color));
                            }
                        });

                    // Terminal input bar
                    ui.horizontal(|ui| {
                        ui.add_space(6.0);
                        ui.label(egui::RichText::new("$").monospace().strong().color(accent_em));
                        let resp = ui.text_edit_singleline(&mut self.terminal_input);
                        if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                            let cmd = self.terminal_input.trim().to_string();
                            if !cmd.is_empty() {
                                self.execute_terminal_command(&cmd);
                                self.terminal_input.clear();
                            }
                        }
                        if ui.button("Execute").clicked() {
                            let cmd = self.terminal_input.trim().to_string();
                            if !cmd.is_empty() {
                                self.execute_terminal_command(&cmd);
                                self.terminal_input.clear();
                            }
                        }
                    });
                },
            );
        }

        // FOOTER STATUS BAR
        ui.painter().line_segment(
            [egui::pos2(full_rect.left(), ui.cursor().top()), egui::pos2(full_rect.right(), ui.cursor().top())],
            egui::Stroke::new(1.0, border_subtle),
        );
        ui.horizontal(|ui| {
            ui.add_space(8.0);
            let cur_file = self.workspace.active_tab().map(|t| t.title.as_str()).unwrap_or("No file");
            ui.label(format!("File: {}", cur_file));
            ui.separator();
            ui.label(format!("Theme: {}", self.theme.name));
            ui.separator();
            ui.label("UTF-8");
            ui.separator();
            ui.label("ARM64 Native");
            ui.separator();
            ui.label("144Hz Smooth");
        });

        // SETTINGS MODAL
        let ctx = ui.ctx().clone();
        if self.layout.settings_open {
            egui::Window::new("⚙ Pengaturan CodeCraft")
                .collapsible(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .show(&ctx, |ui| {
                    ui.heading("Tampilan & Input");
                    ui.checkbox(&mut self.layout.accessory_bar_visible, "Tampilkan Virtual Touch Accessory Bar");
                    ui.label(egui::RichText::new("(Matikan jika kamu memakai keyboard fisik MatePad 12X agar layar lebih luas)").small().color(egui::Color32::GRAY));
                    ui.add_space(10.0);

                    ui.heading("Tema Editor & Sintaks");
                    for (i, tname) in self.theme_names.iter().enumerate() {
                        if ui.selectable_label(self.current_theme_idx == i, tname).clicked() {
                            self.current_theme_idx = i;
                            self.theme = match i {
                                1 => Theme::lapce_dark(),
                                2 => Theme::artcraft_zinc(),
                                _ => Theme::sublime_monokai(),
                            };
                        }
                    }
                    ui.add_space(6.0);
                    ui.label("Custom Theme: Simpan 'theme.toml' di /sdcard/Download/ArtCraft/Themes/ untuk memuat tema buatan sendiri.");
                    if ui.button("Muat theme.toml kustom").clicked() {
                        self.load_custom_theme();
                    }

                    ui.add_space(12.0);
                    if ui.button("Tutup").clicked() {
                        self.layout.settings_open = false;
                    }
                });
        }

        // CREATE NEW FILE MODAL
        if self.save_dialog_open {
            egui::Window::new("Buat Berkas Baru")
                .collapsible(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .show(&ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Nama Berkas:");
                        ui.text_edit_singleline(&mut self.new_filename);
                    });
                    ui.horizontal(|ui| {
                        if ui.button("Buat & Buka").clicked() {
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
                        if ui.button("Batal").clicked() {
                            self.save_dialog_open = false;
                        }
                    });
                });
        }
    }
}

impl CodeCraftApp {
    fn render_tree_nodes(
        ui: &mut egui::Ui,
        nodes: &[FileTreeNode],
        path_to_open: &mut Option<PathBuf>,
        path_to_toggle: &mut Option<PathBuf>,
    ) {
        for node in nodes {
            if node.is_dir {
                let icon = if node.is_expanded { "📂" } else { "📁" };
                let label = format!("{} {}", icon, node.name);
                if ui.selectable_label(false, label).clicked() {
                    *path_to_toggle = Some(node.path.clone());
                }
                if node.is_expanded {
                    ui.indent(&node.name, |ui| {
                        Self::render_tree_nodes(ui, &node.children, path_to_open, path_to_toggle);
                    });
                }
            } else {
                let ext = node.path.extension().and_then(|e| e.to_str()).unwrap_or("");
                let icon = match ext {
                    "rs" => "🦀",
                    "java" => "☕",
                    "py" => "🐍",
                    "c" | "cpp" | "h" => "⚙",
                    "json" | "toml" => "📋",
                    _ => "📄",
                };
                let label = format!("{} {}", icon, node.name);
                if ui.selectable_label(false, label).clicked() {
                    *path_to_open = Some(node.path.clone());
                }
            }
        }
    }

    fn open_file_from_path(&mut self, path: &Path) {
        if let Ok(content) = std::fs::read_to_string(path) {
            let filename = path.file_name().unwrap_or_default().to_string_lossy().to_string();
            self.workspace.open_file(filename.clone(), Some(path.to_string_lossy().to_string()), &content);
            self.status_msg = format!("Dibuka: {}", filename);
        } else {
            self.status_msg = "Gagal membaca berkas".to_string();
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
                self.status_msg = format!("Tersimpan: {}", tab.title);
                self.workspace.refresh_file_tree();
            } else {
                self.status_msg = "Gagal menyimpan berkas!".to_string();
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
                    self.status_msg = "Tema kustom theme.toml berhasil dimuat!".to_string();
                }
                Err(e) => {
                    self.status_msg = format!("Error parse theme.toml: {}", e);
                }
            }
        } else {
            // Write sample theme.toml
            let sample = Theme::sublime_monokai();
            if let Ok(serialized) = sample.to_toml() {
                let _ = std::fs::write(&theme_file, serialized);
                self.status_msg = "Contoh theme.toml dibuat di Themes/theme.toml".to_string();
            }
        }
    }

    fn run_active_file(&mut self) {
        self.layout.terminal_visible = true;
        let (tab_title, tab_path_opt) = match self.workspace.active_tab() {
            Some(t) => (t.title.clone(), t.path.clone()),
            None => {
                self.terminal_capture.emit_system("Error: Tidak ada berkas aktif untuk dijalankan.\n");
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

            self.terminal_capture.emit_system(format!("\n▶ Menjalankan [{}] via Runner: {}\n", tab_title, name));

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

                // Spawn shell command
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
                            content: format!("[Process completed with status: {:?}]\n", out.status.code()),
                        });
                    }
                    Err(e) => {
                        let _ = capture.send(codecraft_toolchain::runner::ExecutionStreamChunk {
                            stream_type: StreamType::Stderr,
                            content: format!("Eksekusi gagal: {}\n", e),
                        });
                    }
                }
            });
        } else {
            self.terminal_capture.emit_system(format!(
                "Pemberitahuan: Tidak ditemukan runner bawaan untuk ekstensi '{}'. Kamu dapat menambahkan runner di runner.toml.\n",
                file_path.extension().and_then(|e| e.to_str()).unwrap_or("")
            ));
        }
    }

    fn execute_terminal_command(&mut self, cmd: &str) {
        if cmd == "help" {
            self.terminal_capture.emit_system("Perintah tersedia: help, clear, ls, pwd, cat <file>, sh <command>\n");
            return;
        }
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
                        content: format!("Gagal menjalankan: {}\n", e),
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
