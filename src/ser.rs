use crate::error::{Error, Result};
use serde::ser::{self, Serialize};
use std::io::Write;

pub struct Serializer<W> {
    writer: W,
}

impl<W: Write> Serializer<W> {
    #[inline]
    fn new(writer: W) -> Self {
        Serializer { writer }
    }

    #[inline]
    fn write_varint(&mut self, value: u64) -> Result<()> {
        if value < 0x80 {
            return self.writer.write_all(&[value as u8]).map_err(Into::into);
        }

        if value < 0x4000 {
            let bytes = [((value & 0x7F) | 0x80) as u8, (value >> 7) as u8];
            return self.writer.write_all(&bytes).map_err(Into::into);
        }

        let mut buf = [0u8; 10];
        let mut v = value;
        let mut i = 0;

        while v >= 0x80 {
            buf[i] = (v as u8 & 0x7F) | 0x80;
            v >>= 7;
            i += 1;
        }
        buf[i] = v as u8;

        self.writer.write_all(&buf[..=i])?;
        Ok(())
    }

    #[inline]
    fn write_signed_varint(&mut self, value: i64) -> Result<()> {
        let zigzag = ((value << 1) ^ (value >> 63)) as u64;
        self.write_varint(zigzag)
    }

    #[inline]
    fn write_compact_len(&mut self, len: usize) -> Result<()> {
        self.write_varint(len as u64)
    }
}

#[inline]
pub fn to_bytes<T>(value: &T) -> Result<Vec<u8>>
where
    T: Serialize,
{
    let mut output = Vec::new();
    value.serialize(&mut Serializer::new(&mut output))?;
    Ok(output)
}

#[inline]
pub fn to_bytes_with_capacity<T>(value: &T, capacity: usize) -> Result<Vec<u8>>
where
    T: Serialize,
{
    let mut output = Vec::with_capacity(capacity);
    value.serialize(&mut Serializer::new(&mut output))?;
    Ok(output)
}

impl<W: Write> ser::Serializer for &mut Serializer<W> {
    type Ok = ();
    type Error = Error;

    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    #[inline]
    fn serialize_bool(self, v: bool) -> Result<()> {
        self.writer.write_all(&[v as u8])?;
        Ok(())
    }

    #[inline]
    fn serialize_i8(self, v: i8) -> Result<()> {
        self.write_signed_varint(v as i64)
    }

    #[inline]
    fn serialize_i16(self, v: i16) -> Result<()> {
        self.write_signed_varint(v as i64)
    }

    #[inline]
    fn serialize_i32(self, v: i32) -> Result<()> {
        self.write_signed_varint(v as i64)
    }

    #[inline]
    fn serialize_i64(self, v: i64) -> Result<()> {
        self.write_signed_varint(v)
    }

    #[inline]
    fn serialize_u8(self, v: u8) -> Result<()> {
        self.write_varint(v as u64)
    }

    #[inline]
    fn serialize_u16(self, v: u16) -> Result<()> {
        self.write_varint(v as u64)
    }

    #[inline]
    fn serialize_u32(self, v: u32) -> Result<()> {
        self.write_varint(v as u64)
    }

    #[inline]
    fn serialize_u64(self, v: u64) -> Result<()> {
        self.write_varint(v)
    }

    #[inline]
    fn serialize_f32(self, v: f32) -> Result<()> {
        self.writer.write_all(&v.to_le_bytes())?;
        Ok(())
    }

    #[inline]
    fn serialize_f64(self, v: f64) -> Result<()> {
        self.writer.write_all(&v.to_le_bytes())?;
        Ok(())
    }

    #[inline]
    fn serialize_char(self, v: char) -> Result<()> {
        let mut buf = [0; 4];
        let s = v.encode_utf8(&mut buf);
        self.serialize_str(s)
    }

    #[inline]
    fn serialize_str(self, v: &str) -> Result<()> {
        self.write_compact_len(v.len())?;
        self.writer.write_all(v.as_bytes())?;
        Ok(())
    }

    #[inline]
    fn serialize_bytes(self, v: &[u8]) -> Result<()> {
        self.write_compact_len(v.len())?;
        self.writer.write_all(v)?;
        Ok(())
    }

    #[inline]
    fn serialize_none(self) -> Result<()> {
        self.writer.write_all(&[0])?;
        Ok(())
    }

    #[inline]
    fn serialize_some<T>(self, value: &T) -> Result<()>
    where
        T: Serialize,
        T: ?Sized,
    {
        self.writer.write_all(&[1])?;
        value.serialize(self)
    }

    #[inline]
    fn serialize_unit(self) -> Result<()> {
        Ok(())
    }

    #[inline]
    fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
        self.serialize_unit()
    }

    #[inline]
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
    ) -> Result<()> {
        self.write_varint(variant_index as u64)
    }

    #[inline]
    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<()>
    where
        T: Serialize,
        T: ?Sized,
    {
        value.serialize(self)
    }

    #[inline]
    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
        value: &T,
    ) -> Result<()>
    where
        T: Serialize,
        T: ?Sized,
    {
        self.write_varint(variant_index as u64)?;
        value.serialize(self)
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        let len = len.ok_or_else(|| Error::Message("Sequences must have known length".into()))?;
        self.write_compact_len(len)?;
        Ok(self)
    }

    #[inline]
    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        Ok(self)
    }

    #[inline]
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        Ok(self)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        self.write_varint(variant_index as u64)?;
        Ok(self)
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap> {
        let len = len.ok_or_else(|| Error::Message("Maps must have known length".into()))?;
        self.write_compact_len(len)?;
        Ok(self)
    }

    #[inline]
    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        Ok(self)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        self.write_varint(variant_index as u64)?;
        Ok(self)
    }
}

impl<W: Write> ser::SerializeSeq for &mut Serializer<W> {
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: Serialize,
        T: ?Sized,
    {
        value.serialize(&mut **self)
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<W: Write> ser::SerializeTuple for &mut Serializer<W> {
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: Serialize,
        T: ?Sized,
    {
        value.serialize(&mut **self)
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<W: Write> ser::SerializeTupleStruct for &mut Serializer<W> {
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: Serialize,
        T: ?Sized,
    {
        value.serialize(&mut **self)
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<W: Write> ser::SerializeTupleVariant for &mut Serializer<W> {
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: Serialize,
        T: ?Sized,
    {
        value.serialize(&mut **self)
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<W: Write> ser::SerializeMap for &mut Serializer<W> {
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
    where
        T: Serialize + ?Sized,
    {
        key.serialize(&mut **self)
    }

    #[inline]
    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
    where
        T: Serialize,
        T: ?Sized,
    {
        value.serialize(&mut **self)
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<W: Write> ser::SerializeStruct for &mut Serializer<W> {
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_field<T>(&mut self, _key: &'static str, value: &T) -> Result<()>
    where
        T: Serialize + ?Sized,
    {
        value.serialize(&mut **self)
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<W: Write> ser::SerializeStructVariant for &mut Serializer<W> {
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_field<T>(&mut self, _key: &'static str, value: &T) -> Result<()>
    where
        T: Serialize + ?Sized,
    {
        value.serialize(&mut **self)
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}
