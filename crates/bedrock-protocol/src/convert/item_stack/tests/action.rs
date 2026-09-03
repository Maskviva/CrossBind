use super::*;

#[test]
fn variant_mapping_is_invertible_and_skips_the_container_pair() {
    for id in 0..=ACTION_MAX {
        if id == ACTION_PLACE_IN_CONTAINER || id == ACTION_TAKE_OUT_CONTAINER {
            continue;
        }
        assert_eq!(action_id(action_variant(id)), id, "id {id} did not survive");
    }
    assert_eq!(action_variant(6), 6);
    assert_eq!(action_variant(9), 7);
    assert_eq!(action_id(7), 9);
}

pub fn v1001_craft_recipe_auto_default(
    recipe_id: i32,
    num_crafts: u8,
    id: i16,
    count: i32,
) -> Vec<u8> {
    let mut w = Writer::new();
    w.write_count(1);
    w.write_varint(1);
    w.write_count(1);
    w.write_u8(13);
    w.write_varint(recipe_id);
    w.write_u8(num_crafts);
    w.write_u8(1);
    w.write_count(1);
    w.write_u8(INTERNAL_TYPE_DEFAULT);
    w.write_i16_le(id);
    w.write_i16_le(0);
    w.write_varint(count);
    w.write_count(0);
    w.write_i32_le(0);
    w.into_vec()
}

#[test]
fn craft_recipe_auto_refuses_an_unknown_item_name_going_down() {
    let mut w = Writer::new();
    w.write_count(1);
    w.write_varint(1);
    w.write_count(1);
    w.write_uvarint(action_variant(13));
    w.write_u8(13);
    w.write_varint(200);
    w.write_u8(1);
    w.write_count(1);
    w.write_u8(RECIPE_DESC_ITEM_NAME);
    Str::write(&mut w, &"modded:widget".to_owned());
    w.write_varint(0);
    w.write_u16_le(1);
    w.write_count(0);
    w.write_i32_le(0);
    let body = w.into_vec();

    let (names, ids) = stone_tables();
    let mut wrapper = PacketWrapper::new(&body);
    assert!(
        !item_stack_request(&mut wrapper, false, &names, &ids).unwrap(),
        "an unknown ingredient name must cancel the request, not fabricate an id"
    );
}

#[test]
fn craft_recipe_auto_sends_complex_alias_as_a_name() {
    let (names, ids) = empty_tables();
    let body = v1001_craft_recipe_auto_complex_alias("minecraft:planks");

    let mut wrapper = PacketWrapper::new(&body);
    assert!(
        item_stack_request(&mut wrapper, true, &names, &ids).unwrap(),
        "an aliased ingredient must not cancel the craft"
    );
    let widened = wrapper.finish();

    let mut r = Reader::new(&widened);
    assert_eq!(r.read_count().unwrap(), 1);
    assert_eq!(r.read_varint().unwrap(), 1);
    assert_eq!(r.read_count().unwrap(), 1);
    assert_eq!(r.read_uvarint().unwrap(), action_variant(13));
    assert_eq!(r.read_u8().unwrap(), 13);
    assert_eq!(r.read_varint().unwrap(), 50);
    assert_eq!(r.read_u8().unwrap(), 1);
    assert_eq!(r.read_count().unwrap(), 1);
    assert_eq!(r.read_u8().unwrap(), RECIPE_DESC_ITEM_NAME);
    assert_eq!(Str::read(&mut r).unwrap(), "minecraft:planks");
    assert_eq!(r.read_varint().unwrap(), 0);
    assert_eq!(r.read_u16_le().unwrap(), 1);
}

pub fn v2168_pull_with_result(name: &str) -> Vec<u8> {
    let mut w = Writer::new();
    w.write_count(1);
    w.write_varint(-3);
    w.write_count(1);
    w.write_uvarint(action_variant(19));
    w.write_u8(19);
    w.write_count(1);
    w.write_uvarint(DESCRIPTOR_DEFAULT);
    w.write_u8(DESCRIPTOR_DEFAULT as u8);
    Str::write(&mut w, &name.to_owned());
    w.write_varint(0);
    w.write_i16_le(64);
    w.write_uvarint(0);
    w.write_count(0);
    w.write_u8(1);
    w.write_count(0);
    w.write_i32_le(0);
    w.into_vec()
}
