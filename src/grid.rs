use std::collections::HashMap;
use std::fmt;
use std::ops;

use crate::cell::*;
use crate::error::GridError;

type Histogram = HashMap<Cell, usize>;

#[derive(Debug)]
pub struct Grid {
    cells: Vec<Vec<Cell>>,
    width: usize,
    height: usize,
    histogram_lines: Vec<Histogram>,
    histogram_columns: Vec<Histogram>,
    num_empty: usize,
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
            histogram_lines: Vec::new(),
            histogram_columns: Vec::new(),
            num_empty: 0,
        };

        // Fill grid with parsed lines
        for line in lines {
            let vec = line?
                .as_ref()
                .chars()
                .filter(|c| !c.is_whitespace())
                .map(Cell::try_from)
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

        // Compute histograms
        for i in 0..grid.height {
            grid.histogram_lines
                .push(Self::fill_histogram(grid.line(i)));
        }

        for j in 0..grid.width {
            grid.histogram_columns
                .push(Self::fill_histogram(grid.column(j)));
        }

        // Compute empty cells
        grid.num_empty = grid
            .histogram_lines
            .iter()
            .fold(0, |acc, histogram| acc + histogram[&Cell::None]);

        // Check if the grid is valid
        grid.is_valid()?;

        Ok(grid)
    }

    pub fn solve(&mut self) -> Result<(), GridError> {
        loop {
            // Fill cells that are constraints
            while self.fill_constraints() {}

            // Fill cell with heuristics
            if !self.fill_heuristics() {
                break;
            }
        }

        // TODO: brute force

        self.is_valid()
    }

    fn is_valid(&self) -> Result<(), GridError> {
        // Check lines
        for i in 0..self.height {
            // Check lane
            let lane: Vec<_> = self.line(i).collect();
            Self::check_lane(&lane)?;

            // Check pair of lanes
            for i_pair in i + 1..self.height {
                Self::check_pair(lane.iter().zip(self.line(i_pair)))?;
            }
        }

        // Check columns
        for j in 0..self.width {
            // Check lane
            let lane: Vec<_> = self.column(j).collect();
            Self::check_lane(&lane)?;

            // Check pair of lanes
            for j_pair in j + 1..self.width {
                Self::check_pair(lane.iter().zip(self.column(j_pair)))?;
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
                    let new = self[(i, j)]
                        .or_else(|| {
                            // If a line is already saturated, fill it with the opposite value
                            Self::fill_saturated(&self.histogram_lines[i])
                        })
                        .or_else_if(j >= 2, || {
                            // Or check 2 previous cells
                            Self::fill_cell(self[(i, j - 2)], self[(i, j - 1)])
                        })
                        .or_else_if(j + 2 < self.width, || {
                            // Or check 2 next cells
                            Self::fill_cell(self[(i, j + 1)], self[(i, j + 2)])
                        })
                        .or_else_if(j >= 1 && j + 1 < self.width, || {
                            // Or check 2 surrounding cells
                            Self::fill_cell(self[(i, j - 1)], self[(i, j + 1)])
                        });

                    self.set(i, j, new);

                    changed |= self[(i, j)].is_some();
                }
            }
        }

        // Process columns
        for j in 0..self.width {
            for i in 0..self.height {
                if self[(i, j)].is_none() {
                    let new = self[(i, j)]
                        .or_else(|| {
                            // If a line is already saturated, fill it with the opposite value
                            Self::fill_saturated(&self.histogram_columns[j])
                        })
                        .or_else_if(i >= 2, || {
                            // Or check 2 previous cells
                            Self::fill_cell(self[(i - 2, j)], self[(i - 1, j)])
                        })
                        .or_else_if(i + 2 < self.height, || {
                            // Or check 2 next cells
                            Self::fill_cell(self[(i + 1, j)], self[(i + 2, j)])
                        })
                        .or_else_if(i >= 1 && i + 1 < self.height, || {
                            // Or check 2 surrounding cells
                            Self::fill_cell(self[(i - 1, j)], self[(i + 1, j)])
                        });

                    self.set(i, j, new);

                    changed |= self[(i, j)].is_some();
                }
            }
        }

        changed
    }

    fn fill_heuristics(&mut self) -> bool {
        let mut changed = false;

        // Process lines
        for i in 0..self.height {
            // Check if a value is close to be filled, and is unbalanced with the other
            if let Some(cell) = Self::get_one_missing(&self.histogram_lines[i], self.width / 2) {
                let lane = self.line(i);

                // Get positions where it cannot be set
                for j in Self::try_one_missing(cell, lane) {
                    self.set(i, j, !cell);
                    changed |= true;
                }
            }
        }

        // Process columns
        for j in 0..self.width {
            // Check if a value is close to be filled, and is unbalanced with the other
            if let Some(cell) = Self::get_one_missing(&self.histogram_columns[j], self.height / 2) {
                let lane = self.column(j);

                // Get positions where it cannot be set
                for i in Self::try_one_missing(cell, lane) {
                    self.set(i, j, !cell);
                    changed |= true;
                }
            }
        }

        changed
    }

    fn set(&mut self, i: usize, j: usize, new: Cell) {
        let old = self[(i, j)];

        self.histogram_lines[i].entry(old).and_modify(|e| *e -= 1);
        self.histogram_lines[i].entry(new).and_modify(|e| *e += 1);

        self.histogram_columns[j].entry(old).and_modify(|e| *e -= 1);
        self.histogram_columns[j].entry(new).and_modify(|e| *e += 1);

        if old.is_none() && new.is_some() {
            self.num_empty -= 1;
        }

        self.cells[i][j] = new;
    }

    fn line(&self, i: usize) -> impl Iterator<Item = Cell> + '_ {
        (0..self.width).map(move |j| self[(i, j)])
    }

    fn column(&self, j: usize) -> impl Iterator<Item = Cell> + '_ {
        (0..self.height).map(move |i| self[(i, j)])
    }

    fn check_lane(lane: &[Cell]) -> Result<(), GridError> {
        let size = lane.len();
        let mut map = Histogram::from_iter(Cell::iter_some().map(|cell| (cell, 0)));

        for i in 0..size {
            // Check if no more than 2 adjacent identical values
            if i + 2 < size && lane[i].is_some() {
                (lane[i] == lane[i + 1] && lane[i] == lane[i + 2])
                    .then(|| Err(GridError::InvalidGrid))
                    .unwrap_or(Ok(()))?;
            }

            *map.entry(lane[i]).or_default() += 1;
        }

        // Check if lane is balanced
        if map[&Cell::Zero] > (size / 2) || map[&Cell::One] > (size / 2) {
            return Err(GridError::InvalidGrid);
        }

        Ok(())
    }

    fn check_pair<I, S, T>(mut pairs: I) -> Result<(), GridError>
    where
        I: Iterator<Item = (S, T)>,
        S: AsRef<Cell>,
        T: AsRef<Cell>,
    {
        pairs
            .any(|(lhs, rhs)| {
                let lhs = lhs.as_ref();
                let rhs = rhs.as_ref();
                lhs.is_none() || lhs != rhs
            })
            .then(|| ())
            .ok_or(GridError::InvalidGrid)
    }

    fn fill_cell(cell0: Cell, cell1: Cell) -> Cell {
        if cell0.is_some() && cell0 == cell1 {
            !cell0
        } else {
            Cell::None
        }
    }

    fn fill_saturated(histogram: &Histogram) -> Cell {
        Cell::iter_some()
            .find(|cell| histogram[cell] >= (histogram[!cell] + histogram[&Cell::None]))
            .map_or(Default::default(), |cell| !cell)
    }

    fn fill_histogram<I>(lane: I) -> Histogram
    where
        I: Iterator<Item = Cell>,
    {
        lane.fold(
            HashMap::from_iter(Cell::iter().map(|cell| (cell, 0))),
            |mut histogram, item| {
                *histogram.entry(item).or_default() += 1;
                histogram
            },
        )
    }

    fn get_one_missing(histogram: &Histogram, half: usize) -> Option<Cell> {
        Cell::iter_some()
            .find(|cell| histogram[cell] == half - 1 && histogram[cell] > histogram[!cell])
    }

    fn try_one_missing<I>(cell: Cell, lane: I) -> Vec<usize>
    where
        I: Iterator<Item = Cell>,
    {
        let mut result = Vec::new();
        let mut none_idx = Vec::new();

        // Replace empty cells by opposite value, but keep track of indice
        let mut lane: Vec<_> = lane
            .enumerate()
            .map(|(idx, c)| match c {
                Cell::None => {
                    none_idx.push(idx);
                    !cell
                }
                _ => c,
            })
            .collect();

        // For each empty place
        for i in none_idx {
            // Try the tested value
            lane[i] = cell;

            if Self::check_lane(&lane).is_err() {
                result.push(i);
            }

            // Restore opposite value
            lane[i] = !cell;
        }

        result
    }
}

impl ops::Index<(usize, usize)> for Grid {
    type Output = Cell;

    fn index(&self, (x, y): (usize, usize)) -> &Self::Output {
        &self.cells[x][y]
    }
}

impl fmt::Display for Grid {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        for i in 0..self.height {
            for j in 0..self.width {
                write!(fmt, "{}", self[(i, j)])?;

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
