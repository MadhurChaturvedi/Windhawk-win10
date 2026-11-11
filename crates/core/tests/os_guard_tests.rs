use windhawk_core::error::OsVersionError;
use windhawk_core::os_guard::{
    get_real_os_version, validate_os_version, validate_windows_10_build, OsVersion,
    MAX_WIN10_BUILD, MIN_WIN10_BUILD,
};

#[test]
fn test_older_windows_rejected() {
    // Windows 7 (NT 6.1)
    let win7 = OsVersion::new(6, 1, 7601);
    match validate_os_version(&win7) {
        Err(OsVersionError::UnsupportedMajorVersion { major, minor, .. }) => {
            assert_eq!(major, 6);
            assert_eq!(minor, 1);
        }
        other => panic!("Expected UnsupportedMajorVersion, got {:?}", other),
    }

    // Windows 8.1 (NT 6.3)
    let win81 = OsVersion::new(6, 3, 9600);
    match validate_os_version(&win81) {
        Err(OsVersionError::UnsupportedMajorVersion { major, minor, .. }) => {
            assert_eq!(major, 6);
            assert_eq!(minor, 3);
        }
        other => panic!("Expected UnsupportedMajorVersion, got {:?}", other),
    }
}

#[test]
fn test_windows_10_too_old_builds_rejected() {
    // Windows 10 1507 (RTM, Build 10240)
    match validate_windows_10_build(10240) {
        Err(OsVersionError::BuildTooOld { build_number }) => assert_eq!(build_number, 10240),
        other => panic!("Expected BuildTooOld, got {:?}", other),
    }

    // Windows 10 1809 (Build 17763)
    match validate_windows_10_build(17763) {
        Err(OsVersionError::BuildTooOld { build_number }) => assert_eq!(build_number, 17763),
        other => panic!("Expected BuildTooOld, got {:?}", other),
    }

    // Windows 10 1909 (Build 18363)
    match validate_windows_10_build(18363) {
        Err(OsVersionError::BuildTooOld { build_number }) => assert_eq!(build_number, 18363),
        other => panic!("Expected BuildTooOld, got {:?}", other),
    }

    // Just below threshold
    match validate_windows_10_build(MIN_WIN10_BUILD - 1) {
        Err(OsVersionError::BuildTooOld { build_number }) => {
            assert_eq!(build_number, MIN_WIN10_BUILD - 1)
        }
        other => panic!("Expected BuildTooOld, got {:?}", other),
    }
}

#[test]
fn test_supported_windows_10_builds_accepted() {
    // Min supported: 19041 (Windows 10 2004)
    assert!(validate_windows_10_build(MIN_WIN10_BUILD).is_ok());

    // 19042 (Windows 10 20H2)
    assert!(validate_windows_10_build(19042).is_ok());

    // 19043 (Windows 10 21H1)
    assert!(validate_windows_10_build(19043).is_ok());

    // 19044 (Windows 10 21H2)
    assert!(validate_windows_10_build(19044).is_ok());

    // 19045 (Windows 10 22H2)
    assert!(validate_windows_10_build(19045).is_ok());

    // Max supported: 21999
    assert!(validate_windows_10_build(MAX_WIN10_BUILD).is_ok());
}

#[test]
fn test_windows_11_builds_rejected() {
    // Windows 11 RTM (Build 22000)
    match validate_windows_10_build(22000) {
        Err(OsVersionError::Windows11NotSupported { build_number }) => {
            assert_eq!(build_number, 22000)
        }
        other => panic!("Expected Windows11NotSupported, got {:?}", other),
    }

    // Windows 11 22H2 (Build 22621)
    match validate_windows_10_build(22621) {
        Err(OsVersionError::Windows11NotSupported { build_number }) => {
            assert_eq!(build_number, 22621)
        }
        other => panic!("Expected Windows11NotSupported, got {:?}", other),
    }

    // Windows 11 23H2 (Build 22631)
    match validate_windows_10_build(22631) {
        Err(OsVersionError::Windows11NotSupported { build_number }) => {
            assert_eq!(build_number, 22631)
        }
        other => panic!("Expected Windows11NotSupported, got {:?}", other),
    }

    // Windows 11 24H2 (Build 26100)
    match validate_windows_10_build(26100) {
        Err(OsVersionError::Windows11NotSupported { build_number }) => {
            assert_eq!(build_number, 26100)
        }
        other => panic!("Expected Windows11NotSupported, got {:?}", other),
    }
}

#[cfg(windows)]
#[test]
fn test_real_system_os_version_query() {
    let ver = get_real_os_version().expect("RtlGetVersion must succeed on Windows");
    assert!(ver.major > 0);
    assert!(ver.build_number > 0);
}
