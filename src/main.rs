use std::error::Error;
use std::io::prelude::*;
use std::fs::File;
use std::str;

#[derive(Debug)]
enum Const {
    ClassIndex(u16),

    StringLiteral(String),
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
                    Const::StringLiteral(r.string(size))
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

    fn resolve_str(&self, idx: usize) -> &str {
        let c = self.table.get(idx - 1);
        
        match c {
            Some(v) => {
                match v {
                    Const::StringLiteral(s) => s,
                    Const::ClassIndex(i) => self.resolve_str(*i as usize),
                    _ => "not supported"
                }
            },
            None => ""
        }
    }
}

#[derive(Debug)]
struct Class<'a> {
    const_pool: &'a ConstPool,
    name: &'a str,
    super_class: &'a str,
    flags: u16,
    interfaces: Vec<&'a str>,
    fields: Vec<Field<'a>>,
    methods: Vec<Field<'a>>,
    attributes: Vec<Attribute<'a>>
}

#[derive(Debug)]
struct Field<'a> {
    flags: u16,
    name: &'a str,
    descriptor: &'a str,
    attributes: Vec<Attribute<'a>>
}

#[derive(Debug)]
struct Attribute<'a> {
    name: &'a str,
    data: Vec<u8>
}


struct ClassFileReader {
    f: File,
}

impl ClassFileReader {
    fn u1(&mut self) -> u8 {
        let mut buf: [u8 ; 1] = [0; 1];
        &self.f.read_exact(&mut buf).expect("must read!");
        buf[0]
    }

    fn u2(&mut self) -> u16 {
        let mut buf: [u8; 2] = [0; 2];
        self.f.read_exact(&mut buf).expect("must read!");
        u16::from_be_bytes(buf)
    }

    fn u4(&mut self) -> u32 {
        let mut buf: [u8; 4] = [0; 4];
        self.f.read_exact(&mut buf).expect("must read!");
        u32::from_be_bytes(buf)
    }

    fn u8(&mut self) -> u64 {
        let mut buf: [u8; 8] = [0; 8];
        self.f.read_exact(&mut buf).expect("must read!");
        u64::from_be_bytes(buf)
    }

    fn bytes(&mut self, count: i32) -> Vec<u8> {
        let mut buf = vec![0u8; count as usize];
        self.f.read_exact(&mut buf).expect("must read");
        buf
    }

    fn string(&mut self, count: i32) -> String {
        let mut buf = vec![0u8; count as usize];
        self.f.read_exact(&mut buf).expect("must read");
        String::from_utf8(buf).expect("must convert!")
    }
}

struct Classloader { }

impl Classloader {
    fn new() -> Classloader {
        Classloader { }
    }

    fn load(&self, f: &str) -> Result<(), Box<dyn Error>> {
        let mut r = ClassFileReader{ f: File::open(f)? };

        let cafebabe = r.u4();
        if cafebabe != 0xCAFEBABE {
            return Err("not a java file".into());
        }

        let major = r.u2();
        let minor = r.u2();
        println!("major: {}, minor: {}", major, minor);

        let const_pool = ConstPool::load(&mut r);

        let class = Class {
            const_pool: &const_pool,
            name: const_pool.resolve_str(r.u2() as usize),
            super_class: const_pool.resolve_str(r.u2() as usize),
            flags: r.u2(),
            interfaces: Self::interfaces(&mut r, &const_pool),
            fields: Self::fields(&mut r, &const_pool),
            methods: Self::fields(&mut r, &const_pool),
            attributes: Self::attr(&mut r, &const_pool),
        };
            
        println!("loaded class: {:?}", class);

        Ok(())
    }

    fn interfaces<'a>(r: &mut ClassFileReader, const_pool: &'a ConstPool) -> Vec<&'a str> {
        let count = r.u2();
        let mut v = Vec::new();
        for _ in 0..count {
            v.push(const_pool.resolve_str(r.u2() as usize));
        }

        v
    }

    fn fields<'a> (r: &mut ClassFileReader, const_pool: &'a ConstPool) -> Vec<Field<'a>> {
        let count = r.u2();
        let mut v = Vec::new();
        for _ in 0..count {
            v.push(Field {
                flags: r.u2(),
                name: const_pool.resolve_str(r.u2() as usize),
                descriptor: const_pool.resolve_str(r.u2() as usize),
                attributes: Self::attr(r, const_pool),
            });
        }

        v
    }

    fn attr<'a>(reader: &mut ClassFileReader, const_pool: &'a ConstPool) -> Vec<Attribute<'a>> {
        let count = reader.u2();
        let mut v = Vec::new();

        for _ in 0..count {
            let name = const_pool.resolve_str(reader.u2() as usize);
            let data_size = reader.u4();
            let data = reader.bytes(data_size as i32);

            let a = Attribute { name: name, data: data };
            v.push(a);
        }

        v
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let cl = Classloader::new();

    cl.load("java/Add.class")?;
        
    Ok(())
}
