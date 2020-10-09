use crate::class::{Class, Const};
use std::ops::{Deref, Add, Neg};
use std::collections::HashMap;

use anyhow::{Result, anyhow};
use std::rc::Rc;

const ILOAD: u8 = 21;
const ILOAD_0: u8 = 26;
const ILOAD_1: u8 = 27;
const ILOAD_2: u8 = 28;
const ILOAD_3: u8 = 29;
const INEG: u8 = 116;
const IADD: u8 = 96;
const IRETURN: u8 = 172;

const LLOAD_0: u8 = 30;
const LLOAD_1: u8 = 31;
const LLOAD_2: u8 = 32;
const LLOAD_3: u8 = 33;
const LNEG: u8 = 117;
const LADD: u8 = 97;
const LRETURN: u8 = 173;

const FLOAD_0: u8 = 34;
const FLOAD_1: u8 = 35;
const FLOAD_2: u8 = 36;
const FLOAD_3: u8 = 37;
const FNEG: u8 = 118;
const FADD: u8 = 98;
const FRETURN: u8 = 174;
const FSTORE_0: u8 = 67;
const FSTORE_1: u8 = 68;
const FSTORE_2: u8 = 69;
const FSTORE_3: u8 = 70;

const DLOAD_0: u8 = 38;
const DLOAD_1: u8 = 39;
const DLOAD_2: u8 = 40;
const DLOAD_3: u8 = 41;
const DNEG: u8 = 119;
const DADD: u8 = 99;
const DRETURN: u8 = 175;

const LDC: u8 = 18;

const INVOKESTATIC: u8 = 184;

#[derive(Debug, Copy, Clone)]
pub enum JTypeValue {
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    Reference(usize),
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

    pub fn run(&mut self, class_name: &str, method_name: &str, args: &[JTypeValue]) -> Result<JTypeValue> {
        return self.thread.execute_method(class_name, method_name, args);
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

    fn execute_method(&mut self, class_name: &str, method_name: &str, args: &[JTypeValue]) -> Result<JTypeValue> {
        println!("running {}.{} with {:?}", class_name, method_name, args);

        let f = self.build_frame(class_name, method_name, args)?;
        self.stack.push(f);

        let result = self.execute()?;

        Ok(result)
    }

    fn build_frame(&self, class_name: &str, method_name: &str, args: &[JTypeValue]) -> Result<Frame> {
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

        let mut locals = Vec::<JTypeValue>::new();
        for a in args.iter() {
            locals.push(*a);
            match a {
                JTypeValue::Double(_) | JTypeValue::Long(_) => locals.push(JTypeValue::Empty),
                _ => { }
            }
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

    fn execute(&mut self) -> Result<JTypeValue> {
        loop {
            let frame = self.top_frame_mut();

            let op = frame.code[frame.ip as usize];
            println!("OP: {}, stack: {:?}", op, frame.operand_stack);

            match op {
                ILOAD => { // TODO implement other LOADs (without underscores)
                    let index = frame.code[frame.ip + 1];
                    let var = frame.locals[index as usize];
                    frame.push_stack(var);
                    frame.increment_ip(2);
                }
                ILOAD_0 | LLOAD_0 | FLOAD_0 | DLOAD_0 => {
                    let var = frame.locals[0];
                    frame.push_stack(var);
                    frame.increment_ip(1)
                },
                ILOAD_1 | LLOAD_1 | FLOAD_1 | DLOAD_1 => {
                    let var = frame.locals[1];
                    frame.push_stack(var);
                    frame.increment_ip(1);
                },
                ILOAD_2 | LLOAD_2 | FLOAD_2 | DLOAD_2 => {
                    let var = frame.locals[2];
                    frame.push_stack(var);
                    frame.increment_ip(1);
                },
                ILOAD_3 | LLOAD_3 | FLOAD_3 | DLOAD_3 => {
                    let var = frame.locals[3];
                    frame.push_stack(var);
                    frame.increment_ip(1);
                },
                INEG | LNEG | FNEG | DNEG => { // ineg
                    let var = frame.pop_stack()?;
                    frame.push_stack(-var);
                    frame.increment_ip(1);
                }
                IADD | LADD | FADD | DADD => { // iadd
                    let a = frame.pop_stack()?;
                    let b = frame.pop_stack()?;
                    frame.push_stack(a + b);
                    frame.increment_ip(1);
                },
                FSTORE_0 => {
                    // TODO test if this works correctly with various vector sizes
                    let v = frame.pop_stack()?;
                    frame.locals.insert(0, v);
                    frame.increment_ip(1)
                },
                FSTORE_1 => {
                    let v = frame.pop_stack()?;
                    frame.locals.insert(1, v);
                    frame.increment_ip(1)
                },
                FSTORE_2 => {
                    let v = frame.pop_stack()?;
                    frame.locals.insert(2, v);
                    frame.increment_ip(1)
                },
                FSTORE_3 => {
                    let v = frame.pop_stack()?;
                    frame.locals.insert(3, v);
                    frame.increment_ip(1)
                },

                // TODO implement other STORE

                LDC => { // TODO implement other LDC e.g. LDC_2W
                    let index = frame.code[frame.ip + 1];

                    let c = frame.class.const_pool.resolve(index as usize)?;

                    match c {
                        Const::Integer(x) => frame.operand_stack.push(JTypeValue::Int(*x)),
                        Const::Float(x) => frame.operand_stack.push(JTypeValue::Float(*x)),
                        _ => panic!("not supported") // TODO implement support for references and String literals
                    }

                    frame.increment_ip(2);
                },

                INVOKESTATIC => { // invokestatic
                    let method_index_byte1 = frame.code[frame.ip + 1];
                    let method_index_byte2 = frame.code[frame.ip + 2];
                    let method_index = u16::from_be_bytes([method_index_byte1, method_index_byte2]);

                    let static_method = frame.class.const_pool.resolve_static_method(method_index as usize)?;

                    let nargs = Self::get_nargs(&static_method.method_desc);

                    // let frame_mut = self.top_frame_mut();
                    let locals = Self::pop_operand_stack_to_locals(frame, nargs);

                    let invoked_method_frame = self.build_frame(&static_method.class_name, &static_method.method_name, &locals)?;

                    self.stack.push(invoked_method_frame);
                    // Currently handled recursively, maybe it could be done iteratively?
                    let result = self.execute()?;

                    let frame_mut = self.top_frame_mut();
                    frame_mut.push_stack(result);
                    frame_mut.increment_ip(3);
                }
                IRETURN | LRETURN | FRETURN | DRETURN => { // ireturn
                    let mut frame =  match self.stack.pop() {
                        Some(f) => f,
                        None => panic!("no frame to pop")
                    };
                    return frame.pop_stack();
                },
                _ => {
                    println!("unknown opcode {}", op);
                    panic!("unknown opcode!")
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

    fn pop_operand_stack_to_locals(frame: &mut Frame, nargs: u32) -> Vec<JTypeValue> {
        let mut locals = Vec::new();
        let mut i = 1;
        while i <= nargs {
            match frame.operand_stack.pop() {
                Some(i) => {
                    let uses_two_entries = match i {
                        JTypeValue::Double(_) | JTypeValue::Long(_) => true,
                        _ => false
                    };

                    locals.push(i);
                    if uses_two_entries {
                        locals.push(JTypeValue::Empty)
                    }
                }
                None => panic!("trying to pop too much from frame operand stack")
            };

            i += 1;
        }
        locals
    }
}

#[derive(Debug)]
struct Frame {
    class: Rc<Class>,
    ip: usize,
    code: Vec<u8>,
    locals: Vec<JTypeValue>,
    operand_stack: Vec<JTypeValue>
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
            None => Err(anyhow!("tried popping stack but nothing found!"))
        }
    }

    pub fn push_stack(&mut self, v: JTypeValue) {
        self.operand_stack.push(v);
    }

    pub fn increment_ip(&mut self, inc: usize) {
        self.ip += inc;
    }
}
