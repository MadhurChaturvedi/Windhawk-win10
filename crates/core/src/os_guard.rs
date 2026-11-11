use crate::error::OsVersionError;
use tracing::{debug, info};

pub const MIN_WIN10_BUILD: u32 = 19041;
pub const MAX_WIN10_BUILD: u32 = 21999;

/// Operating System version details.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OsVersion {
    pub major: u32,
    pub minor: u32,
    pub build_number: u32,
}

impl OsVersion {
    pub fn new(major: u32, minor: u32, build_number: u32) -> Self {
        Self {
            major,
            minor,
            build_number,
        }
    }
}

/// Pure validation logic for OS version (mockable for unit tests).
///
/// Accepts:
/// - Windows 10 (major == 10, minor == 0) with build number in [19041, 21999].
///
/// Rejects:
/// - Any major/minor outside Windows 10 (e.g. Windows 7/8/XP).
/// - Builds < 19041 (older Windows 10 builds like 1507, 1607, 1809, 1909).
/// - Builds >= 22000 (Windows 11 builds like 22000, 22621, 26100).
pub fn validate_os_version(version: &OsVersion) -> Result<(), OsVersionError> {
    if version.major != 10 || version.minor != 0 {
        return Err(OsVersionError::UnsupportedMajorVersion {
            major: version.major,
            minor: version.minor,
            build_number: version.build_number,
        });
    }

    if version.build_number < MIN_WIN10_BUILD {
        return Err(OsVersionError::BuildTooOld {
            build_number: version.build_number,
        });
    }

    if version.build_number > MAX_WIN10_BUILD {
        return Err(OsVersionError::Windows11NotSupported {
            build_number: version.build_number,
        });
    }

    Ok(())
}

/// Convenience helper to validate a Windows 10 build number directly.
pub fn validate_windows_10_build(build_number: u32) -> Result<(), OsVersionError> {
    validate_os_version(&OsVersion::new(10, 0, build_number))
}

#[cfg(windows)]
#[repr(C)]
#[allow(non_snake_case)]
struct RTL_OSVERSIONINFOW {
    dwOSVersionInfoSize: u32,
    dwMajorVersion: u32,
    dwMinorVersion: u32,
    dwBuildNumber: u32,
    dwPlatformId: u32,
    szCSDVersion: [u16; 128],
}

#[cfg(windows)]
extern "system" {
    fn RtlGetVersion(lpVersionInformation: *mut RTL_OSVERSIONINFOW) -> i32;
}

/// Queries the true OS version directly via NTDLL `RtlGetVersion`.
/// Unlike Win32 `GetVersionEx`, `RtlGetVersion` is immune to compatibility manifests.
#[cfg(windows)]
pub fn get_real_os_version() -> Result<OsVersion, OsVersionError> {
    unsafe {
        let mut info = RTL_OSVERSIONINFOW {
            dwOSVersionInfoSize: std::mem::size_of::<RTL_OSVERSIONINFOW>() as u32,
            dwMajorVersion: 0,
            dwMinorVersion: 0,
            dwBuildNumber: 0,
            dwPlatformId: 0,
            szCSDVersion: [0; 128],
        };

        let status = RtlGetVersion(&mut info);
        if status != 0 {
            return Err(OsVersionError::QueryFailed(status));
        }

        let version = OsVersion {
            major: info.dwMajorVersion,
            minor: info.dwMinorVersion,
            build_number: info.dwBuildNumber,
        };

        debug!(
            "Queried OS version from RtlGetVersion: {}.{}.{}",
            version.major, version.minor, version.build_number
        );
        Ok(version)
    }
}

#[cfg(not(windows))]
pub fn get_real_os_version() -> Result<OsVersion, OsVersionError> {
    Err(OsVersionError::QueryFailed(-1))
}

/// Checks the real operating system version and enforces the Windows 10 range.
pub fn ensure_os_supported() -> Result<OsVersion, OsVersionError> {
    let version = get_real_os_version()?;
    info!(
        "Enforcing OS compatibility check: major={}, minor={}, build={}",
        version.major, version.minor, version.build_number
    );
    validate_os_version(&version)?;
    info!(
        "OS version check passed: Windows 10 build {} is supported",
        version.build_number
    );
    Ok(version)
}

