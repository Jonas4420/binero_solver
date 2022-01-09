use std::fmt;
use std::ops;

use crate::error::GridError;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Cell {
    Zero,
    One,
}

impl Cell {
    pub fn iter() -> impl Iterator<Item = Cell> {
        vec![Self::Zero, Self::One].into_iter()
    }
}

impl ops::Not for &Cell {
    type Output = &'static Cell;

    fn not(self) -> Self::Output {
        match self {
            Cell::Zero => &Cell::One,
            Cell::One => &Cell::Zero,
        }
    }
}

impl ops::Not for Cell {
    type Output = Self;

    fn not(self) -> Self::Output {
        *(!&self)
    }
}

impl TryFrom<char> for Cell {
    type Error = GridError;

    fn try_from(c: char) -> Result<Self, Self::Error> {
        match c {
            '0' => Ok(Self::Zero),
            '1' => Ok(Self::One),
            _ => Err(GridError::InvalidChar(c)),
        }
    }
}

impl fmt::Display for Cell {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Zero => write!(fmt, "0"),
            Self::One => write!(fmt, "1"),
        }
    }
}
