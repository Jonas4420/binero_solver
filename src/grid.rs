use std::collections::HashMap;
use std::fmt;
use std::ops;

use crate::cell::*;
use crate::error::GridError;
use crate::index::*;

type Histogram = HashMap<Cell, usize>;
type GridCell = Option<Cell>;

#[derive(Clone, Debug)]
pub struct Grid {
    cells: Vec<Vec<GridCell>>,
    width: usize,
    height: usize,
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

        // Check if the grid is valid
        grid.is_valid()?;

        Ok(grid)
    }

    pub fn solve(&mut self) -> Result<(), GridError> {
        loop {
            loop {
                // Fill grid with constraints
                if !self.fill_constraints() {
                    break;
                }
            }

            // Fill grid with heuristics
            if !self.fill_heuristics() {
                break;
            }
        }

        // Check that grid is still valid
        self.is_valid()?;

        // Bruteforce remaining empty cells
        self.get_empty()
            .map(|idx| self.fill_bruteforce(idx))
            .unwrap_or(Ok(()))
    }

    pub fn is_valid(&self) -> Result<(), GridError> {
        for i in self.lines() {
            // Check lane
            let lane: Vec<_> = self.line(i).collect();
            Self::check_lane(&lane)?;

            // Check pair of lanes
            for i_pair in i + 1..self.height {
                Self::check_pair(lane.iter().copied().zip(self.line(i_pair)))?;
            }
        }

        for j in self.columns() {
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

    fn get_empty(&self) -> Option<Index> {
        self.lines()
            .find_map(|i| (0..self.width).find_map(|j| self[(i, j)].is_none().then(|| Index(i, j))))
    }

    fn fill_constraints(&mut self) -> bool {
        let mut changed = false;

        // Process lines
        for i in self.lines() {
            let saturated = Self::fill_saturated(self.line(i));

            for j in self.columns() {
                let idx = Index(i, j);

                if self[idx].is_none() {
                    // If a line is already saturated, fill it with the opposite value
                    let new = saturated
                        .or_else(|| {
                            // Or check 2 previous cells
                            (j >= 2)
                                .then(|| Self::fill_cell(self[idx.col(-2)], self[idx.col(-1)]))
                                .flatten()
                        })
                        .or_else(|| {
                            // Or check 2 next cells
                            (j + 2 < self.width)
                                .then(|| Self::fill_cell(self[idx.col(1)], self[idx.col(2)]))
                                .flatten()
                        })
                        .or_else(|| {
                            // Or check 2 surrounding cells
                            (j >= 1 && j + 1 < self.width)
                                .then(|| Self::fill_cell(self[idx.col(-1)], self[idx.col(1)]))
                                .flatten()
                        });

                    changed |= self.set(idx, new);
                }
            }
        }

        // Process columns
        for j in self.columns() {
            let saturated = Self::fill_saturated(self.column(j));

            for i in self.lines() {
                let idx = Index(i, j);

                if self[idx].is_none() {
                    // If a line is already saturated, fill it with the opposite value
                    let new = saturated
                        .or_else(|| {
                            // Or check 2 previous cells
                            (i >= 2)
                                .then(|| Self::fill_cell(self[idx.line(-2)], self[idx.line(-1)]))
                                .flatten()
                        })
                        .or_else(|| {
                            // Or check 2 next cells
                            (i + 2 < self.height)
                                .then(|| Self::fill_cell(self[idx.line(1)], self[idx.line(2)]))
                                .flatten()
                        })
                        .or_else(|| {
                            // Or check 2 surrounding cells
                            (i >= 1 && i + 1 < self.height)
                                .then(|| Self::fill_cell(self[idx.line(-1)], self[idx.line(1)]))
                                .flatten()
                        });

                    changed |= self.set(idx, new);
                }
            }
        }

        changed
    }

    fn fill_heuristics(&mut self) -> bool {
        let mut changed = false;

        // Process lines
        for i in self.lines() {
            // Check if a value is close to be filled, and is unbalanced with the other
            for (j, cell) in Self::try_missings(self.line(i)) {
                changed |= self.set((i, j), cell);
            }
        }

        // Process columns
        for j in self.columns() {
            // Check if a value is close to be filled, and is unbalanced with the other
            for (i, cell) in Self::try_missings(self.column(j)) {
                changed |= self.set((i, j), cell);
            }
        }

        changed
    }

    fn fill_bruteforce(&mut self, idx: Index) -> Result<(), GridError> {
        for cell in Cell::iter() {
            let mut grid = self.clone();
            grid.set(idx, Some(cell));

            if grid.solve().is_ok() {
                *self = grid;
                return Ok(());
            }
        }

        Err(GridError::NoSolution)
    }

    fn set<I>(&mut self, idx: I, new: GridCell) -> bool
    where
        I: Into<Index>,
    {
        let idx = idx.into();
        let old = self[idx];

        self.cells[idx.0][idx.1] = new;

        old != new
    }

    fn lines(&self) -> impl Iterator<Item = usize> {
        0..self.height
    }

    fn columns(&self) -> impl Iterator<Item = usize> {
        0..self.width
    }

    fn line(&self, i: usize) -> impl Iterator<Item = GridCell> + '_ {
        self.columns().map(move |j| self[(i, j)])
    }

    fn column(&self, j: usize) -> impl Iterator<Item = GridCell> + '_ {
        self.lines().map(move |i| self[(i, j)])
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

    fn fill_saturated<I>(lane: I) -> GridCell
    where
        I: Iterator<Item = GridCell>,
    {
        Self::find_count(lane, |map, size, cell| {
            (map[&cell] >= size / 2).then(|| !cell)
        })
    }

    fn get_missings<I>(lane: I, offset: usize) -> GridCell
    where
        I: Iterator<Item = GridCell>,
    {
        Self::find_count(lane, |map, size, cell| {
            (map[&cell] > map[&!cell] && map[&cell] + offset == (size / 2)).then(|| cell)
        })
    }

    fn find_count<I, F>(lane: I, f: F) -> GridCell
    where
        I: Iterator<Item = GridCell>,
        F: Fn(&Histogram, usize, Cell) -> GridCell,
    {
        let mut map = Histogram::from_iter(Cell::iter().map(|cell| (cell, 0)));
        let size = lane.fold(0, |size, cell| {
            if let Some(cell) = cell {
                *map.entry(cell).or_default() += 1;
            }

            size + 1
        });

        Cell::iter().find_map(|cell| f(&map, size, cell))
    }

    fn try_missings<I>(lane: I) -> HashMap<usize, GridCell>
    where
        I: Iterator<Item = GridCell>,
    {
        let mut result = HashMap::new();
        let lane: Vec<GridCell> = lane.collect();

        for missings in 1..3 {
            let mut none_idx = Vec::new();

            // TODO
            if let Some(cell) = Self::get_missings(lane.iter().copied(), missings) {
                // Replace empty cells by opposite value, but keep track of indice
                let mut lane: Vec<_> = lane
                    .iter()
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

                    let is_possible = if missings == 1 {
                        Self::check_lane(&lane).is_ok()
                    } else {
                        none_idx.iter().copied().filter(|j| i != *j).any(|j| {
                            lane[j] = Some(cell);
                            let is_possible = Self::check_lane(&lane).is_ok();
                            lane[j] = Some(!cell);
                            is_possible
                        })
                    };

                    if !is_possible {
                        result.insert(i, Some(!cell));
                    }

                    // Restore opposite value
                    lane[i] = Some(!cell);
                }
            }
        }

        result
    }
}

impl<I> ops::Index<I> for Grid
where
    I: Into<Index>,
{
    type Output = GridCell;

    fn index(&self, idx: I) -> &Self::Output {
        let idx = idx.into();
        &self.cells[idx.0][idx.1]
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
