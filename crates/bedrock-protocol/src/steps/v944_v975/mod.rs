mod inventory;
mod movement;
mod session;
mod world;

#[cfg(test)]
mod tests;

use crate::direction::Direction;
use crate::mapping::IdShift;
use crate::packet_ids::ids;
use crate::rewriter::SoundRewriter;
use crate::translator::Translator;
use inventory::{inventory_slot, player_enchant_options, player_equipment};
use movement::client_movement_prediction_sync;
use session::{party_changed, start_game, update_client_options};
use world::{actor_event, level_sound_event, locator_bar, play_sound};

const SOUND: IdShift = IdShift::inserted(2, 599);

const HEARTBEAT_KEY: u32 = 126;

const V975_ONLY: &[u16] = &[ids::SERVER_STORE_INFO, ids::SERVER_PRESENCE_INFO];

const DEBUG_DRAWER: &[u16] = &[ids::SERVER_SCRIPT_DEBUG_DRAWER];

const TELEMETRY: &[u16] = &[ids::SERVERBOUND_DIAGNOSTICS];

fn make(name: &'static str, server: u32, client: u32, to_v975_clientbound: bool) -> Translator {
    let to_client = to_v975_clientbound;
    let to_server = !to_v975_clientbound;

    let mut step = Translator::new(name, server, client)
        .clientbound(ids::START_GAME, move |w, _| start_game(w))
        .clientbound(ids::PLAYER_EQUIPMENT, move |w, _| {
            player_equipment(w, to_client)
        })
        .clientbound(ids::INVENTORY_SLOT, move |w, _| {
            inventory_slot(w, to_client)
        })
        .clientbound(ids::PLAYER_ENCHANT_OPTIONS, move |w, _| {
            player_enchant_options(w, to_client)
        })
        .clientbound(ids::LOCATOR_BAR, move |w, _| locator_bar(w, to_client))
        .clientbound(ids::PLAY_SOUND, move |w, _| play_sound(w, to_client))
        .clientbound(ids::ACTOR_EVENT, move |w, _| actor_event(w, to_client))
        .clientbound(ids::LEVEL_SOUND_EVENT, move |w, _| {
            level_sound_event(w, to_client)
        })
        .serverbound(ids::LEVEL_SOUND_EVENT, move |w, _| {
            level_sound_event(w, to_server)
        })
        .serverbound(ids::PLAYER_EQUIPMENT, move |w, _| {
            player_equipment(w, to_server)
        })
        .serverbound(ids::ACTOR_EVENT, move |w, _| actor_event(w, to_server))
        .serverbound(ids::PARTY_CHANGED, move |w, _| party_changed(w, to_server))
        .serverbound(ids::UPDATE_CLIENT_OPTIONS, move |w, _| {
            update_client_options(w, to_server)
        })
        .serverbound(ids::CLIENT_MOVEMENT_PREDICTION_SYNC, move |w, _| {
            client_movement_prediction_sync(w, to_server)
        })
        .cancel_all(Direction::Clientbound, DEBUG_DRAWER)
        .cancel_all(Direction::Serverbound, TELEMETRY);

    if !to_v975_clientbound {
        step = step.cancel_all(Direction::Clientbound, V975_ONLY);
    } else {
        step = step.cancel_all(Direction::Serverbound, V975_ONLY);
    }

    let client_is_newer = client > server;
    SoundRewriter::new(SOUND, client_is_newer, HEARTBEAT_KEY)
        .with_item_encoding_change()
        .without_level_sound_event()
        .register(step)
}

pub fn downgrade() -> Translator {
    make("v975->v944", 944, 975, true)
}

pub fn upgrade() -> Translator {
    make("v944->v975", 975, 944, false)
}
