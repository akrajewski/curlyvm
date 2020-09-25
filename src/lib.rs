pub mod class;
pub mod jvm;

#[cfg(test)]
mod tests {
    use std::rc::Rc;
    use std::borrow::Borrow;

    struct Test {
        rc: Rc<str>
    }

    #[test]
    fn test_rc_str() {
    }

    #[test]
    fn it_works() {
        let text = String::from("Haha");
        let rc: Rc<str> = text.into();
        assert!(rc == "Haha".into());
    }
}