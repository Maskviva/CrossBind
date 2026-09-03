use super::movement::passthrough_optional;
use bedrock_codec::prelude::*;

pub(crate) fn start_game(w: &mut PacketWrapper) -> Result<()> {
    w.passthrough::<VarInt64>()?;
    w.passthrough::<UVarInt64>()?;
    w.passthrough::<VarInt>()?;
    w.passthrough::<Vec3>()?;
    w.passthrough::<Vec2>()?;

    w.passthrough::<LevelSettingsV944>()?;

    w.passthrough::<Str>()?;
    w.passthrough::<Str>()?;
    w.passthrough::<Str>()?;
    w.passthrough::<Bool>()?;
    w.passthrough::<VarInt>()?;
    w.passthrough::<Bool>()?;
    w.passthrough::<Int64Le>()?;
    w.passthrough::<VarInt>()?;

    w.passthrough_each(|w| {
        w.passthrough::<Str>()?;
        w.passthrough::<NamedCompoundTag>()?;
        Ok(())
    })?;

    w.passthrough::<Str>()?;
    w.passthrough::<Bool>()?;
    w.passthrough::<Str>()?;
    w.passthrough::<NamedCompoundTag>()?;

    w.read::<Int64Le>()?;
    w.write::<Int64Le>(0);

    Ok(())
}

pub(crate) fn party_changed(w: &mut PacketWrapper, to_v975: bool) -> Result<()> {
    if w.passthrough::<Bool>()? {
        w.passthrough::<Str>()?;
        if to_v975 {
            w.write::<Bool>(false);
        } else {
            w.read::<Bool>()?;
        }
    }
    Ok(())
}

pub(crate) fn update_client_options(w: &mut PacketWrapper, to_v975: bool) -> Result<()> {
    passthrough_optional::<Byte>(w)?;
    if to_v975 {
        w.write::<Bool>(false);
    } else if w.read::<Bool>()? {
        w.read::<Bool>()?;
    }
    Ok(())
}
