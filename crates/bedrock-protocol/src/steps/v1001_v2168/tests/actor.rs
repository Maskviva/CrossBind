use super::*;
use bedrock_codec::Writer;

#[allow(unused)]
const SUB_CHUNK_RESULT_LEVEL_CHUNK_DOESNT_EXIST: u8 = 2;

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
