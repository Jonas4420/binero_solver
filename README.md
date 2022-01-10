# Binero

Binero are grid puzzles, in the same fashion as magic squares, or Sudoku.
This program attempts to solve such puzzles.

## Rules

A grid has an even number of lines and an even number of columns, usually forming a square.
Each cell can only have values `0` or `1`, and some will be already filled.

To solve a puzzle, one has to fill all the empty cells, with 3 constraints in mind:
- Each line and column should have a balanced amount of `0` and `1`,
- In an individual lane, there cannot be more than two consecutive similar digit,
- Any pair of line and any pair of column should be different.

## Example

Here is an 4x4 example grid:

| **1** | **1** |       | **0** |
| ----- | ----- | ----- | ----- |
|       | **0** |       |       |
|       |       | **0** |       |
|       | **1** |       | **0** |

And its solution:

| **1** | **1** | **0** | **0** |
| ----- | ----- | ----- | ----- |
| **0** | **0** | **1** | **1** |
| **1** | **0** | **0** | **1** |
| **0** | **1** |   1   | **0** |

## Input format

A grid has to be stored in a text file, with each line containing the digits for the line.

Cell values are using the characters `0` and `1`, and empty ones are encoded with the dash character (`-`).

There can be spaces between values, and empty lines are ignored. Lines starting with `#` are totally skipped, and can be treated as comments.
