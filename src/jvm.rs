use crate::class::Class;
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

pub struct JVM<'a> {
    thread: JThread<'a>,
    heap: Heap,
    method_area: Rc<MethodArea>
}

impl <'a> JVM<'a> {
    pub fn new() -> Result<Self> {
        let mut method_area = MethodArea::new();
        let class = crate::class::load("java/Add.class")?;
        println!("class: {:?}", class);
        method_area.classes.insert(class.name.clone(), class);

        let method_area = Rc::new(method_area);
        let thread = JThread::new(method_area.clone());

        Ok(Self { method_area, thread, heap: Heap })
    }

    pub fn run(&'a mut self, class_name: &str, method_name: &str, args: &[i32]) -> Result<i32> {
        self.thread.execute_method(class_name, method_name, args);
        Ok(0)
    }

    // pub fn load(&mut self, path: &str) -> Result<()> {
    //     let class = crate::class::load(path)?;
    //     println!("class: {:?}", class);
    //     self.method_area.classes.insert(class.name.clone(), class);
    //     Ok(())
    // }

    // pub fn run_method(&self, class_name: &str, method_name: &str, args: &[i32]) -> Result<JTypeValue>  {
    //     println!("running {}.{} with {:?}", class_name, method_name, args);
    //
    //     let class = match self.method_area.classes.get(class_name) {
    //         Some(c) => c,
    //         None => return Err(anyhow!("no such class error"))
    //     };
    //
    //     let method = match class.methods.iter().find(|&m|  m.name.deref() == method_name) {
    //         Some(m) => m,
    //         None => return Err(anyhow!("no such method"))
    //     };
    //
    //     let code = match method.attributes.iter().find(|&a| a.name.deref() == "Code") {
    //         Some(c) => c,
    //         None => return Err(anyhow!("'code' attribute not found!"))
    //     };
    //
    //     let _max_locals = u16::from_be_bytes([code.data[2],code.data[3]]) as usize;
    //
    //     let mut locals = Vec::<i32>::new();
    //     for a in args.iter() {
    //         locals.push(*a);
    //     }
    //
    //     let mut frame = Frame {
    //         class,
    //         code: &code.data[8..],
    //         ip: 0,
    //         locals,
    //         operand_stack: Vec::new()
    //     };
    //
    //     let result = self.execute_frame(&mut frame)?;
    //
    //     Ok(JTypeValue::Int(result))
    // }
    //
    // fn execute_frame(&self, f: &mut Frame) -> Result<i32> {
    //     loop {
    //         let op = f.code[f.ip as usize];
    //
    //         println!("OP: {}, stack: {:?}", op, f.operand_stack);
    //
    //         match op {
    //             26 => { // iload_0
    //                 f.operand_stack.push(f.locals[0]);
    //                 f.ip += 1;
    //             },
    //             27 => { // iload_1
    //                 f.operand_stack.push(f.locals[1]);
    //                 f.ip += 1;
    //             },
    //             116 => { // ineg
    //                 let v = f.pop_stack()?;
    //                 f.operand_stack.push(-v);
    //                 f.ip += 1;
    //             }
    //             96 => { // iadd
    //                 let a = f.pop_stack()?;
    //                 let b = f.pop_stack()?;
    //                 f.operand_stack.push(a + b);
    //                 f.ip += 1;
    //             },
    //             172 => { // ireturn
    //                 return f.pop_stack()
    //             },
    //             184 => { // invokestatic
    //                 let index = u16::from_be_bytes([f.code[(f.ip+1) as usize], f.code[(f.ip+2) as usize]]);
    //
    //                 let static_method = f.class.const_pool.resolve_static_method(index as usize)?;
    //
    //                 println!("prepping to invoke {:?}", static_method);
    //
    //                 let mut nargs = 0;
    //
    //                 let desc = static_method.method_desc.deref();
    //                 for c in desc[1..].chars() {
    //                     if c == ')' {
    //                         break;
    //                     }
    //
    //                     nargs += 1;
    //                 }
    //
    //                 println!("nargs = {}", nargs);
    //
    //                 let mut locals = Vec::<i32>::new();
    //                 while nargs > 0 {
    //                     locals.push(f.pop_stack()?);
    //                     nargs -= 1;
    //                 }
    //
    //                 let r = self.run_method(
    //                     static_method.class_name.deref(),
    //                     static_method.method_name.deref(),
    //                     &locals
    //                 )?;
    //
    //                 println!("result from method: {:?}", r);
    //
    //                 match r {
    //                     JTypeValue::Int(i) => f.operand_stack.push(i),
    //                     _ => return Err(anyhow!("unsupported JTypeValue"))
    //                 };
    //
    //                 f.ip += 3;
    //             }
    //             _ => {
    //                 println!("unknown opcode {}", op);
    //             }
    //         }
    //     }
    // }

}

struct Heap;

struct MethodArea {
    classes: HashMap<Rc<str>, Class>
}

impl MethodArea {
    fn new() -> Self {
        Self { classes: HashMap::new() }
    }
}

struct JThread<'a> {
    stack: Vec<Frame<'a>>,
    method_area: Rc<MethodArea>,
}

impl <'a> JThread<'a> {
    fn new(method_area: Rc<MethodArea>) -> Self {
        Self {stack: Vec::new(), method_area}
    }

    fn execute_method(&'a mut self, class_name: &str, method_name: &str, args: &[i32]) -> Result<JTypeValue> {
        println!("running {}.{} with {:?}", class_name, method_name, args);

        // let class = match self.method_area.classes.get(class_name) {
        //     Some(c) => c,
        //     None => return Err(anyhow!("no such class error"))
        // };
        //
        // let method = match class.methods.iter().find(|&m|  m.name.deref() == method_name) {
        //     Some(m) => m,
        //     None => return Err(anyhow!("no such method"))
        // };
        //
        // let code = match method.attributes.iter().find(|&a| a.name.deref() == "Code") {
        //     Some(c) => c,
        //     None => return Err(anyhow!("'code' attribute not found!"))
        // };
        //
        // let _max_locals = u16::from_be_bytes([code.data[2],code.data[3]]) as usize;
        //
        // let mut locals = Vec::<i32>::new();
        // for a in args.iter() {
        //     locals.push(*a);
        // }
        //
        // let mut frame = Frame {
        //     class,
        //     code: &code.data[8..],
        //     ip: 0,
        //     locals,
        //     operand_stack: Vec::new()
        // };

        let f = self.build_frame(class_name, method_name, args)?;
        self.stack.push(f);

        let result = self.execute_frame()?;
        Ok(JTypeValue::Int(result))
    }

    fn build_frame(&'a self, class_name: &str, method_name: &str, args: &[i32]) -> Result<Frame<'a>> {
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

        Ok(Frame {
            class,
            code: &code.data[8..],
            ip: 0,
            locals,
            operand_stack: Vec::new()
        })
    }

    fn execute_frame(&'a mut self) -> Result<i32> {
        loop {

            let mut f =  match self.stack.last_mut() {
                Some(f) => f,
                None => return Err(anyhow!("did not find a frame on the stack!"))
            };

            let op = f.code[f.ip as usize];

            println!("OP: {}, stack: {:?}", op, f.operand_stack);

            match op {
                26 => { // iload_0
                    f.operand_stack.push(f.locals[0]);
                    f.ip += 1;
                },
                27 => { // iload_1
                    f.operand_stack.push(f.locals[1]);
                    f.ip += 1;
                },
                116 => { // ineg
                    let v = f.pop_stack()?;
                    f.operand_stack.push(-v);
                    f.ip += 1;
                }
                96 => { // iadd
                    let a = f.pop_stack()?;
                    let b = f.pop_stack()?;
                    f.operand_stack.push(a + b);
                    f.ip += 1;
                },
                172 => { // ireturn
                    return f.pop_stack()
                },
                184 => { // invokestatic
                    let index = u16::from_be_bytes([f.code[(f.ip+1) as usize], f.code[(f.ip+2) as usize]]);

                    let static_method = f.class.const_pool.resolve_static_method(index as usize)?;

                    println!("prepping to invoke {:?}", static_method);

                    let mut nargs = 0;

                    let desc = static_method.method_desc.deref();
                    for c in desc[1..].chars() {
                        if c == ')' {
                            break;
                        }

                        nargs += 1;
                    }

                    println!("nargs = {}", nargs);


                    let mut locals = Vec::<i32>::new();
                    while nargs > 0 {
                        locals.push(f.pop_stack()?);
                        nargs -= 1;
                    }

                    f.ip += 3;

                    {
                        let fr = self.build_frame(static_method.class_name.deref(), static_method.method_name.deref(), &locals)?;
                        println!("would run frame: {:?}", fr);
                        self.stack.push(fr);
                    }



                    // f.ip += 3;
                }
                _ => {
                    println!("unknown opcode {}", op);
                }
            }
        }
    }
}

#[derive(Debug)]
struct Frame<'a> {
    class: &'a Class,
    ip: u32,
    code: &'a [u8],
    locals: Vec<i32>,
    operand_stack: Vec<i32>
}

impl <'a> Frame<'a> {
    pub fn pop_stack(&'a mut self) -> Result<i32> {
        match self.operand_stack.pop() {
            Some(v) => Ok(v),
            None => Err(anyhow!("tried popping stack but nothing found!"))
        }
    }
}
