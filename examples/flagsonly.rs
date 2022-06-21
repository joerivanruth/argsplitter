use std::{error::Error, path::PathBuf};

use argsplitter::{ArgSplitter, ItemOs};

fn main() -> Result<(), Box<dyn Error>> {
    let mut verbose = false;
    let mut files: Vec<PathBuf> = vec![];
    let mut mode = "default".to_string();

    let mut args = ArgSplitter::new();

    use ItemOs::*;
    while let Some(item) = args.item_os()? {
        match item {
            Word(w) => files.push(w.into()),
            Flag("-v" | "--verbose") => verbose = true,
            Flag("-m" | "--mode") => mode = args.param()?,
            a => return Err(a.unexpected())?,
        }
    }

    println!("Verbose={verbose:?} mode={mode:?} files={files:?}");
    Ok(())
}
