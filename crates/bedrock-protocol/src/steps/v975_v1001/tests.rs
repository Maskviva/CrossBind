#[cfg(test)]
use crate::sound_events::{id_to_name, name_to_id};
use crate::steps::v975_v1001::*;
use bedrock_codec::prelude::*;
use bedrock_codec::{Codec, PacketWrapper, Writer};

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
        assert_eq!(
            run(|w| level_chunk(w, true), &input),
            input,
            "count {count} was rewritten"
        );
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
            step.is_cancelled(
                Direction::Serverbound,
                ids::PARTY_DESTINATION_COOKIE_RESPONSE
            ),
            "{} should cancel the v1001-only party cookie response",
            step.name
        );
    }
}

#[test]
fn diagnostics_is_rewritten_not_dropped_and_debug_drawer_passes_through() {
    let down = downgrade();
    assert!(down
        .handler(Direction::Serverbound, ids::SERVERBOUND_DIAGNOSTICS)
        .is_some());
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
    assert_eq!(
        v1001.len(),
        buf.len() + 3,
        "v1001 StartGame must be 3 bytes wider"
    );

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
    assert_eq!(
        down.finish(),
        build_air_inventory_v975(),
        "item upgrade must round-trip"
    );
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
