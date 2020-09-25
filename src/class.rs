use std::rc::Rc;
use std::io::prelude::*;
use std::fs::File;
use anyhow::{Result, Context, anyhow};

#[derive(Debug)]
enum Const {
    ClassIndex(u16),

    StringLiteral(Rc<str>),
    StringIndex(u16),

    NameType(u16, u16),
    FieldMethod(u16, u16),

    Unknown,
}

#[derive(Debug)]
struct ConstPool {
    table: Vec<Const>
}

impl ConstPool {
    fn load(r: &mut ClassFileReader) -> ConstPool {
        let const_pool_size = r.u2();

        let mut table = Vec::new();

        for _ in 1..const_pool_size {
            let tag = r.u1();

            let c = match tag {
                0x01 => {
                    let size = r.u2() as i32;
                    let rc: Rc<str> = r.string(size).into();
                    Const::StringLiteral(rc)
                },
                0x07 => Const::ClassIndex(r.u2()),
                0x08 => Const::StringIndex(r.u2()),
                0x09 | 0x0a => Const::FieldMethod(r.u2(), r.u2()),
                0x0c => Const::NameType(r.u2(), r.u2()),
                _ => {
                    println!("unknown tag: {:x?}", tag);
                    Const::Unknown
                }
            };

            table.push(c)
        }

        ConstPool {table}
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
}

#[derive(Debug)]
pub struct Class {
    version_major: u16,
    version_minor: u16,
    const_pool: ConstPool,
    name: Rc<str>,
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
        &self.class_file.read_exact(&mut buf).expect("must read!");
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
        name: const_pool.resolve_str(r.u2() as usize).with_context(|| format!("error while resolving class name"))?,
        super_class: const_pool.resolve_str(r.u2() as usize).with_context(|| format!("error while resolving super class name"))?,
        interfaces: interfaces(&mut r, &const_pool)?,
        fields: fields(&mut r, &const_pool)?,
        methods: fields(&mut r, &const_pool)?,
        attributes: attr(&mut r, &const_pool)?,
        const_pool,
    };

    Ok(class)
}

fn interfaces<'a>(r: &mut ClassFileReader, const_pool: &ConstPool) -> Result<Vec<Rc<str>>> {
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