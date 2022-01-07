use std::error;
use std::fmt;
use std::io;

#[derive(Debug)]
pub enum GridError {
    AdjacentCells,
    SameLanes,
    EmptyGrid,
    LaneUnbalanced,
    InvalidChar(char),
    WidthMismatch(usize, usize),
    IOError(io::Error),
}

impl fmt::Display for GridError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "error: ")?;

        match self {
            Self::AdjacentCells => {
                write!(fmt, "3 consecutive cells have the same value")
            }
            Self::SameLanes => {
                write!(fmt, "2 columns or 2 lines have the same value")
            }
            Self::EmptyGrid => {
                write!(fmt, "parsed grid has no lines")
            }
            Self::LaneUnbalanced => {
                write!(fmt, "a line has unbalanced number of 0s and 1s")
            }
            Self::InvalidChar(c) => {
                write!(fmt, "unknown character '{}'", c)
            }
            Self::WidthMismatch(prev, curr) => {
                write!(fmt, "a line has width {}, but previous ones are {}", curr, prev)
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
