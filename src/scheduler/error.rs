use crate::{
    format::{
        Debug, Formatter
    }
};

pub enum Error {
    BadPriority,
    ValueOverflow,
    BadFileDescriptor,
    FileNotFound,
}

impl Debug for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::BadPriority => f.write_str("BadPriority"),
            Error::ValueOverflow => f.write_str("ValueOverflow"),
            Error::BadFileDescriptor => f.write_str("BadFileDescriptor"),
            Error::FileNotFound => f.write_str("FileNotFound"),
        }
    }
}