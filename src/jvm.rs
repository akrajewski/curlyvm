use crate::class::{Class};
use std::ops::Deref;
use std::collections::HashMap;

use anyhow::{Result, anyhow};
use std::rc::Rc;

#[derive(Debug)]
pub enum JTypeValue {
    Bool(bool),
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i32),
    Char(u16),
    RetAddr(u32),
}

pub struct JVM {
    thread: JThread,
    _heap: Heap,
    method_area: Rc<MethodArea>
}

impl JVM {
    pub fn new() -> Result<Self> {
        let mut method_area = MethodArea::new();
        let class = crate::class::load("java/Add.class")?;
        println!("class: {:?}", class);
        method_area.classes.insert(class.name.clone(), Rc::new(class));

        let method_area = Rc::new(method_area);
        let thread = JThread::new(method_area.clone());

        Ok(Self { method_area, thread, _heap: Heap })
    }

    pub fn run(&mut self, class_name: &str, method_name: &str, args: &[i32]) -> Result<i32> {
        let result = self.thread.execute_method(class_name, method_name, args)?;
        match result {
            JTypeValue::Int(v) => Ok(v),
            _ => panic!("unsupported result type")
        }
    }
}

struct Heap;

#[derive(Debug)]
struct MethodArea {
    classes: HashMap<Rc<str>, Rc<Class>>
}

impl MethodArea {
    fn new() -> Self {
        Self { classes: HashMap::new() }
    }
}

struct JThread {
    stack: Vec<Frame>,
    method_area: Rc<MethodArea>,
}

impl JThread {
    fn new(method_area: Rc<MethodArea>) -> Self {
        Self {stack: Vec::new(), method_area}
    }

    fn execute_method(&mut self, class_name: &str, method_name: &str, args: &[i32]) -> Result<JTypeValue> {
        println!("running {}.{} with {:?}", class_name, method_name, args);

        let f = self.build_frame(class_name, method_name, args)?;
        self.stack.push(f);

        let result = self.execute_frame()?;
        Ok(JTypeValue::Int(result))
    }

    fn build_frame(&self, class_name: &str, method_name: &str, args: &[i32]) -> Result<Frame> {
        let class = match self.method_area.classes.get(class_name) {
            Some(c) => c,
            None => return Err(anyhow!("no such class error"))
        };

        let method = match class.methods.iter().find(|&m|  m.name.deref() == method_name) {
            Some(m) => m,
            None => return Err(anyhow!("no such method"))
        };

        let code = match method.attributes.iter().find(|&a| a.name.deref() == "Code") {
            Some(c) => c,
            None => return Err(anyhow!("'code' attribute not found!"))
        };

        let _max_locals = u16::from_be_bytes([code.data[2],code.data[3]]) as usize;

        let mut locals = Vec::<i32>::new();
        for a in args.iter() {
            locals.push(*a);
        }

        let frame = Frame::new(class.clone(),  code.data[8..].to_vec(), locals);
        Ok(frame)
    }

    fn top_frame(&self) -> &Frame {
        match self.stack.last() {
            Some(f) => f,
            None => panic!("tried to get top frame, but there is nothing!")
        }
    }

    fn top_frame_mut(&mut self) -> &mut Frame {
        match self.stack.last_mut() {
            Some(f) => f,
            None => panic!("tried to get top frame, but there is nothing")
        }
    }

    fn execute_frame(&mut self) -> Result<i32> {
        loop {
            let frame = self.top_frame();

            let op = frame.code[frame.ip as usize];
            println!("OP: {}, stack: {:?}", op, frame.operand_stack);

            match op {
                26 => {
                    let var = frame.locals[0];
                    let frame_mut = self.top_frame_mut();
                    frame_mut.push_stack(var);
                    frame_mut.increment_ip(1)
                },
                27 => { // iload_1
                    let var = frame.locals[1];
                    let frame_mut = self.top_frame_mut();
                    frame_mut.push_stack(var);
                    frame_mut.increment_ip(1);
                },
                116 => { // ineg
                    let frame_mut = self.top_frame_mut();
                    let v = frame_mut.pop_stack()?;
                    frame_mut.push_stack(-v);
                    frame_mut.increment_ip(1);
                }
                96 => { // iadd
                    let frame_mut = self.top_frame_mut();

                    let a = frame_mut.pop_stack()?;
                    let b = frame_mut.pop_stack()?;
                    frame_mut.push_stack(a + b);
                    frame_mut.increment_ip(1);
                },
                184 => { // invokestatic
                    let index = u16::from_be_bytes([frame.code[(frame.ip+1) as usize], frame.code[(frame.ip+2) as usize]]);

                    let static_method = frame.class.const_pool.resolve_static_method(index as usize)?;

                    let nargs = Self::get_nargs(&static_method.method_desc);
                    let locals = Self::locals_from_operand_stack(&frame, nargs);

                    let invoked_method_frame = self.build_frame(&static_method.class_name, &static_method.method_name, &locals)?;

                    self.stack.push(invoked_method_frame);
                    let result = self.execute_frame()?;
                    self.stack.pop();

                    let frame_mut = self.top_frame_mut();
                    frame_mut.push_stack(result);
                    frame_mut.increment_ip(3);
                }
                172 => { // ireturn
                    let frame_mut = self.top_frame_mut();
                    return frame_mut.pop_stack();
                },
                _ => {
                    println!("unknown opcode {}", op);
                    panic!("unknown odpcode!")
                }
            }
        }
    }

    fn get_nargs(desc: &str) -> u32 {
        let mut nargs = 0;
        for c in desc[1..].chars() {
            if c == ')' {
                break;
            }

            nargs += 1;
        }
        nargs
    }

    fn locals_from_operand_stack(frame: &Frame, nargs: u32) -> Vec<i32> {
        let mut locals = Vec::<i32>::new();
        let len = frame.operand_stack.len();
        let mut i = 1;
        while i <= nargs {
            locals.push(frame.operand_stack[len - (i as usize)]);
            i += 1;
        }
        locals
    }
}

#[derive(Debug)]
struct Frame {
    class: Rc<Class>,
    ip: u32,
    code: Vec<u8>,
    locals: Vec<i32>,
    operand_stack: Vec<i32>
}

impl Frame {

    pub fn new(class: Rc<Class>, code: Vec<u8>, locals: Vec<i32>) -> Self {
        Self {
            class,
            code,
            ip: 0,
            locals,
            operand_stack: Vec::new()
        }
    }

    pub fn pop_stack(&mut self) -> Result<i32> {
        match self.operand_stack.pop() {
            Some(v) => Ok(v),
            None => Err(anyhow!("tried popping stack but nothing found!"))
        }
    }

    pub fn push_stack(&mut self, v: i32) {
        self.operand_stack.push(v);
    }

    pub fn increment_ip(&mut self, inc: u32) {
        self.ip += inc;
    }
}
