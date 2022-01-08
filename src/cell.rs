use std::fmt;
use std::ops;

use crate::error::GridError;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Cell {
    None,
    Zero,
    One,
}

impl Cell {
    pub fn or_else<F>(self, f: F) -> Self
    where
        F: FnOnce() -> Self,
    {
        self.or_else_if(true, f)
    }

    pub fn or_else_if<F>(self, cond: bool, f: F) -> Self
    where
        F: FnOnce() -> Self,
    {
        if self.is_some() || !cond {
            self
        } else {
            f()
        }
    }

    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }

    pub fn is_some(&self) -> bool {
        !self.is_none()
    }

    pub fn iter() -> impl Iterator<Item = Cell> {
        vec![Self::None, Self::Zero, Self::One].into_iter()
    }

    pub fn iter_some() -> impl Iterator<Item = Cell> {
        vec![Self::Zero, Self::One].into_iter()
    }
}

impl Default for Cell {
    fn default() -> Self {
        Cell::None
    }
}

impl ops::Not for Cell {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Self::None => Self::None,
            Self::Zero => Self::One,
            Self::One => Self::Zero,
        }
    }
}

impl ops::Not for &Cell {
    type Output = &'static Cell;

    fn not(self) -> Self::Output {
        match self {
            Cell::None => &Cell::None,
            Cell::Zero => &Cell::One,
            Cell::One => &Cell::Zero,
        }
    }
}

impl TryFrom<char> for Cell {
    type Error = GridError;

    fn try_from(c: char) -> Result<Self, Self::Error> {
        match c {
            '0' => Ok(Self::Zero),
            '1' => Ok(Self::One),
            '-' => Ok(Self::None),
            _ => Err(GridError::InvalidChar(c)),
        }
    }
}

impl fmt::Display for Cell {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Zero => write!(fmt, "0"),
            Self::One => write!(fmt, "1"),
            _ => write!(fmt, "-"),
        }
    }
}
