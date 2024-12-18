use serde::{
    de::{self, DeserializeSeed, MapAccess, SeqAccess, Unexpected, Visitor},
    forward_to_deserialize_any, Deserialize,
};

use crate::error::*;

pub struct Deserializer<'de> {
    input: &'de rmpv::Value,
}

impl<'de> Deserializer<'de> {
    pub fn from_value(input: &'de rmpv::Value) -> Self {
        Deserializer { input }
    }
}

pub fn from_value<'a, T>(s: &'a rmpv::Value) -> Result<T>
where
    T: Deserialize<'a>,
{
    let mut deserializer = Deserializer::from_value(s);
    T::deserialize(&mut deserializer)
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.input {
            rmpv::Value::Nil => self.deserialize_unit(visitor),
            rmpv::Value::Boolean(_) => self.deserialize_bool(visitor),
            rmpv::Value::Integer(_) => self.deserialize_i64(visitor),
            rmpv::Value::String(_) => self.deserialize_string(visitor),
            rmpv::Value::Array(_) => self.deserialize_seq(visitor),
            rmpv::Value::Map(_) => self.deserialize_map(visitor),
            rmpv::Value::Binary(_) => self.deserialize_bytes(visitor),
            _ => Err(Error::UnsupportedType),
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_bool(
            self.input
                .as_bool()
                .ok_or(Error::TypeError("expected bool".to_string()))?,
        )
    }

    // The `parse_signed` function is generic over the integer type `T` so here
    // it is invoked with `T=i8`. The next 8 methods are similar.
    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i8(
            self.input
                .as_i64()
                .map(|v| v as i8)
                .ok_or(Error::TypeError("expected i8".to_string()))?,
        )
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i16(
            self.input
                .as_i64()
                .map(|v| v as i16)
                .ok_or(Error::TypeError("expected i16".to_string()))?,
        )
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i32(
            self.input
                .as_i64()
                .ok_or(Error::TypeError("expected i32".to_string()))? as i32,
        )
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i64(
            self.input
                .as_i64()
                .ok_or(Error::TypeError("expected i64".to_string()))?,
        )
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u8(
            self.input
                .as_u64()
                .map(|v| v as u8)
                .ok_or(Error::TypeError("expected u8".to_string()))?,
        )
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u16(
            self.input
                .as_u64()
                .map(|v| v as u16)
                .ok_or(Error::TypeError("expected u16".to_string()))?,
        )
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u32(
            self.input
                .as_u64()
                .map(|v| v as u32)
                .ok_or(Error::TypeError("expected u32".to_string()))?,
        )
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u64(
            self.input
                .as_u64()
                .ok_or(Error::TypeError("expected u64".to_string()))?,
        )
    }

    fn deserialize_f32<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        Err(Error::UnsupportedType)
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_f64(
            self.input
                .as_f64()
                .ok_or(Error::TypeError("expected f64".to_string()))?,
        )
    }

    fn deserialize_char<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        Err(Error::UnsupportedType)
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_borrowed_str(
            self.input
                .as_str()
                .ok_or(Error::TypeError(format!("expected string: {}", self.input)))?,
        )
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_borrowed_str(
            self.input
                .as_str()
                .ok_or(Error::TypeError(format!("expected string: {}", self.input)))?,
        )
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_bytes(
            self.input
                .as_slice()
                .ok_or(Error::TypeError("expected binary".to_string()))?,
        )
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_bytes(
            self.input
                .as_slice()
                .ok_or(Error::TypeError("expected binary".to_string()))?,
        )
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.input {
            rmpv::Value::Nil => visitor.visit_none(),
            _ => visitor.visit_some(self),
        }
    }

    // In Serde, unit means an anonymous value containing no data.
    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.input {
            rmpv::Value::Nil => visitor.visit_unit(),
            _ => Err(Error::TypeError("expected nil".to_string())),
        }
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        // FIXME: We only support unit variants for now
        visitor.visit_enum(UnitVariantAccess::new(self))
    }

    // Unit struct means a named value containing no data.
    fn deserialize_unit_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    // As is done here, serializers are encouraged to treat newtype structs as
    // insignificant wrappers around the data they contain. That means not
    // parsing anything other than the contained value.
    fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    // Deserialization of compound types like sequences and maps happens by
    // passing the visitor an "Access" object that gives it the ability to
    // iterate through the data contained in the sequence.
    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.input {
            rmpv::Value::Binary(v) => visitor.visit_bytes(v),
            rmpv::Value::Ext(_, _) => serde::Deserializer::deserialize_any(
                ExtDeserializer::new(self.input.clone()),
                visitor,
            ),
            rmpv::Value::Array(_) => visitor.visit_seq(ArrayAccess::new(self)),
            _ => Err(Error::TypeError("expected sequence type".to_string())),
        }
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if let rmpv::Value::Map(_) = self.input {
            visitor.visit_map(ValueMapAccess::new(self))
        } else {
            Err(Error::TypeError("expected map".to_string()))
        }
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
}

struct ExtValueDeserializer {
    value: rmpv::Value,
}

impl ExtValueDeserializer {
    fn new(value: rmpv::Value) -> Self {
        ExtValueDeserializer { value }
    }
}

impl<'de> serde::Deserializer<'de> for ExtValueDeserializer {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        let ret = visitor.visit_bytes(self.value.as_slice().unwrap())?;
        Ok(ret)
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

struct ExtIdDeserializer {
    id: rmpv::Value,
}

impl ExtIdDeserializer {
    fn new(id: rmpv::Value) -> Self {
        ExtIdDeserializer { id }
    }
}

impl<'de> serde::Deserializer<'de> for ExtIdDeserializer {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        let ret = visitor.visit_i8(self.id.as_i64().unwrap() as i8)?;
        Ok(ret)
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

struct ExtDeserializer {
    id: rmpv::Value,
    data: rmpv::Value,
    offset: usize,
}

impl ExtDeserializer {
    fn new(value: rmpv::Value) -> Self {
        let (id, data) = value.as_ext().expect("expected ext");
        ExtDeserializer {
            id: rmpv::Value::from(id),
            data: rmpv::Value::from(data),
            offset: 0,
        }
    }
}

impl<'de> serde::Deserializer<'de> for ExtDeserializer {
    type Error = Error;

    fn deserialize_any<V>(mut self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        let ret = visitor.visit_seq(&mut self)?;
        Ok(ret)
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

impl<'de> SeqAccess<'de> for ExtDeserializer {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Error>
    where
        T: DeserializeSeed<'de>,
    {
        match self.offset {
            0 => {
                self.offset += 1;
                let de = ExtIdDeserializer::new(self.id.clone());
                let v = seed.deserialize(de)?;
                Ok(Some(v))
            }
            1 => {
                self.offset += 1;
                let de = ExtValueDeserializer::new(self.data.clone());
                let v = seed.deserialize(de)?;
                Ok(Some(v))
            }
            _ => Ok(None),
        }
    }
}

// struct ExtAccess<'a, 'de: 'a> {
//     de: &'a mut Deserializer<'de>,
//     offset: usize,
// }
//
// impl<'a, 'de> ExtAccess<'a, 'de> {
//     fn new(de: &'a mut Deserializer<'de>) -> Self {
//         ExtAccess { de, offset: 0 }
//     }
// }
//
// impl<'de, 'a> SeqAccess<'de> for ExtAccess<'a, 'de> {
//     type Error = Error;
//
//     fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
//     where
//         T: DeserializeSeed<'de>,
//     {
//         let (id, val) = self.de.input.as_ext().expect("expected ext");
//         match self.offset {
//             0 => {
//                 self.offset += 1;
//                 let value = rmpv::Value::from(id);
//                 let mut de = Deserializer::from_value(&value);
//                 seed.deserialize(&mut de).map(Some)
//             }
//             1 => {
//                 self.offset += 1;
//                 let value = rmpv::Value::Binary(val.to_vec());
//                 let mut de = Deserializer::from_value(&value);
//                 seed.deserialize(&mut de).map(Some)
//             }
//             _ => Ok(None),
//         }
//     }
// }

struct ArrayAccess<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
    offset: usize,
}

impl<'a, 'de> ArrayAccess<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>) -> Self {
        ArrayAccess { de, offset: 0 }
    }
}

impl<'de, 'a> SeqAccess<'de> for ArrayAccess<'a, 'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: DeserializeSeed<'de>,
    {
        let arr = self
            .de
            .input
            .as_array()
            .ok_or(Error::TypeError("expected array".to_string()))?;
        if self.offset < arr.len() {
            let mut d = Deserializer::from_value(&arr[self.offset]);
            self.offset += 1;
            Ok(Some(
                seed.deserialize(&mut d)
                    .map_err(|e| Error::Message(e.to_string()))?,
            ))
        } else {
            Ok(None)
        }
    }
}

struct ValueMapAccess<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
    offset: usize,
}

impl<'a, 'de> ValueMapAccess<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>) -> Self {
        ValueMapAccess { de, offset: 0 }
    }
}

impl<'de, 'a> MapAccess<'de> for ValueMapAccess<'a, 'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: DeserializeSeed<'de>,
    {
        let m = self
            .de
            .input
            .as_map()
            .ok_or(Error::TypeError("expected map".to_string()))?;
        if self.offset < m.len() {
            let mut d = Deserializer::from_value(&m[self.offset].0);
            self.offset += 1;
            Ok(Some(
                seed.deserialize(&mut d)
                    .map_err(|e| Error::Message(e.to_string()))?,
            ))
        } else {
            Ok(None)
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: DeserializeSeed<'de>,
    {
        let m = self
            .de
            .input
            .as_map()
            .ok_or(Error::TypeError("expected map".to_string()))?;
        let mut d = Deserializer::from_value(&m[self.offset - 1].1);
        seed.deserialize(&mut d)
            .map_err(|e| Error::Message(e.to_string()))
    }
}

struct UnitVariantAccess<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
}

impl<'a, 'de> UnitVariantAccess<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>) -> Self {
        UnitVariantAccess { de }
    }
}

impl<'de, 'a> de::EnumAccess<'de> for UnitVariantAccess<'a, 'de> {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self), Error>
    where
        V: DeserializeSeed<'de>,
    {
        let variant = seed.deserialize(&mut *self.de)?;
        Ok((variant, self))
    }
}

impl<'de, 'a> de::VariantAccess<'de> for UnitVariantAccess<'a, 'de> {
    type Error = Error;

    fn unit_variant(self) -> Result<(), Error> {
        Ok(())
    }

    fn newtype_variant_seed<T>(self, _seed: T) -> Result<T::Value, Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        Err(de::Error::invalid_type(
            Unexpected::UnitVariant,
            &"newtype variant",
        ))
    }

    fn tuple_variant<V>(self, _len: usize, _visitor: V) -> Result<V::Value, Error>
    where
        V: de::Visitor<'de>,
    {
        Err(de::Error::invalid_type(
            Unexpected::UnitVariant,
            &"tuple variant",
        ))
    }

    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: de::Visitor<'de>,
    {
        Err(de::Error::invalid_type(
            Unexpected::UnitVariant,
            &"struct variant",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_derive::Deserialize;

    use serde_with::{serde_as, Bytes};

    #[test]
    fn test_exttype() {
        #[serde_as]
        #[derive(Deserialize, Debug, PartialEq)]
        #[serde(rename = "_ExtStruct")]
        struct Foo(#[serde_as(as = "(_, Bytes)")] (u8, Vec<u8>));

        let ext = rmpv::Value::Ext(42, vec![1, 2, 3]);
        let foo: Foo = from_value(&ext).unwrap();
        assert_eq!(Foo((42, vec![1, 2, 3])), foo);
    }

    #[test]
    fn test_deserialize() {
        use super::*;
        use serde_derive::Deserialize;

        super::from_value::<i8>(&rmpv::Value::from("foo")).expect_err("expected unimplemented");

        assert_eq!(
            "string",
            from_value::<String>(&rmpv::Value::from("string")).unwrap()
        );

        assert_eq!(42, from_value::<i64>(&rmpv::Value::from(42)).unwrap());

        #[derive(Debug, PartialEq, Deserialize)]
        struct TestStruct {
            a: i32,
            b: String,
        }

        assert_eq!(
            TestStruct {
                a: 42,
                b: "string".to_string()
            },
            from_value::<TestStruct>(&rmpv::Value::Map(vec![
                (rmpv::Value::from("a"), rmpv::Value::from(42)),
                (rmpv::Value::from("b"), rmpv::Value::from("string")),
            ]))
            .unwrap()
        );
    }
}
