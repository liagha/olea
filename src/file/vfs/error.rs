use core::fmt::Formatter;
use crate::format::Debug;

#[derive(PartialEq)]
pub enum Error {
    NotImplemented,
    InvalidArgument,
    InvalidFsPath,
    BadFileDescriptor,
    FileNotFound,
    DirectoryNotFound,
    PermissionDenied,
    IoError,
    OutOfMemory,
    AlreadyExists,
    NotADirectory,
    IsADirectory,
    SymlinkLoop,
}

impl Debug for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::NotImplemented => write!(f, "not implemented."),
            Error::InvalidArgument => write!(f, "invalid argument."),
            Error::InvalidFsPath => write!(f, "invalid fs path."),
            Error::BadFileDescriptor => write!(f, "bad file descriptor."),
            Error::FileNotFound => write!(f, "file not found."),
            Error::DirectoryNotFound => write!(f, "directory not found."),
            Error::PermissionDenied => write!(f, "permission denied."),
            Error::IoError => write!(f, "I/O error."),
            Error::OutOfMemory => write!(f, "out of memory."),
            Error::AlreadyExists => write!(f, "already exists."),
            Error::NotADirectory => write!(f, "not a directory."),
            Error::IsADirectory => write!(f, "is a directory."),
            Error::SymlinkLoop => write!(f, "symlink loop detected."),
        }
    }
}