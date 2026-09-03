use super::{
    BOSS_APPEARANCE_PROPERTIES, BOSS_HEALTH_PERCENTAGE, BOSS_HIDE, BOSS_REGISTER_PLAYER,
    BOSS_REQUEST, BOSS_SHOW, BOSS_TEXTURE, BOSS_TITLE, BOSS_UNREGISTER_PLAYER,
};
use crate::sound_events::{id_to_name, name_to_id};
use bedrock_codec::prelude::*;

pub(crate) fn level_sound_event(w: &mut PacketWrapper, to_v1001: bool) -> Result<()> {
    if to_v1001 {
        let id = w.read::<UVarInt>()?;
        let name = id_to_name(id).unwrap_or("");
        w.write::<Str>(name.to_owned());
        w.passthrough_all();
    } else {
        let name = w.read::<Str>()?;
        let id = name_to_id(&name).unwrap_or(0);
        w.write::<UVarInt>(id);
        w.passthrough_all();
    }
    Ok(())
}

pub(crate) fn boss_event(w: &mut PacketWrapper, to_v1001: bool) -> Result<()> {
    if to_v1001 {
        let boss_id = w.read::<VarInt64>()?;
        let event_type = w.read::<UVarInt>()?;

        let mut player_id: i64 = 0;
        let mut title = String::new();
        let mut filtered_title = String::new();
        let mut health: f32 = 0.0;
        let mut colour: u32 = 0;
        let mut overlay: u32 = 0;

        match event_type {
            BOSS_SHOW => {
                title = w.read::<Str>()?;
                filtered_title = w.read::<Str>()?;
                health = w.read::<FloatLe>()?;
                w.read::<UShortLe>()?;
                colour = w.read::<UVarInt>()?;
                overlay = w.read::<UVarInt>()?;
            }
            BOSS_REGISTER_PLAYER | BOSS_UNREGISTER_PLAYER | BOSS_REQUEST => {
                player_id = w.read::<VarInt64>()?;
            }
            BOSS_HIDE => {}
            BOSS_HEALTH_PERCENTAGE => {
                health = w.read::<FloatLe>()?;
            }
            BOSS_TITLE => {
                title = w.read::<Str>()?;
                filtered_title = w.read::<Str>()?;
            }
            BOSS_APPEARANCE_PROPERTIES => {
                w.read::<UShortLe>()?;
                colour = w.read::<UVarInt>()?;
                overlay = w.read::<UVarInt>()?;
            }
            BOSS_TEXTURE => {
                colour = w.read::<UVarInt>()?;
                overlay = w.read::<UVarInt>()?;
            }
            _ => {}
        }
        if w.has_remaining() {
            return Err(Error::Invalid("BossEvent v975->v1001 decode left bytes"));
        }

        w.write::<VarInt64>(boss_id);
        w.write::<VarInt64>(player_id);
        w.write::<Byte>(event_type as u8);
        w.write::<Str>(title);
        w.write::<Str>(filtered_title);
        w.write::<FloatLe>(health);
        w.write::<Byte>(colour_to_v1001(colour));
        w.write::<Byte>((overlay & 0xFF) as u8);
    } else {
        let boss_id = w.read::<VarInt64>()?;
        let player_id = w.read::<VarInt64>()?;
        let event_type = w.read::<Byte>()? as u32;
        let title = w.read::<Str>()?;
        let filtered_title = w.read::<Str>()?;
        let health = w.read::<FloatLe>()?;
        let colour = w.read::<Byte>()? as u32;
        let overlay = w.read::<Byte>()? as u32;
        if w.has_remaining() {
            return Err(Error::Invalid("BossEvent v1001->v975 decode left bytes"));
        }

        w.write::<VarInt64>(boss_id);
        w.write::<UVarInt>(event_type);
        match event_type {
            BOSS_SHOW => {
                w.write::<Str>(title);
                w.write::<Str>(filtered_title);
                w.write::<FloatLe>(health);
                w.write::<UShortLe>(0);
                w.write::<UVarInt>(colour_to_v975(colour) as u32);
                w.write::<UVarInt>(overlay);
            }
            BOSS_REGISTER_PLAYER | BOSS_UNREGISTER_PLAYER | BOSS_REQUEST => {
                w.write::<VarInt64>(player_id);
            }
            BOSS_HIDE => {}
            BOSS_HEALTH_PERCENTAGE => {
                w.write::<FloatLe>(health);
            }
            BOSS_TITLE => {
                w.write::<Str>(title);
                w.write::<Str>(filtered_title);
            }
            BOSS_APPEARANCE_PROPERTIES => {
                w.write::<UShortLe>(0);
                w.write::<UVarInt>(colour_to_v975(colour) as u32);
                w.write::<UVarInt>(overlay);
            }
            BOSS_TEXTURE => {
                w.write::<UVarInt>(colour_to_v975(colour) as u32);
                w.write::<UVarInt>(overlay);
            }
            _ => {}
        }
    }
    Ok(())
}

fn colour_to_v1001(colour: u32) -> u8 {
    match colour {
        6 => 7,
        other => (other & 0xFF) as u8,
    }
}

fn colour_to_v975(colour: u32) -> u8 {
    match colour {
        7 => 6,
        other => (other & 0xFF) as u8,
    }
}
