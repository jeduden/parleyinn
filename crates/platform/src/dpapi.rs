//! A module of the one crate that holds unsafe code. It hands
//! DPAPI a blob over the caller's bytes, copies out the blob DPAPI
//! returns, and frees it once; every unsafe block says why it is sound.

use std::fmt;
use std::ptr;

/// A blob as DPAPI takes and returns it, laid out as its
/// `CRYPT_INTEGER_BLOB`: a length and a pointer to that many bytes.
#[repr(C)]
#[derive(Debug)]
pub struct Blob {
    pub len: u32,
    pub data: *mut u8,
}

/// What DPAPI does for the shim.
///
/// # Safety
///
/// An implementation reads only the `len` bytes the input blob points
/// at, writes only the output blob, and on success leaves in it a
/// buffer of `len` bytes that stays valid until handed to `free`.
pub unsafe trait Api {
    /// Protects the input blob's bytes for the running OS user into the
    /// output blob; false on failure.
    ///
    /// # Safety
    ///
    /// `input` points at a blob whose data holds `len` readable bytes,
    /// and `output` at a writable blob.
    unsafe fn protect(&self, input: *const Blob, output: *mut Blob) -> bool;

    /// Unprotects the input blob's bytes into the output blob; false on
    /// failure, such as a blob another OS user protected.
    ///
    /// # Safety
    ///
    /// As [`protect`](Self::protect).
    unsafe fn unprotect(&self, input: *const Blob, output: *mut Blob) -> bool;

    /// Frees an output blob's buffer of `len` bytes.
    ///
    /// # Safety
    ///
    /// `data` and `len` are the output blob of one successful call, freed
    /// once.
    unsafe fn free(&self, data: *mut u8, len: u32);

    /// The error the last failed call left.
    fn last_error(&self) -> u32;
}

/// Why DPAPI did not protect or unprotect the bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    /// The bytes are longer than a blob can hold.
    TooLong,
    /// The call failed with this error.
    Failed(u32),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooLong => f.write_str("the data is too long for DPAPI"),
            Self::Failed(code) => write!(f, "DPAPI failed with error {code}"),
        }
    }
}

impl std::error::Error for Error {}

/// `plain`, protected by `api` for the running OS user.
///
/// # Errors
///
/// Fails when `plain` is too long for a blob, or the call fails.
pub fn protect(api: &dyn Api, plain: &[u8]) -> Result<Vec<u8>, Error> {
    call(api, plain, true)
}

/// The bytes `blob` protects, unprotected by `api`.
///
/// # Errors
///
/// Fails when `blob` is too long for a blob, or the call fails.
pub fn unprotect(api: &dyn Api, blob: &[u8]) -> Result<Vec<u8>, Error> {
    call(api, blob, false)
}

/// A blob's length for `len` bytes.
fn length(len: usize) -> Result<u32, Error> {
    u32::try_from(len).map_err(|_| Error::TooLong)
}

/// Protects `data` through `api`, or unprotects it when `protecting` is
/// false, and copies out the bytes it returns.
fn call(api: &dyn Api, data: &[u8], protecting: bool) -> Result<Vec<u8>, Error> {
    let input = Blob {
        len: length(data.len())?,
        data: data.as_ptr().cast_mut(),
    };
    let mut output = Blob {
        len: 0,
        data: ptr::null_mut(),
    };
    // SAFETY: `input` points at `data`, which outlives the call and
    // which DPAPI only reads; `output` is a local DPAPI writes.
    let ok = unsafe {
        if protecting {
            api.protect(&input, &mut output)
        } else {
            api.unprotect(&input, &mut output)
        }
    };
    if !ok {
        return Err(Error::Failed(api.last_error()));
    }
    if output.data.is_null() {
        return Ok(Vec::new());
    }
    // SAFETY: on success `output` holds a buffer of `len` bytes, valid
    // until it is freed below.
    let out = unsafe { std::slice::from_raw_parts(output.data, output.len as usize) }.to_vec();
    for i in 0..output.len as usize {
        // SAFETY: as above; each byte of the buffer is overwritten once,
        // volatile so the wipe of a buffer about to be freed stays.
        unsafe { ptr::write_volatile(output.data.add(i), 0) };
    }
    // SAFETY: the buffer came from this successful call and is freed
    // once, after its bytes were copied out and wiped.
    unsafe { api.free(output.data, output.len) };

    Ok(out)
}

/// DPAPI itself, scoped to the running OS user, with no prompt.
#[cfg(windows)]
#[derive(Debug, Clone, Copy)]
pub struct Windows;

#[cfg(windows)]
// SAFETY: it hands DPAPI the blobs it is given, as DPAPI documents: the
// input is only read, the output written, and the buffer DPAPI returns
// is freed with LocalFree.
unsafe impl Api for Windows {
    unsafe fn protect(&self, input: *const Blob, output: *mut Blob) -> bool {
        use windows_sys::Win32::Security::Cryptography::{
            CRYPTPROTECT_UI_FORBIDDEN, CryptProtectData,
        };
        // SAFETY: the caller keeps `Api::protect`'s contract.
        unsafe {
            let (inp, mut out) = blobs(input);
            let ok = CryptProtectData(
                &inp,
                ptr::null(),
                ptr::null(),
                ptr::null(),
                ptr::null(),
                CRYPTPROTECT_UI_FORBIDDEN,
                &mut out,
            ) != 0;
            (*output).len = out.cbData;
            (*output).data = out.pbData;
            ok
        }
    }

    unsafe fn unprotect(&self, input: *const Blob, output: *mut Blob) -> bool {
        use windows_sys::Win32::Security::Cryptography::{
            CRYPTPROTECT_UI_FORBIDDEN, CryptUnprotectData,
        };
        // SAFETY: the caller keeps `Api::unprotect`'s contract.
        unsafe {
            let (inp, mut out) = blobs(input);
            let ok = CryptUnprotectData(
                &inp,
                ptr::null_mut(),
                ptr::null(),
                ptr::null(),
                ptr::null(),
                CRYPTPROTECT_UI_FORBIDDEN,
                &mut out,
            ) != 0;
            (*output).len = out.cbData;
            (*output).data = out.pbData;
            ok
        }
    }

    unsafe fn free(&self, data: *mut u8, _len: u32) {
        // SAFETY: the caller hands back a buffer DPAPI returned, once.
        unsafe {
            windows_sys::Win32::Foundation::LocalFree(data.cast());
        }
    }

    fn last_error(&self) -> u32 {
        // SAFETY: GetLastError reads the calling thread's last error.
        unsafe { windows_sys::Win32::Foundation::GetLastError() }
    }
}

/// DPAPI's input blob for `input`, and an empty output blob.
///
/// # Safety
///
/// `input` points at a readable blob.
#[cfg(windows)]
unsafe fn blobs(
    input: *const Blob,
) -> (
    windows_sys::Win32::Security::Cryptography::CRYPT_INTEGER_BLOB,
    windows_sys::Win32::Security::Cryptography::CRYPT_INTEGER_BLOB,
) {
    use windows_sys::Win32::Security::Cryptography::CRYPT_INTEGER_BLOB;
    // SAFETY: the caller passes a readable blob.
    let (len, data) = unsafe { ((*input).len, (*input).data) };
    (
        CRYPT_INTEGER_BLOB {
            cbData: len,
            pbData: data,
        },
        CRYPT_INTEGER_BLOB {
            cbData: 0,
            pbData: ptr::null_mut(),
        },
    )
}

#[cfg(test)]
mod tests;
