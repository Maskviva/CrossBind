use bedrock_codec::prelude::*;

pub(crate) fn client_movement_prediction_sync(w: &mut PacketWrapper, to_v975: bool) -> Result<()> {
    passthrough_bitset(w)?;
    for _ in 0..9 {
        w.passthrough::<FloatLe>()?;
    }
    if to_v975 {
        for _ in 0..3 {
            w.write::<FloatLe>(0.0);
        }
    } else {
        for _ in 0..3 {
            w.read::<FloatLe>()?;
        }
    }
    w.passthrough::<VarInt64>()?;
    w.passthrough::<Bool>()?;
    Ok(())
}

fn passthrough_bitset(w: &mut PacketWrapper) -> Result<()> {
    loop {
        if (w.passthrough::<Byte>()? & 0x80) == 0 {
            return Ok(());
        }
    }
}

pub(crate) fn byte_width(w: &mut PacketWrapper, to_v975: bool) -> Result<()> {
    if to_v975 {
        let v = w.read::<Byte>()?;
        w.write::<UVarInt>(u32::from(v));
    } else {
        let v = w.read::<UVarInt>()?;
        w.write::<Byte>(v.min(0xFF) as u8);
    }
    Ok(())
}

pub(crate) fn passthrough_optional<C: Codec>(w: &mut PacketWrapper) -> Result<bool> {
    let present = w.passthrough::<Bool>()?;
    if present {
        w.passthrough::<C>()?;
    }
    Ok(present)
}
