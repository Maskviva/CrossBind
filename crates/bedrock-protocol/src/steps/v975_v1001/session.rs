use bedrock_codec::prelude::*;

pub(crate) fn start_game(w: &mut PacketWrapper, to_v1001: bool) -> Result<()> {
    w.passthrough::<VarInt64>()?;
    w.passthrough::<UVarInt64>()?;
    w.passthrough::<VarInt>()?;
    w.passthrough::<Vec3>()?;
    w.passthrough::<Vec2>()?;

    w.passthrough::<LevelSettingsV944>()?;

    if to_v1001 {
        w.write::<VarInt>(0);
        w.write::<Bool>(false);
    } else {
        w.read::<VarInt>()?;
        w.read::<Bool>()?;
    }

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

    if to_v1001 {
        w.write::<Bool>(false);
    } else {
        w.read::<Bool>()?;
    }

    w.passthrough_all();
    Ok(())
}

pub(crate) fn biome_definition_list(w: &mut PacketWrapper) -> Result<()> {
    let count = w.reader().read_count()?;
    w.writer().write_count(count);

    for _ in 0..count {
        w.passthrough::<ShortLe>()?;
        w.passthrough::<ShortLe>()?;
        for _ in 0..5 {
            w.passthrough::<FloatLe>()?;
        }
        w.passthrough::<IntLe>()?;
        w.passthrough::<Bool>()?;

        if w.passthrough::<Bool>()? {
            let tags = w.reader().read_count()?;
            w.writer().write_count(tags);
            for _ in 0..tags {
                w.passthrough::<UShortLe>()?;
            }
        }

        if w.passthrough::<Bool>()? {
            return Err(Error::Invalid(
                "BiomeDefinitionList carries ChunkGeneration, whose surface \
                 builders changed shape at 1.26.30 and are not translated",
            ));
        }
    }

    w.passthrough_all();
    Ok(())
}
