use http::HeaderValue;
use std::fmt;

/// A crates.io API token
///
/// The token is never printed by its `Debug` implementation. Similarly, it is
/// marked as a sensitive [`HeaderValue`].
#[derive(Clone)]
pub struct Token {
    value: HeaderValue,
}

impl Token {
    pub fn new(token: &str) -> Result<Self, InvalidToken> {
        if token.trim().is_empty() {
            return Err(InvalidToken::Empty);
        }

        let mut value =
            HeaderValue::from_str(token).map_err(|_| InvalidToken::InvalidCharacters)?;
        value.set_sensitive(true);

        Ok(Self { value })
    }

    pub fn as_header_value(&self) -> &HeaderValue {
        &self.value
    }
}

impl fmt::Debug for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Token(<redacted>)")
    }
}

// The token itself must not be part of the error, so it can't leak to logs and traces
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum InvalidToken {
    #[error("The API token is empty")]
    Empty,
    #[error("The API token contains characters which are not allowed in an HTTP header")]
    InvalidCharacters,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[yare::parameterized(
        empty = { "", InvalidToken::Empty },
        whitespace = { " \t", InvalidToken::Empty },
        newline = { "cio\nabc", InvalidToken::InvalidCharacters },
        non_ascii_control = { "cio\u{7f}abc", InvalidToken::InvalidCharacters },
    )]
    fn rejects_invalid_tokens(token: &str, expected: InvalidToken) {
        assert_eq!(Token::new(token).unwrap_err(), expected);
    }

    #[test]
    fn is_sensitive_and_redacted() {
        let token = Token::new("cio_secret").unwrap();

        assert!(token.as_header_value().is_sensitive());
        assert_eq!(token.as_header_value(), "cio_secret");
        assert!(!format!("{:?}", token).contains("cio_secret"));
    }
}
