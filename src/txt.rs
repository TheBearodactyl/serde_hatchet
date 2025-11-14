use crate::{Error, Result};
use serde::{Deserialize, Serialize};

#[inline]
pub fn to_string<T>(value: &T) -> Result<String>
where
    T: Serialize,
{
    let bytes = crate::ser::to_bytes(value)?;
    Ok(bytes_to_hex(&bytes))
}

#[inline]
pub fn from_string<'a, T>(input: &'a str) -> Result<T>
where
    T: for<'de> Deserialize<'de>,
{
    let bytes = hex_to_bytes(input)?;
    from_string_impl(&bytes)
}

fn from_string_impl<T>(bytes: &[u8]) -> Result<T>
where
    T: for<'de> Deserialize<'de>,
{
    crate::de::from_bytes(bytes)
}

#[inline]
fn bytes_to_hex(bytes: &[u8]) -> String {
    const HEX_CHARS: &[u8; 16] = b"0123456789abcdef";

    let mut hex = String::with_capacity(bytes.len() * 2);
    for &byte in bytes {
        hex.push(HEX_CHARS[(byte >> 4) as usize] as char);
        hex.push(HEX_CHARS[(byte & 0x0f) as usize] as char);
    }
    hex
}

fn hex_to_bytes(hex: &str) -> Result<Vec<u8>> {
    let hex_clean: String = hex.chars().filter(|c| !c.is_whitespace()).collect();

    if !hex_clean.len().is_multiple_of(2) {
        return Err(Error::Message("Hex string must have even length".into()));
    }

    let mut bytes = Vec::with_capacity(hex_clean.len() / 2);
    let hex_bytes = hex_clean.as_bytes();

    for i in (0..hex_bytes.len()).step_by(2) {
        let high = hex_char_to_nibble(hex_bytes[i])?;
        let low = hex_char_to_nibble(hex_bytes[i + 1])?;
        bytes.push((high << 4) | low);
    }

    Ok(bytes)
}

#[inline]
fn hex_char_to_nibble(c: u8) -> Result<u8> {
    match c {
        b'0'..=b'9' => Ok(c - b'0'),
        b'a'..=b'f' => Ok(c - b'a' + 10),
        b'A'..=b'F' => Ok(c - b'A' + 10),
        _ => Err(Error::Message(format!(
            "Invalid hex character: '{}'",
            c as char
        ))),
    }
}
