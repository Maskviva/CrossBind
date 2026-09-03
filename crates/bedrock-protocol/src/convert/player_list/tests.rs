use crate::convert::player_list::codes::*;
use crate::convert::player_list::*;
use crate::ConnState;
#[cfg(test)]
use bedrock_codec::prelude::*;
use bedrock_codec::{PacketWrapper, Writer};

fn skin_bytes_v1001(w: &mut Writer) {
    Str::write(w, &"skin-id".to_string());
    Str::write(w, &"playfab".to_string());
    ByteArray::write(w, &b"{patch}".to_vec());
    w.write_u32_le(64);
    w.write_u32_le(32);
    ByteArray::write(w, &vec![1u8, 2, 3, 4]);
    w.write_u32_le(1);
    w.write_u32_le(8);
    w.write_u32_le(8);
    ByteArray::write(w, &vec![9u8]);
    w.write_u32_le(2);
    w.write_f32_le(1.5);
    w.write_u32_le(3);
    w.write_u32_le(0);
    w.write_u32_le(0);
    ByteArray::write(w, &Vec::new());
    ByteArray::write(w, &b"{geo}".to_vec());
    ByteArray::write(w, &b"1.8.0".to_vec());
    ByteArray::write(w, &b"{anim}".to_vec());
    Str::write(w, &"cape-id".to_string());
    Str::write(w, &"full-id".to_string());
    Str::write(w, &"wide".to_string());
    Str::write(w, &"#b37b62".to_string());
    w.write_u32_le(1);
    Str::write(w, &"piece-id".to_string());
    Str::write(w, &"persona_eyes".to_string());
    Str::write(w, &"3d29a1a4-1c1e-4e4d-9b2f-000000000001".to_string());
    w.write_bool(true);
    Str::write(w, &"product".to_string());
    w.write_u32_le(1);
    Str::write(w, &"persona_eyes".to_string());
    w.write_u32_le(4);
    for c in ["#ffa12722", "#ff2f1f0f", "#ff3aafd9", "#ff000000"] {
        Str::write(w, &c.to_string());
    }
    w.write_bool(false);
    w.write_bool(true);
    w.write_bool(false);
    w.write_bool(true);
    w.write_bool(false);
}

fn v1001_add() -> Vec<u8> {
    let mut w = Writer::new();
    w.write_u8(ACTION_ADD);
    w.write_count(1);
    Uuid::write(
        &mut w,
        &MceUuid {
            msb: 0x0123_4567_89ab_cdef,
            lsb: 0xfedc_ba98_7654_3210,
        },
    );
    w.write_varint64(42);
    Str::write(&mut w, &"RSxiaotong".to_string());
    Str::write(&mut w, &"2535424392028628".to_string());
    Str::write(&mut w, &String::new());
    w.write_i32_le(7);
    skin_bytes_v1001(&mut w);
    w.write_bool(false);
    w.write_bool(true);
    w.write_bool(false);
    w.write_u32_le(0xdd_cc_bb_aa);
    w.write_bool(true);
    w.into_vec()
}

#[test]
fn v1001_add_becomes_a_v2168_entry_with_the_name_intact() {
    let input = v1001_add();
    let mut state = ConnState::new(975);
    let mut wrapper = PacketWrapper::new(&input);
    assert!(player_list(&mut wrapper, &mut state, true).unwrap());
    assert!(state.notices.is_empty());

    let out = wrapper.finish();
    let mut r = Reader::new(&out);
    assert_eq!(r.read_count().unwrap(), 1);
    assert_eq!(r.read_uvarint().unwrap(), VARIANT_ADD);
    assert_eq!(r.read_u8().unwrap(), ACTION_ADD);
    Uuid::read(&mut r).unwrap();
    assert_eq!(r.read_varint64().unwrap(), 42);
    assert_eq!(Str::read(&mut r).unwrap(), "RSxiaotong");
    assert_eq!(Str::read(&mut r).unwrap(), "2535424392028628");
}

#[test]
fn add_round_trips_through_v2168_and_back() {
    let input = v1001_add();
    let mut state = ConnState::new(975);

    let mut up = PacketWrapper::new(&input);
    assert!(player_list(&mut up, &mut state, true).unwrap());
    let v2168_bytes = up.finish();

    let mut down = PacketWrapper::new(&v2168_bytes);
    assert!(player_list(&mut down, &mut state, false).unwrap());
    assert_eq!(
        down.finish(),
        input,
        "v1001 -> v2168 -> v1001 must be exact"
    );
    assert!(state.notices.is_empty());
}

#[test]
fn remove_lists_carry_only_uuids() {
    let mut w = Writer::new();
    w.write_u8(ACTION_REMOVE);
    w.write_count(2);
    Uuid::write(&mut w, &MceUuid { msb: 1, lsb: 2 });
    Uuid::write(&mut w, &MceUuid { msb: 3, lsb: 4 });
    let input = w.into_vec();

    let mut state = ConnState::new(975);
    let mut wrapper = PacketWrapper::new(&input);
    assert!(player_list(&mut wrapper, &mut state, true).unwrap());
    let out = wrapper.finish();

    let mut r = Reader::new(&out);
    assert_eq!(r.read_count().unwrap(), 2);
    for expected in [MceUuid { msb: 1, lsb: 2 }, MceUuid { msb: 3, lsb: 4 }] {
        assert_eq!(r.read_uvarint().unwrap(), VARIANT_REMOVE);
        assert_eq!(r.read_u8().unwrap(), ACTION_REMOVE);
        assert_eq!(Uuid::read(&mut r).unwrap(), expected);
    }
    assert!(
        !r.has_remaining(),
        "a removal carries nothing past the UUID"
    );

    let mut down = PacketWrapper::new(&out);
    assert!(player_list(&mut down, &mut state, false).unwrap());
    assert_eq!(down.finish(), input);
}

#[test]
fn player_colour_is_a_byte_reversal_not_a_repack() {
    let input = v1001_add();
    let mut state = ConnState::new(975);
    let mut up = PacketWrapper::new(&input);
    player_list(&mut up, &mut state, true).unwrap();
    let out = up.finish();
    let tail = &out[out.len() - 4..];
    assert_eq!(tail, &[0xdd, 0xcc, 0xbb, 0xaa]);
}

#[test]
fn a_mixed_v2168_list_is_cancelled_rather_than_half_applied() {
    let mut w = Writer::new();
    w.write_count(2);
    w.write_uvarint(VARIANT_REMOVE);
    w.write_u8(ACTION_REMOVE);
    Uuid::write(&mut w, &MceUuid { msb: 1, lsb: 2 });
    w.write_uvarint(VARIANT_ADD);
    w.write_u8(ACTION_ADD);
    Uuid::write(&mut w, &MceUuid { msb: 3, lsb: 4 });
    w.write_varint64(1);
    Str::write(&mut w, &"Someone".to_string());
    Str::write(&mut w, &String::new());
    Str::write(&mut w, &String::new());
    w.write_i32_le(0);
    let mut skin = Writer::new();
    skin_bytes_v1001(&mut skin);
    let _ = skin;
    let input = w.into_vec();

    let mut state = ConnState::new(2168);
    let mut wrapper = PacketWrapper::new(&input);
    assert!(!player_list(&mut wrapper, &mut state, false).unwrap());
    assert_eq!(state.notices.len(), 1);
}

#[test]
fn colour_and_uuid_helpers_round_trip() {
    assert_eq!(format_rgb_colour(parse_hex_colour("#b37b62")), "#b37b62");
    assert_eq!(
        format_argb_colour(parse_hex_colour("#ffa12722")),
        "#ffa12722"
    );
    assert_eq!(format_argb_colour(parse_hex_colour("#0")), "#0");
    assert_eq!(parse_hex_colour("#0"), 0);
    assert_eq!(parse_hex_colour(""), 0);
    let id = "3d29a1a4-1c1e-4e4d-9b2f-000000000001";
    assert_eq!(format_uuid(&parse_uuid(id)), id);
    assert_eq!(format_uuid(&parse_uuid("not a uuid")), "");
}

#[test]
fn persona_piece_names_map_onto_the_documented_enum_positions() {
    for (name, expected) in [
        ("persona_skeleton", 1u32),
        ("persona_body", 2),
        ("persona_skin", 3),
        ("persona_bottom", 4),
        ("persona_feet", 5),
        ("persona_top", 7),
        ("persona_mouth", 12),
        ("persona_eyes", 13),
        ("persona_hair", 14),
        ("persona_facial_hair", 11),
    ] {
        assert_eq!(piece_type_to_v2168(name), expected, "{name}");
        assert_eq!(piece_type_to_v1001(expected), name);
    }
    assert_eq!(piece_type_to_v2168("persona_something_new"), 0);
}

#[test]
fn tint_piece_names_lose_and_regain_their_prefix() {
    assert_eq!(tint_type_to_v2168("persona_eyes"), "eyes");
    assert_eq!(tint_type_to_v1001("eyes"), "persona_eyes");
    assert_eq!(tint_type_to_v2168("persona_hand"), "hands");
    assert_eq!(tint_type_to_v1001("hands"), "persona_hand");
    assert_eq!(tint_type_to_v1001("unsupported"), "unsupported");
}
