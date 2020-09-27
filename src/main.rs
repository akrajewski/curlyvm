use anyhow::Result;
use curlyvm;


fn main() -> Result<()> {
    let mut jvm = curlyvm::jvm::JVM::new()?;
    let v = jvm.run_method("Add", "subtract", &[2, 3])?;
    println!("Got result: {:?}", v);

    Ok(())
}
