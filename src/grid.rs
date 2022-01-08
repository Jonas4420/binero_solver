use std::collections::HashMap;
use std::fmt;
use std::ops;

use crate::cell::*;
use crate::error::GridError;

type Histogram = HashMap<Cell, usize>;

#[derive(Debug)]
pub struct Grid {
    cells: Vec<Vec<Cell>>,
    histogram_lines: Vec<Histogram>,
    histogram_columns: Vec<Histogram>,
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
            cells: Default::default(),
            histogram_lines: Vec::new(),
            histogram_columns: Vec::new(),
            height: 0,
            width: 0,
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
            let mut histogram = HashMap::from_iter(Cell::iter().map(|cell| (cell, 0)));

            for j in 0..grid.width {
                *histogram.entry(grid[(i, j)]).or_insert(0) += 1;
            }

            grid.histogram_lines.push(histogram);
        }

        for j in 0..grid.width {
            let mut histogram = HashMap::from_iter(Cell::iter().map(|cell| (cell, 0)));

            for i in 0..grid.height {
                *histogram.entry(grid[(i, j)]).or_insert(0) += 1;
            }

            grid.histogram_columns.push(histogram);
        }

        // Check if the grid is valid
        grid.is_valid()?;

        Ok(grid)
    }

    pub fn solve(&mut self) -> Result<(), GridError> {
        while self.fill_cells() {}

        // TODO

        self.is_valid()
    }

    fn is_filled(&self) -> bool {
        for i in 0..self.height {
            if (0..self.width).any(|j| self[(i, j)].is_none()) {
                return false;
            }
        }

        true
    }

    fn is_valid(&self) -> Result<(), GridError> {
        // No more than 2 consecutive values in a line
        for i in 0..self.height {
            for j in 0..self.width - 2 {
                Self::check_cells(self[(i, j)], self[(i, j + 1)], self[(i, j + 2)])?;
            }
        }

        // No more than 2 consecutive values in a column
        for j in 0..self.width {
            for i in 0..self.height - 2 {
                Self::check_cells(self[(i, j)], self[(i + 1, j)], self[(i + 2, j)])?;
            }
        }

        // Check that full lines are balanced
        for i in 0..self.height {
            Self::check_balance(&self.histogram_lines[i])?;
        }

        // Check that full columns are balanced
        for j in 0..self.width {
            Self::check_balance(&self.histogram_columns[j])?;
        }

        // Each line pairs are different
        for i0 in 0..self.height - 1 {
            for i1 in i0 + 1..self.height {
                Self::check_lanes((0..self.width).map(|j| (self[(i0, j)], self[(i1, j)])))?;
            }
        }

        // Each column pairs are different
        for j0 in 0..self.width - 1 {
            for j1 in j0 + 1..self.width {
                Self::check_lanes((0..self.height).map(|i| (self[(i, j0)], self[(i, j1)])))?;
            }
        }

        Ok(())
    }

    fn fill_cells(&mut self) -> bool {
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

    fn set(&mut self, x: usize, y: usize, new: Cell) {
        let old = self[(x, y)];

        self.histogram_lines[x].entry(old).and_modify(|e| *e -= 1);
        self.histogram_lines[x].entry(new).and_modify(|e| *e += 1);

        self.histogram_columns[y].entry(old).and_modify(|e| *e -= 1);
        self.histogram_columns[y].entry(new).and_modify(|e| *e += 1);

        self.cells[x][y] = new;
    }

    fn check_cells(cell0: Cell, cell1: Cell, cell2: Cell) -> Result<(), GridError> {
        if cell0.is_none() || cell0 != cell1 || cell0 != cell2 {
            Ok(())
        } else {
            Err(GridError::InvalidGrid)
        }
    }

    fn check_balance(histogram: &Histogram) -> Result<(), GridError> {
        Cell::iter_some()
            .find(|cell| histogram[cell] > histogram[!cell] + histogram[&Cell::None])
            .map_or(Ok(()), |_| Err(GridError::InvalidGrid))
    }

    fn check_lanes<I>(mut lanes: I) -> Result<(), GridError>
    where
        I: Iterator<Item = (Cell, Cell)>,
    {
        if lanes.any(|(cell0, cell1)| cell0.is_none() || cell0 != cell1) {
            Ok(())
        } else {
            Err(GridError::InvalidGrid)
        }
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
