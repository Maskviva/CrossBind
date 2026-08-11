use core::marker::PhantomData;

use crate::{Codec, Reader, Result, Writer};

macro_rules! scalar {
    ($(#[$doc:meta])* $name:ident => $value:ty, $read:ident, $write:ident) => {
        $(#[$doc])*
        pub struct $name;
        impl Codec for $name {
            type Value = $value;
            #[inline]
            fn read(r: &mut Reader<'_>) -> Result<$value> {
                r.$read()
            }
            #[inline]
            fn write(w: &mut Writer, v: &$value) {
                w.$write(*v)
            }
        }
    };
}

scalar!(
    Byte => u8, read_u8, write_u8);
scalar!(
    SByte => i8, read_i8, write_i8);
scalar!(
    Bool => bool, read_bool, write_bool);
scalar!(
    ShortLe => i16, read_i16_le, write_i16_le);
scalar!(
    UShortLe => u16, read_u16_le, write_u16_le);
scalar!(
    IntLe => i32, read_i32_le, write_i32_le);
scalar!(
    UIntLe => u32, read_u32_le, write_u32_le);
scalar!(
    IntBe => i32, read_i32_be, write_i32_be);
scalar!(
    Int64Le => i64, read_i64_le, write_i64_le);
scalar!(
    UInt64Le => u64, read_u64_le, write_u64_le);
scalar!(
    FloatLe => f32, read_f32_le, write_f32_le);
scalar!(
    DoubleLe => f64, read_f64_le, write_f64_le);
scalar!(
    VarInt => i32, read_varint, write_varint);
scalar!(
    UVarInt => u32, read_uvarint, write_uvarint);
scalar!(
    VarInt64 => i64, read_varint64, write_varint64);
scalar!(
    UVarInt64 => u64, read_uvarint64, write_uvarint64);

pub struct Str;
impl Codec for Str {
    type Value = String;
    fn read(r: &mut Reader<'_>) -> Result<String> {
        r.read_string()
    }
    fn write(w: &mut Writer, v: &String) {
        w.write_string(v)
    }
}

pub struct RemainingBytes;
impl Codec for RemainingBytes {
    type Value = Vec<u8>;
    fn read(r: &mut Reader<'_>) -> Result<Vec<u8>> {
        Ok(r.read_remaining().to_vec())
    }
    fn write(w: &mut Writer, v: &Vec<u8>) {
        w.write_bytes(v)
    }
}

pub struct ByteArray;
impl Codec for ByteArray {
    type Value = Vec<u8>;
    fn read(r: &mut Reader<'_>) -> Result<Vec<u8>> {
        let len = r.read_count()?;
        Ok(r.read_bytes(len)?.to_vec())
    }
    fn write(w: &mut Writer, v: &Vec<u8>) {
        w.write_count(v.len());
        w.write_bytes(v);
    }
}

pub struct BlockPos;
impl Codec for BlockPos {
    type Value = (i32, i32, i32);
    fn read(r: &mut Reader<'_>) -> Result<(i32, i32, i32)> {
        Ok((r.read_varint()?, r.read_varint()?, r.read_varint()?))
    }
    fn write(w: &mut Writer, v: &(i32, i32, i32)) {
        w.write_varint(v.0);
        w.write_varint(v.1);
        w.write_varint(v.2);
    }
}

pub struct NetworkBlockPos;
impl Codec for NetworkBlockPos {
    type Value = (i32, i32, i32);
    fn read(r: &mut Reader<'_>) -> Result<(i32, i32, i32)> {
        let x = r.read_varint()?;
        let y = r.read_uvarint()? as i32;
        let z = r.read_varint()?;
        Ok((x, y, z))
    }
    fn write(w: &mut Writer, v: &(i32, i32, i32)) {
        w.write_varint(v.0);
        w.write_uvarint(v.1 as u32);
        w.write_varint(v.2);
    }
}

pub struct Vec3;
impl Codec for Vec3 {
    type Value = (f32, f32, f32);
    fn read(r: &mut Reader<'_>) -> Result<(f32, f32, f32)> {
        Ok((r.read_f32_le()?, r.read_f32_le()?, r.read_f32_le()?))
    }
    fn write(w: &mut Writer, v: &(f32, f32, f32)) {
        w.write_f32_le(v.0);
        w.write_f32_le(v.1);
        w.write_f32_le(v.2);
    }
}

pub struct Vec2;
impl Codec for Vec2 {
    type Value = (f32, f32);
    fn read(r: &mut Reader<'_>) -> Result<(f32, f32)> {
        Ok((r.read_f32_le()?, r.read_f32_le()?))
    }
    fn write(w: &mut Writer, v: &(f32, f32)) {
        w.write_f32_le(v.0);
        w.write_f32_le(v.1);
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct MceUuid {
    pub msb: u64,
    pub lsb: u64,
}

pub struct Uuid;
impl Codec for Uuid {
    type Value = MceUuid;
    fn read(r: &mut Reader<'_>) -> Result<MceUuid> {
        Ok(MceUuid {
            msb: r.read_u64_le()?,
            lsb: r.read_u64_le()?,
        })
    }
    fn write(w: &mut Writer, v: &MceUuid) {
        w.write_u64_le(v.msb);
        w.write_u64_le(v.lsb);
    }
}

pub struct Array<C>(PhantomData<C>);
impl<C: Codec> Codec for Array<C> {
    type Value = Vec<C::Value>;
    fn read(r: &mut Reader<'_>) -> Result<Vec<C::Value>> {
        let count = r.read_count()?;
        let mut out = Vec::with_capacity(count);
        for _ in 0..count {
            out.push(C::read(r)?);
        }
        Ok(out)
    }
    fn write(w: &mut Writer, v: &Vec<C::Value>) {
        w.write_count(v.len());
        for item in v {
            C::write(w, item);
        }
    }
}

pub struct ArrayU32<C>(PhantomData<C>);
impl<C: Codec> Codec for ArrayU32<C> {
    type Value = Vec<C::Value>;
    fn read(r: &mut Reader<'_>) -> Result<Vec<C::Value>> {
        let count = r.read_u32_le()? as usize;
        if count > r.remaining() {
            return Err(crate::Error::LengthLimit {
                got: count,
                limit: r.remaining(),
            });
        }
        let mut out = Vec::with_capacity(count);
        for _ in 0..count {
            out.push(C::read(r)?);
        }
        Ok(out)
    }
    fn write(w: &mut Writer, v: &Vec<C::Value>) {
        w.write_u32_le(v.len() as u32);
        for item in v {
            C::write(w, item);
        }
    }
}

pub struct ArrayI32<C>(PhantomData<C>);
impl<C: Codec> Codec for ArrayI32<C> {
    type Value = Vec<C::Value>;
    fn read(r: &mut Reader<'_>) -> Result<Vec<C::Value>> {
        let raw = r.read_i32_le()?;
        if raw < 0 {
            return Err(crate::Error::Invalid("negative array count"));
        }
        let count = raw as usize;
        if count > r.remaining() {
            return Err(crate::Error::LengthLimit {
                got: count,
                limit: r.remaining(),
            });
        }
        let mut out = Vec::with_capacity(count);
        for _ in 0..count {
            out.push(C::read(r)?);
        }
        Ok(out)
    }
    fn write(w: &mut Writer, v: &Vec<C::Value>) {
        w.write_i32_le(v.len() as i32);
        for item in v {
            C::write(w, item);
        }
    }
}

pub struct Optional<C>(PhantomData<C>);
impl<C: Codec> Codec for Optional<C> {
    type Value = Option<C::Value>;
    fn read(r: &mut Reader<'_>) -> Result<Option<C::Value>> {
        if r.read_bool()? {
            Ok(Some(C::read(r)?))
        } else {
            Ok(None)
        }
    }
    fn write(w: &mut Writer, v: &Option<C::Value>) {
        match v {
            Some(inner) => {
                w.write_bool(true);
                C::write(w, inner);
            }
            None => w.write_bool(false),
        }
    }
}

pub struct Pair<C>(PhantomData<C>);
impl<C: Codec> Codec for Pair<C> {
    type Value = (C::Value, C::Value);
    fn read(r: &mut Reader<'_>) -> Result<(C::Value, C::Value)> {
        Ok((C::read(r)?, C::read(r)?))
    }
    fn write(w: &mut Writer, v: &(C::Value, C::Value)) {
        C::write(w, &v.0);
        C::write(w, &v.1);
    }
}
