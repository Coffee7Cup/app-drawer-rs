use core::fmt;
use std::fmt::{Display, Formatter, write};

#[derive(Debug)]
pub enum Error {
    NitiIpcError(String),
    InternalError(String),
    NiriAppFetchError,
    ParseError,
    CacheParse,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::NitiIpcError(msg) => {
                write!(f, "NIRI IPC ERROR {} ", msg)
            }

            Self::InternalError(msg) => {
                write!(f, "INTERNAL ERROR {} ", msg)
            }

            Self::ParseError => write!(f, "Error while casting the types"),

            Self::CacheParse => write!(f, "Error while parsing Cache"),

            Self::NiriAppFetchError => write!(f, "Error while fetching apps."),
        }
    }
}
