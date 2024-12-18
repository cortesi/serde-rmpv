use serde::{ser, Serialize};

use crate::{
    error::{Error, Result},
    MSGPACK_EXT_STRUCT_NAME,
};

pub fn to_value<T>(value: &T) -> Result<rmpv::Value>
where
    T: Serialize,
{
    let mut serializer = Serializer {
        output: rmpv::Value::Nil,
    };
    value.serialize(&mut serializer)?;
    Ok(serializer.output)
}

pub struct Serializer {
    output: rmpv::Value,
}

impl Serializer {
    // Serialize a single element of the sequence.
    fn serialize_seq_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        match &mut self.output {
            rmpv::Value::Array(ref mut vec) => {
                let mut serializer = Serializer {
                    output: rmpv::Value::Nil,
                };
                value.serialize(&mut serializer)?;
                vec.push(serializer.output);
                Ok(())
            }
            _ => Err(Error::Message("expected array".to_string())),
        }
    }
}

impl ser::Serializer for &mut Serializer {
    type Ok = ();

    // The error type when some error occurs during serialization.
    type Error = Error;

    // Associated types for keeping track of additional state while serializing
    // compound data structures like sequences and maps. In this case no
    // additional state is required beyond what is already stored in the
    // Serializer struct.
    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, v: bool) -> Result<()> {
        self.output = rmpv::Value::Boolean(v);
        Ok(())
    }

    fn serialize_i8(self, v: i8) -> Result<()> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i16(self, v: i16) -> Result<()> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i32(self, v: i32) -> Result<()> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i64(self, v: i64) -> Result<()> {
        self.output = rmpv::Value::Integer(v.into());
        Ok(())
    }

    fn serialize_u8(self, v: u8) -> Result<()> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u16(self, v: u16) -> Result<()> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u32(self, v: u32) -> Result<()> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u64(self, v: u64) -> Result<()> {
        self.output = rmpv::Value::Integer(v.into());
        Ok(())
    }

    fn serialize_f32(self, v: f32) -> Result<()> {
        self.output = rmpv::Value::F32(v);
        Ok(())
    }

    fn serialize_f64(self, v: f64) -> Result<()> {
        self.output = rmpv::Value::F64(v);
        Ok(())
    }

    // Serialize a char as a single-character string.
    fn serialize_char(self, v: char) -> Result<()> {
        self.serialize_str(&v.to_string())
    }

    fn serialize_str(self, v: &str) -> Result<()> {
        self.output = rmpv::Value::String(v.to_string().into());
        Ok(())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<()> {
        self.output = rmpv::Value::Binary(v.into());
        Ok(())
    }

    // A present optional is represented as just the contained value.
    fn serialize_some<T>(self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    // In Serde, unit means an anonymous value containing no data. Map this to
    // msgpack as `null`.
    fn serialize_unit(self) -> Result<()> {
        self.output = rmpv::Value::Nil;
        Ok(())
    }

    // Unit struct means a named value containing no data. Again, since there is
    // no data, map this to msgpack as `nil`.
    fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
        self.serialize_unit()
    }

    // When serializing a unit variant (or any other kind of variant), formats
    // can choose whether to keep track of it by index or by name. Binary
    // formats typically use the index of the variant and human-readable formats
    // typically use the name.
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<()> {
        self.serialize_str(variant)
    }

    // As is done here, serializers are encouraged to treat newtype structs as
    // insignificant wrappers around the data they contain.
    fn serialize_newtype_struct<T>(self, name: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        if name == MSGPACK_EXT_STRUCT_NAME {
            let nv = to_value(&value)?;
            if let rmpv::Value::Array(vec) = nv {
                if vec.len() == 2 {
                    let id: i8 = vec[0].as_u64().unwrap().try_into().unwrap();
                    if let rmpv::Value::Binary(data) = &vec[1] {
                        self.output = rmpv::Value::Ext(id, data.clone());
                        return Ok(());
                    }
                }
            }
            Err(Error::Message("invalid ext struct".to_string()))
        } else {
            value.serialize(self)
        }
    }

    fn serialize_none(self) -> Result<()> {
        self.serialize_unit()
    }

    // NewType variants are represented as Array<Vec[ENUM_NAME, VARIANT_NAME, DATA]>
    fn serialize_newtype_variant<T>(
        self,
        name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        let mut serializer = Serializer {
            output: rmpv::Value::Nil,
        };
        value.serialize(&mut serializer)?;

        self.output = rmpv::Value::Array(vec![
            rmpv::Value::String(name.into()),
            rmpv::Value::String(variant.into()),
            serializer.output,
        ]);
        Ok(())
    }

    // Now we get to the serialization of compound types.
    //
    // The start of the sequence, each value, and the end are three separate
    // method calls.
    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        self.output = rmpv::Value::Array(Vec::new());
        Ok(self)
    }

    // Tuples look just like sequences.
    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        self.serialize_seq(Some(len))
    }

    // Tuple structs look just like sequences.
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        self.serialize_seq(Some(len))
    }

    // Tuple variants are represented as Array<Vec[ENUM_NAME, VARIANT_NAME, ... DATA ...]>.
    fn serialize_tuple_variant(
        self,
        name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        self.output = rmpv::Value::Array(vec![
            rmpv::Value::String(name.into()),
            rmpv::Value::String(variant.into()),
        ]);
        Ok(self)
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        self.output = rmpv::Value::Map(Vec::new());
        Ok(self)
    }

    // Structs look just like maps.
    fn serialize_struct(self, _name: &'static str, len: usize) -> Result<Self::SerializeStruct> {
        self.serialize_map(Some(len))
    }

    // Struct variants are represented as `[ ENUM_NAME, VARIANT_NAME: { K: V, ... } ]`.
    // This is the externally tagged representation.
    fn serialize_struct_variant(
        self,
        name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        self.output = rmpv::Value::Array(vec![
            rmpv::Value::String(name.into()),
            rmpv::Value::String(variant.into()),
            rmpv::Value::Map(Vec::new()),
        ]);
        Ok(self)
    }
}

impl ser::SerializeSeq for &mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.serialize_seq_element(value)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl ser::SerializeTuple for &mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.serialize_seq_element(value)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl ser::SerializeTupleStruct for &mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.serialize_seq_element(value)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl ser::SerializeTupleVariant for &mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.serialize_seq_element(value)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl ser::SerializeMap for &mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        match &mut self.output {
            rmpv::Value::Map(ref mut vec) => {
                let mut serializer = Serializer {
                    output: rmpv::Value::Nil,
                };
                key.serialize(&mut serializer)?;
                vec.push((serializer.output, rmpv::Value::Nil));
                Ok(())
            }
            _ => Err(Error::Message("expected map".to_string())),
        }
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        match &mut self.output {
            rmpv::Value::Map(ref mut vec) => {
                let mut serializer = Serializer {
                    output: rmpv::Value::Nil,
                };
                value.serialize(&mut serializer)?;
                let last = vec.len() - 1;
                vec[last].1 = serializer.output;
                Ok(())
            }
            _ => Err(Error::Message("expected map".to_string())),
        }
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl ser::SerializeStruct for &mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        match &mut self.output {
            rmpv::Value::Map(ref mut vec) => {
                let mut keyser = Serializer {
                    output: rmpv::Value::Nil,
                };
                key.serialize(&mut keyser)?;

                let mut valser = Serializer {
                    output: rmpv::Value::Nil,
                };
                value.serialize(&mut valser)?;

                vec.push((keyser.output, valser.output));
                Ok(())
            }
            _ => Err(Error::Message("expected map".to_string())),
        }
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl ser::SerializeStructVariant for &mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        match &mut self.output {
            rmpv::Value::Array(ref mut vec) => {
                let mut serializer = Serializer {
                    output: rmpv::Value::Nil,
                };
                value.serialize(&mut serializer)?;

                let last_off = vec.len() - 1;
                let last = &mut vec[last_off];
                match last {
                    rmpv::Value::Map(ref mut map) => {
                        map.push((rmpv::Value::String(key.into()), serializer.output));
                    }
                    _ => return Err(Error::Message("expected map".to_string())),
                }
                Ok(())
            }
            _ => Err(Error::Message("expected array".to_string())),
        }
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::collections::HashMap;

    use serde_derive::Serialize;
    use serde_with::{serde_as, Bytes};

    #[test]
    fn test_ext_struct() {
        #[serde_as]
        #[derive(Serialize)]
        #[serde(rename = "_ExtStruct")]
        struct Foo(#[serde_as(as = "(_, Bytes)")] (i8, Vec<u8>));

        let foo = Foo((42, vec![1, 2, 3]));
        assert_eq!(to_value(&foo).unwrap(), rmpv::Value::Ext(42, vec![1, 2, 3]));
    }

    #[test]
    fn test_serialize() {
        let v: u64 = 23;
        assert_eq!(to_value(&v).unwrap(), rmpv::Value::from(23));
        assert_eq!(
            to_value(&[1, 2, 3]).unwrap(),
            rmpv::Value::Array(vec![
                rmpv::Value::from(1),
                rmpv::Value::from(2),
                rmpv::Value::from(3)
            ])
        );

        let f32val: f32 = 42.0;
        assert_eq!(to_value(&f32val).unwrap(), rmpv::Value::F32(42.0));
        let f64val: f64 = 42.0;
        assert_eq!(to_value(&f64val).unwrap(), rmpv::Value::F64(42.0));

        #[derive(Serialize)]
        struct TupleStruct(u8, u8);
        assert_eq!(
            to_value(&TupleStruct(1, 2)).unwrap(),
            rmpv::Value::Array(vec![rmpv::Value::from(1), rmpv::Value::from(2)])
        );

        #[derive(Serialize)]
        enum TEnum {
            Tuple(u8, u8),
            Newtype(u8),
            Struct { a: u8, b: u8 },
        }
        assert_eq!(
            to_value(&TEnum::Tuple(1, 2)).unwrap(),
            rmpv::Value::Array(vec![
                rmpv::Value::String("TEnum".into()),
                rmpv::Value::String("Tuple".into()),
                rmpv::Value::from(1),
                rmpv::Value::from(2)
            ])
        );
        assert_eq!(
            to_value(&TEnum::Newtype(2)).unwrap(),
            rmpv::Value::Array(vec![
                rmpv::Value::String("TEnum".into()),
                rmpv::Value::String("Newtype".into()),
                rmpv::Value::from(2)
            ])
        );
        assert_eq!(
            to_value(&TEnum::Struct { a: 1, b: 2 }).unwrap(),
            rmpv::Value::Array(vec![
                rmpv::Value::String("TEnum".into()),
                rmpv::Value::String("Struct".into()),
                rmpv::Value::Map(vec![
                    (rmpv::Value::String("a".into()), rmpv::Value::from(1)),
                    (rmpv::Value::String("b".into()), rmpv::Value::from(2))
                ])
            ])
        );

        let map = {
            let mut map = HashMap::new();
            map.insert("a", 1);
            map
        };
        assert_eq!(
            to_value(&map).unwrap(),
            rmpv::Value::Map(vec![(
                rmpv::Value::String("a".into()),
                rmpv::Value::from(1)
            ),])
        );

        #[derive(Serialize)]
        struct S {
            a: u8,
            b: u8,
        }
        assert_eq!(
            to_value(&S { a: 1, b: 2 }).unwrap(),
            rmpv::Value::Map(vec![
                (rmpv::Value::String("a".into()), rmpv::Value::from(1)),
                (rmpv::Value::String("b".into()), rmpv::Value::from(2))
            ])
        );
    }
}
