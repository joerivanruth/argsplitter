#![allow(dead_code, unused_imports, unused_import_braces)]

use std::process::{self, ExitCode};

use anyhow::anyhow;
use argsplitter::{
    main_support::{self, report_errors},
    ArgSplitter,
};

const USAGE: &str = r###"
Usage: anyhow <ARGS...>
"###;

fn main() -> ExitCode {
    // report_errors also works for anyhow::Error
    let ret: Result<(), anyhow::Error> = my_main();
    report_errors(USAGE, ret)
}

fn my_main() -> anyhow::Result<()> {
    let mut argsplitter = ArgSplitter::new();

    if let Some(a) = argsplitter.item_os()? {
        Err(a.unexpected())?
    } else {
        Err(anyhow!("inner error").context("outer error"))
    }
}
