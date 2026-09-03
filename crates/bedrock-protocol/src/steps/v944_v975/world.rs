use super::movement::passthrough_optional;
use super::SOUND;
use bedrock_codec::prelude::*;

pub(crate) fn play_sound(w: &mut PacketWrapper, to_v975: bool) -> Result<()> {
    w.passthrough::<Str>()?;
    w.passthrough::<BlockPos>()?;
    w.passthrough::<FloatLe>()?;
    w.passthrough::<FloatLe>()?;
    if to_v975 {
        w.write::<Bool>(false);
    } else if w.read::<Bool>()? {
        w.read::<UInt64Le>()?;
    }
    Ok(())
}

pub(crate) fn actor_event(w: &mut PacketWrapper, to_v975: bool) -> Result<()> {
    w.passthrough::<UVarInt64>()?;
    w.passthrough::<Byte>()?;
    w.passthrough::<VarInt>()?;
    if to_v975 {
        w.write::<Bool>(false);
    } else if w.read::<Bool>()? {
        w.read::<Vec3>()?;
    }
    Ok(())
}

pub(crate) fn level_sound_event(w: &mut PacketWrapper, to_v975: bool) -> Result<()> {
    let event = w.read::<UVarInt>()?;
    let mapped = if to_v975 {
        SOUND.up(event)
    } else {
        SOUND.down(event)
    };
    w.write::<UVarInt>(mapped);

    w.passthrough::<Vec3>()?;
    w.passthrough::<VarInt>()?;
    w.passthrough::<Str>()?;
    w.passthrough::<Bool>()?;
    w.passthrough::<Bool>()?;
    w.passthrough::<Int64Le>()?;

    if to_v975 {
        w.write::<Bool>(false);
    } else if w.read::<Bool>()? {
        w.read::<Vec3>()?;
    }
    Ok(())
}

pub(crate) fn locator_bar(w: &mut PacketWrapper, to_v975: bool) -> Result<()> {
    w.passthrough_each(|w| {
        w.passthrough::<Uuid>()?;
        w.passthrough::<UIntLe>()?;
        passthrough_optional::<Bool>(w)?;
        if w.passthrough::<Bool>()? {
            w.passthrough::<Vec3>()?;
            w.passthrough::<VarInt>()?;
        }

        if to_v975 {
            if w.read::<Bool>()? {
                w.read::<UIntLe>()?;
            }
            w.write::<Bool>(false);
            w.write::<Bool>(false);
        } else {
            if w.read::<Bool>()? {
                w.read::<Str>()?;
            }
            if w.read::<Bool>()? {
                w.read::<Vec2>()?;
            }
            w.write::<Bool>(false);
        }

        passthrough_optional::<IntLe>(w)?;
        passthrough_optional::<Bool>(w)?;
        passthrough_optional::<VarInt64>(w)?;
        w.passthrough::<UVarInt>()?;
        Ok(())
    })?;
    Ok(())
}
