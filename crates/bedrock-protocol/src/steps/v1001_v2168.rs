use bedrock_codec::prelude::*;

use crate::connection::ConnState;
use crate::direction::Direction;
use crate::packet_ids::ids;
use crate::pipeline::trace_limit;
use crate::steps::crafting_data_v2168::crafting_data;
use crate::steps::item_stack_v2168::{
    cache_item_registry, item_stack_request, item_stack_response,
};
use crate::steps::player_list_v2168::player_list;
use crate::steps::set_score_v2168::{set_score, set_scoreboard_identity};
use crate::translator::Translator;

const SUB_CHUNK_MODE_LIMITLESS: u32 = 0xFFFF_FFFF;
const SUB_CHUNK_MODE_LIMITED: u32 = 0xFFFF_FFFE;

const INPUT_FLAG_BITSET_SIZE_V1001: u32 = 65;
const INPUT_FLAG_COUNT_V2168: u32 = 66;

const FLAG_PERFORM_ITEM_INTERACTION: u32 = 34;
const FLAG_PERFORM_BLOCK_ACTIONS: u32 = 35;
const FLAG_PERFORM_ITEM_STACK_REQUEST: u32 = 36;
const FLAG_CLIENT_PREDICTED_VEHICLE: u32 = 45;

const MOVE_DELTA_HAS_X: u16 = 1 << 0;
const MOVE_DELTA_HAS_Y: u16 = 1 << 1;
const MOVE_DELTA_HAS_Z: u16 = 1 << 2;
const MOVE_DELTA_HAS_ROT_X: u16 = 1 << 3;
const MOVE_DELTA_HAS_ROT_Y: u16 = 1 << 4;
const MOVE_DELTA_HAS_ROT_Z: u16 = 1 << 5;
const MOVE_DELTA_ON_GROUND: u16 = 1 << 6;
const MOVE_DELTA_TELEPORT: u16 = 1 << 7;
const MOVE_DELTA_FORCE_MOVE: u16 = 1 << 8;

const MOVE_MODE_TELEPORT: u8 = 2;

#[allow(dead_code)]
const MOVE_DELTA_TELEPORT_UNUSED: u16 = MOVE_DELTA_TELEPORT;

const PACK_RESPONSE_SHIFT: u32 = 1;
const PACK_RESPONSE_SEND_PACKS_V2168: u32 = 1;

const PACK_RESPONSE_NAMES: [&str; 4] = [
    "cancel",
    "downloading",
    "downloadingfinished",
    "resourcepackstackfinished",
];

fn start_game(w: &mut PacketWrapper, to_v2168: bool) -> Result<()> {
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

fn server_join_information(w: &mut PacketWrapper, to_v2168: bool) -> Result<()> {
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

fn creative_item_stack(w: &mut PacketWrapper, to_v2168: bool) -> Result<()> {
    if to_v2168 {
        w.map::<NetworkItemInstanceDescriptor, NetworkItemInstanceDescriptorV2168>()?;
    } else {
        w.map::<NetworkItemInstanceDescriptorV2168, NetworkItemInstanceDescriptor>()?;
    }
    Ok(())
}

fn creative_content(w: &mut PacketWrapper, to_v2168: bool) -> Result<()> {
    let groups = w.passthrough::<UVarInt>()?;
    for _ in 0..groups {
        if to_v2168 {
            let category = w.reader().read_i32_le()?;
            w.writer().write_u8(category as u8);
        } else {
            let category = w.reader().read_u8()?;
            w.writer().write_i32_le(category as i32);
        }
        w.passthrough::<Str>()?;
        creative_item_stack(w, to_v2168)?;
    }

    let items = w.passthrough::<UVarInt>()?;
    for _ in 0..items {
        w.passthrough::<UVarInt>()?;
        creative_item_stack(w, to_v2168)?;
        w.passthrough::<UVarInt>()?;
    }

    w.passthrough_all();
    Ok(())
}

fn dimension_data(w: &mut PacketWrapper, to_v2168: bool) -> Result<()> {
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

fn resource_packs_info(w: &mut PacketWrapper, to_v2168: bool) -> Result<()> {
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

fn resource_pack_client_response(w: &mut PacketWrapper, to_v2168: bool) -> Result<()> {
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

fn full_chunk_data(w: &mut PacketWrapper, to_v2168: bool) -> Result<()> {
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

const SUB_CHUNK_RESULT_LEVEL_CHUNK_DOESNT_EXIST: u8 = 2;
const SUB_CHUNK_RESULT_SUCCESS_ALL_AIR: u8 = 6;
const HEIGHT_MAP_NONE: u8 = 0;
const HEIGHT_MAP_HAS_DATA: u8 = 1;
const HEIGHT_MAP_LEN: usize = 256;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SubChunkShape {
    type_bytes: bool,
    announce_blob_hash: bool,
    zero_blob_hash: bool,
    empty_payload: bool,
    strip_content: bool,
}

impl SubChunkShape {
    const DOCUMENTED: SubChunkShape = SubChunkShape {
        type_bytes: true,
        announce_blob_hash: true,
        zero_blob_hash: true,
        empty_payload: true,
        strip_content: false,
    };

    const SUPPRESS_ZERO_HASH: SubChunkShape = SubChunkShape {
        zero_blob_hash: false,
        ..SubChunkShape::DOCUMENTED
    };

    const SUPPRESS_EMPTY_PAYLOAD: SubChunkShape = SubChunkShape {
        empty_payload: false,
        ..SubChunkShape::SUPPRESS_ZERO_HASH
    };

    const DEFAULT: SubChunkShape = SubChunkShape::SUPPRESS_EMPTY_PAYLOAD;

    const NO_BLOB_HASH: SubChunkShape = SubChunkShape {
        announce_blob_hash: false,
        ..SubChunkShape::DEFAULT
    };
}

fn sub_chunk_shape() -> Option<SubChunkShape> {
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

fn sub_chunk(w: &mut PacketWrapper, to_v2168: bool, shape: SubChunkShape) -> Result<()> {
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
        let present =
            cache && shape.announce_blob_hash && (shape.zero_blob_hash || hash != 0);
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

fn read_bitset(w: &mut PacketWrapper, size: u32) -> Result<Vec<u32>> {
    let mut set = Vec::new();
    let mut base = 0u32;
    while base < size {
        let byte = w.reader().read_u8()?;
        for bit in 0..7u32 {
            if byte & (1 << bit) != 0 {
                let id = base + bit;
                if id >= size {
                    return Err(Error::BadDiscriminant {
                        what: "player auth input flag",
                        value: id as i64,
                    });
                }
                set.push(id);
            }
        }
        if byte & 0x80 == 0 {
            return Ok(set);
        }
        base += 7;
    }
    Err(Error::BadDiscriminant {
        what: "player auth input bitset",
        value: size as i64,
    })
}

fn write_bitset(w: &mut PacketWrapper, set: &[u32], size: u32) {
    let highest = set.iter().copied().max().unwrap_or(0);
    let bytes = ((highest / 7) + 1).min(size.div_ceil(7));
    for group in 0..bytes {
        let mut byte = 0u8;
        for bit in 0..7u32 {
            if set.contains(&(group * 7 + bit)) {
                byte |= 1 << bit;
            }
        }
        if group + 1 < bytes {
            byte |= 0x80;
        }
        w.writer().write_u8(byte);
    }
}

const PLAYER_ACTION_START_BREAK: i32 = 0;
const PLAYER_ACTION_ABORT_BREAK: i32 = 1;
const PLAYER_ACTION_CRACK_BREAK: i32 = 18;
const PLAYER_ACTION_PREDICT_DESTROY_BLOCK: i32 = 26;
const PLAYER_ACTION_CONTINUE_DESTROY_BLOCK: i32 = 27;

fn block_action_has_position(action: i32) -> bool {
    matches!(
        action,
        PLAYER_ACTION_START_BREAK
            | PLAYER_ACTION_ABORT_BREAK
            | PLAYER_ACTION_CRACK_BREAK
            | PLAYER_ACTION_PREDICT_DESTROY_BLOCK
            | PLAYER_ACTION_CONTINUE_DESTROY_BLOCK
    )
}

const INVENTORY_SOURCE_CONTAINER: u32 = 0;
const INVENTORY_SOURCE_WORLD: u32 = 2;
const INVENTORY_SOURCE_TODO: u32 = 99999;

fn read_double_optional_from(r: &mut Reader<'_>) -> Result<bool> {
    if !r.read_bool()? {
        return Ok(false);
    }
    r.read_bool()
}

fn item_stack_to_v1001(r: &mut Reader<'_>, out: &mut Writer) -> Result<()> {
    let item = ItemInstanceV2168::read(r)?;
    ItemInstance::write(out, &item);
    Ok(())
}

fn block_actions_to_v1001(r: &mut Reader<'_>, out: &mut Writer) -> Result<()> {
    let count = r.read_count()?;
    out.write_varint(count as i32);
    for _ in 0..count {
        let action = r.read_varint()?;
        let position = BlockPos::read(r)?;
        let face = r.read_varint()?;
        out.write_varint(action);
        if block_action_has_position(action) {
            BlockPos::write(out, &position);
            out.write_varint(face);
        }
    }
    Ok(())
}

fn inventory_action_to_v1001(r: &mut Reader<'_>, out: &mut Writer) -> Result<()> {
    let source = r.read_uvarint()?;
    out.write_uvarint(source);

    let window_id = if read_double_optional_from(r)? {
        Some(r.read_i8()?)
    } else {
        None
    };
    let source_flags = if read_double_optional_from(r)? {
        Some(r.read_uvarint()?)
    } else {
        None
    };
    match source {
        INVENTORY_SOURCE_CONTAINER | INVENTORY_SOURCE_TODO => {
            out.write_varint(window_id.unwrap_or(0) as i32)
        }
        INVENTORY_SOURCE_WORLD => out.write_uvarint(source_flags.unwrap_or(0)),
        _ => {}
    }

    out.write_uvarint(r.read_uvarint()?);
    item_stack_to_v1001(r, out)?;
    item_stack_to_v1001(r, out)?;
    Ok(())
}

fn item_interaction_to_v1001(r: &mut Reader<'_>, out: &mut Writer) -> Result<()> {
    let legacy_request_id = r.read_varint()?;
    out.write_varint(legacy_request_id);

    let mut slots: Vec<(u8, Vec<u8>)> = Vec::new();
    if r.read_bool()? {
        let count = r.read_count()?;
        for _ in 0..count {
            let container = r.read_u8()?;
            slots.push((container, ByteArray::read(r)?));
        }
    }
    if legacy_request_id < -1 && (legacy_request_id & 1) == 0 {
        out.write_count(slots.len());
        for (container, payload) in &slots {
            out.write_u8(*container);
            out.write_count(payload.len());
            out.write_bytes(payload);
        }
    }

    let action_count = if read_double_optional_from(r)? {
        r.read_count()?
    } else {
        0
    };
    out.write_count(action_count);
    for _ in 0..action_count {
        inventory_action_to_v1001(r, out)?;
    }

    out.write_uvarint(r.read_varint()? as u32);
    out.write_uvarint(r.read_u8()? as u32);
    BlockPos::write(out, &BlockPos::read(r)?);
    out.write_varint(r.read_u8()? as i32);
    out.write_varint(r.read_varint()?);
    item_stack_to_v1001(r, out)?;
    Vec3::write(out, &Vec3::read(r)?);
    Vec3::write(out, &Vec3::read(r)?);
    out.write_uvarint(r.read_uvarint()?);
    out.write_u8(r.read_u8()?);
    out.write_u8(r.read_u8()?);
    Ok(())
}

fn set_flag(flags: &mut Vec<u32>, flag: u32, present: bool) {
    match (flags.iter().position(|f| *f == flag), present) {
        (None, true) => flags.push(flag),
        (Some(at), false) => {
            flags.remove(at);
        }
        _ => {}
    }
}

fn player_auth_input(w: &mut PacketWrapper, state: &mut ConnState, to_v2168: bool) -> Result<()> {
    if to_v2168 {
        player_auth_input_to_v2168(w)
    } else {
        player_auth_input_to_v1001(w, Some(state))
    }
}

fn player_auth_input_to_v1001(
    w: &mut PacketWrapper,
    mut state: Option<&mut ConnState>,
) -> Result<()> {
    w.passthrough::<FloatLe>()?;
    w.passthrough::<FloatLe>()?;
    w.passthrough::<Vec3>()?;
    w.passthrough::<Vec2>()?;
    w.passthrough::<FloatLe>()?;

    let mut flags: Vec<u32> = Vec::new();
    if w.reader().read_bool()? {
        let count = w.reader().read_count()?;
        for _ in 0..count {
            let id = w.reader().read_varint()?;
            if id < 0 || id as u32 >= INPUT_FLAG_COUNT_V2168 {
                return Err(Error::BadDiscriminant {
                    what: "player auth input flag",
                    value: id as i64,
                });
            }
            if (id as u32) < INPUT_FLAG_BITSET_SIZE_V1001 {
                flags.push(id as u32);
            }
        }
    }

    let input_mode = w.reader().read_uvarint()?;
    let play_mode = w.reader().read_uvarint()?;
    let interaction_model = w.reader().read_varint()?;
    let interact_pitch = w.reader().read_f32_le()?;
    let interact_yaw = w.reader().read_f32_le()?;
    let tick = w.reader().read_uvarint64()?;
    let delta = Vec3::read(w.reader())?;

    let mut payload = Writer::new();

    let has_item_interaction = read_double_optional(w)?;
    if has_item_interaction {
        item_interaction_to_v1001(w.reader(), &mut payload)?;
    }

    if read_double_optional(w)? {
        if trace_limit() != 0 {
            if let Some(state) = state.as_deref_mut() {
                state
                    .notices
                    .push("auth input: tick dropped, carries an ItemStackRequest".to_owned());
            }
        }
        w.cancel();
        return Ok(());
    }

    let has_block_actions = read_double_optional(w)?;
    if has_block_actions {
        block_actions_to_v1001(w.reader(), &mut payload)?;
    }

    let rotation = if read_double_optional(w)? {
        Some(Vec2::read(w.reader())?)
    } else {
        None
    };
    let vehicle = if read_double_optional(w)? {
        Some(w.reader().read_varint64()?)
    } else {
        None
    };
    let has_vehicle = match (rotation, vehicle) {
        (Some(rotation), Some(vehicle)) => {
            Vec2::write(&mut payload, &rotation);
            payload.write_varint64(vehicle);
            true
        }
        _ => false,
    };

    set_flag(
        &mut flags,
        FLAG_PERFORM_ITEM_INTERACTION,
        has_item_interaction,
    );
    set_flag(&mut flags, FLAG_PERFORM_ITEM_STACK_REQUEST, false);
    set_flag(&mut flags, FLAG_PERFORM_BLOCK_ACTIONS, has_block_actions);
    set_flag(&mut flags, FLAG_CLIENT_PREDICTED_VEHICLE, has_vehicle);

    if has_item_interaction || has_block_actions {
        if trace_limit() != 0 {
            if let Some(state) = state.as_deref_mut() {
                if trace_limit() != 0 {
                    let bytes = payload.len();
                    state.notices.push(format!(
                        "auth input: interaction={has_item_interaction} block_actions={has_block_actions} flags={flags:?} payload={bytes} B"
                    ));
                }
            }
        }
    }

    write_bitset(w, &flags, INPUT_FLAG_BITSET_SIZE_V1001);
    w.writer().write_uvarint(input_mode);
    w.writer().write_uvarint(play_mode);
    w.writer().write_uvarint(interaction_model as u32);
    w.writer().write_f32_le(interact_pitch);
    w.writer().write_f32_le(interact_yaw);
    w.writer().write_uvarint64(tick);
    Vec3::write(w.writer(), &delta);
    let payload = payload.into_vec();
    w.writer().write_bytes(&payload);

    w.passthrough_all();
    Ok(())
}

fn player_auth_input_to_v2168(w: &mut PacketWrapper) -> Result<()> {
    w.passthrough::<FloatLe>()?;
    w.passthrough::<FloatLe>()?;
    w.passthrough::<Vec3>()?;
    w.passthrough::<Vec2>()?;
    w.passthrough::<FloatLe>()?;

    let flags = read_bitset(w, INPUT_FLAG_BITSET_SIZE_V1001)?;
    w.writer().write_bool(true);
    w.writer().write_count(flags.len());
    for id in &flags {
        w.writer().write_varint(*id as i32);
    }

    w.passthrough::<UVarInt>()?;
    w.passthrough::<UVarInt>()?;

    let model = w.reader().read_uvarint()?;
    w.writer().write_varint(model as i32);

    w.passthrough::<FloatLe>()?;
    w.passthrough::<FloatLe>()?;
    w.passthrough::<UVarInt64>()?;
    w.passthrough::<Vec3>()?;

    let has = |flag: u32| flags.contains(&flag);

    if has(FLAG_PERFORM_ITEM_INTERACTION)
        || has(FLAG_PERFORM_ITEM_STACK_REQUEST)
        || has(FLAG_PERFORM_BLOCK_ACTIONS)
    {
        w.cancel();
        return Ok(());
    }
    for _ in 0..3 {
        w.writer().write_bool(true);
        w.writer().write_bool(false);
    }

    if has(FLAG_CLIENT_PREDICTED_VEHICLE) {
        let rotation = w.read::<Vec2>()?;
        let vehicle = w.reader().read_varint64()?;
        w.writer().write_bool(true);
        w.writer().write_bool(true);
        w.write::<Vec2>(rotation);
        w.writer().write_bool(true);
        w.writer().write_bool(true);
        w.writer().write_varint64(vehicle);
    } else {
        for _ in 0..2 {
            w.writer().write_bool(true);
            w.writer().write_bool(false);
        }
    }

    w.passthrough_all();
    Ok(())
}

fn read_double_optional(w: &mut PacketWrapper) -> Result<bool> {
    if !w.reader().read_bool()? {
        return Ok(false);
    }
    w.reader().read_bool()
}

fn move_delta_actor(w: &mut PacketWrapper, to_v2168: bool) -> Result<()> {
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

fn move_player(w: &mut PacketWrapper, to_v2168: bool) -> Result<()> {
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

fn convert_item(w: &mut PacketWrapper, to_v2168: bool) -> Result<()> {
    if to_v2168 {
        w.map::<ItemInstanceV975, ItemInstanceV2168>()?;
    } else {
        w.map::<ItemInstanceV2168, ItemInstanceV975>()?;
    }
    Ok(())
}

fn convert_legacy_item(w: &mut PacketWrapper, to_v2168: bool) -> Result<()> {
    if to_v2168 {
        w.map::<ItemInstance, ItemInstanceV2168>()?;
    } else {
        w.map::<ItemInstanceV2168, ItemInstance>()?;
    }
    Ok(())
}

const TRANSACTION_ITEM_USE: u32 = 2;
const TRANSACTION_ITEM_USE_ON_ENTITY: u32 = 3;
const TRANSACTION_ITEM_RELEASE: u32 = 4;

fn double_optional<'a>(
    w: &mut PacketWrapper<'a>,
    from_v2168: bool,
    value: impl FnOnce(&mut PacketWrapper<'a>) -> Result<()>,
) -> Result<()> {
    let outer = w.read::<Bool>()?;
    let inner = if outer || !from_v2168 {
        w.read::<Bool>()?
    } else {
        false
    };
    w.write::<Bool>(true);
    w.write::<Bool>(inner);
    if inner {
        value(w)?;
    }
    Ok(())
}

fn inventory_action(w: &mut PacketWrapper, to_v2168: bool) -> Result<()> {
    w.passthrough::<UVarInt>()?;
    double_optional(w, !to_v2168, |w| {
        w.passthrough::<SByte>()?;
        Ok(())
    })?;
    double_optional(w, !to_v2168, |w| {
        w.passthrough::<UVarInt>()?;
        Ok(())
    })?;
    w.passthrough::<UVarInt>()?;
    convert_item(w, to_v2168)?;
    convert_item(w, to_v2168)?;
    Ok(())
}

fn copy_legacy_set_item_slots(w: &mut PacketWrapper) -> Result<()> {
    w.passthrough_each(|w| {
        w.passthrough::<Byte>()?;
        w.passthrough_each(|w| {
            w.passthrough::<Byte>()?;
            Ok(())
        })?;
        Ok(())
    })?;
    Ok(())
}

fn use_item_transaction(w: &mut PacketWrapper, to_v2168: bool) -> Result<()> {
    w.passthrough::<VarInt>()?;
    w.passthrough::<Byte>()?;
    w.passthrough::<BlockPos>()?;
    w.passthrough::<Byte>()?;
    w.passthrough::<VarInt>()?;
    convert_item(w, to_v2168)?;
    w.passthrough::<Vec3>()?;
    w.passthrough::<Vec3>()?;
    w.passthrough::<UVarInt>()?;
    w.passthrough::<Byte>()?;
    w.passthrough::<Byte>()?;
    Ok(())
}

fn use_item_on_entity_transaction(w: &mut PacketWrapper, to_v2168: bool) -> Result<()> {
    w.passthrough::<UVarInt64>()?;
    w.passthrough::<VarInt>()?;
    w.passthrough::<VarInt>()?;
    convert_item(w, to_v2168)?;
    w.passthrough::<Vec3>()?;
    w.passthrough::<Vec3>()?;
    Ok(())
}

fn release_item_transaction(w: &mut PacketWrapper, to_v2168: bool) -> Result<()> {
    w.passthrough::<VarInt>()?;
    w.passthrough::<VarInt>()?;
    convert_item(w, to_v2168)?;
    w.passthrough::<Vec3>()?;
    Ok(())
}

fn inventory_transaction(w: &mut PacketWrapper, to_v2168: bool) -> Result<()> {
    w.passthrough::<VarInt>()?;
    if w.passthrough::<Bool>()? {
        copy_legacy_set_item_slots(w)?;
    }
    w.passthrough::<Bool>()?;
    let transaction_type = w.passthrough::<UVarInt>()?;
    w.passthrough::<Bool>()?;
    w.passthrough_each(|w| inventory_action(w, to_v2168))?;
    match transaction_type {
        TRANSACTION_ITEM_USE => use_item_transaction(w, to_v2168)?,
        TRANSACTION_ITEM_USE_ON_ENTITY => use_item_on_entity_transaction(w, to_v2168)?,
        TRANSACTION_ITEM_RELEASE => release_item_transaction(w, to_v2168)?,
        _ => {}
    }
    Ok(())
}

fn actor_data(w: &mut PacketWrapper, to_v2168: bool) -> Result<()> {
    if to_v2168 {
        w.map::<ActorDataList, ActorDataListV2168>()?;
    } else {
        w.map::<ActorDataListV2168, ActorDataList>()?;
    }
    Ok(())
}

fn set_actor_data(w: &mut PacketWrapper, to_v2168: bool) -> Result<()> {
    w.passthrough::<UVarInt64>()?;
    actor_data(w, to_v2168)?;
    w.passthrough_all();
    Ok(())
}

fn add_actor(w: &mut PacketWrapper, to_v2168: bool) -> Result<()> {
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

fn add_item_actor(w: &mut PacketWrapper, to_v2168: bool) -> Result<()> {
    w.passthrough::<VarInt64>()?;
    w.passthrough::<UVarInt64>()?;
    convert_legacy_item(w, to_v2168)?;
    w.passthrough::<Vec3>()?;
    w.passthrough::<Vec3>()?;
    actor_data(w, to_v2168)?;
    w.passthrough_all();
    Ok(())
}

fn add_player(w: &mut PacketWrapper, to_v2168: bool) -> Result<()> {
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

fn player_equipment(w: &mut PacketWrapper, to_v2168: bool) -> Result<()> {
    w.passthrough::<UVarInt64>()?;
    convert_item(w, to_v2168)?;
    w.passthrough_all();
    Ok(())
}

fn mob_armor_equipment(w: &mut PacketWrapper, to_v2168: bool) -> Result<()> {
    w.passthrough::<UVarInt64>()?;
    for _ in 0..5 {
        convert_item(w, to_v2168)?;
    }
    w.passthrough_all();
    Ok(())
}

fn inventory_content(w: &mut PacketWrapper, to_v2168: bool) -> Result<()> {
    w.passthrough::<UVarInt>()?;
    let count = w.passthrough::<UVarInt>()?;
    for _ in 0..count {
        convert_item(w, to_v2168)?;
    }
    w.passthrough::<Byte>()?;
    w.passthrough::<Optional<UIntLe>>()?;
    convert_item(w, to_v2168)?;
    w.passthrough_all();
    Ok(())
}

fn inventory_slot(w: &mut PacketWrapper, to_v2168: bool) -> Result<()> {
    w.passthrough::<UVarInt>()?;
    w.passthrough::<UVarInt>()?;
    if w.passthrough::<Bool>()? {
        w.passthrough::<Byte>()?;
        w.passthrough::<Optional<UIntLe>>()?;
    }
    if w.passthrough::<Bool>()? {
        convert_item(w, to_v2168)?;
    }
    convert_item(w, to_v2168)?;
    w.passthrough_all();
    Ok(())
}

const PLAY_SOUND_ONCE: i32 = 0;

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

fn play_sound(w: &mut PacketWrapper, to_v2168: bool) -> Result<()> {
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

const STRUCTURE_REDSTONE_SAVE_MODE_MAX: u8 = 3;

fn structure_block_update(
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

fn transfer(w: &mut PacketWrapper, to_v2168: bool) -> Result<()> {
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

fn anvil_damage(w: &mut PacketWrapper, to_v2168: bool) -> Result<()> {
    if to_v2168 {
        w.read::<Byte>()?;
    } else {
        w.write::<Byte>(0);
    }
    w.passthrough_all();
    Ok(())
}

fn serverbound_diagnostics(w: &mut PacketWrapper, to_v2168: bool) -> Result<()> {
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

pub fn downgrade() -> Translator {
    build("v2168->v1001", 1001, 2168)
}

pub fn upgrade() -> Translator {
    build("v1001->v2168", 2168, 1001)
}

fn build(name: &'static str, server_protocol: u32, client_protocol: u32) -> Translator {
    let to_client_v2168 = client_protocol == 2168;
    let to_server_v2168 = server_protocol == 2168;
    let sub_chunk_shape = sub_chunk_shape();

    let mut step = Translator::new(name, server_protocol, client_protocol)
        .clientbound(ids::ITEM_REGISTRY, cache_item_registry)
        .clientbound(ids::CRAFTING_DATA, move |w, state| {
            if !crafting_data(w, state, to_client_v2168)? {
                w.cancel();
            }
            Ok(())
        })
        .serverbound(ids::CRAFTING_DATA, move |w, state| {
            if !crafting_data(w, state, to_server_v2168)? {
                w.cancel();
            }
            Ok(())
        })
        .clientbound(ids::PLAYER_LIST, move |w, state| {
            if !player_list(w, state, to_client_v2168)? {
                w.cancel();
            }
            Ok(())
        })
        .serverbound(ids::ITEM_STACK_REQUEST, move |w, state| {
            let names = std::mem::take(&mut state.item_ids);
            let ids = std::mem::take(&mut state.item_names);
            let outcome = item_stack_request(w, to_server_v2168, &names, &ids);
            state.item_ids = names;
            state.item_names = ids;
            if !outcome? {
                w.cancel();
            }
            Ok(())
        })
        .clientbound(ids::ITEM_STACK_RESPONSE, move |w, _| {
            item_stack_response(w, to_client_v2168)
        })
        .clientbound(ids::START_GAME, move |w, _| start_game(w, to_client_v2168))
        .clientbound(ids::RESOURCE_PACKS_INFO, move |w, _| {
            resource_packs_info(w, to_client_v2168)
        })
        .clientbound(ids::FULL_CHUNK_DATA, move |w, _| {
            full_chunk_data(w, to_client_v2168)
        })
        .clientbound(ids::MOVE_DELTA_ACTOR, move |w, _| {
            move_delta_actor(w, to_client_v2168)
        })
        .clientbound(ids::MOVE_PLAYER, move |w, _| {
            move_player(w, to_client_v2168)
        })
        .clientbound(ids::PLAYER_EQUIPMENT, move |w, _| {
            player_equipment(w, to_client_v2168)
        })
        .clientbound(ids::MOB_ARMOR_EQUIPMENT, move |w, _| {
            mob_armor_equipment(w, to_client_v2168)
        })
        .clientbound(ids::INVENTORY_CONTENT, move |w, _| {
            inventory_content(w, to_client_v2168)
        })
        .clientbound(ids::INVENTORY_SLOT, move |w, _| {
            inventory_slot(w, to_client_v2168)
        })
        .clientbound(ids::INVENTORY_TRANSACTION, move |w, _| {
            inventory_transaction(w, to_client_v2168)
        })
        .clientbound(ids::SET_ACTOR_DATA, move |w, _| {
            set_actor_data(w, to_client_v2168)
        })
        .clientbound(ids::ADD_ACTOR, move |w, _| add_actor(w, to_client_v2168))
        .clientbound(ids::ADD_ITEM_ACTOR, move |w, _| {
            add_item_actor(w, to_client_v2168)
        })
        .clientbound(ids::ADD_PLAYER, move |w, _| add_player(w, to_client_v2168))
        .clientbound(ids::PLAY_SOUND, move |w, _| play_sound(w, to_client_v2168))
        .clientbound(ids::STRUCTURE_BLOCK_UPDATE, move |w, s| {
            structure_block_update(w, s, to_client_v2168)
        })
        .serverbound(ids::STRUCTURE_BLOCK_UPDATE, move |w, s| {
            structure_block_update(w, s, to_server_v2168)
        })
        .clientbound(ids::TRANSFER, move |w, _| transfer(w, to_client_v2168))
        .clientbound(ids::CREATIVE_CONTENT, move |w, _| {
            creative_content(w, to_client_v2168)
        })
        .clientbound(ids::DIMENSION_DATA, move |w, _| {
            dimension_data(w, to_client_v2168)
        })
        .serverbound(ids::RESOURCE_PACK_CLIENT_RESPONSE, move |w, _| {
            resource_pack_client_response(w, to_server_v2168)
        })
        .serverbound(ids::PLAYER_AUTH_INPUT, move |w, s| {
            player_auth_input(w, s, to_server_v2168)
        })
        .serverbound(ids::MOVE_PLAYER, move |w, _| {
            move_player(w, to_server_v2168)
        })
        .serverbound(ids::PLAYER_EQUIPMENT, move |w, _| {
            player_equipment(w, to_server_v2168)
        })
        .serverbound(ids::INVENTORY_TRANSACTION, move |w, _| {
            inventory_transaction(w, to_server_v2168)
        })
        .serverbound(ids::ANVIL_DAMAGE, move |w, _| {
            anvil_damage(w, to_server_v2168)
        })
        .serverbound(ids::SERVERBOUND_DIAGNOSTICS, move |w, _| {
            serverbound_diagnostics(w, to_server_v2168)
        });

    step = match sub_chunk_shape {
        Some(shape) => step.clientbound(ids::SUB_CHUNK, move |w, _| {
            sub_chunk(w, to_client_v2168, shape)
        }),
        None => step.cancel(Direction::Clientbound, ids::SUB_CHUNK),
    };

    step = step
        .clientbound(ids::SET_SCORE, move |w, state| {
            if !set_score(w, state, to_client_v2168)? {
                w.cancel();
            }
            Ok(())
        })
        .clientbound(ids::SET_SCOREBOARD_IDENTITY, move |w, state| {
            if !set_scoreboard_identity(w, state, to_client_v2168)? {
                w.cancel();
            }
            Ok(())
        });

    step = step.cancel_all(
        Direction::Clientbound,
        &[ids::PLAYER_SKIN, ids::MAP_DATA, ids::PLAYER_LOCATION],
    );
    step = step.cancel_all(Direction::Serverbound, &[ids::PLAYER_SKIN]);

    if !to_client_v2168 {
        step = step.cancel(
            Direction::Clientbound,
            ids::SERVER_PLAYER_POST_MOVE_POSITION,
        );
    }

    step
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipeline::{translate, Outcome};
    use crate::{build_registry, ConnState};

    fn run(handler: impl Fn(&mut PacketWrapper) -> Result<()>, input: &[u8]) -> Vec<u8> {
        let mut w = PacketWrapper::new(input);
        handler(&mut w).expect("handler failed");
        w.finish()
    }

    #[test]
    fn play_sound_never_asks_the_client_to_repeat_forever() {
        let mut w = Writer::new();
        w.write_string("mob.pig.say");
        w.write_varint(64);
        w.write_varint(72);
        w.write_varint(-16);
        w.write_f32_le(1.0);
        w.write_f32_le(1.0);
        w.write_bool(false);
        let v1001 = w.into_vec();

        let v2168 = run(|w| play_sound(w, true), &v1001);

        let mut r = Reader::new(&v2168);
        assert_eq!(r.read_string().expect("SoundName"), "mob.pig.say");
        for _ in 0..3 {
            r.read_varint().expect("Position");
        }
        r.read_f32_le().expect("Volume");
        r.read_f32_le().expect("Pitch");

        let loops = r.read_varint().expect("LoopCount");
        assert!(
            loops >= 0,
            "LoopCount {loops} is negative, which is the engine's \
             loop-forever sentinel: one /playsound would never stop"
        );
        if std::env::var_os("CROSSBIND_PLAYSOUND_LOOPS").is_none() {
            assert_eq!(loops, PLAY_SOUND_ONCE, "default has to be a single play");
        }

        assert!(!r.read_bool().expect("ServerSoundHandle"));
        assert!(!r.has_remaining());

        assert_eq!(run(|w| play_sound(w, false), &v2168), v1001);
    }

    #[test]
    fn structure_block_update_reaches_the_server_instead_of_being_dropped() {
        assert!(
            !downgrade().is_cancelled(Direction::Serverbound, ids::STRUCTURE_BLOCK_UPDATE),
            "cancelling this is what made structure block edits revert on exit"
        );

        let mut state = ConnState::new(1001);
        let mut w = Writer::new();
        w.write_bytes(&[0x11, 0x22, 0x33, 0x44]);
        w.write_u8(1);
        w.write_bool(true);
        w.write_bool(false);
        let v2168 = w.into_vec();

        let mut wrapper = PacketWrapper::new(&v2168);
        structure_block_update(&mut wrapper, &mut state, false).expect("handler failed");
        assert!(!wrapper.is_cancelled());
        let v1001 = wrapper.finish();
        assert_eq!(v1001, vec![0x11, 0x22, 0x33, 0x44, 0x02, 0x01, 0x00]);

        let mut wrapper = PacketWrapper::new(&v1001);
        structure_block_update(&mut wrapper, &mut state, true).expect("handler failed");
        assert_eq!(wrapper.finish(), v2168);
        assert!(state.notices.is_empty());
    }

    #[test]
    fn structure_block_update_keeps_the_length_it_was_given() {
        let mut state = ConnState::new(1001);
        for save_mode in 0..=STRUCTURE_REDSTONE_SAVE_MODE_MAX {
            for trigger in [false, true] {
                for waterlogged in [false, true] {
                    let mut w = Writer::new();
                    w.write_bytes(&[0xAA; 20]);
                    w.write_u8(save_mode);
                    w.write_bool(trigger);
                    w.write_bool(waterlogged);
                    let v2168 = w.into_vec();

                    let mut wrapper = PacketWrapper::new(&v2168);
                    structure_block_update(&mut wrapper, &mut state, false)
                        .expect("handler failed");
                    let v1001 = wrapper.finish();
                    assert_eq!(v1001.len(), v2168.len(), "save mode {save_mode}");
                    assert_eq!(&v1001[..20], &v2168[..20], "the head must be untouched");

                    let mut wrapper = PacketWrapper::new(&v1001);
                    structure_block_update(&mut wrapper, &mut state, true).expect("handler failed");
                    assert_eq!(wrapper.finish(), v2168, "save mode {save_mode}");
                }
            }
        }
        assert!(state.notices.is_empty());
    }

    #[test]
    fn structure_block_update_with_an_unrecognised_tail_is_dropped() {
        let cases: [(&[u8], bool); 4] = [
            (&[0x11, 0x22, 0x07, 0x05, 0x00], false),
            (&[0x11, 0x22, 0x09, 0x00, 0x00], false),
            (&[0x11, 0x22, 0x01, 0x00, 0x00], true),
            (&[0x00, 0x01], false),
        ];
        for (body, to_v2168) in cases {
            let mut state = ConnState::new(1001);
            let mut wrapper = PacketWrapper::new(body);
            structure_block_update(&mut wrapper, &mut state, to_v2168).expect("handler failed");
            assert!(
                wrapper.is_cancelled(),
                "a tail this shape must fall back to dropping, not to a guess: {body:02x?}"
            );
            assert_eq!(state.notices.len(), 1, "and it has to say so once");
        }
    }

    #[test]
    fn step_endpoints_are_the_documented_pair() {
        assert_eq!(downgrade().server_protocol, 1001);
        assert_eq!(downgrade().client_protocol, 2168);
        assert_eq!(upgrade().server_protocol, 2168);
        assert_eq!(upgrade().client_protocol, 1001);
    }

    #[test]
    fn bitset_and_flag_list_round_trip() {
        for set in [
            vec![],
            vec![0u32],
            vec![6, 7],
            vec![0, 34, 64],
            vec![1, 2, 3, 45],
        ] {
            let mut w = PacketWrapper::new(&[]);
            write_bitset(&mut w, &set, INPUT_FLAG_BITSET_SIZE_V1001);
            let bytes = w.finish();
            let mut w = PacketWrapper::new(&bytes);
            let decoded = read_bitset(&mut w, INPUT_FLAG_BITSET_SIZE_V1001).unwrap();
            assert_eq!(decoded, set, "bitset round trip failed for {set:?}");
        }
    }

    #[test]
    fn bitset_rejects_a_flag_past_the_end() {
        let body = [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x7F];
        let mut w = PacketWrapper::new(&body);
        assert!(read_bitset(&mut w, INPUT_FLAG_BITSET_SIZE_V1001).is_err());
    }

    fn auth_input_v1001(flags: &[u32]) -> Vec<u8> {
        let mut w = PacketWrapper::new(&[]);
        for _ in 0..2 {
            w.write::<FloatLe>(1.5);
        }
        w.write::<Vec3>((1.0, 2.0, 3.0));
        w.write::<Vec2>((0.5, 0.25));
        w.write::<FloatLe>(0.75);
        write_bitset(&mut w, flags, INPUT_FLAG_BITSET_SIZE_V1001);
        w.writer().write_uvarint(1);
        w.writer().write_uvarint(0);
        w.writer().write_uvarint(2);
        w.write::<FloatLe>(0.0);
        w.write::<FloatLe>(0.0);
        w.writer().write_uvarint64(1234);
        w.write::<Vec3>((0.0, 0.0, 0.0));
        if flags.contains(&FLAG_CLIENT_PREDICTED_VEHICLE) {
            w.write::<Vec2>((9.0, 8.0));
            w.writer().write_varint64(-77);
        }
        w.write::<Vec2>((0.0, 0.0));
        w.write::<Vec3>((0.0, 0.0, 0.0));
        w.write::<Vec2>((0.0, 0.0));
        w.finish()
    }

    #[test]
    fn auth_input_up_then_down_is_the_identity() {
        for flags in [vec![], vec![0u32, 12], vec![FLAG_CLIENT_PREDICTED_VEHICLE]] {
            let original = auth_input_v1001(&flags);
            let widened = run(player_auth_input_to_v2168, &original);
            let back = run(|w| player_auth_input_to_v1001(w, None), &widened);
            assert_eq!(back, original, "round trip failed for flags {flags:?}");
        }
    }

    #[test]
    fn move_delta_up_then_down_is_the_identity() {
        for flags in [
            0u16,
            MOVE_DELTA_HAS_X | MOVE_DELTA_HAS_Z,
            MOVE_DELTA_HAS_X | MOVE_DELTA_HAS_Y | MOVE_DELTA_HAS_Z | MOVE_DELTA_ON_GROUND,
            MOVE_DELTA_HAS_ROT_Y | MOVE_DELTA_FORCE_MOVE,
        ] {
            let mut w = PacketWrapper::new(&[]);
            w.write::<UVarInt64>(42);
            w.writer().write_u16_le(flags);
            for bit in [MOVE_DELTA_HAS_X, MOVE_DELTA_HAS_Y, MOVE_DELTA_HAS_Z] {
                if flags & bit != 0 {
                    w.writer().write_f32_le(3.5);
                }
            }
            for bit in [
                MOVE_DELTA_HAS_ROT_X,
                MOVE_DELTA_HAS_ROT_Y,
                MOVE_DELTA_HAS_ROT_Z,
            ] {
                if flags & bit != 0 {
                    w.writer().write_u8(200);
                }
            }
            let original = w.finish();

            let widened = run(|w| move_delta_actor(w, true), &original);
            let back = run(|w| move_delta_actor(w, false), &widened);
            assert_eq!(back, original, "round trip failed for flags {flags:#x}");
        }
    }

    #[test]
    fn full_chunk_data_round_trips_each_request_mode() {
        for count in [3u32, SUB_CHUNK_MODE_LIMITED, SUB_CHUNK_MODE_LIMITLESS] {
            for cache in [false, true] {
                let mut w = PacketWrapper::new(&[]);
                w.write::<VarInt>(4);
                w.write::<VarInt>(-9);
                w.write::<VarInt>(0);
                w.writer().write_uvarint(count);
                if count == SUB_CHUNK_MODE_LIMITED {
                    w.writer().write_u16_le(7);
                }
                w.write::<Bool>(cache);
                if cache {
                    w.writer().write_count(2);
                    w.write::<UInt64Le>(11);
                    w.write::<UInt64Le>(22);
                }
                w.writer().write_count(3);
                w.writer().write_bytes(&[1, 2, 3]);
                let original = w.finish();

                let widened = run(|w| full_chunk_data(w, true), &original);
                let back = run(|w| full_chunk_data(w, false), &widened);
                assert_eq!(back, original, "count {count:#x} cache {cache}");
            }
        }
    }

    #[test]
    fn pack_response_round_trips_and_shifts_the_enum() {
        let mut w = PacketWrapper::new(&[]);
        w.write::<Byte>(2);
        w.write::<UShortLe>(2);
        w.write::<Str>("a_1.0.0".into());
        w.write::<Str>("b_2.0.0".into());
        let original = w.finish();

        let widened = run(|w| resource_pack_client_response(w, true), &original);
        assert_eq!(widened[0], 1, "v2168 numbering is one lower");

        let back = run(|w| resource_pack_client_response(w, false), &widened);
        assert_eq!(back, original);
    }

    #[test]
    fn pack_response_without_packs_still_round_trips() {
        let mut w = PacketWrapper::new(&[]);
        w.write::<Byte>(4);
        w.write::<UShortLe>(0);
        let original = w.finish();
        let widened = run(|w| resource_pack_client_response(w, true), &original);
        let back = run(|w| resource_pack_client_response(w, false), &widened);
        assert_eq!(back, original);
    }

    #[test]
    fn pack_response_rejects_an_out_of_range_value() {
        let body = [0u8, 0, 0];
        let mut w = PacketWrapper::new(&body);
        assert!(resource_pack_client_response(&mut w, true).is_err());
    }

    #[test]
    fn item_bearing_packets_round_trip() {
        let mut w = PacketWrapper::new(&[]);
        w.write::<UVarInt64>(7);
        for i in 0..5 {
            w.write::<ItemInstanceV975>(Item {
                network_id: 300 + i,
                count: 1,
                aux_value: 0,
                has_net_id: true,
                stack_net_id: i,
                net_id_variant: 0,
                block_runtime_id: 0,
                user_data: Vec::new(),
            });
        }
        let original = w.finish();
        let widened = run(|w| mob_armor_equipment(w, true), &original);
        assert_eq!(widened.len(), original.len() - 5, "one byte lost per item");
        let back = run(|w| mob_armor_equipment(w, false), &widened);
        assert_eq!(back, original);
    }

    fn join_info_v1001() -> Vec<u8> {
        let mut w = PacketWrapper::new(&[]);
        w.write::<Bool>(true);
        w.write::<Bool>(true);
        w.write::<Uuid>(MceUuid { msb: 1, lsb: 2 });
        w.write::<Str>("experience".into());
        w.write::<Uuid>(MceUuid { msb: 3, lsb: 4 });
        w.write::<Str>("world".into());
        w.write::<Str>("creator".into());
        w.write::<Uuid>(MceUuid { msb: 5, lsb: 6 });
        w.write::<Str>("scenario".into());
        w.write::<Str>("server".into());
        w.write::<Bool>(true);
        w.write::<Str>("store".into());
        w.write::<Str>("store name".into());
        w.write::<Bool>(true);
        w.write::<Optional<Str>>(None);
        w.write::<Optional<Str>>(None);
        w.write::<Str>("rich".into());
        w.finish()
    }

    #[test]
    fn join_info_up_then_down_is_the_identity() {
        let original = join_info_v1001();
        let widened = run(|w| server_join_information(w, true), &original);
        assert_ne!(widened, original, "the optionals must change the encoding");
        let back = run(|w| server_join_information(w, false), &widened);
        assert_eq!(back, original);
    }

    #[test]
    fn an_absent_join_info_leaves_the_trailing_ids_alone() {
        let ids: [u8; 4] = [0, 0, 0, 0];

        for inner in [vec![0u8], vec![1u8, 0, 0, 0]] {
            let mut original = inner.clone();
            original.extend_from_slice(&ids);

            for to_v2168 in [true, false] {
                let mut w = PacketWrapper::new(&original);
                server_join_information(&mut w, to_v2168).expect("join info");
                w.passthrough_all();
                let out = w.finish();
                assert_eq!(
                    out, original,
                    "absent join info must be byte-identical in both directions"
                );
            }

            let mut w = PacketWrapper::new(&original);
            server_join_information(&mut w, true).expect("join info");
            assert_eq!(
                w.finish().len(),
                inner.len() + ids.len(),
                "handler must not consume the trailing IDs"
            );
        }
    }

    #[test]
    fn creative_content_round_trips_including_an_air_icon() {
        let mut w = PacketWrapper::new(&[]);
        w.write::<UVarInt>(2);
        for (category, air) in [(0i32, true), (3, false)] {
            w.write::<IntLe>(category);
            w.write::<Str>("itemGroup.name.planks".into());
            w.write::<NetworkItemInstanceDescriptor>(if air {
                Item::default()
            } else {
                Item {
                    network_id: 5,
                    count: 1,
                    aux_value: 0,
                    block_runtime_id: 7,
                    ..Item::default()
                }
            });
        }
        w.write::<UVarInt>(1);
        w.write::<UVarInt>(101);
        w.write::<NetworkItemInstanceDescriptor>(Item {
            network_id: 9,
            count: 64,
            aux_value: 2,
            block_runtime_id: 0,
            ..Item::default()
        });
        w.write::<UVarInt>(1);
        let original = w.finish();

        let widened = run(|w| creative_content(w, true), &original);

        assert_eq!(
            widened.len(),
            original.len() - 2 * 3 + 5,
            "category narrowing and air widening must both show up in the length"
        );

        let back = run(|w| creative_content(w, false), &widened);
        assert_eq!(back, original);
    }

    fn v1001_stack(w: &mut Writer, net_id: i32) {
        w.write_i16_le(261);
        w.write_u16_le(1);
        w.write_uvarint(0);
        w.write_bool(true);
        w.write_uvarint(0);
        w.write_varint(net_id);
        w.write_uvarint(0);
        w.write_count(0);
    }

    #[test]
    fn inventory_transaction_use_item_reaches_the_server() {
        let mut w = Writer::new();
        w.write_varint(0);
        w.write_bool(false);
        w.write_bool(true);
        w.write_uvarint(TRANSACTION_ITEM_USE);
        w.write_bool(true);
        w.write_count(1);
        w.write_uvarint(0);
        w.write_bool(true);
        w.write_bool(true);
        w.write_i8(-1);
        w.write_bool(true);
        w.write_bool(false);
        w.write_uvarint(4);
        v1001_stack(&mut w, 5);
        v1001_stack(&mut w, 6);
        w.write_varint(0);
        w.write_u8(0);
        w.write_varint(10);
        w.write_varint(64);
        w.write_varint(-7);
        w.write_u8(1);
        w.write_varint(3);
        v1001_stack(&mut w, 7);
        for v in [0.5f32, 65.0, -6.5, 0.5, 1.0, 0.5] {
            w.write_f32_le(v);
        }
        w.write_uvarint(134);
        w.write_u8(0);
        w.write_u8(0);
        let original = w.into_vec();

        let widened = run(|w| inventory_transaction(w, true), &original);
        assert_eq!(widened.len(), original.len() - 3);

        let back = run(|w| inventory_transaction(w, false), &widened);
        assert_eq!(back, original);
    }

    #[test]
    fn inventory_transaction_copies_the_legacy_slot_block() {
        let mut w = Writer::new();
        w.write_varint(-4);
        w.write_bool(true);
        w.write_count(1);
        w.write_u8(12);
        w.write_count(3);
        for slot in [0u8, 1, 2] {
            w.write_u8(slot);
        }
        w.write_bool(true);
        w.write_uvarint(TRANSACTION_ITEM_RELEASE);
        w.write_bool(true);
        w.write_count(0);
        w.write_varint(1);
        w.write_varint(0);
        v1001_stack(&mut w, 9);
        for v in [1.0f32, 2.0, 3.0] {
            w.write_f32_le(v);
        }
        let original = w.into_vec();

        let widened = run(|w| inventory_transaction(w, true), &original);
        assert_eq!(widened.len(), original.len() - 1);

        let back = run(|w| inventory_transaction(w, false), &widened);
        assert_eq!(back, original);
    }

    #[test]
    fn inventory_transaction_is_translated_not_cancelled() {
        let mut w = Writer::new();
        w.write_varint(0);
        w.write_bool(false);
        w.write_bool(true);
        w.write_uvarint(0);
        w.write_bool(true);
        w.write_count(0);
        let body = w.into_vec();

        let registry = build_registry(1001);
        let mut state = ConnState::new(1001);
        state.client_protocol = 2168;
        for direction in [Direction::Serverbound, Direction::Clientbound] {
            let result = translate(
                &registry,
                &mut state,
                direction,
                ids::INVENTORY_TRANSACTION,
                &body,
            );
            assert_eq!(
                result.outcome,
                Outcome::Rewritten(body.clone()),
                "{} InventoryTransaction must be translated",
                direction.as_str(),
            );
        }
    }

    fn v1001_metadata(w: &mut Writer) {
        w.write_count(2);
        w.write_uvarint(0);
        w.write_uvarint(7);
        w.write_varint64(-1);
        w.write_uvarint(4);
        w.write_uvarint(4);
        w.write_string("steve");
    }

    #[test]
    fn set_actor_data_gains_one_byte_per_metadata_entry() {
        let mut w = Writer::new();
        w.write_uvarint64(42);
        v1001_metadata(&mut w);
        w.write_count(0);
        w.write_count(0);
        w.write_uvarint64(9001);
        let original = w.into_vec();

        let widened = run(|w| set_actor_data(w, true), &original);
        assert_eq!(widened.len(), original.len() + 2);

        let back = run(|w| set_actor_data(w, false), &widened);
        assert_eq!(back, original);
    }

    #[test]
    fn set_actor_data_keeps_its_tail_intact() {
        let mut w = Writer::new();
        w.write_uvarint64(1);
        v1001_metadata(&mut w);
        w.write_bytes(&[0xDE, 0xAD, 0xBE, 0xEF]);
        let original = w.into_vec();

        let widened = run(|w| set_actor_data(w, true), &original);
        assert_eq!(&widened[widened.len() - 4..], &[0xDE, 0xAD, 0xBE, 0xEF]);
    }

    #[test]
    fn add_item_actor_converts_both_the_stack_and_the_metadata() {
        let mut w = Writer::new();
        w.write_varint64(-5);
        w.write_uvarint64(5);
        w.write_varint(261);
        w.write_u16_le(1);
        w.write_uvarint(0);
        w.write_bool(true);
        w.write_varint(77);
        w.write_varint(0);
        w.write_count(0);
        for v in [1.0f32, 64.0, -3.0, 0.0, 0.1, 0.0] {
            w.write_f32_le(v);
        }
        v1001_metadata(&mut w);
        w.write_bool(false);
        let original = w.into_vec();

        let widened = run(|w| add_item_actor(w, true), &original);
        let back = run(|w| add_item_actor(w, false), &widened);
        assert_eq!(back, original);

        assert_eq!(widened.len(), original.len() + 2);
    }

    #[test]
    fn add_item_actor_round_trips_an_air_stack() {
        let mut w = Writer::new();
        w.write_varint64(-5);
        w.write_uvarint64(5);
        w.write_varint(0);
        for v in [0.0f32; 6] {
            w.write_f32_le(v);
        }
        w.write_count(0);
        w.write_bool(false);
        let original = w.into_vec();

        let widened = run(|w| add_item_actor(w, true), &original);
        assert_eq!(widened.len(), original.len() + 7, "air widens 1 B -> 8 B");
        let back = run(|w| add_item_actor(w, false), &widened);
        assert_eq!(back, original);
    }

    #[test]
    fn add_player_walks_to_its_metadata() {
        let mut w = Writer::new();
        w.write_u64_le(1);
        w.write_u64_le(2);
        w.write_string("RSxiaotong");
        w.write_uvarint64(7);
        w.write_string("");
        for v in [0.0f32; 6] {
            w.write_f32_le(v);
        }
        for v in [0.0f32; 3] {
            w.write_f32_le(v);
        }
        w.write_varint(0);
        w.write_varint(1);
        v1001_metadata(&mut w);
        w.write_bytes(&[0x00, 0x00]);
        let original = w.into_vec();

        let widened = run(|w| add_player(w, true), &original);
        let back = run(|w| add_player(w, false), &widened);
        assert_eq!(back, original);
    }

    #[test]
    fn add_actor_steps_over_the_attribute_list() {
        let mut w = Writer::new();
        w.write_varint64(-1);
        w.write_uvarint64(1);
        w.write_string("minecraft:pig");
        for v in [0.0f32; 6] {
            w.write_f32_le(v);
        }
        for v in [0.0f32; 4] {
            w.write_f32_le(v);
        }
        w.write_count(1);
        w.write_string("minecraft:health");
        for v in [0.0f32, 10.0, 10.0] {
            w.write_f32_le(v);
        }
        v1001_metadata(&mut w);
        let original = w.into_vec();

        let widened = run(|w| add_actor(w, true), &original);
        assert_eq!(widened.len(), original.len() + 2);
        let back = run(|w| add_actor(w, false), &widened);
        assert_eq!(back, original);
    }

    #[test]
    fn inventory_transaction_is_cancelled_in_neither_direction() {
        let step = downgrade();
        assert!(!step.is_cancelled(Direction::Clientbound, ids::INVENTORY_TRANSACTION));
        assert!(!step.is_cancelled(Direction::Serverbound, ids::INVENTORY_TRANSACTION));
    }

    fn v1001_entry(w: &mut Writer, result: u8, payload: &[u8], height_map: bool) {
        w.write_i8(0);
        w.write_i8(1);
        w.write_i8(-1);
        w.write_u8(result);
        if result != SUB_CHUNK_RESULT_SUCCESS_ALL_AIR {
            w.write_count(payload.len());
            w.write_bytes(payload);
        }
        if height_map {
            w.write_u8(HEIGHT_MAP_HAS_DATA);
            w.write_bytes(&[3u8; HEIGHT_MAP_LEN]);
        } else {
            w.write_u8(0);
        }
        w.write_u8(0);
        w.write_u64_le(0xDEAD_BEEF_CAFE_F00D);
    }

    fn v1001_sub_chunk(entries: &[(u8, Vec<u8>, bool)]) -> Vec<u8> {
        let mut w = Writer::new();
        w.write_bool(true);
        w.write_varint(1000);
        w.write_varint(-5);
        w.write_varint(2);
        w.write_varint(-4);
        w.write_u32_le(entries.len() as u32);
        for (result, payload, height_map) in entries {
            v1001_entry(&mut w, *result, payload, *height_map);
        }
        w.into_vec()
    }

    #[test]
    fn sub_chunk_round_trips_a_mixed_entry_list() {
        let original = v1001_sub_chunk(&[
            (1, vec![9; 40], true),
            (SUB_CHUNK_RESULT_SUCCESS_ALL_AIR, vec![], false),
            (1, vec![7; 12], false),
        ]);

        let widened = run(|w| sub_chunk(w, true, SubChunkShape::DOCUMENTED), &original);
        let back = run(|w| sub_chunk(w, false, SubChunkShape::DOCUMENTED), &widened);
        assert_eq!(back, original);
    }

    #[test]
    fn sub_chunk_position_widens_and_the_count_narrows() {
        let original = v1001_sub_chunk(&[(1, vec![1, 2, 3], false)]);
        let widened = run(|w| sub_chunk(w, true, SubChunkShape::DOCUMENTED), &original);

        assert_eq!(widened.len(), original.len() + 9 - 3 + 4);

        assert_eq!(
            &widened[3..15],
            &[0xFB, 0xFF, 0xFF, 0xFF, 0x02, 0x00, 0x00, 0x00, 0xFC, 0xFF, 0xFF, 0xFF,]
        );
        assert_eq!(widened[15], 0x01, "entry count is a uvarint now");
    }

    #[test]
    fn sub_chunk_without_the_cache_has_no_blob_hash_on_the_v1001_side() {
        let mut w = Writer::new();
        w.write_bool(false);
        w.write_varint(0);
        w.write_varint(0);
        w.write_varint(0);
        w.write_varint(0);
        w.write_u32_le(2);
        for _ in 0..2 {
            w.write_i8(0);
            w.write_i8(0);
            w.write_i8(0);
            w.write_u8(SUB_CHUNK_RESULT_SUCCESS_ALL_AIR);
            w.write_count(2);
            w.write_bytes(&[4, 5]);
            w.write_u8(0);
            w.write_u8(0);
        }
        let original = w.into_vec();

        let widened = run(|w| sub_chunk(w, true, SubChunkShape::DOCUMENTED), &original);
        let back = run(|w| sub_chunk(w, false, SubChunkShape::DOCUMENTED), &widened);
        assert_eq!(back, original);
    }

    #[test]
    fn every_sub_chunk_shape_round_trips() {
        let original = v1001_sub_chunk(&[
            (1, vec![9; 40], true),
            (SUB_CHUNK_RESULT_SUCCESS_ALL_AIR, vec![], false),
            (1, vec![7; 12], false),
        ]);
        for type_bytes in [true, false] {
            for announce_blob_hash in [true, false] {
                for zero_blob_hash in [true, false] {
                    for empty_payload in [true, false] {
                        let shape = SubChunkShape {
                            type_bytes,
                            announce_blob_hash,
                            zero_blob_hash,
                            empty_payload,
                            strip_content: false,
                        };
                        let widened = run(|w| sub_chunk(w, true, shape), &original);
                        let back = run(|w| sub_chunk(w, false, shape), &widened);
                        if announce_blob_hash {
                            assert_eq!(back, original, "shape {shape:?} is not its own inverse");
                        } else {
                            assert_eq!(
                                back.len(),
                                original.len(),
                                "shape {shape:?} moved something other than the hash"
                            );
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn air_mode_empties_every_entry_and_keeps_the_framing() {
        let original = v1001_sub_chunk(&[
            (1, vec![9; 40], true),
            (SUB_CHUNK_RESULT_SUCCESS_ALL_AIR, vec![], false),
            (1, vec![7; 12], false),
        ]);
        let air = SubChunkShape {
            strip_content: true,
            ..SubChunkShape::DOCUMENTED
        };
        let stripped = run(|w| sub_chunk(w, true, air), &original);

        assert_eq!(stripped.len(), 3 + 12 + 1 + 3 * 10);
        for entry in 0..3 {
            let at = 16 + entry * 10;
            assert_eq!(
                &stripped[at..at + 10],
                &[
                    stripped[at],
                    stripped[at + 1],
                    stripped[at + 2],
                    SUB_CHUNK_RESULT_SUCCESS_ALL_AIR,
                    0,
                    HEIGHT_MAP_NONE,
                    0,
                    HEIGHT_MAP_NONE,
                    0,
                    0,
                ]
            );
        }
    }

    #[test]
    fn sub_chunk_matches_the_captured_packet_byte_for_byte() {
        let entry = |y: u8| [0xfe, y, 0x03, 0x06, 0x02, 0x02, 0, 0, 0, 0, 0, 0, 0, 0];
        let mut input = vec![0x01, 0xd0, 0x0f, 0x07, 0x00, 0x09, 0x02, 0x00, 0x00, 0x00];
        input.extend_from_slice(&entry(0xe0));
        input.extend_from_slice(&entry(0xe1));

        let widened = run(|w| sub_chunk(w, true, SubChunkShape::DOCUMENTED), &input);

        let out_entry = |y: u8| {
            [
                0xfe, y, 0x03, 0x06, 0x00, 0x02, 0x00, 0x02, 0x00, 0x01, 0, 0, 0, 0, 0, 0, 0, 0,
            ]
        };
        let mut expected = vec![
            0x01, 0xd0, 0x0f, 0xfc, 0xff, 0xff, 0xff, 0x00, 0x00, 0x00, 0x00, 0xfb, 0xff, 0xff,
            0xff, 0x02,
        ];
        expected.extend_from_slice(&out_entry(0xe0));
        expected.extend_from_slice(&out_entry(0xe1));
        assert_eq!(widened, expected);

        let lean = run(
            |w| {
                sub_chunk(
                    w,
                    true,
                    SubChunkShape {
                        type_bytes: false,
                        announce_blob_hash: true,
                        zero_blob_hash: false,
                        empty_payload: true,
                        strip_content: false,
                    },
                )
            },
            &input,
        );
        assert_eq!(lean.len(), widened.len() - 2 * (2 + 8));
    }

    #[test]
    fn the_default_shape_announces_a_real_hash_and_hides_a_zero_one() {
        assert!(
            SubChunkShape::DEFAULT.announce_blob_hash,
            "a v2168 client leaves the world when an entry with a payload \
             carries no blob hash; see the capture in the devdocs"
        );
        assert!(!SubChunkShape::DEFAULT.zero_blob_hash);
        assert!(SubChunkShape::DEFAULT.type_bytes);
        assert!(!SubChunkShape::DEFAULT.strip_content);

        let original = v1001_sub_chunk(&[(SUB_CHUNK_RESULT_SUCCESS_ALL_AIR, vec![], false)]);
        let kept = run(|w| sub_chunk(w, true, SubChunkShape::DEFAULT), &original);
        assert_eq!(kept[kept.len() - 9], 0x01, "a real hash stays present");

        let mut all_air = original.clone();
        let len = all_air.len();
        all_air[len - 8..].fill(0);
        let dropped = run(|w| sub_chunk(w, true, SubChunkShape::DEFAULT), &all_air);
        assert_eq!(*dropped.last().unwrap(), 0x00, "a zero hash goes absent");
        assert_eq!(dropped.len(), kept.len() - 8);
    }

    #[test]
    fn an_empty_payload_is_written_absent_by_default() {
        assert!(!SubChunkShape::DEFAULT.empty_payload);
        assert!(SubChunkShape::SUPPRESS_ZERO_HASH.empty_payload);

        let mut w = Writer::new();
        w.write_bool(true);
        w.write_varint(1000);
        w.write_varint(0);
        w.write_varint(0);
        w.write_varint(0);
        w.write_u32_le(1);
        w.write_i8(-114);
        w.write_i8(12);
        w.write_i8(-114);
        w.write_u8(SUB_CHUNK_RESULT_LEVEL_CHUNK_DOESNT_EXIST);
        w.write_count(0);
        w.write_u8(HEIGHT_MAP_NONE);
        w.write_u8(HEIGHT_MAP_NONE);
        w.write_u64_le(0);
        let original = w.into_vec();

        let absent = run(|w| sub_chunk(w, true, SubChunkShape::DEFAULT), &original);
        let announced = run(|w| sub_chunk(w, true, SubChunkShape::SUPPRESS_ZERO_HASH), &original);

        assert_eq!(
            absent.len(),
            announced.len() - 1,
            "the only difference is the zero-length byte array behind the presence bool"
        );
        assert_eq!(
            absent[20], 0x00,
            "a v975 chunk-doesn't-exist entry hands over no sub-chunk, so the optional is None"
        );
        assert_eq!(announced[20], 0x01);
        assert_eq!(absent[19], SUB_CHUNK_RESULT_LEVEL_CHUNK_DOESNT_EXIST);

        let back = run(|w| sub_chunk(w, false, SubChunkShape::DEFAULT), &absent);
        assert_eq!(back, original, "None widens back into the empty array v975 wrote");
    }

    #[test]
    fn a_real_payload_survives_the_empty_payload_rule() {
        let original = v1001_sub_chunk(&[(1, vec![9; 40], true), (1, vec![7; 12], false)]);
        let widened = run(|w| sub_chunk(w, true, SubChunkShape::DEFAULT), &original);
        let back = run(|w| sub_chunk(w, false, SubChunkShape::DEFAULT), &widened);
        assert_eq!(back, original);
    }

    #[test]
    fn mode_e_drops_every_hash_and_is_not_the_default() {
        assert!(!SubChunkShape::NO_BLOB_HASH.announce_blob_hash);
        assert_ne!(
            SubChunkShape::DEFAULT,
            SubChunkShape::NO_BLOB_HASH,
            "mode e is opt-in: it costs 8 bytes per entry and a v2168 client \
             refuses the result"
        );

        let original = v1001_sub_chunk(&[(SUB_CHUNK_RESULT_SUCCESS_ALL_AIR, vec![], false)]);
        let lean = run(|w| sub_chunk(w, true, SubChunkShape::NO_BLOB_HASH), &original);
        let announced = run(|w| sub_chunk(w, true, SubChunkShape::DEFAULT), &original);

        assert_eq!(*lean.last().unwrap(), 0x00);
        assert_eq!(
            lean.len(),
            announced.len() - 8,
            "8 bytes per entry, which is what the -617 in the capture was"
        );

        let back = run(|w| sub_chunk(w, false, SubChunkShape::NO_BLOB_HASH), &lean);
        assert_eq!(
            back.len(),
            original.len(),
            "only the hash is lost; the framing has to survive"
        );
    }

    fn v2168_auth_input(
        flags: &[u32],
        block_actions: &[(i32, (i32, i32, i32), i32)],
        stack_request: bool,
    ) -> Vec<u8> {
        let mut w = Writer::new();
        w.write_f32_le(1.0);
        w.write_f32_le(2.0);
        for v in [3.0f32, 4.0, 5.0] {
            w.write_f32_le(v);
        }
        for v in [0.0f32, 0.0] {
            w.write_f32_le(v);
        }
        w.write_f32_le(6.0);

        w.write_bool(true);
        w.write_count(flags.len());
        for f in flags {
            w.write_varint(*f as i32);
        }

        w.write_uvarint(1);
        w.write_uvarint(0);
        w.write_varint(0);
        w.write_f32_le(7.0);
        w.write_f32_le(8.0);
        w.write_uvarint64(1234);
        for v in [0.0f32, 0.0, 0.0] {
            w.write_f32_le(v);
        }

        w.write_bool(false);
        if stack_request {
            w.write_bool(true);
            w.write_bool(true);
            w.write_count(0);
        } else {
            w.write_bool(false);
        }
        if block_actions.is_empty() {
            w.write_bool(false);
        } else {
            w.write_bool(true);
            w.write_bool(true);
            w.write_count(block_actions.len());
            for (action, pos, face) in block_actions {
                w.write_varint(*action);
                w.write_varint(pos.0);
                w.write_varint(pos.1);
                w.write_varint(pos.2);
                w.write_varint(*face);
            }
        }
        w.write_bool(false);
        w.write_bool(false);

        w.write_bytes(&[0xAA, 0xBB]);
        w.into_vec()
    }

    #[test]
    fn block_actions_reach_the_server_and_set_their_own_flag() {
        let input = v2168_auth_input(
            &[],
            &[
                (PLAYER_ACTION_START_BREAK, (10, 64, -3), 1),
                (8, (0, 0, 0), 0),
            ],
            false,
        );
        let out = run(|w| player_auth_input_to_v1001(w, None), &input);

        let mut r = Reader::new(&out);
        for _ in 0..8 {
            r.read_f32_le().unwrap();
        }
        let mut set = Vec::new();
        let mut base = 0u32;
        loop {
            let byte = r.read_u8().unwrap();
            for bit in 0..7u32 {
                if byte & (1 << bit) != 0 {
                    set.push(base + bit);
                }
            }
            if byte & 0x80 == 0 {
                break;
            }
            base += 7;
        }
        assert_eq!(set, vec![FLAG_PERFORM_BLOCK_ACTIONS]);

        assert_eq!(r.read_uvarint().unwrap(), 1);
        assert_eq!(r.read_uvarint().unwrap(), 0);
        assert_eq!(r.read_uvarint().unwrap(), 0);
        r.read_f32_le().unwrap();
        r.read_f32_le().unwrap();
        assert_eq!(r.read_uvarint64().unwrap(), 1234);
        for _ in 0..3 {
            r.read_f32_le().unwrap();
        }

        assert_eq!(r.read_varint().unwrap(), 2);
        assert_eq!(r.read_varint().unwrap(), PLAYER_ACTION_START_BREAK);
        assert_eq!(BlockPos::read(&mut r).unwrap(), (10, 64, -3));
        assert_eq!(r.read_varint().unwrap(), 1);
        assert_eq!(r.read_varint().unwrap(), 8);
        assert_eq!(r.read_bytes(2).unwrap(), &[0xAA, 0xBB], "tail survives");
        assert_eq!(r.remaining(), 0);
    }

    #[test]
    fn a_flag_without_its_payload_is_cleared() {
        let input = v2168_auth_input(&[FLAG_PERFORM_BLOCK_ACTIONS], &[], false);
        let out = run(|w| player_auth_input_to_v1001(w, None), &input);

        let mut r = Reader::new(&out);
        for _ in 0..8 {
            r.read_f32_le().unwrap();
        }
        assert_eq!(r.read_u8().unwrap(), 0, "no flags, one empty bitset group");
        assert_eq!(r.read_uvarint().unwrap(), 1);
    }

    #[test]
    fn a_tick_carrying_an_item_stack_request_is_dropped() {
        let input = v2168_auth_input(&[], &[], true);
        let mut wrapper = PacketWrapper::new(&input);
        player_auth_input_to_v1001(&mut wrapper, None).expect("handler failed");
        assert!(wrapper.is_cancelled());
    }

    #[test]
    fn dimension_data_gains_and_loses_the_pack_id() {
        let mut w = PacketWrapper::new(&[]);
        w.write::<UVarInt>(1);
        w.write::<Str>("overworld".into());
        for v in [320i32, -64, 0, 0] {
            w.write::<VarInt>(v);
        }
        let original = w.finish();
        let widened = run(|w| dimension_data(w, true), &original);
        assert_eq!(widened.len(), original.len() + 16);
        let back = run(|w| dimension_data(w, false), &widened);
        assert_eq!(back, original);
    }

    fn v975_start_game_with_a_populated_tail() -> Vec<u8> {
        let mut w = Writer::new();
        w.write_varint64(-42);
        w.write_uvarint64(42);
        w.write_varint(1);
        for v in [1.5f32, 64.0, -3.25, 12.0, 90.0] {
            w.write_f32_le(v);
        }

        let mut ls = LevelSettings {
            seed: 0,
            spawn_biome_type: 0,
            spawn_biome_name: String::new(),
            spawn_dimension: 0,
            generator: 1,
            game_type: 0,
            is_hardcore: false,
            game_difficulty: 2,
            default_spawn_x: 8,
            default_spawn_y: 64,
            default_spawn_z: -8,
            achievements_disabled: true,
            editor_world_type: 0,
            created_in_editor: false,
            exported_from_editor: false,
            day_cycle_stop_time: 0,
            education_edition_offer: 0,
            education_features_enabled: false,
            education_product_id: String::new(),
            rain_level: 0.0,
            lightning_level: 0.0,
            has_confirmed_platform_locked_content: false,
            multiplayer_intended: true,
            lan_broadcasting_intended: true,
            xbox_live_broadcast_setting: 0,
            platform_broadcast_setting: 0,
            commands_enabled: true,
            texture_packs_required: false,
            game_rules: Vec::new(),
            experiments: Vec::new(),
            ever_toggled: false,
            has_bonus_chest: false,
            start_with_map: false,
            player_permissions: 1,
            server_chunk_tick_range: 12,
            has_locked_behavior_pack: false,
            has_locked_resource_pack: false,
            is_from_locked_template: false,
            use_msa_gamertags_only: false,
            created_from_template: false,
            template_with_locked_settings: false,
            only_spawn_v1_villagers: false,
            persona_disabled: false,
            custom_skins_disabled: false,
            emote_chat_muted: false,
            base_game_version: "1.26.20".into(),
            limited_world_width: 0,
            limited_world_depth: 0,
            nether_type: true,
            edu_shared_uri_button_name: String::new(),
            edu_shared_uri_link_uri: String::new(),
            override_force_experimental: None,
            chat_restriction_level: 0,
            disable_player_interactions: false,
        };
        ls.game_rules = vec![
            GameRule {
                name: "showcoordinates".into(),
                editable: true,
                value: GameRuleValue::Bool(true),
            },
            GameRule {
                name: "randomtickspeed".into(),
                editable: true,
                value: GameRuleValue::Int(3),
            },
        ];
        LevelSettingsV944::write(&mut w, &ls);

        w.write_string("level-id");
        w.write_string("Bedrock level");
        w.write_string("");
        w.write_bool(false);
        w.write_varint(0);
        w.write_bool(true);
        w.write_i64_le(6000);
        w.write_varint(0);
        w.write_uvarint(0);
        w.write_string("");
        w.write_bool(true);
        w.write_string("1.26.20");
        w.write_bytes(&[0x00]);
        w.write_i64_le(0);
        w.write_u64_le(0);
        w.write_u64_le(0);
        w.write_bool(false);
        w.write_bool(true);
        w.write_bool(true);

        w.write_bytes(&join_info_v1001());
        w.write_string("server-id");
        w.write_string("scenario-id");
        w.write_string("world-id");
        w.write_string("owner-id");
        w.into_vec()
    }

    #[test]
    fn start_game_survives_the_v975_to_v2168_chain() {
        let original = v975_start_game_with_a_populated_tail();

        let registry = build_registry(975);
        let mut state = ConnState::new(975);
        state.client_protocol = 2168;
        let up = translate(
            &registry,
            &mut state,
            Direction::Clientbound,
            ids::START_GAME,
            &original,
        );
        let widened = match up.outcome {
            Outcome::Rewritten(body) => body,
            other => panic!("StartGame must be rewritten toward v2168, got {other:?}"),
        };

        let registry = build_registry(2168);
        let mut state = ConnState::new(2168);
        state.client_protocol = 975;
        let down = translate(
            &registry,
            &mut state,
            Direction::Clientbound,
            ids::START_GAME,
            &widened,
        );
        let back = match down.outcome {
            Outcome::Rewritten(body) => body,
            other => panic!("StartGame must be rewritten toward v975, got {other:?}"),
        };

        assert_eq!(
            back, original,
            "a StartGame that goes 975 -> 1001 -> 2168 and back must be unchanged"
        );
    }

    #[test]
    fn a_v2168_client_reaches_a_v1001_server() {
        let registry = build_registry(1001);
        let chain = registry.chain(2168).expect("no chain to v2168");
        assert_eq!(chain.len(), 1, "v1001 <-> v2168 must be a single hop");
    }

    #[test]
    fn the_v2168_only_packet_is_dropped_toward_an_older_client() {
        let registry = build_registry(2168);
        let mut state = ConnState::new(2168);
        state.client_protocol = 1001;
        let result = translate(
            &registry,
            &mut state,
            Direction::Clientbound,
            ids::SERVER_PLAYER_POST_MOVE_POSITION,
            &[1, 2, 3],
        );
        assert_eq!(result.outcome, Outcome::Drop);
    }
}
