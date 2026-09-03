use super::*;
use crate::ConnState;
use bedrock_codec::{PacketWrapper, Reader, Writer};

#[allow(unused)]
const SUB_CHUNK_RESULT_LEVEL_CHUNK_DOESNT_EXIST: u8 = 2;

#[test]
fn play_sound_never_asks_the_client_to_repeat_forever() {
    let mut w = Writer::new();
    w.write_string("mob.pig.say");
    w.write_varint(64);
    w.write_varint(72);
    w.write_varint(-16);
    w.write_f32_le(1.0);
    w.write_f32_le(1.0);
    w.write_bool(false);
    let v1001 = w.into_vec();

    let v2168 = run(|w| play_sound(w, true), &v1001);

    let mut r = Reader::new(&v2168);
    assert_eq!(r.read_string().expect("SoundName"), "mob.pig.say");
    for _ in 0..3 {
        r.read_varint().expect("Position");
    }
    r.read_f32_le().expect("Volume");
    r.read_f32_le().expect("Pitch");

    let loops = r.read_varint().expect("LoopCount");
    assert!(
        loops >= 0,
        "LoopCount {loops} is negative, which is the engine's \
         loop-forever sentinel: one /playsound would never stop"
    );
    if std::env::var_os("CROSSBIND_PLAYSOUND_LOOPS").is_none() {
        assert_eq!(loops, PLAY_SOUND_ONCE, "default has to be a single play");
    }

    assert!(!r.read_bool().expect("ServerSoundHandle"));
    assert!(!r.has_remaining());

    assert_eq!(run(|w| play_sound(w, false), &v2168), v1001);
}

#[test]
fn structure_block_update_reaches_the_server_instead_of_being_dropped() {
    assert!(
        !downgrade().is_cancelled(Direction::Serverbound, ids::STRUCTURE_BLOCK_UPDATE),
        "cancelling this is what made structure block edits revert on exit"
    );

    let mut state = ConnState::new(1001);
    let mut w = Writer::new();
    w.write_bytes(&[0x11, 0x22, 0x33, 0x44]);
    w.write_u8(1);
    w.write_bool(true);
    w.write_bool(false);
    let v2168 = w.into_vec();

    let mut wrapper = PacketWrapper::new(&v2168);
    structure_block_update(&mut wrapper, &mut state, false).expect("handler failed");
    assert!(!wrapper.is_cancelled());
    let v1001 = wrapper.finish();
    assert_eq!(v1001, vec![0x11, 0x22, 0x33, 0x44, 0x02, 0x01, 0x00]);

    let mut wrapper = PacketWrapper::new(&v1001);
    structure_block_update(&mut wrapper, &mut state, true).expect("handler failed");
    assert_eq!(wrapper.finish(), v2168);
    assert!(state.notices.is_empty());
}

#[test]
fn structure_block_update_keeps_the_length_it_was_given() {
    let mut state = ConnState::new(1001);
    for save_mode in 0..=STRUCTURE_REDSTONE_SAVE_MODE_MAX {
        for trigger in [false, true] {
            for waterlogged in [false, true] {
                let mut w = Writer::new();
                w.write_bytes(&[0xAA; 20]);
                w.write_u8(save_mode);
                w.write_bool(trigger);
                w.write_bool(waterlogged);
                let v2168 = w.into_vec();

                let mut wrapper = PacketWrapper::new(&v2168);
                structure_block_update(&mut wrapper, &mut state, false).expect("handler failed");
                let v1001 = wrapper.finish();
                assert_eq!(v1001.len(), v2168.len(), "save mode {save_mode}");
                assert_eq!(&v1001[..20], &v2168[..20], "the head must be untouched");

                let mut wrapper = PacketWrapper::new(&v1001);
                structure_block_update(&mut wrapper, &mut state, true).expect("handler failed");
                assert_eq!(wrapper.finish(), v2168, "save mode {save_mode}");
            }
        }
    }
    assert!(state.notices.is_empty());
}

#[test]
fn structure_block_update_with_an_unrecognised_tail_is_dropped() {
    let cases: [(&[u8], bool); 4] = [
        (&[0x11, 0x22, 0x07, 0x05, 0x00], false),
        (&[0x11, 0x22, 0x09, 0x00, 0x00], false),
        (&[0x11, 0x22, 0x01, 0x00, 0x00], true),
        (&[0x00, 0x01], false),
    ];
    for (body, to_v2168) in cases {
        let mut state = ConnState::new(1001);
        let mut wrapper = PacketWrapper::new(body);
        structure_block_update(&mut wrapper, &mut state, to_v2168).expect("handler failed");
        assert!(
            wrapper.is_cancelled(),
            "a tail this shape must fall back to dropping, not to a guess: {body:02x?}"
        );
        assert_eq!(state.notices.len(), 1, "and it has to say so once");
    }
}
