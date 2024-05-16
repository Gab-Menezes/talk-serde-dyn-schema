use std::collections::HashSet;

use crate::{
    ty::{Field, Ty},
    JsonValue,
};
use bincode::Options;
use serde_json::Map;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("expected {expected}, got {got}")]
    UnexpectedType { expected: &'static str, got: &'static str },
    #[error("an element in a byte array was not an integer between 0 and 255")]
    NotAByte,
    #[error("expected {expected_len} fields ({expected:?}), got {got_len} ({got:?}). Missing: {missing:?} | Extra: {extra:?}")]
    FieldMismatch {
        expected_len: usize,
        expected: HashSet<String>,
        got_len: usize,
        got: HashSet<String>,
        missing: HashSet<String>,
        extra: HashSet<String>,
    },
    #[error("can't serialize data")]
    Serialization,
}

pub type Result<T> = std::result::Result<T, Error>;

pub fn serialize(ty: &Ty, value: &JsonValue) -> Result<Vec<u8>> {
    let mut buffer = Vec::new();
    serialize_into(ty, value, &mut buffer)?;
    Ok(buffer)
}

pub fn serialize_into(ty: &Ty, value: &JsonValue, buffer: &mut Vec<u8>) -> Result<()> {
    let opt = bincode::DefaultOptions::new();
    match ty {
        Ty::Bool => {
            let value = value.as_bool().ok_or_else(|| unexpected_type("a boolean", value))?;
            opt.serialize_into(buffer, &value).map_err(|_| Error::Serialization)?;
        }
        Ty::U64 => {
            let value = value
                .as_u64()
                .ok_or_else(|| unexpected_type("a non-negative integer", value))?;
            opt.serialize_into(buffer, &value).map_err(|_| Error::Serialization)?;
        }
        Ty::I64 => {
            let value = value.as_i64().ok_or_else(|| unexpected_type("an integer", value))?;
            opt.serialize_into(buffer, &value).map_err(|_| Error::Serialization)?;
        }
        Ty::F64 => {
            let value = value.as_f64().ok_or_else(|| unexpected_type("a number", value))?;
            opt.serialize_into(buffer, &value).map_err(|_| Error::Serialization)?;
        }
        Ty::Bytes => {
            let bytes = value.as_array().ok_or_else(|| unexpected_type("a byte array", value))?;
            opt.serialize_into(&mut *buffer, &bytes.len())
                .map_err(|_| Error::Serialization)?;
            for b in bytes {
                let b: u8 = b
                    .as_u64()
                    .ok_or(Error::NotAByte)?
                    .try_into()
                    .map_err(|_| Error::NotAByte)?;
                opt.serialize_into(&mut *buffer, &b).map_err(|_| Error::Serialization)?;
            }
        }
        Ty::String => {
            let value = value.as_str().ok_or_else(|| unexpected_type("a string", value))?;
            opt.serialize_into(buffer, value).map_err(|_| Error::Serialization)?;
        }
        Ty::Array { inner } => {
            let array = value.as_array().ok_or_else(|| unexpected_type("an array", value))?;
            opt.serialize_into(&mut *buffer, &array.len())
                .map_err(|_| Error::Serialization)?;
            for element in array {
                serialize_into(inner, element, buffer)?;
            }
        }
        Ty::Struct { fields } => {
            let object = value.as_object().ok_or_else(|| unexpected_type("an object", value))?;

            if fields.len() != object.len() {
                return Err(field_mismatch(fields, object));
            }

            for field in fields.iter() {
                let value = object.get(&*field.name).ok_or_else(|| field_mismatch(fields, object))?;
                serialize_into(&field.ty, value, buffer)?;
            }
        }
    }
    Ok(())
}

fn unexpected_type(expected: &'static str, value: &JsonValue) -> Error {
    let got = match value {
        JsonValue::Null => "null",
        JsonValue::Bool(_) => "a boolean",
        JsonValue::Number(_) => "a number",
        JsonValue::String(_) => "a string",
        JsonValue::Array(_) => "an array",
        JsonValue::Object(_) => "an object",
    };
    Error::UnexpectedType { expected, got }
}

fn field_mismatch<'a>(fields: &Box<[Field]>, object: &Map<String, JsonValue>) -> Error {
    let expected: HashSet<_> = fields.into_iter().map(|field| field.name.to_string()).collect();
    let got: HashSet<_> = object.into_iter().map(|(name, _)| name.clone()).collect();
    let extra: HashSet<_> = got.difference(&expected).cloned().collect();
    let missing: HashSet<_> = expected.difference(&got).cloned().collect();
    Error::FieldMismatch {
        expected_len: fields.len(),
        expected,
        got_len: object.len(),
        got,
        missing,
        extra,
    }
}
