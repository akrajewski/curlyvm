use std::error::Error;
use std::io::prelude::*;
use std::fs::File;
use std::str;

use anyhow::Result;
use curlyvm;


fn main() -> Result<()> {
    let c = curlyvm::class::load("java/Add.class")?;
    println!("loaded class: {:?}", c);
    Ok(())
}
