//! The host id behind the node identity (REC-24): a value outside the
//! home that a cloned image or a restored snapshot does not carry over.
//!
//! - macOS: the hardware UUID `gethostuuid(2)` returns, the value `ioreg`
//!   shows as `IOPlatformUUID`. It is a system call, not a program
//!   start (I4, SEC-01), and it takes a wait, so it cannot block past
//!   one (ENG-03).
//! - Windows: the `MachineGuid` value under
//!   `HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\Cryptography`, read-only
//!   through the 64-bit registry view, which needs no administrator.
//!
//! Each read goes through a trait a test fakes, so Miri checks the
//! pointer logic on any OS (ENG-09). The value is no secret, but every
//! buffer that holds it is bounded and its length checked. An
//! unavailable value is `None`, never a guess.

use std::time::Duration;

/// How long `gethostuuid` may wait (ENG-03).
pub const WAIT: Duration = Duration::from_millis(100);

/// The bytes the registry read may fill: room for a GUID's 36 units and
/// its terminator, doubled.
const UNITS: usize = 64;

/// `REG_SZ`, the registry type of a string.
const REG_SZ: u32 = 1;

/// Where a host id is read from.
pub trait HostId {
    /// The host id, or none when it is unavailable.
    fn read(&self) -> Option<String>;
}

/// The `timespec` `gethostuuid` takes as its wait.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Timespec {
    pub tv_sec: i64,
    pub tv_nsec: i64,
}

impl Timespec {
    /// `wait` as a timespec, its seconds clamped to what one holds.
    fn of(wait: Duration) -> Self {
        Self {
            tv_sec: i64::try_from(wait.as_secs()).unwrap_or(i64::MAX),
            tv_nsec: i64::from(wait.subsec_nanos()),
        }
    }
}

/// What `gethostuuid` does for the reader.
///
/// # Safety
///
/// An implementation writes only the 16 bytes `id` points at and reads
/// only the timespec `wait` points at.
pub unsafe trait HostUuid {
    /// Writes the hardware UUID into the 16 bytes at `id`, waiting no
    /// longer than `wait`; zero on success, another value on failure or
    /// timeout.
    ///
    /// # Safety
    ///
    /// `id` points at 16 writable bytes and `wait` at a readable
    /// timespec.
    unsafe fn host_uuid(&self, id: *mut u8, wait: *const Timespec) -> i32;
}

/// The hardware UUID `api` returns within `wait`, written as `ioreg`
/// shows it; none on a failure, a timeout or an all-zero value.
pub fn mac_uuid(api: &dyn HostUuid, wait: Duration) -> Option<String> {
    let mut id = [0u8; 16];
    let spec = Timespec::of(wait);
    // SAFETY: `id` is a local of 16 writable bytes and `spec` a local
    // timespec, both alive for the call.
    let status = unsafe { api.host_uuid(id.as_mut_ptr(), &spec) };
    if status != 0 || id == [0u8; 16] {
        return None;
    }
    let hex: String = id.iter().map(|b| format!("{b:02X}")).collect();

    Some(format!(
        "{}-{}-{}-{}-{}",
        &hex[0..8],
        &hex[8..12],
        &hex[12..16],
        &hex[16..20],
        &hex[20..32]
    ))
}

/// What the registry read does for the reader.
///
/// # Safety
///
/// An implementation writes at most `*size` bytes at `data` and sets
/// `*size` and `*kind`, and touches nothing else.
pub unsafe trait Registry {
    /// Reads the `MachineGuid` value into the buffer at `data` of `*size`
    /// bytes, leaving the bytes it holds in `*size` and its registry
    /// type in `*kind`; zero on success, a Windows error code otherwise.
    ///
    /// # Safety
    ///
    /// `data` points at `*size` writable bytes, aligned for `u16`, and
    /// `size` and `kind` at writable words.
    unsafe fn machine_guid(&self, data: *mut u16, size: *mut u32, kind: *mut u32) -> u32;
}

/// The `MachineGuid` `api` reads; none when the read fails, the value is
/// not a string, its size is odd or beyond the buffer, it is not UTF-16,
/// holds a terminator inside it, or is empty.
pub fn machine_guid(api: &dyn Registry) -> Option<String> {
    let mut data = [0u16; UNITS];
    let mut size = u32::try_from(UNITS * 2).ok()?;
    let capacity = size;
    let mut kind = 0u32;
    // SAFETY: `data` is a local of `size` writable bytes, aligned for
    // u16, and `size` and `kind` are locals.
    let status = unsafe { api.machine_guid(data.as_mut_ptr(), &mut size, &mut kind) };
    if status != 0 || kind != REG_SZ || size % 2 != 0 || size > capacity {
        return None;
    }
    let held = &data[..(size / 2) as usize];
    let text = String::from_utf16(held).ok()?;
    let text = text.trim_end_matches('\0').trim();

    (!text.is_empty() && !text.contains('\0')).then(|| text.to_owned())
}

/// `text` as UTF-16 with a terminator, for the registry's wide calls.
#[cfg(any(windows, test))]
fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(std::iter::once(0)).collect()
}

/// `gethostuuid` itself.
#[cfg(target_os = "macos")]
#[derive(Debug, Clone, Copy)]
pub struct Gethostuuid;

#[cfg(target_os = "macos")]
unsafe extern "C" {
    /// `gethostuuid(2)`, in libSystem, which every macOS executable
    /// links. Declared here so no crate is added for one function.
    fn gethostuuid(id: *mut u8, wait: *const Timespec) -> i32;
}

#[cfg(target_os = "macos")]
// SAFETY: it passes the pointers it is given to `gethostuuid`, which
// writes the 16 bytes at `id` and reads the timespec at `wait`.
unsafe impl HostUuid for Gethostuuid {
    unsafe fn host_uuid(&self, id: *mut u8, wait: *const Timespec) -> i32 {
        // SAFETY: the caller keeps `HostUuid::host_uuid`'s contract.
        unsafe { gethostuuid(id, wait) }
    }
}

/// The registry itself, read-only, through the 64-bit view.
#[cfg(windows)]
#[derive(Debug, Clone, Copy)]
pub struct Hklm;

#[cfg(windows)]
// SAFETY: it opens the key read-only, reads the one value into the
// buffer it is given, bounded by `*size`, and closes the key.
unsafe impl Registry for Hklm {
    unsafe fn machine_guid(&self, data: *mut u16, size: *mut u32, kind: *mut u32) -> u32 {
        use std::ptr;
        use windows_sys::Win32::System::Registry::{
            HKEY, HKEY_LOCAL_MACHINE, KEY_READ, KEY_WOW64_64KEY, RegCloseKey, RegOpenKeyExW,
            RegQueryValueExW,
        };
        let key = wide(r"SOFTWARE\Microsoft\Cryptography");
        let name = wide("MachineGuid");
        let mut hkey: HKEY = ptr::null_mut();
        // SAFETY: `key` is a terminated wide string and `hkey` a local.
        let status = unsafe {
            RegOpenKeyExW(
                HKEY_LOCAL_MACHINE,
                key.as_ptr(),
                0,
                KEY_READ | KEY_WOW64_64KEY,
                &mut hkey,
            )
        };
        if status != 0 {
            return status;
        }
        // SAFETY: `hkey` is the key just opened; `name` is terminated;
        // the caller keeps `Registry::machine_guid`'s contract for the
        // buffer and the two words.
        let status =
            unsafe { RegQueryValueExW(hkey, name.as_ptr(), ptr::null(), kind, data.cast(), size) };
        // SAFETY: `hkey` was opened above and is closed once.
        unsafe { RegCloseKey(hkey) };

        status
    }
}

/// The running OS's host id: the hardware UUID on macOS, the
/// `MachineGuid` on Windows, none elsewhere (Linux reads its machine id
/// from a file, which needs no unsafe code).
#[derive(Debug, Clone, Copy)]
pub struct Os;

#[cfg(target_os = "macos")]
impl HostId for Os {
    fn read(&self) -> Option<String> {
        mac_uuid(&Gethostuuid, WAIT)
    }
}

#[cfg(windows)]
impl HostId for Os {
    fn read(&self) -> Option<String> {
        machine_guid(&Hklm)
    }
}

#[cfg(not(any(target_os = "macos", windows)))]
impl HostId for Os {
    fn read(&self) -> Option<String> {
        None
    }
}

#[cfg(test)]
mod tests;
