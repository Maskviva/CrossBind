use super::*;

pub fn v1001_craft_recipe_auto_molang(expression: &str, version: u8) -> Vec<u8> {
    let mut w = Writer::new();
    w.write_count(1);
    w.write_varint(1);
    w.write_count(1);
    w.write_u8(13);
    w.write_varint(50);
    w.write_u8(1);
    w.write_u8(1);
    w.write_count(1);
    w.write_u8(INTERNAL_TYPE_MOLANG);
    Str::write(&mut w, &expression.to_owned());
    w.write_u8(version);
    w.write_varint(1);
    w.write_count(0);
    w.write_i32_le(0);
    w.into_vec()
}

pub fn v1001_craft_recipe_auto_complex_alias(alias: &str) -> Vec<u8> {
    let mut w = Writer::new();
    w.write_count(1);
    w.write_varint(1);
    w.write_count(1);
    w.write_u8(13);
    w.write_varint(50);
    w.write_u8(1);
    w.write_u8(1);
    w.write_count(1);
    w.write_u8(INTERNAL_TYPE_COMPLEX_ALIAS);
    Str::write(&mut w, &alias.to_owned());
    w.write_varint(1);
    w.write_count(0);
    w.write_i32_le(0);
    w.into_vec()
}

#[test]
fn craft_recipe_auto_air_ingredient_canonicalises_to_empty() {
    let (names, ids) = empty_tables();

    let mut w = Writer::new();
    w.write_count(1);
    w.write_varint(1);
    w.write_count(1);
    w.write_u8(13);
    w.write_varint(200);
    w.write_u8(1);
    w.write_u8(1);
    w.write_count(1);
    w.write_u8(INTERNAL_TYPE_DEFAULT);
    w.write_i16_le(0);
    w.write_varint(1);
    w.write_count(0);
    w.write_i32_le(0);
    let original = w.into_vec();

    let mut up = PacketWrapper::new(&original);
    assert!(item_stack_request(&mut up, true, &names, &ids).unwrap());
    let widened = up.finish();

    let mut expected = Writer::new();
    expected.write_count(1);
    expected.write_varint(1);
    expected.write_count(1);
    expected.write_u8(13);
    expected.write_varint(200);
    expected.write_u8(1);
    expected.write_u8(1);
    expected.write_count(1);
    expected.write_u8(INTERNAL_TYPE_INVALID);
    expected.write_varint(1);
    expected.write_count(0);
    expected.write_i32_le(0);

    let mut down = PacketWrapper::new(&widened);
    assert!(item_stack_request(&mut down, false, &names, &ids).unwrap());
    assert_eq!(down.finish(), expected.into_vec());
}
