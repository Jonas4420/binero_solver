use std::collections::HashMap;
use std::fmt;
use std::ops;

use crate::cell::*;
use crate::error::GridError;

type Histogram = HashMap<Cell, usize>;
type GridCell = Option<Cell>;

#[derive(Debug)]
pub struct Grid {
    cells: Vec<Vec<GridCell>>,
    width: usize,
    height: usize,
    missing_lines: Vec<Histogram>,
    missing_columns: Vec<Histogram>,
}

impl Grid {
    pub fn parse<I, S, E>(lines: I) -> Result<Grid, GridError>
    where
        I: Iterator<Item = Result<S, E>>,
        S: AsRef<str>,
        GridError: From<E>,
    {
        let mut grid = Grid {
            cells: Vec::new(),
            height: 0,
            width: 0,
            missing_lines: Vec::new(),
            missing_columns: Vec::new(),
        };

        // Fill grid with parsed lines
        for line in lines {
            let vec = line?
                .as_ref()
                .chars()
                .take_while(|c| *c != '#')
                .filter(|c| !c.is_whitespace())
                .map(|c| match c {
                    '-' => Ok(None),
                    _ => Cell::try_from(c).map(Some),
                })
                .collect::<Result<Vec<_>, _>>()?;

            if !vec.is_empty() {
                if grid.cells.is_empty() {
                    // Set width of the grid
                    if (vec.len() % 2) != 0 {
                        return Err(GridError::OddDimension);
                    }

                    grid.width = vec.len();
                } else if vec.len() != grid.width {
                    return Err(GridError::WidthMismatch);
                }

                grid.cells.push(vec);
            }
        }

        // Set height of the grid
        grid.height = grid.cells.len();

        if grid.height == 0 {
            return Err(GridError::EmptyGrid);
        } else if (grid.height % 2) != 0 {
            return Err(GridError::OddDimension);
        }

        // Compute number of missing cells
        for i in 0..grid.height {
            grid.missing_lines
                .push(Self::fill_missings(grid.width, grid.line(i)));
        }

        for j in 0..grid.width {
            grid.missing_columns
                .push(Self::fill_missings(grid.height, grid.column(j)));
        }

        // Check if the grid is valid
        grid.is_valid()?;

        Ok(grid)
    }

    pub fn solve(&mut self) -> Result<(), GridError> {
        // Fill grid with constraints so it remains valid
        loop {
            // Fill cells that are constraints
            while self.fill_constraints() {}

            // Fill cell with heuristics
            if !self.fill_heuristics() {
                break;
            }
        }

        // Check if the grid is still valid
        self.is_valid()?;

        // Bruteforce remaining positions
        self.fill_bruteforce()?;

        Ok(())
    }

    fn is_solved(&self) -> bool {
        self.missing_lines
            .iter()
            .all(|missings| Cell::iter().all(|cell| missings[&cell] == 0))
    }

    fn is_valid(&self) -> Result<(), GridError> {
        // Check lines
        for i in 0..self.height {
            // Check lane
            let lane: Vec<_> = self.line(i).collect();
            Self::check_lane(&lane)?;

            // Check pair of lanes
            for i_pair in i + 1..self.height {
                Self::check_pair(lane.iter().copied().zip(self.line(i_pair)))?;
            }
        }

        // Check columns
        for j in 0..self.width {
            // Check lane
            let lane: Vec<_> = self.column(j).collect();
            Self::check_lane(&lane)?;

            // Check pair of lanes
            for j_pair in j + 1..self.width {
                Self::check_pair(lane.iter().copied().zip(self.column(j_pair)))?;
            }
        }

        Ok(())
    }

    fn fill_constraints(&mut self) -> bool {
        let mut changed = false;

        // Process lines
        for i in 0..self.height {
            for j in 0..self.width {
                if self[(i, j)].is_none() {
                    // If a line is already saturated, fill it with the opposite value
                    let new = Self::fill_saturated(&self.missing_lines[i])
                        .or_else(|| {
                            // Or check 2 previous cells
                            (j >= 2)
                                .then(|| Self::fill_cell(self[(i, j - 2)], self[(i, j - 1)]))
                                .flatten()
                        })
                        .or_else(|| {
                            // Or check 2 next cells
                            (j + 2 < self.width)
                                .then(|| Self::fill_cell(self[(i, j + 1)], self[(i, j + 2)]))
                                .flatten()
                        })
                        .or_else(|| {
                            // Or check 2 surrounding cells
                            (j >= 1 && j + 1 < self.width)
                                .then(|| Self::fill_cell(self[(i, j - 1)], self[(i, j + 1)]))
                                .flatten()
                        });

                    changed |= self.set(i, j, new);
                }
            }
        }

        // Process columns
        for j in 0..self.width {
            for i in 0..self.height {
                if self[(i, j)].is_none() {
                    // If a line is already saturated, fill it with the opposite value
                    let new = Self::fill_saturated(&self.missing_columns[j])
                        .or_else(|| {
                            // Or check 2 previous cells
                            (i >= 2)
                                .then(|| Self::fill_cell(self[(i - 2, j)], self[(i - 1, j)]))
                                .flatten()
                        })
                        .or_else(|| {
                            // Or check 2 next cells
                            (i + 2 < self.height)
                                .then(|| Self::fill_cell(self[(i + 1, j)], self[(i + 2, j)]))
                                .flatten()
                        })
                        .or_else(|| {
                            // Or check 2 surrounding cells
                            (i >= 1 && i + 1 < self.height)
                                .then(|| Self::fill_cell(self[(i - 1, j)], self[(i + 1, j)]))
                                .flatten()
                        });

                    changed |= self.set(i, j, new);
                }
            }
        }

        changed
    }

    fn fill_heuristics(&mut self) -> bool {
        let mut changed = false;

        // Process lines
        for i in 0..self.height {
            for missings in 1..=2 {
                // Check if a value is close to be filled, and is unbalanced with the other
                if let Some(cell) = Self::get_missings(&self.missing_lines[i], missings) {
                    for j in Self::try_missings(cell, self.line(i), missings) {
                        changed |= self.set(i, j, Some(!cell));
                    }
                }
            }
        }

        // Process columns
        for j in 0..self.width {
            for missings in 1..=2 {
                // Check if a value is close to be filled, and is unbalanced with the other
                if let Some(cell) = Self::get_missings(&self.missing_columns[j], missings) {
                    for i in Self::try_missings(cell, self.column(j), missings) {
                        changed |= self.set(i, j, Some(!cell));
                    }
                }
            }
        }

        changed
    }

    fn fill_bruteforce(&mut self) -> Result<(), GridError> {
        if self.is_solved() {
            return Ok(());
        }

        Ok(())
    }

    fn set(&mut self, i: usize, j: usize, new: GridCell) -> bool {
        let old = self[(i, j)];

        if let Some(old) = old {
            self.missing_lines[i].entry(old).and_modify(|e| *e += 1);
            self.missing_columns[j].entry(old).and_modify(|e| *e += 1);
        }

        if let Some(new) = new {
            self.missing_lines[i].entry(new).and_modify(|e| *e -= 1);
            self.missing_columns[j].entry(new).and_modify(|e| *e -= 1);
        }

        self.cells[i][j] = new;

        old != self.cells[i][j]
    }

    fn line(&self, i: usize) -> impl Iterator<Item = GridCell> + '_ {
        (0..self.width).map(move |j| self[(i, j)])
    }

    fn column(&self, j: usize) -> impl Iterator<Item = GridCell> + '_ {
        (0..self.height).map(move |i| self[(i, j)])
    }

    fn check_lane(lane: &[GridCell]) -> Result<(), GridError> {
        let size = lane.len();
        let mut map = Histogram::from_iter(Cell::iter().map(|cell| (cell, 0)));

        for i in 0..size {
            // Check if no more than 2 adjacent identical values
            if i + 2 < size {
                if let Some(cell) = lane[i] {
                    (lane[i] == lane[i + 1] && lane[i] == lane[i + 2])
                        .then(|| Err(GridError::InvalidGrid))
                        .unwrap_or(Ok(()))?;

                    *map.entry(cell).or_default() += 1;
                }
            }
        }

        // Check if lane is balanced
        if Cell::iter().any(|cell| map[&cell] > (size / 2)) {
            return Err(GridError::InvalidGrid);
        }

        Ok(())
    }

    fn check_pair<I>(mut pairs: I) -> Result<(), GridError>
    where
        I: Iterator<Item = (GridCell, GridCell)>,
    {
        pairs
            .any(|(lhs, rhs)| lhs.is_none() || lhs != rhs)
            .then(|| ())
            .ok_or(GridError::InvalidGrid)
    }

    fn fill_cell(cell0: GridCell, cell1: GridCell) -> GridCell {
        cell0
            .zip(cell1)
            .and_then(|(value0, value1)| (value0 == value1).then(|| !value0))
    }

    fn fill_saturated(missings: &Histogram) -> GridCell {
        Cell::iter().find(|cell| missings[cell] != 0 && missings[!cell] == 0)
    }

    fn fill_missings<I>(size: usize, lane: I) -> Histogram
    where
        I: Iterator<Item = GridCell>,
    {
        lane.fold(
            Histogram::from_iter(Cell::iter().map(|cell| (cell, size / 2))),
            |mut histogram, cell| {
                if let Some(cell) = cell {
                    histogram.entry(cell).and_modify(|e| *e -= 1);
                }
                histogram
            },
        )
    }

    fn get_missings(missings: &Histogram, offset: usize) -> GridCell {
        Cell::iter().find(|cell| missings[cell] == offset && missings[!cell] > missings[cell])
    }

    fn try_missings<I>(cell: Cell, lane: I, missings: usize) -> Vec<usize>
    where
        I: Iterator<Item = GridCell>,
    {
        let mut result = Vec::new();
        let mut none_idx = Vec::new();

        // Replace empty cells by opposite value, but keep track of indice
        let mut lane: Vec<_> = lane
            .enumerate()
            .map(|(idx, c)| {
                c.or_else(|| {
                    none_idx.push(idx);
                    Some(!cell)
                })
            })
            .collect();

        // For each empty place
        for i in none_idx.iter().copied() {
            // Try the tested value
            lane[i] = Some(cell);

            let is_possible = match missings {
                1 => Self::try_missings_one,
                2 => Self::try_missings_two,
                _ => Self::try_missings_none,
            };

            if !is_possible(&mut lane, cell, &none_idx, i) {
                result.push(i);
            }

            // Restore opposite value
            lane[i] = Some(!cell);
        }

        result
    }

    fn try_missings_none(_: &mut [GridCell], _: Cell, _: &[usize], _: usize) -> bool {
        true
    }

    fn try_missings_one(lane: &mut [GridCell], _: Cell, _: &[usize], _: usize) -> bool {
        Self::check_lane(lane).is_ok()
    }

    fn try_missings_two(lane: &mut [GridCell], cell: Cell, none_idx: &[usize], i: usize) -> bool {
        none_idx.iter().copied().filter(|j| i != *j).any(|j| {
            lane[j] = Some(cell);
            let is_valid = Self::check_lane(lane).is_ok();
            lane[j] = Some(!cell);
            is_valid
        })
    }
}

impl ops::Index<(usize, usize)> for Grid {
    type Output = GridCell;

    fn index(&self, (x, y): (usize, usize)) -> &Self::Output {
        &self.cells[x][y]
    }
}

impl fmt::Display for Grid {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        for i in 0..self.height {
            for j in 0..self.width {
                match self[(i, j)] {
                    Some(cell) => cell.fmt(fmt)?,
                    None => write!(fmt, "-")?,
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
