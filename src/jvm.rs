use crate::class::{Class, Const};
use std::ops::{Deref, Add, Neg};
use std::collections::HashMap;

use anyhow::{Result, anyhow};
use std::rc::Rc;
use std::cell::RefCell;
use std::os::macos::raw::stat;
use std::borrow::Borrow;
pub use crate::jvm::types::JTypeValue;
use crate::jvm::objects::{Heap, Object, Array};
use crate::jvm::frame::Frame;
use crate::jvm::types::NULL_REF;


mod frame;
mod types;
mod objects;

const ACONST_NULL: u8 = 1;

const ALOAD: u8 = 25;
const ALOAD_0: u8 = 42;
const ALOAD_1: u8 = 43;
const ALOAD_2: u8 = 44;
const ALOAD_3: u8 = 45;
const ASTORE: u8 = 58;
const ASTORE_0: u8 = 75;
const ASTORE_1: u8 = 76;
const ASTORE_2: u8 = 77;
const ASTORE_3: u8 = 78;

const BIPUSH: u8 = 16;

const ICONST_M1: u8 = 2;
const ICONST_0: u8 = 3;
const ICONST_1: u8 = 4;
const ICONST_2: u8 = 5;
const ICONST_3: u8 = 6;
const ICONST_4: u8 = 7;
const ICONST_5: u8 = 8;


const ILOAD: u8 = 21;
const ILOAD_0: u8 = 26;
const ILOAD_1: u8 = 27;
const ILOAD_2: u8 = 28;
const ILOAD_3: u8 = 29;
const IALOAD: u8 = 46;
const INEG: u8 = 116;
const IADD: u8 = 96;
const IRETURN: u8 = 172;
const ISTORE: u8 = 54;
const ISTORE_0: u8 = 59;
const ISTORE_1: u8 = 60;
const ISTORE_2: u8 = 61;
const ISTORE_3: u8 = 62;
const IASTORE: u8 = 79;

const LCONST_0: u8 = 9;
const LCONST_1: u8 = 10;
const LLOAD: u8 = 22;
const LLOAD_0: u8 = 30;
const LLOAD_1: u8 = 31;
const LLOAD_2: u8 = 32;
const LLOAD_3: u8 = 33;
const LNEG: u8 = 117;
const LADD: u8 = 97;
const LRETURN: u8 = 173;
const LSTORE: u8 = 55;
const LSTORE_0: u8 = 63;
const LSTORE_1: u8 = 64;
const LSTORE_2: u8 = 65;
const LSTORE_3: u8 = 66;

const FCONST_0: u8 = 11;
const FCONST_1: u8 = 12;
const FCONST_2: u8 = 13;
const FLOAD: u8 = 23;
const FLOAD_0: u8 = 34;
const FLOAD_1: u8 = 35;
const FLOAD_2: u8 = 36;
const FLOAD_3: u8 = 37;
const FNEG: u8 = 118;
const FADD: u8 = 98;
const FRETURN: u8 = 174;
const FSTORE: u8 = 56;
const FSTORE_0: u8 = 67;
const FSTORE_1: u8 = 68;
const FSTORE_2: u8 = 69;
const FSTORE_3: u8 = 70;

const DCONST_0: u8 = 14;
const DCONST_1: u8 = 15;
const DLOAD: u8 = 24;
const DLOAD_0: u8 = 38;
const DLOAD_1: u8 = 39;
const DLOAD_2: u8 = 40;
const DLOAD_3: u8 = 41;
const DNEG: u8 = 119;
const DADD: u8 = 99;
const DRETURN: u8 = 175;
const DSTORE: u8 = 57;
const DSTORE_0: u8 = 71;
const DSTORE_1: u8 = 72;
const DSTORE_2: u8 = 73;
const DSTORE_3: u8 = 74;

const IFEQ: u8 = 153;
const IFNE: u8 = 154;
const IFLT: u8 = 155;
const IFGE: u8 = 156;
const IFGT: u8 = 157;
const IFLE: u8 = 158;
const IF_ICMPEQ: u8 = 159;
const IF_ICMPNE: u8 = 160;
const IF_ICMPLT: u8 = 161;
const IF_ICMPGE: u8 = 162;
const IF_ICMPGT: u8 = 163;
const IF_ICMPLE: u8 = 164;
const IF_ACMPEQ: u8 = 165;
const IF_ACMPNE: u8 = 166;
const IFNULL: u8 = 198;
const IFNONNULL: u8 = 199;

const NEWARRAY: u8 = 188;

const GETFIELD: u8 = 180;
const PUTFIELD: u8 = 181;

const LDC: u8 = 18;

const NEW: u8 = 187;

const INVOKESPECIAL: u8 = 183;
const INVOKESTATIC: u8 = 184;
const INVOKEVIRTUAL: u8 = 182;

const DUP: u8 = 89;

const RETURN: u8 = 177;

pub struct JVM {
    thread: JThread,
    heap: Rc<RefCell<Heap>>,
    method_area: Rc<MethodArea>
}

impl JVM {
    pub fn new() -> Result<Self> {
        let mut method_area = MethodArea::new();
        let class = crate::class::load("java/Add.class")?;
        println!("class: {:?}", class);
        method_area.classes.insert(class.name.clone(), Rc::new(class));

        let method_area = Rc::new(method_area);
        let heap = Rc::new(RefCell::new(Heap::new()));
        let thread = JThread::new(method_area.clone(), heap.clone());

        Ok(Self { method_area, thread, heap })
    }

    pub fn run(&mut self, class_name: &str, method_name: &str, args: &[JTypeValue]) -> Result<JTypeValue> {
        return self.thread.execute_method(class_name, method_name, args);
    }
}


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
    heap: Rc<RefCell<Heap>>,
}

impl JThread {
    fn new(method_area: Rc<MethodArea>, heap: Rc<RefCell<Heap>>) -> Self {
        Self {stack: Vec::new(), method_area, heap}
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
                ACONST_NULL => {
                    frame.push_stack(NULL_REF);
                    frame.inc_ip(1);
                },
                ICONST_M1 => {
                    frame.push_stack(JTypeValue::Int(-1));
                    frame.inc_ip(1);
                }
                ICONST_0 => {
                    frame.push_stack(JTypeValue::Int(0));
                    frame.inc_ip(1);
                },
                ICONST_1 => {
                    frame.push_stack(JTypeValue::Int(1));
                    frame.inc_ip(1);
                },
                ICONST_2 => {
                    frame.push_stack(JTypeValue::Int(2));
                    frame.inc_ip(1);
                },
                ICONST_3 => {
                    frame.push_stack(JTypeValue::Int(3));
                    frame.inc_ip(1);
                },
                ICONST_4 => {
                    frame.push_stack(JTypeValue::Int(4));
                    frame.inc_ip(1);
                },
                LCONST_0 => {
                    frame.push_stack(JTypeValue::Long(0));
                    frame.inc_ip(1);
                },
                LCONST_1 => {
                    frame.push_stack(JTypeValue::Long(1));
                    frame.inc_ip(1);
                }
                FCONST_0 => {
                    frame.push_stack(JTypeValue::Float(0.0));
                    frame.inc_ip(1);
                },
                FCONST_1 => {
                    frame.push_stack(JTypeValue::Float(1.0));
                    frame.inc_ip(1);
                },
                FCONST_2 => {
                    frame.push_stack(JTypeValue::Float(2.0));
                    frame.inc_ip(1);
                },
                ICONST_5 => {
                    frame.push_stack(JTypeValue::Int(5));
                    frame.inc_ip(1);
                },
                DCONST_0 => {
                    frame.push_stack(JTypeValue::Double(0.0));
                    frame.inc_ip(1);
                },
                DCONST_1 => {
                    frame.push_stack(JTypeValue::Double(1.0));
                    frame.inc_ip(1);
                },
                ALOAD | ILOAD | LLOAD | FLOAD | DLOAD => {
                    let index = frame.code[frame.ip + 1];
                    let var = frame.locals[index as usize];
                    frame.push_stack(var);
                    frame.inc_ip(2);
                },
                ALOAD_0 | ILOAD_0 | LLOAD_0 | FLOAD_0 | DLOAD_0 => {
                    let var = frame.locals[0];
                    frame.push_stack(var);
                    frame.inc_ip(1)
                },
                ALOAD_1 | ILOAD_1 | LLOAD_1 | FLOAD_1 | DLOAD_1 => {
                    let var = frame.locals[1];
                    frame.push_stack(var);
                    frame.inc_ip(1);
                },
                ALOAD_2 | ILOAD_2 | LLOAD_2 | FLOAD_2 | DLOAD_2 => {
                    let var = frame.locals[2];
                    frame.push_stack(var);
                    frame.inc_ip(1);
                },
                ALOAD_3 | ILOAD_3 | LLOAD_3 | FLOAD_3 | DLOAD_3 => {
                    let var = frame.locals[3];
                    frame.push_stack(var);
                    frame.inc_ip(1);
                },
                INEG | LNEG | FNEG | DNEG => { // ineg
                    let var = frame.pop_stack()?;
                    frame.push_stack(-var);
                    frame.inc_ip(1);
                }
                IADD | LADD | FADD | DADD => { // iadd
                    let a = frame.pop_stack()?;
                    let b = frame.pop_stack()?;
                    frame.push_stack(a + b);
                    frame.inc_ip(1);
                },
                ASTORE | ISTORE | LSTORE | FSTORE | DSTORE => {
                    let v = frame.pop_stack()?;
                    let index = frame.code[frame.ip + 1];
                    frame.locals[index as usize] = v;
                    frame.inc_ip(2);
                }
                ASTORE_0 | ISTORE_0 | LSTORE_0 | FSTORE_0 | DSTORE_0 => {
                    let v = frame.pop_stack()?;
                    frame.locals[0] = v;
                    frame.inc_ip(1)
                },
                ASTORE_1 | ISTORE_1 | LSTORE_1 | FSTORE_1 | DSTORE_1 => {
                    let v = frame.pop_stack()?;
                    frame.locals[1] = v;
                    frame.inc_ip(1)
                },
                ASTORE_2 | ISTORE_2 | LSTORE_2 | FSTORE_2 | DSTORE_2 => {
                    let v = frame.pop_stack()?;
                    frame.locals[2] = v;
                    frame.inc_ip(1)
                },
                ASTORE_3 | ISTORE_3 | LSTORE_3 | FSTORE_3 | DSTORE_3 => {
                    let v = frame.pop_stack()?;
                    frame.locals[3] = v;
                    frame.inc_ip(1)
                },
                IASTORE => {
                    let value = frame.pop_stack()?;
                    let index = frame.pop_int();
                    let arr_ref = frame.pop_ref();

                    {
                        let mut heap = self.heap.borrow_mut();
                        let arr = heap.get_arr_mut(arr_ref);
                        arr.set(index as usize, value);
                    }

                    let frame_mut = self.top_frame_mut();
                    frame_mut.inc_ip(1);
                },

                IALOAD => {
                    let index = frame.pop_int();
                    let array_ref = frame.pop_ref();

                    let val = {
                        let heap = RefCell::borrow(&self.heap);
                        let arr = heap.get_arr(array_ref);
                        arr.get_int(index as usize)
                    };

                    let frame_mut = self.top_frame_mut();
                    frame_mut.push_stack(JTypeValue::Int(val));
                    frame_mut.inc_ip(1);
                }

                LDC => { // TODO implement other LDC e.g. LDC_2W
                    let index = frame.code[frame.ip + 1];

                    let c = frame.class.const_pool.resolve(index as usize)?;

                    match c {
                        Const::Integer(x) => frame.operand_stack.push(JTypeValue::Int(*x)),
                        Const::Float(x) => frame.operand_stack.push(JTypeValue::Float(*x)),
                        _ => panic!("not supported") // TODO implement support for references and String literals
                    }

                    frame.inc_ip(2);
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
                    frame_mut.inc_ip(3);
                }

                IRETURN | LRETURN | FRETURN | DRETURN => {
                    let mut frame =  match self.stack.pop() {
                        Some(f) => f,
                        None => panic!("no frame to pop")
                    };
                    return frame.pop_stack();
                },

                RETURN => {
                    self.stack.pop();
                    return Ok(JTypeValue::Empty);
                }

                DUP => {
                    let top_value = match frame.operand_stack.last() {
                        Some(v) => *v,
                        None => panic!("no value to dup!")
                    };

                    frame.push_stack(top_value);
                    frame.inc_ip(1);
                },

                // NEW
                NEW => {
                    let class_index_byte1 = frame.code[frame.ip + 1];
                    let class_index_byte2 = frame.code[frame.ip + 2];
                    let class_index = u16::from_be_bytes([class_index_byte1, class_index_byte2]);

                    // build an object for the class
                    let obj = Object::new(frame.class.clone());
                    let obj_ref = self.heap.borrow_mut().allocate_obj(obj);

                    let frame_mut = self.top_frame_mut();
                    frame_mut.push_stack(JTypeValue::Ref(obj_ref));
                    frame_mut.inc_ip(3);
                },

                BIPUSH => {
                    let byte = frame.code[frame.ip + 1];
                    frame.push_stack(JTypeValue::Int(byte as i32));
                    frame.inc_ip(2);
                },

                // TODO implement INVOKEVIRTUAL properly
                INVOKESPECIAL | INVOKEVIRTUAL => {
                    let method_index_byte1 = frame.code[frame.ip + 1];
                    let method_index_byte2 = frame.code[frame.ip + 2];
                    let method_index = u16::from_be_bytes([method_index_byte1, method_index_byte2]);

                    let static_method = frame.class.const_pool.resolve_static_method(method_index as usize)?;

                    // nargs + 1 because we also need to pass instance object reference
                    let nargs = Self::get_nargs(&static_method.method_desc) + 1;
                    let locals = Self::pop_operand_stack_to_locals(frame, nargs);
                    print!("locals: {:?}", locals);

                    // TODO remove this hack once java/lang/Object can be properly loaded!
                    if static_method.class_name.deref() == "java/lang/Object" {
                        frame.inc_ip(3);
                        continue;
                    }

                    let invoked_method_frame = self.build_frame(&static_method.class_name, &static_method.method_name, &locals)?;

                    self.stack.push(invoked_method_frame);
                    // Currently handled recursively, maybe it could be done iteratively?
                    let result = self.execute()?;

                    let frame_mut = self.top_frame_mut();
                    frame_mut.push_stack(result);
                    frame_mut.inc_ip(3);
                },

                GETFIELD => {
                    let field_index_byte1 = frame.code[frame.ip + 1];
                    let field_index_byte2 = frame.code[frame.ip + 2];
                    let field_index = u16::from_be_bytes([field_index_byte1, field_index_byte2]);

                    let obj_ref = match frame.pop_stack()? {
                        JTypeValue::Ref(r) => r,
                        _ => panic!("GETFIELD called on value type different than object ref")
                    };

                    let value = {
                        let heap = RefCell::borrow(&self.heap);
                        let object = heap.get_obj(obj_ref);
                        object.field_value(field_index as usize)
                    };

                    let frame_mut = self.top_frame_mut();
                    frame_mut.push_stack(value);
                    frame_mut.inc_ip(3);
                },

                PUTFIELD => {
                    let field_index_byte1 = frame.code[frame.ip + 1];
                    let field_index_byte2 = frame.code[frame.ip + 2];
                    let field_index = u16::from_be_bytes([field_index_byte1, field_index_byte2]);

                    let val = frame.pop_stack()?;

                    let obj_ref = match frame.pop_stack()? {
                        JTypeValue::Ref(r) => r,
                        _ => { panic!("PUTFIELD called on value type different than object ref") }
                    };

                    {
                        let mut heap = self.heap.borrow_mut();
                        let object = heap.get_obj_mut(obj_ref);
                        object.fields.insert(field_index as usize, val);
                    }

                    let frame_mut = self.top_frame_mut();
                    frame_mut.inc_ip(3);
                },

                IFEQ..=IFLE => {

                    let val = match frame.pop_stack()? {
                        JTypeValue::Int(i) => i,
                        _ => { panic!("popped value must be an int") }
                    };

                    let result = match op {
                        IFEQ => val == 0,
                        IFNE => val != 0,
                        IFLT => val < 0,
                        IFLE => val <= 0,
                        IFGT => val > 0,
                        IFGE => val >= 0,
                        _ => { panic!("impossible - op is already within expected range")}
                    };

                    if result {
                        // Jump to the provided branch
                        let branch_byte_1 = frame.code[frame.ip + 1];
                        let branch_byte_2: u8 = frame.code[frame.ip + 2];
                        let branch_index = u16::from_be_bytes([branch_byte_1, branch_byte_2]);
                        frame.ip = branch_index as usize;
                    } else {
                        // Just move ahead
                        frame.inc_ip(3);
                    }
                },

                IF_ICMPEQ..=IF_ICMPLE => {
                    let val2 = match frame.pop_stack()? {
                        JTypeValue::Int(i) => i,
                        _ => { panic!("popped value must be an int") }
                    };
                    let val1 = match frame.pop_stack()? {
                        JTypeValue::Int(i) => i,
                        _ => { panic!("popped value must be an int") }
                    };

                    let result = match op {
                        IF_ICMPEQ => val1 == val2,
                        IF_ICMPNE => val1 != val2,
                        IF_ICMPLT => val1 < val2,
                        IF_ICMPLE => val1 <= val2,
                        IF_ICMPGT => val1 > val2,
                        IF_ICMPGE => val1 >= val2,
                        _ =>  { panic!("impossible - op is already within expected range")}
                    };

                    if result {
                        // Jump to the provided branch
                        let branch_byte_1 = frame.code[frame.ip + 1];
                        let branch_byte_2: u8 = frame.code[frame.ip + 2];
                        let branch_index = u16::from_be_bytes([branch_byte_1, branch_byte_2]);
                        frame.ip = branch_index as usize;
                    } else {
                        // Just move ahead
                        frame.inc_ip(3);
                    }
                },

                IF_ACMPEQ..=IF_ACMPNE => {
                    let val2 = match frame.pop_stack()? {
                        JTypeValue::Ref(r) => r,
                        _ => panic!("popped value must be an int")
                    };
                    let val1 = match frame.pop_stack()? {
                        JTypeValue::Ref(r) => r,
                        _ => panic!("popped value must be an int")
                    };

                    let result = match op {
                        IF_ACMPEQ => val1 == val2,
                        IF_ACMPNE => val1 != val2,
                        _ => panic!("impossible - op is already within expected range")
                    };

                    if result {
                        // Jump to the provided branch
                        let branch_byte_1 = frame.code[frame.ip + 1];
                        let branch_byte_2: u8 = frame.code[frame.ip + 2];
                        let branch_index = u16::from_be_bytes([branch_byte_1, branch_byte_2]);
                        frame.ip = branch_index as usize;
                    } else {
                        // Just move ahead
                        frame.inc_ip(3);
                    }
                },

                IFNULL | IFNONNULL => {
                    let val = frame.pop_stack()?;

                    let result = match op {
                        IFNULL => val == NULL_REF,
                        IFNONNULL => val != NULL_REF,
                        _ => panic!("impossible - op is already within expected range")
                    };

                    if result {
                        // Jump to the provided branch
                        let branch_byte_1 = frame.code[frame.ip + 1];
                        let branch_byte_2: u8 = frame.code[frame.ip + 2];
                        let branch_index = u16::from_be_bytes([branch_byte_1, branch_byte_2]);
                        frame.ip = branch_index as usize;
                    } else {
                        // Just move ahead
                        frame.inc_ip(3);
                    }
                },

                NEWARRAY => {
                    let count = frame.pop_int();

                    // TODO if count < 0, throw NegativeArraySizeException

                    let array = Array::new(count as usize);
                    let arr_ref = self.heap.borrow_mut().allocate_arr(array);

                    let frame_mut = self.top_frame_mut();
                    frame_mut.push_stack(JTypeValue::Ref(arr_ref));
                    frame_mut.inc_ip(2);
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

                    // locals.push(i);
                    locals.insert(0, i);
                    if uses_two_entries {
                        locals.insert(1, JTypeValue::Empty);
                        // locals.push(JTypeValue::Empty)
                    }
                }
                None => panic!("trying to pop too much from frame operand stack")
            };

            i += 1;
        }
        locals
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_get_nargs() {
        let nargs = JThread::get_nargs("(II)I");
        assert_eq!(nargs, 2);
    }



}