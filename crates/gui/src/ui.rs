use eframe::egui::{self, Color32, RichText, Rounding, Stroke, Vec2};
use std::ffi::c_void;
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use tracing::{error, info};
use windhawk_core::exporter::export_notes_to_file;
use windhawk_core::model::Note;
use windhawk_core::paths::{default_export_dir, default_export_path};
use windhawk_core::tweaks::{get_available_tweaks, read_tweak_status, toggle_tweak, Win10Tweak};
use windhawk_core::OsVersion;
use windows_sys::Win32::Foundation::HWND;
use windows_sys::Win32::UI::Shell::ShellExecuteW;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    MessageBoxW, MB_ICONERROR, MB_ICONINFORMATION, MB_OK, SW_SHOWNORMAL,
};

// OpenFileName Flags
const OFN_OVERWRITEPROMPT_FLAG: u32 = 0x00000002;
const OFN_PATHMUSTEXIST_FLAG: u32 = 0x00000800;
const OFN_NOCHANGEDIR_FLAG: u32 = 0x00000008;

pub fn to_wide(s: &str) -> Vec<u16> {
    OsStr::new(s)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

pub fn show_error_dialog(title: &str, message: &str) {
    let title_w = to_wide(title);
    let msg_w = to_wide(message);
    unsafe {
        MessageBoxW(
            std::ptr::null_mut(),
            msg_w.as_ptr(),
            title_w.as_ptr(),
            MB_ICONERROR | MB_OK,
        );
    }
}

#[allow(dead_code)]
pub fn show_info_dialog(title: &str, message: &str) {
    let title_w = to_wide(title);
    let msg_w = to_wide(message);
    unsafe {
        MessageBoxW(
            std::ptr::null_mut(),
            msg_w.as_ptr(),
            title_w.as_ptr(),
            MB_ICONINFORMATION | MB_OK,
        );
    }
}

pub fn prompt_save_csv_path(default_path: &Path) -> Option<PathBuf> {
    let mut file_buf = [0u16; 1024];

    if let Some(file_name) = default_path.file_name().and_then(|f| f.to_str()) {
        let name_w = to_wide(file_name);
        let len = name_w.len().min(file_buf.len() - 1);
        file_buf[..len].copy_from_slice(&name_w[..len]);
    }

    let initial_dir_w: Vec<u16> = default_path
        .parent()
        .and_then(|p| p.to_str())
        .map(to_wide)
        .unwrap_or_else(|| to_wide("."));

    let filter_w = to_wide("CSV Files (*.csv)\0*.csv\0All Files (*.*)\0*.*\0");
    let title_w = to_wide("Export Notes to CSV");
    let def_ext_w = to_wide("csv");

    #[repr(C)]
    #[allow(non_snake_case, clippy::upper_case_acronyms)]
    struct OPENFILENAMEW {
        lStructSize: u32,
        hwndOwner: HWND,
        hInstance: *mut c_void,
        lpstrFilter: *const u16,
        lpstrCustomFilter: *mut u16,
        nMaxCustFilter: u32,
        nFilterIndex: u32,
        lpstrFile: *mut u16,
        nMaxFile: u32,
        lpstrFileTitle: *mut u16,
        nMaxFileTitle: u32,
        lpstrInitialDir: *const u16,
        lpstrTitle: *const u16,
        Flags: u32,
        nFileOffset: u16,
        nFileExtension: u16,
        lpstrDefExt: *const u16,
        lCustData: isize,
        lpfnHook: Option<unsafe extern "system" fn(HWND, u32, usize, isize) -> usize>,
        lpTemplateName: *const u16,
        pvReserved: *mut c_void,
        dwReserved: u32,
        FlagsEx: u32,
    }

    extern "system" {
        fn GetSaveFileNameW(lpofn: *mut OPENFILENAMEW) -> i32;
    }

    let mut ofn: OPENFILENAMEW = unsafe { std::mem::zeroed() };
    ofn.lStructSize = std::mem::size_of::<OPENFILENAMEW>() as u32;
    ofn.hwndOwner = std::ptr::null_mut();
    ofn.lpstrFilter = filter_w.as_ptr();
    ofn.lpstrFile = file_buf.as_mut_ptr();
    ofn.nMaxFile = file_buf.len() as u32;
    ofn.lpstrInitialDir = initial_dir_w.as_ptr();
    ofn.lpstrTitle = title_w.as_ptr();
    ofn.lpstrDefExt = def_ext_w.as_ptr();
    ofn.Flags = OFN_OVERWRITEPROMPT_FLAG | OFN_PATHMUSTEXIST_FLAG | OFN_NOCHANGEDIR_FLAG;

    let res = unsafe { GetSaveFileNameW(&mut ofn) };
    if res != 0 {
        let end = file_buf
            .iter()
            .position(|&c| c == 0)
            .unwrap_or(file_buf.len());
        let path_str = String::from_utf16_lossy(&file_buf[..end]);
        Some(PathBuf::from(path_str))
    } else {
        None
    }
}

pub fn open_folder_in_explorer(path: &Path) {
    let operation_w = to_wide("open");
    let path_w = to_wide(path.to_str().unwrap_or("."));
    unsafe {
        ShellExecuteW(
            std::ptr::null_mut(),
            operation_w.as_ptr(),
            path_w.as_ptr(),
            std::ptr::null(),
            std::ptr::null(),
            SW_SHOWNORMAL,
        );
    }
}

#[derive(PartialEq, Clone, Copy)]
pub enum NavCategory {
    All,
    Tweaks,
    Taskbar,
    Explorer,
    Themes,
    NewNote,
}

pub struct WindhawkApp {
    pub os_version: OsVersion,
    pub notes: Vec<Note>,
    pub tweaks: Vec<Win10Tweak>,
    pub selected_note_index: usize,
    pub active_category: NavCategory,
    pub search_query: String,
    pub toast_message: Option<(String, std::time::Instant)>,
    // For adding a new custom note:
    pub new_title: String,
    pub new_body: String,
    pub new_category: String,
}

impl WindhawkApp {
    pub fn new(os_version: OsVersion) -> Self {
        let mut notes = Note::sample_notes();
        let tweaks = get_available_tweaks();

        // Include current system tweaks as initial notes if not already present
        for tweak in &tweaks {
            let status = read_tweak_status(tweak);
            let tweak_note = tweak.to_note(status);
            if !notes.iter().any(|n| n.id == tweak_note.id) {
                notes.push(tweak_note);
            }
        }

        Self {
            os_version,
            notes,
            tweaks,
            selected_note_index: 0,
            active_category: NavCategory::All,
            search_query: String::new(),
            toast_message: Some((
                format!(
                    "Verified Compatible: Windows 10 Build {}",
                    os_version.build_number
                ),
                std::time::Instant::now(),
            )),
            new_title: String::new(),
            new_body: String::new(),
            new_category: "Custom".into(),
        }
    }

    pub fn set_toast(&mut self, msg: impl Into<String>) {
        self.toast_message = Some((msg.into(), std::time::Instant::now()));
    }

    fn filtered_note_indices(&self) -> Vec<usize> {
        let q = self.search_query.to_lowercase();
        self.notes
            .iter()
            .enumerate()
            .filter(|(_, note)| {
                // Category filter
                let cat_match = match self.active_category {
                    NavCategory::All | NavCategory::NewNote => true,
                    NavCategory::Tweaks => note.id >= 100,
                    NavCategory::Taskbar => {
                        note.title.to_lowercase().contains("taskbar")
                            || note.body.to_lowercase().contains("taskbar")
                    }
                    NavCategory::Explorer => {
                        note.title.to_lowercase().contains("explorer")
                            || note.body.to_lowercase().contains("explorer")
                    }
                    NavCategory::Themes => {
                        note.title.to_lowercase().contains("theme")
                            || note.title.to_lowercase().contains("dark")
                            || note.body.to_lowercase().contains("theme")
                    }
                };

                if !cat_match {
                    return false;
                }

                // Search query filter
                if q.is_empty() {
                    true
                } else {
                    note.title.to_lowercase().contains(&q) || note.body.to_lowercase().contains(&q)
                }
            })
            .map(|(idx, _)| idx)
            .collect()
    }
}

impl eframe::App for WindhawkApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Modern custom dark palette styling
        let mut visuals = egui::Visuals::dark();
        visuals.panel_fill = Color32::from_rgb(22, 24, 29); // #16181d
        visuals.window_fill = Color32::from_rgb(26, 29, 36); // #1a1d24
        visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(32, 35, 45);
        visuals.widgets.inactive.bg_fill = Color32::from_rgb(38, 42, 54);
        visuals.widgets.hovered.bg_fill = Color32::from_rgb(52, 58, 75);
        visuals.widgets.active.bg_fill = Color32::from_rgb(14, 116, 144); // Cyan 700
        visuals.widgets.noninteractive.rounding = Rounding::same(6.0_f32);
        visuals.widgets.inactive.rounding = Rounding::same(6.0_f32);
        visuals.widgets.hovered.rounding = Rounding::same(6.0_f32);
        visuals.widgets.active.rounding = Rounding::same(6.0_f32);
        ctx.set_visuals(visuals);

        // TOP HEADER BAR
        egui::TopBottomPanel::top("top_header")
            .frame(
                egui::Frame::none()
                    .fill(Color32::from_rgb(18, 20, 24))
                    .inner_margin(egui::Margin::symmetric(16.0, 12.0)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    // Windhawk Falcon branding
                    ui.label(
                        RichText::new("🦅 WINDHAWK")
                            .color(Color32::from_rgb(56, 189, 248)) // Sky 400
                            .strong()
                            .size(17.0),
                    );

                    ui.label(
                        RichText::new("— Windows 10 Notes & Tweaks Exporter")
                            .color(Color32::from_rgb(203, 213, 225)) // Slate 300
                            .size(14.0),
                    );

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Export Notes CSV button in prominent header position
                        let export_btn = egui::Button::new(
                            RichText::new("📁 Export notes (CSV)")
                                .color(Color32::WHITE)
                                .strong(),
                        )
                        .fill(Color32::from_rgb(14, 116, 144)) // Ocean cyan
                        .min_size(Vec2::new(140.0, 28.0));

                        if ui.add(export_btn).clicked() {
                            info!("'Export notes' button clicked in fancy GUI");
                            let def_path = default_export_path();
                            if let Some(target_path) = prompt_save_csv_path(&def_path) {
                                match export_notes_to_file(&self.notes, &target_path) {
                                    Ok(count) => {
                                        let msg = format!(
                                            "Successfully exported {} notes to {}",
                                            count,
                                            target_path.display()
                                        );
                                        self.set_toast(msg);
                                    }
                                    Err(e) => {
                                        error!("Export failed: {}", e);
                                        show_error_dialog("Export Failed", &e.to_string());
                                    }
                                }
                            }
                        }

                        // Windows 10 status pill
                        ui.scope(|ui| {
                            let pill_color = Color32::from_rgb(34, 197, 94); // Green 500
                            let pill_bg = Color32::from_rgba_premultiplied(34, 197, 94, 25);
                            egui::Frame::none()
                                .fill(pill_bg)
                                .stroke(Stroke::new(1.0_f32, pill_color))
                                .rounding(Rounding::same(12.0_f32))
                                .inner_margin(egui::Margin::symmetric(10.0, 4.0))
                                .show(ui, |ui| {
                                    ui.label(
                                        RichText::new(format!(
                                            "● Windows 10 Build {} (Compatible)",
                                            self.os_version.build_number
                                        ))
                                        .color(pill_color)
                                        .size(11.5)
                                        .strong(),
                                    );
                                });
                        });
                    });
                });
            });

        // BOTTOM STATUS BAR / TOAST
        egui::TopBottomPanel::bottom("bottom_bar")
            .frame(
                egui::Frame::none()
                    .fill(Color32::from_rgb(15, 17, 21))
                    .inner_margin(egui::Margin::symmetric(16.0, 8.0)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if let Some((ref msg, ref timestamp)) = self.toast_message {
                        if timestamp.elapsed().as_secs() < 8 {
                            ui.label(
                                RichText::new(format!("✔  {}", msg))
                                    .color(Color32::from_rgb(74, 222, 128))
                                    .size(12.0)
                                    .strong(),
                            );
                        } else {
                            self.toast_message = None;
                        }
                    } else {
                        ui.label(
                            RichText::new(format!(
                                "Total Notes: {} | Active Windows 10 Tweaks: {} | Dual-Tier Guard: Active",
                                self.notes.len(),
                                self.tweaks.len()
                            ))
                            .color(Color32::from_rgb(148, 163, 184))
                            .size(11.5),
                        );
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            RichText::new("RFC 4180 • UTF-8 BOM • Windows 10 Exclusive")
                                .color(Color32::from_rgb(100, 116, 139))
                                .size(11.0),
                        );
                    });
                });
            });

        // LEFT NAVIGATION SIDEBAR
        egui::SidePanel::left("left_sidebar")
            .resizable(false)
            .default_width(240.0)
            .frame(
                egui::Frame::none()
                    .fill(Color32::from_rgb(22, 25, 32))
                    .inner_margin(egui::Margin::symmetric(12.0, 12.0)),
            )
            .show(ctx, |ui| {
                // Search box
                ui.label(
                    RichText::new("SEARCH & FILTER")
                        .size(10.0)
                        .color(Color32::from_rgb(148, 163, 184))
                        .strong(),
                );
                ui.add_space(4.0);
                ui.add(
                    egui::TextEdit::singleline(&mut self.search_query)
                        .hint_text("🔍 Search notes and tweaks...")
                        .desired_width(f32::INFINITY),
                );
                ui.add_space(14.0);

                // Categories
                ui.label(
                    RichText::new("CATEGORIES")
                        .size(10.0)
                        .color(Color32::from_rgb(148, 163, 184))
                        .strong(),
                );
                ui.add_space(4.0);

                let cat_button = |ui: &mut egui::Ui,
                                  name: &str,
                                  cat: NavCategory,
                                  current: NavCategory|
                 -> bool {
                    let is_selected = cat == current;
                    let text = if is_selected {
                        RichText::new(name).color(Color32::WHITE).strong()
                    } else {
                        RichText::new(name).color(Color32::from_rgb(203, 213, 225))
                    };
                    let btn = egui::Button::new(text)
                        .fill(if is_selected {
                            Color32::from_rgb(37, 99, 235) // Royal blue
                        } else {
                            Color32::from_rgb(30, 34, 44)
                        })
                        .min_size(Vec2::new(f32::INFINITY, 30.0));
                    ui.add(btn).clicked()
                };

                if cat_button(ui, "📋  All Notes", NavCategory::All, self.active_category) {
                    self.active_category = NavCategory::All;
                }
                ui.add_space(2.0);
                if cat_button(
                    ui,
                    "⚙  Windows 10 Tweaks",
                    NavCategory::Tweaks,
                    self.active_category,
                ) {
                    self.active_category = NavCategory::Tweaks;
                }
                ui.add_space(2.0);
                if cat_button(
                    ui,
                    "🗔  Taskbar & Shell",
                    NavCategory::Taskbar,
                    self.active_category,
                ) {
                    self.active_category = NavCategory::Taskbar;
                }
                ui.add_space(2.0);
                if cat_button(
                    ui,
                    "📁  File Explorer",
                    NavCategory::Explorer,
                    self.active_category,
                ) {
                    self.active_category = NavCategory::Explorer;
                }
                ui.add_space(2.0);
                if cat_button(
                    ui,
                    "🎨  Visual Themes",
                    NavCategory::Themes,
                    self.active_category,
                ) {
                    self.active_category = NavCategory::Themes;
                }
                ui.add_space(2.0);
                if cat_button(
                    ui,
                    "➕  New Custom Note",
                    NavCategory::NewNote,
                    self.active_category,
                ) {
                    self.active_category = NavCategory::NewNote;
                }

                ui.add_space(20.0);
                ui.separator();
                ui.add_space(10.0);

                // Quick actions
                ui.label(
                    RichText::new("ACTIONS")
                        .size(10.0)
                        .color(Color32::from_rgb(148, 163, 184))
                        .strong(),
                );
                ui.add_space(6.0);

                if ui
                    .add(
                        egui::Button::new(RichText::new("📂  Open Export Folder").size(12.0))
                            .min_size(Vec2::new(f32::INFINITY, 28.0)),
                    )
                    .clicked()
                {
                    let dir = default_export_dir();
                    open_folder_in_explorer(&dir);
                    self.set_toast(format!("Opened folder: {}", dir.display()));
                }
                ui.add_space(4.0);

                if ui
                    .add(
                        egui::Button::new(RichText::new("📋  Copy CSV to Clipboard").size(12.0))
                            .min_size(Vec2::new(f32::INFINITY, 28.0)),
                    )
                    .clicked()
                {
                    match windhawk_core::exporter::export_notes_to_bytes(&self.notes) {
                        Ok(bytes) => {
                            if let Ok(csv_str) = std::str::from_utf8(&bytes[3..]) {
                                ui.ctx().copy_text(csv_str.to_string());
                                self.set_toast("Copied complete CSV to clipboard!");
                            }
                        }
                        Err(e) => {
                            self.set_toast(format!("Error copying CSV: {}", e));
                        }
                    }
                }
            });

        // CENTRAL CONTENT AREA
        egui::CentralPanel::default()
            .frame(
                egui::Frame::none()
                    .fill(Color32::from_rgb(26, 29, 36))
                    .inner_margin(egui::Margin::same(16.0)),
            )
            .show(ctx, |ui| {
                if self.active_category == NavCategory::NewNote {
                    self.render_new_note_view(ui);
                } else {
                    self.render_main_split_view(ui);
                }
            });
    }
}

impl WindhawkApp {
    fn render_new_note_view(&mut self, ui: &mut egui::Ui) {
        ui.heading(
            RichText::new("Create New Windows 10 Tweak or Note")
                .color(Color32::WHITE)
                .size(18.0),
        );
        ui.label(
            RichText::new("Add custom tweaks, registry paths, or Windhawk modding instructions.")
                .color(Color32::from_rgb(148, 163, 184)),
        );
        ui.add_space(14.0);

        ui.label(RichText::new("Note Title:").strong());
        ui.add(
            egui::TextEdit::singleline(&mut self.new_title)
                .hint_text("e.g. Restore Classic Volume Mixer on Taskbar")
                .desired_width(f32::INFINITY),
        );
        ui.add_space(8.0);

        ui.label(RichText::new("Category / Tag:").strong());
        ui.add(
            egui::TextEdit::singleline(&mut self.new_category)
                .hint_text("Taskbar, Explorer, Theme, Custom")
                .desired_width(f32::INFINITY),
        );
        ui.add_space(8.0);

        ui.label(RichText::new("Note Body & Instructions:").strong());
        ui.add(
            egui::TextEdit::multiline(&mut self.new_body)
                .hint_text("Describe registry tweak, hook details, or mod notes...")
                .desired_rows(8)
                .desired_width(f32::INFINITY),
        );
        ui.add_space(14.0);

        ui.horizontal(|ui| {
            if ui
                .add(
                    egui::Button::new(
                        RichText::new("➕ Save & Add Note")
                            .color(Color32::WHITE)
                            .strong(),
                    )
                    .fill(Color32::from_rgb(16, 185, 129)) // Emerald 500
                    .min_size(Vec2::new(140.0, 32.0)),
                )
                .clicked()
            {
                if !self.new_title.trim().is_empty() {
                    let next_id = self.notes.iter().map(|n| n.id).max().unwrap_or(0) + 1;
                    let now = chrono_or_simple_timestamp();
                    let full_body = format!(
                        "{}\n\nCategory: {}",
                        self.new_body.trim(),
                        self.new_category.trim()
                    );
                    let new_note = Note::new(next_id, self.new_title.trim(), full_body, &now, &now);
                    self.notes.push(new_note);
                    self.set_toast(format!("Added note: '{}'", self.new_title.trim()));
                    self.new_title.clear();
                    self.new_body.clear();
                    self.active_category = NavCategory::All;
                } else {
                    self.set_toast("Title cannot be empty!");
                }
            }

            if ui
                .add(egui::Button::new("Cancel").min_size(Vec2::new(80.0, 32.0)))
                .clicked()
            {
                self.active_category = NavCategory::All;
            }
        });
    }

    fn render_main_split_view(&mut self, ui: &mut egui::Ui) {
        let matching_indices = self.filtered_note_indices();

        if matching_indices.is_empty() {
            ui.vertical_centered(|ui| {
                ui.add_space(60.0);
                ui.label(
                    RichText::new("🔍 No matching notes or tweaks found")
                        .size(16.0)
                        .color(Color32::from_rgb(148, 163, 184)),
                );
                ui.label(
                    RichText::new("Try adjusting your search query or selecting 'All Notes'.")
                        .color(Color32::from_rgb(100, 116, 139)),
                );
            });
            return;
        }

        if !matching_indices.contains(&self.selected_note_index) {
            self.selected_note_index = matching_indices[0];
        }

        // Two-column layout: Left column = cards list; Right column = note details & live tweak controls
        ui.columns(2, |columns| {
            // LEFT COLUMN: Card list
            columns[0].vertical(|ui| {
                ui.label(
                    RichText::new(format!("ITEMS ({} found)", matching_indices.len()))
                        .size(10.5)
                        .color(Color32::from_rgb(148, 163, 184))
                        .strong(),
                );
                ui.add_space(6.0);

                egui::ScrollArea::vertical()
                    .id_salt("notes_list_scroll")
                    .show(ui, |ui| {
                        for &idx in &matching_indices {
                            let is_selected = idx == self.selected_note_index;
                            let note = &self.notes[idx];
                            let is_tweak = note.id >= 100;

                            let card_bg = if is_selected {
                                Color32::from_rgb(37, 50, 75)
                            } else {
                                Color32::from_rgb(32, 35, 45)
                            };

                            let card_stroke = if is_selected {
                                Stroke::new(1.5_f32, Color32::from_rgb(56, 189, 248))
                            } else {
                                Stroke::new(1.0_f32, Color32::from_rgb(45, 50, 65))
                            };

                            egui::Frame::none()
                                .fill(card_bg)
                                .stroke(card_stroke)
                                .rounding(Rounding::same(6.0_f32))
                                .inner_margin(egui::Margin::same(10.0))
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(
                                            RichText::new(format!("#{}", note.id))
                                                .color(Color32::from_rgb(100, 116, 139))
                                                .size(11.0),
                                        );

                                        if is_tweak {
                                            ui.label(
                                                RichText::new("⚙ TWEAK")
                                                    .color(Color32::from_rgb(56, 189, 248))
                                                    .size(10.0)
                                                    .strong(),
                                            );
                                        }

                                        let first_line_body =
                                            note.body.lines().next().unwrap_or("");
                                        if first_line_body.contains("ACTIVE [Enabled]")
                                            || note.body.contains("ACTIVE [Enabled]")
                                        {
                                            ui.label(
                                                RichText::new("● ON")
                                                    .color(Color32::from_rgb(34, 197, 94))
                                                    .size(10.0)
                                                    .strong(),
                                            );
                                        }
                                    });

                                    let title_btn = egui::Button::new(
                                        RichText::new(&note.title).size(13.0).strong(),
                                    )
                                    .frame(false);

                                    if ui.add(title_btn).clicked() {
                                        self.selected_note_index = idx;
                                    }

                                    let preview = note
                                        .body
                                        .lines()
                                        .find(|l| !l.is_empty())
                                        .unwrap_or("")
                                        .chars()
                                        .take(70)
                                        .collect::<String>();

                                    ui.label(
                                        RichText::new(preview)
                                            .color(Color32::from_rgb(148, 163, 184))
                                            .size(11.0),
                                    );
                                });

                            ui.add_space(6.0);
                        }
                    });
            });

            // RIGHT COLUMN: Detail & Live Tweak Action Panel
            columns[1].vertical(|ui| {
                ui.label(
                    RichText::new("INSPECTOR & LIVE ACTIONS")
                        .size(10.5)
                        .color(Color32::from_rgb(148, 163, 184))
                        .strong(),
                );
                ui.add_space(6.0);

                if self.selected_note_index < self.notes.len() {
                    let sel_id = self.notes[self.selected_note_index].id;
                    let associated_tweak = self.tweaks.iter().find(|t| t.id == sel_id).cloned();

                    // If this is a real Windows 10 tweak, show live registry status and toggle button!
                    if let Some(tweak) = associated_tweak {
                        let live_status = read_tweak_status(&tweak);
                        let is_enabled = live_status == Some(true);

                        egui::Frame::none()
                            .fill(Color32::from_rgb(20, 30, 48))
                            .stroke(Stroke::new(1.0_f32, Color32::from_rgb(56, 189, 248)))
                            .rounding(Rounding::same(8.0_f32))
                            .inner_margin(egui::Margin::same(12.0))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label(
                                        RichText::new("⚡ Live Windows 10 System Tweak")
                                            .color(Color32::from_rgb(56, 189, 248))
                                            .strong(),
                                    );

                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            if is_enabled {
                                                ui.label(
                                                    RichText::new("● ACTIVE (Enabled)")
                                                        .color(Color32::from_rgb(34, 197, 94))
                                                        .strong(),
                                                );
                                            } else {
                                                ui.label(
                                                    RichText::new("○ INACTIVE (Default)")
                                                        .color(Color32::from_rgb(148, 163, 184)),
                                                );
                                            }
                                        },
                                    );
                                });

                                ui.add_space(4.0);
                                ui.label(
                                    RichText::new(format!(
                                        "Registry: HKCU\\{}\\{}",
                                        tweak.registry_path, tweak.value_name
                                    ))
                                    .size(10.5)
                                    .color(Color32::from_rgb(148, 163, 184)),
                                );
                                ui.add_space(8.0);

                                let toggle_btn_text = if is_enabled {
                                    RichText::new("↩ Disable Tweak (Restore Windows Default)")
                                        .color(Color32::WHITE)
                                } else {
                                    RichText::new("⚡ Apply Tweak to Windows 10")
                                        .color(Color32::WHITE)
                                        .strong()
                                };

                                let toggle_color = if is_enabled {
                                    Color32::from_rgb(180, 83, 9) // Amber 700
                                } else {
                                    Color32::from_rgb(16, 185, 129) // Emerald 500
                                };

                                if ui
                                    .add(
                                        egui::Button::new(toggle_btn_text)
                                            .fill(toggle_color)
                                            .min_size(Vec2::new(f32::INFINITY, 30.0)),
                                    )
                                    .clicked()
                                {
                                    match toggle_tweak(&tweak) {
                                        Ok(new_state) => {
                                            let state_str =
                                                if new_state { "Enabled" } else { "Disabled" };
                                            self.set_toast(format!(
                                                "✔ Windows 10 tweak '{}' is now {}",
                                                tweak.title, state_str
                                            ));

                                            // Update note body
                                            let updated_note = tweak.to_note(Some(new_state));
                                            self.notes[self.selected_note_index] = updated_note;
                                        }
                                        Err(e) => {
                                            self.set_toast(format!("Failed to apply tweak: {}", e));
                                        }
                                    }
                                }
                            });
                        ui.add_space(10.0);
                    }

                    // Editable Note view
                    ui.label(RichText::new("Title:").size(11.0).strong());
                    ui.add(
                        egui::TextEdit::singleline(&mut self.notes[self.selected_note_index].title)
                            .desired_width(f32::INFINITY),
                    );
                    ui.add_space(6.0);

                    ui.label(RichText::new("Description / Details:").size(11.0).strong());
                    ui.add(
                        egui::TextEdit::multiline(&mut self.notes[self.selected_note_index].body)
                            .desired_rows(8)
                            .desired_width(f32::INFINITY),
                    );
                    ui.add_space(8.0);

                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(format!(
                                "Created: {} | Updated: {}",
                                self.notes[self.selected_note_index].created_at,
                                self.notes[self.selected_note_index].updated_at
                            ))
                            .size(10.0)
                            .color(Color32::from_rgb(100, 116, 139)),
                        );
                    });

                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        if ui
                            .add(egui::Button::new("💾 Save Note").min_size(Vec2::new(90.0, 26.0)))
                            .clicked()
                        {
                            self.notes[self.selected_note_index].updated_at =
                                chrono_or_simple_timestamp();
                            self.set_toast("Saved note changes!");
                        }

                        if ui
                            .add(
                                egui::Button::new(
                                    RichText::new("🗑 Delete")
                                        .color(Color32::from_rgb(248, 113, 113)),
                                )
                                .min_size(Vec2::new(80.0, 26.0)),
                            )
                            .clicked()
                        {
                            self.notes.remove(self.selected_note_index);
                            self.set_toast("Deleted note.");
                            if self.selected_note_index > 0 {
                                self.selected_note_index -= 1;
                            }
                        }
                    });
                }
            });
        });
    }
}

fn chrono_or_simple_timestamp() -> String {
    let now = std::time::SystemTime::now();
    let duration = now
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = duration.as_secs();
    // Simple UTC representation without heavy dependencies
    format!(
        "2026-11-11T{:02}:{:02}:{:02}Z",
        (secs / 3600) % 24,
        (secs / 60) % 60,
        secs % 60
    )
}
