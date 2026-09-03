#[cfg(test)]
use crate::steps::v944_v975::inventory::{air, convert_item};
use crate::steps::v944_v975::*;
use bedrock_codec::prelude::*;
use bedrock_codec::{Codec, PacketWrapper, Writer};

fn sample_item() -> Item {
    Item {
        network_id: 42,
        count: 7,
        aux_value: 3,
        has_net_id: true,
        stack_net_id: 11,
        net_id_variant: 0,
        block_runtime_id: 5,
        user_data: vec![9, 9],
    }
}

fn round_trip(v944: &[u8], f: fn(&mut PacketWrapper, bool) -> Result<()>) {
    let mut up = PacketWrapper::new(v944);
    f(&mut up, true).unwrap();
    let v975 = up.finish();

    let mut down = PacketWrapper::new(&v975);
    f(&mut down, false).unwrap();
    assert_eq!(down.finish(), v944, "round trip changed the packet");
}

#[test]
fn item_survives_a_round_trip_through_v975() {
    let mut w = Writer::new();
    ItemInstance::write(&mut w, &sample_item());
    let v944 = w.into_vec();

    let mut up = PacketWrapper::new(&v944);
    convert_item(&mut up, true).unwrap();
    let v975 = up.finish();

    let mut down = PacketWrapper::new(&v975);
    convert_item(&mut down, false).unwrap();
    assert_eq!(down.finish(), v944);
}

#[test]
fn air_survives_despite_the_shortcut_difference() {
    let v944 = vec![0x00u8];
    let mut up = PacketWrapper::new(&v944);
    convert_item(&mut up, true).unwrap();
    let v975 = up.finish();
    assert!(v975.len() > 1);

    let mut down = PacketWrapper::new(&v975);
    convert_item(&mut down, false).unwrap();
    assert_eq!(down.finish(), v944);
}

#[test]
fn player_equipment_widens_and_narrows_the_slot_fields() {
    let mut w = Writer::new();
    UVarInt64::write(&mut w, &7);
    ItemInstance::write(&mut w, &sample_item());
    Byte::write(&mut w, &2);
    Byte::write(&mut w, &0);
    Byte::write(&mut w, &1);
    let v944 = w.into_vec();

    let mut up = PacketWrapper::new(&v944);
    player_equipment(&mut up, true).unwrap();
    let v975 = up.finish();
    assert!(v975.len() > v944.len(), "the item form should have grown");

    let mut down = PacketWrapper::new(&v975);
    player_equipment(&mut down, false).unwrap();
    assert_eq!(down.finish(), v944);
}

#[test]
fn actor_event_round_trips() {
    let mut w = Writer::new();
    UVarInt64::write(&mut w, &9);
    Byte::write(&mut w, &3);
    VarInt::write(&mut w, &-4);
    round_trip(&w.into_vec(), actor_event);
}

#[test]
fn party_changed_round_trips_with_and_without_the_optional() {
    let mut absent = Writer::new();
    Bool::write(&mut absent, &false);
    round_trip(&absent.into_vec(), party_changed);

    let mut present = Writer::new();
    Bool::write(&mut present, &true);
    Str::write(&mut present, &"abc".to_string());
    round_trip(&present.into_vec(), party_changed);
}

#[test]
fn update_client_options_round_trips() {
    let mut none = Writer::new();
    Bool::write(&mut none, &false);
    round_trip(&none.into_vec(), update_client_options);

    let mut some = Writer::new();
    Bool::write(&mut some, &true);
    Byte::write(&mut some, &2);
    round_trip(&some.into_vec(), update_client_options);
}

#[test]
fn movement_prediction_sync_moves_exactly_three_floats() {
    let mut w = Writer::new();
    Byte::write(&mut w, &0x00);
    for i in 0..9 {
        FloatLe::write(&mut w, &(i as f32));
    }
    VarInt64::write(&mut w, &123);
    Bool::write(&mut w, &true);
    let v944 = w.into_vec();

    let mut up = PacketWrapper::new(&v944);
    client_movement_prediction_sync(&mut up, true).unwrap();
    let v975 = up.finish();
    assert_eq!(v975.len(), v944.len() + 12);

    let mut down = PacketWrapper::new(&v975);
    client_movement_prediction_sync(&mut down, false).unwrap();
    assert_eq!(down.finish(), v944);
}

#[test]
fn inventory_slot_round_trips_with_a_real_storage_item() {
    let mut w = Writer::new();
    UVarInt::write(&mut w, &1);
    UVarInt::write(&mut w, &4);
    Byte::write(&mut w, &12);
    Bool::write(&mut w, &false);
    ItemInstance::write(&mut w, &sample_item());
    ItemInstance::write(&mut w, &sample_item());
    round_trip(&w.into_vec(), inventory_slot);
}

#[test]
fn inventory_slot_maps_air_storage_to_an_absent_optional() {
    let mut w = Writer::new();
    UVarInt::write(&mut w, &0);
    UVarInt::write(&mut w, &0);
    Byte::write(&mut w, &0);
    Bool::write(&mut w, &false);
    ItemInstance::write(&mut w, &air());
    ItemInstance::write(&mut w, &sample_item());
    round_trip(&w.into_vec(), inventory_slot);
}

#[test]
fn player_enchant_options_round_trips() {
    let mut w = Writer::new();
    UVarInt::write(&mut w, &1);
    UVarInt::write(&mut w, &30);
    IntLe::write(&mut w, &2);
    for _ in 0..3 {
        UVarInt::write(&mut w, &1);
        Byte::write(&mut w, &5);
        Byte::write(&mut w, &2);
    }
    Str::write(&mut w, &"sharp".to_string());
    UVarInt::write(&mut w, &77);
    round_trip(&w.into_vec(), player_enchant_options);
}

#[test]
fn level_sound_event_shifts_the_id_back_and_forth() {
    let mut w = Writer::new();
    UVarInt::write(&mut w, &700);
    Vec3::write(&mut w, &(1.0, 2.0, 3.0));
    VarInt::write(&mut w, &0);
    Str::write(&mut w, &"pig".to_string());
    Bool::write(&mut w, &false);
    Bool::write(&mut w, &false);
    Int64Le::write(&mut w, &-1);
    round_trip(&w.into_vec(), level_sound_event);
}

#[test]
fn v975_only_packets_are_dropped_toward_v944() {
    let up = upgrade();
    for id in V975_ONLY {
        assert!(
            up.is_cancelled(Direction::Clientbound, *id),
            "packet {id} should not reach a v944 client"
        );
    }
    let down = downgrade();
    for id in V975_ONLY {
        assert!(
            down.is_cancelled(Direction::Serverbound, *id),
            "packet {id} should not reach a v944 server"
        );
    }
}

#[test]
fn debug_drawer_and_diagnostics_are_cancelled_in_both_steps() {
    for step in [downgrade(), upgrade()] {
        assert!(step.is_cancelled(Direction::Clientbound, ids::SERVER_SCRIPT_DEBUG_DRAWER));
        assert!(step.is_cancelled(Direction::Serverbound, ids::SERVERBOUND_DIAGNOSTICS));
    }
}

#[test]
fn step_endpoints_are_the_documented_pair() {
    let d = downgrade();
    assert_eq!((d.server_protocol, d.client_protocol), (944, 975));
    let u = upgrade();
    assert_eq!((u.server_protocol, u.client_protocol), (975, 944));
}
