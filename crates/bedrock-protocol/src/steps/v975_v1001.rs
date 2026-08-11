use bedrock_codec::prelude::*;

use crate::connection::ConnState;
use crate::direction::Direction;
use crate::packet_ids::ids;
use crate::sound_events::{id_to_name, name_to_id};
use crate::translator::Translator;

const CONTAINER_INVENTORY: u32 = 0;
const WORLD_INTERACTION: u32 = 2;
const NON_IMPLEMENTED_FEATURE_TODO: u32 = 99999;

const ITEM_USE_TRANSACTION: u32 = 2;
const ITEM_USE_ON_ENTITY_TRANSACTION: u32 = 3;
const ITEM_RELEASE_TRANSACTION: u32 = 4;

const BOSS_SHOW: u32 = 0;
const BOSS_REGISTER_PLAYER: u32 = 1;
const BOSS_HIDE: u32 = 2;
const BOSS_UNREGISTER_PLAYER: u32 = 3;
const BOSS_HEALTH_PERCENTAGE: u32 = 4;
const BOSS_TITLE: u32 = 5;
const BOSS_APPEARANCE_PROPERTIES: u32 = 6;
const BOSS_TEXTURE: u32 = 7;
const BOSS_REQUEST: u32 = 8;

fn start_game(w: &mut PacketWrapper, to_v1001: bool) -> Result<()> {
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

fn convert_item(w: &mut PacketWrapper, to_v1001: bool) -> Result<()> {
    if to_v1001 {
        w.map::<ItemInstance, ItemInstanceV975>()?;
    } else {
        w.map::<ItemInstanceV975, ItemInstance>()?;
    }
    Ok(())
}

fn inventory_content(w: &mut PacketWrapper, to_v1001: bool) -> Result<()> {
    w.passthrough::<UVarInt>()?;

    let content_count = w.passthrough::<UVarInt>()?;
    for _ in 0..content_count {
        convert_item(w, to_v1001)?;
    }

    w.passthrough::<Byte>()?;
    let has_dynamic = w.passthrough::<Bool>()?;
    if has_dynamic {
        w.passthrough::<UIntLe>()?;
    }

    convert_item(w, to_v1001)?;
    Ok(())
}

fn mob_armour(w: &mut PacketWrapper, to_v1001: bool) -> Result<()> {
    w.passthrough::<UVarInt64>()?;
    for _ in 0..5 {
        convert_item(w, to_v1001)?;
    }
    Ok(())
}

fn biome_definition_list(w: &mut PacketWrapper) -> Result<()> {
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

fn level_sound_event(w: &mut PacketWrapper, to_v1001: bool) -> Result<()> {
    if to_v1001 {
        let id = w.read::<UVarInt>()?;
        let name = id_to_name(id).unwrap_or("");
        w.write::<Str>(name.to_owned());
        w.passthrough_all();
    } else {
        let name = w.read::<Str>()?;
        let id = name_to_id(&name).unwrap_or(0);
        w.write::<UVarInt>(id);
        w.passthrough_all();
    }
    Ok(())
}

fn colour_to_v1001(colour: u32) -> u8 {
    match colour {
        6 => 7,
        other => (other & 0xFF) as u8,
    }
}

fn colour_to_v975(colour: u32) -> u8 {
    match colour {
        7 => 6,
        other => (other & 0xFF) as u8,
    }
}

fn boss_event(w: &mut PacketWrapper, to_v1001: bool) -> Result<()> {
    if to_v1001 {
        let boss_id = w.read::<VarInt64>()?;
        let event_type = w.read::<UVarInt>()?;

        let mut player_id: i64 = 0;
        let mut title = String::new();
        let mut filtered_title = String::new();
        let mut health: f32 = 0.0;
        let mut colour: u32 = 0;
        let mut overlay: u32 = 0;

        match event_type {
            BOSS_SHOW => {
                title = w.read::<Str>()?;
                filtered_title = w.read::<Str>()?;
                health = w.read::<FloatLe>()?;
                w.read::<UShortLe>()?;
                colour = w.read::<UVarInt>()?;
                overlay = w.read::<UVarInt>()?;
            }
            BOSS_REGISTER_PLAYER | BOSS_UNREGISTER_PLAYER | BOSS_REQUEST => {
                player_id = w.read::<VarInt64>()?;
            }
            BOSS_HIDE => {}
            BOSS_HEALTH_PERCENTAGE => {
                health = w.read::<FloatLe>()?;
            }
            BOSS_TITLE => {
                title = w.read::<Str>()?;
                filtered_title = w.read::<Str>()?;
            }
            BOSS_APPEARANCE_PROPERTIES => {
                w.read::<UShortLe>()?;
                colour = w.read::<UVarInt>()?;
                overlay = w.read::<UVarInt>()?;
            }
            BOSS_TEXTURE => {
                colour = w.read::<UVarInt>()?;
                overlay = w.read::<UVarInt>()?;
            }
            _ => {}
        }
        if w.has_remaining() {
            return Err(Error::Invalid("BossEvent v975->v1001 decode left bytes"));
        }

        w.write::<VarInt64>(boss_id);
        w.write::<VarInt64>(player_id);
        w.write::<Byte>(event_type as u8);
        w.write::<Str>(title);
        w.write::<Str>(filtered_title);
        w.write::<FloatLe>(health);
        w.write::<Byte>(colour_to_v1001(colour));
        w.write::<Byte>((overlay & 0xFF) as u8);
    } else {
        let boss_id = w.read::<VarInt64>()?;
        let player_id = w.read::<VarInt64>()?;
        let event_type = w.read::<Byte>()? as u32;
        let title = w.read::<Str>()?;
        let filtered_title = w.read::<Str>()?;
        let health = w.read::<FloatLe>()?;
        let colour = w.read::<Byte>()? as u32;
        let overlay = w.read::<Byte>()? as u32;
        if w.has_remaining() {
            return Err(Error::Invalid("BossEvent v1001->v975 decode left bytes"));
        }

        w.write::<VarInt64>(boss_id);
        w.write::<UVarInt>(event_type);
        match event_type {
            BOSS_SHOW => {
                w.write::<Str>(title);
                w.write::<Str>(filtered_title);
                w.write::<FloatLe>(health);
                w.write::<UShortLe>(0);
                w.write::<UVarInt>(colour_to_v975(colour) as u32);
                w.write::<UVarInt>(overlay);
            }
            BOSS_REGISTER_PLAYER | BOSS_UNREGISTER_PLAYER | BOSS_REQUEST => {
                w.write::<VarInt64>(player_id);
            }
            BOSS_HIDE => {}
            BOSS_HEALTH_PERCENTAGE => {
                w.write::<FloatLe>(health);
            }
            BOSS_TITLE => {
                w.write::<Str>(title);
                w.write::<Str>(filtered_title);
            }
            BOSS_APPEARANCE_PROPERTIES => {
                w.write::<UShortLe>(0);
                w.write::<UVarInt>(colour_to_v975(colour) as u32);
                w.write::<UVarInt>(overlay);
            }
            BOSS_TEXTURE => {
                w.write::<UVarInt>(colour_to_v975(colour) as u32);
                w.write::<UVarInt>(overlay);
            }
            _ => {}
        }
    }
    Ok(())
}

#[derive(Clone)]
struct OldAction {
    source_type: u32,
    window_id: Option<i32>,
    source_flags: Option<u32>,
    slot: u32,
    old_item: Item,
    new_item: Item,
}

#[derive(Clone)]
struct NewAction {
    source_type: u32,
    window_id: Option<i8>,
    source_flags: Option<u32>,
    slot: u32,
    old_item: Item,
    new_item: Item,
}

fn read_old_action(w: &mut PacketWrapper) -> Result<OldAction> {
    let source_type = w.read::<UVarInt>()?;
    let mut window_id = None;
    let mut source_flags = None;
    if source_type == CONTAINER_INVENTORY || source_type == NON_IMPLEMENTED_FEATURE_TODO {
        window_id = Some(w.read::<VarInt>()?);
    } else if source_type == WORLD_INTERACTION {
        source_flags = Some(w.read::<UVarInt>()?);
    }
    let slot = w.read::<UVarInt>()?;
    let old_item = w.read::<ItemInstance>()?;
    let new_item = w.read::<ItemInstance>()?;
    Ok(OldAction {
        source_type,
        window_id,
        source_flags,
        slot,
        old_item,
        new_item,
    })
}

fn write_old_action(w: &mut PacketWrapper, a: &OldAction) -> Result<()> {
    w.write::<UVarInt>(a.source_type);
    if a.source_type == CONTAINER_INVENTORY || a.source_type == NON_IMPLEMENTED_FEATURE_TODO {
        w.write::<VarInt>(a.window_id.ok_or(Error::Invalid("missing WindowID"))?);
    } else if a.source_type == WORLD_INTERACTION {
        w.write::<UVarInt>(a.source_flags.ok_or(Error::Invalid("missing SourceFlags"))?);
    }
    w.write::<UVarInt>(a.slot);
    w.write::<ItemInstance>(a.old_item.clone());
    w.write::<ItemInstance>(a.new_item.clone());
    Ok(())
}

fn read_new_action(w: &mut PacketWrapper) -> Result<NewAction> {
    let source_type = w.read::<UVarInt>()?;
    w.read::<Bool>()?;
    let has_container = w.read::<Bool>()?;
    let window_id = if has_container {
        Some(w.read::<SByte>()?)
    } else {
        None
    };
    w.read::<Bool>()?;
    let has_flags = w.read::<Bool>()?;
    let source_flags = if has_flags {
        Some(w.read::<UVarInt>()?)
    } else {
        None
    };
    let slot = w.read::<UVarInt>()?;
    let old_item = w.read::<ItemInstanceV975>()?;
    let new_item = w.read::<ItemInstanceV975>()?;
    Ok(NewAction {
        source_type,
        window_id,
        source_flags,
        slot,
        old_item,
        new_item,
    })
}

fn write_new_action(w: &mut PacketWrapper, a: &NewAction) -> Result<()> {
    let has_container = a.source_type == CONTAINER_INVENTORY
        || a.source_type == NON_IMPLEMENTED_FEATURE_TODO;
    let has_flags = a.source_type == WORLD_INTERACTION;
    w.write::<UVarInt>(a.source_type);
    w.write::<Bool>(true);
    w.write::<Bool>(has_container);
    if has_container {
        w.write::<SByte>(a.window_id.ok_or(Error::Invalid("missing WindowID"))?);
    }
    w.write::<Bool>(true);
    w.write::<Bool>(has_flags);
    if has_flags {
        w.write::<UVarInt>(a.source_flags.ok_or(Error::Invalid("missing SourceFlags"))?);
    }
    w.write::<UVarInt>(a.slot);
    w.write::<ItemInstanceV975>(a.old_item.clone());
    w.write::<ItemInstanceV975>(a.new_item.clone());
    Ok(())
}

fn old_to_new_action(a: OldAction) -> NewAction {
    NewAction {
        source_type: a.source_type,
        window_id: a.window_id.map(|v| v as i8),
        source_flags: a.source_flags,
        slot: a.slot,
        old_item: a.old_item,
        new_item: a.new_item,
    }
}

fn new_to_old_action(a: NewAction) -> OldAction {
    OldAction {
        source_type: a.source_type,
        window_id: a.window_id.map(|v| v as i32),
        source_flags: a.source_flags,
        slot: a.slot,
        old_item: a.old_item,
        new_item: a.new_item,
    }
}

fn copy_legacy_set_item_slots(w: &mut PacketWrapper) -> Result<()> {
    let slot_count = w.passthrough::<UVarInt>()?;
    for _ in 0..slot_count {
        w.passthrough::<Byte>()?;
        let slots_len = w.passthrough::<UVarInt>()?;
        for _ in 0..slots_len {
            w.passthrough::<Byte>()?;
        }
    }
    Ok(())
}

fn inventory_transaction(w: &mut PacketWrapper, to_v1001: bool) -> Result<()> {
    if to_v1001 {
        let legacy_request_id = w.passthrough::<VarInt>()?;
        let has_legacy = legacy_request_id != 0;
        w.write::<Bool>(has_legacy);
        if has_legacy {
            copy_legacy_set_item_slots(w)?;
        }
        w.write::<Bool>(true);
        let transaction_type = w.passthrough::<UVarInt>()?;
        w.write::<Bool>(true);
        let action_count = w.passthrough::<UVarInt>()?;
        for _ in 0..action_count {
            let action = read_old_action(w)?;
            write_new_action(w, &old_to_new_action(action))?;
        }
        match transaction_type {
            ITEM_USE_TRANSACTION => use_item_old_to_new(w)?,
            ITEM_USE_ON_ENTITY_TRANSACTION => use_item_on_entity_old_to_new(w)?,
            ITEM_RELEASE_TRANSACTION => release_item_old_to_new(w)?,
            _ => {}
        }
    } else {
        w.passthrough::<VarInt>()?;
        let has_legacy = w.read::<Bool>()?;
        if has_legacy {
            copy_legacy_set_item_slots(w)?;
        }
        w.read::<Bool>()?;
        let transaction_type = w.passthrough::<UVarInt>()?;
        w.read::<Bool>()?;
        let action_count = w.passthrough::<UVarInt>()?;
        for _ in 0..action_count {
            let action = read_new_action(w)?;
            write_old_action(w, &new_to_old_action(action))?;
        }
        match transaction_type {
            ITEM_USE_TRANSACTION => use_item_new_to_old(w)?,
            ITEM_USE_ON_ENTITY_TRANSACTION => use_item_on_entity_new_to_old(w)?,
            ITEM_RELEASE_TRANSACTION => release_item_new_to_old(w)?,
            _ => {}
        }
    }
    Ok(())
}

fn use_item_old_to_new(w: &mut PacketWrapper) -> Result<()> {
    let action_type = w.read::<UVarInt>()?;
    w.write::<VarInt>(action_type as i32);
    let trigger_type = w.read::<UVarInt>()?;
    w.write::<Byte>(trigger_type as u8);
    w.passthrough::<BlockPos>()?;
    let block_face = w.read::<VarInt>()?;
    w.write::<SByte>(block_face as i8);
    w.passthrough::<VarInt>()?;
    w.map::<ItemInstance, ItemInstanceV975>()?;
    w.passthrough::<Vec3>()?;
    w.passthrough::<Vec3>()?;
    w.passthrough::<UVarInt>()?;
    let client_prediction = w.read::<UVarInt>()?;
    w.write::<Byte>(client_prediction as u8);
    w.passthrough::<Byte>()?;
    Ok(())
}

fn use_item_new_to_old(w: &mut PacketWrapper) -> Result<()> {
    let action_type = w.read::<VarInt>()?;
    w.write::<UVarInt>(action_type as u32);
    let trigger_type = w.read::<Byte>()?;
    w.write::<UVarInt>(trigger_type as u32);
    w.passthrough::<BlockPos>()?;
    let block_face = w.read::<SByte>()?;
    w.write::<VarInt>(block_face as i32);
    w.passthrough::<VarInt>()?;
    w.map::<ItemInstanceV975, ItemInstance>()?;
    w.passthrough::<Vec3>()?;
    w.passthrough::<Vec3>()?;
    w.passthrough::<UVarInt>()?;
    let client_prediction = w.read::<Byte>()?;
    w.write::<UVarInt>(client_prediction as u32);
    w.passthrough::<Byte>()?;
    Ok(())
}

fn use_item_on_entity_old_to_new(w: &mut PacketWrapper) -> Result<()> {
    w.passthrough::<UVarInt64>()?;
    let action_type = w.read::<UVarInt>()?;
    w.write::<VarInt>(action_type as i32);
    w.passthrough::<VarInt>()?;
    w.map::<ItemInstance, ItemInstanceV975>()?;
    w.passthrough::<Vec3>()?;
    w.passthrough::<Vec3>()?;
    Ok(())
}

fn use_item_on_entity_new_to_old(w: &mut PacketWrapper) -> Result<()> {
    w.passthrough::<UVarInt64>()?;
    let action_type = w.read::<VarInt>()?;
    w.write::<UVarInt>(action_type as u32);
    w.passthrough::<VarInt>()?;
    w.map::<ItemInstanceV975, ItemInstance>()?;
    w.passthrough::<Vec3>()?;
    w.passthrough::<Vec3>()?;
    Ok(())
}

fn release_item_old_to_new(w: &mut PacketWrapper) -> Result<()> {
    let action_type = w.read::<UVarInt>()?;
    w.write::<VarInt>(action_type as i32);
    w.passthrough::<VarInt>()?;
    w.map::<ItemInstance, ItemInstanceV975>()?;
    w.passthrough::<Vec3>()?;
    Ok(())
}

fn release_item_new_to_old(w: &mut PacketWrapper) -> Result<()> {
    let action_type = w.read::<VarInt>()?;
    w.write::<UVarInt>(action_type as u32);
    w.passthrough::<VarInt>()?;
    w.map::<ItemInstanceV975, ItemInstance>()?;
    w.passthrough::<Vec3>()?;
    Ok(())
}

fn sub_chunk_request(w: &mut PacketWrapper, to_v1001: bool) -> Result<()> {
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

fn client_cache_blob_status(w: &mut PacketWrapper, to_v1001: bool) -> Result<()> {
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

const SUB_CHUNK_MODE_LIMITLESS: u32 = 0xFFFF_FFFF;
const SUB_CHUNK_MODE_LIMITED: u32 = 0xFFFF_FFFE;

fn level_chunk(w: &mut PacketWrapper, to_v1001: bool) -> Result<()> {
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

fn client_cache_status(w: &mut PacketWrapper) -> Result<()> {
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

fn diagnostics(w: &mut PacketWrapper, _state: &mut ConnState) -> Result<()> {
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

fn make(name: &'static str, server: u32, client: u32, client_is_1001: bool) -> Translator {
    let cb_to_v1001 = client_is_1001;
    let sb_to_v1001 = !client_is_1001;

    let mut step = Translator::new(name, server, client)
        .clientbound(ids::START_GAME, move |w, _| start_game(w, cb_to_v1001))
        .clientbound(ids::INVENTORY_CONTENT, move |w, _| inventory_content(w, cb_to_v1001))
        .clientbound(ids::MOB_ARMOR_EQUIPMENT, move |w, _| mob_armour(w, cb_to_v1001))
        .clientbound(ids::BIOME_DEFINITION_LIST, move |w, _| biome_definition_list(w))
        .clientbound(ids::FULL_CHUNK_DATA, move |w, _| level_chunk(w, cb_to_v1001))
        .clientbound(ids::LEVEL_SOUND_EVENT, move |w, _| level_sound_event(w, cb_to_v1001))
        .clientbound(ids::BOSS_EVENT, move |w, _| boss_event(w, cb_to_v1001))
        .clientbound(
            ids::INVENTORY_TRANSACTION,
            move |w, _| inventory_transaction(w, cb_to_v1001),
        )
        .serverbound(ids::LEVEL_SOUND_EVENT, move |w, _| level_sound_event(w, sb_to_v1001))
        .serverbound(
            ids::INVENTORY_TRANSACTION,
            move |w, _| inventory_transaction(w, sb_to_v1001),
        )
        .serverbound(
            ids::CLIENT_CACHE_BLOB_STATUS,
            move |w, _| client_cache_blob_status(w, sb_to_v1001),
        )
        .serverbound(ids::SUB_CHUNK_REQUEST, move |w, _| sub_chunk_request(w, sb_to_v1001))
        .serverbound(ids::BOSS_EVENT, move |w, _| boss_event(w, sb_to_v1001));

    if !blob_cache_enabled() {
        step = step.serverbound(ids::CLIENT_CACHE_STATUS, move |w, _| client_cache_status(w));
    }

    step = step.cancel(
        Direction::Serverbound,
        ids::PARTY_DESTINATION_COOKIE_RESPONSE,
    );

    if client_is_1001 {
        step = step.serverbound(ids::SERVERBOUND_DIAGNOSTICS, diagnostics);
    }

    step.cancel(Direction::Serverbound, ids::PARTY_DESTINATION_COOKIE_RESPONSE)
}

pub fn downgrade() -> Translator {
    make("v1001->v975", 975, 1001, true)
}

pub fn upgrade() -> Translator {
    make("v975->v1001", 1001, 975, false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::direction::Direction;

    fn run(handler: impl Fn(&mut PacketWrapper) -> Result<()>, input: &[u8]) -> Vec<u8> {
        let mut w = PacketWrapper::new(input);
        handler(&mut w).expect("handler failed");
        w.finish()
    }

    #[test]
    fn level_chunk_drops_the_highest_sub_chunk_cap() {
        let mut w = Writer::new();
        w.write_varint(-3);
        w.write_varint(0);
        w.write_varint(1000);
        w.write_uvarint(SUB_CHUNK_MODE_LIMITED);
        w.write_u16_le(36);
        w.write_bool(false);
        w.write_count(3);
        w.write_bytes(&[1, 2, 1]);
        let input = w.into_vec();

        let out = run(|w| level_chunk(w, true), &input);

        assert_eq!(out.len(), input.len() - 2);
        let mut r = Reader::new(&out);
        assert_eq!(r.read_varint().unwrap(), -3);
        assert_eq!(r.read_varint().unwrap(), 0);
        assert_eq!(r.read_varint().unwrap(), 1000);
        assert_eq!(r.read_uvarint().unwrap(), SUB_CHUNK_MODE_LIMITLESS);
        assert!(!r.read_bool().unwrap(), "CacheEnabled survives");
        assert_eq!(r.read_count().unwrap(), 3);
        assert_eq!(r.read_bytes(3).unwrap(), &[1, 2, 1]);
        assert_eq!(r.remaining(), 0);
    }

    #[test]
    fn level_chunk_leaves_limitless_and_plain_counts_alone() {
        for count in [SUB_CHUNK_MODE_LIMITLESS, 0, 7] {
            let mut w = Writer::new();
            w.write_varint(1);
            w.write_varint(2);
            w.write_varint(0);
            w.write_uvarint(count);
            w.write_bool(true);
            w.write_bytes(&[0xAB, 0xCD]);
            let input = w.into_vec();
            assert_eq!(run(|w| level_chunk(w, true), &input), input, "count {count} was rewritten");
        }
    }

    #[test]
    fn level_chunk_keeps_the_cap_when_the_client_is_the_older_one() {
        let mut w = Writer::new();
        w.write_varint(-3);
        w.write_varint(0);
        w.write_varint(1000);
        w.write_uvarint(SUB_CHUNK_MODE_LIMITED);
        w.write_u16_le(36);
        w.write_bool(false);
        w.write_count(0);
        let input = w.into_vec();
        assert_eq!(run(|w| level_chunk(w, false), &input), input);
    }

    #[test]
    fn the_blob_cache_is_on_unless_asked_otherwise() {
        assert!(blob_cache_enabled());
        assert!(!downgrade().touches(Direction::Serverbound, ids::CLIENT_CACHE_STATUS));
    }

    #[test]
    fn packets_that_survived_into_v1001_are_never_dropped() {
        let survivors = [
            ids::DIMENSION_DATA,
            ids::PARTY_CHANGED,
            ids::UPDATE_CLIENT_OPTIONS,
        ];
        for step in [downgrade(), upgrade()] {
            for id in survivors {
                assert!(
                    !step.is_cancelled(Direction::Clientbound, id),
                    "{}: packet {id} still exists in v1001 and must not be dropped",
                    step.name
                );
                assert!(
                    !step.is_cancelled(Direction::Serverbound, id),
                    "{}: packet {id} still exists in v1001 and must not be dropped",
                    step.name
                );
            }
        }
    }

    #[test]
    fn party_destination_cookie_response_is_cancelled_serverbound() {
        for step in [downgrade(), upgrade()] {
            assert!(
                step.is_cancelled(Direction::Serverbound, ids::PARTY_DESTINATION_COOKIE_RESPONSE),
                "{} should cancel the v1001-only party cookie response",
                step.name
            );
        }
    }

    #[test]
    fn diagnostics_is_rewritten_not_dropped_and_debug_drawer_passes_through() {
        let down = downgrade();
        assert!(down.handler(Direction::Serverbound, ids::SERVERBOUND_DIAGNOSTICS).is_some());
        assert!(!down.is_cancelled(Direction::Serverbound, ids::SERVERBOUND_DIAGNOSTICS));
        assert!(!down.is_cancelled(Direction::Clientbound, ids::SERVER_SCRIPT_DEBUG_DRAWER));
        assert!(!down.is_cancelled(Direction::Serverbound, ids::SERVER_SCRIPT_DEBUG_DRAWER));

        let up = upgrade();
        assert!(!up.is_cancelled(Direction::Serverbound, ids::SERVERBOUND_DIAGNOSTICS));
        assert!(!up.is_cancelled(Direction::Clientbound, ids::SERVER_SCRIPT_DEBUG_DRAWER));
    }

    #[test]
    fn step_endpoints_are_the_documented_pair() {
        let d = downgrade();
        assert_eq!((d.server_protocol, d.client_protocol), (975, 1001));
        let u = upgrade();
        assert_eq!((u.server_protocol, u.client_protocol), (1001, 975));
    }

    fn build_biome_definition(w: &mut Writer, tags: &[u16], chunk_generation: bool) {
        w.write_i16_le(3);
        w.write_i16_le(21);
        for v in [0.8f32, 0.4, 0.0, 0.125, 0.05] {
            w.write_f32_le(v);
        }
        w.write_i32_le(-0x0033_2211);
        w.write_bool(true);
        w.write_bool(true);
        w.write_count(tags.len());
        for t in tags {
            w.write_u16_le(*t);
        }
        w.write_bool(chunk_generation);
    }

    #[test]
    fn biome_definition_list_passes_through_when_chunk_generation_is_absent() {
        let mut w = Writer::new();
        w.write_count(2);
        build_biome_definition(&mut w, &[1, 2, 3], false);
        build_biome_definition(&mut w, &[], false);
        w.write_count(2);
        w.write_string("minecraft:plains");
        w.write_string("minecraft:river");
        let original = w.into_vec();

        let mut wrapper = PacketWrapper::new(&original);
        biome_definition_list(&mut wrapper).expect("absent ChunkGeneration must translate");
        assert_eq!(
            wrapper.finish(),
            original,
            "the two versions are byte-identical when ChunkGeneration is absent"
        );
    }

    #[test]
    fn biome_definition_list_refuses_rather_than_corrupting_chunk_generation() {
        let mut w = Writer::new();
        w.write_count(1);
        build_biome_definition(&mut w, &[7], true);
        w.write_bytes(&[0u8; 32]);
        let body = w.into_vec();

        let mut wrapper = PacketWrapper::new(&body);
        assert!(
            biome_definition_list(&mut wrapper).is_err(),
            "a present ChunkGeneration must fail loudly, so the pipeline forwards \
             the original bytes and names the packet"
        );
    }

    #[test]
    fn sound_map_round_trips_known_entries() {
        assert_eq!(id_to_name(1), Some("hit"));
        assert_eq!(name_to_id("hit"), Some(1));
        assert_eq!(id_to_name(0), Some("item.use.on"));
        assert_eq!(name_to_id("this-sound-does-not-exist"), None);
    }

    fn build_v975_start_game() -> Vec<u8> {
        let mut w = Writer::new();
        w.write_varint64(0);
        w.write_uvarint64(0);
        w.write_varint(0);
        w.write_f32_le(0.0);
        w.write_f32_le(0.0);
        w.write_f32_le(0.0);
        w.write_f32_le(0.0);
        w.write_f32_le(0.0);

        let ls = LevelSettings {
            seed: 0,
            spawn_biome_type: 0,
            spawn_biome_name: String::new(),
            spawn_dimension: 0,
            generator: 0,
            game_type: 0,
            is_hardcore: false,
            game_difficulty: 0,
            default_spawn_x: 0,
            default_spawn_y: 0,
            default_spawn_z: 0,
            achievements_disabled: false,
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
            multiplayer_intended: false,
            lan_broadcasting_intended: false,
            xbox_live_broadcast_setting: 0,
            platform_broadcast_setting: 0,
            commands_enabled: false,
            texture_packs_required: false,
            game_rules: Vec::new(),
            experiments: Vec::new(),
            ever_toggled: false,
            has_bonus_chest: false,
            start_with_map: false,
            player_permissions: 0,
            server_chunk_tick_range: 0,
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
            base_game_version: String::new(),
            limited_world_width: 0,
            limited_world_depth: 0,
            nether_type: false,
            edu_shared_uri_button_name: String::new(),
            edu_shared_uri_link_uri: String::new(),
            override_force_experimental: None,
            chat_restriction_level: 0,
            disable_player_interactions: false,
        };
        LevelSettingsV944::write(&mut w, &ls);

        w.write_string("");
        w.write_string("");
        w.write_string("");
        w.write_bool(false);
        w.write_varint(0);
        w.write_bool(false);
        w.write_i64_le(0);
        w.write_varint(0);
        w.write_uvarint(0);
        w.write_string("");
        w.write_bool(false);
        w.write_string("");
        w.write_bytes(&[0x00]);
        w.write_i64_le(0);
        w.write_u64_le(0);
        w.write_u64_le(0);
        w.write_bool(false);
        w.write_bool(false);
        w.write_bool(false);
        w.into_vec()
    }

    #[test]
    fn start_game_gains_and_loses_the_three_v1001_fields() {
        let buf = build_v975_start_game();

        let mut up = PacketWrapper::new(&buf);
        start_game(&mut up, true).expect("upgrade start_game");
        let v1001 = up.finish();
        assert_eq!(v1001.len(), buf.len() + 3, "v1001 StartGame must be 3 bytes wider");

        let mut down = PacketWrapper::new(&v1001);
        start_game(&mut down, false).expect("downgrade start_game");
        assert_eq!(down.finish(), buf, "start_game must round-trip");
    }

    #[test]
    fn inventory_content_air_slot_round_trips() {
        let mut buf = Writer::new();
        buf.write_uvarint(49);
        buf.write_uvarint(1);
        buf.write_varint(0);
        buf.write_bytes(&[0x00]);
        buf.write_bool(false);
        buf.write_varint(0);
        let buf_len = buf.len();
        let buf = buf.into_vec();

        let mut up = PacketWrapper::new(&buf);
        inventory_content(&mut up, true).expect("upgrade inventory content");
        let v1001 = up.finish();
        assert!(v1001.len() > buf_len, "NEW air is wider than OLD air");

        let mut down = PacketWrapper::new(&v1001);
        inventory_content(&mut down, false).expect("downgrade inventory content");
        assert_eq!(down.finish(), build_air_inventory_v975(), "item upgrade must round-trip");
    }

    fn build_air_inventory_v975() -> Vec<u8> {
        let mut buf = Writer::new();
        buf.write_uvarint(49);
        buf.write_uvarint(1);
        buf.write_varint(0);
        buf.write_bytes(&[0x00]);
        buf.write_bool(false);
        buf.write_varint(0);
        buf.into_vec()
    }
}
