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
        // TODO
        write!(fmt, "")
    }
}

impl error::Error for GridError {}

impl From<io::Error> for GridError {
    fn from(err: io::Error) -> Self {
        Self::IOError(err)
    }
}
