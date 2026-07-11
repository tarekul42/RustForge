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
