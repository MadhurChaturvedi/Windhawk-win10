use crate::error::WindhawkError;
use crate::model::Note;
use serde::{Deserialize, Serialize};
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use tracing::{debug, info};

#[cfg(windows)]
use windows_sys::Win32::System::Registry::{
    RegCloseKey, RegCreateKeyW, RegOpenKeyExW, RegQueryValueExW, RegSetValueExW, HKEY,
    HKEY_CURRENT_USER, KEY_READ, REG_DWORD,
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
            registry_path: "Software\\Policies\\Microsoft\\Windows\\Explorer".into(),
            value_name: "DisableSearchBoxSuggestions".into(),
            enabled_value: 1,
            disabled_value: 0,
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
    debug!("Broadcasting shell refresh notifications (SHChangeNotify & WM_SETTINGCHANGE)");
    unsafe {
        // Refresh Explorer views
        SHChangeNotify(
            SHCNE_ASSOCCHANGED as i32,
            SHCNF_IDLIST,
            std::ptr::null(),
            std::ptr::null(),
        );

        // Broadcast setting change to top-level windows
        let env_w = to_wide("Environment");
        let mut result: usize = 0;
        SendMessageTimeoutW(
            HWND_BROADCAST,
            WM_SETTINGCHANGE,
            0,
            env_w.as_ptr() as isize,
            SMTO_ABORTIFHUNG,
            1000,
            &mut result,
        );
    }
}

#[cfg(not(windows))]
pub fn notify_shell_changes() {}
