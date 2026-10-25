//! What an object of the clone is.

/// The largest object this crate reads, whole or as the result of a
/// delta chain. A commit is a few hundred bytes; the bound keeps a
/// hostile pack from making the core allocate without limit.
pub(crate) const MAX_OBJECT: usize = 16 << 20;

/// The four kinds of git object.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Kind {
    Commit,
    Tree,
    Blob,
    Tag,
}

impl Kind {
    /// The kind a pack entry's type number (1 to 4) names.
    pub(crate) fn from_pack(code: u8) -> Option<Self> {
        match code {
            1 => Some(Self::Commit),
            2 => Some(Self::Tree),
            3 => Some(Self::Blob),
            4 => Some(Self::Tag),
            _ => None,
        }
    }

    /// The name a loose object's header and its id's hash spell.
    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::Commit => "commit",
            Self::Tree => "tree",
            Self::Blob => "blob",
            Self::Tag => "tag",
        }
    }

    /// The kind a loose object's header names.
    pub(crate) fn from_name(name: &[u8]) -> Option<Self> {
        match name {
            b"commit" => Some(Self::Commit),
            b"tree" => Some(Self::Tree),
            b"blob" => Some(Self::Blob),
            b"tag" => Some(Self::Tag),
            _ => None,
        }
    }
}

/// An object read from the clone.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Object {
    pub(crate) kind: Kind,
    pub(crate) data: Vec<u8>,
}

#[cfg(test)]
mod tests;
