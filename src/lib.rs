//! Serde integration for the rmpv MessagePack Value type.
//!
//! This crate handles all Serde data model types and includes special support for MessagePack's
//! Ext type through the [`MSGPACK_EXT_STRUCT_NAME`] type annotation.

mod de;
mod error;
mod ser;

pub use error::Error;

/// Name of the Serde newtype struct to represent MessagePack's Ext type
///
/// MessagePack Ext format: Ext(tag, binary)
/// Serde data model: _ExtStruct((tag, binary))
///
/// # Example
/// ```rust,ignore
/// #[derive(Debug, PartialEq, Serialize, Deserialize)]
/// #[serde(rename = "_ExtStruct")]
/// struct ExtStruct((i8, Vec<u8>));
/// ```
pub const MSGPACK_EXT_STRUCT_NAME: &str = "_ExtStruct";

/// Deserializes rmpv::Value into a target type.
///
/// # Errors
/// Returns an error if:
/// - Value cannot be deserialized into target type
/// - Value contains unsupported or invalid data for target type
pub fn from_value<'a, T>(s: &'a rmpv::Value) -> Result<T, Error>
where
    T: serde::de::Deserialize<'a>,
{
    de::from_value(s)
}

/// Serializes a type into rmpv::Value.
///
/// # Errors
/// Returns an error if:
/// - Value cannot be serialized
/// - Value contains unsupported types
pub fn to_value<T>(value: &T) -> Result<rmpv::Value, Error>
where
    T: serde::ser::Serialize,
{
    ser::to_value(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    use serde_derive::{Deserialize, Serialize};
    use serde_with::{serde_as, Bytes};

    #[test]
    fn test_exttype_idemp() {
        #[serde_as]
        #[derive(Serialize, Deserialize, Debug, PartialEq)]
        #[serde(rename = "_ExtStruct")]
        struct Foo(#[serde_as(as = "(_, Bytes)")] (i8, Vec<u8>));

        let f = Foo((42, vec![1, 2, 3]));
        let val = to_value(&f).unwrap();
        let f2 = from_value(&val).unwrap();
        assert_eq!(f, f2);
    }
}
