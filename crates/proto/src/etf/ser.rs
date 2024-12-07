use num_bigint::BigInt;
use serde::{ser, Serialize};

use super::{error::Error, Sign, term::*};

pub struct Serializer;

pub fn to_term<T>(value: &T) -> Result<Term, anyhow::Error>
where
    T: Serialize,
{
    let mut serializer = Serializer {};
    let result = value.serialize(&mut serializer)?;
    Ok(result)
}

impl<'a> ser::Serializer for &'a mut Serializer {
    type Ok = Term;

    type Error = Error;

    type SerializeSeq = SerializeSeq<'a>;

    type SerializeTuple = SerializeSeq<'a>;

    type SerializeTupleStruct = SerializeSeq<'a>;

    type SerializeTupleVariant = SerializeSeq<'a>;

    type SerializeMap = SerializeSeq<'a>;

    type SerializeStruct = SerializeSeq<'a>;

    type SerializeStructVariant = SerializeSeq<'a>;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        let v = if v { "true" } else { "false" };

        Ok(Term::from(SmallAtomUtf8(v.to_string())))
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        Ok(Term::from(Integer(v as i32)))
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        Ok(Term::from(Integer(v as i32)))
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        Ok(Term::from(Integer(v)))
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        let sign = if v > 0 {
            Sign::Positive
        } else {
            Sign::Negative
        };

        Ok(Term::from(SmallBig {
            length: 8,
            sign: sign.clone(),
            n: BigInt::from_bytes_le((&sign).into(), &v.to_le_bytes()),
        }))
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        Ok(Term::from(SmallInteger(v)))
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        Ok(Term::from(Integer(v as i32)))
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        let sign = Sign::Positive;
        Ok(Term::from(SmallBig {
            length: 4,
            sign: sign.clone(),
            n: BigInt::from_bytes_le((&sign).into(), &v.to_le_bytes()),
        }))
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        let sign = Sign::Positive;
        Ok(Term::from(SmallBig {
            length: 8,
            sign: sign.clone(),
            n: BigInt::from_bytes_le((&sign).into(), &v.to_le_bytes()),
        }))
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        self.serialize_f64(v as f64)
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        Ok(Term::from(NewFloat(v)))
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        Ok(Term::from(SmallAtomUtf8(v.to_string())))
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        Ok(Term::from(SmallAtomUtf8(v.to_string())))
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        Ok(Term::from(Binary {
            length: v.len() as u32,
            data: v.to_vec(),
        }))
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Ok(Term::from(Nil))
    }

    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize + ?Sized,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        self.serialize_none()
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        self.serialize_none()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        self.serialize_str(variant)
    }

    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize + ?Sized,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize + ?Sized,
    {
        let value = value.serialize(self)?;
        Ok(Term::SmallTuple(SmallTuple {
            arity: 2,
            elems: vec![Term::from(SmallAtomUtf8(variant.to_string())), value],
        }))
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        let elems = len.map(Vec::with_capacity).unwrap_or_default();
        Ok(SerializeSeq {
            ser: self,
            elems,
            name: None,
        })
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Ok(SerializeSeq {
            ser: self,
            elems: Vec::with_capacity(len),
            name: None,
        })
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Ok(SerializeSeq {
            ser: self,
            elems: Vec::with_capacity(len),
            name: None,
        })
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Ok(SerializeSeq {
            ser: self,
            elems: Vec::with_capacity(len),
            name: Some(variant),
        })
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        let elems = len.map(Vec::with_capacity).unwrap_or_default();
        Ok(SerializeSeq {
            ser: self,
            elems,
            name: None,
        })
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(SerializeSeq {
            ser: self,
            elems: Vec::with_capacity(len),
            name: None,
        })
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Ok(SerializeSeq {
            ser: self,
            elems: Vec::with_capacity(len),
            name: Some(variant),
        })
    }
}

pub struct SerializeSeq<'a> {
    ser: &'a mut Serializer,
    // Only used by tuple variant
    name: Option<&'a str>,
    elems: Vec<Term>,
}

impl ser::SerializeSeq for SerializeSeq<'_> {
    // Must match the `Ok` type of the serializer.
    type Ok = Term;
    // Must match the `Error` type of the serializer.
    type Error = Error;

    // Serialize a single element of the sequence.
    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        let v = value.serialize(&mut *self.ser)?;
        self.elems.push(v);

        Ok(())
    }

    // Close the sequence.
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(Term::List(List {
            length: self.elems.len() as u32,
            elems: self.elems,
            tail: Box::new(Term::Nil(Nil)),
        }))
    }
}

impl ser::SerializeTuple for SerializeSeq<'_> {
    type Ok = Term;
    type Error = Error;

    // Serialize a single element of the sequence.
    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        let v = value.serialize(&mut *self.ser)?;
        self.elems.push(v);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        let term = if self.elems.len() > u8::MAX as usize {
            Term::LargeTuple(LargeTuple {
                arity: self.elems.len() as u32,
                elems: self.elems,
            })
        } else {
            Term::SmallTuple(SmallTuple {
                arity: self.elems.len() as u8,
                elems: self.elems,
            })
        };

        Ok(term)
    }
}

impl ser::SerializeTupleStruct for SerializeSeq<'_> {
    type Ok = Term;
    type Error = Error;

    // Serialize a single element of the sequence.
    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        let v = value.serialize(&mut *self.ser)?;
        self.elems.push(v);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        let term = if self.elems.len() > u8::MAX as usize {
            Term::LargeTuple(LargeTuple {
                arity: self.elems.len() as u32,
                elems: self.elems,
            })
        } else {
            Term::SmallTuple(SmallTuple {
                arity: self.elems.len() as u8,
                elems: self.elems,
            })
        };

        Ok(term)
    }
}

impl ser::SerializeTupleVariant for SerializeSeq<'_> {
    type Ok = Term;
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        let v = value.serialize(&mut *self.ser)?;
        self.elems.push(v);
        Ok(())
    }

    // TupleVariant serizalized a tuple { name, {elems}}
    fn end(self) -> Result<Self::Ok, Self::Error> {
        let data = if self.elems.len() > u8::MAX as usize {
            Term::LargeTuple(LargeTuple {
                arity: self.elems.len() as u32,
                elems: self.elems,
            })
        } else {
            Term::SmallTuple(SmallTuple {
                arity: self.elems.len() as u8,
                elems: self.elems,
            })
        };

        Ok(Term::SmallTuple(SmallTuple {
            arity: 2,
            elems: vec![SmallAtomUtf8(self.name.unwrap().to_string()).into(), data],
        }))
    }
}

impl ser::SerializeMap for SerializeSeq<'_> {
    type Ok = Term;
    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        let k = key.serialize(&mut *self.ser)?;
        self.elems.push(k);
        Ok(())
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        let v = value.serialize(&mut *self.ser)?;
        self.elems.push(v);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(Term::Map(Map {
            arity: (self.elems.len() / 2) as u32,
            pairs: self.elems,
        }))
    }
}

impl ser::SerializeStruct for SerializeSeq<'_> {
    type Ok = Term;
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        let v = value.serialize(&mut *self.ser)?;
        self.elems.push(SmallAtomUtf8(key.to_string()).into());
        self.elems.push(v);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(Term::Map(Map {
            arity: (self.elems.len() / 2) as u32,
            pairs: self.elems,
        }))
    }
}

impl ser::SerializeStructVariant for SerializeSeq<'_> {
    type Ok = Term;
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        let v = value.serialize(&mut *self.ser)?;
        self.elems.push(SmallAtomUtf8(key.to_string()).into());
        self.elems.push(v);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(Term::SmallTuple(SmallTuple {
            arity: 2,
            elems: vec![
                SmallAtomUtf8(self.name.unwrap().to_string()).into(),
                Term::Map(Map {
                    arity: (self.elems.len() / 2) as u32,
                    pairs: self.elems,
                }),
            ],
        }))
    }
}

#[cfg(test)]
mod test {
    use serde_derive::Serialize;

    use super::*;

    #[test]
    fn ser_struct() {
        #[derive(Debug, Serialize)]
        struct T1 {
            k: u32,
            v: String,
        }

        let v1 = T1 {
            k: 2,
            v: "ser".to_string(),
        };

        let v = to_term(&v1);
        println!("{:?}", v);

        #[derive(Debug, Serialize)]
        struct T2(u8, T1);
        let v2 = T2(
            3,
            T1 {
                k: 3,
                v: "ser".to_string(),
            },
        );
        let v = to_term(&v2);
        println!("{:?}", v);

        // struct variants
        #[derive(Debug, Serialize)]
        enum T3 {
            Baz { a: u16, b: String },
        }
        let v3 = T3::Baz {
            a: 10,
            b: "variants".to_string(),
        };
        let v = to_term(&v3);
        println!("{:?}", v);
    }
}
