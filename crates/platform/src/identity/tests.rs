use std::cell::Cell;
use std::time::Duration;

use super::*;

/// A stand-in for `gethostuuid`: it writes `uuid` through the pointer
/// it is given and returns `status`, and it remembers the wait it saw.
struct FakeMac {
    status: i32,
    uuid: [u8; 16],
    wait: Cell<Option<(i64, i64)>>,
}

impl FakeMac {
    fn new(status: i32, uuid: [u8; 16]) -> Self {
        Self {
            status,
            uuid,
            wait: Cell::new(None),
        }
    }
}

// SAFETY: it writes the 16 bytes the pointer holds and reads the
// timespec once, as the trait's contract allows.
unsafe impl HostUuid for FakeMac {
    unsafe fn host_uuid(&self, id: *mut u8, wait: *const Timespec) -> i32 {
        // SAFETY: the caller passes a readable timespec and 16 writable
        // bytes.
        unsafe {
            let w = &*wait;
            self.wait.set(Some((w.tv_sec, w.tv_nsec)));
            for (i, b) in self.uuid.iter().enumerate() {
                id.add(i).write(*b);
            }
        }
        self.status
    }
}

const UUID: [u8; 16] = [
    0x56, 0x4D, 0x1B, 0x2E, 0x0A, 0xFF, 0x4C, 0x9D, 0x80, 0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD,
];

#[test]
fn the_hardware_uuid_reads_as_the_hyphenated_form_ioreg_shows() {
    let mac = FakeMac::new(0, UUID);

    assert_eq!(
        mac_uuid(&mac, Duration::from_millis(100)).as_deref(),
        Some("564D1B2E-0AFF-4C9D-8001-23456789ABCD")
    );
}

#[test]
fn the_hardware_uuid_is_asked_for_within_the_wait_it_is_given() {
    let mac = FakeMac::new(0, UUID);

    mac_uuid(&mac, Duration::from_millis(100));

    assert_eq!(mac.wait.get(), Some((0, 100_000_000)));
}

#[test]
fn a_wait_too_long_for_a_timespec_is_clamped() {
    let spec = Timespec::of(Duration::from_secs(u64::MAX));

    assert_eq!(spec.tv_sec, i64::MAX);
    assert_eq!(spec.tv_nsec, 0);
}

#[test]
fn a_timed_out_or_failed_hardware_uuid_call_is_unavailable() {
    let mac = FakeMac::new(-1, UUID);

    assert_eq!(mac_uuid(&mac, Duration::from_millis(100)), None);
}

#[test]
fn an_all_zero_hardware_uuid_is_unavailable() {
    let mac = FakeMac::new(0, [0; 16]);

    assert_eq!(mac_uuid(&mac, Duration::from_millis(100)), None);
}

/// A stand-in for the registry read: it writes `units` through the
/// pointer, as many as the buffer's size in bytes allows, and reports
/// `size` bytes (the bytes written, unless it lies) of type `kind`.
struct FakeRegistry {
    status: u32,
    kind: u32,
    units: Vec<u16>,
    size: Option<u32>,
    offered: Cell<u32>,
}

impl FakeRegistry {
    fn holding(text: &str) -> Self {
        let mut units: Vec<u16> = text.encode_utf16().collect();
        units.push(0);
        Self::with_units(units)
    }

    fn with_units(units: Vec<u16>) -> Self {
        Self {
            status: 0,
            kind: REG_SZ,
            units,
            size: None,
            offered: Cell::new(0),
        }
    }
}

// SAFETY: it writes no more than the `size` bytes it is offered into
// the buffer and sets the two outputs, as the trait's contract allows.
unsafe impl Registry for FakeRegistry {
    unsafe fn machine_guid(&self, data: *mut u16, size: *mut u32, kind: *mut u32) -> u32 {
        // SAFETY: the caller passes a writable buffer of `*size` bytes
        // and two writable words.
        unsafe {
            self.offered.set(*size);
            let room = (*size as usize) / 2;
            let n = self.units.len().min(room);
            for (i, u) in self.units.iter().take(n).enumerate() {
                data.add(i).write(*u);
            }
            *size = self.size.unwrap_or((n * 2) as u32);
            *kind = self.kind;
        }
        self.status
    }
}

const GUID: &str = "0f1e2d3c-4b5a-6978-8796-a5b4c3d2e1f0";

#[test]
fn the_machine_guid_reads_as_the_registry_holds_it() {
    let reg = FakeRegistry::holding(GUID);

    assert_eq!(machine_guid(&reg).as_deref(), Some(GUID));
}

#[test]
fn the_machine_guid_is_read_into_a_bounded_buffer() {
    let reg = FakeRegistry::holding(GUID);

    machine_guid(&reg);

    assert_eq!(reg.offered.get(), 128);
}

#[test]
fn a_machine_guid_with_no_terminator_reads_whole() {
    let reg = FakeRegistry::with_units(GUID.encode_utf16().collect());

    assert_eq!(machine_guid(&reg).as_deref(), Some(GUID));
}

#[test]
fn a_missing_value_or_one_too_large_is_unavailable() {
    for status in [2, 234] {
        let mut reg = FakeRegistry::holding(GUID);
        reg.status = status;

        assert_eq!(machine_guid(&reg), None);
    }
}

#[test]
fn a_value_that_is_not_a_string_is_unavailable() {
    let mut reg = FakeRegistry::holding(GUID);
    reg.kind = 4;

    assert_eq!(machine_guid(&reg), None);
}

#[test]
fn a_size_that_is_odd_or_beyond_the_buffer_is_unavailable() {
    for size in [37, 130, 1000] {
        let mut reg = FakeRegistry::holding(GUID);
        reg.size = Some(size);

        assert_eq!(machine_guid(&reg), None);
    }
}

#[test]
fn a_value_that_is_not_utf16_is_unavailable() {
    let reg = FakeRegistry::with_units(vec![0x30, 0xD800, 0x31, 0]);

    assert_eq!(machine_guid(&reg), None);
}

#[test]
fn a_value_with_a_terminator_inside_it_is_unavailable() {
    let reg = FakeRegistry::with_units(vec![0x30, 0, 0x31, 0]);

    assert_eq!(machine_guid(&reg), None);
}

#[test]
fn an_empty_or_blank_value_is_unavailable() {
    for text in ["", "  "] {
        assert_eq!(machine_guid(&FakeRegistry::holding(text)), None);
    }
    assert_eq!(machine_guid(&FakeRegistry::with_units(Vec::new())), None);
}

#[test]
fn a_wide_string_ends_in_a_terminator() {
    assert_eq!(wide("ab"), vec![0x61, 0x62, 0]);
}

#[cfg(not(any(target_os = "macos", windows)))]
#[test]
fn an_os_with_no_source_reads_none() {
    assert_eq!(Os.read(), None);
}
