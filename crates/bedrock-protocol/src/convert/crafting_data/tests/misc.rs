use super::*;
use bedrock_codec::PacketWrapper;

fn registry() -> (HashMap<String, i32>, HashMap<i32, String>) {
    let mut names = HashMap::new();
    let mut ids = HashMap::new();
    names.insert("minecraft:stone".to_string(), 5);
    ids.insert(5, "minecraft:stone".to_string());
    names.insert("minecraft:iron_ingot".to_string(), 9);
    ids.insert(9, "minecraft:iron_ingot".to_string());
    (names, ids)
}

fn state_with_registry() -> ConnState {
    let mut s = ConnState::new(2168);
    let (names, ids) = registry();
    s.item_ids = names;
    s.item_names = ids;
    s
}

fn plain_output(id: i32) -> Item {
    Item {
        network_id: id,
        count: 1,
        aux_value: 0,
        has_net_id: false,
        stack_net_id: 0,
        net_id_variant: 0,
        block_runtime_id: 0,
        user_data: Vec::new(),
    }
}

fn v1001_bytes() -> Vec<u8> {
    let mut w = Writer::new();
    w.write_count(1);
    w.write_varint(TYPE_SHAPELESS);
    Str::write(&mut w, &"recipe:stone_from_iron".to_string());
    w.write_count(1);
    w.write_u8(DESC_DEFAULT);
    w.write_i16_le(9);
    w.write_i16_le(0);
    w.write_varint(1);
    w.write_count(1);
    NetworkItemInstanceDescriptor::write(&mut w, &plain_output(5));
    Uuid::write(&mut w, &MceUuid::default());
    Str::write(&mut w, &"crafting_table".to_string());
    w.write_varint(0);
    w.write_u8(1);
    w.write_uvarint(7);
    w.write_count(0);
    w.write_count(0);
    w.write_count(0);
    w.write_bool(false);
    w.into_vec()
}

#[test]
fn v1001_to_v2168_round_trips_a_shapeless_recipe() {
    let mut state = state_with_registry();
    let input = v1001_bytes();
    let mut wrapper = PacketWrapper::new(&input);
    let ok = crafting_data(&mut wrapper, &mut state, true).unwrap();
    assert!(ok, "recipe with a known ingredient must not be dropped");
    assert!(
        state.notices.is_empty(),
        "no ingredient should have been skipped"
    );

    let out = wrapper.finish();
    let mut r = Reader::new(&out);

    assert_eq!(r.read_count().unwrap(), 0);
    assert_eq!(r.read_count().unwrap(), 1);
    assert_eq!(Str::read(&mut r).unwrap(), "recipe:stone_from_iron");
    assert_eq!(r.read_count().unwrap(), 1);
    assert_eq!(r.read_uvarint().unwrap(), u32::from(DESC_DEFAULT));
    assert_eq!(Str::read(&mut r).unwrap(), "name");
    assert_eq!(Str::read(&mut r).unwrap(), "minecraft:iron_ingot");
    assert_eq!(r.read_varint().unwrap(), 0);
    assert_eq!(r.read_varint().unwrap(), 1);
}

#[test]
fn v2168_to_v1001_round_trips_the_same_recipe() {
    let mut state = state_with_registry();
    let input = v1001_bytes();

    let mut up = PacketWrapper::new(&input);
    crafting_data(&mut up, &mut state, true).unwrap();
    let v2168_bytes = up.finish();

    let mut down = PacketWrapper::new(&v2168_bytes);
    let ok = crafting_data(&mut down, &mut state, false).unwrap();
    assert!(ok);
    assert!(state.notices.is_empty());

    let round_tripped = down.finish();
    assert_eq!(
        round_tripped, input,
        "v1001 -> v2168 -> v1001 must be the identity"
    );
}

#[test]
fn unknown_ingredient_drops_only_that_recipe() {
    let mut state = state_with_registry();
    let mut w = Writer::new();
    w.write_count(1);
    w.write_varint(TYPE_SHAPELESS);
    Str::write(&mut w, &"recipe:unknown".to_string());
    w.write_count(1);
    w.write_u8(DESC_DEFAULT);
    w.write_i16_le(999);
    w.write_i16_le(0);
    w.write_varint(1);
    w.write_count(0);
    Uuid::write(&mut w, &MceUuid::default());
    Str::write(&mut w, &"crafting_table".to_string());
    w.write_varint(0);
    w.write_u8(1);
    w.write_uvarint(1);
    w.write_count(0);
    w.write_count(0);
    w.write_count(0);
    w.write_bool(false);
    let input = w.into_vec();

    let mut wrapper = PacketWrapper::new(&input);
    let ok = crafting_data(&mut wrapper, &mut state, true).unwrap();
    assert!(
        ok,
        "a decodable packet with one bad ingredient is still forwarded"
    );
    assert_eq!(state.notices.len(), 1);
    assert!(state.notices[0].contains("1 of 1 recipes dropped"));
    assert!(
        state.notices[0].contains("999"),
        "the notice must name the unresolved id, got: {}",
        state.notices[0]
    );
    assert!(
        state.notices[0].contains("2-entry"),
        "the notice must say how big the registry was, got: {}",
        state.notices[0]
    );

    let out = wrapper.finish();
    let mut r = Reader::new(&out);
    assert_eq!(r.read_count().unwrap(), 0);
    assert_eq!(r.read_count().unwrap(), 0);
}

#[test]
fn complex_alias_ingredient_survives_as_a_name() {
    let mut state = state_with_registry();
    let mut w = Writer::new();
    w.write_count(1);
    w.write_varint(TYPE_SHAPELESS);
    Str::write(&mut w, &"recipe:from_alias".to_string());
    w.write_count(1);
    w.write_u8(DESC_COMPLEX_ALIAS);
    Str::write(&mut w, &"minecraft:planks".to_string());
    w.write_varint(1);
    w.write_count(1);
    NetworkItemInstanceDescriptor::write(&mut w, &plain_output(5));
    Uuid::write(&mut w, &MceUuid::default());
    Str::write(&mut w, &"crafting_table".to_string());
    w.write_varint(0);
    w.write_u8(1);
    w.write_uvarint(3);
    w.write_count(0);
    w.write_count(0);
    w.write_count(0);
    w.write_bool(false);
    let input = w.into_vec();

    let mut wrapper = PacketWrapper::new(&input);
    assert!(crafting_data(&mut wrapper, &mut state, true).unwrap());
    assert!(
        state.notices.is_empty(),
        "an aliased ingredient must not drop the recipe, got: {:?}",
        state.notices
    );

    let out = wrapper.finish();
    let mut r = Reader::new(&out);
    assert_eq!(r.read_count().unwrap(), 0);
    assert_eq!(r.read_count().unwrap(), 1);
    assert_eq!(Str::read(&mut r).unwrap(), "recipe:from_alias");
    assert_eq!(r.read_count().unwrap(), 1);
    assert_eq!(r.read_uvarint().unwrap(), u32::from(DESC_DEFAULT));
    assert_eq!(Str::read(&mut r).unwrap(), "name");
    assert_eq!(Str::read(&mut r).unwrap(), "minecraft:planks");
}

#[test]
fn recipe_type_discriminants_are_not_dense() {
    assert_eq!(TYPE_MULTI, 4);
    assert_eq!(TYPE_USER_DATA_SHAPELESS, 5);
    assert_eq!(TYPE_SHAPELESS_CHEMISTRY, 6);
    assert_eq!(TYPE_SHAPED_CHEMISTRY, 7);
    assert_eq!(TYPE_SMITHING_TRANSFORM, 8);
    assert_eq!(TYPE_SMITHING_TRIM, 9);
}

#[test]
fn multi_recipe_round_trips_through_the_correct_vector() {
    let mut state = state_with_registry();
    let mut w = Writer::new();
    w.write_count(1);
    w.write_varint(TYPE_MULTI);
    Uuid::write(&mut w, &MceUuid::default());
    w.write_uvarint(42);
    w.write_count(0);
    w.write_count(0);
    w.write_count(0);
    w.write_bool(false);
    let input = w.into_vec();

    let mut wrapper = PacketWrapper::new(&input);
    let ok = crafting_data(&mut wrapper, &mut state, true).unwrap();
    assert!(ok);
    assert!(state.notices.is_empty());

    let out = wrapper.finish();
    let mut r = Reader::new(&out);
    assert_eq!(r.read_count().unwrap(), 0);
    assert_eq!(r.read_count().unwrap(), 0);
    assert_eq!(r.read_count().unwrap(), 1);
    Uuid::read(&mut r).unwrap();
    assert_eq!(r.read_uvarint().unwrap(), 42);
}

#[test]
fn shaped_recipe_grid_has_no_length_prefix_on_v1001() {
    let mut state = state_with_registry();
    let mut w = Writer::new();
    w.write_count(1);
    w.write_varint(TYPE_SHAPED);
    Str::write(&mut w, &"recipe:shaped".to_string());
    w.write_varint(2);
    w.write_varint(1);
    for _ in 0..2 {
        w.write_u8(DESC_DEFAULT);
        w.write_i16_le(5);
        w.write_i16_le(0);
        w.write_varint(1);
    }
    w.write_count(1);
    NetworkItemInstanceDescriptor::write(&mut w, &plain_output(5));
    Uuid::write(&mut w, &MceUuid::default());
    Str::write(&mut w, &"crafting_table".to_string());
    w.write_varint(0);
    w.write_bool(false);
    w.write_u8(1);
    w.write_uvarint(3);
    w.write_count(0);
    w.write_count(0);
    w.write_count(0);
    w.write_bool(false);
    let input = w.into_vec();

    let mut wrapper = PacketWrapper::new(&input);
    let ok = crafting_data(&mut wrapper, &mut state, true).unwrap();
    assert!(ok);
    assert!(state.notices.is_empty());

    let out = wrapper.finish();
    let mut r = Reader::new(&out);
    assert_eq!(r.read_count().unwrap(), 1);
    Str::read(&mut r).unwrap();
    assert_eq!(r.read_varint().unwrap(), 2);
    assert_eq!(r.read_varint().unwrap(), 1);
    assert_eq!(r.read_count().unwrap(), 2);
}

#[test]
fn unlock_requirement_absent_on_v1001_has_no_outer_flag() {
    let mut state = state_with_registry();
    let mut w = Writer::new();
    w.write_count(1);
    w.write_varint(TYPE_SHAPELESS);
    Str::write(&mut w, &"recipe:no_unlock".to_string());
    w.write_count(0);
    w.write_count(0);
    Uuid::write(&mut w, &MceUuid::default());
    Str::write(&mut w, &"crafting_table".to_string());
    w.write_varint(0);
    w.write_u8(2);
    w.write_uvarint(9);
    w.write_count(0);
    w.write_count(0);
    w.write_count(0);
    w.write_bool(false);
    let input = w.into_vec();

    let mut up = PacketWrapper::new(&input);
    crafting_data(&mut up, &mut state, true).unwrap();
    let v2168_bytes = up.finish();
    let mut r = Reader::new(&v2168_bytes);
    r.read_count().unwrap();
    assert_eq!(r.read_count().unwrap(), 1);
    Str::read(&mut r).unwrap();
    r.read_count().unwrap();
    r.read_count().unwrap();
    Uuid::read(&mut r).unwrap();
    Str::read(&mut r).unwrap();
    r.read_varint().unwrap();
    assert!(
        r.read_bool().unwrap(),
        "v2168 outer Optional must be present"
    );
    assert_eq!(r.read_varint().unwrap(), 2);
    assert!(
        !r.read_bool().unwrap(),
        "context != 0 means no ingredient table"
    );
}

#[test]
fn molang_ingredient_keeps_stack_size_aligned_after_conversion() {
    let mut state = state_with_registry();

    let mut w = Writer::new();
    w.write_count(1);
    w.write_varint(TYPE_SHAPELESS);
    Str::write(&mut w, &"recipe:molang_test".to_string());
    w.write_count(1);
    w.write_u8(DESC_MOLANG);
    Str::write(&mut w, &"q.foo".to_string());
    w.write_u8(3);
    w.write_varint(7);
    w.write_count(1);
    NetworkItemInstanceDescriptor::write(&mut w, &plain_output(5));
    Uuid::write(&mut w, &MceUuid::default());
    Str::write(&mut w, &"crafting_table".to_string());
    w.write_varint(0);
    w.write_u8(1);
    w.write_uvarint(11);
    w.write_count(0);
    w.write_count(0);
    w.write_count(0);
    w.write_bool(false);
    let input = w.into_vec();

    let mut wrapper = PacketWrapper::new(&input);
    let ok = crafting_data(&mut wrapper, &mut state, true).unwrap();
    assert!(ok);
    let up = wrapper.finish();

    let mut r = Reader::new(&up);
    assert_eq!(r.read_count().unwrap(), 0);
    assert_eq!(r.read_count().unwrap(), 1);
    assert_eq!(Str::read(&mut r).unwrap(), "recipe:molang_test");
    assert_eq!(r.read_count().unwrap(), 1);
    assert_eq!(r.read_uvarint().unwrap(), u32::from(DESC_DEFAULT));
    assert_eq!(Str::read(&mut r).unwrap(), "molang");
    assert_eq!(Str::read(&mut r).unwrap(), "q.foo");
    assert_eq!(r.read_varint().unwrap(), 0, "aux_value defaults to 0");
    assert_eq!(
        r.read_varint().unwrap(),
        7,
        "stack_size must survive the conversion"
    );
}

#[test]
fn malformed_packet_is_cancelled_not_forwarded() {
    let mut state = state_with_registry();
    let input = vec![0xFFu8];
    let mut wrapper = PacketWrapper::new(&input);
    let ok = crafting_data(&mut wrapper, &mut state, true).unwrap();
    assert!(
        !ok,
        "an undecodable packet must be reported as cancel, not forwarded"
    );
    assert!(state.notices.iter().any(|n| n.contains("cannot decode")));
}

#[allow(unused)]
fn shapeless_entry(w: &mut Writer, recipe_id: &str, output: &Item) {
    w.write_varint(TYPE_SHAPELESS);
    Str::write(w, &recipe_id.to_string());
    w.write_count(1);
    w.write_u8(DESC_DEFAULT);
    w.write_i16_le(9);
    w.write_i16_le(0);
    w.write_varint(1);
    w.write_count(1);
    NetworkItemInstanceDescriptor::write(w, output);
    Uuid::write(w, &MceUuid::default());
    Str::write(w, &"crafting_table".to_string());
    w.write_varint(0);
    w.write_u8(1);
    w.write_uvarint(7);
}
