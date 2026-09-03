use super::*;
use crate::steps::v1001_v2168::inventory::*;
use bedrock_codec::prelude::*;
use bedrock_codec::PacketWrapper;

#[allow(unused)]
const SUB_CHUNK_RESULT_LEVEL_CHUNK_DOESNT_EXIST: u8 = 2;

#[test]
fn inventory_slot_survives_container_id_none() {
    let body = inventory_slot_body(0xFF);

    let mut wrapper = PacketWrapper::new(&body);
    inventory_slot(&mut wrapper, true).expect("up-translation must not fail");
    let widened = wrapper.finish();
    assert_eq!(
        widened, body,
        "the packet must be a passthrough for air items"
    );

    let mut wrapper = PacketWrapper::new(&widened);
    inventory_slot(&mut wrapper, false).expect("down-translation must not fail");
    assert_eq!(wrapper.finish(), body);
}

#[test]
fn inventory_slot_container_id_is_a_single_byte() {
    for id in [0x00u8, 0x7F, 0x80, 0xFE, 0xFF] {
        let body = inventory_slot_body(id);
        let mut wrapper = PacketWrapper::new(&body);
        inventory_slot(&mut wrapper, false).expect("handler failed");
        let out = wrapper.finish();
        assert_eq!(out.len(), body.len(), "no length change for id {id:#x}");
        assert_eq!(
            out[0], id,
            "the container id byte must be preserved verbatim"
        );
    }
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
