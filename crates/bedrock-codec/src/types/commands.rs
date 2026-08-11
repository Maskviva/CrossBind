use crate::types::enums::{
    command_origin_type as cot, command_permission_level as cpl, label_from_value,
    value_from_label,
};
use crate::types::primitives::{Array, MceUuid, Uuid};
use crate::{Codec, Reader, Result, Writer};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandParameter {
    pub name: String,
    pub symbol: i32,
    pub optional: bool,
    pub options: u8,
}

pub struct CommandParameterCodec;
impl Codec for CommandParameterCodec {
    type Value = CommandParameter;
    fn read(r: &mut Reader<'_>) -> Result<CommandParameter> {
        Ok(CommandParameter {
            name: r.read_string()?,
            symbol: r.read_i32_le()?,
            optional: r.read_bool()?,
            options: r.read_u8()?,
        })
    }
    fn write(w: &mut Writer, v: &CommandParameter) {
        w.write_string(&v.name);
        w.write_i32_le(v.symbol);
        w.write_bool(v.optional);
        w.write_u8(v.options);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandOverload {
    pub chaining: bool,
    pub parameters: Vec<CommandParameter>,
}

pub struct CommandOverloadCodec;
impl Codec for CommandOverloadCodec {
    type Value = CommandOverload;
    fn read(r: &mut Reader<'_>) -> Result<CommandOverload> {
        let chaining = r.read_bool()?;
        let parameters = Array::<CommandParameterCodec>::read(r)?;
        Ok(CommandOverload {
            chaining,
            parameters,
        })
    }
    fn write(w: &mut Writer, v: &CommandOverload) {
        w.write_bool(v.chaining);
        Array::<CommandParameterCodec>::write(w, &v.parameters);
    }
}

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandSubcommand {
    pub name: String,
    pub values: Vec<(u32, u32)>,
}

pub struct CommandSubcommandV860;
impl Codec for CommandSubcommandV860 {
    type Value = CommandSubcommand;
    fn read(r: &mut Reader<'_>) -> Result<CommandSubcommand> {
        let name = r.read_string()?;
        let count = r.read_count()?;
        let mut values = Vec::with_capacity(count);
        for _ in 0..count {
            values.push((r.read_u16_le()? as u32, r.read_u16_le()? as u32));
        }
        Ok(CommandSubcommand { name, values })
    }
    fn write(w: &mut Writer, v: &CommandSubcommand) {
        w.write_string(&v.name);
        w.write_count(v.values.len());
        for (a, b) in &v.values {
            w.write_u16_le(*a as u16);
            w.write_u16_le(*b as u16);
        }
    }
}

pub struct CommandSubcommandV898;
impl Codec for CommandSubcommandV898 {
    type Value = CommandSubcommand;
    fn read(r: &mut Reader<'_>) -> Result<CommandSubcommand> {
        let name = r.read_string()?;
        let count = r.read_count()?;
        let mut values = Vec::with_capacity(count);
        for _ in 0..count {
            values.push((r.read_uvarint()?, r.read_uvarint()?));
        }
        Ok(CommandSubcommand { name, values })
    }
    fn write(w: &mut Writer, v: &CommandSubcommand) {
        w.write_string(&v.name);
        w.write_count(v.values.len());
        for (a, b) in &v.values {
            w.write_uvarint(*a);
            w.write_uvarint(*b);
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandDefinition {
    pub name: String,
    pub description: String,
    pub flags: u16,
    pub permission: u32,
    pub alias_index: i32,
    pub subcommand_indices: Vec<u32>,
    pub overloads: Vec<CommandOverload>,
}

pub struct CommandDefinitionV860;
impl Codec for CommandDefinitionV860 {
    type Value = CommandDefinition;
    fn read(r: &mut Reader<'_>) -> Result<CommandDefinition> {
        let name = r.read_string()?;
        let description = r.read_string()?;
        let flags = r.read_u16_le()?;
        let permission = r.read_u8()? as u32;
        let alias_index = r.read_i32_le()?;
        let sub_count = r.read_count()?;
        let mut subcommand_indices = Vec::with_capacity(sub_count);
        for _ in 0..sub_count {
            subcommand_indices.push(r.read_u16_le()? as u32);
        }
        let overloads = Array::<CommandOverloadCodec>::read(r)?;
        Ok(CommandDefinition {
            name,
            description,
            flags,
            permission,
            alias_index,
            subcommand_indices,
            overloads,
        })
    }
    fn write(w: &mut Writer, v: &CommandDefinition) {
        w.write_string(&v.name);
        w.write_string(&v.description);
        w.write_u16_le(v.flags);
        w.write_u8(v.permission as u8);
        w.write_i32_le(v.alias_index);
        w.write_count(v.subcommand_indices.len());
        for index in &v.subcommand_indices {
            w.write_u16_le(*index as u16);
        }
        Array::<CommandOverloadCodec>::write(w, &v.overloads);
    }
}

pub struct CommandDefinitionV898;
impl Codec for CommandDefinitionV898 {
    type Value = CommandDefinition;
    fn read(r: &mut Reader<'_>) -> Result<CommandDefinition> {
        let name = r.read_string()?;
        let description = r.read_string()?;
        let flags = r.read_u16_le()?;
        let permission = value_from_label(&cpl::LABELS, &r.read_string()?, cpl::ANY);
        let alias_index = r.read_i32_le()?;
        let sub_count = r.read_count()?;
        let mut subcommand_indices = Vec::with_capacity(sub_count);
        for _ in 0..sub_count {
            subcommand_indices.push(r.read_i32_le()? as u32);
        }
        let overloads = Array::<CommandOverloadCodec>::read(r)?;
        Ok(CommandDefinition {
            name,
            description,
            flags,
            permission,
            alias_index,
            subcommand_indices,
            overloads,
        })
    }
    fn write(w: &mut Writer, v: &CommandDefinition) {
        w.write_string(&v.name);
        w.write_string(&v.description);
        w.write_u16_le(v.flags);
        w.write_string(label_from_value(&cpl::LABELS, v.permission));
        w.write_i32_le(v.alias_index);
        w.write_count(v.subcommand_indices.len());
        for index in &v.subcommand_indices {
            w.write_i32_le(*index as i32);
        }
        Array::<CommandOverloadCodec>::write(w, &v.overloads);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandOrigin {
    pub origin_type: u32,
    pub uuid: MceUuid,
    pub request_id: String,
    pub player_id: i64,
}

pub struct CommandOriginV860;
impl Codec for CommandOriginV860 {
    type Value = CommandOrigin;
    fn read(r: &mut Reader<'_>) -> Result<CommandOrigin> {
        let origin_type = r.read_uvarint()?;
        let uuid = Uuid::read(r)?;
        let request_id = r.read_string()?;
        let player_id = if origin_type == cot::TEST || origin_type == cot::AUTOMATION_PLAYER {
            r.read_varint64()?
        } else {
            -1
        };
        Ok(CommandOrigin {
            origin_type,
            uuid,
            request_id,
            player_id,
        })
    }
    fn write(w: &mut Writer, v: &CommandOrigin) {
        w.write_uvarint(v.origin_type);
        Uuid::write(w, &v.uuid);
        w.write_string(&v.request_id);
        if v.origin_type == cot::TEST || v.origin_type == cot::AUTOMATION_PLAYER {
            w.write_varint64(v.player_id);
        }
    }
}

pub struct CommandOriginV898;
impl Codec for CommandOriginV898 {
    type Value = CommandOrigin;
    fn read(r: &mut Reader<'_>) -> Result<CommandOrigin> {
        let origin_type = value_from_label(&cot::LABELS, &r.read_string()?, cot::PLAYER);
        let uuid = Uuid::read(r)?;
        let request_id = r.read_string()?;
        let player_id = r.read_i64_le()?;
        Ok(CommandOrigin {
            origin_type,
            uuid,
            request_id,
            player_id,
        })
    }
    fn write(w: &mut Writer, v: &CommandOrigin) {
        w.write_string(label_from_value(&cot::LABELS, v.origin_type));
        Uuid::write(w, &v.uuid);
        w.write_string(&v.request_id);
        w.write_i64_le(v.player_id);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandOutputMessage {
    pub message_id: String,
    pub internal: bool,
    pub parameters: Vec<String>,
}

pub struct CommandOutputMessageV860;
impl Codec for CommandOutputMessageV860 {
    type Value = CommandOutputMessage;
    fn read(r: &mut Reader<'_>) -> Result<CommandOutputMessage> {
        let internal = r.read_bool()?;
        let message_id = r.read_string()?;
        let parameters = Array::<crate::types::primitives::Str>::read(r)?;
        Ok(CommandOutputMessage {
            message_id,
            internal,
            parameters,
        })
    }
    fn write(w: &mut Writer, v: &CommandOutputMessage) {
        w.write_bool(v.internal);
        w.write_string(&v.message_id);
        Array::<crate::types::primitives::Str>::write(w, &v.parameters);
    }
}

pub struct CommandOutputMessageV898;
impl Codec for CommandOutputMessageV898 {
    type Value = CommandOutputMessage;
    fn read(r: &mut Reader<'_>) -> Result<CommandOutputMessage> {
        let message_id = r.read_string()?;
        let internal = r.read_bool()?;
        let parameters = Array::<crate::types::primitives::Str>::read(r)?;
        Ok(CommandOutputMessage {
            message_id,
            internal,
            parameters,
        })
    }
    fn write(w: &mut Writer, v: &CommandOutputMessage) {
        w.write_string(&v.message_id);
        w.write_bool(v.internal);
        Array::<crate::types::primitives::Str>::write(w, &v.parameters);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn index_width_thresholds() {
        assert_eq!(EnumIndexWidth::for_table_size(0), EnumIndexWidth::U8);
        assert_eq!(EnumIndexWidth::for_table_size(256), EnumIndexWidth::U8);
        assert_eq!(EnumIndexWidth::for_table_size(257), EnumIndexWidth::U16);
        assert_eq!(EnumIndexWidth::for_table_size(65536), EnumIndexWidth::U16);
        assert_eq!(EnumIndexWidth::for_table_size(65537), EnumIndexWidth::U32);
    }

    #[test]
    fn permission_survives_the_label_round_trip() {
        let def = CommandDefinition {
            name: "give".into(),
            description: "".into(),
            flags: 0,
            permission: 2,
            alias_index: -1,
            subcommand_indices: vec![],
            overloads: vec![],
        };
        let mut w = Writer::new();
        CommandDefinitionV898::write(&mut w, &def);
        let bytes = w.into_vec();
        let mut r = Reader::new(&bytes);
        assert_eq!(CommandDefinitionV898::read(&mut r).unwrap(), def);
    }

    #[test]
    fn unknown_permission_label_falls_back_to_any() {
        let mut w = Writer::new();
        w.write_string("give");
        w.write_string("");
        w.write_u16_le(0);
        w.write_string("someFutureLevel");
        w.write_i32_le(-1);
        w.write_count(0);
        w.write_count(0);
        let bytes = w.into_vec();
        let mut r = Reader::new(&bytes);
        let def = CommandDefinitionV898::read(&mut r).unwrap();
        assert_eq!(def.permission, cpl::ANY);
    }

    #[test]
    fn origin_encodings_differ_and_both_round_trip() {
        let origin = CommandOrigin {
            origin_type: cot::PLAYER,
            uuid: MceUuid { msb: 1, lsb: 2 },
            request_id: "r".into(),
            player_id: -1,
        };

        let mut a = Writer::new();
        CommandOriginV860::write(&mut a, &origin);
        assert_eq!(a.len(), 19);

        let mut b = Writer::new();
        CommandOriginV898::write(&mut b, &origin);
        assert_eq!(b.len(), 33);

        let a_bytes = a.into_vec();
        let mut ra = Reader::new(&a_bytes);
        assert_eq!(CommandOriginV860::read(&mut ra).unwrap(), origin);

        let b_bytes = b.into_vec();
        let mut rb = Reader::new(&b_bytes);
        assert_eq!(CommandOriginV898::read(&mut rb).unwrap(), origin);
    }

    #[test]
    fn v860_origin_carries_a_player_id_for_test_origins() {
        let origin = CommandOrigin {
            origin_type: cot::TEST,
            uuid: MceUuid::default(),
            request_id: String::new(),
            player_id: 7,
        };
        let mut w = Writer::new();
        CommandOriginV860::write(&mut w, &origin);
        let bytes = w.into_vec();
        let mut r = Reader::new(&bytes);
        assert_eq!(CommandOriginV860::read(&mut r).unwrap(), origin);
        assert_eq!(r.remaining(), 0);
    }
}
