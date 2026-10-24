use android_activity::AndroidApp;
use jni::objects::JValue;
use std::path::PathBuf;

const WORKSPACE_DIR: &str = "/sdcard/Download/ArtCraft";

fn ensure_workspace_dir() -> PathBuf {
    let p = PathBuf::from(WORKSPACE_DIR);
    let _ = std::fs::create_dir_all(&p);
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

struct CodeCraftApp {
    editor_text: String,
    current_file: String,
    files: Vec<String>,
    layout: codecraft_ui::layout::LayoutState,
    status_msg: String,
    save_dialog_open: bool,
    open_dialog_open: bool,
    new_filename: String,
}

impl Default for CodeCraftApp {
    fn default() -> Self {
        let base_dir = ensure_workspace_dir();
        let mut files = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&base_dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_file() {
                    let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("").to_ascii_lowercase();
                    if ["rs", "toml", "py", "js", "ts", "json", "md", "txt", "c", "cpp", "h"].contains(&ext.as_str()) {
                        files.push(p.file_name().unwrap().to_string_lossy().to_string());
                    }
                }
            }
        }
        if files.is_empty() {
            files.push("main.rs".to_string());
        }

        Self {
            editor_text: "// CodeCraft Pure Rust Mobile Code Editor\n// Optimized for MatePad 12X (Sublime Engine + Lapce UI)\n\nfn main() {\n    println!(\"Hello from CodeCraft Mobile!\");\n}\n".to_string(),
            current_file: "main.rs".to_string(),
            files,
            layout: codecraft_ui::layout::LayoutState::default(),
            status_msg: "Siap (Ready)".to_string(),
            save_dialog_open: false,
            open_dialog_open: false,
            new_filename: "script.rs".to_string(),
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
    }

    fn logic(&mut self, _ctx: &egui::Context, _frame: &mut eframe::Frame) {}

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let full_rect = ui.max_rect();

        // Top Action Bar
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("CodeCraft").strong().color(egui::Color32::from_rgb(16, 185, 129)));
            ui.separator();
            if ui.button("📁 Explorer").clicked() {
                self.layout.toggle_sidebar();
            }
            if ui.button("💾 Simpan (Ctrl+S)").clicked() {
                self.save_current_file();
            }
            if ui.button("📂 Buka File").clicked() {
                self.open_dialog_open = true;
            }
            if ui.button("➕ File Baru").clicked() {
                self.save_dialog_open = true;
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(egui::RichText::new(&self.status_msg).small().color(egui::Color32::LIGHT_GRAY));
            });
        });
        ui.separator();

        // Main Body: Sidebar + Editor
        let body_height = full_rect.height() - 70.0;
        ui.horizontal(|ui| {
            if self.layout.sidebar_visible {
                ui.allocate_ui_with_layout(
                    egui::vec2(220.0, body_height),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        ui.heading("EXPLORER");
                        ui.label(egui::RichText::new(WORKSPACE_DIR).small().monospace());
                        ui.separator();

                        egui::ScrollArea::vertical().show(ui, |ui| {
                            let mut file_to_load = None;
                            for fname in &self.files {
                                let is_sel = *fname == self.current_file;
                                if ui.selectable_label(is_sel, format!("📄 {}", fname)).clicked() {
                                    file_to_load = Some(fname.clone());
                                }
                            }
                            if let Some(f) = file_to_load {
                                self.load_file(&f);
                            }
                        });
                    },
                );
                ui.separator();
            }

            // Editor Canvas with Touch Scrolling
            ui.allocate_ui_with_layout(
                egui::vec2(ui.available_width(), body_height),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    ui.label(egui::RichText::new(&self.current_file).strong());
                    ui.separator();

                    egui::ScrollArea::both()
                        .auto_shrink([false, false])
                        .scroll_source(egui::scroll_area::ScrollSource {
                            mouse_wheel: true,
                            drag: egui::scroll_area::DragScroll::Always,
                            scroll_bar: true,
                        })
                        .show(ui, |ui| {
                            let text_edit = egui::TextEdit::multiline(&mut self.editor_text)
                                .font(egui::TextStyle::Monospace)
                                .code_editor()
                                .desired_width(f32::INFINITY)
                                .desired_rows(32);
                            let resp = ui.add(text_edit);
                            if resp.changed() {
                                self.status_msg = "Ada perubahan belum tersimpan".to_string();
                            }
                        });
                },
            );
        });

        // Bottom Status Bar
        ui.separator();
        ui.horizontal(|ui| {
            ui.label(format!("File: {}", self.current_file));
            ui.separator();
            ui.label("UTF-8");
            ui.separator();
            ui.label("Rust (ARM64 Native)");
            ui.separator();
            ui.label("120Hz Touch Ready");
        });

        // Dialog: Open File
        let ctx = ui.ctx().clone();
        if self.open_dialog_open {
            egui::Window::new("Buka Berkas CodeCraft")
                .collapsible(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .show(&ctx, |ui| {
                    ui.label("Pilih berkas dari /sdcard/Download/ArtCraft:");
                    egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                        for fname in self.files.clone() {
                            if ui.button(&fname).clicked() {
                                self.load_file(&fname);
                                self.open_dialog_open = false;
                            }
                        }
                    });
                    if ui.button("Tutup").clicked() {
                        self.open_dialog_open = false;
                    }
                });
        }

        // Dialog: New / Save File
        if self.save_dialog_open {
            egui::Window::new("Buat File Baru")
                .collapsible(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .show(&ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Nama File:");
                        ui.text_edit_singleline(&mut self.new_filename);
                    });
                    ui.horizontal(|ui| {
                        if ui.button("Buat & Buka").clicked() {
                            let fname = self.new_filename.trim().to_string();
                            if !fname.is_empty() {
                                self.current_file = fname.clone();
                                if !self.files.contains(&fname) {
                                    self.files.push(fname);
                                }
                                self.editor_text = String::new();
                                self.save_current_file();
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
    fn save_current_file(&mut self) {
        let base_dir = ensure_workspace_dir();
        let target = base_dir.join(&self.current_file);
        if let Ok(_) = std::fs::write(&target, &self.editor_text) {
            self.status_msg = format!("Tersimpan: {}", self.current_file);
            if !self.files.contains(&self.current_file) {
                self.files.push(self.current_file.clone());
            }
        } else {
            self.status_msg = "Gagal menyimpan berkas!".to_string();
        }
    }

    fn load_file(&mut self, filename: &str) {
        let base_dir = ensure_workspace_dir();
        let target = base_dir.join(filename);
        if let Ok(content) = std::fs::read_to_string(&target) {
            self.editor_text = content;
            self.current_file = filename.to_string();
            self.status_msg = format!("Dibuka: {}", filename);
        } else {
            self.status_msg = format!("Gagal membuka {}", filename);
        }
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
