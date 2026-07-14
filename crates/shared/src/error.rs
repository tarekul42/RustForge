use std::fmt;

/// A boxed error type for wrapping `Send + Sync` errors.
#[derive(Debug)]
pub struct Error {
    inner: Box<dyn std::error::Error + Send + Sync>,
}

impl Error {
    /// Create a new error from any `Send + Sync + 'static` error type.
    pub fn new<E>(err: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Self {
            inner: Box::new(err),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt(f)
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&*self.inner)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error as StdError;
    use std::io;

    #[test]
    fn display_delegates_to_inner() {
        let err = Error::new(io::Error::new(io::ErrorKind::NotFound, "file missing"));
        assert_eq!(err.to_string(), "file missing");
    }

    #[test]
    fn source_returns_inner() {
        let inner = io::Error::new(io::ErrorKind::PermissionDenied, "denied");
        let err = Error::new(inner);
        assert!(err.source().is_some());
    }

    #[test]
    fn debug_shows_inner() {
        let err = Error::new(io::Error::other("oops"));
        let debug = format!("{err:?}");
        assert!(debug.contains("Error"));
    }

    #[test]
    fn error_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<Error>();
    }

    #[test]
    fn from_boxed_error() {
        let err = Error::new(io::Error::new(io::ErrorKind::InvalidData, "bad data"));
        assert!(err.to_string().contains("bad data"));
    }
}
