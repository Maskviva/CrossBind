use super::parameter::{CommandOverload, CommandOverloadCodec};
use crate::types::enums::command_origin_type::LABELS;
use crate::types::enums::command_permission_level::ANY;
use crate::types::enums::{label_from_value, value_from_label};
use crate::types::primitives::Array;
use crate::{Codec, Reader, Result, Writer};
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
        let permission = value_from_label(&LABELS, &r.read_string()?, ANY);
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
        w.write_string(label_from_value(&LABELS, v.permission));
        w.write_i32_le(v.alias_index);
        w.write_count(v.subcommand_indices.len());
        for index in &v.subcommand_indices {
            w.write_i32_le(*index as i32);
        }
        Array::<CommandOverloadCodec>::write(w, &v.overloads);
    }
}

// #[derive(Debug, Clone, PartialEq, Eq)]
