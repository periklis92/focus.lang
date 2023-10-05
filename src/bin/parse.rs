use std::fs::File;

use focus_third::parser::{Parser, ParserError};

#[derive(Debug)]
enum ParseCliError {
    MissingInput,
    FileError(std::io::Error),
    ReadWriteError(std::io::Error),
    ErrorWhileParsing(ParserError),
}

fn main() -> Result<(), ParseCliError> {
    let Some(input_filename) = std::env::args().nth(1) else {
        eprintln!("Please provide a filename as the first argument.");
        return Err(ParseCliError::MissingInput);
    };

    let source =
        std::io::read_to_string(File::open(input_filename).map_err(ParseCliError::FileError)?)
            .map_err(ParseCliError::ReadWriteError)?;

    let mut parser = Parser::new(&source);
    let mut tree = Vec::new();

    loop {
        match parser.parse() {
            Ok(expr) => tree.push(expr),
            Err(ParserError::EndOfSource) => break,
            Err(e) => return Err(ParseCliError::ErrorWhileParsing(e)),
        }
    }

    let mut f = File::create("out.txt").map_err(ParseCliError::FileError)?;

    Ok(())
}
