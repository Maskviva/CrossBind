use super::inventory::convert_legacy_item;
use bedrock_codec::prelude::*;

fn actor_data(w: &mut PacketWrapper, to_v2168: bool) -> Result<()> {
    if to_v2168 {
        w.map::<ActorDataList, ActorDataListV2168>()?;
    } else {
        w.map::<ActorDataListV2168, ActorDataList>()?;
    }
    Ok(())
}

pub(crate) fn set_actor_data(w: &mut PacketWrapper, to_v2168: bool) -> Result<()> {
    w.passthrough::<UVarInt64>()?;
    actor_data(w, to_v2168)?;
    w.passthrough_all();
    Ok(())
}

pub(crate) fn add_actor(w: &mut PacketWrapper, to_v2168: bool) -> Result<()> {
    w.passthrough::<VarInt64>()?;
    w.passthrough::<UVarInt64>()?;
    w.passthrough::<Str>()?;
    w.passthrough::<Vec3>()?;
    w.passthrough::<Vec3>()?;
    for _ in 0..4 {
        w.passthrough::<FloatLe>()?;
    }
    w.passthrough_each(|w| {
        w.passthrough::<Str>()?;
        w.passthrough::<FloatLe>()?;
        w.passthrough::<FloatLe>()?;
        w.passthrough::<FloatLe>()?;
        Ok(())
    })?;
    actor_data(w, to_v2168)?;
    w.passthrough_all();
    Ok(())
}

pub(crate) fn add_item_actor(w: &mut PacketWrapper, to_v2168: bool) -> Result<()> {
    w.passthrough::<VarInt64>()?;
    w.passthrough::<UVarInt64>()?;
    convert_legacy_item(w, to_v2168)?;
    w.passthrough::<Vec3>()?;
    w.passthrough::<Vec3>()?;
    actor_data(w, to_v2168)?;
    w.passthrough_all();
    Ok(())
}

pub(crate) fn add_player(w: &mut PacketWrapper, to_v2168: bool) -> Result<()> {
    w.passthrough::<Uuid>()?;
    w.passthrough::<Str>()?;
    w.passthrough::<UVarInt64>()?;
    w.passthrough::<Str>()?;
    w.passthrough::<Vec3>()?;
    w.passthrough::<Vec3>()?;
    for _ in 0..3 {
        w.passthrough::<FloatLe>()?;
    }
    convert_legacy_item(w, to_v2168)?;
    w.passthrough::<VarInt>()?;
    actor_data(w, to_v2168)?;
    w.passthrough_all();
    Ok(())
}
