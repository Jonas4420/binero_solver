use std::error;
use std::fmt;

#[derive(Debug)]
pub enum GridError {
    AdjacentCells,
    SameLanes,
    EmptyGrid,
    InvalidChar(char),
    WidthMismatch(usize, usize),
}

impl fmt::Display for GridError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        // TODO
        write!(fmt, "")
    }
}

impl error::Error for GridError {}
