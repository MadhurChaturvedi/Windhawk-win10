use std::ffi::{c_void, OsStr};
use std::os::windows::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use tracing::{error, info};
use windhawk_core::model::Note;
use windhawk_core::paths::default_export_path;
use windhawk_core::{export_notes_to_file, OsVersion};
use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows_sys::Win32::Graphics::Gdi::{GetStockObject, UpdateWindow, DEFAULT_GUI_FONT};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::UI::Controls::{
    InitCommonControlsEx, ICC_STANDARD_CLASSES, INITCOMMONCONTROLSEX,
};
use windows_sys::Win32::UI::WindowsAndMessaging::*;

// Control IDs
const BTN_EXPORT_ID: usize = 1001;
const BTN_EXIT_ID: usize = 1002;
const EDIT_PREVIEW_ID: usize = 1003;

// Win32 Styles & Constants
const SS_LEFT_STYLE: u32 = 0x00000000;
const COLOR_WINDOW_SYS: u32 = 5;

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

pub fn show_error_dialog(hwnd: HWND, title: &str, message: &str) {
    let title_w = to_wide(title);
    let msg_w = to_wide(message);
    unsafe {
        MessageBoxW(hwnd, msg_w.as_ptr(), title_w.as_ptr(), MB_ICONERROR | MB_OK);
    }
}

pub fn show_info_dialog(hwnd: HWND, title: &str, message: &str) {
    let title_w = to_wide(title);
    let msg_w = to_wide(message);
    unsafe {
        MessageBoxW(
            hwnd,
            msg_w.as_ptr(),
            title_w.as_ptr(),
            MB_ICONINFORMATION | MB_OK,
        );
    }
}

pub fn prompt_save_csv_path(hwnd: HWND, default_path: &Path) -> Option<PathBuf> {
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
        lpfnHook: Option<unsafe extern "system" fn(HWND, u32, WPARAM, LPARAM) -> usize>,
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
    ofn.hwndOwner = hwnd;
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

pub struct AppState {
    pub os_version: OsVersion,
    pub notes: Vec<Note>,
}

static APP_STATE: OnceLock<AppState> = OnceLock::new();

pub unsafe fn run_gui(os_version: OsVersion) -> Result<(), anyhow::Error> {
    let icex = INITCOMMONCONTROLSEX {
        dwSize: std::mem::size_of::<INITCOMMONCONTROLSEX>() as u32,
        dwICC: ICC_STANDARD_CLASSES,
    };
    InitCommonControlsEx(&icex);

    let sample_notes = Note::sample_notes();
    let _ = APP_STATE.set(AppState {
        os_version,
        notes: sample_notes,
    });

    let h_instance = GetModuleHandleW(std::ptr::null());
    let class_name = to_wide("WindhawkWin10NotesExporterClass");

    let wnd_class = WNDCLASSEXW {
        cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
        style: CS_HREDRAW | CS_VREDRAW,
        lpfnWndProc: Some(wnd_proc),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: h_instance,
        hIcon: LoadIconW(std::ptr::null_mut(), IDI_APPLICATION),
        hCursor: LoadCursorW(std::ptr::null_mut(), IDC_ARROW),
        hbrBackground: (COLOR_WINDOW_SYS + 1) as isize as *mut c_void,
        lpszMenuName: std::ptr::null(),
        lpszClassName: class_name.as_ptr(),
        hIconSm: LoadIconW(std::ptr::null_mut(), IDI_APPLICATION),
    };

    RegisterClassExW(&wnd_class);

    let window_title = to_wide(&format!(
        "Windhawk Notes Exporter — Windows 10 (Build {})",
        os_version.build_number
    ));

    let hwnd = CreateWindowExW(
        0,
        class_name.as_ptr(),
        window_title.as_ptr(),
        WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX,
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        640,
        480,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
        h_instance,
        std::ptr::null(),
    );

    if hwnd.is_null() {
        return Err(anyhow::anyhow!("Failed to create main application window"));
    }

    ShowWindow(hwnd, SW_SHOW);
    UpdateWindow(hwnd);

    let mut msg: MSG = std::mem::zeroed();
    while GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) > 0 {
        TranslateMessage(&msg);
        DispatchMessageW(&msg);
    }

    Ok(())
}

unsafe extern "system" fn wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_CREATE => {
            let h_font = GetStockObject(DEFAULT_GUI_FONT);
            let state = match APP_STATE.get() {
                Some(s) => s,
                None => return -1,
            };

            // Header title label
            let header_text = to_wide("Windhawk Windows 10 Notes Exporter");
            let static_class = to_wide("STATIC");
            let h_header = CreateWindowExW(
                0,
                static_class.as_ptr(),
                header_text.as_ptr(),
                WS_CHILD | WS_VISIBLE | SS_LEFT_STYLE,
                20,
                15,
                580,
                25,
                hwnd,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null(),
            );
            if !h_header.is_null() {
                SendMessageW(h_header, WM_SETFONT, h_font as usize, 1);
            }

            // Subtitle with OS build verification
            let sub_text = to_wide(&format!(
                "Verified Compatible: Windows 10 Build {} | RFC 4180 CSV with UTF-8 BOM",
                state.os_version.build_number
            ));
            let h_sub = CreateWindowExW(
                0,
                static_class.as_ptr(),
                sub_text.as_ptr(),
                WS_CHILD | WS_VISIBLE | SS_LEFT_STYLE,
                20,
                42,
                580,
                20,
                hwnd,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null(),
            );
            if !h_sub.is_null() {
                SendMessageW(h_sub, WM_SETFONT, h_font as usize, 1);
            }

            // Read-only multi-line preview edit control
            let edit_class = to_wide("EDIT");
            let mut preview_text = String::new();
            preview_text.push_str(&format!(
                "Loaded {} notes ready for export:\r\n\r\n",
                state.notes.len()
            ));
            for note in &state.notes {
                preview_text.push_str(&format!(
                    "[{}] {}\r\n  Created: {} | Updated: {}\r\n  Body: {}\r\n\r\n",
                    note.id, note.title, note.created_at, note.updated_at, note.body
                ));
            }
            let preview_w = to_wide(&preview_text);

            let h_edit = CreateWindowExW(
                WS_EX_CLIENTEDGE,
                edit_class.as_ptr(),
                preview_w.as_ptr(),
                WS_CHILD
                    | WS_VISIBLE
                    | WS_VSCROLL
                    | ES_MULTILINE as u32
                    | ES_AUTOVSCROLL as u32
                    | ES_READONLY as u32,
                20,
                70,
                585,
                290,
                hwnd,
                EDIT_PREVIEW_ID as *mut c_void,
                std::ptr::null_mut(),
                std::ptr::null(),
            );
            if !h_edit.is_null() {
                SendMessageW(h_edit, WM_SETFONT, h_font as usize, 1);
            }

            // Export notes button
            let btn_class = to_wide("BUTTON");
            let btn_export_text = to_wide("Export notes");
            let h_btn_export = CreateWindowExW(
                0,
                btn_class.as_ptr(),
                btn_export_text.as_ptr(),
                WS_CHILD | WS_VISIBLE | BS_DEFPUSHBUTTON as u32,
                20,
                380,
                140,
                36,
                hwnd,
                BTN_EXPORT_ID as *mut c_void,
                std::ptr::null_mut(),
                std::ptr::null(),
            );
            if !h_btn_export.is_null() {
                SendMessageW(h_btn_export, WM_SETFONT, h_font as usize, 1);
            }

            // Exit button
            let btn_exit_text = to_wide("Close");
            let h_btn_exit = CreateWindowExW(
                0,
                btn_class.as_ptr(),
                btn_exit_text.as_ptr(),
                WS_CHILD | WS_VISIBLE | BS_PUSHBUTTON as u32,
                465,
                380,
                140,
                36,
                hwnd,
                BTN_EXIT_ID as *mut c_void,
                std::ptr::null_mut(),
                std::ptr::null(),
            );
            if !h_btn_exit.is_null() {
                SendMessageW(h_btn_exit, WM_SETFONT, h_font as usize, 1);
            }

            0
        }
        WM_COMMAND => {
            let ctrl_id = wparam & 0xFFFF;
            match ctrl_id {
                BTN_EXPORT_ID => {
                    handle_export_click(hwnd);
                    0
                }
                BTN_EXIT_ID => {
                    DestroyWindow(hwnd);
                    0
                }
                _ => DefWindowProcW(hwnd, msg, wparam, lparam),
            }
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            0
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

fn handle_export_click(hwnd: HWND) {
    info!("'Export notes' button clicked");
    let state = match APP_STATE.get() {
        Some(s) => s,
        None => {
            show_error_dialog(hwnd, "Error", "Application state not initialized.");
            return;
        }
    };

    let def_path = default_export_path();
    let chosen_path = match prompt_save_csv_path(hwnd, &def_path) {
        Some(p) => p,
        None => {
            info!("Export notes cancelled by user in file save dialog");
            return;
        }
    };

    info!("Exporting notes to chosen path: {}", chosen_path.display());
    match export_notes_to_file(&state.notes, &chosen_path) {
        Ok(count) => {
            let success_msg = format!(
                "Successfully exported {} notes to:\n\n{}\n\nFormat: RFC 4180 CSV with UTF-8 BOM",
                count,
                chosen_path.display()
            );
            show_info_dialog(hwnd, "Export Complete", &success_msg);
        }
        Err(e) => {
            error!("Failed to export notes: {}", e);
            let err_msg = format!("Failed to export notes:\n\n{}", e);
            show_error_dialog(hwnd, "Export Error", &err_msg);
        }
    }
}
