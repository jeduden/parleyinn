//! The names a repository goes by: its bound commit or none yet.

/// A commit's id in git's SHA-1 object format: forty lowercase
/// hexadecimal digits.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CommitId(String);

impl CommitId {
    /// The id `text` spells, or `None` unless it is exactly forty
    /// lowercase hexadecimal digits.
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        let digits = text.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'));
        (text.len() == 40 && digits).then(|| Self(text.to_owned()))
    }

    /// The forty lowercase hexadecimal digits.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The id of the object whose twenty raw bytes are `bytes`.
    pub(crate) fn from_bytes(bytes: &[u8; 20]) -> Self {
        const DIGITS: &[u8; 16] = b"0123456789abcdef";
        let text = bytes
            .iter()
            .flat_map(|b| [DIGITS[usize::from(b >> 4)], DIGITS[usize::from(b & 15)]])
            .map(char::from)
            .collect();
        Self(text)
    }

    /// The twenty raw bytes of the id.
    pub(crate) fn to_bytes(&self) -> [u8; 20] {
        let digit = |b: u8| if b > b'9' { b - b'a' + 10 } else { b - b'0' };
        let mut out = [0; 20];
        for (byte, pair) in out.iter_mut().zip(self.0.as_bytes().chunks(2)) {
            *byte = pair.iter().fold(0, |acc, &d| (acc << 4) | digit(d));
        }
        out
    }
}

/// What a directory's repository is called (LANE-02).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RepositoryIdentity {
    /// Named by its lowest root commit: the same on every node whose
    /// clone holds it, whatever the path.
    Bound(CommitId),
    /// No root commit is known yet: no commit, or a shallow clone whose
    /// history stops short of one. It carries nothing: the caller mints
    /// the node-local id and binds later, without rewriting the record.
    Provisional,
}

impl RepositoryIdentity {
    /// `Bound` when a root commit was found, `Provisional` otherwise.
    #[must_use]
    pub fn from_root(root: Option<CommitId>) -> Self {
        root.map_or(Self::Provisional, Self::Bound)
    }
}

#[cfg(test)]
mod tests;
