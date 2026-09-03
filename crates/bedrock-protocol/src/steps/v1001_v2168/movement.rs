use bedrock_codec::prelude::*;

pub(crate) fn move_delta_actor(w: &mut PacketWrapper, to_v2168: bool) -> Result<()> {
    w.passthrough::<UVarInt64>()?;

    if to_v2168 {
        let flags = w.reader().read_u16_le()?;
        let write_optional_f32 = |w: &mut PacketWrapper, present: bool| -> Result<()> {
            if present {
                let v = w.reader().read_f32_le()?;
                w.writer().write_bool(true);
                w.writer().write_f32_le(v);
            } else {
                w.writer().write_bool(false);
            }
            Ok(())
        };
        write_optional_f32(w, flags & MOVE_DELTA_HAS_X != 0)?;
        write_optional_f32(w, flags & MOVE_DELTA_HAS_Y != 0)?;
        write_optional_f32(w, flags & MOVE_DELTA_HAS_Z != 0)?;

        for bit in [
            MOVE_DELTA_HAS_ROT_X,
            MOVE_DELTA_HAS_ROT_Y,
            MOVE_DELTA_HAS_ROT_Z,
        ] {
            if flags & bit != 0 {
                let v = w.reader().read_u8()?;
                w.writer().write_bool(true);
                w.writer().write_u8(v);
            } else {
                w.writer().write_bool(false);
            }
        }

        w.writer().write_bool(flags & MOVE_DELTA_ON_GROUND != 0);
        w.writer().write_bool(flags & MOVE_DELTA_FORCE_MOVE != 0);
        w.writer().write_bool(false);
        w.writer().write_bool(false);
    } else {
        let mut flags = 0u16;
        let mut positions = Vec::new();
        for bit in [MOVE_DELTA_HAS_X, MOVE_DELTA_HAS_Y, MOVE_DELTA_HAS_Z] {
            if w.reader().read_bool()? {
                flags |= bit;
                positions.push(w.reader().read_f32_le()?);
            }
        }
        let mut rotations = Vec::new();
        for bit in [
            MOVE_DELTA_HAS_ROT_X,
            MOVE_DELTA_HAS_ROT_Y,
            MOVE_DELTA_HAS_ROT_Z,
        ] {
            if w.reader().read_bool()? {
                flags |= bit;
                rotations.push(w.reader().read_u8()?);
            }
        }
        if w.reader().read_bool()? {
            flags |= MOVE_DELTA_ON_GROUND;
        }
        if w.reader().read_bool()? {
            flags |= MOVE_DELTA_FORCE_MOVE;
        }
        w.reader().read_bool()?;
        w.reader().read_bool()?;

        w.writer().write_u16_le(flags);
        for v in positions {
            w.writer().write_f32_le(v);
        }
        for v in rotations {
            w.writer().write_u8(v);
        }
    }

    w.passthrough_all();
    Ok(())
}

pub(crate) fn move_player(w: &mut PacketWrapper, to_v2168: bool) -> Result<()> {
    w.passthrough::<UVarInt64>()?;
    w.passthrough::<Vec3>()?;
    w.passthrough::<FloatLe>()?;
    w.passthrough::<FloatLe>()?;
    w.passthrough::<FloatLe>()?;
    let mode = w.passthrough::<Byte>()?;
    w.passthrough::<Bool>()?;
    w.passthrough::<UVarInt64>()?;

    if to_v2168 {
        if mode == MOVE_MODE_TELEPORT {
            let cause = w.reader().read_i32_le()?;
            let source = w.reader().read_i32_le()?;
            w.writer().write_bool(true);
            w.writer().write_i32_le(cause);
            w.writer().write_i32_le(source);
        } else {
            w.writer().write_bool(false);
        }
    } else if w.reader().read_bool()? {
        let cause = w.reader().read_i32_le()?;
        let source = w.reader().read_i32_le()?;
        if mode == MOVE_MODE_TELEPORT {
            w.writer().write_i32_le(cause);
            w.writer().write_i32_le(source);
        }
    } else if mode == MOVE_MODE_TELEPORT {
        w.writer().write_i32_le(0);
        w.writer().write_i32_le(0);
    }

    w.passthrough_all();
    Ok(())
}

pub(crate) fn read_double_optional(w: &mut PacketWrapper) -> Result<bool> {
    if !w.reader().read_bool()? {
        return Ok(false);
    }
    w.reader().read_bool()
}

pub(crate) const MOVE_DELTA_HAS_X: u16 = 1 << 0;
pub(crate) const MOVE_DELTA_HAS_Y: u16 = 1 << 1;
pub(crate) const MOVE_DELTA_HAS_Z: u16 = 1 << 2;
pub(crate) const MOVE_DELTA_HAS_ROT_X: u16 = 1 << 3;
pub(crate) const MOVE_DELTA_HAS_ROT_Y: u16 = 1 << 4;
pub(crate) const MOVE_DELTA_HAS_ROT_Z: u16 = 1 << 5;
pub(crate) const MOVE_DELTA_ON_GROUND: u16 = 1 << 6;
pub(crate) const MOVE_DELTA_TELEPORT: u16 = 1 << 7;
pub(crate) const MOVE_DELTA_FORCE_MOVE: u16 = 1 << 8;
pub(crate) const MOVE_MODE_TELEPORT: u8 = 2;

#[allow(dead_code)]
const MOVE_DELTA_TELEPORT_UNUSED: u16 = MOVE_DELTA_TELEPORT;
