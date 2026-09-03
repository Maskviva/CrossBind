use crate::types::primitives::Array;
use crate::{Codec, Reader, Result, Writer};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SoftEnum {
    pub name: String,
    pub values: Vec<String>,
}

pub struct SoftEnumCodec;

impl Codec for SoftEnumCodec {
    type Value = SoftEnum;
    fn read(r: &mut Reader<'_>) -> Result<SoftEnum> {
        Ok(SoftEnum {
            name: r.read_string()?,
            values: Array::<crate::types::primitives::Str>::read(r)?,
        })
    }
    fn write(w: &mut Writer, v: &SoftEnum) {
        w.write_string(&v.name);
        Array::<crate::types::primitives::Str>::write(w, &v.values);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandConstraint {
    pub value_index: i32,
    pub enum_index: i32,
    pub constraints: Vec<u8>,
}

pub struct CommandConstraintCodec;

impl Codec for CommandConstraintCodec {
    type Value = CommandConstraint;
    fn read(r: &mut Reader<'_>) -> Result<CommandConstraint> {
        Ok(CommandConstraint {
            value_index: r.read_i32_le()?,
            enum_index: r.read_i32_le()?,
            constraints: Array::<crate::types::primitives::Byte>::read(r)?,
        })
    }
    fn write(w: &mut Writer, v: &CommandConstraint) {
        w.write_i32_le(v.value_index);
        w.write_i32_le(v.enum_index);
        Array::<crate::types::primitives::Byte>::write(w, &v.constraints);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandEnum {
    pub name: String,
    pub values: Vec<u32>,
}

pub struct CommandEnumV898;

impl Codec for CommandEnumV898 {
    type Value = CommandEnum;
    fn read(r: &mut Reader<'_>) -> Result<CommandEnum> {
        let name = r.read_string()?;
        let count = r.read_count()?;
        let mut values = Vec::with_capacity(count);
        for _ in 0..count {
            values.push(r.read_i32_le()? as u32);
        }
        Ok(CommandEnum { name, values })
    }
    fn write(w: &mut Writer, v: &CommandEnum) {
        w.write_string(&v.name);
        w.write_count(v.values.len());
        for value in &v.values {
            w.write_i32_le(*value as i32);
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnumIndexWidth {
    U8,
    U16,
    U32,
}

impl EnumIndexWidth {
    pub fn for_table_size(values_size: usize) -> EnumIndexWidth {
        if values_size <= 0x100 {
            EnumIndexWidth::U8
        } else if values_size <= 0x10000 {
            EnumIndexWidth::U16
        } else {
            EnumIndexWidth::U32
        }
    }

    fn read(self, r: &mut Reader<'_>) -> Result<u32> {
        Ok(match self {
            EnumIndexWidth::U8 => r.read_u8()? as u32,
            EnumIndexWidth::U16 => r.read_u16_le()? as u32,
            EnumIndexWidth::U32 => r.read_u32_le()?,
        })
    }

    fn write(self, w: &mut Writer, value: u32) {
        match self {
            EnumIndexWidth::U8 => w.write_u8(value as u8),
            EnumIndexWidth::U16 => w.write_u16_le(value as u16),
            EnumIndexWidth::U32 => w.write_u32_le(value),
        }
    }
}

pub fn read_command_enum_v860(r: &mut Reader<'_>, width: EnumIndexWidth) -> Result<CommandEnum> {
    let name = r.read_string()?;
    let count = r.read_count()?;
    let mut values = Vec::with_capacity(count);
    for _ in 0..count {
        values.push(width.read(r)?);
    }
    Ok(CommandEnum { name, values })
}

pub fn write_command_enum_v860(w: &mut Writer, v: &CommandEnum, width: EnumIndexWidth) {
    w.write_string(&v.name);
    w.write_count(v.values.len());
    for value in &v.values {
        width.write(w, *value);
    }
}
