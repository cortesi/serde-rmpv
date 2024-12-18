mod de;
mod error;
mod ser;

pub use de::*;
pub use error::*;
pub use ser::*;

pub const MSGPACK_EXT_STRUCT_NAME: &str = "_ExtStruct";

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
