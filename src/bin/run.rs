use std::{fs::File, io::Write, path::Path};

use focus_third::{
    compiler::{Compiler, CompilerError},
    parser::ParserError,
    stdlib,
    vm::Vm,
};

#[derive(Debug)]
enum RunCliError {
    MissingInput,
    ReadWriteError(std::io::Error),
    FileError(std::io::Error),
    CompilerError(CompilerError),
}

impl From<CompilerError> for RunCliError {
    fn from(value: CompilerError) -> Self {
        Self::CompilerError(value)
    }
}

fn main() -> Result<(), RunCliError> {
    let Some(input_filename) = std::env::args().nth(1) else {
        eprintln!("Please provide a filename as the first argument.");
        return Err(RunCliError::MissingInput);
    };

    let source = std::io::read_to_string(
        File::open(input_filename.clone()).map_err(RunCliError::FileError)?,
    )
    .map_err(RunCliError::ReadWriteError)?;
    let mut compiler = Compiler::new(&source);
    compiler.add_module(stdlib::string::module());
    compiler.add_module(stdlib::io::module());
    compiler.add_module(stdlib::iter::module());
    let module = compiler.compile_module("<main>")?;

    let mut out = File::create(Path::new(&input_filename).with_extension("flb"))
        .map_err(RunCliError::FileError)?;
    module
        .dump(&mut out)
        .map_err(|e| RunCliError::ReadWriteError(e))?;

    let mut interpreter = Vm::new().with_modules(vec![
        stdlib::string::module(),
        stdlib::io::module(),
        stdlib::iter::module(),
    ]);
    interpreter.execute_module(module, "main");
    let last_value = interpreter.stack().last().unwrap();
    println!("{last_value}");

    Ok(())
}
