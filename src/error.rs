use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum IoError {
    CouldNotWrite,
    UserDisconnected,
}

impl Display for IoError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("IO error. User disconnected.")
    }
}

impl Error for IoError {}
