use crate::error::{Error, Result};
use serde::de::{self, Deserialize, DeserializeSeed, Visitor};

pub struct Deserializer<'de> {
    input: &'de [u8],
    pos: usize,
}

impl<'de> Deserializer<'de> {
    #[inline]
    fn new(input: &'de [u8]) -> Self {
        Deserializer { input, pos: 0 }
    }

    #[inline(always)]
    fn read_byte(&mut self) -> Result<u8> {
        if let Some(&byte) = self.input.get(self.pos) {
            self.pos += 1;
            Ok(byte)
        } else {
            Err(Error::Eof)
        }
    }

    #[inline(always)]
    fn read_bytes(&mut self, n: usize) -> Result<&'de [u8]> {
        let end = self.pos.checked_add(n).ok_or(Error::Eof)?;
        if let Some(bytes) = self.input.get(self.pos..end) {
            self.pos = end;
            Ok(bytes)
        } else {
            Err(Error::Eof)
        }
    }

    #[inline]
    fn read_varint(&mut self) -> Result<u64> {
        let byte = self.read_byte()?;

        if byte < 0x80 {
            return Ok(byte as u64);
        }

        let mut result = (byte & 0x7F) as u64;
        let mut shift = 7;

        loop {
            let byte = self.read_byte()?;
            result |= ((byte & 0x7F) as u64) << shift;

            if byte < 0x80 {
                return Ok(result);
            }

            shift += 7;
            if shift >= 64 {
                return Err(Error::InvalidVarint);
            }
        }
    }

    #[inline]
    fn read_signed_varint(&mut self) -> Result<i64> {
        let zigzag = self.read_varint()?;
        Ok(((zigzag >> 1) as i64) ^ -((zigzag & 1) as i64))
    }

    #[inline]
    fn read_compact_len(&mut self) -> Result<usize> {
        self.read_varint().map(|v| v as usize)
    }
}

#[inline]
pub fn from_bytes<'a, T>(input: &'a [u8]) -> Result<T>
where
    T: Deserialize<'a>,
{
    let mut deserializer = Deserializer::new(input);
    T::deserialize(&mut deserializer)
}

impl<'de> de::Deserializer<'de> for &mut Deserializer<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        Err(Error::Message("deserialize_any not supported".into()))
    }

    #[inline]
    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let byte = self.read_byte()?;
        match byte {
            0 => visitor.visit_bool(false),
            1 => visitor.visit_bool(true),
            _ => Err(Error::InvalidBool),
        }
    }

    #[inline]
    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i8(self.read_signed_varint()? as i8)
    }

    #[inline]
    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i16(self.read_signed_varint()? as i16)
    }

    #[inline]
    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i32(self.read_signed_varint()? as i32)
    }

    #[inline]
    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i64(self.read_signed_varint()?)
    }

    #[inline]
    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u8(self.read_varint()? as u8)
    }

    #[inline]
    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u16(self.read_varint()? as u16)
    }

    #[inline]
    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u32(self.read_varint()? as u32)
    }

    #[inline]
    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u64(self.read_varint()?)
    }

    #[inline]
    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let bytes = self.read_bytes(4)?;
        let value = f32::from_le_bytes(unsafe { *(bytes.as_ptr() as *const [u8; 4]) });
        visitor.visit_f32(value)
    }

    #[inline]
    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let bytes = self.read_bytes(8)?;
        let value = f64::from_le_bytes(unsafe { *(bytes.as_ptr() as *const [u8; 8]) });
        visitor.visit_f64(value)
    }

    #[inline]
    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let len = self.read_compact_len()?;
        let bytes = self.read_bytes(len)?;
        let s = std::str::from_utf8(bytes).map_err(|_| Error::InvalidUtf8)?;
        let ch = s
            .chars()
            .next()
            .ok_or(Error::Message("Empty char".into()))?;
        visitor.visit_char(ch)
    }

    #[inline]
    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let len = self.read_compact_len()?;
        let bytes = self.read_bytes(len)?;
        let s = std::str::from_utf8(bytes).map_err(|_| Error::InvalidUtf8)?;
        visitor.visit_borrowed_str(s)
    }

    #[inline]
    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    #[inline]
    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let len = self.read_compact_len()?;
        let bytes = self.read_bytes(len)?;
        visitor.visit_borrowed_bytes(bytes)
    }

    #[inline]
    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_bytes(visitor)
    }

    #[inline]
    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let tag = self.read_byte()?;
        match tag {
            0 => visitor.visit_none(),
            1 => visitor.visit_some(self),
            _ => Err(Error::Message("Invalid option tag".into())),
        }
    }

    #[inline]
    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_unit()
    }

    #[inline]
    fn deserialize_unit_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    #[inline]
    fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let len = self.read_compact_len()?;
        visitor.visit_seq(SeqAccess::new(self, len))
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_seq(SeqAccess::new(self, len))
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_tuple(len, visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let len = self.read_compact_len()?;
        visitor.visit_map(MapAccess::new(self, len))
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_tuple(fields.len(), visitor)
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
        visitor.visit_enum(EnumAccess::new(self))
    }

    #[inline]
    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_u32(visitor)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
}

struct SeqAccess<'a, 'de> {
    de: &'a mut Deserializer<'de>,
    remaining: usize,
}

impl<'a, 'de> SeqAccess<'a, 'de> {
    #[inline]
    fn new(de: &'a mut Deserializer<'de>, len: usize) -> Self {
        SeqAccess { de, remaining: len }
    }
}

impl<'de, 'a> de::SeqAccess<'de> for SeqAccess<'a, 'de> {
    type Error = Error;

    #[inline]
    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: DeserializeSeed<'de>,
    {
        if self.remaining == 0 {
            return Ok(None);
        }
        self.remaining -= 1;
        seed.deserialize(&mut *self.de).map(Some)
    }
}

struct MapAccess<'a, 'de> {
    de: &'a mut Deserializer<'de>,
    remaining: usize,
}

impl<'a, 'de> MapAccess<'a, 'de> {
    #[inline]
    fn new(de: &'a mut Deserializer<'de>, len: usize) -> Self {
        MapAccess { de, remaining: len }
    }
}

impl<'de, 'a> de::MapAccess<'de> for MapAccess<'a, 'de> {
    type Error = Error;

    #[inline]
    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: DeserializeSeed<'de>,
    {
        if self.remaining == 0 {
            return Ok(None);
        }
        self.remaining -= 1;
        seed.deserialize(&mut *self.de).map(Some)
    }

    #[inline]
    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: DeserializeSeed<'de>,
    {
        seed.deserialize(&mut *self.de)
    }
}

struct EnumAccess<'a, 'de> {
    de: &'a mut Deserializer<'de>,
}

impl<'a, 'de> EnumAccess<'a, 'de> {
    #[inline]
    fn new(de: &'a mut Deserializer<'de>) -> Self {
        EnumAccess { de }
    }
}

impl<'de, 'a> de::EnumAccess<'de> for EnumAccess<'a, 'de> {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)>
    where
        V: DeserializeSeed<'de>,
    {
        let val = seed.deserialize(&mut *self.de)?;
        Ok((val, self))
    }
}

impl<'de, 'a> de::VariantAccess<'de> for EnumAccess<'a, 'de> {
    type Error = Error;

    #[inline]
    fn unit_variant(self) -> Result<()> {
        Ok(())
    }

    #[inline]
    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
    where
        T: DeserializeSeed<'de>,
    {
        seed.deserialize(self.de)
    }

    fn tuple_variant<V>(self, len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        de::Deserializer::deserialize_tuple(self.de, len, visitor)
    }

    fn struct_variant<V>(self, fields: &'static [&'static str], visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        de::Deserializer::deserialize_tuple(self.de, fields.len(), visitor)
    }
}
