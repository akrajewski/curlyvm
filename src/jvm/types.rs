use std::ops::{Add, Neg};

pub const NULL_REF: JTypeValue = JTypeValue::Ref(0);

#[derive(Debug, Copy, Clone)]
pub enum JTypeValue {
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    Ref(usize),
    RetAddr(u32),

    // Dummy value, used for padding in local variable array, see https://docs.oracle.com/javase/specs/jvms/se7/html/jvms-2.html#jvms-2.6.1
    // "A value of type long or type double occupies two consecutive local variables. "
    Empty,
}

impl From<i32> for JTypeValue {
    fn from(x: i32) -> Self {
        return JTypeValue::Int(x);
    }
}

impl Add<JTypeValue> for JTypeValue {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {

        // this function features panics but they are not expected to happen
        // since Java compilation guarantees that the types will be correct
        match self {
            Self::Int(a) => match rhs {
                Self::Int(b) => Self::Int(a + b),
                _ => panic!("unsupported operation: adding int to non-int")
            },
            Self::Long(a) => match rhs {
                Self::Long(b) => Self::Long(a + b),
                _ => panic!("unsupported operation: adding long to non-long")
            },
            Self::Float(a) => match rhs {
                Self::Float(b) => Self::Float(a + b),
                _ => panic!("unsupported operation: adding float to non-float"),
            },
            Self::Double(a) => match rhs {
                Self::Double(b) => Self::Double(a + b),
                _ => panic!("unsuppported operation: adding double to non-double"),
            }

            _ => panic!("unsupported operation!")
        }
    }
}

impl Neg for JTypeValue {
    type Output = Self;

    fn neg(self) -> Self::Output {
        match self {
            Self::Int(a) => Self::Int(-a),
            Self::Long(a) => Self::Long(-a),
            Self::Float(a) => Self::Float(-a),
            Self::Double(a) => Self::Double(-a),
            _ => panic!("unsupported operation: cannot neg {:?}!", self)
        }
    }
}

impl PartialEq for JTypeValue {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Self::Int(a) => match other {
                Self::Int(b) => a == b,
                _ => panic!("unsupported operation, comparing int with non-int"),
            },
            Self::Long(a) => match other {
                Self::Long(b) => a == b,
                _ => panic!("unsupported operation: comparing long with non-long")
            },
            Self::Float(a) => match other {
                Self::Float(b) => a == b,
                _ => panic!("unsupported operation: comparing float with non-float"),
            },
            Self::Double(a) => match other {
                Self::Double(b) => a == b,
                _ => panic!("unsuppported operation: comparing double to non-double"),
            },
            Self::Ref(a) => match other {
                Self::Ref(b) => a == b,
                _ => panic!("unsuppported operation: comparing ref to non-ref"),
            },
            _ => panic!("unsupported operation")
        }
    }
}

