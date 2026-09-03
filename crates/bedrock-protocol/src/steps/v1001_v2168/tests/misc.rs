use super::*;
use crate::pipeline::{translate, Outcome};
use crate::{build_registry, ConnState};
use bedrock_codec::prelude::*;
use bedrock_codec::{PacketWrapper, Writer};

#[allow(unused)]
const SUB_CHUNK_RESULT_LEVEL_CHUNK_DOESNT_EXIST: u8 = 2;

pub(crate) fn run(handler: impl Fn(&mut PacketWrapper) -> Result<()>, input: &[u8]) -> Vec<u8> {
    let mut w = PacketWrapper::new(input);
    handler(&mut w).expect("handler failed");
    w.finish()
}

#[test]
fn step_endpoints_are_the_documented_pair() {
    assert_eq!(downgrade().server_protocol, 1001);
    assert_eq!(downgrade().client_protocol, 2168);
    assert_eq!(upgrade().server_protocol, 2168);
    assert_eq!(upgrade().client_protocol, 1001);
}

fn air_item_v975() -> Vec<u8> {
    vec![0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
}

pub(crate) fn inventory_slot_body(id: u8) -> Vec<u8> {
    let mut w = Writer::new();
    w.write_u8(id);
    w.write_uvarint(5);
    w.write_bool(false);
    w.write_bool(false);
    w.write_bytes(&air_item_v975());
    w.into_vec()
}

pub(crate) fn join_info_v1001() -> Vec<u8> {
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

pub(crate) fn v1001_stack(w: &mut Writer, net_id: i32) {
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

pub(crate) fn v1001_metadata(w: &mut Writer) {
    w.write_count(2);
    w.write_uvarint(0);
    w.write_uvarint(7);
    w.write_varint64(-1);
    w.write_uvarint(4);
    w.write_uvarint(4);
    w.write_string("steve");
}

#[test]
fn inventory_transaction_is_cancelled_in_neither_direction() {
    let step = downgrade();
    assert!(!step.is_cancelled(Direction::Clientbound, ids::INVENTORY_TRANSACTION));
    assert!(!step.is_cancelled(Direction::Serverbound, ids::INVENTORY_TRANSACTION));
}

pub(crate) fn v2168_auth_input(
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
