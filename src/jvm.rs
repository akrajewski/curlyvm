use crate::class::Class;
use anyhow::{Result, anyhow};

pub struct Frame<'a> {
    class: &'a Class,
    ip: u32,
    code: &'a [u8],
    locals: Vec<i32>,
    stack: Vec<i32>
}

impl Frame<'_> {
    pub fn pop_stack(&mut self) -> Result<i32> {
        match self.stack.pop() {
            Some(v) => Ok(v),
            None => Err(anyhow!("tried popping stack but nothing found!"))
        }
    }
}

pub fn run_method<'a>(class: &Class, method_name: &str, args: &[&str]) -> Result<i32> {
    let method = match class.methods.iter().find(|&m|  m.name == method_name.into()) {
        Some(m) => m,
        None => return Err(anyhow!("no such method"))
    };

    let code = match method.attributes.iter().find(|&a| a.name == "Code".into()) {
        Some(c) => c,
        None => return Err(anyhow!("'code' attribute not found!"))
    };

    let _max_locals = u16::from_be_bytes([code.data[2],code.data[3]]) as usize;

    let mut locals = Vec::<i32>::new();
    for a in args.iter() {
        locals.push(a.parse()?);
    }

    let mut frame = Frame {
        class,
        code: &code.data[8..],
        ip: 0,
        locals,
        stack: Vec::new()
    };

    run(&mut frame)
}

fn run(f: &mut Frame) -> Result<i32> {
    loop {
        let op = f.code[f.ip as usize];

        println!("OP: {}, stack: {:?}", op, f.stack);

        match op {
            26 => { // iload_0
                f.stack.push(f.locals[0]);
            },
            27 => { // iload_1
                f.stack.push(f.locals[1]);
            },
            96 => {
                let a = f.pop_stack()?;
                let b = f.pop_stack()?;
                f.stack.push(a + b);
            },
            172 => { // ireturn
                return f.pop_stack()
            },
            _ => {
                println!("unknown opcode {}", op);
            }
        }

        f.ip += 1;
    }
}

