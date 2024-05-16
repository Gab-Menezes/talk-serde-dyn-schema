
use bincode::{Options};
use serde::{
    de::{DeserializeSeed, SeqAccess, Visitor}, Deserializer, Serialize,
};
use serde_json::Map;

use crate::ty::{Field, Ty};

pub fn deserialize(ty: &Ty, value: &[u8]) -> bincode::Result<serde_json::Value> {
    let opt = bincode::DefaultOptions::new();
    let seed = TypedValue { ty };
    opt.deserialize_from_seed(seed, value)
}

#[derive(Copy, Clone)]
struct TypedValue<'a> {
    ty: &'a Ty,
}

impl<'de, 'a> DeserializeSeed<'de> for TypedValue<'a> {
    type Value = serde_json::Value;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        match self.ty {
            Ty::Bool => deserializer.deserialize_bool(BoolVisitor),
            Ty::U64 => deserializer.deserialize_u64(IntVisitor),
            Ty::I64 => deserializer.deserialize_u64(IntVisitor),
            Ty::F64 => deserializer.deserialize_u64(FloatVisitor),
            Ty::Bytes => deserializer.deserialize_bytes(BytesVisitor),
            Ty::String => deserializer.deserialize_string(StringVisitor),
            Ty::Array { inner } => deserializer.deserialize_seq(ArrayVisitor { inner }),
            Ty::Struct { fields } => deserializer.deserialize_tuple(fields.len(), StructVisitor { fields }),
        }
    }
}

struct IntVisitor;
struct BoolVisitor;
struct FloatVisitor;
struct StringVisitor;
struct BytesVisitor;

impl<'de> Visitor<'de> for IntVisitor {
    type Value = serde_json::Value;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "an integer")
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(v.serialize(serde_json::value::Serializer).unwrap())
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(v.serialize(serde_json::value::Serializer).unwrap())
    }
}
impl<'de> Visitor<'de> for BoolVisitor {
    type Value = serde_json::Value;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "a bool")
    }

    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(v.serialize(serde_json::value::Serializer).unwrap())
    }
}
impl<'de> Visitor<'de> for FloatVisitor {
    type Value = serde_json::Value;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "a float")
    }

    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(v.serialize(serde_json::value::Serializer).unwrap())
    }
}
impl<'de> Visitor<'de> for StringVisitor {
    type Value = serde_json::Value;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "a string")
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(v.serialize(serde_json::value::Serializer).unwrap())
    }
}
impl<'de> Visitor<'de> for BytesVisitor {
    type Value = serde_json::Value;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "a strubg")
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(v.serialize(serde_json::value::Serializer).unwrap())
    }
}

struct StructVisitor<'a> {
    pub fields: &'a [Field],
}
impl<'a, 'de> Visitor<'de> for StructVisitor<'a> {
    type Value = serde_json::Value;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "an object")
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        let mut map = Map::with_capacity(self.fields.len());
        for field in self.fields {
            let seed = TypedValue { ty: &field.ty };
            let v = seq
                .next_element_seed(seed)?
                .ok_or_else(|| serde::de::Error::custom(format!("missing value for field {:?}", self.fields)))?;
            map.insert(field.name.to_string(), v);
        }
        Ok(serde_json::Value::Object(map))
    }
}

struct ArrayVisitor<'a> {
    pub inner: &'a Ty,
}
impl<'a, 'de> Visitor<'de> for ArrayVisitor<'a> {
    type Value = serde_json::Value;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "an array")
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        let tv = TypedValue { ty: &self.inner };
        let mut v = Vec::new();
        while let Some(val) = seq.next_element_seed(tv)? {
            v.push(val);
        }
        Ok(serde_json::Value::Array(v))
    }
}
