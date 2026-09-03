use crate::types::enums::{command_origin_type as cot, label_from_value, value_from_label};
use crate::types::primitives::{Array, MceUuid, Uuid};
use crate::{Codec, Reader, Result, Writer};

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
