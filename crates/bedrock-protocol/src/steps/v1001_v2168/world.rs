use crate::connection::ConnState;
use bedrock_codec::prelude::*;

pub(crate) fn play_sound(w: &mut PacketWrapper, to_v2168: bool) -> Result<()> {
    w.passthrough::<Str>()?;
    w.passthrough::<BlockPos>()?;
    w.passthrough::<FloatLe>()?;
    w.passthrough::<FloatLe>()?;
    if to_v2168 {
        w.writer().write_varint(play_sound_loop_count());
    } else {
        w.reader().read_varint()?;
    }
    w.passthrough_all();
    Ok(())
}

fn play_sound_loop_count() -> i32 {
    static COUNT: std::sync::OnceLock<i32> = std::sync::OnceLock::new();
    *COUNT.get_or_init(|| {
        std::env::var("CROSSBIND_PLAYSOUND_LOOPS")
            .ok()
            .and_then(|raw| raw.trim().parse::<i32>().ok())
            .unwrap_or(PLAY_SOUND_ONCE)
    })
}

pub fn play_sound_loop_label() -> String {
    let count = play_sound_loop_count();
    if count < 0 {
        format!("{count} (negative: every sound repeats forever)")
    } else {
        format!("{count}")
    }
}

pub(crate) fn structure_block_update(
    w: &mut PacketWrapper,
    state: &mut ConnState,
    to_v2168: bool,
) -> Result<()> {
    let body = w.reader().read_remaining();
    let Some((head, save_mode, trigger, waterlogged)) = split_structure_tail(body, to_v2168) else {
        state.notices.push(format!(
            "StructureBlockUpdate tail is not the expected \
             [RedstoneSaveMode][ShouldTrigger][Waterlogged]; dropping it \
             ({} B, last three bytes {:02x?})",
            body.len(),
            &body[body.len().saturating_sub(3)..],
        ));
        w.cancel();
        return Ok(());
    };

    w.writer().write_bytes(head);
    if to_v2168 {
        w.writer().write_u8(save_mode);
    } else {
        w.writer().write_varint(save_mode as i32);
    }
    w.writer().write_bool(trigger);
    w.writer().write_bool(waterlogged);
    Ok(())
}

fn split_structure_tail(body: &[u8], to_v2168: bool) -> Option<(&[u8], u8, bool, bool)> {
    if body.len() < 3 {
        return None;
    }
    let (head, tail) = body.split_at(body.len() - 3);
    let (raw_mode, trigger, waterlogged) = (tail[0], tail[1], tail[2]);
    if trigger > 1 || waterlogged > 1 {
        return None;
    }
    let save_mode = if to_v2168 {
        if raw_mode & 0x81 != 0 {
            return None;
        }
        raw_mode >> 1
    } else {
        raw_mode
    };
    if save_mode > STRUCTURE_REDSTONE_SAVE_MODE_MAX {
        return None;
    }
    Some((head, save_mode, trigger == 1, waterlogged == 1))
}

pub(crate) fn anvil_damage(w: &mut PacketWrapper, to_v2168: bool) -> Result<()> {
    if to_v2168 {
        w.read::<Byte>()?;
    } else {
        w.write::<Byte>(0);
    }
    w.passthrough_all();
    Ok(())
}

pub(crate) fn serverbound_diagnostics(w: &mut PacketWrapper, to_v2168: bool) -> Result<()> {
    for _ in 0..9 {
        w.passthrough::<FloatLe>()?;
    }
    let mem_count = w.passthrough::<UVarInt>()?;
    for _ in 0..mem_count {
        w.passthrough::<Byte>()?;
        w.passthrough::<Int64Le>()?;
    }
    let entity_count = w.passthrough::<UVarInt>()?;
    for _ in 0..entity_count {
        w.passthrough::<Str>()?;
        w.passthrough::<Str>()?;
        w.passthrough::<Int64Le>()?;
        w.passthrough::<Byte>()?;
    }
    let system_count = w.passthrough::<UVarInt>()?;
    for _ in 0..system_count {
        w.passthrough::<Str>()?;
        w.passthrough::<Int64Le>()?;
        w.passthrough::<Int64Le>()?;
        w.passthrough::<Byte>()?;
    }

    if to_v2168 {
        w.writer().write_count(0);
    } else if w.reader().read_count()? != 0 {
        w.cancel();
        return Ok(());
    }

    w.passthrough_all();
    Ok(())
}

pub(crate) const PLAY_SOUND_ONCE: i32 = 0;
pub(crate) const STRUCTURE_REDSTONE_SAVE_MODE_MAX: u8 = 3;
