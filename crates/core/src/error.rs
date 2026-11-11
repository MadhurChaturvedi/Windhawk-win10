use thiserror::Error;

#[derive(Error, Debug)]
pub enum OsVersionError {
    #[error("Unsupported Windows version: build {build_number}. This application requires Windows 10 build 19041 (version 2004) or later.")]
    BuildTooOld { build_number: u32 },

    #[error("Windows 11 detected (build {build_number}). Windows 11 (build >= 22000) is not supported by this build; it is already supported by upstream Windhawk.")]
    Windows11NotSupported { build_number: u32 },

    #[error("Unsupported operating system version: {major}.{minor} (build {build_number}). Windows 10 (builds 19041-21999) is required.")]
    UnsupportedMajorVersion {
        major: u32,
        minor: u32,
        build_number: u32,
    },

    #[error("Failed to query OS version via RtlGetVersion (NTSTATUS error: {0:#x})")]
    QueryFailed(i32),
}

#[derive(Error, Debug)]
pub enum WindhawkError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("CSV serialization error: {0}")]
    Csv(#[from] csv::Error),

    #[error("OS compatibility error: {0}")]
    OsVersion(#[from] OsVersionError),

    #[error("Path handling error: {0}")]
    PathError(String),

    #[error("Invalid data error: {0}")]
    InvalidData(String),
}
