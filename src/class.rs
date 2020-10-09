use std::rc::Rc;
use std::io::prelude::*;
use std::fs::File;
use anyhow::{Result, Context, anyhow};

#[derive(Debug)]
pub struct StaticMethod {
    pub class_name: Rc<str>,
    pub method_name: Rc<str>,
    pub method_desc: Rc<str>,
}

#[derive(Debug)]
pub enum Const {
    ClassIndex(u16),

    StringLiteral(Rc<str>),
    StringIndex(u16),

    NameType(u16, u16),
    FieldMethod(u16, u16),

    Integer(i32),
    Long(i64),
    Double(f64),
    Float(f32),

    // Used for padding values of LONG and DOUBLE constants
    // see https://docs.oracle.com/javase/specs/jvms/se7/html/jvms-4.html#jvms-4.4.5
    Unusable,

    Unknown,
}

#[derive(Debug)]
pub struct ConstPool {
    size: u16,
    table: Vec<Const>
}

const CONSTANT_UTF8: u8 = 1;
const CONSTANT_INTEGER: u8 = 3;
const CONSTANT_FLOAT: u8 = 4;
const CONSTANT_LONG: u8 = 5;
const CONSTANT_DOUBLE: u8 = 6;
const CONSTANT_CLASS: u8 = 7;
const CONSTANT_STRING: u8 = 8;
const CONSTANT_FIELDREF: u8 = 9;
const CONSTANT_METHODREF: u8 = 10;
const CONSTANT_NAMEANDTYPE: u8 = 12;

impl ConstPool {
    fn load(r: &mut ClassFileReader) -> ConstPool {
        let const_pool_size = r.u2();

        let mut table = Vec::new();

        // https://docs.oracle.com/javase/specs/jvms/se7/html/jvms-4.html#jvms-4.1
        // The value of the constant_pool_count item is equal to the number of entries in the constant_pool table plus one.
        // A constant_pool index is considered valid if it is greater than zero and less than constant_pool_count
        let mut i = 1;
        while i < const_pool_size {
            let tag = r.u1();

            let c = match tag {
                CONSTANT_UTF8 => {
                    let size = r.u2() as i32;
                    let rc: Rc<str> = r.string(size).into();
                    Const::StringLiteral(rc)
                },
                CONSTANT_CLASS => Const::ClassIndex(r.u2()),
                CONSTANT_STRING => Const::StringIndex(r.u2()),
                CONSTANT_FIELDREF | CONSTANT_METHODREF => Const::FieldMethod(r.u2(), r.u2()),
                CONSTANT_NAMEANDTYPE => Const::NameType(r.u2(), r.u2()),
                CONSTANT_DOUBLE => {
                    let bytes = r.u8();

                    // Skip ahead, double uses two spots!
                    i += 1;

                    Const::Double(f64::from_bits(bytes))
                },
                CONSTANT_FLOAT => {
                    let bytes = r.u4();
                    Const::Float(f32::from_bits(bytes))
                },
                CONSTANT_LONG => {
                    let bytes = r.u8();

                    // Skip ahead, double uses two spots!
                    i += 1;

                    Const::Long(bytes as i64)
                },
                CONSTANT_INTEGER => {
                    let bytes = r.u4();
                    Const::Integer(bytes as i32)
                }
                _ => {
                    println!("unknown tag: {:?}", tag);
                    Const::Unknown
                }
            };

            println!("resolved {:?}", c);
            i += 1;

            let takes_two_entries = match c {
                Const::Double(_) | Const::Long(_) => true,
                _ => false
            };

            table.push(c);
            if takes_two_entries {
                // We inject empty value to allow easy indexing logic later
                table.push(Const::Unusable)
            }
        }

        ConstPool {size: const_pool_size, table}
    }

    pub fn resolve(&self, idx: usize) -> Result<&Const> {
        match self.table.get(idx - 1) {
            Some(c) => Ok(c),
            None => Err(anyhow!("unknown index {}", idx)),
        }
    }


    fn resolve_str(&self, idx: usize) -> Result<Rc<str>> {
        let c = self.table.get(idx - 1);

        match c {
            Some(v) => {
                match v {
                    Const::StringLiteral(s) => Ok(s.clone()),
                    Const::ClassIndex(i) => self.resolve_str(*i as usize),
                    _ => Err(anyhow!("not supported"))
                }
            },
            None => Err(anyhow!("unknown index {}!", idx))
        }
    }

    pub fn resolve_static_method(&self, idx: usize) -> Result<StaticMethod> {
        match self.table.get(idx - 1) {

            Some(Const::FieldMethod(class_idx, name_type_index)) => {
                let class_name = match self.table.get(*class_idx as usize - 1) {
                    Some(Const::ClassIndex(idx)) => {
                        match self.table.get(*idx as usize - 1) {
                            Some(Const::StringLiteral(s)) => s,
                            Some(_) | None => return Err(anyhow!("ClassIndex does not point to StringLiteral"))
                        }
                    }
                    Some(_) | None => return Err(anyhow!("class index does not point to ClassIndex"))
                };

                let (method_name, method_desc) = match self.table.get(*name_type_index as usize - 1) {
                    Some(Const::NameType(name_idx, type_idx)) => {

                        let name = match self.table.get(*name_idx as usize - 1) {
                            Some(Const::StringLiteral(s)) => s,
                            Some(_) | None => return Err(anyhow!("name_idx does not point to StringLiteral"))
                        };

                        let desc = match self.table.get(*type_idx as usize - 1) {
                            Some(Const::StringLiteral(s)) => s,
                            Some(_) | None => return Err(anyhow!("type_idx does not point to StringLiteral"))
                        };

                        (name, desc)
                    },
                    Some(_) | None => return Err(anyhow!("name_type_idx does not point to NameType"))
                };

                Ok(StaticMethod {
                    class_name: class_name.clone(),
                    method_name: method_name.clone(),
                    method_desc: method_desc.clone(),
                })
            },
            _ => Err(anyhow!("index does not point to FieldMethod"))
        }
    }
}

#[derive(Debug)]
pub struct Class {
    version_major: u16,
    version_minor: u16,
    pub const_pool: ConstPool,
    pub name: Rc<str>,
    super_class: Rc<str>,
    flags: u16,
    interfaces: Vec<Rc<str>>,
    fields: Vec<Field>,
    pub methods: Vec<Field>,
    pub attributes: Vec<Attribute>
}

#[derive(Debug)]
pub struct Field {
    flags: u16,
    pub name: Rc<str>,
    descriptor: Rc<str>,
    pub attributes: Vec<Attribute>
}

#[derive(Debug)]
pub struct Attribute {
    pub name: Rc<str>,
    pub data: Vec<u8>
}

struct ClassFileReader {
    class_file: File,
}

impl ClassFileReader {

    fn new(path: &str) -> Result<ClassFileReader> {
        let class_file = File::open(path).with_context(|| format!("failed to open class file {}", path))?;
        Ok(ClassFileReader{ class_file })
    }

    fn u1(&mut self) -> u8 {
        let mut buf: [u8 ; 1] = [0; 1];
        self.class_file.read_exact(&mut buf).expect("must read!");
        buf[0]
    }

    fn u2(&mut self) -> u16 {
        let mut buf: [u8; 2] = [0; 2];
        self.class_file.read_exact(&mut buf).expect("must read!");
        u16::from_be_bytes(buf)
    }

    fn u4(&mut self) -> u32 {
        let mut buf: [u8; 4] = [0; 4];
        self.class_file.read_exact(&mut buf).expect("must read!");
        u32::from_be_bytes(buf)
    }

    fn u8(&mut self) -> u64 {
        let mut buf: [u8; 8] = [0; 8];
        self.class_file.read_exact(&mut buf).expect("must read!");
        u64::from_be_bytes(buf)
    }

    fn bytes(&mut self, count: i32) -> Vec<u8> {
        let mut buf = vec![0u8; count as usize];
        self.class_file.read_exact(&mut buf).expect("must read");
        buf
    }

    fn string(&mut self, count: i32) -> String {
        let mut buf = vec![0u8; count as usize];
        self.class_file.read_exact(&mut buf).expect("must read");
        String::from_utf8(buf).expect("must convert!")
    }
}

pub fn load(path: &str) -> Result<Class> {
    let mut r = ClassFileReader::new(path)?;

    if r.u4() != 0xCAFEBABE {
        return Err(anyhow!("not a java file"));
    }

    let version_major = r.u2();
    let version_minor = r.u2();

    let const_pool = ConstPool::load(&mut r);

    let class = Class {
        version_major,
        version_minor,
        flags: r.u2(),
        name: const_pool.resolve_str(r.u2() as usize).with_context(|| "error while resolving class name")?,
        super_class: const_pool.resolve_str(r.u2() as usize).with_context(|| "error while resolving super class name")?,
        interfaces: interfaces(&mut r, &const_pool)?,
        fields: fields(&mut r, &const_pool)?,
        methods: fields(&mut r, &const_pool)?,
        attributes: attr(&mut r, &const_pool)?,
        const_pool,
    };

    Ok(class)
}

fn interfaces(r: &mut ClassFileReader, const_pool: &ConstPool) -> Result<Vec<Rc<str>>> {
    let count = r.u2();
    let mut v = Vec::new();
    for _ in 0..count {
        v.push(const_pool.resolve_str(r.u2() as usize)?);
    }
    Ok(v)
}

fn fields(r: &mut ClassFileReader, const_pool: &ConstPool) -> Result<Vec<Field>> {
    let count = r.u2();
    let mut v = Vec::new();
    for _ in 0..count {
        v.push(Field {
            flags: r.u2(),
            name: const_pool.resolve_str(r.u2() as usize)?,
            descriptor: const_pool.resolve_str(r.u2() as usize)?,
            attributes: attr(r, const_pool)?,
        });
    }
    Ok(v)
}

fn attr(reader: &mut ClassFileReader, const_pool: &ConstPool) -> Result<Vec<Attribute>> {
    let count = reader.u2();
    let mut v = Vec::new();
    for _ in 0..count {
        let name = const_pool.resolve_str(reader.u2() as usize)?;
        let data_size = reader.u4();
        let data = reader.bytes(data_size as i32);
        v.push(Attribute { name, data });
    }
    Ok(v)
}