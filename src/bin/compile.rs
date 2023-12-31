use std::{fs::File, path::Path};

use focus_lang::{compiler::CompilerError, state::ModuleLoader, stdlib};

#[derive(Debug)]
enum CompileCliError {
    MissingInput,
    ReadWriteError(std::io::Error),
    FileError(std::io::Error),
    CompilerError(CompilerError),
}

impl From<CompilerError> for CompileCliError {
    fn from(value: CompilerError) -> Self {
        Self::CompilerError(value)
    }
}

fn main() -> Result<(), CompileCliError> {
    let Some(input_filename) = std::env::args().nth(1) else {
        eprintln!("Please provide a filename as the first argument.");
        return Err(CompileCliError::MissingInput);
    };

    let mut module_loader = ModuleLoader::new("");
    module_loader.add_modules(stdlib::modules());
    module_loader.load_module(&input_filename);

    let mut out = File::create(Path::new(&input_filename).with_extension("flb"))
        .map_err(CompileCliError::FileError)?;
    module_loader
        .module_at(3)
        .unwrap()
        .dump(&mut out)
        .map_err(|e| CompileCliError::ReadWriteError(e))?;

    Ok(())
}
