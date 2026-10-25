//! Search without the store: the query compiler (SEC-04) and the ranking
//! of hits ([`rank`], RCL-02). The compiler is the only code that builds
//! a search query. Caller text is never FTS5 syntax. It splits on whitespace
//! into literal terms, and each term is quoted whole, so `OR`, `NEAR(x)`
//! and a stray quote match as the words they are and run no operator.
//! The boolean and phrase operators come only through the structured
//! parameters `any_of`, `all_of` and `phrase`. A query holds at most
//! [`MAX_TERMS`] terms of at most [`MAX_TERM_CHARS`] characters, and a
//! term may not start with the wildcard `*`.
//!
//! The compiler is pure: it reads no clock, no file and no store, so a
//! fuzz target can run it on hostile text. The deadline a query runs
//! under is the store's (SEC-04, ENG-03).

pub mod rank;

use std::fmt;

/// The most terms a query holds, across its text and every structured
/// parameter (SEC-04).
pub const MAX_TERMS: usize = 16;

/// The most characters one term holds (SEC-04).
pub const MAX_TERM_CHARS: usize = 64;

/// What a caller asks to find: its typed text and the structured
/// operators.
#[derive(Debug, Clone, Copy, Default)]
pub struct Request<'a> {
    /// Typed text: literal terms, all of which must match.
    pub text: &'a str,
    /// Terms of which at least one must match.
    pub any_of: &'a [String],
    /// Terms, each of which must match.
    pub all_of: &'a [String],
    /// Words that must match next to each other, in this order.
    pub phrase: Option<&'a str>,
}

/// A query compiled to one FTS5 match expression of quoted terms and
/// the operators the structured parameters asked for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Compiled {
    expression: String,
    terms: usize,
}

impl Compiled {
    /// The FTS5 match expression.
    #[must_use]
    pub fn expression(&self) -> &str {
        &self.expression
    }

    /// How many terms the query held, searchable or not.
    #[must_use]
    pub fn terms(&self) -> usize {
        self.terms
    }
}

/// Why a query is refused. Each is fixed text: it never repeats what the
/// caller typed (RCL-04).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    /// No term holds a letter or digit, so nothing can match.
    Empty,
    /// More than [`MAX_TERMS`] terms.
    TooManyTerms,
    /// A term longer than [`MAX_TERM_CHARS`] characters.
    TermTooLong,
    /// A term starting with `*`.
    LeadingWildcard,
    /// A term holding a control character.
    Control,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("the query holds no searchable term"),
            Self::TooManyTerms => write!(f, "more than {MAX_TERMS} terms"),
            Self::TermTooLong => write!(f, "term longer than {MAX_TERM_CHARS} characters"),
            Self::LeadingWildcard => f.write_str("leading wildcard"),
            Self::Control => f.write_str("a term holds a control character"),
        }
    }
}

impl std::error::Error for Error {}

/// Compiles `request` to a match expression.
///
/// # Errors
///
/// Fails when the query holds more than [`MAX_TERMS`] terms, a term of
/// more than [`MAX_TERM_CHARS`] characters, a term starting with `*` or
/// holding a control character, or no term with a letter or digit.
pub fn compile(request: &Request<'_>) -> Result<Compiled, Error> {
    let mut seen = 0;
    let mut required = Vec::new();
    for term in request.text.split_whitespace() {
        required.extend(literal(term, &mut seen)?);
    }
    for term in request.all_of {
        required.extend(literal(term, &mut seen)?);
    }
    if let Some(phrase) = request.phrase {
        required.extend(literal(phrase, &mut seen)?);
    }
    let mut any = Vec::new();
    for term in request.any_of {
        any.extend(literal(term, &mut seen)?);
    }
    match any.len() {
        0 => {}
        1 => required.extend(any),
        _ => required.push(format!("({})", any.join(" OR "))),
    }
    if required.is_empty() {
        return Err(Error::Empty);
    }

    Ok(Compiled {
        expression: required.join(" AND "),
        terms: seen,
    })
}

/// One term as a quoted FTS5 phrase, counted in `seen`; none for a
/// term with nothing to match, an empty one or one with no letter or
/// digit, which the tokenizer would drop whole.
fn literal(term: &str, seen: &mut usize) -> Result<Option<String>, Error> {
    if term.is_empty() {
        return Ok(None);
    }
    *seen += 1;
    if *seen > MAX_TERMS {
        return Err(Error::TooManyTerms);
    }
    if term.starts_with('*') {
        return Err(Error::LeadingWildcard);
    }
    if term.chars().take(MAX_TERM_CHARS + 1).count() > MAX_TERM_CHARS {
        return Err(Error::TermTooLong);
    }
    if term.chars().any(char::is_control) {
        return Err(Error::Control);
    }
    let searchable = term.chars().any(char::is_alphanumeric);

    Ok(searchable.then(|| format!("\"{}\"", term.replace('"', "\"\""))))
}

#[cfg(test)]
mod tests;
