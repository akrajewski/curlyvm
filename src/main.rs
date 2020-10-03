use anyhow::Result;


fn main() -> Result<()> {
    let mut jvm = curlyvm::jvm::JVM::new()?;
    let v = jvm.run("Add", "subtract", &[2, 3])?;
    println!("Got result: {:?}", v);

    Ok(())
}
