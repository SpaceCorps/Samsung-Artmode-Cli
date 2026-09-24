//! Exit codes double as the machine-readable `code:` field in the error envelope. An agent
//! branches on these, so they must stay stable.

use std::fmt;

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorCode {
    Error = 1,
    Network = 2,
    AuthRequired = 3,
    NotFound = 4,
    RateLimited = 5,
    InvalidInput = 6,
    NoAccount = 7,
}

impl ErrorCode {
    pub fn name(self) -> &'static str {
        match self {
            ErrorCode::Error => "error",
            ErrorCode::Network => "network",
            ErrorCode::AuthRequired => "auth_required",
            ErrorCode::NotFound => "not_found",
            ErrorCode::RateLimited => "rate_limited",
            ErrorCode::InvalidInput => "invalid_input",
            ErrorCode::NoAccount => "no_account",
        }
    }
}

#[derive(Debug)]
pub struct Error {
    pub code: ErrorCode,
    pub message: String,
    /// Extra context - the upstream response body, device IP, and so on.
    pub detail: Option<String>,
    /// A literal command that fixes this. Agents surface it verbatim.
    pub remediation: Option<String>,
}

pub type Result<T> = std::result::Result<T, Error>;

#[allow(dead_code)]
impl Error {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Error { code, message: message.into(), detail: None, remediation: None }
    }

    pub fn detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    pub fn fix(mut self, remediation: impl Into<String>) -> Self {
        self.remediation = Some(remediation.into());
        self
    }

    pub fn invalid(message: impl Into<String>) -> Self {
        Error::new(ErrorCode::InvalidInput, message)
    }

    pub fn auth(message: impl Into<String>) -> Self {
        Error::new(ErrorCode::AuthRequired, message)
    }

    pub fn network(message: impl Into<String>) -> Self {
        Error::new(ErrorCode::Network, message)
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Error::new(ErrorCode::NotFound, message)
    }

    pub fn no_account(message: impl Into<String>) -> Self {
        Error::new(ErrorCode::NoAccount, message)
    }

    pub fn other(message: impl Into<String>) -> Self {
        Error::new(ErrorCode::Error, message)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        match e.kind() {
            std::io::ErrorKind::NotFound => Error::invalid(e.to_string()),
            std::io::ErrorKind::ConnectionRefused
            | std::io::ErrorKind::ConnectionReset
            | std::io::ErrorKind::TimedOut => Error::network(e.to_string()),
            _ => Error::other(e.to_string()).detail("io"),
        }
    }
}

impl From<tungstenite::Error> for Error {
    fn from(e: tungstenite::Error) -> Self {
        match &e {
            tungstenite::Error::ConnectionClosed | tungstenite::Error::AlreadyClosed => {
                Error::network("Connection to TV was closed unexpectedly.").detail(e.to_string())
            }
            tungstenite::Error::Io(io_err) => {
                Error::network(format!("Network I/O error: {io_err}")).detail(io_err.to_string())
            }
            tungstenite::Error::Tls(tls_err) => {
                Error::network(format!("TLS handshake with TV failed: {tls_err}")).detail(tls_err.to_string())
            }
            _ => Error::network(format!("WebSocket communication error: {e}")).detail(e.to_string()),
        }
    }
}
