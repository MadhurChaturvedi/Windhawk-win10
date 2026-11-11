use crate::error::WindhawkError;
use crate::model::Note;
use serde::{Deserialize, Serialize};
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use tracing::{debug, info};

#[cfg(windows)]
use windows_sys::Win32::System::Registry::{
    RegCloseKey, RegCreateKeyW, RegOpenKeyExW, RegQueryValueExW, RegSetValueExW, HKEY,
    HKEY_CURRENT_USER, KEY_READ, REG_BINARY, REG_DWORD,
};
#[cfg(windows)]
use windows_sys::Win32::UI::Shell::{SHChangeNotify, SHCNE_ASSOCCHANGED, SHCNF_IDLIST};
#[cfg(windows)]
use windows_sys::Win32::UI::WindowsAndMessaging::{
    SendMessageTimeoutW, HWND_BROADCAST, SMTO_ABORTIFHUNG, WM_SETTINGCHANGE,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Win10Tweak {
    pub id: u64,
    pub title: String,
    pub description: String,
    pub category: String,
    pub registry_path: String,
    pub value_name: String,
    pub enabled_value: u32,
    pub disabled_value: u32,
}

impl Win10Tweak {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: u64,
        title: impl Into<String>,
        description: impl Into<String>,
        category: impl Into<String>,
        registry_path: impl Into<String>,
        value_name: impl Into<String>,
        enabled_value: u32,
        disabled_value: u32,
    ) -> Self {
        Self {
            id,
            title: title.into(),
            description: description.into(),
            category: category.into(),
            registry_path: registry_path.into(),
            value_name: value_name.into(),
            enabled_value,
            disabled_value,
        }
    }

    /// Converts this tweak into a project Note representation for export & archiving.
    pub fn to_note(&self, status: Option<bool>) -> Note {
        let status_str = match status {
            Some(true) => "ACTIVE [Enabled]",
            Some(false) => "INACTIVE [Disabled]",
            None => "NOT CONFIGURED",
        };

        let body = format!(
            "{}\n\nCategory: {}\nRegistry: HKCU\\{}\\{}\nCurrent System Status: {}",
            self.description, self.category, self.registry_path, self.value_name, status_str
        );

        Note::new(
            self.id,
            &self.title,
            body,
            "2026-11-11T12:00:00Z",
            "2026-11-11T15:30:00Z",
        )
    }
}

/// Returns the official curated list of Windows 10 productivity & customization tweaks.
pub fn get_available_tweaks() -> Vec<Win10Tweak> {
    vec![
        Win10Tweak {
            id: 101,
            title: "Never Combine Taskbar Buttons (Show Labels)".into(),
            description: "Restores classic Windows taskbar buttons with full title labels visible on every window.".into(),
            category: "Taskbar".into(),
            registry_path: "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\Advanced".into(),
            value_name: "TaskbarGlomLevel".into(),
            enabled_value: 2,
            disabled_value: 0,
        },
        Win10Tweak {
            id: 102,
            title: "Show File Name Extensions in Explorer".into(),
            description: "Always shows file extensions (.exe, .rs, .csv, .txt) in File Explorer for clarity and safety.".into(),
            category: "File Explorer".into(),
            registry_path: "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\Advanced".into(),
            value_name: "HideFileExt".into(),
            enabled_value: 0, // 0 = show extensions
            disabled_value: 1, // 1 = hide extensions
        },
        Win10Tweak {
            id: 103,
            title: "Show Hidden Files and Folders".into(),
            description: "Reveals hidden items across all folder directories in Windows 10.".into(),
            category: "File Explorer".into(),
            registry_path: "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\Advanced".into(),
            value_name: "Hidden".into(),
            enabled_value: 1, // 1 = show hidden
            disabled_value: 2, // 2 = hide
        },
        Win10Tweak {
            id: 104,
            title: "Windows 10 Apps Dark Theme".into(),
            description: "Enforces modern dark theme styling across Windows 10 applications and Settings.".into(),
            category: "Visual Theme".into(),
            registry_path: "Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize".into(),
            value_name: "AppsUseLightTheme".into(),
            enabled_value: 0, // 0 = dark mode
            disabled_value: 1, // 1 = light mode
        },
        Win10Tweak {
            id: 105,
            title: "Windows 10 Taskbar & Shell Dark Theme".into(),
            description: "Enforces dark styling on the Windows 10 taskbar, Start menu, and Action Center.".into(),
            category: "Visual Theme".into(),
            registry_path: "Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize".into(),
            value_name: "SystemUsesLightTheme".into(),
            enabled_value: 0, // 0 = dark mode
            disabled_value: 1, // 1 = light mode
        },
        Win10Tweak {
            id: 106,
            title: "Disable Bing Web Search in Start Menu".into(),
            description: "Speeds up Start menu searches by keeping queries purely local to your Windows 10 PC.".into(),
            category: "Start Menu".into(),
            registry_path: "Software\\Microsoft\\Windows\\CurrentVersion\\Search".into(),
            value_name: "BingSearchEnabled".into(),
            enabled_value: 0, // 0 = Bing search disabled (pure local search)
            disabled_value: 1, // 1 = Bing search enabled (Windows default)
        },
        Win10Tweak {
            id: 107,
            title: "Show Seconds in Taskbar Clock".into(),
            description: "Displays real-time seconds (HH:MM:SS) in the Windows 10 system tray clock.".into(),
            category: "Taskbar".into(),
            registry_path: "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\Advanced".into(),
            value_name: "ShowSecondsInSystemClock".into(),
            enabled_value: 1, // 1 = show seconds
            disabled_value: 0, // 0 = hide seconds
        },
        Win10Tweak {
            id: 108,
            title: "Use Small Taskbar Buttons".into(),
            description: "Shrinks the height of the Windows 10 taskbar and icons for maximum desktop workspace.".into(),
            category: "Taskbar".into(),
            registry_path: "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\Advanced".into(),
            value_name: "TaskbarSmallIcons".into(),
            enabled_value: 1, // 1 = small icons
            disabled_value: 0, // 0 = regular icons
        },
        Win10Tweak {
            id: 109,
            title: "Show 'This PC' Desktop Icon".into(),
            description: "Places the classic 'This PC' (My Computer) shortcut directly onto your Windows 10 desktop.".into(),
            category: "File Explorer".into(),
            registry_path: "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\HideDesktopIcons\\NewStartPanel".into(),
            value_name: "{20D04FE0-3AEA-1069-A2D8-08002B30309D}".into(),
            enabled_value: 0, // 0 = show icon on desktop
            disabled_value: 1, // 1 = hide icon
        },
    ]
}

#[cfg(windows)]
fn to_wide(s: &str) -> Vec<u16> {
    OsStr::new(s)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

/// Reads the current tweak state directly from the Windows 10 registry.
#[cfg(windows)]
pub fn read_tweak_status(tweak: &Win10Tweak) -> Option<bool> {
    let subkey_w = to_wide(&tweak.registry_path);
    let value_w = to_wide(&tweak.value_name);

    unsafe {
        let mut hkey: HKEY = std::ptr::null_mut();
        let res = RegOpenKeyExW(HKEY_CURRENT_USER, subkey_w.as_ptr(), 0, KEY_READ, &mut hkey);
        if res != 0 {
            return None;
        }

        let mut data_type: u32 = 0;
        let mut data: u32 = 0;
        let mut data_len = std::mem::size_of::<u32>() as u32;

        let query_res = RegQueryValueExW(
            hkey,
            value_w.as_ptr(),
            std::ptr::null_mut(),
            &mut data_type,
            &mut data as *mut u32 as *mut u8,
            &mut data_len,
        );

        RegCloseKey(hkey);

        if query_res == 0 && data_type == REG_DWORD {
            Some(data == tweak.enabled_value)
        } else {
            None
        }
    }
}

#[cfg(not(windows))]
pub fn read_tweak_status(_tweak: &Win10Tweak) -> Option<bool> {
    None
}

/// Writes the tweak state into the Windows 10 registry and informs the Shell.
#[cfg(windows)]
pub fn set_tweak_status(tweak: &Win10Tweak, enable: bool) -> Result<(), WindhawkError> {
    let subkey_w = to_wide(&tweak.registry_path);
    let value_w = to_wide(&tweak.value_name);
    let target_val = if enable {
        tweak.enabled_value
    } else {
        tweak.disabled_value
    };

    info!(
        "Writing Windows 10 Registry tweak: HKCU\\{}\\{} = {}",
        tweak.registry_path, tweak.value_name, target_val
    );

    unsafe {
        let mut hkey: HKEY = std::ptr::null_mut();
        let res = RegCreateKeyW(HKEY_CURRENT_USER, subkey_w.as_ptr(), &mut hkey);

        if res != 0 {
            return Err(WindhawkError::PathError(format!(
                "Failed to open/create registry key: HKCU\\{} (error code: {})",
                tweak.registry_path, res
            )));
        }

        let set_res = RegSetValueExW(
            hkey,
            value_w.as_ptr(),
            0,
            REG_DWORD,
            &target_val as *const u32 as *const u8,
            std::mem::size_of::<u32>() as u32,
        );

        RegCloseKey(hkey);

        if set_res != 0 {
            return Err(WindhawkError::PathError(format!(
                "Failed to set registry value {} (error code: {})",
                tweak.value_name, set_res
            )));
        }
    }

    notify_shell_changes();
    Ok(())
}

#[cfg(not(windows))]
pub fn set_tweak_status(_tweak: &Win10Tweak, _enable: bool) -> Result<(), WindhawkError> {
    Ok(())
}

/// Toggles the current state of a Windows 10 tweak.
pub fn toggle_tweak(tweak: &Win10Tweak) -> Result<bool, WindhawkError> {
    let current = read_tweak_status(tweak).unwrap_or(false);
    let target = !current;
    set_tweak_status(tweak, target)?;
    Ok(target)
}

/// Notifies the Windows Explorer Shell that system/environment settings have changed.
#[cfg(windows)]
pub fn notify_shell_changes() {
    debug!("Broadcasting comprehensive shell refresh notifications (SHChangeNotify & WM_SETTINGCHANGE)");
    unsafe {
        // Refresh Explorer views & desktop icons
        SHChangeNotify(
            SHCNE_ASSOCCHANGED as i32,
            SHCNF_IDLIST,
            std::ptr::null(),
            std::ptr::null(),
        );

        // Broadcast setting change to top-level windows across all relevant targets
        let targets = [
            "ImmersiveColorSet",
            "TraySettings",
            "ShellState",
            "Environment",
            "Policy",
        ];
        for target in targets {
            let target_w = to_wide(target);
            let mut result: usize = 0;
            SendMessageTimeoutW(
                HWND_BROADCAST,
                WM_SETTINGCHANGE,
                0,
                target_w.as_ptr() as isize,
                SMTO_ABORTIFHUNG,
                1000,
                &mut result,
            );
        }
    }
}

#[cfg(not(windows))]
pub fn notify_shell_changes() {}

/// Reads the current Windows 10 accent/taskbar color as [R, G, B].
#[cfg(windows)]
pub fn get_taskbar_current_color() -> Option<[u8; 3]> {
    let subkey_w = to_wide("Software\\Microsoft\\Windows\\DWM");
    let val_w = to_wide("AccentColor");
    unsafe {
        let mut hkey: HKEY = std::ptr::null_mut();
        if RegOpenKeyExW(HKEY_CURRENT_USER, subkey_w.as_ptr(), 0, KEY_READ, &mut hkey) != 0 {
            return None;
        }
        let mut data: u32 = 0;
        let mut len = std::mem::size_of::<u32>() as u32;
        let mut data_type: u32 = 0;
        let q = RegQueryValueExW(
            hkey,
            val_w.as_ptr(),
            std::ptr::null_mut(),
            &mut data_type,
            &mut data as *mut u32 as *mut u8,
            &mut len,
        );
        RegCloseKey(hkey);
        if q == 0 && data_type == REG_DWORD {
            // Stored in ABGR format: 0xAABBGGRR
            let r = (data & 0xFF) as u8;
            let g = ((data >> 8) & 0xFF) as u8;
            let b = ((data >> 16) & 0xFF) as u8;
            Some([r, g, b])
        } else {
            None
        }
    }
}

#[cfg(not(windows))]
pub fn get_taskbar_current_color() -> Option<[u8; 3]> {
    None
}

/// Reads whether Windows 10 has ColorPrevalence enabled (shows accent color on Taskbar/Start/Action Center).
#[cfg(windows)]
pub fn get_taskbar_color_prevalence() -> bool {
    let subkey_w = to_wide("Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize");
    let val_w = to_wide("ColorPrevalence");
    unsafe {
        let mut hkey: HKEY = std::ptr::null_mut();
        if RegOpenKeyExW(HKEY_CURRENT_USER, subkey_w.as_ptr(), 0, KEY_READ, &mut hkey) != 0 {
            return false;
        }
        let mut data: u32 = 0;
        let mut len = std::mem::size_of::<u32>() as u32;
        let mut data_type: u32 = 0;
        let q = RegQueryValueExW(
            hkey,
            val_w.as_ptr(),
            std::ptr::null_mut(),
            &mut data_type,
            &mut data as *mut u32 as *mut u8,
            &mut len,
        );
        RegCloseKey(hkey);
        q == 0 && data_type == REG_DWORD && data == 1
    }
}

#[cfg(not(windows))]
pub fn get_taskbar_color_prevalence() -> bool {
    false
}

/// Applies any custom RGB color to the Windows 10 Taskbar, Start Menu, and Action Center.
#[cfg(windows)]
pub fn set_taskbar_custom_color(
    r: u8,
    g: u8,
    b: u8,
    enable_prevalence: bool,
) -> Result<(), WindhawkError> {
    // 1. Calculate formats:
    // ABGR: 0xFF BB GG RR (used by DWM AccentColor, AccentColorMenu, StartColorMenu)
    let abgr: u32 = 0xFF00_0000 | ((b as u32) << 16) | ((g as u32) << 8) | (r as u32);
    // ARGB: 0xC4 RR GG BB (used by DWM ColorizationColor)
    let argb: u32 = 0xC400_0000 | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32);
    let prevalence_val: u32 = if enable_prevalence { 1 } else { 0 };

    info!(
        "Setting Windows 10 Taskbar Color: RGB({}, {}, {}) -> ABGR=0x{:08X}, Prevalence={}",
        r, g, b, abgr, prevalence_val
    );

    unsafe {
        // Set ColorPrevalence in Themes\Personalize
        let personalize_w =
            to_wide("Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize");
        let mut hkey: HKEY = std::ptr::null_mut();
        if RegCreateKeyW(HKEY_CURRENT_USER, personalize_w.as_ptr(), &mut hkey) == 0 {
            let val_name = to_wide("ColorPrevalence");
            RegSetValueExW(
                hkey,
                val_name.as_ptr(),
                0,
                REG_DWORD,
                &prevalence_val as *const u32 as *const u8,
                std::mem::size_of::<u32>() as u32,
            );
            RegCloseKey(hkey);
        }

        // Set DWM ColorPrevalence, AccentColor, and ColorizationColor
        let dwm_w = to_wide("Software\\Microsoft\\Windows\\DWM");
        if RegCreateKeyW(HKEY_CURRENT_USER, dwm_w.as_ptr(), &mut hkey) == 0 {
            let prev_name = to_wide("ColorPrevalence");
            RegSetValueExW(
                hkey,
                prev_name.as_ptr(),
                0,
                REG_DWORD,
                &prevalence_val as *const u32 as *const u8,
                std::mem::size_of::<u32>() as u32,
            );
            let accent_name = to_wide("AccentColor");
            RegSetValueExW(
                hkey,
                accent_name.as_ptr(),
                0,
                REG_DWORD,
                &abgr as *const u32 as *const u8,
                std::mem::size_of::<u32>() as u32,
            );
            let colorize_name = to_wide("ColorizationColor");
            RegSetValueExW(
                hkey,
                colorize_name.as_ptr(),
                0,
                REG_DWORD,
                &argb as *const u32 as *const u8,
                std::mem::size_of::<u32>() as u32,
            );
            RegCloseKey(hkey);
        }

        // Set Explorer\Accent keys
        let accent_w = to_wide("Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\Accent");
        if RegCreateKeyW(HKEY_CURRENT_USER, accent_w.as_ptr(), &mut hkey) == 0 {
            let menu_name = to_wide("AccentColorMenu");
            RegSetValueExW(
                hkey,
                menu_name.as_ptr(),
                0,
                REG_DWORD,
                &abgr as *const u32 as *const u8,
                std::mem::size_of::<u32>() as u32,
            );
            let start_name = to_wide("StartColorMenu");
            RegSetValueExW(
                hkey,
                start_name.as_ptr(),
                0,
                REG_DWORD,
                &abgr as *const u32 as *const u8,
                std::mem::size_of::<u32>() as u32,
            );

            // Generate 32-byte AccentPalette (8 color variations of 4 bytes: RGBA)
            let mut palette = [0u8; 32];
            for i in 0..8 {
                let factor = 0.5 + (i as f32 * 0.1);
                palette[i * 4] = ((r as f32 * factor).min(255.0)) as u8;
                palette[i * 4 + 1] = ((g as f32 * factor).min(255.0)) as u8;
                palette[i * 4 + 2] = ((b as f32 * factor).min(255.0)) as u8;
                palette[i * 4 + 3] = 255;
            }
            let pal_name = to_wide("AccentPalette");
            RegSetValueExW(
                hkey,
                pal_name.as_ptr(),
                0,
                REG_BINARY,
                palette.as_ptr(),
                palette.len() as u32,
            );
            RegCloseKey(hkey);
        }
    }

    notify_shell_changes();
    Ok(())
}

#[cfg(not(windows))]
pub fn set_taskbar_custom_color(
    _r: u8,
    _g: u8,
    _b: u8,
    _enable_prevalence: bool,
) -> Result<(), WindhawkError> {
    Ok(())
}
