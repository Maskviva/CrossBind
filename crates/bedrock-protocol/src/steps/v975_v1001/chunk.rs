use super::{SUB_CHUNK_MODE_LIMITED, SUB_CHUNK_MODE_LIMITLESS};
use crate::connection::ConnState;
use bedrock_codec::prelude::*;

pub(crate) fn sub_chunk_request(w: &mut PacketWrapper, to_v1001: bool) -> Result<()> {
    w.passthrough::<VarInt>()?;
    if to_v1001 {
        let pos_x = w.read::<VarInt>()?;
        let pos_y = w.read::<VarInt>()?;
        let pos_z = w.read::<VarInt>()?;
        let offset_count = w.read::<UIntLe>()?;
        let offset_bytes = w.reader().read_bytes(offset_count as usize * 3)?.to_vec();
        w.write::<UVarInt>(offset_count);
        w.writer().write_bytes(&offset_bytes);
        w.write::<IntLe>(pos_x);
        w.write::<IntLe>(pos_y);
        w.write::<IntLe>(pos_z);
    } else {
        let offset_count = w.read::<UVarInt>()?;
        let offset_bytes = w.reader().read_bytes(offset_count as usize * 3)?.to_vec();
        let pos_x = w.read::<IntLe>()?;
        let pos_y = w.read::<IntLe>()?;
        let pos_z = w.read::<IntLe>()?;
        w.write::<VarInt>(pos_x);
        w.write::<VarInt>(pos_y);
        w.write::<VarInt>(pos_z);
        w.write::<UIntLe>(offset_count);
        w.writer().write_bytes(&offset_bytes);
    }
    Ok(())
}

pub(crate) fn client_cache_blob_status(w: &mut PacketWrapper, to_v1001: bool) -> Result<()> {
    if to_v1001 {
        let miss_count = w.read::<UVarInt>()?;
        let hit_count = w.read::<UVarInt>()?;
        let miss_hashes = w.reader().read_bytes(miss_count as usize * 8)?.to_vec();
        let hit_hashes = w.reader().read_bytes(hit_count as usize * 8)?.to_vec();
        w.write::<UVarInt>(miss_count);
        w.writer().write_bytes(&miss_hashes);
        w.write::<UVarInt>(hit_count);
        w.writer().write_bytes(&hit_hashes);
    } else {
        let miss_count = w.read::<UVarInt>()?;
        let miss_hashes = w.reader().read_bytes(miss_count as usize * 8)?.to_vec();
        let hit_count = w.read::<UVarInt>()?;
        let hit_hashes = w.reader().read_bytes(hit_count as usize * 8)?.to_vec();
        w.write::<UVarInt>(miss_count);
        w.write::<UVarInt>(hit_count);
        w.writer().write_bytes(&miss_hashes);
        w.writer().write_bytes(&hit_hashes);
    }
    Ok(())
}

pub(crate) fn level_chunk(w: &mut PacketWrapper, to_v1001: bool) -> Result<()> {
    w.passthrough::<VarInt>()?;
    w.passthrough::<VarInt>()?;
    w.passthrough::<VarInt>()?;

    let count = w.read::<UVarInt>()?;
    if to_v1001 && count == SUB_CHUNK_MODE_LIMITED {
        w.read::<UShortLe>()?;
        w.write::<UVarInt>(SUB_CHUNK_MODE_LIMITLESS);
    } else {
        w.write::<UVarInt>(count);
    }

    w.passthrough_all();
    Ok(())
}

pub(crate) fn client_cache_status(w: &mut PacketWrapper) -> Result<()> {
    w.read::<Bool>()?;
    w.write::<Bool>(false);
    Ok(())
}

pub fn blob_cache_enabled() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| {
        !matches!(
            std::env::var("CROSSBIND_BLOB_CACHE")
                .unwrap_or_default()
                .trim()
                .to_ascii_lowercase()
                .as_str(),
            "off" | "0" | "false" | "no"
        )
    })
}

pub(crate) fn diagnostics(w: &mut PacketWrapper, _state: &mut ConnState) -> Result<()> {
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
    w.read::<RemainingBytes>()?;
    Ok(())
}
