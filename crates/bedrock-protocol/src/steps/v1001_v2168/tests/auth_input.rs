use super::*;
use crate::steps::v1001_v2168::auth_input::*;
use crate::steps::v1001_v2168::bits::write_bitset;
use bedrock_codec::prelude::*;
use bedrock_codec::PacketWrapper;

#[allow(unused)]
const SUB_CHUNK_RESULT_LEVEL_CHUNK_DOESNT_EXIST: u8 = 2;

fn auth_input_v1001(flags: &[u32]) -> Vec<u8> {
    let mut w = PacketWrapper::new(&[]);
    for _ in 0..2 {
        w.write::<FloatLe>(1.5);
    }
    w.write::<Vec3>((1.0, 2.0, 3.0));
    w.write::<Vec2>((0.5, 0.25));
    w.write::<FloatLe>(0.75);
    write_bitset(&mut w, flags, INPUT_FLAG_BITSET_SIZE_V1001);
    w.writer().write_uvarint(1);
    w.writer().write_uvarint(0);
    w.writer().write_uvarint(2);
    w.write::<FloatLe>(0.0);
    w.write::<FloatLe>(0.0);
    w.writer().write_uvarint64(1234);
    w.write::<Vec3>((0.0, 0.0, 0.0));
    if flags.contains(&FLAG_CLIENT_PREDICTED_VEHICLE) {
        w.write::<Vec2>((9.0, 8.0));
        w.writer().write_varint64(-77);
    }
    w.write::<Vec2>((0.0, 0.0));
    w.write::<Vec3>((0.0, 0.0, 0.0));
    w.write::<Vec2>((0.0, 0.0));
    w.finish()
}

#[test]
fn auth_input_up_then_down_is_the_identity() {
    for flags in [vec![], vec![0u32, 12], vec![FLAG_CLIENT_PREDICTED_VEHICLE]] {
        let original = auth_input_v1001(&flags);
        let widened = run(player_auth_input_to_v2168, &original);
        let back = run(|w| player_auth_input_to_v1001(w, None), &widened);
        assert_eq!(back, original, "round trip failed for flags {flags:?}");
    }
}

#[test]
fn block_actions_reach_the_server_and_set_their_own_flag() {
    let input = v2168_auth_input(
        &[],
        &[
            (PLAYER_ACTION_START_BREAK, (10, 64, -3), 1),
            (8, (0, 0, 0), 0),
        ],
        false,
    );
    let out = run(|w| player_auth_input_to_v1001(w, None), &input);

    let mut r = Reader::new(&out);
    for _ in 0..8 {
        r.read_f32_le().unwrap();
    }
    let mut set = Vec::new();
    let mut base = 0u32;
    loop {
        let byte = r.read_u8().unwrap();
        for bit in 0..7u32 {
            if byte & (1 << bit) != 0 {
                set.push(base + bit);
            }
        }
        if byte & 0x80 == 0 {
            break;
        }
        base += 7;
    }
    assert_eq!(set, vec![FLAG_PERFORM_BLOCK_ACTIONS]);

    assert_eq!(r.read_uvarint().unwrap(), 1);
    assert_eq!(r.read_uvarint().unwrap(), 0);
    assert_eq!(r.read_uvarint().unwrap(), 0);
    r.read_f32_le().unwrap();
    r.read_f32_le().unwrap();
    assert_eq!(r.read_uvarint64().unwrap(), 1234);
    for _ in 0..3 {
        r.read_f32_le().unwrap();
    }

    assert_eq!(r.read_varint().unwrap(), 2);
    assert_eq!(r.read_varint().unwrap(), PLAYER_ACTION_START_BREAK);
    assert_eq!(BlockPos::read(&mut r).unwrap(), (10, 64, -3));
    assert_eq!(r.read_varint().unwrap(), 1);
    assert_eq!(r.read_varint().unwrap(), 8);
    assert_eq!(r.read_bytes(2).unwrap(), &[0xAA, 0xBB], "tail survives");
    assert_eq!(r.remaining(), 0);
}

#[test]
fn a_flag_without_its_payload_is_cleared() {
    let input = v2168_auth_input(&[FLAG_PERFORM_BLOCK_ACTIONS], &[], false);
    let out = run(|w| player_auth_input_to_v1001(w, None), &input);

    let mut r = Reader::new(&out);
    for _ in 0..8 {
        r.read_f32_le().unwrap();
    }
    assert_eq!(r.read_u8().unwrap(), 0, "no flags, one empty bitset group");
    assert_eq!(r.read_uvarint().unwrap(), 1);
}

#[test]
fn a_tick_carrying_an_item_stack_request_is_dropped() {
    let input = v2168_auth_input(&[], &[], true);
    let mut wrapper = PacketWrapper::new(&input);
    player_auth_input_to_v1001(&mut wrapper, None).expect("handler failed");
    assert!(wrapper.is_cancelled());
}
