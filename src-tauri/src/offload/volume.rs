// Ported from DIT-Pro (https://github.com/WillZ5/DIT-Pro), MIT License,
// Copyright (c) 2026 WillZ. See repository NOTICE. Adapted for Meta Report offload mode.
//
// Space queries and physical-volume identity protect preflight, scheduling and
// SAFE_TO_FORMAT topology checks.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Storage device type for IO scheduling
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum DeviceType {
    HDD,
    SSD,
    SD,
    RAID,
    Network,
    Unknown,
}

impl std::fmt::Display for DeviceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeviceType::HDD => write!(f, "HDD"),
            DeviceType::SSD => write!(f, "SSD"),
            DeviceType::SD => write!(f, "SD"),
            DeviceType::RAID => write!(f, "RAID"),
            DeviceType::Network => write!(f, "Network"),
            DeviceType::Unknown => write!(f, "Unknown"),
        }
    }
}

impl DeviceType {
    /// Parse from string (loose match, e.g. UI/settings values)
    pub fn from_str_loose(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "HDD" => DeviceType::HDD,
            "SSD" | "NVME" => DeviceType::SSD, // NVMe merged into SSD
            "SD" | "SDCARD" | "CF" | "CFEXPRESS" => DeviceType::SD,
            "RAID" => DeviceType::RAID,
            "NETWORK" => DeviceType::Network,
            _ => DeviceType::Unknown,
        }
    }
}

/// Space usage summary for a volume
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeSpaceInfo {
    pub total_bytes: u64,
    pub available_bytes: u64,
    pub used_bytes: u64,
    pub usage_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct VolumeIdentity {
    /// Stable enough to distinguish mounted physical volumes during one job.
    pub key: String,
    /// Media/volume binding used across resume. Unlike `key`, this changes when
    /// another card is inserted into the same reader or physical-disk slot.
    #[serde(default)]
    pub fingerprint: String,
    pub path: String,
    pub device_type: DeviceType,
    pub total_bytes: u64,
    pub available_bytes: u64,
    /// True only when the platform proved that this identity represents
    /// physical storage rather than a path, network share or disk image.
    #[serde(default)]
    pub is_physical: bool,
}

pub fn identify_volume(path: &Path) -> Result<VolumeIdentity> {
    let canonical = std::fs::canonicalize(path)
        .with_context(|| format!("Failed to resolve volume path {:?}", path))?;
    let space = get_volume_space(&canonical)?;
    #[cfg(target_os = "macos")]
    let (key, fingerprint, device_type, is_physical) = macos_volume_identity(&canonical)
        .unwrap_or_else(|| {
            use std::os::unix::fs::MetadataExt;
            let fallback = format!(
                "dev:{}",
                std::fs::metadata(&canonical)
                    .map(|metadata| metadata.dev())
                    .unwrap_or_default()
            );
            (fallback.clone(), fallback, DeviceType::Unknown, false)
        });
    #[cfg(all(unix, not(target_os = "macos")))]
    let (key, fingerprint, device_type, is_physical) = {
        use std::os::unix::fs::MetadataExt;
        let key = format!("dev:{}", std::fs::metadata(&canonical)?.dev());
        (key.clone(), key, DeviceType::Unknown, false)
    };
    #[cfg(windows)]
    let (key, fingerprint, device_type, is_physical) = windows_volume_identity(&canonical)?;
    #[cfg(not(any(unix, windows)))]
    let (key, fingerprint, device_type, is_physical) = (
        format!("path:{}", canonical.display()),
        format!("path:{}", canonical.display()),
        DeviceType::Unknown,
        false,
    );

    Ok(VolumeIdentity {
        key,
        fingerprint,
        path: canonical.to_string_lossy().into_owned(),
        device_type,
        total_bytes: space.total_bytes,
        available_bytes: space.available_bytes,
        is_physical,
    })
}

#[cfg(windows)]
fn windows_volume_identity(path: &Path) -> Result<(String, String, DeviceType, bool)> {
    use std::os::windows::ffi::OsStrExt;
    use windows::core::PCWSTR;
    use windows::Win32::Storage::FileSystem::{
        GetDriveTypeW, GetVolumeInformationW, GetVolumeNameForVolumeMountPointW, GetVolumePathNameW,
    };

    let path_wide: Vec<u16> = path.as_os_str().encode_wide().chain([0]).collect();
    let mut root = vec![0_u16; 261];
    unsafe { GetVolumePathNameW(PCWSTR(path_wide.as_ptr()), &mut root) }
        .context("GetVolumePathNameW failed")?;
    let root_len = root
        .iter()
        .position(|value| *value == 0)
        .unwrap_or(root.len());
    let root_string = String::from_utf16_lossy(&root[..root_len]);

    let mut guid = vec![0_u16; 261];
    let guid_key = unsafe { GetVolumeNameForVolumeMountPointW(PCWSTR(root.as_ptr()), &mut guid) }
        .ok()
        .map(|_| {
            let length = guid
                .iter()
                .position(|value| *value == 0)
                .unwrap_or(guid.len());
            String::from_utf16_lossy(&guid[..length]).to_ascii_lowercase()
        });

    let mut serial = 0_u32;
    let mut filesystem = vec![0_u16; 64];
    unsafe {
        GetVolumeInformationW(
            PCWSTR(root.as_ptr()),
            None,
            Some(&mut serial),
            None,
            None,
            Some(&mut filesystem),
        )
    }
    .context("GetVolumeInformationW failed")?;
    let filesystem_len = filesystem
        .iter()
        .position(|value| *value == 0)
        .unwrap_or(filesystem.len());
    let filesystem_name = String::from_utf16_lossy(&filesystem[..filesystem_len]);
    let fallback_key = guid_key.clone().unwrap_or_else(|| {
        format!(
            "win-volume:{serial:08x}:{}:{}",
            filesystem_name.to_ascii_lowercase(),
            root_string.to_ascii_lowercase()
        )
    });
    let fingerprint = format!(
        "win-media:{}:{serial:08x}:{}",
        guid_key
            .as_deref()
            .unwrap_or(&root_string)
            .to_ascii_lowercase(),
        filesystem_name.to_ascii_lowercase()
    );
    let device_type = match unsafe { GetDriveTypeW(PCWSTR(root.as_ptr())) } {
        2 => DeviceType::SD,      // DRIVE_REMOVABLE
        4 => DeviceType::Network, // DRIVE_REMOTE
        _ => DeviceType::Unknown,
    };
    let physical = guid_key
        .as_deref()
        .and_then(windows_storage_device_identity);
    Ok(physical
        .map(|(key, bus_type)| (key, fingerprint.clone(), bus_type, true))
        .unwrap_or((fallback_key, fingerprint, device_type, false)))
}

#[cfg(windows)]
fn windows_storage_device_identity(volume_guid: &str) -> Option<(String, DeviceType)> {
    use std::mem::size_of;
    use std::os::windows::ffi::OsStrExt;
    use windows::core::PCWSTR;
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::Storage::FileSystem::{
        BusTypeAta, BusTypeAtapi, BusTypeNvme, BusTypeRAID, BusTypeSas, BusTypeSata, BusTypeSd,
        BusTypeUsb, CreateFileW, FILE_ATTRIBUTE_NORMAL, FILE_SHARE_READ, FILE_SHARE_WRITE,
        OPEN_EXISTING,
    };
    use windows::Win32::System::Ioctl::{
        PropertyStandardQuery, StorageDeviceProperty, IOCTL_STORAGE_GET_DEVICE_NUMBER,
        IOCTL_STORAGE_QUERY_PROPERTY, STORAGE_DEVICE_DESCRIPTOR, STORAGE_DEVICE_NUMBER,
        STORAGE_PROPERTY_QUERY,
    };
    use windows::Win32::System::IO::DeviceIoControl;

    let device_path = volume_guid.trim_end_matches(['\\', '/']);
    let wide: Vec<u16> = std::ffi::OsStr::new(device_path)
        .encode_wide()
        .chain([0])
        .collect();
    let handle = unsafe {
        CreateFileW(
            PCWSTR(wide.as_ptr()),
            0,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            None,
            OPEN_EXISTING,
            FILE_ATTRIBUTE_NORMAL,
            None,
        )
    }
    .ok()?;

    let result = (|| {
        let mut number = STORAGE_DEVICE_NUMBER::default();
        let mut returned = 0_u32;
        unsafe {
            DeviceIoControl(
                handle,
                IOCTL_STORAGE_GET_DEVICE_NUMBER,
                None,
                0,
                Some((&mut number as *mut STORAGE_DEVICE_NUMBER).cast()),
                size_of::<STORAGE_DEVICE_NUMBER>() as u32,
                Some(&mut returned),
                None,
            )
        }
        .ok()?;

        let query = STORAGE_PROPERTY_QUERY {
            PropertyId: StorageDeviceProperty,
            QueryType: PropertyStandardQuery,
            AdditionalParameters: [0],
        };
        let mut descriptor = STORAGE_DEVICE_DESCRIPTOR::default();
        unsafe {
            DeviceIoControl(
                handle,
                IOCTL_STORAGE_QUERY_PROPERTY,
                Some((&query as *const STORAGE_PROPERTY_QUERY).cast()),
                size_of::<STORAGE_PROPERTY_QUERY>() as u32,
                Some((&mut descriptor as *mut STORAGE_DEVICE_DESCRIPTOR).cast()),
                size_of::<STORAGE_DEVICE_DESCRIPTOR>() as u32,
                Some(&mut returned),
                None,
            )
        }
        .ok()?;

        let device_type = if descriptor.BusType == BusTypeSd {
            DeviceType::SD
        } else if descriptor.BusType == BusTypeNvme {
            DeviceType::SSD
        } else if descriptor.BusType == BusTypeRAID {
            DeviceType::RAID
        } else if descriptor.BusType == BusTypeAta
            || descriptor.BusType == BusTypeAtapi
            || descriptor.BusType == BusTypeSata
            || descriptor.BusType == BusTypeSas
            || descriptor.BusType == BusTypeUsb
        {
            DeviceType::Unknown
        } else {
            // Virtual, file-backed, Storage Spaces and unknown bus types are
            // deliberately not accepted as proof of physical independence.
            return None;
        };
        Some((
            format!("win-physical:{}:{}", number.DeviceType, number.DeviceNumber),
            device_type,
        ))
    })();
    let _ = unsafe { CloseHandle(handle) };
    result
}

#[cfg(target_os = "macos")]
fn macos_volume_identity(path: &Path) -> Option<(String, String, DeviceType, bool)> {
    let diskutil = |target: &std::ffi::OsStr| {
        std::process::Command::new("/usr/sbin/diskutil")
            .arg("info")
            .arg(target)
            .output()
            .ok()
            .filter(|output| output.status.success())
    };

    // `diskutil info` accepts a mount point but rejects descendants such as
    // `/Volumes/Card/project/day-01`. Resolve that descendant through POSIX
    // `df` to its /dev/diskNsM device before asking diskutil for topology.
    let output = diskutil(path.as_os_str()).or_else(|| {
        let df = std::process::Command::new("/bin/df")
            .arg("-P")
            .arg(path)
            .output()
            .ok()?;
        if !df.status.success() {
            return None;
        }
        let device = macos_df_device(&String::from_utf8_lossy(&df.stdout))?;
        diskutil(std::ffi::OsStr::new(&device))
    })?;
    parse_macos_diskutil_info(&String::from_utf8_lossy(&output.stdout))
}

#[cfg(target_os = "macos")]
fn macos_df_device(output: &str) -> Option<String> {
    output
        .lines()
        .skip(1)
        .find_map(|line| line.split_whitespace().next())
        .filter(|device| device.starts_with("/dev/disk"))
        .map(str::to_string)
}

#[cfg(target_os = "macos")]
fn parse_macos_diskutil_info(info: &str) -> Option<(String, String, DeviceType, bool)> {
    let value = |field: &str| {
        info.lines().find_map(|line| {
            let (name, value) = line.split_once(':')?;
            (name.trim() == field).then(|| value.trim().to_string())
        })
    };

    // APFS exposes synthesized container disks. Physical Store is the actual
    // medium; otherwise Part of Whole groups all partitions of one drive.
    let physical_store = value("APFS Physical Store")
        .or_else(|| value("Part of Whole"))
        .or_else(|| value("Device Identifier"))?;
    let physical = macos_whole_disk_identifier(&physical_store);
    let fingerprint = value("Volume UUID")
        .or_else(|| value("Disk / Partition UUID"))
        .map(|uuid| format!("mac-volume:{}", uuid.to_ascii_lowercase()))
        .unwrap_or_else(|| {
            format!(
                "mac-media:{}:{}:{}",
                value("Device Tree Path").unwrap_or_default(),
                value("Disk Size").unwrap_or_default(),
                physical
            )
        });
    let protocol = value("Protocol").unwrap_or_default().to_ascii_lowercase();
    let removable = value("Removable Media")
        .unwrap_or_default()
        .to_ascii_lowercase();
    let solid_state = value("Solid State")
        .unwrap_or_default()
        .eq_ignore_ascii_case("yes");
    let virtual_media = value("Virtual")
        .or_else(|| value("Disk Image"))
        .unwrap_or_default()
        .eq_ignore_ascii_case("yes")
        || protocol.contains("disk image")
        || protocol.contains("virtual");
    let device_type = if protocol.contains("secure digital")
        || protocol.contains("sd")
        || removable.contains("removable")
    {
        DeviceType::SD
    } else if solid_state {
        DeviceType::SSD
    } else {
        DeviceType::HDD
    };
    Some((
        format!("mac-physical:{physical}"),
        fingerprint,
        device_type,
        !virtual_media,
    ))
}

#[cfg(target_os = "macos")]
fn macos_whole_disk_identifier(identifier: &str) -> String {
    let trimmed = identifier.trim();
    let Some(suffix) = trimmed.strip_prefix("disk") else {
        return trimmed.to_string();
    };
    let digit_count = suffix
        .chars()
        .take_while(|character| character.is_ascii_digit())
        .count();
    if digit_count == 0 {
        trimmed.to_string()
    } else {
        format!("disk{}", &suffix[..digit_count])
    }
}

impl VolumeSpaceInfo {
    /// Check if there's enough space for a given file size
    pub fn has_space_for(&self, required_bytes: u64) -> bool {
        self.available_bytes >= required_bytes
    }

    /// Check if the volume is critically low on space (< 1 GB)
    pub fn is_critically_low(&self) -> bool {
        self.available_bytes < 1_073_741_824 // 1 GB
    }

    /// Check if the volume is low on space (< 10 GB)
    pub fn is_low(&self) -> bool {
        self.available_bytes < 10_737_418_240 // 10 GB
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Unix (macOS / Linux) — Volume space queries
// ═══════════════════════════════════════════════════════════════════════════

/// Query the filesystem for space information on a path.
/// Uses statvfs on Unix systems with sanity checks.
/// For network volumes where statvfs returns garbage, falls back to `df`.
#[cfg(unix)]
pub fn get_volume_space(path: &Path) -> Result<VolumeSpaceInfo> {
    let info = get_volume_space_statvfs(path)?;

    // Sanity check: statvfs often returns garbage on network mounts (SMB/NFS).
    // Detect: total < 1MB, or total < available, or usage >= 100% with free = 0
    // but available > 0 — all signs of broken statvfs.
    let looks_sane = info.total_bytes >= 1_048_576 // at least 1 MB
        && info.total_bytes <= 1_000_000_000_000_000_000 // at most 1 EB
        && info.total_bytes >= info.available_bytes
        && info.usage_percent <= 100.0;

    if looks_sane {
        return Ok(info);
    }

    // Fall back to `df -k` which reports correct values for network mounts
    if let Ok(df_info) = get_volume_space_df(path) {
        return Ok(df_info);
    }

    // If df also fails, return zeros → frontend shows "unknown capacity"
    Ok(VolumeSpaceInfo {
        total_bytes: 0,
        available_bytes: 0,
        used_bytes: 0,
        usage_percent: 0.0,
    })
}

/// Raw statvfs query (fast, but unreliable for network mounts).
#[cfg(unix)]
fn get_volume_space_statvfs(path: &Path) -> Result<VolumeSpaceInfo> {
    use std::ffi::CString;
    use std::mem::MaybeUninit;

    let c_path =
        CString::new(path.to_string_lossy().as_ref()).context("Invalid path for statvfs")?;

    let mut stat = MaybeUninit::<libc::statvfs>::uninit();
    let ret = unsafe { libc::statvfs(c_path.as_ptr(), stat.as_mut_ptr()) };

    if ret != 0 {
        anyhow::bail!(
            "statvfs failed for {:?}: errno {}",
            path,
            std::io::Error::last_os_error()
        );
    }

    let stat = unsafe { stat.assume_init() };

    // Use u128 for intermediate calculations to prevent overflow with
    // extremely large drives or weird block sizes.
    let block_size = stat.f_frsize as u128;
    if block_size == 0 {
        anyhow::bail!("Filesystem reported zero fragment size for {:?}", path);
    }

    let total_bytes = (stat.f_blocks as u128) * block_size;
    let free_bytes = (stat.f_bfree as u128) * block_size;
    let available_bytes = (stat.f_bavail as u128) * block_size;

    // used_bytes is calculated from total minus free blocks.
    let used_bytes = total_bytes.saturating_sub(free_bytes);

    let usage_percent = if total_bytes > 0 {
        (used_bytes as f64 / total_bytes as f64) * 100.0
    } else {
        0.0
    };

    Ok(VolumeSpaceInfo {
        total_bytes: total_bytes as u64,
        available_bytes: available_bytes as u64,
        used_bytes: used_bytes as u64,
        usage_percent,
    })
}

/// Parse `df -k <path>` output to get space info.
/// Used as fallback when statvfs returns garbage (common on SMB/NFS mounts).
#[cfg(unix)]
fn get_volume_space_df(path: &Path) -> Result<VolumeSpaceInfo> {
    use std::process::Command;

    let output = Command::new("df")
        .args(["-k", &path.to_string_lossy()])
        .output()
        .context("Failed to run df")?;

    if !output.status.success() {
        anyhow::bail!("df failed for {:?}", path);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    // df -k output: "Filesystem 1024-blocks Used Available Capacity ..."
    // Skip header line, parse second line
    let data_line = stdout
        .lines()
        .nth(1)
        .context("df output missing data line")?;

    // Fields may be separated by variable whitespace.
    // For network mounts the filesystem field can contain spaces (e.g. "//user@host/share"),
    // so parse from the end to get the numeric fields reliably.
    let fields: Vec<&str> = data_line.split_whitespace().collect();
    // Typical df -k output fields (macOS):
    //   Filesystem  1024-blocks  Used  Available  Capacity  iused  ifree  %iused  Mounted-on
    // We need at least: total(1), used(2), available(3), capacity(4)
    if fields.len() < 5 {
        anyhow::bail!("df output too few fields: {}", data_line);
    }

    // Parse from right: Mounted-on(-1), %iused(-2), ifree(-3), iused(-4),
    //                    Capacity(-5), Available(-6), Used(-7), 1024-blocks(-8)
    // On macOS `df -k` has 9 columns. Parse by index from end for robustness.
    let len = fields.len();
    let total_kb: u64 = fields[len - 8].parse().unwrap_or(0);
    let used_kb: u64 = fields[len - 7].parse().unwrap_or(0);
    let available_kb: u64 = fields[len - 6].parse().unwrap_or(0);

    let total_bytes = total_kb * 1024;
    let used_bytes = used_kb * 1024;
    let available_bytes = available_kb * 1024;
    let usage_percent = if total_bytes > 0 {
        (used_bytes as f64 / total_bytes as f64) * 100.0
    } else {
        0.0
    };

    Ok(VolumeSpaceInfo {
        total_bytes,
        available_bytes,
        used_bytes,
        usage_percent,
    })
}

// ═══════════════════════════════════════════════════════════════════════════
// Windows — Volume space queries via GetDiskFreeSpaceExW
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(windows)]
pub fn get_volume_space(path: &Path) -> Result<VolumeSpaceInfo> {
    use std::os::windows::ffi::OsStrExt;
    use windows::core::PCWSTR;
    use windows::Win32::Storage::FileSystem::GetDiskFreeSpaceExW;

    // Convert path to null-terminated wide string
    let wide_path: Vec<u16> = path
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    let mut free_bytes_available: u64 = 0;
    let mut total_bytes: u64 = 0;
    let mut total_free_bytes: u64 = 0;

    unsafe {
        GetDiskFreeSpaceExW(
            PCWSTR(wide_path.as_ptr()),
            Some(&mut free_bytes_available as *mut u64),
            Some(&mut total_bytes as *mut u64),
            Some(&mut total_free_bytes as *mut u64),
        )
        .context("GetDiskFreeSpaceExW failed")?;
    }

    let used_bytes = total_bytes.saturating_sub(total_free_bytes);
    let usage_percent = if total_bytes > 0 {
        (used_bytes as f64 / total_bytes as f64) * 100.0
    } else {
        0.0
    };

    Ok(VolumeSpaceInfo {
        total_bytes,
        available_bytes: free_bytes_available,
        used_bytes,
        usage_percent,
    })
}

/// Fallback for unsupported platforms
#[cfg(not(any(unix, windows)))]
pub fn get_volume_space(_path: &Path) -> Result<VolumeSpaceInfo> {
    anyhow::bail!("Volume space query not implemented on this platform")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_space_info_helpers() {
        let info = VolumeSpaceInfo {
            total_bytes: 100 * 1_073_741_824,
            available_bytes: 5 * 1_073_741_824,
            used_bytes: 95 * 1_073_741_824,
            usage_percent: 95.0,
        };
        assert!(info.has_space_for(1_073_741_824));
        assert!(!info.has_space_for(10 * 1_073_741_824));
        assert!(!info.is_critically_low());
        assert!(info.is_low());
    }

    #[test]
    fn test_get_volume_space_on_tempdir() {
        // На реальной ФС должен вернуть вменяемые числа.
        let dir = tempfile::tempdir().unwrap();
        let info = get_volume_space(dir.path()).unwrap();
        assert!(info.total_bytes > 0, "total_bytes should be > 0");
        assert!(info.total_bytes >= info.available_bytes);
        assert!(info.usage_percent >= 0.0 && info.usage_percent <= 100.0);
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn macos_identity_uses_physical_store_not_synthesized_volume() {
        let info = "\
Device Identifier: disk3s3\n\
Part of Whole: disk3\n\
Protocol: Apple Fabric\n\
Removable Media: Fixed\n\
Solid State: Yes\n\
APFS Physical Store: disk0s2\n\
Volume UUID: 11111111-2222-3333-4444-555555555555\n";
        let (key, fingerprint, device_type, is_physical) = parse_macos_diskutil_info(info).unwrap();
        assert_eq!(key, "mac-physical:disk0");
        assert_eq!(
            fingerprint,
            "mac-volume:11111111-2222-3333-4444-555555555555"
        );
        assert_eq!(device_type, DeviceType::SSD);
        assert!(is_physical);
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn macos_partitions_on_one_disk_share_physical_key() {
        assert_eq!(macos_whole_disk_identifier("disk0s2"), "disk0");
        assert_eq!(macos_whole_disk_identifier("disk12s3s1"), "disk12");
        assert_eq!(macos_whole_disk_identifier("disk4"), "disk4");
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn macos_fat32_card_is_physical_without_apfs_metadata() {
        let info = "\
Device Identifier: disk4s1\n\
Part of Whole: disk4\n\
File System Personality: MS-DOS FAT32\n\
Protocol: Secure Digital\n\
Volume UUID: B994882E-0231-3413-A73E-47D5FD908BD1\n\
Removable Media: Removable\n";
        let (key, fingerprint, device_type, is_physical) = parse_macos_diskutil_info(info).unwrap();
        assert_eq!(key, "mac-physical:disk4");
        assert_eq!(
            fingerprint,
            "mac-volume:b994882e-0231-3413-a73e-47d5fd908bd1"
        );
        assert_eq!(device_type, DeviceType::SD);
        assert!(is_physical);
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn macos_hfs_usb_ssd_is_physical_without_apfs_metadata() {
        let info = "\
Device Identifier: disk5s3\n\
Part of Whole: disk5\n\
File System Personality: Journaled HFS+\n\
Protocol: USB\n\
Volume UUID: E8A1784F-1EA1-3957-841B-EDED8F1A2A2F\n\
Removable Media: Fixed\n\
Solid State: Yes\n";
        let (key, fingerprint, device_type, is_physical) = parse_macos_diskutil_info(info).unwrap();
        assert_eq!(key, "mac-physical:disk5");
        assert_eq!(
            fingerprint,
            "mac-volume:e8a1784f-1ea1-3957-841b-eded8f1a2a2f"
        );
        assert_eq!(device_type, DeviceType::SSD);
        assert!(is_physical);
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn macos_df_resolves_nested_folder_to_disk_device() {
        let df = "Filesystem 512-blocks Used Available Capacity iused ifree %iused Mounted on\n/dev/disk4s1 31142976 7040 31135936 1% 0 0 0% /Volumes/Kingston\n";
        assert_eq!(macos_df_device(df).as_deref(), Some("/dev/disk4s1"));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn macos_disk_image_never_counts_as_physical_destination() {
        let info = "\
Device Identifier: disk9s1\n\
Part of Whole: disk9\n\
Protocol: Disk Image\n\
Virtual: Yes\n\
Removable Media: Fixed\n";
        let (key, _, _, is_physical) = parse_macos_diskutil_info(info).unwrap();
        assert_eq!(key, "mac-physical:disk9");
        assert!(!is_physical);
    }
}
