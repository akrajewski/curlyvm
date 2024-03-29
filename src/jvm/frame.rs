use crate::class::Class;
use std::rc::Rc;
use anyhow::{Result, anyhow};
use crate::jvm::JTypeValue;

#[derive(Debug)]
pub struct Frame {
    pub class: Rc<Class>,
    pub ip: usize,
    pub code: Vec<u8>,
    pub locals: Vec<JTypeValue>,
    pub operand_stack: Vec<JTypeValue>
}

impl Frame {
    pub fn new(class: Rc<Class>, code: Vec<u8>, locals: Vec<JTypeValue>) -> Self {
        Self {
            class,
            code,
            ip: 0,
            locals,
            operand_stack: Vec::new()
        }
    }

    pub fn pop_stack(&mut self) -> Result<JTypeValue> {
        match self.operand_stack.pop() {
            Some(v) => Ok(v),
            None => Err(anyhow!("tried popping stack but nothing found"))
        }
    }

    pub fn pop_int(&mut self) -> i32 {
        match self.operand_stack.pop() {
            Some(v) => match v {
                JTypeValue::Int(i) => i,
                _ => panic!("tried popping int but something else found")
            },
            None => panic!("tied popping int but nothing found")
        }
    }

    pub fn pop_ref(&mut self) -> usize {
        match self.operand_stack.pop() {
            Some(v) => match v {
                JTypeValue::Ref(r) => r,
                _ => panic!("tried popping ref but something else found")
            },
            None => panic!("tried popping ref but nothing found")
        }
    }

    pub fn push_stack(&mut self, v: JTypeValue) {
        if let JTypeValue::Empty = v {
            // NOOP - do not push empty values to stack
            return;
        }

        self.operand_stack.push(v );
    }

    pub fn inc_ip(&mut self, inc: usize) {
        self.ip += inc;
    }
}