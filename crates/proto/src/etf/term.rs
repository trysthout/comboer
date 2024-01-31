use crate::{Encoder, Len};
use bytes::Buf;
use num_bigint::BigInt;

use super::*;

trait TermTag {
    const TAG: u8;
}
//#[derive(Debug, Clone)]
//pub enum Term {
//    SmallInteger(SmallInteger),
//    Integer(Integer),
//    SmallAtomUtf8(SmallAtomUtf8),
//    AtomUtf8(AtomUtf8),
//    Pid(Pid),
//    NewPid(NewPid),
//    Float(Float),
//    NewFloat(NewFloat),
//    Port(Port),
//    NewPort(NewPort),
//    V4Port(V4Port),
//    SmallTuple(SmallTuple),
//    LargeTuple(LargeTuple),
//    Map(Map),
//    Nil(Nil),
//    List(List),
//    Export(Export),
//    NewerReference(NewerReference),
//    Binary(Binary),
//    StringExt(StringExt),
//
//    // invalid
//    Invalid,
//}

//impl Len for Term {
//    fn len(&self) -> usize {
//        println!("self {:?}", self);
//        match self {
//            Self::SmallAtomUtf8(v) => v.len(),
//            Self::AtomUtf8(v) => v.len(),
//            Self::SmallInteger(v) => v.len(),
//            Self::Integer(v) => v.len(),
//            Self::Pid(v) => v.len(),
//            Self::NewPid(v) => v.len(),
//            Self::Float(v) => v.len(),
//            Self::NewFloat(v) => v.len(),
//            Self::Port(v) => v.len(),
//            Self::NewPort(v) => v.len(),
//            Self::V4Port(v) => v.len(),
//            Self::SmallTuple(v) => v.len(),
//            Self::LargeTuple(v) => v.len(),
//            Self::Map(v) => v.len(),
//            Self::Nil(v) => v.len(),
//            Self::List(v) => v.len(),
//            Self::Export(v) => v.len(),
//            Self::NewerReference(v) => v.len(),
//            Self::Binary(v) => v.len(),
//            Self::StringExt(v) => v.len(),
//        }
//    }
//}

///
/// 1	2	Len
/// 118	Len	AtomName
///
#[derive(Debug, Clone, PartialEq)]
pub struct AtomUtf8(pub String);

impl TermTag for AtomUtf8 {
    const TAG: u8 = ATOM_UTF8_EXT;
}

impl Encoder for AtomUtf8 {
    type Error = anyhow::Error;
    fn encode<W: std::io::Write>(&self, w: &mut W) -> Result<(), Self::Error> {
        w.write_all(&ATOM_UTF8_EXT.to_be_bytes())?;
        w.write_all(&(self.0.len() as u16).to_be_bytes())?;
        w.write_all(self.0.as_bytes())?;
        Ok(())
    }
}

impl<'a> From<&'a [u8]> for AtomUtf8 {
    fn from(mut value: &'a [u8]) -> Self {
        value.get_u8();
        let n = value.get_u16() as usize;
        let (node_bytes, _) = value.split_at(n);
        let s = String::from_utf8_lossy(node_bytes).to_string();
        AtomUtf8(s)
    }
}

impl Len for AtomUtf8 {
    fn len(&self) -> usize {
        1 + 2 + self.0.len()
    }
}

///
/// 1	 1	    Len
/// 119  Len	AtomName
///
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SmallAtomUtf8(pub String);

impl TermTag for SmallAtomUtf8 {
    const TAG: u8 = SMALL_ATOM_UTF8_EXT;
}

impl Encoder for SmallAtomUtf8 {
    type Error = anyhow::Error;
    fn encode<W: std::io::Write>(&self, w: &mut W) -> Result<(), anyhow::Error> {
        w.write_all(&SMALL_ATOM_UTF8_EXT.to_be_bytes())?;
        w.write_all(&(self.0.len() as u8).to_be_bytes())?;
        w.write_all(self.0.as_bytes())?;
        Ok(())
    }
}

impl<'a> From<&'a [u8]> for SmallAtomUtf8 {
    fn from(mut value: &'a [u8]) -> Self {
        value.get_u8();
        let n = value.get_u8() as usize;
        let (node_bytes, _) = value.split_at(n);
        let s = String::from_utf8_lossy(node_bytes).to_string();
        value.advance(n);
        SmallAtomUtf8(s)
    }
}

impl Len for SmallAtomUtf8 {
    fn len(&self) -> usize {
        1 + 1 + self.0.len()
    }
}

///
/// 1	 N	    4	4	    1
/// 103	 Node	ID	Serial	Creation
///
#[derive(Debug, Clone, PartialEq)]
pub struct Pid {
    pub node: SmallAtomUtf8,
    pub id: u32,
    pub serial: u32,
    pub creation: u8,
}

impl TermTag for Pid {
    const TAG: u8 = PID_EXT;
}

impl Encoder for Pid {
    type Error = anyhow::Error;
    fn encode<W: std::io::Write>(&self, w: &mut W) -> Result<(), anyhow::Error> {
        w.write_all(&PID_EXT.to_be_bytes())?;
        self.node.encode(w)?;
        w.write_all(&self.id.to_be_bytes())?;
        w.write_all(&self.serial.to_be_bytes())?;
        w.write_all(&self.creation.to_be_bytes())?;
        Ok(())
    }
}

impl<'a> From<&'a [u8]> for Pid {
    fn from(mut value: &'a [u8]) -> Self {
        value.get_u8();
        let atom = SmallAtomUtf8::from(value);
        value.advance(atom.len());
        let id = value.get_u32();
        let serial = value.get_u32();
        let creation = value.get_u8();
        Pid {
            node: atom,
            id,
            serial,
            creation,
        }
    }
}

impl Len for Pid {
    fn len(&self) -> usize {
        1 + self.node.len() + 4 + 4 + 1
    }
}

///
///
/// 1	N	    4	4	    4
/// 88	Node	ID	Serial	Creation
///
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NewPid {
    pub node: SmallAtomUtf8,
    pub id: u32,
    pub serial: u32,
    pub creation: u32,
}

impl TermTag for NewPid {
    const TAG: u8 = NEW_PID_EXT;
}

impl Encoder for NewPid {
    type Error = anyhow::Error;
    fn encode<W: std::io::Write>(&self, w: &mut W) -> Result<(), anyhow::Error> {
        w.write_all(&NEW_PID_EXT.to_be_bytes())?;
        self.node.encode(w)?;
        w.write_all(&self.id.to_be_bytes())?;
        w.write_all(&self.serial.to_be_bytes())?;
        w.write_all(&self.creation.to_be_bytes())?;

        Ok(())
    }
}

impl<'a> From<&'a [u8]> for NewPid {
    fn from(mut value: &'a [u8]) -> Self {
        value.get_u8();
        let atom = SmallAtomUtf8::from(value);
        value.advance(atom.len());
        let id = value.get_u32();
        let serial = value.get_u32();
        let creation = value.get_u32();
        NewPid {
            node: atom,
            id,
            serial,
            creation,
        }
    }
}

impl Len for NewPid {
    fn len(&self) -> usize {
        1 + self.node.len() + 4 + 4 + 4
    }
}

///
///
/// 1	1
/// 97	Int
///
#[derive(Debug, Clone, PartialEq)]
pub struct SmallInteger(pub u8);

impl TermTag for SmallInteger {
    const TAG: u8 = SMALL_INTEGER_EXT;
}

impl Encoder for SmallInteger {
    type Error = anyhow::Error;
    fn encode<W: std::io::Write>(&self, w: &mut W) -> Result<(), anyhow::Error> {
        w.write_all(&SMALL_INTEGER_EXT.to_be_bytes())?;
        w.write_all(&self.0.to_be_bytes())?;

        Ok(())
    }
}

impl From<&[u8]> for SmallInteger {
    fn from(mut value: &[u8]) -> Self {
        value.get_u8();
        let inner = value.get_u8();
        SmallInteger(inner)
    }
}

impl Len for SmallInteger {
    fn len(&self) -> usize {
        1 + 1
    }
}

///
/// 1	4
/// 98	Int
///
#[derive(Debug, Clone, PartialEq)]
pub struct Integer(pub i32);

impl TermTag for Integer {
    const TAG: u8 = INTEGER_EXT;
}

impl Encoder for Integer {
    type Error = anyhow::Error;
    fn encode<W: std::io::Write>(&self, w: &mut W) -> Result<(), anyhow::Error> {
        w.write_all(&INTEGER_EXT.to_be_bytes())?;
        w.write_all(&self.0.to_be_bytes())?;

        Ok(())
    }
}

impl<'a> From<&'a [u8]> for Integer {
    fn from(mut value: &'a [u8]) -> Self {
        value.get_u8();
        let inner = value.get_i32();
        Integer(inner)
    }
}

impl Len for Integer {
    fn len(&self) -> usize {
        1 + 4
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum FixedInteger {
    SmallInteger(SmallInteger),
    Integer(Integer),
}

impl Encoder for FixedInteger {
    type Error = anyhow::Error;
    fn encode<W: std::io::Write>(&self, w: &mut W) -> Result<(), anyhow::Error> {
        match self {
            FixedInteger::SmallInteger(v) => v.encode(w),
            FixedInteger::Integer(v) => v.encode(w),
        }
    }
}

impl<'a> From<&'a [u8]> for FixedInteger {
    fn from(value: &'a [u8]) -> Self {
        match value[0] {
            SMALL_INTEGER_EXT => FixedInteger::SmallInteger(SmallInteger::from(value)),
            INTEGER_EXT => FixedInteger::Integer(Integer::from(value)),
            _ => unreachable!(),
        }
    }
}

impl Len for FixedInteger {
    fn len(&self) -> usize {
        match self {
            FixedInteger::SmallInteger(v) => v.len(),
            FixedInteger::Integer(v) => v.len(),
        }
    }
}

///
/// 1	31
/// 99	Float string
///
#[derive(Debug, Clone, PartialEq)]
pub struct Float(String);

impl TermTag for Float {
    const TAG: u8 = FLOAT_EXT;
}

impl Encoder for Float {
    type Error = anyhow::Error;
    fn encode<W: std::io::Write>(&self, w: &mut W) -> Result<(), anyhow::Error> {
        w.write_all(&FLOAT_EXT.to_be_bytes())?;
        w.write_all(self.0.as_bytes())?;

        Ok(())
    }
}

impl<'a> From<&'a [u8]> for Float {
    fn from(mut value: &'a [u8]) -> Self {
        value.get_u8();
        let (inner, _) = value.split_at(31);
        let inner = format!("{:0<31}", String::from_utf8_lossy(inner));
        Float(inner)
    }
}

impl Len for Float {
    fn len(&self) -> usize {
        1 + 31
    }
}

///
/// 1	8
/// 70	IEEE float
///
///
#[derive(Debug, Clone, PartialEq)]
pub struct NewFloat(pub f64);

impl TermTag for NewFloat {
    const TAG: u8 = NEW_FLOAT_EXT;
}

impl Encoder for NewFloat {
    type Error = anyhow::Error;
    fn encode<W: std::io::Write>(&self, w: &mut W) -> Result<(), anyhow::Error> {
        w.write_all(&NEW_FLOAT_EXT.to_be_bytes())?;
        w.write_all(&self.0.to_be_bytes())?;

        Ok(())
    }
}

impl<'a> From<&'a [u8]> for NewFloat {
    fn from(mut value: &'a [u8]) -> Self {
        value.get_u8();
        NewFloat(value.get_f64())
    }
}

impl Len for NewFloat {
    fn len(&self) -> usize {
        1 + 8
    }
}

///
/// 1	N	 4	1
/// 102	Node ID	Creation
///
#[derive(Debug, Clone, PartialEq)]
pub struct Port {
    pub node: SmallAtomUtf8,
    pub id: u32,
    pub creation: u8,
}

impl TermTag for Port {
    const TAG: u8 = PORT_EXT;
}

impl Encoder for Port {
    type Error = anyhow::Error;
    fn encode<W: std::io::Write>(&self, w: &mut W) -> Result<(), anyhow::Error> {
        w.write_all(&PORT_EXT.to_be_bytes())?;
        self.node.encode(w)?;
        w.write_all(&self.id.to_be_bytes())?;
        w.write_all(&self.creation.to_be_bytes())?;

        Ok(())
    }
}

impl<'a> From<&'a [u8]> for Port {
    fn from(mut value: &'a [u8]) -> Self {
        value.get_u8();
        let atom = SmallAtomUtf8::from(value);
        value.advance(atom.len());
        let id = value.get_u32();
        let creation = value.get_u8();
        Port {
            node: atom,
            id,
            creation,
        }
    }
}

impl Len for Port {
    fn len(&self) -> usize {
        1 + self.node.len() + 4 + 1
    }
}

///
/// 1	N	    4	 4
/// 89	Node    ID	 Creation
///
#[derive(Debug, Clone, PartialEq)]
pub struct NewPort {
    pub node: SmallAtomUtf8,
    pub id: u32,
    pub creation: u32,
}

impl TermTag for NewPort {
    const TAG: u8 = NEW_PORT_EXT;
}

impl Encoder for NewPort {
    type Error = anyhow::Error;
    fn encode<W: std::io::Write>(&self, w: &mut W) -> Result<(), anyhow::Error> {
        w.write_all(&NEW_PORT_EXT.to_be_bytes())?;
        self.node.encode(w)?;
        w.write_all(&self.id.to_be_bytes())?;
        w.write_all(&self.creation.to_be_bytes())?;

        Ok(())
    }
}

impl<'a> From<&'a [u8]> for NewPort {
    fn from(mut value: &'a [u8]) -> Self {
        value.get_u8();
        let atom = SmallAtomUtf8::from(value);
        value.advance(atom.len());
        let id = value.get_u32();
        let creation = value.get_u32();
        NewPort {
            node: atom,
            id,
            creation,
        }
    }
}

impl Len for NewPort {
    fn len(&self) -> usize {
        1 + self.node.len() + 4 + 4
    }
}

///
/// 1	N	    8	4
/// 120	Node	ID	Creation
///
#[derive(Debug, Clone, PartialEq)]
pub struct V4Port {
    pub node: SmallAtomUtf8,
    pub id: u64,
    pub creation: u32,
}

impl TermTag for V4Port {
    const TAG: u8 = V4_PORT_EXT;
}

impl Encoder for V4Port {
    type Error = anyhow::Error;
    fn encode<W: std::io::Write>(&self, w: &mut W) -> Result<(), anyhow::Error> {
        w.write_all(&V4_PORT_EXT.to_be_bytes())?;
        self.node.encode(w)?;
        w.write_all(&self.id.to_be_bytes())?;
        w.write_all(&self.creation.to_be_bytes())?;
        Ok(())
    }
}

impl<'a> From<&'a [u8]> for V4Port {
    fn from(mut value: &'a [u8]) -> Self {
        value.get_u8();
        let atom = SmallAtomUtf8::from(value);
        value.advance(atom.len());
        let id = value.get_u64();
        let creation = value.get_u32();
        V4Port {
            node: atom,
            id,
            creation,
        }
    }
}

impl Len for V4Port {
    fn len(&self) -> usize {
        1 + self.node.len() + 8 + 4
    }
}

///
/// 1	 1	    N
/// 104	 Arity	Elements
///
#[derive(Debug, Clone, PartialEq)]
pub struct SmallTuple {
    pub arity: u8,
    pub elems: Vec<Term>,
}

impl TermTag for SmallTuple {
    const TAG: u8 = SMALL_TUPLE_EXT;
}

impl Encoder for SmallTuple {
    type Error = anyhow::Error;
    fn encode<W: std::io::Write>(&self, w: &mut W) -> Result<(), anyhow::Error> {
        w.write_all(&SMALL_TUPLE_EXT.to_be_bytes())?;
        w.write_all(&self.arity.to_be_bytes())?;
        for e in &self.elems {
            e.encode(w)?;
        }

        Ok(())
    }
}

impl<'a> From<&'a [u8]> for SmallTuple {
    fn from(mut value: &'a [u8]) -> Self {
        value.get_u8();
        let arity = value.get_u8();
        let mut elems = Vec::with_capacity(arity as usize);
        (0..arity).for_each(|_| {
            let term = Term::from(value);
            value.advance(term.len());
            elems.push(term);
        });

        SmallTuple { arity, elems }
    }
}

impl From<Vec<Term>> for SmallTuple {
    fn from(value: Vec<Term>) -> Self {
        Self {
            arity: value.len() as u8,
            elems: value,
        }
    }
}

impl Len for SmallTuple {
    fn len(&self) -> usize {
        1 + 1 + self.elems.iter().map(|e| e.len()).sum::<usize>()
    }
}

///
/// 1	 4	    N
/// 105	 Arity	Elements
///
#[derive(Debug, Clone, PartialEq)]
pub struct LargeTuple {
    pub arity: u32,
    pub elems: Vec<Term>,
}

impl TermTag for LargeTuple {
    const TAG: u8 = LARGE_TUPLE_EXT;
}

impl Encoder for LargeTuple {
    type Error = anyhow::Error;
    fn encode<W: std::io::Write>(&self, w: &mut W) -> Result<(), anyhow::Error> {
        let _ = w.write_all(&LARGE_TUPLE_EXT.to_be_bytes());
        let _ = w.write_all(&self.arity.to_be_bytes());
        for e in &self.elems {
            e.encode(w)?
        }

        Ok(())
    }
}

impl<'a> From<&'a [u8]> for LargeTuple {
    fn from(mut value: &'a [u8]) -> Self {
        value.get_u8();
        let arity = value.get_u32();
        let mut elems = Vec::with_capacity(10);
        (0..arity).for_each(|_| {
            let term = Term::from(value);
            value.advance(term.len());
            elems.push(term);
        });

        LargeTuple { arity, elems }
    }
}

impl Len for LargeTuple {
    fn len(&self) -> usize {
        1 + 4 + self.elems.iter().map(|e| e.len()).sum::<usize>()
    }
}

///
/// 1	4	    N
/// 116	Arity	Pairs
///
#[derive(Debug, Clone, PartialEq)]
pub struct Map {
    pub arity: u32,
    pub pairs: Vec<Term>,
}

impl TermTag for Map {
    const TAG: u8 = MAP_EXT;
}

impl Encoder for Map {
    type Error = anyhow::Error;
    fn encode<W: std::io::Write>(&self, w: &mut W) -> Result<(), anyhow::Error> {
        let _ = w.write_all(&MAP_EXT.to_be_bytes());
        let _ = w.write_all(&self.arity.to_be_bytes());
        for i in 0..self.arity * 2 {
            self.pairs[i as usize].encode(w)?;
        }

        Ok(())
    }
}

impl<'a> From<&'a [u8]> for Map {
    fn from(mut value: &'a [u8]) -> Self {
        value.get_u8();
        let arity = value.get_u32();
        let mut pairs = Vec::with_capacity(10);
        (0..arity * 2).for_each(|_| {
            let term = Term::from(value);
            value.advance(term.len());
            pairs.push(term);
        });

        Map { arity, pairs }
    }
}

impl Len for Map {
    fn len(&self) -> usize {
        1 + 4 + self.pairs.iter().map(|e| e.len()).sum::<usize>()
    }
}

///
/// 1
/// 106
///
#[derive(Debug, Clone, PartialEq)]
pub struct Nil;

impl TermTag for Nil {
    const TAG: u8 = NIL_EXT;
}

impl Encoder for Nil {
    type Error = anyhow::Error;
    fn encode<W: std::io::Write>(&self, w: &mut W) -> Result<(), anyhow::Error> {
        w.write_all(&NIL_EXT.to_be_bytes())?;

        Ok(())
    }
}

impl<'a> From<&'a [u8]> for Nil {
    fn from(mut value: &'a [u8]) -> Self {
        value.get_u8();
        Nil
    }
}

impl Len for Nil {
    fn len(&self) -> usize {
        1
    }
}

///
/// 1	2	Len
/// 107	Length	Characters
///
#[derive(Debug, Clone, PartialEq)]
pub struct StringExt {
    pub length: u16,
    pub chars: Vec<u8>,
}

impl TermTag for StringExt {
    const TAG: u8 = STRING_EXT;
}

impl Encoder for StringExt {
    type Error = anyhow::Error;
    fn encode<W: std::io::Write>(&self, w: &mut W) -> Result<(), Self::Error> {
        w.write_all(&STRING_EXT.to_be_bytes())?;
        w.write_all(&self.length.to_be_bytes())?;
        w.write_all(&self.chars)?;

        Ok(())
    }
}

impl<'a> From<&'a [u8]> for StringExt {
    fn from(mut value: &'a [u8]) -> Self {
        value.get_u8();
        let length = value.get_u16();
        let (chars, _) = value.split_at(length as usize);
        value.advance(length as usize);
        StringExt {
            length,
            chars: chars.to_vec(),
        }
    }
}

impl Len for StringExt {
    fn len(&self) -> usize {
        1 + 2 + self.chars.len()
    }
}

///
/// 1	4
/// 108	Length	Elements	Tail
///
#[derive(Debug, Clone, PartialEq)]
pub struct List {
    pub length: u32,
    pub elems: Vec<Term>,
    pub tail: Box<Term>,
}

impl TermTag for List {
    const TAG: u8 = LIST_EXT;
}

impl Encoder for List {
    type Error = anyhow::Error;
    fn encode<W: std::io::Write>(&self, w: &mut W) -> Result<(), anyhow::Error> {
        w.write_all(&LIST_EXT.to_be_bytes())?;
        w.write_all(&self.length.to_be_bytes())?;
        for i in 0..self.length {
            self.elems[i as usize].encode(w)?;
        }
        self.tail.encode(w)?;

        Ok(())
    }
}

impl<'a> From<&'a [u8]> for List {
    fn from(mut value: &'a [u8]) -> Self {
        println!("value {:?}", value);
        value.get_u8();
        let length = value.get_u32();
        let mut elems = Vec::with_capacity(length as usize);
        (0..length).for_each(|_| {
            let term = Term::from(value);
            println!("value1 term {:?} {:?}", term, value);
            value.advance(term.len());
            elems.push(term);
        });

        if !value.is_empty() && value[0] == NIL_EXT {
            let tail = Term::from(value);
            value.advance(tail.len());
            return List {
                length,
                elems,
                tail: Box::new(tail),
            };
        }

        let tail = Term::from(value);
        value.advance(tail.len());

        List {
            length,
            elems,
            tail: Box::new(tail),
        }
    }
}

impl Len for List {
    fn len(&self) -> usize {
        1 + 4 + self.elems.iter().map(|x| x.len()).sum::<usize>() + self.tail.len()
    }
}

///
/// 1	4	Len
/// 109	Len	Data
///
#[derive(Debug, Clone, PartialEq)]
pub struct Binary {
    pub length: u32,
    pub data: Vec<u8>,
}

impl TermTag for Binary {
    const TAG: u8 = BINARY_EXT;
}

impl Encoder for Binary {
    type Error = anyhow::Error;
    fn encode<W: std::io::Write>(&self, w: &mut W) -> Result<(), anyhow::Error> {
        w.write_all(&BINARY_EXT.to_be_bytes())?;
        w.write_all(&self.length.to_be_bytes())?;
        w.write_all(&self.data[0..self.length as usize])?;

        Ok(())
    }
}

impl From<&[u8]> for Binary {
    fn from(mut value: &[u8]) -> Self {
        value.get_u8();
        let length = value.get_u32();
        let (data, _) = value.split_at(length as usize);
        value.advance(length as usize);
        Binary {
            length,
            data: data.to_vec(),
        }
    }
}

impl Len for Binary {
    fn len(&self) -> usize {
        1 + 4 + self.data.len()
    }
}

///
/// 1	1	1	    n
/// 110	n	Sign	d(0) ... d(n-1)
///
#[derive(Debug, Clone, PartialEq)]
pub struct SmallBig {
    pub length: u8,
    pub sign: Sign,
    pub n: BigInt,
}

impl TermTag for SmallBig {
    const TAG: u8 = SMALL_BIG_EXT;
}

impl Encoder for SmallBig {
    type Error = anyhow::Error;
    fn encode<W: std::io::Write>(&self, w: &mut W) -> Result<(), anyhow::Error> {
        w.write_all(&SMALL_BIG_EXT.to_be_bytes())?;
        w.write_all(&self.length.to_be_bytes())?;
        let sign: u8 = (&self.sign).into();
        w.write_all(&sign.to_be_bytes())?;
        w.write_all(&self.n.to_bytes_le().1)?;

        Ok(())
    }
}

impl<'a> From<&'a [u8]> for SmallBig {
    fn from(mut value: &'a [u8]) -> Self {
        value.get_u8();
        let length = value.get_u8();
        let sign: Sign = value.get_u8().into();
        let (data, _) = value.split_at(length as usize);
        value.advance(length as usize);
        let n = BigInt::from_bytes_le((&sign).into(), data);
        SmallBig { length, sign, n }
    }
}

impl Len for SmallBig {
    fn len(&self) -> usize {
        1 + 1 + 1 + self.length as usize
    }
}

///
///  1	4	1	    n
/// 111	n	Sign	d(0) ... d(n-1)
///
#[derive(Debug, Clone, PartialEq)]
pub struct LargeBig {
    pub length: u32,
    pub sign: Sign,
    pub n: BigInt,
}

impl TermTag for LargeBig {
    const TAG: u8 = LARGE_BIG_EXT;
}

impl Encoder for LargeBig {
    type Error = anyhow::Error;
    fn encode<W: std::io::Write>(&self, w: &mut W) -> Result<(), anyhow::Error> {
        w.write_all(&LARGE_BIG_EXT.to_be_bytes())?;
        w.write_all(&self.length.to_be_bytes())?;
        let sign: u8 = (&self.sign).into();
        w.write_all(&sign.to_be_bytes())?;
        w.write_all(&self.n.to_bytes_le().1)?;

        Ok(())
    }
}

impl<'a> From<&'a [u8]> for LargeBig {
    fn from(mut value: &'a [u8]) -> Self {
        value.get_u8();
        let length = value.get_u32();
        let sign: Sign = value.get_u8().into();
        let (data, _) = value.split_at(length as usize);
        value.advance(length as usize);
        let n = BigInt::from_bytes_le((&sign).into(), data);
        LargeBig { length, sign, n }
    }
}

///
///
/// 1	2	N	    4	        N'
/// 90	Len	Node	Creation	ID ...
///
#[derive(Debug, Clone, PartialEq)]
pub struct NewerReference {
    pub length: u16,
    pub node: SmallAtomUtf8,
    pub creation: u32,
    pub id: Vec<u32>,
}

impl TermTag for NewerReference {
    const TAG: u8 = NEWER_REFERENCE_EXT;
}

impl Encoder for NewerReference {
    type Error = anyhow::Error;
    fn encode<W: std::io::Write>(&self, w: &mut W) -> Result<(), anyhow::Error> {
        w.write_all(&NEWER_REFERENCE_EXT.to_be_bytes())?;
        w.write_all(&self.length.to_be_bytes())?;
        self.node.encode(w)?;
        w.write_all(&self.creation.to_be_bytes())?;
        for id in self.id.iter() {
            w.write_all(&id.to_be_bytes())?;
        }

        Ok(())
    }
}

impl<'a> From<&'a [u8]> for NewerReference {
    fn from(mut value: &'a [u8]) -> Self {
        value.get_u8();
        let length = value.get_u16();
        let node = SmallAtomUtf8::from(value);
        value.advance(node.len());
        let creation = value.get_u32();
        let mut id = Vec::with_capacity(length as usize);
        (0..length).for_each(|_| {
            id.push(value.get_u32());
        });
        NewerReference {
            length,
            node,
            creation,
            id,
        }
    }
}

impl Len for NewerReference {
    fn len(&self) -> usize {
        1 + 2 + self.node.len() + 4 + self.id.len() * 4
    }
}

///
/// 1	4	1	    Len
/// 77	Len	Bits	Data
///
#[derive(Debug, Clone, PartialEq)]
pub struct BitBinary {
    pub length: u32,
    pub bits: u8,
    pub data: Vec<u8>,
}

impl TermTag for BitBinary {
    const TAG: u8 = BIT_BINARY_EXT;
}

impl Encoder for BitBinary {
    type Error = anyhow::Error;
    fn encode<W: std::io::Write>(&self, w: &mut W) -> Result<(), anyhow::Error> {
        w.write_all(&BIT_BINARY_EXT.to_be_bytes())?;
        w.write_all(&self.length.to_be_bytes())?;
        w.write_all(&self.bits.to_be_bytes())?;
        if !self.data.is_empty() {
            w.write_all(&self.data[0..self.length as usize - 1])?;
            w.write_all(&(self.data[self.length as usize - 1] << (8 - self.bits)).to_be_bytes())?;
        }

        Ok(())
    }
}

impl<'a> From<&'a [u8]> for BitBinary {
    fn from(mut value: &'a [u8]) -> Self {
        value.get_u8();
        let length = value.get_u32();
        let bits = value.get_u8();
        let mut data = value.chunk().to_vec();
        if !data.is_empty() {
            let last = data[length as usize - 1] >> (8 - bits);
            data[length as usize - 1] = last;
        }

        BitBinary { length, bits, data }
    }
}

impl Len for BitBinary {
    fn len(&self) -> usize {
        1 + 4 + 1 + self.data.len()
    }
}

///
///
/// 1	N1	    N2	        N3
/// 113	Module	Function	Arity
///
#[derive(Debug, Clone, PartialEq)]
pub struct Export {
    pub module: SmallAtomUtf8,
    pub fun: Box<Term>,
    pub arity: SmallInteger,
}

impl TermTag for Export {
    const TAG: u8 = EXPORT_EXT;
}

impl Encoder for Export {
    type Error = anyhow::Error;
    fn encode<W: std::io::Write>(&self, w: &mut W) -> Result<(), anyhow::Error> {
        w.write_all(&EXPORT_EXT.to_be_bytes())?;
        self.module.encode(w)?;
        self.fun.encode(w)?;
        self.arity.encode(w)?;

        Ok(())
    }
}

impl<'a> From<&'a [u8]> for Export {
    fn from(mut value: &'a [u8]) -> Self {
        value.get_u8();
        let module = SmallAtomUtf8::from(value);
        value.advance(module.len());
        let fun = Term::from(value);
        value.advance(fun.len());
        let arity = SmallInteger::from(value);
        value.advance(arity.len());

        Export {
            module,
            fun: Box::new(fun),
            arity,
        }
    }
}

impl Len for Export {
    fn len(&self) -> usize {
        1 + self.module.len() + self.fun.len() + self.arity.len()
    }
}

///
/// 1	4	    1	    16	    4	    4	    N1	    N2	        N3	    N4	N5
/// 112	Size	Arity	Uniq	Index	NumFree	Module	OldIndex	OldUniq	Pid	Free Vars
///
#[derive(Debug, Clone, PartialEq)]
pub struct NewFun {
    pub size: u32,
    pub arity: u8,
    pub uniq: [u8; 16],
    pub index: u32,
    pub num_free: u32,
    pub module: SmallAtomUtf8,
    pub old_index: FixedInteger,
    pub old_uniq: FixedInteger,
    pub pid: NewPid,
    pub free_vars: Vec<Term>,
}

impl TermTag for NewFun {
    const TAG: u8 = NEW_FUN_EXT;
}

impl Encoder for NewFun {
    type Error = anyhow::Error;
    fn encode<W: std::io::Write>(&self, w: &mut W) -> Result<(), anyhow::Error> {
        w.write_all(&NEW_FUN_EXT.to_be_bytes())?;
        w.write_all(&self.size.to_be_bytes())?;
        w.write_all(&self.arity.to_be_bytes())?;
        w.write_all(&self.uniq)?;
        w.write_all(&self.index.to_be_bytes())?;
        w.write_all(&self.num_free.to_be_bytes())?;
        self.module.encode(w)?;
        self.old_index.encode(w)?;
        self.old_uniq.encode(w)?;
        self.pid.encode(w)?;
        for v in &self.free_vars {
            v.encode(w)?;
        }

        Ok(())
    }
}

impl<'a> From<&'a [u8]> for NewFun {
    fn from(mut value: &'a [u8]) -> Self {
        value.get_u8();
        let size = value.get_u32();
        let arity = value.get_u8();
        let (uniq, mut value) = value.split_at(16);
        let index = value.get_u32();
        let num_free = value.get_u32();
        let module = SmallAtomUtf8::from(value);
        value.advance(module.len());
        let old_index = FixedInteger::from(value);
        value.advance(old_index.len());
        let old_uniq = FixedInteger::from(value);
        value.advance(old_uniq.len());
        let pid = NewPid::from(value);
        value.advance(pid.len());

        let mut free_vars = Vec::with_capacity(num_free as usize);
        (0..num_free).for_each(|_| {
            let term = Term::from(value);
            value.advance(term.len());
            free_vars.push(term);
        });

        NewFun {
            size,
            arity,
            uniq: uniq.try_into().unwrap(),
            index,
            num_free,
            module,
            old_index,
            old_uniq,
            pid,
            free_vars,
        }
    }
}

impl Len for NewFun {
    fn len(&self) -> usize {
        1 + 4
            + 1
            + 16
            + 4
            + 4
            + self.module.len()
            + self.old_index.len()
            + self.old_uniq.len()
            + self.pid.len()
            + self.free_vars.iter().map(|x| x.len()).sum::<usize>()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Local(Vec<u8>);

impl TermTag for Local {
    const TAG: u8 = LOCAL_EXT;
}

impl Encoder for Local {
    type Error = anyhow::Error;
    fn encode<W: std::io::Write>(&self, w: &mut W) -> Result<(), Self::Error> {
        w.write_all(&LOCAL_EXT.to_be_bytes())?;
        w.write_all(&self.0)?;
        Ok(())
    }
}

impl<'a> From<&'a [u8]> for Local {
    fn from(mut value: &'a [u8]) -> Self {
        value.get_u8();
        let data = value.chunk();
        Local(data.to_vec())
    }
}

impl Len for Local {
    fn len(&self) -> usize {
        1 + self.0.len()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PidOrAtom {
    Pid(NewPid),
    Atom(SmallAtomUtf8),
}

impl Encoder for PidOrAtom {
    type Error = anyhow::Error;
    fn encode<W: std::io::Write>(&self, w: &mut W) -> Result<(), Self::Error> {
        match self {
            Self::Pid(v) => v.encode(w),
            Self::Atom(v) => v.encode(w),
        }
    }
}

impl From<&[u8]> for PidOrAtom {
    fn from(value: &[u8]) -> Self {
        match value[0] {
            NEW_PID_EXT => Self::Pid(NewPid::from(value)),
            SMALL_ATOM_UTF8_EXT => Self::Atom(SmallAtomUtf8::from(value)),
            _ => unreachable!(),
        }
    }
}

impl Len for PidOrAtom {
    fn len(&self) -> usize {
        match self {
            Self::Pid(v) => v.len(),
            Self::Atom(v) => v.len(),
        }
    }
}

impl From<&NewPid> for PidOrAtom {
    fn from(value: &NewPid) -> Self {
        Self::Pid(value.clone())
    }
}

impl From<&SmallAtomUtf8> for PidOrAtom {
    fn from(value: &SmallAtomUtf8) -> Self {
        Self::Atom(value.clone())
    }
}

impl From<&PidOrAtom> for PidOrAtom {
    fn from(value: &PidOrAtom) -> Self {
        value.clone()
    }
}

macro_rules! impl_from_into_term {
    ($($t:ident),+) => {
        #[derive(Debug, Clone, PartialEq)]
        pub enum Term {
            $($t($t),)+
        }

        impl Encoder for Term {
            type Error = anyhow::Error;
            fn encode<W: std::io::Write>(&self, w: &mut W) -> Result<(), Self::Error> {
                match self {
                    $(Self::$t(v) => v.encode(w), )+
                }
            }
        }

        impl From<&[u8]> for Term {
            fn from(value: &[u8]) -> Self {
                match value[0] {
                    $($t::TAG => Self::$t($t::from(value)),)+
                    _ => unreachable!()
                }
            }
        }

        impl Len for Term {
            fn len(&self) -> usize {
                match self {
                    $(Self::$t(v) => v.len(), )+
                }
            }
        }

        $(
            impl From<$t> for Term {
                fn from(value: $t) -> Self {
                    Term::$t(value)
                }
            }

            impl From<&$t> for Term {
                fn from(value: &$t) -> Self {
                    Term::$t(value.clone())
                }
            }

            impl TryFrom<Term> for $t {
                type Error = anyhow::Error;

                fn try_from(value: Term) -> Result<Self, Self::Error> {
                    if let Term::$t(v) = value {
                        Ok(v)
                    } else {
                        Err(anyhow::anyhow!("cannot convert value type"))
                    }
                }
            }

            impl TryFrom<&Term> for $t {
                type Error = anyhow::Error;

                fn try_from(value: &Term) -> Result<Self, Self::Error> {
                    if let Term::$t(v) = value {
                        Ok(v.clone())
                    } else {
                        Err(anyhow::anyhow!("cannot convert value type"))
                    }
                }
            }

        )*
    };
}

impl_from_into_term!(
    SmallInteger,
    Integer,
    SmallAtomUtf8,
    AtomUtf8,
    Pid,
    NewPid,
    Float,
    NewFloat,
    Port,
    NewPort,
    V4Port,
    SmallTuple,
    LargeTuple,
    Map,
    Nil,
    List,
    Export,
    NewerReference,
    Binary,
    StringExt,
    SmallBig,
    NewFun
);

#[cfg(test)]
mod test {
    use bytes::{BufMut, BytesMut};

    use super::*;

    #[test]
    fn new_pid() {
        let pid = NewPid {
            node: SmallAtomUtf8("test".to_string()),
            id: 11_u32,
            serial: 12_u32,
            creation: 13_u32,
        };

        let mut enc = BytesMut::new().writer();
        pid.encode(&mut enc).unwrap();
        let enc = enc.into_inner();
        println!("{:?}", &enc[..]);
        let pid = Term::from(&enc[..]);
        println!("{:?} {:?}", pid, pid.len());
    }

    #[test]
    fn small_tuple() {
        let s = SmallTuple {
            arity: 3,
            elems: vec![
                Term::SmallAtomUtf8(SmallAtomUtf8("test1".to_string())),
                Term::SmallAtomUtf8(SmallAtomUtf8("test2".to_string())),
                Term::SmallAtomUtf8(SmallAtomUtf8("test3".to_string())),
            ],
        };

        let mut enc = vec![];
        s.encode(&mut enc).unwrap();
        println!("{:?}", &enc[..]);
        let de = SmallTuple::from(&enc[..]);
        println!("{:?}", de);
    }

    #[test]
    fn map() {
        //let enc = vec![
        //    116, 0, 0, 0, 2, 119, 1, 97, 119, 1, 97, 119, 1, 98, 119, 1, 98,
        //];
        let enc = [
            116, 0, 0, 0, 2, 97, 1, 88, 119, 8, 97, 64, 102, 101, 100, 111, 114, 97, 0, 0, 0, 115,
            0, 0, 0, 0, 101, 129, 113, 138, 119, 1, 98, 97, 1,
        ];
        let m = Map::from(&enc[..]);
        println!("{:?}", m);
    }

    #[test]
    fn list() {
        // List { length: 1, elems: [List(List { length: 2, elems: [SmallAtomUtf8(SmallAtomUtf8("a")),
        // SmallAtomUtf8(SmallAtomUtf8("b"))], tail: Nil(Nil) })], tail: SmallAtomUtf8(SmallAtomUtf8("c")) }
        let enc = [
            108, 0, 0, 0, 1, 108, 0, 0, 0, 2, 119, 1, 97, 119, 1, 98, 106, 119, 1, 99,
        ];
        let l = List::from(&enc[..]);
        println!("{:?}", l);
        let tail = SmallAtomUtf8::try_from(*l.tail).unwrap();
        assert_eq!(tail.0, "c");
        let enc = [108, 0, 0, 0, 2, 119, 1, 97, 119, 1, 98, 119, 1, 99];
        let l = List::from(&enc[..]);
        println!("{:?}", l);
        let enc = [108, 0, 0, 0, 2, 119, 1, 97, 119, 1, 98, 106];
        let l = Term::from(&enc[..]);
        println!("{:?}", l);
    }

    #[test]
    fn small_big() {
        let enc = vec![110, 6, 0, 179, 115, 4, 214, 250, 111];
        let s = SmallBig::from(&enc[..]);
        println!("{:?}", s);
        let mut expect = vec![];
        s.encode(&mut expect).unwrap();
        println!("{:?}", enc);
        assert_eq!(enc, expect);
    }

    #[test]
    fn bit_binary() {
        // <<3::5, 6::5, 7::3>> =>
        // <<25, 23::size(5)>>
        let enc = vec![77, 0, 0, 0, 2, 5, 25, 184];
        let b = BitBinary::from(&enc[..]);
        assert_eq!(b.length, 2);
        assert_eq!(b.bits, 5);
        assert_eq!(b.data, vec![25, 23]);
        let mut expect = vec![];
        b.encode(&mut expect).unwrap();
        assert_eq!(enc, expect);
    }

    #[test]
    fn new_fun() {
        let enc = vec![
            112, 0, 0, 0, 234, 2, 201, 188, 156, 143, 16, 126, 173, 32, 90, 79, 205, 206, 160, 193,
            177, 248, 0, 0, 0, 41, 0, 0, 0, 1, 119, 8, 101, 114, 108, 95, 101, 118, 97, 108, 97,
            41, 98, 6, 77, 228, 228, 88, 119, 8, 97, 64, 102, 101, 100, 111, 114, 97, 0, 0, 0, 115,
            0, 0, 0, 0, 101, 129, 113, 138, 104, 6, 97, 131, 116, 0, 0, 0, 0, 119, 4, 110, 111,
            110, 101, 104, 2, 119, 5, 118, 97, 108, 117, 101, 113, 119, 6, 101, 108, 105, 120, 105,
            114, 119, 21, 101, 118, 97, 108, 95, 101, 120, 116, 101, 114, 110, 97, 108, 95, 104,
            97, 110, 100, 108, 101, 114, 97, 3, 116, 0, 0, 0, 0, 108, 0, 0, 0, 1, 104, 5, 119, 6,
            99, 108, 97, 117, 115, 101, 97, 131, 108, 0, 0, 0, 2, 104, 3, 119, 3, 118, 97, 114, 97,
            131, 119, 4, 95, 97, 64, 49, 104, 3, 119, 3, 118, 97, 114, 97, 131, 119, 4, 95, 98, 64,
            49, 106, 106, 108, 0, 0, 0, 1, 104, 5, 119, 2, 111, 112, 97, 131, 119, 1, 43, 104, 3,
            119, 3, 118, 97, 114, 97, 131, 119, 4, 95, 97, 64, 49, 104, 3, 119, 3, 118, 97, 114,
            97, 131, 119, 4, 95, 98, 64, 49, 106, 106,
        ];

        let f = NewFun::from(&enc[..]);
        assert_eq!(f.module.0, "erl_eval");
        assert_eq!(f.free_vars.len(), 1);
        assert_eq!(f.pid.node.0, "a@fedora");
    }
}
