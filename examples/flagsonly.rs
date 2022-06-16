use std::{error::Error, path::PathBuf};

use argsplitter::{ItemOs, Splitter};

fn main() -> Result<(), Box<dyn Error>> {
    let mut verbose = false;
    let mut files: Vec<PathBuf> = vec![];
    let mut mode = "default".to_string();

    let mut args = Splitter::new();

    use ItemOs::*;
    while let Some(item) = args.item_os()? {
        match item {
            Word(w) => files.push(w.into()),
            Flag("-v" | "--verbose") => verbose = true,
            Flag("-m" | "--mode") => mode = args.param()?,
            a => a.unexpected()?,
        }
    }

    println!("Verbose={verbose:?} mode={mode:?} files={files:?}");
    Ok(())
}
