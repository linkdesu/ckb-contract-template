use std::error::Error;
use std::str;

use ckb_types::bytes;
use ckb_types::packed::{Byte, Byte32, Bytes, Script};
use ckb_types::prelude::{Builder, Entity};
use serde_json::Value;

use super::constants::*;
use super::error;

pub fn hex_to_bytes(input: &str) -> Vec<u8> {
    let hex = input.trim_start_matches("0x");
    if hex == "" {
        Vec::new()
    } else {
        hex::decode(hex).expect("Expect input to valid hex")
    }
}

pub fn hex_to_bytes_2(input: &str) -> bytes::Bytes {
    let hex = input.trim_start_matches("0x");
    if hex == "" {
        bytes::Bytes::new()
    } else {
        let data: Vec<u8> = hex::decode(hex).expect("Expect input to valid hex");
        bytes::Bytes::from(data)
    }
}

pub fn bytes_to_hex(input: &[u8]) -> String {
    if input.is_empty() {
        String::from("0x")
    } else {
        String::from("0x") + &hex::encode(input)
    }
}

pub fn hex_to_byte32(input: &str) -> Result<Byte32, Box<dyn Error>> {
    let bytes = hex_to_bytes(input);
    Ok(Byte32::from_compatible_slice(&bytes).expect("Convert to Byte32 should not fail"))
}

pub fn slice_to_byte32(slice: &[u8]) -> Byte32 {
    Byte32::from_compatible_slice(slice).expect("Convert to Byte32 should not fail")
}

pub fn slice_to_bytes(slice: &[u8]) -> Bytes {
    Bytes::new_builder()
        .set(slice.to_owned().into_iter().map(Byte::new).collect())
        .build()
}

pub fn hex_to_u64(input: &str) -> Result<u64, Box<dyn Error>> {
    let hex = input.trim_start_matches("0x");
    if hex == "" {
        Ok(0u64)
    } else {
        Ok(u64::from_str_radix(hex, 16)?)
    }
}

pub fn merge_json(target: &mut Value, source: Value) {
    if source.is_null() {
        return;
    }

    match (target, source) {
        (a @ &mut Value::Object(_), Value::Object(b)) => {
            let a = a.as_object_mut().unwrap();
            for (k, v) in b {
                merge_json(a.entry(k).or_insert(Value::Null), v);
            }
        }
        (a @ &mut Value::Array(_), Value::Array(b)) => {
            let a = a.as_array_mut().unwrap();
            for v in b {
                a.push(v);
            }
        }
        (a, b) => *a = b,
    }
}

pub fn get_type_id_bytes(name: &str) -> Vec<u8> {
    hex_to_bytes(
        TYPE_ID_TABLE
            .get(name)
            .expect(&format!("Can not find type ID for {}", name)),
    )
}

/// Parse u64 in JSON
///
/// Support both **number** and **string** format.
pub fn parse_json_u64(field_name: &str, field: &Value, default: Option<u64>) -> u64 {
    if let Some(val) = field.as_u64() {
        val
    } else if let Some(val) = field.as_str() {
        val.replace("_", "")
            .parse()
            .expect(&format!("{} should be u64 in string", field_name))
    } else {
        if let Some(val) = default {
            return val;
        } else {
            panic!("{} is missing", field_name);
        }
    }
}

/// Parse u32 in JSON
///
/// Support both **number** and **string** format.
pub fn parse_json_u32(field_name: &str, field: &Value, default: Option<u32>) -> u32 {
    if let Some(val) = field.as_u64() {
        val as u32
    } else if let Some(val) = field.as_str() {
        val.replace("_", "")
            .parse()
            .expect(&format!("{} should be u32 in string", field_name))
    } else {
        if let Some(val) = default {
            return val;
        } else {
            panic!("{} is missing", field_name);
        }
    }
}

/// Parse u8 in JSON
pub fn parse_json_u8(field_name: &str, field: &Value, default: Option<u8>) -> u8 {
    if let Some(val) = field.as_u64() {
        if val > u8::MAX as u64 {
            panic!("{} should be u8", field_name)
        } else {
            val as u8
        }
    } else if let Some(val) = field.as_str() {
        val.replace("_", "")
            .parse()
            .expect(&format!("{} should be u8 in string", field_name))
    } else {
        if let Some(val) = default {
            return val;
        } else {
            panic!("{} is missing", field_name);
        }
    }
}

/// Parse hex string in JSON
///
/// Prefix "0x" is optional.
pub fn parse_json_hex(field_name: &str, field: &Value) -> Vec<u8> {
    let mut hex = field.as_str().expect(&format!("{} is missing", field_name));
    hex = hex.trim_start_matches("0x");

    if hex == "" {
        Vec::new()
    } else {
        hex::decode(hex).expect(&format!("{} is should be hex string", field_name))
    }
}

/// Parse hex string in JSON, if it is not exist return the default value.
pub fn parse_json_hex_with_default(field_name: &str, field: &Value, default: Vec<u8>) -> Vec<u8> {
    if field.is_null() {
        default
    } else {
        parse_json_hex(field_name, field)
    }
}

/// Parse string in JSON
///
/// All string will be treated as utf8 encoding.
pub fn parse_json_str<'a>(field_name: &str, field: &'a Value) -> &'a str {
    field.as_str().expect(&format!("{} is missing", field_name))
}

pub fn parse_json_str_with_default<'a>(field_name: &str, field: &'a Value, default: &'a str) -> &'a str {
    if field.is_null() {
        default
    } else {
        parse_json_str(field_name, field)
    }
}

/// Parse array in JSON
pub fn parse_json_array<'a>(field_name: &str, field: &'a Value) -> &'a [Value] {
    field
        .as_array()
        .map(|v| v.as_slice())
        .expect(&format!("{} is missing", field_name))
}
