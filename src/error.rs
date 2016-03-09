use git2::Error as GitError;
use std::env::VarError;
use std::error::Error as StdError;
use std::io::Error as IoError;

use std::fmt;
use std::convert::From;

use self::Error::*;

#[derive(Debug)]
pub enum Error {
    Git(GitError),
    Var(VarError),
    Io(IoError),
}

impl From<GitError> for Error {
    fn from(err: GitError) -> Error {
        Error::Git(err)
    }
}

impl From<VarError> for Error {
    fn from(err: VarError) -> Error {
        Error::Var(err)
    }
}

impl From<IoError> for Error {
    fn from(err: IoError) -> Error {
        Error::Io(err)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Git(ref e) => e.fmt(f),
            Var(ref e) => e.fmt(f),
            Io(ref e) => e.fmt(f),
        }

    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            Git(ref e) => e.description(),
            Var(ref e) => e.description(),
            Io(ref e) => e.description(),
        }
    }
}
