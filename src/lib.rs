pub mod class;
pub mod jvm;

#[cfg(test)]
mod tests {

    use anyhow::Result;
    use crate::jvm::JTypeValue;
    use crate::jvm::JVM;

    #[test]
    fn it_works() -> Result<()> {

        let mut jvm = JVM::new()?;
        let v = jvm.run("Add", "addMany",
                    &[JTypeValue::Int(1),JTypeValue::Int(1),JTypeValue::Int(1),JTypeValue::Int(1),JTypeValue::Int(1),JTypeValue::Int(1)])?;

        match v {
            JTypeValue::Int(i) => assert_eq!(6, i),
            _ => assert!(false)
        }

        Ok(())
    }
}