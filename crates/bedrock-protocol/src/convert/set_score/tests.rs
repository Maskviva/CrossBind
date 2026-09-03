#[cfg(test)]
use crate::convert::set_score::model::Identity;
use crate::convert::set_score::read::read_score_v2168;
use crate::convert::set_score::*;
use bedrock_codec::{Reader, Writer};

const GOLDEN_V1001_CHANGE: &[u8] = &[
    0x00, 0x03, 0x02, 0x03, 0x6F, 0x62, 0x6A, 0x05, 0x00, 0x00, 0x00, 0x01, 0x12, 0x04, 0x03, 0x6F,
    0x62, 0x6A, 0x06, 0x00, 0x00, 0x00, 0x02, 0x16, 0x06, 0x03, 0x6F, 0x62, 0x6A, 0x07, 0x00, 0x00,
    0x00, 0x03, 0x04, 0x66, 0x61, 0x6B, 0x65,
];

const GOLDEN_V1001_REMOVE: &[u8] = &[0x01, 0x01, 0x02, 0x02, 0x72, 0x6D, 0x00, 0x00, 0x00, 0x00];

const GOLDEN_V2168: &[u8] = &[
    0x04, 0x00, 0x06, 0x52, 0x65, 0x6D, 0x6F, 0x76, 0x65, 0x02, 0x01, 0x02, 0x72, 0x6D, 0x01, 0x0C,
    0x43, 0x68, 0x61, 0x6E, 0x67, 0x65, 0x50, 0x6C, 0x61, 0x79, 0x65, 0x72, 0x04, 0x03, 0x6F, 0x62,
    0x6A, 0x05, 0x00, 0x00, 0x00, 0x12, 0x02, 0x0C, 0x43, 0x68, 0x61, 0x6E, 0x67, 0x65, 0x45, 0x6E,
    0x74, 0x69, 0x74, 0x79, 0x06, 0x03, 0x6F, 0x62, 0x6A, 0x06, 0x00, 0x00, 0x00, 0x16, 0x03, 0x10,
    0x43, 0x68, 0x61, 0x6E, 0x67, 0x65, 0x46, 0x61, 0x6B, 0x65, 0x50, 0x6C, 0x61, 0x79, 0x65, 0x72,
    0x08, 0x03, 0x6F, 0x62, 0x6A, 0x07, 0x00, 0x00, 0x00, 0x04, 0x66, 0x61, 0x6B, 0x65,
];

const GOLDEN_IDENTITY_V1001_UPDATE: &[u8] = &[0x00, 0x02, 0x02, 0x0E, 0x04, 0x10];
const GOLDEN_IDENTITY_V1001_REMOVE: &[u8] = &[0x01, 0x01, 0x02];
const GOLDEN_IDENTITY_V2168_UPDATE: &[u8] = &[0x00, 0x02, 0x02, 0x01, 0x0E, 0x04, 0x01, 0x10];
const GOLDEN_IDENTITY_V2168_REMOVE: &[u8] = &[0x01, 0x01, 0x02, 0x00];

fn run(input: &[u8], to_v2168: bool, score: bool) -> Option<Vec<u8>> {
    let mut w = PacketWrapper::new(input);
    let body: fn(&mut PacketWrapper, bool) -> Result<bool> =
        if score { score_body } else { identity_body };
    match body(&mut w, to_v2168) {
        Ok(true) => Some(w.finish()),
        Ok(false) => None,
        Err(err) => panic!("failed to translate: {err}"),
    }
}

#[test]
fn the_v2168_writer_reproduces_the_reference_bytes() {
    let entries = read_score_v2168(&mut Reader::new(GOLDEN_V2168)).expect("decode golden");
    assert_eq!(entries.len(), 4);
    let mut out = Writer::new();
    write_score_v2168(&mut out, &entries);
    assert_eq!(out.into_vec(), GOLDEN_V2168);
}

#[test]
fn the_v2168_reader_pulls_the_reference_fields_out() {
    let entries = read_score_v2168(&mut Reader::new(GOLDEN_V2168)).expect("decode golden");
    assert_eq!(entries[0].scoreboard_id, 1);
    assert_eq!(entries[0].objective.as_deref(), Some("rm"));
    assert!(entries[0].identity.is_none());
    assert!(matches!(entries[1].identity, Some(Identity::Player(9))));
    assert_eq!(entries[1].score, 5);
    assert!(matches!(entries[2].identity, Some(Identity::Entity(11))));
    match entries[3].identity.as_ref().expect("fake player") {
        Identity::FakePlayer(name) => assert_eq!(name, "fake"),
        _ => panic!("entry 3 is not a fake player"),
    }
    assert_eq!(entries[3].scoreboard_id, 4);
    assert_eq!(entries[3].score, 7);
}

#[test]
fn a_v1001_change_packet_upgrades_to_the_reference_shape() {
    let up = run(GOLDEN_V1001_CHANGE, true, true).expect("upgrade");
    let mut expected = Writer::new();
    expected.write_count(3);
    expected.write_uvarint(VARIANT_CHANGE_PLAYER);
    expected.write_string(NAME_CHANGE_PLAYER);
    expected.write_varint64(1);
    expected.write_string("obj");
    expected.write_i32_le(5);
    expected.write_varint64(9);
    expected.write_uvarint(VARIANT_CHANGE_ENTITY);
    expected.write_string(NAME_CHANGE_ENTITY);
    expected.write_varint64(2);
    expected.write_string("obj");
    expected.write_i32_le(6);
    expected.write_varint64(11);
    expected.write_uvarint(VARIANT_CHANGE_FAKE_PLAYER);
    expected.write_string(NAME_CHANGE_FAKE_PLAYER);
    expected.write_varint64(3);
    expected.write_string("obj");
    expected.write_i32_le(7);
    expected.write_string("fake");
    assert_eq!(up, expected.into_vec());
    assert_eq!(
        run(&up, false, true).expect("downgrade"),
        GOLDEN_V1001_CHANGE
    );
}

#[test]
fn a_v1001_remove_packet_round_trips() {
    let up = run(GOLDEN_V1001_REMOVE, true, true).expect("upgrade");
    let mut expected = Writer::new();
    expected.write_count(1);
    expected.write_uvarint(VARIANT_REMOVE);
    expected.write_string(NAME_REMOVE);
    expected.write_varint64(1);
    expected.write_bool(true);
    expected.write_string("rm");
    assert_eq!(up, expected.into_vec());
    assert_eq!(
        run(&up, false, true).expect("downgrade"),
        GOLDEN_V1001_REMOVE
    );
}

#[test]
fn a_real_sidebar_entry_upgrades_byte_for_byte() {
    let mut w = Writer::new();
    w.write_u8(ACTION_CHANGE);
    w.write_count(1);
    w.write_varint64(1);
    w.write_string("rcc_bar");
    w.write_i32_le(12);
    w.write_u8(IDENTITY_FAKE_PLAYER);
    w.write_string("§f§l个人信息");
    let captured = w.into_vec();

    let up = run(&captured, true, true).expect("upgrade");
    let mut expected = Writer::new();
    expected.write_count(1);
    expected.write_uvarint(VARIANT_CHANGE_FAKE_PLAYER);
    expected.write_string(NAME_CHANGE_FAKE_PLAYER);
    expected.write_varint64(1);
    expected.write_string("rcc_bar");
    expected.write_i32_le(12);
    expected.write_string("§f§l个人信息");
    assert_eq!(up, expected.into_vec());
    assert_eq!(run(&up, false, true).expect("downgrade"), captured);
}

#[test]
fn neither_body_survives_the_other_version_reader() {
    let mut w = PacketWrapper::new(GOLDEN_V1001_CHANGE);
    assert!(
        score_body(&mut w, false).is_err(),
        "a v1001 body read as v2168 gives count 0 and a tail of leftover bytes"
    );
    let mut w = PacketWrapper::new(GOLDEN_V2168);
    assert!(score_body(&mut w, true).is_err());
}

#[test]
fn a_mixed_v2168_list_cannot_go_down() {
    assert!(run(GOLDEN_V2168, false, true).is_none());
}

#[test]
fn a_removal_without_an_objective_name_cannot_go_down() {
    let mut w = Writer::new();
    w.write_count(1);
    w.write_uvarint(VARIANT_REMOVE);
    w.write_string(NAME_REMOVE);
    w.write_varint64(1);
    w.write_bool(false);
    assert!(run(&w.into_vec(), false, true).is_none());
}

#[test]
fn an_action_name_that_disagrees_with_its_variant_is_refused() {
    let mut w = Writer::new();
    w.write_count(1);
    w.write_uvarint(VARIANT_CHANGE_FAKE_PLAYER);
    w.write_string(NAME_CHANGE_PLAYER);
    w.write_varint64(1);
    assert!(read_score_v2168(&mut Reader::new(&w.into_vec())).is_err());
}

#[test]
fn the_action_name_is_matched_case_insensitively() {
    let mut w = Writer::new();
    w.write_count(1);
    w.write_uvarint(VARIANT_CHANGE_FAKE_PLAYER);
    w.write_string("changefakeplayer");
    w.write_varint64(1);
    w.write_string("obj");
    w.write_i32_le(3);
    w.write_string("fake");
    let entries = read_score_v2168(&mut Reader::new(&w.into_vec())).expect("lower case name");
    assert_eq!(entries.len(), 1);
}

#[test]
fn scoreboard_identities_match_the_reference_bytes_both_ways() {
    assert_eq!(
        run(GOLDEN_IDENTITY_V1001_UPDATE, true, false).expect("update up"),
        GOLDEN_IDENTITY_V2168_UPDATE
    );
    assert_eq!(
        run(GOLDEN_IDENTITY_V2168_UPDATE, false, false).expect("update down"),
        GOLDEN_IDENTITY_V1001_UPDATE
    );
    assert_eq!(
        run(GOLDEN_IDENTITY_V1001_REMOVE, true, false).expect("remove up"),
        GOLDEN_IDENTITY_V2168_REMOVE
    );
    assert_eq!(
        run(GOLDEN_IDENTITY_V2168_REMOVE, false, false).expect("remove down"),
        GOLDEN_IDENTITY_V1001_REMOVE
    );
}

#[test]
fn only_an_explicit_off_stops_the_translation() {
    assert!(parse_enabled(None));
    assert!(parse_enabled(Some("")));
    assert!(parse_enabled(Some("on")));
    assert!(parse_enabled(Some("probe")));
    assert!(!parse_enabled(Some("off")));
    assert!(!parse_enabled(Some(" OFF ")));
    assert!(!parse_enabled(Some("drop")));
}
