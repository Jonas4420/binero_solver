use std::error;
use std::fmt;
use std::io;

#[derive(Debug)]
pub enum GridError {
    EmptyGrid,
    InvalidChar(char),
    InvalidGrid,
    NoSolution,
    OddDimension,
    WidthMismatch,
    IOError(io::Error),
}

impl fmt::Display for GridError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "error: ")?;

        match self {
            Self::EmptyGrid => {
                write!(fmt, "gris is empty")
            }
            Self::InvalidChar(c) => {
                write!(fmt, "unknown character '{}'", c)
            }
            Self::InvalidGrid => {
                write!(fmt, "grid is invalid")
            }
            Self::NoSolution => {
                write!(fmt, "grid has no solution")
            }
            Self::OddDimension => {
                write!(fmt, "grid has odd dimensions")
            }
            Self::WidthMismatch => {
                write!(fmt, "not all lines of the grid have the same length")
            }
            Self::IOError(err) => err.fmt(fmt),
        }
    }
}

impl error::Error for GridError {}

impl From<io::Error> for GridError {
    fn from(err: io::Error) -> Self {
        Self::IOError(err)
    }
}
