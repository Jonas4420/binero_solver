use std::error;
use std::fmt;

#[derive(Debug)]
pub enum GridError {
    EmptyGrid,
    InvalidChar(char),
    InvalidGrid,
    NoSolution,
    OddDimension,
    WidthMismatch,
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
        }
    }
}

impl error::Error for GridError {}
