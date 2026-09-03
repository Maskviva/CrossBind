mod actor;
mod auth_input;
mod bits;
mod chunk;
mod creative;
mod inventory;
mod movement;
mod session;
#[cfg(test)]
mod tests;
mod world;

pub use chunk::*;
pub use world::*;

use crate::convert::{
    cache_item_registry, crafting_data, item_stack_request, item_stack_response, player_list,
    set_score, set_scoreboard_identity,
};
use crate::direction::Direction;
use crate::packet_ids::ids;
use crate::translator::Translator;
use actor::{add_actor, add_item_actor, add_player, set_actor_data};
use auth_input::player_auth_input;
use chunk::{full_chunk_data, sub_chunk, sub_chunk_shape};
use creative::creative_content;
use inventory::{
    inventory_content, inventory_slot, inventory_transaction, mob_armor_equipment, player_equipment,
};
use movement::{move_delta_actor, move_player};
use session::{
    dimension_data, resource_pack_client_response, resource_packs_info, start_game, transfer,
};
use world::{anvil_damage, play_sound, serverbound_diagnostics, structure_block_update};

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
