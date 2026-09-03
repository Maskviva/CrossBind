use crate::prelude::enums::command_origin_type::{PLAYER, TEST};
use crate::prelude::enums::command_permission_level::ANY;
use crate::prelude::*;

#[test]
fn index_width_thresholds() {
    assert_eq!(EnumIndexWidth::for_table_size(0), EnumIndexWidth::U8);
    assert_eq!(EnumIndexWidth::for_table_size(256), EnumIndexWidth::U8);
    assert_eq!(EnumIndexWidth::for_table_size(257), EnumIndexWidth::U16);
    assert_eq!(EnumIndexWidth::for_table_size(65536), EnumIndexWidth::U16);
    assert_eq!(EnumIndexWidth::for_table_size(65537), EnumIndexWidth::U32);
}

#[test]
fn permission_survives_the_label_round_trip() {
    let def = CommandDefinition {
        name: "give".into(),
        description: "".into(),
        flags: 0,
        permission: 2,
        alias_index: -1,
        subcommand_indices: vec![],
        overloads: vec![],
    };
    let mut w = Writer::new();
    CommandDefinitionV898::write(&mut w, &def);
    let bytes = w.into_vec();
    let mut r = Reader::new(&bytes);
    assert_eq!(CommandDefinitionV898::read(&mut r).unwrap(), def);
}

#[test]
fn unknown_permission_label_falls_back_to_any() {
    let mut w = Writer::new();
    w.write_string("give");
    w.write_string("");
    w.write_u16_le(0);
    w.write_string("someFutureLevel");
    w.write_i32_le(-1);
    w.write_count(0);
    w.write_count(0);
    let bytes = w.into_vec();
    let mut r = Reader::new(&bytes);
    let def = CommandDefinitionV898::read(&mut r).unwrap();
    assert_eq!(def.permission, ANY);
}

#[test]
fn origin_encodings_differ_and_both_round_trip() {
    let origin = CommandOrigin {
        origin_type: PLAYER,
        uuid: MceUuid { msb: 1, lsb: 2 },
        request_id: "r".into(),
        player_id: -1,
    };

    let mut a = Writer::new();
    CommandOriginV860::write(&mut a, &origin);
    assert_eq!(a.len(), 19);

    let mut b = Writer::new();
    CommandOriginV898::write(&mut b, &origin);
    assert_eq!(b.len(), 33);

    let a_bytes = a.into_vec();
    let mut ra = Reader::new(&a_bytes);
    assert_eq!(CommandOriginV860::read(&mut ra).unwrap(), origin);

    let b_bytes = b.into_vec();
    let mut rb = Reader::new(&b_bytes);
    assert_eq!(CommandOriginV898::read(&mut rb).unwrap(), origin);
}

#[test]
fn v860_origin_carries_a_player_id_for_test_origins() {
    let origin = CommandOrigin {
        origin_type: TEST,
        uuid: MceUuid::default(),
        request_id: String::new(),
        player_id: 7,
    };
    let mut w = Writer::new();
    CommandOriginV860::write(&mut w, &origin);
    let bytes = w.into_vec();
    let mut r = Reader::new(&bytes);
    assert_eq!(CommandOriginV860::read(&mut r).unwrap(), origin);
    assert_eq!(r.remaining(), 0);
}
