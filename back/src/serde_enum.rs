use serde::ser::{self, Impossible, Serialize};
use serde::de::{self, Deserialize, IntoDeserializer, Visitor};
use std::{error, result};
use std::fmt::{self, Display, Formatter};

#[derive(Debug)]
pub enum Error {
    Message(String),
    NotEnum,
    EmptyString,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str(error::Error::description(self))
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Message(ref string) => string,
            Error::NotEnum => "not an enum",
            Error::EmptyString => "empty string",
        }
    }
}

impl ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Error {
        Error::Message(msg.to_string())
    }
}

impl de::Error for Error {
    fn custom<T: Display>(msg: T) -> Error {
        Error::Message(msg.to_string())
    }
}

type Result<T> = result::Result<T, Error>;

struct Serializer {
    output: String,
}

pub fn to_string<T>(value: &T) -> Result<String>
where
    T: Serialize,
{
    let mut serializer = Serializer {
        output: String::new(),
    };
    value.serialize(&mut serializer)?;
    Ok(serializer.output)
}

impl<'a> ser::Serializer for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    type SerializeSeq = Impossible<(), Error>;
    type SerializeTuple = Impossible<(), Error>;
    type SerializeTupleStruct = Impossible<(), Error>;
    type SerializeTupleVariant = Impossible<(), Error>;
    type SerializeMap = Impossible<(), Error>;
    type SerializeStruct = Impossible<(), Error>;
    type SerializeStructVariant = Impossible<(), Error>;

    fn serialize_bool(self, _: bool) -> Result<()> {
        Err(Error::NotEnum)
    }

    fn serialize_i8(self, _: i8) -> Result<()> {
        Err(Error::NotEnum)
    }

    fn serialize_i16(self, _: i16) -> Result<()> {
        Err(Error::NotEnum)
    }

    fn serialize_i32(self, _: i32) -> Result<()> {
        Err(Error::NotEnum)
    }

    fn serialize_i64(self, _: i64) -> Result<()> {
        Err(Error::NotEnum)
    }

    fn serialize_u8(self, _: u8) -> Result<()> {
        Err(Error::NotEnum)
    }

    fn serialize_u16(self, _: u16) -> Result<()> {
        Err(Error::NotEnum)
    }

    fn serialize_u32(self, _: u32) -> Result<()> {
        Err(Error::NotEnum)
    }

    fn serialize_u64(self, _: u64) -> Result<()> {
        Err(Error::NotEnum)
    }

    fn serialize_f32(self, _: f32) -> Result<()> {
        Err(Error::NotEnum)
    }

    fn serialize_f64(self, _: f64) -> Result<()> {
        Err(Error::NotEnum)
    }

    fn serialize_char(self, _: char) -> Result<()> {
        Err(Error::NotEnum)
    }

    fn serialize_str(self, _: &str) -> Result<()> {
        Err(Error::NotEnum)
    }

    fn serialize_bytes(self, _: &[u8]) -> Result<()> {
        Err(Error::NotEnum)
    }

    fn serialize_none(self) -> Result<()> {
        Err(Error::NotEnum)
    }

    fn serialize_some<T>(self, _: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        Err(Error::NotEnum)
    }

    fn serialize_unit(self) -> Result<()> {
        Err(Error::NotEnum)
    }

    fn serialize_unit_struct(self, _: &'static str) -> Result<()> {
        Err(Error::NotEnum)
    }

    fn serialize_unit_variant(self, _: &'static str, _: u32, variant: &'static str) -> Result<()> {
        self.output += variant;
        Ok(())
    }

    fn serialize_newtype_struct<T>(self, _: &'static str, _: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        Err(Error::NotEnum)
    }

    fn serialize_newtype_variant<T>(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: &T,
    ) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        Err(Error::NotEnum)
    }

    fn serialize_seq(self, _: Option<usize>) -> Result<Self::SerializeSeq> {
        Err(Error::NotEnum)
    }

    fn serialize_tuple(self, _: usize) -> Result<Self::SerializeTuple> {
        Err(Error::NotEnum)
    }

    fn serialize_tuple_struct(
        self,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        Err(Error::NotEnum)
    }

    fn serialize_tuple_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        Err(Error::NotEnum)
    }

    fn serialize_map(self, _: Option<usize>) -> Result<Self::SerializeMap> {
        Err(Error::NotEnum)
    }

    fn serialize_struct(self, _: &'static str, _: usize) -> Result<Self::SerializeStruct> {
        Err(Error::NotEnum)
    }

    fn serialize_struct_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeStructVariant> {
        Err(Error::NotEnum)
    }
}

struct Deserializer<'de> {
    input: &'de str,
}

impl<'de> Deserializer<'de> {
    fn new(input: &'de str) -> Self {
        Deserializer { input: input }
    }

    fn parse_string(&mut self) -> Result<&'de str> {
        let len = self.input.len();
        if len == 0 {
            return Err(Error::EmptyString);
        }

        let string = &self.input[..len];
        self.input = &self.input[len..len];

        Ok(string)
    }
}

pub fn from_str<'a, T>(s: &'a str) -> Result<T>
where
    T: Deserialize<'a>,
{
    let mut deserializer = Deserializer::new(s);
    T::deserialize(&mut deserializer)
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_enum("", &[], visitor)
    }

    fn deserialize_bool<V>(self, _: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        Err(Error::NotEnum)
    }

    fn deserialize_i8<V>(self, _: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        Err(Error::NotEnum)
    }

    fn deserialize_i16<V>(self, _: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        Err(Error::NotEnum)
    }

    fn deserialize_i32<V>(self, _: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        Err(Error::NotEnum)
    }

    fn deserialize_i64<V>(self, _: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        Err(Error::NotEnum)
    }

    fn deserialize_u8<V>(self, _: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        Err(Error::NotEnum)
    }

    fn deserialize_u16<V>(self, _: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        Err(Error::NotEnum)
    }

    fn deserialize_u32<V>(self, _: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        Err(Error::NotEnum)
    }

    fn deserialize_u64<V>(self, _: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        Err(Error::NotEnum)
    }

    fn deserialize_f32<V>(self, _: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        Err(Error::NotEnum)
    }

    fn deserialize_f64<V>(self, _: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        Err(Error::NotEnum)
    }

    fn deserialize_char<V>(self, _: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        Err(Error::NotEnum)
    }

    fn deserialize_str<V>(self, _: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        Err(Error::NotEnum)
    }

    fn deserialize_string<V>(self, _: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        Err(Error::NotEnum)
    }

    fn deserialize_bytes<V>(self, _: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        Err(Error::NotEnum)
    }

    fn deserialize_byte_buf<V>(self, _: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        Err(Error::NotEnum)
    }

    fn deserialize_option<V>(self, _: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        Err(Error::NotEnum)
    }

    fn deserialize_unit<V>(self, _: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        Err(Error::NotEnum)
    }

    fn deserialize_unit_struct<V>(self, _: &'static str, _: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        Err(Error::NotEnum)
    }

    fn deserialize_newtype_struct<V>(self, _: &'static str, _: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        Err(Error::NotEnum)
    }

    fn deserialize_seq<V>(self, _: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        Err(Error::NotEnum)
    }

    fn deserialize_tuple<V>(self, _: usize, _: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        Err(Error::NotEnum)
    }

    fn deserialize_tuple_struct<V>(self, _: &'static str, _: usize, _: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        Err(Error::NotEnum)
    }

    fn deserialize_map<V>(self, _: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        Err(Error::NotEnum)
    }

    fn deserialize_struct<V>(
        self,
        _: &'static str,
        _: &'static [&'static str],
        _: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        Err(Error::NotEnum)
    }

    fn deserialize_enum<V>(
        self,
        _: &'static str,
        _: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_enum(self.parse_string()?.into_deserializer())
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_borrowed_str(self.parse_string()?)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
}

#[test]
fn test_ser_enum() {
    #[derive(Serialize)]
    enum E {
        One,
        Two,
        Three,
    }

    let e = E::One;
    let expected = r"One";
    assert_eq!(to_string(&e).unwrap(), expected);

    let e = E::Two;
    let expected = r"Two";
    assert_eq!(to_string(&e).unwrap(), expected);

    let e = E::Three;
    let expected = r"Three";
    assert_eq!(to_string(&e).unwrap(), expected);
}

#[test]
fn test_de_enum() {
    #[derive(Deserialize, PartialEq, Debug)]
    enum E {
        One,
        Two,
        Three,
    }

    let e = r"One";
    let expected = E::One;
    assert_eq!(expected, from_str(e).unwrap());

    let e = r"Two";
    let expected = E::Two;
    assert_eq!(expected, from_str(e).unwrap());

    let e = r"Three";
    let expected = E::Three;
    assert_eq!(expected, from_str(e).unwrap());
}
