use bedrock_codec::prelude::*;

pub(crate) fn full_chunk_data(w: &mut PacketWrapper, to_v2168: bool) -> Result<()> {
    w.passthrough::<VarInt>()?;
    w.passthrough::<VarInt>()?;
    w.passthrough::<VarInt>()?;

    if to_v2168 {
        let count = w.reader().read_uvarint()?;
        let (count, limit) = match count {
            SUB_CHUNK_MODE_LIMITLESS => (0, Some(-1i32)),
            SUB_CHUNK_MODE_LIMITED => {
                let highest = w.reader().read_u16_le()?;
                (0, Some(highest as i32))
            }
            plain => (plain, None),
        };
        w.writer().write_uvarint(count);
        Optional::<VarInt>::write(w.writer(), &limit);

        let cache = w.passthrough::<Bool>()?;
        if cache {
            let hashes = w.reader().read_count()?;
            w.writer().write_count(hashes);
            for _ in 0..hashes {
                w.passthrough::<UInt64Le>()?;
            }
        } else {
            w.writer().write_count(0);
        }
    } else {
        let count = w.reader().read_uvarint()?;
        let limit = Optional::<VarInt>::read(w.reader())?;
        match limit {
            Some(-1) => w.writer().write_uvarint(SUB_CHUNK_MODE_LIMITLESS),
            Some(highest) => {
                w.writer().write_uvarint(SUB_CHUNK_MODE_LIMITED);
                w.writer().write_u16_le(highest as u16);
            }
            None => w.writer().write_uvarint(count),
        }

        let cache = w.passthrough::<Bool>()?;
        let hashes = w.reader().read_count()?;
        if cache {
            w.writer().write_count(hashes);
            for _ in 0..hashes {
                w.passthrough::<UInt64Le>()?;
            }
        } else {
            for _ in 0..hashes {
                w.read::<UInt64Le>()?;
            }
        }
    }

    w.passthrough_all();
    Ok(())
}

pub(crate) fn sub_chunk(w: &mut PacketWrapper, to_v2168: bool, shape: SubChunkShape) -> Result<()> {
    let cache = w.passthrough::<Bool>()?;
    w.passthrough::<VarInt>()?;

    for _ in 0..3 {
        if to_v2168 {
            w.map::<VarInt, IntLe>()?;
        } else {
            w.map::<IntLe, VarInt>()?;
        }
    }

    let count = if to_v2168 {
        let n = w.read::<UIntLe>()?;
        w.write::<UVarInt>(n);
        n
    } else {
        let n = w.read::<UVarInt>()?;
        w.write::<UIntLe>(n);
        n
    };

    for _ in 0..count {
        sub_chunk_entry(w, to_v2168, cache, shape)?;
    }
    Ok(())
}

fn sub_chunk_entry(
    w: &mut PacketWrapper,
    to_v2168: bool,
    cache: bool,
    shape: SubChunkShape,
) -> Result<()> {
    for _ in 0..3 {
        w.passthrough::<SByte>()?;
    }

    if to_v2168 && shape.strip_content {
        return strip_sub_chunk_entry(w, cache, shape);
    }

    let result = w.passthrough::<Byte>()?;

    let source_writes_payload = !cache || result != SUB_CHUNK_RESULT_SUCCESS_ALL_AIR;
    if to_v2168 {
        let payload = if source_writes_payload {
            w.read::<ByteArray>()?
        } else {
            Vec::new()
        };
        let present = !payload.is_empty() || (source_writes_payload && shape.empty_payload);
        w.write::<Bool>(present);
        if present {
            w.write::<ByteArray>(payload);
        }
    } else {
        let present = w.read::<Bool>()?;
        let payload = if present {
            w.read::<ByteArray>()?
        } else {
            Vec::new()
        };
        if source_writes_payload {
            w.write::<ByteArray>(payload);
        }
    }

    height_map(w, to_v2168, shape)?;
    height_map(w, to_v2168, shape)?;

    if to_v2168 {
        let hash = if cache { w.read::<UInt64Le>()? } else { 0 };
        let present = cache && shape.announce_blob_hash && (shape.zero_blob_hash || hash != 0);
        w.write::<Bool>(present);
        if present {
            w.write::<UInt64Le>(hash);
        }
    } else {
        let present = w.read::<Bool>()?;
        let hash = if present { w.read::<UInt64Le>()? } else { 0 };
        if cache {
            w.write::<UInt64Le>(hash);
        }
    }

    Ok(())
}

fn strip_sub_chunk_entry(w: &mut PacketWrapper, cache: bool, shape: SubChunkShape) -> Result<()> {
    let result = w.read::<Byte>()?;
    w.write::<Byte>(SUB_CHUNK_RESULT_SUCCESS_ALL_AIR);

    if !cache || result != SUB_CHUNK_RESULT_SUCCESS_ALL_AIR {
        w.read::<ByteArray>()?;
    }
    w.write::<Bool>(false);

    for _ in 0..2 {
        let map_type = w.read::<Byte>()?;
        if map_type == HEIGHT_MAP_HAS_DATA {
            w.reader().read_bytes(HEIGHT_MAP_LEN)?;
        }
        if shape.type_bytes {
            w.write::<Byte>(HEIGHT_MAP_NONE);
        }
        w.write::<Bool>(false);
    }

    if cache {
        w.read::<UInt64Le>()?;
    }
    w.write::<Bool>(false);
    Ok(())
}

fn height_map(w: &mut PacketWrapper, to_v2168: bool, shape: SubChunkShape) -> Result<()> {
    if to_v2168 {
        let map_type = w.read::<Byte>()?;
        if shape.type_bytes {
            w.write::<Byte>(map_type);
        }
        let present = map_type == HEIGHT_MAP_HAS_DATA;
        w.write::<Bool>(present);
        if present {
            let data = w.reader().read_bytes(HEIGHT_MAP_LEN)?.to_vec();
            w.writer().write_bytes(&data);
        }
    } else {
        let map_type = if shape.type_bytes {
            w.read::<Byte>()?
        } else {
            HEIGHT_MAP_NONE
        };
        let present = w.read::<Bool>()?;
        let data = if present {
            w.reader().read_bytes(HEIGHT_MAP_LEN)?.to_vec()
        } else {
            Vec::new()
        };
        let map_type = if !shape.type_bytes && present {
            HEIGHT_MAP_HAS_DATA
        } else {
            map_type
        };
        w.write::<Byte>(map_type);
        if map_type == HEIGHT_MAP_HAS_DATA {
            if data.len() == HEIGHT_MAP_LEN {
                w.writer().write_bytes(&data);
            } else {
                w.writer().write_bytes(&[0u8; HEIGHT_MAP_LEN]);
            }
        }
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct SubChunkShape {
    pub(crate) type_bytes: bool,
    pub(crate) announce_blob_hash: bool,
    pub(crate) zero_blob_hash: bool,
    pub(crate) empty_payload: bool,
    pub(crate) strip_content: bool,
}

impl SubChunkShape {
    pub(crate) const DOCUMENTED: SubChunkShape = SubChunkShape {
        type_bytes: true,
        announce_blob_hash: true,
        zero_blob_hash: true,
        empty_payload: true,
        strip_content: false,
    };

    pub(crate) const SUPPRESS_ZERO_HASH: SubChunkShape = SubChunkShape {
        zero_blob_hash: false,
        ..SubChunkShape::DOCUMENTED
    };

    pub(crate) const SUPPRESS_EMPTY_PAYLOAD: SubChunkShape = SubChunkShape {
        empty_payload: false,
        ..SubChunkShape::SUPPRESS_ZERO_HASH
    };

    pub(crate) const DEFAULT: SubChunkShape = SubChunkShape::SUPPRESS_EMPTY_PAYLOAD;

    pub(crate) const NO_BLOB_HASH: SubChunkShape = SubChunkShape {
        announce_blob_hash: false,
        ..SubChunkShape::DEFAULT
    };
}

pub(crate) fn sub_chunk_shape() -> Option<SubChunkShape> {
    static SHAPE: std::sync::OnceLock<Option<SubChunkShape>> = std::sync::OnceLock::new();
    *SHAPE.get_or_init(|| {
        let raw = std::env::var("CROSSBIND_SUBCHUNK").unwrap_or_default();
        match raw.trim().to_ascii_lowercase().as_str() {
            "off" | "drop" | "0" => None,
            "a" => Some(SubChunkShape::DOCUMENTED),
            "b" => Some(SubChunkShape {
                type_bytes: false,
                ..SubChunkShape::DOCUMENTED
            }),
            "c" => Some(SubChunkShape::SUPPRESS_ZERO_HASH),
            "d" => Some(SubChunkShape {
                type_bytes: false,
                ..SubChunkShape::SUPPRESS_ZERO_HASH
            }),
            "e" => Some(SubChunkShape::NO_BLOB_HASH),
            "f" => Some(SubChunkShape {
                type_bytes: false,
                ..SubChunkShape::NO_BLOB_HASH
            }),
            "g" => Some(SubChunkShape::SUPPRESS_EMPTY_PAYLOAD),
            "h" => Some(SubChunkShape {
                type_bytes: false,
                ..SubChunkShape::SUPPRESS_EMPTY_PAYLOAD
            }),
            "air" | "strip" => Some(SubChunkShape {
                strip_content: true,
                ..SubChunkShape::DEFAULT
            }),
            _ => Some(SubChunkShape::DEFAULT),
        }
    })
}

pub fn sub_chunk_mode_label() -> &'static str {
    let shape = match sub_chunk_shape() {
        None => return "off (packet dropped)",
        Some(shape) => shape,
    };
    if shape.strip_content {
        return "air (framing only, no blocks)";
    }
    if shape == SubChunkShape::SUPPRESS_EMPTY_PAYLOAD {
        return "g (default: a zero hash and an empty payload are both left absent)";
    }
    if shape == SubChunkShape::SUPPRESS_ZERO_HASH {
        return "c (empty payload announced present — a v2168 client dies on a long teleport)";
    }
    if shape == SubChunkShape::DOCUMENTED {
        return "a (schema shape, hash 0 included — a v2168 client stalls a second after spawn)";
    }
    if !shape.announce_blob_hash {
        return "e/f (no blob announced — a v2168 client refuses this)";
    }
    "b/d/h (height map type bytes omitted)"
}

pub(crate) const SUB_CHUNK_MODE_LIMITLESS: u32 = 0xFFFF_FFFF;
pub(crate) const SUB_CHUNK_MODE_LIMITED: u32 = 0xFFFF_FFFE;
pub(crate) const SUB_CHUNK_RESULT_SUCCESS_ALL_AIR: u8 = 6;
pub(crate) const HEIGHT_MAP_NONE: u8 = 0;
pub(crate) const HEIGHT_MAP_HAS_DATA: u8 = 1;
pub(crate) const HEIGHT_MAP_LEN: usize = 256;
