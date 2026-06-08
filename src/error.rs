use core::fmt;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum Error {
    NitiIpcError(String),
    InternalError(String),
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
        }
    }
}
