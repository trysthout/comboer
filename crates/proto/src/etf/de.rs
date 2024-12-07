use std::convert::TryFrom;
use std::fmt::Debug;
use std::slice::{Chunks, Iter};

use num_traits::cast::*;
use serde::de::{self, DeserializeSeed, EnumAccess, VariantAccess, Visitor};
use serde::forward_to_deserialize_any;

use super::{error::Error, term::*};

pub struct Deserializer<'de> {
    input: &'de Term,
}

impl<'de> Deserializer<'de> {
    pub fn from_term(input: &'de Term) -> Self {
        Deserializer { input }
    }
}

pub fn from_term<'a, T>(input: &'a Term) -> Result<T, anyhow::Error>
where
    T: de::Deserialize<'a>,
{
    let mut deserializer = Deserializer::from_term(input);
    let t = T::deserialize(&mut deserializer)?;
    Ok(t)
}

impl<'de> Deserializer<'de> {
    // Parse the atom identifier `true` or `false`.
    fn parse_bool(&self) -> Result<bool, Error> {
        match self.input {
            Term::SmallAtomUtf8(v) => match v.0.as_str() {
                "true" => Ok(true),
                "false" => Ok(false),
                _ => Err(Error::InvalidBoolean),
            },
            _ => Err(Error::ExpectedBoolean),
        }
    }

    fn parse_number<T: FromPrimitive + Debug>(&self) -> Result<T, Error> {
        match self.input {
            Term::SmallInteger(v) => T::from_u8(v.0).ok_or(Error::InvalidPrimitiveNumber),
            Term::Integer(v) => T::from_i32(v.0).ok_or(Error::InvalidPrimitiveNumber),

            Term::NewFloat(v) => T::from_f64(v.0).ok_or(Error::InvalidPrimitiveNumber),
            _ => Err(Error::ExpectedPrimitiveNumber),
        }
    }

    fn parse_bigint<T: TryFrom<&'de num_bigint::BigInt>>(&self) -> Result<T, Error> {
        match self.input {
            Term::SmallBig(v) => T::try_from(&v.n).map_err(|_| {
                Error::TryFromBigInt(format!(
                    "Cannot to attempt conversion from bigint type: {}",
                    &v.n
                ))
            }),
            _ => Err(Error::ExpectedBigInt),
        }
    }

    fn parse_string(&self) -> Result<String, Error> {
        match self.input {
            Term::SmallAtomUtf8(v) => Ok(v.0.clone()),
            Term::AtomUtf8(v) => Ok(v.0.clone()),
            Term::Binary(v) => Ok(String::from_utf8_lossy(&v.data).to_string()),
            Term::StringExt(v) => Ok(String::from_utf8_lossy(&v.chars).to_string()),
            _ => Err(Error::ExpectedString),
        }
    }

    fn parse_binary(&self) -> Result<&[u8], Error> {
        match self.input {
            Term::Binary(v) => Ok(&v.data),
            _ => Err(Error::ExpectedBytes),
        }
    }

    fn is_none(&self) -> Result<bool, Error> {
        match self.input {
            Term::Nil(_) => Ok(true),
            _ => Ok(false),
        }
    }
}

impl<'de> de::Deserializer<'de> for &mut Deserializer<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(Error::ExpectedType)
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_bool(self.parse_bool()?)
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i8(self.parse_number()?)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i16(self.parse_number()?)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i32(self.parse_number()?)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i64(self.parse_number()?)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u8(self.parse_number()?)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u16(self.parse_number()?)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u32(self.parse_bigint()?)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u64(self.parse_bigint()?)
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_f32(self.parse_number()?)
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_f64(self.parse_number()?)
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let c = self
            .parse_string()?
            .chars()
            .next()
            .ok_or(Error::ExpectedChar)?;
        visitor.visit_char(c)
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_str(&self.parse_string()?)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_string(self.parse_string()?)
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_bytes(self.parse_binary()?)
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_byte_buf(self.parse_binary()?.to_vec())
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if self.is_none()? {
            visitor.visit_none()
        } else {
            visitor.visit_some(self)
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if self.is_none()? {
            visitor.visit_unit()
        } else {
            Err(Error::ExpectedNil)
        }
    }

    // Unit struct means a named value containing no data.
    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    // As is done here, serializers are encouraged to treat newtype structs as
    // insignificant wrappers around the data they contain. That means not
    // parsing anything other than the contained value.
    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    // Deserialization of compound types like sequences and maps happens by
    // passing the visitor an "Access" object that gives it the ability to
    // iterate through the data contained in the sequence.
    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.input {
            Term::List(v) => visitor.visit_seq(DeserializerSeq::new(&v.elems)),
            Term::Binary(v) => visitor.visit_byte_buf(v.data.clone()),

            _ => Err(Error::ExpectedList),
        }
    }

    // Tuples look just like sequences in JSON. Some formats may be able to
    // represent tuples more efficiently.
    //
    // As indicated by the length parameter, the `Deserialize` implementation
    // for a tuple in the Serde data model is required to know the length of the
    // tuple before even looking at the input data.
    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.input {
            Term::SmallTuple(v) => visitor.visit_seq(DeserializerSeq::new(&v.elems)),
            Term::LargeTuple(v) => visitor.visit_seq(DeserializerSeq::new(&v.elems)),
            _ => Err(Error::ExpectedTuple),
        }
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_tuple(len, visitor)
    }

    // Much like `deserialize_seq` but calls the visitors `visit_map` method
    // with a `MapAccess` implementation, rather than the visitor's `visit_seq`
    // method with a `SeqAccess` implementation.
    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.input {
            Term::Map(v) => visitor.visit_map(DeserializerMap::new(&v.pairs)),
            _ => Err(Error::ExpectedMap),
        }
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.input {
            v @ Term::SmallAtomUtf8(_) | v @ Term::AtomUtf8(_) => {
                visitor.visit_enum(DeserializerEnumUnit::new(v))
            }
            Term::SmallTuple(SmallTuple { arity: _, elems })
            | Term::LargeTuple(LargeTuple { arity: _, elems }) => match elems.as_slice() {
                [variant, value] => {
                    visitor.visit_enum(DeserializerEnum::new(self, (variant, value)))
                }
                _ => Err(Error::InvalidEnumTuple),
            },
            _ => Err(Error::ExpectedAtomOrTuple),
        }
    }

    // An identifier in Serde is the type that identifies a field of a struct or
    // the variant of an enum. In JSON, struct fields and enum variants are
    // represented as strings. In other formats they may be represented as
    // numeric indices.
    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    // Like `deserialize_any` but indicates to the `Deserializer` that it makes
    // no difference which `Visitor` method is called because the data is
    // ignored.
    //
    // Some deserializers are able to implement this more efficiently than
    // `deserialize_any`, for example by rapidly skipping over matched
    // delimiters without paying close attention to the data in between.
    //
    // Some formats are not able to implement this at all. Formats that can
    // implement `deserialize_any` and `deserialize_ignored_any` are known as
    // self-describing.
    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
}

struct DeserializerSeq<'de> {
    input: Iter<'de, Term>,
}

impl<'de> DeserializerSeq<'de> {
    pub fn new(input: &'de [Term]) -> Self {
        DeserializerSeq {
            input: input.iter(),
        }
    }
}

impl<'de> de::SeqAccess<'de> for DeserializerSeq<'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        match self.input.next() {
            Some(item) => seed
                .deserialize(&mut Deserializer::from_term(item))
                .map(Some),
            None => Ok(None),
        }
    }
}

struct DeserializerMap<'de> {
    input: Chunks<'de, Term>,
    current_value: Option<&'de Term>,
}

impl<'de> DeserializerMap<'de> {
    pub fn new(input: &'de [Term]) -> Self {
        DeserializerMap {
            input: input.chunks(2),
            current_value: None,
        }
    }
}

impl<'de> de::MapAccess<'de> for DeserializerMap<'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>,
    {
        match self.input.next() {
            Some(pair) => {
                if pair.len() < 2 {
                    return Err(Error::InvalidMapPair);
                }

                let k = &pair[0];
                self.current_value = Some(&pair[1]);

                seed.deserialize(&mut Deserializer::from_term(k)).map(Some)
            }

            None => Ok(None),
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        match self.current_value {
            Some(v) => {
                self.current_value = None;
                seed.deserialize(&mut Deserializer::from_term(v))
            }
            None => Err(Error::InvalidMapPair),
        }
    }
}

pub struct DeserializerEnum<'a, 'de: 'a> {
    _de: &'a mut Deserializer<'de>,
    input: (&'de Term, &'de Term),
}

impl<'a, 'de> DeserializerEnum<'a, 'de> {
    fn new(
        de: &'a mut Deserializer<'de>,
        input: (&'de Term, &'de Term),
    ) -> DeserializerEnum<'a, 'de> {
        DeserializerEnum { _de: de, input }
    }
}

impl<'de> EnumAccess<'de> for DeserializerEnum<'_, 'de> {
    type Error = Error;

    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        let v = seed.deserialize(DeserializerEnumVariant::new(self.input.0))?;
        Ok((v, self))
    }
}

impl<'de> VariantAccess<'de> for DeserializerEnum<'_, 'de> {
    type Error = Error;

    fn unit_variant(self) -> Result<(), Self::Error> {
        Err(Error::ExpectedAtom)
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        seed.deserialize(&mut Deserializer::from_term(self.input.1))
    }

    fn tuple_variant<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let mut deserializer = Deserializer::from_term(self.input.1);
        de::Deserializer::deserialize_tuple(&mut deserializer, len, visitor)
    }

    // Struct variants are represented in JSON as `{ NAME: { K: V, ... } }` so
    // deserialize the inner map here.
    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let mut deserializer = Deserializer::from_term(self.input.1);
        de::Deserializer::deserialize_map(&mut deserializer, visitor)
    }
}

struct DeserializerEnumUnit<'de> {
    input: &'de Term,
}

impl<'de> DeserializerEnumUnit<'de> {
    fn new(input: &'de Term) -> DeserializerEnumUnit<'de> {
        DeserializerEnumUnit { input }
    }
}

impl<'de> EnumAccess<'de> for DeserializerEnumUnit<'de> {
    type Error = Error;

    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        let v = seed.deserialize(&mut Deserializer::from_term(self.input))?;
        Ok((v, self))
    }
}

impl<'de> VariantAccess<'de> for DeserializerEnumUnit<'_> {
    type Error = Error;

    fn unit_variant(self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn newtype_variant_seed<T>(self, _seed: T) -> Result<T::Value, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        unimplemented!()
    }

    fn tuple_variant<V>(self, _len: usize, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    // Struct variants are represented in JSON as `{ NAME: { K: V, ... } }` so
    // deserialize the inner map here.
    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }
}

struct DeserializerEnumVariant<'a> {
    input: &'a Term,
}

impl<'a> DeserializerEnumVariant<'a> {
    pub fn new(input: &'a Term) -> Self {
        DeserializerEnumVariant { input }
    }
}

impl<'de, 'a: 'de> de::Deserializer<'de> for DeserializerEnumVariant<'a> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.input {
            Term::SmallAtomUtf8(v) => visitor.visit_string(v.0.clone()),
            Term::AtomUtf8(v) => visitor.visit_string(v.0.clone()),
            _ => Err(Error::ExpectedAtom),
        }
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
            bytes byte_buf option unit unit_struct newtype_struct seq tuple
            tuple_struct map struct enum identifier ignored_any
    }
}

#[cfg(test)]
mod test {
    use serde_derive::Deserialize;

    use crate::etf::{de::from_term, Sign, term::*};

    #[test]
    fn der_struct() {
        #[derive(Debug, Deserialize)]
        struct T1 {
            v1: u16,
            v2: String,
        }

        let t = Term::Map(Map {
            arity: 2,
            pairs: vec![
                SmallAtomUtf8("v1".to_string()).into(),
                SmallInteger(8).into(),
                SmallAtomUtf8("v2".to_string()).into(),
                Binary {
                    length: 5,
                    data: "hello".as_bytes().to_vec(),
                }
                .into(),
            ],
        });

        let der = from_term::<T1>(&t).unwrap();

        assert_eq!(der.v1, 8);
        assert_eq!(der.v2, "hello");
        println!("{:?}", der);

        #[derive(Debug, Deserialize)]
        struct T2(u16, u64, #[serde(with = "serde_bytes")] Vec<u8>);
        let t = Term::SmallTuple(SmallTuple {
            arity: 3,
            elems: vec![
                SmallInteger(8).into(),
                SmallBig {
                    length: 8,
                    sign: Sign::Positive,
                    n: 11111111111111111_u64.into(),
                }
                .into(),
                Binary {
                    length: 5,
                    data: "hello".as_bytes().to_vec(),
                }
                .into(),
            ],
        });
        let der = from_term::<T2>(&t).unwrap();
        println!("{:?}", der);
        assert_eq!(der.0, 8);
        assert_eq!(der.1, 11111111111111111_u64);
        assert_eq!(der.2, [104, 101, 108, 108, 111]);

        #[derive(Debug, Deserialize, PartialEq)]
        struct T3;
        let t = Term::Nil(Nil);
        let der = from_term::<T3>(&t).unwrap();
        println!("{:?}", der);
        assert_eq!(T3, der);

        #[derive(Debug, Deserialize)]
        struct Len(u8);
        let t = Term::SmallInteger(SmallInteger(3));
        let der = from_term::<Len>(&t).unwrap();
        println!("{:?}", der);
        assert_eq!(3, der.0)
    }

    #[test]
    fn der_enum() {
        #[derive(Debug, Deserialize, PartialEq)]
        enum T1 {
            A,
            B,
        }

        let der = from_term::<T1>(&Term::SmallAtomUtf8(SmallAtomUtf8("A".to_string()))).unwrap();
        println!("{:?}", der);
        assert_eq!(der, T1::A);

        #[derive(Debug, Deserialize, PartialEq)]
        enum T2 {
            A,
            B { a: String, b: Vec<u8> },
        }

        let der = from_term::<T2>(&Term::SmallTuple(SmallTuple {
            arity: 2,
            elems: vec![
                SmallAtomUtf8("B".to_string()).into(),
                Map {
                    arity: 1,
                    pairs: vec![
                        SmallAtomUtf8("a".to_string()).into(),
                        SmallAtomUtf8("123".to_string()).into(),
                        SmallAtomUtf8("b".to_string()).into(),
                        List {
                            length: 3,
                            elems: vec![
                                SmallInteger(1).into(),
                                SmallInteger(2).into(),
                                SmallInteger(3).into(),
                            ],
                            tail: Box::new(Term::Nil(Nil)),
                        }
                        .into(),
                    ],
                }
                .into(),
            ],
        }))
        .unwrap();
        println!("{:?}", der);
        assert_eq!(
            T2::B {
                a: "123".to_string(),
                b: vec![1, 2, 3]
            },
            der
        );

        #[derive(Debug, Deserialize, PartialEq)]
        enum T3 {
            A(String, u32),
        }

        let der = from_term::<T3>(&Term::SmallTuple(SmallTuple {
            arity: 2,
            elems: vec![
                SmallAtomUtf8("A".to_string()).into(),
                SmallTuple {
                    arity: 2,
                    elems: vec![
                        SmallAtomUtf8("A".to_string()).into(),
                        SmallBig {
                            length: 8,
                            sign: Sign::Positive,
                            n: 111111111_u32.into(),
                        }
                        .into(),
                    ],
                }
                .into(),
            ],
        }))
        .unwrap();
        println!("{:?}", der);
        assert_eq!(T3::A("A".to_string(), 111111111_u32), der);
    }
}
