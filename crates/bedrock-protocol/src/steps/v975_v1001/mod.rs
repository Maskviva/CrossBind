mod action;
mod chunk;
mod inventory;
mod session;
#[cfg(test)]
mod tests;
mod world;

use crate::direction::Direction;
use crate::packet_ids::ids;
use crate::translator::Translator;
pub(crate) use chunk::{
    blob_cache_enabled, client_cache_blob_status, client_cache_status, diagnostics, level_chunk,
    sub_chunk_request,
};
use inventory::{inventory_content, inventory_transaction, mob_armour};
use session::{biome_definition_list, start_game};
use world::{boss_event, level_sound_event};

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

const SUB_CHUNK_MODE_LIMITLESS: u32 = 0xFFFF_FFFF;

const SUB_CHUNK_MODE_LIMITED: u32 = 0xFFFF_FFFE;

fn make(name: &'static str, server: u32, client: u32, client_is_1001: bool) -> Translator {
    let cb_to_v1001 = client_is_1001;
    let sb_to_v1001 = !client_is_1001;

    let mut step = Translator::new(name, server, client)
        .clientbound(ids::START_GAME, move |w, _| start_game(w, cb_to_v1001))
        .clientbound(ids::INVENTORY_CONTENT, move |w, _| {
            inventory_content(w, cb_to_v1001)
        })
        .clientbound(ids::MOB_ARMOR_EQUIPMENT, move |w, _| {
            mob_armour(w, cb_to_v1001)
        })
        .clientbound(ids::BIOME_DEFINITION_LIST, move |w, _| {
            biome_definition_list(w)
        })
        .clientbound(ids::FULL_CHUNK_DATA, move |w, _| {
            level_chunk(w, cb_to_v1001)
        })
        .clientbound(ids::LEVEL_SOUND_EVENT, move |w, _| {
            level_sound_event(w, cb_to_v1001)
        })
        .clientbound(ids::BOSS_EVENT, move |w, _| boss_event(w, cb_to_v1001))
        .clientbound(ids::INVENTORY_TRANSACTION, move |w, _| {
            inventory_transaction(w, cb_to_v1001)
        })
        .serverbound(ids::LEVEL_SOUND_EVENT, move |w, _| {
            level_sound_event(w, sb_to_v1001)
        })
        .serverbound(ids::INVENTORY_TRANSACTION, move |w, _| {
            inventory_transaction(w, sb_to_v1001)
        })
        .serverbound(ids::CLIENT_CACHE_BLOB_STATUS, move |w, _| {
            client_cache_blob_status(w, sb_to_v1001)
        })
        .serverbound(ids::SUB_CHUNK_REQUEST, move |w, _| {
            sub_chunk_request(w, sb_to_v1001)
        })
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

    step.cancel(
        Direction::Serverbound,
        ids::PARTY_DESTINATION_COOKIE_RESPONSE,
    )
}

pub fn downgrade() -> Translator {
    make("v1001->v975", 975, 1001, true)
}

pub fn upgrade() -> Translator {
    make("v975->v1001", 1001, 975, false)
}
