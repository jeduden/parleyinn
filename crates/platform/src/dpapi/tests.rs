use std::cell::Cell;

use super::*;

/// A stand-in for DPAPI: "protecting" reverses the bytes into a buffer
/// of its own, which `free` takes back, and it counts what it frees.
#[derive(Default)]
struct Fake {
    fail: Option<u32>,
    freed: Cell<usize>,
    /// How many buffers came back to `free` still holding a nonzero byte.
    unwiped: Cell<usize>,
    empty: bool,
}

impl Fake {
    /// # Safety
    ///
    /// As [`Api::protect`].
    unsafe fn reverse(&self, input: *const Blob, output: *mut Blob) -> bool {
        if self.fail.is_some() {
            return false;
        }
        // SAFETY: the caller passes an input blob whose data holds len
        // readable bytes, and a writable output blob.
        let (len, data) = unsafe { ((*input).len, (*input).data) };
        if self.empty {
            // SAFETY: as above.
            unsafe {
                (*output).len = 0;
                (*output).data = std::ptr::null_mut();
            }
            return true;
        }
        let bytes: Vec<u8> = if len == 0 {
            Vec::new()
        } else {
            // SAFETY: as above.
            let input = unsafe { std::slice::from_raw_parts(data, len as usize) };
            input.iter().rev().copied().collect()
        };
        let out = Box::into_raw(bytes.into_boxed_slice());
        // SAFETY: as above.
        unsafe {
            (*output).len = out.len() as u32;
            (*output).data = out.cast::<u8>();
        }
        true
    }
}

// SAFETY: the fake keeps the contract of `Api`: it reads only the input
// blob's bytes, writes only the output blob, and `free` takes back
// exactly the buffers it handed out.
unsafe impl Api for Fake {
    unsafe fn protect(&self, input: *const Blob, output: *mut Blob) -> bool {
        // SAFETY: the caller keeps `Api::protect`'s contract.
        unsafe { self.reverse(input, output) }
    }

    unsafe fn unprotect(&self, input: *const Blob, output: *mut Blob) -> bool {
        // SAFETY: the caller keeps `Api::unprotect`'s contract.
        unsafe { self.reverse(input, output) }
    }

    unsafe fn free(&self, data: *mut u8, len: u32) {
        self.freed.set(self.freed.get() + 1);
        let slice = std::ptr::slice_from_raw_parts_mut(data, len as usize);
        // SAFETY: `data` and `len` are a buffer `reverse` leaked from a
        // boxed slice, handed back once.
        let buffer = unsafe { Box::from_raw(slice) };
        if buffer.iter().any(|&b| b != 0) {
            self.unwiped.set(self.unwiped.get() + 1);
        }
        drop(buffer);
    }

    fn last_error(&self) -> u32 {
        self.fail.unwrap_or(0)
    }
}

#[test]
fn protected_bytes_come_back_and_the_buffer_is_freed_once() {
    let api = Fake::default();

    let blob = protect(&api, b"seed").unwrap();
    let plain = unprotect(&api, &blob).unwrap();

    assert_eq!(blob, b"dees");
    assert_eq!(plain, b"seed");
    assert_eq!(api.freed.get(), 2);
}

#[test]
fn the_buffer_dpapi_returns_is_wiped_before_it_is_freed() {
    let api = Fake::default();

    let blob = protect(&api, b"seed").unwrap();
    let plain = unprotect(&api, &blob).unwrap();

    assert_eq!(plain, b"seed");
    assert_eq!(api.freed.get(), 2);
    assert_eq!(api.unwiped.get(), 0);
}

#[test]
fn an_empty_answer_reads_as_no_bytes() {
    let api = Fake {
        empty: true,
        ..Fake::default()
    };

    assert_eq!(protect(&api, b"seed").unwrap(), b"");
    assert_eq!(api.freed.get(), 0);
    let api = Fake::default();
    assert_eq!(protect(&api, b"").unwrap(), b"");
    assert_eq!(api.freed.get(), 1);
}

#[test]
fn a_failing_call_reports_its_error_and_frees_nothing() {
    let api = Fake {
        fail: Some(13),
        ..Fake::default()
    };

    let err = protect(&api, b"seed").unwrap_err();

    assert_eq!(err, Error::Failed(13));
    assert_eq!(err.to_string(), "DPAPI failed with error 13");
    assert_eq!(unprotect(&api, b"x").unwrap_err(), Error::Failed(13));
    assert_eq!(api.freed.get(), 0);
}

#[test]
fn data_too_long_for_dpapi_is_refused_before_the_call() {
    assert_eq!(length(usize::MAX), Err(Error::TooLong));
    assert_eq!(length(4), Ok(4));
    assert_eq!(Error::TooLong.to_string(), "the data is too long for DPAPI");
}
