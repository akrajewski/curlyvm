use anyhow::Result;
use curlyvm;


fn main() -> Result<()> {
    let c = curlyvm::class::load("java/Add.class")?;

    println!("loaded class: {:?}", c);

    let v = curlyvm::jvm::run_method(&c, "add", &["2", "3"])?;

    println!("Got result: {}", v);

    Ok(())
}
