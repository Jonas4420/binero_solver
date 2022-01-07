use std::env;
use std::fs;
use std::io;
use std::io::BufRead;

mod error;
mod grid;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = env::args().collect::<Vec<String>>();
    let file = fs::File::open(&args[1])?;
    let reader = io::BufReader::new(file);

    let grid = grid::Grid::parse(reader.lines())?;
    println!("{}", grid);

    Ok(())
}
