use std::fmt;
use std::ops;

use crate::error::GridError;

#[derive(Debug)]
pub struct Grid {
    cells: Vec<Vec<Option<Cell>>>,
    width: usize,
    height: usize,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Cell {
    Zero,
    One,
}

impl Grid {
    pub fn parse<I, S, E>(lines: I) -> Result<Grid, GridError>
    where
        I: Iterator<Item = Result<S, E>>,
        S: AsRef<str>,
        E: std::error::Error,
    {
        let mut cells: Vec<Vec<_>> = Vec::new();

        for line in lines {
            // TODO: Return better error
            let line = line.unwrap();
            let mut vec = Vec::new();

            for c in line.as_ref().chars() {
                match c {
                    ' ' | '\t' => {}
                    '0' => vec.push(Some(Cell::Zero)),
                    '1' => vec.push(Some(Cell::One)),
                    '-' => vec.push(None),
                    _ => {
                        return Err(GridError::InvalidChar(c));
                    }
                };
            }

            if !vec.is_empty() {
                if let Some(prev) = cells.last() {
                    if vec.len() != prev.len() {
                        return Err(GridError::WidthMismatch(prev.len(), vec.len()));
                    }
                }

                cells.push(vec);
            }
        }

        if cells.is_empty() {
            return Err(GridError::EmptyGrid);
        }

        let height = cells.len();
        let width = cells[0].len();

        Ok(Grid { cells, height, width })
    }
}

impl fmt::Display for Grid {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        for i in 0..self.height {
            for j in 0..self.width {
                match &self.cells[i][j] {
                    Some(cell) => {
                        write!(fmt, "{}", cell)?;
                    }
                    None => {
                        write!(fmt, "-")?;
                    }
                }

                if j < self.width - 1 {
                    write!(fmt, " ")?;
                }
            }

            if i < self.height - 1 {
                writeln!(fmt)?;
            }
        }

        Ok(())
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

impl ops::Not for Cell {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Self::Zero => Self::One,
            Self::One => Self::Zero,
        }
    }
}
