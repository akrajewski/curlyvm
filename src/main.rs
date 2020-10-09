use anyhow::Result;
use curlyvm::jvm::JTypeValue;


fn main() -> Result<()> {
    let mut jvm = curlyvm::jvm::JVM::new()?;
    let v = jvm.run("Add", "doubleAdd", &[JTypeValue::Double(2.0), JTypeValue::Double(3.0)])?;
    println!("Got result: {:?}", v);

    Ok(())
}
