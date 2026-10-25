//! Loose objects: one zlib file each under `objects/xx/`.

use std::io::BufReader;
use std::path::Path;

use crate::fsio::open_below;
use crate::inflate::zlib_inflate;
use crate::object::{Kind, Object};
use crate::{CommitId, Error};

/// The longest header (`commit 123456\0`) a loose object may have.
const HEADER: usize = 32;

const BAD_HEADER: Error = Error::Corrupt("loose object header is invalid");

/// Where the loose file of `id` is below the object directory, whether
/// or not there is one.
pub(crate) fn rel_of(id: &CommitId) -> String {
    let (fan, rest) = id.as_str().split_at(2);
    format!("{fan}/{rest}")
}

/// The kind and body of an object's inflated bytes: `kind size NUL body`.
fn split_header(mut data: Vec<u8>) -> Result<Object, Error> {
    let window = data.get(..HEADER).unwrap_or(&data);
    let end = window.iter().position(|&b| b == 0).ok_or(BAD_HEADER)?;
    let header = window.get(..end).ok_or(BAD_HEADER)?;
    let space = header.iter().position(|&b| b == b' ').ok_or(BAD_HEADER)?;
    let (name, size) = header.split_at(space);
    let size = size.get(1..).ok_or(BAD_HEADER)?;
    let digits = !size.is_empty() && size.iter().all(u8::is_ascii_digit);
    let size = std::str::from_utf8(size)
        .ok()
        .filter(|_| digits)
        .and_then(|text| text.parse::<usize>().ok())
        .ok_or(BAD_HEADER)?;
    let kind = Kind::from_name(name).ok_or(BAD_HEADER)?;
    data.drain(..=end);
    if data.len() != size {
        return Err(BAD_HEADER);
    }
    Ok(Object { kind, data })
}

/// The loose object `id`, or `None` when it is not stored loose.
///
/// # Errors
///
/// [`Error::Corrupt`] when the file is not an object;
/// [`Error::TooLarge`] when it holds more than `max_size` bytes.
pub(crate) fn read_up_to(
    objects: &Path,
    id: &CommitId,
    max_size: usize,
) -> Result<Option<Object>, Error> {
    let Some(file) = open_below(objects, &rel_of(id))? else {
        return Ok(None);
    };
    let data = zlib_inflate(BufReader::new(file), max_size.saturating_add(HEADER))?;
    let object = split_header(data)?;
    if object.data.len() > max_size {
        return Err(Error::TooLarge("loose object"));
    }
    Ok(Some(object))
}

#[cfg(test)]
mod tests;
