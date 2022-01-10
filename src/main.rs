use std::env;
use std::fs;
use std::io;
use std::io::BufRead;

mod cell;
mod error;
mod grid;
mod index;

fn main() {
    try_main().unwrap_or_else(|err| {
        eprintln!("{}", err);
        std::process::exit(1);
    });
}

fn try_main() -> Result<(), Box<dyn std::error::Error>> {
    let args = env::args().collect::<Vec<String>>();

    if args.len() < 2 {
        return Err(format!("usage: {} <FILE>", args[0]).into());
    }

    let file = fs::File::open(&args[1]).map_err(|err| format!("{}: {}", args[1], err))?;
    let reader = io::BufReader::new(file);

    let mut grid = grid::Grid::parse(reader.lines())?;

    println!("Input grid:");
    println!("{}", grid);

    grid.solve()?;

    println!("Solution:");
    println!("{}", grid);

    Ok(())
}
