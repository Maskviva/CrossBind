use bedrock_codec::prelude::*;

pub(crate) fn start_game(w: &mut PacketWrapper, to_v2168: bool) -> Result<()> {
    w.passthrough::<VarInt64>()?;
    w.passthrough::<UVarInt64>()?;
    w.passthrough::<VarInt>()?;
    w.passthrough::<Vec3>()?;
    w.passthrough::<Vec2>()?;

    if to_v2168 {
        w.map::<LevelSettingsV944, LevelSettingsV2168>()?;
    } else {
        w.map::<LevelSettingsV2168, LevelSettingsV944>()?;
    }

    w.passthrough::<VarInt>()?;
    w.passthrough::<Bool>()?;
    w.passthrough::<Str>()?;
    w.passthrough::<Str>()?;
    w.passthrough::<Str>()?;
    w.passthrough::<Bool>()?;
    w.passthrough::<VarInt>()?;
    w.passthrough::<Bool>()?;
    w.passthrough::<Int64Le>()?;
    w.passthrough::<VarInt>()?;

    let block_prop_count = w.passthrough::<UVarInt>()?;
    for _ in 0..block_prop_count {
        w.passthrough::<Str>()?;
        w.passthrough::<NamedCompoundTag>()?;
    }

    w.passthrough::<Str>()?;
    w.passthrough::<Bool>()?;
    w.passthrough::<Str>()?;
    w.passthrough::<NamedCompoundTag>()?;

    w.read::<Int64Le>()?;
    w.write::<Int64Le>(0);

    w.passthrough::<Uuid>()?;
    w.passthrough::<Bool>()?;
    w.passthrough::<Bool>()?;
    w.passthrough::<Bool>()?;

    if to_v2168 {
        w.read::<Bool>()?;
    } else {
        w.write::<Bool>(false);
    }

    server_join_information(w, to_v2168)?;

    w.passthrough_all();
    Ok(())
}

pub(crate) fn server_join_information(w: &mut PacketWrapper, to_v2168: bool) -> Result<()> {
    if !w.passthrough::<Bool>()? {
        return Ok(());
    }

    if w.passthrough::<Bool>()? {
        w.passthrough::<Uuid>()?;
        w.passthrough::<Str>()?;
        if to_v2168 {
            let world_id = w.read::<Uuid>()?;
            w.write::<Optional<Uuid>>(Some(world_id));
            let world_name = w.read::<Str>()?;
            w.write::<Optional<Str>>(Some(world_name));
            w.passthrough::<Str>()?;
            let target = w.read::<Uuid>()?;
            w.write::<Optional<Uuid>>(Some(target));
            let scenario = w.read::<Str>()?;
            w.write::<Optional<Str>>(Some(scenario));
            let server = w.read::<Str>()?;
            w.write::<Optional<Str>>(Some(server));
        } else {
            let world_id = w.read::<Optional<Uuid>>()?;
            w.write::<Uuid>(world_id.unwrap_or_default());
            let world_name = w.read::<Optional<Str>>()?;
            w.write::<Str>(world_name.unwrap_or_default());
            w.passthrough::<Str>()?;
            let target = w.read::<Optional<Uuid>>()?;
            w.write::<Uuid>(target.unwrap_or_default());
            let scenario = w.read::<Optional<Str>>()?;
            w.write::<Str>(scenario.unwrap_or_default());
            let server = w.read::<Optional<Str>>()?;
            w.write::<Str>(server.unwrap_or_default());
        }
    }

    if w.passthrough::<Bool>()? {
        w.passthrough::<Str>()?;
        w.passthrough::<Str>()?;
    }

    if w.passthrough::<Bool>()? {
        if to_v2168 {
            w.read::<Optional<Str>>()?;
            w.read::<Optional<Str>>()?;
            let rich = w.read::<Str>()?;
            w.write::<Optional<Str>>(Some(rich));
        } else {
            w.write::<Optional<Str>>(None);
            w.write::<Optional<Str>>(None);
            let rich = w.read::<Optional<Str>>()?;
            w.write::<Str>(rich.unwrap_or_default());
        }
    }

    Ok(())
}

pub(crate) fn dimension_data(w: &mut PacketWrapper, to_v2168: bool) -> Result<()> {
    let count = w.passthrough::<UVarInt>()?;
    for _ in 0..count {
        w.passthrough::<Str>()?;
        for _ in 0..4 {
            w.passthrough::<VarInt>()?;
        }
        if to_v2168 {
            w.write::<Uuid>(MceUuid::default());
        } else {
            w.read::<Uuid>()?;
        }
    }
    w.passthrough_all();
    Ok(())
}

pub(crate) fn resource_packs_info(w: &mut PacketWrapper, to_v2168: bool) -> Result<()> {
    w.passthrough::<Bool>()?;
    w.passthrough::<Bool>()?;
    w.passthrough::<Bool>()?;
    w.passthrough::<Bool>()?;
    w.passthrough::<Uuid>()?;
    w.passthrough::<Str>()?;

    if to_v2168 {
        let count = w.read::<UShortLe>()?;
        w.writer().write_count(count as usize);
    } else {
        let count = w.reader().read_count()?;
        let count = u16::try_from(count).map_err(|_| Error::BadDiscriminant {
            what: "resource pack count",
            value: count as i64,
        })?;
        w.write::<UShortLe>(count);
    }

    w.passthrough_all();
    Ok(())
}

pub(crate) fn resource_pack_client_response(w: &mut PacketWrapper, to_v2168: bool) -> Result<()> {
    if to_v2168 {
        let old = w.read::<Byte>()? as u32;
        let new = old
            .checked_sub(PACK_RESPONSE_SHIFT)
            .filter(|v| (*v as usize) < PACK_RESPONSE_NAMES.len())
            .ok_or(Error::BadDiscriminant {
                what: "resource pack response",
                value: old as i64,
            })?;
        let count = w.read::<UShortLe>()? as usize;
        let mut packs = Vec::with_capacity(count);
        for _ in 0..count {
            packs.push(w.read::<Str>()?);
        }

        w.writer().write_uvarint(new);
        w.write::<Str>(PACK_RESPONSE_NAMES[new as usize].to_string());
        if new == PACK_RESPONSE_SEND_PACKS_V2168 {
            w.writer().write_count(packs.len());
            for pack in &packs {
                w.write::<Str>(pack.clone());
            }
        }
    } else {
        let new = w.reader().read_uvarint()?;
        if (new as usize) >= PACK_RESPONSE_NAMES.len() {
            return Err(Error::BadDiscriminant {
                what: "resource pack response",
                value: new as i64,
            });
        }
        w.read::<Str>()?;

        let mut packs = Vec::new();
        if new == PACK_RESPONSE_SEND_PACKS_V2168 {
            let count = w.reader().read_count()?;
            for _ in 0..count {
                packs.push(w.read::<Str>()?);
            }
        }

        w.write::<Byte>((new + PACK_RESPONSE_SHIFT) as u8);
        w.write::<UShortLe>(packs.len() as u16);
        for pack in &packs {
            w.write::<Str>(pack.clone());
        }
    }

    w.passthrough_all();
    Ok(())
}

const PACK_RESPONSE_SHIFT: u32 = 1;

const PACK_RESPONSE_SEND_PACKS_V2168: u32 = 1;

const PACK_RESPONSE_NAMES: [&str; 4] = [
    "cancel",
    "downloading",
    "downloadingfinished",
    "resourcepackstackfinished",
];

pub(crate) fn transfer(w: &mut PacketWrapper, to_v2168: bool) -> Result<()> {
    w.passthrough::<Str>()?;
    w.passthrough::<UShortLe>()?;
    w.passthrough::<Bool>()?;
    if to_v2168 {
        w.writer().write_bool(false);
    } else if w.reader().read_bool()? {
        w.reader().read_remaining();
    }
    w.passthrough_all();
    Ok(())
}
