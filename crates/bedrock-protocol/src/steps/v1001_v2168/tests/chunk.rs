use super::*;
use bedrock_codec::prelude::*;
use bedrock_codec::PacketWrapper;

const SUB_CHUNK_RESULT_LEVEL_CHUNK_DOESNT_EXIST: u8 = 2;

#[test]
fn full_chunk_data_round_trips_each_request_mode() {
    for count in [3u32, SUB_CHUNK_MODE_LIMITED, SUB_CHUNK_MODE_LIMITLESS] {
        for cache in [false, true] {
            let mut w = PacketWrapper::new(&[]);
            w.write::<VarInt>(4);
            w.write::<VarInt>(-9);
            w.write::<VarInt>(0);
            w.writer().write_uvarint(count);
            if count == SUB_CHUNK_MODE_LIMITED {
                w.writer().write_u16_le(7);
            }
            w.write::<Bool>(cache);
            if cache {
                w.writer().write_count(2);
                w.write::<UInt64Le>(11);
                w.write::<UInt64Le>(22);
            }
            w.writer().write_count(3);
            w.writer().write_bytes(&[1, 2, 3]);
            let original = w.finish();

            let widened = run(|w| full_chunk_data(w, true), &original);
            let back = run(|w| full_chunk_data(w, false), &widened);
            assert_eq!(back, original, "count {count:#x} cache {cache}");
        }
    }
}

fn v1001_entry(w: &mut Writer, result: u8, payload: &[u8], height_map: bool) {
    w.write_i8(0);
    w.write_i8(1);
    w.write_i8(-1);
    w.write_u8(result);
    if result != SUB_CHUNK_RESULT_SUCCESS_ALL_AIR {
        w.write_count(payload.len());
        w.write_bytes(payload);
    }
    if height_map {
        w.write_u8(HEIGHT_MAP_HAS_DATA);
        w.write_bytes(&[3u8; HEIGHT_MAP_LEN]);
    } else {
        w.write_u8(0);
    }
    w.write_u8(0);
    w.write_u64_le(0xDEAD_BEEF_CAFE_F00D);
}

fn v1001_sub_chunk(entries: &[(u8, Vec<u8>, bool)]) -> Vec<u8> {
    let mut w = Writer::new();
    w.write_bool(true);
    w.write_varint(1000);
    w.write_varint(-5);
    w.write_varint(2);
    w.write_varint(-4);
    w.write_u32_le(entries.len() as u32);
    for (result, payload, height_map) in entries {
        v1001_entry(&mut w, *result, payload, *height_map);
    }
    w.into_vec()
}

#[test]
fn sub_chunk_round_trips_a_mixed_entry_list() {
    let original = v1001_sub_chunk(&[
        (1, vec![9; 40], true),
        (SUB_CHUNK_RESULT_SUCCESS_ALL_AIR, vec![], false),
        (1, vec![7; 12], false),
    ]);

    let widened = run(|w| sub_chunk(w, true, SubChunkShape::DOCUMENTED), &original);
    let back = run(|w| sub_chunk(w, false, SubChunkShape::DOCUMENTED), &widened);
    assert_eq!(back, original);
}

#[test]
fn sub_chunk_position_widens_and_the_count_narrows() {
    let original = v1001_sub_chunk(&[(1, vec![1, 2, 3], false)]);
    let widened = run(|w| sub_chunk(w, true, SubChunkShape::DOCUMENTED), &original);

    assert_eq!(widened.len(), original.len() + 9 - 3 + 4);

    assert_eq!(
        &widened[3..15],
        &[0xFB, 0xFF, 0xFF, 0xFF, 0x02, 0x00, 0x00, 0x00, 0xFC, 0xFF, 0xFF, 0xFF,]
    );
    assert_eq!(widened[15], 0x01, "entry count is a uvarint now");
}

#[test]
fn sub_chunk_without_the_cache_has_no_blob_hash_on_the_v1001_side() {
    let mut w = Writer::new();
    w.write_bool(false);
    w.write_varint(0);
    w.write_varint(0);
    w.write_varint(0);
    w.write_varint(0);
    w.write_u32_le(2);
    for _ in 0..2 {
        w.write_i8(0);
        w.write_i8(0);
        w.write_i8(0);
        w.write_u8(SUB_CHUNK_RESULT_SUCCESS_ALL_AIR);
        w.write_count(2);
        w.write_bytes(&[4, 5]);
        w.write_u8(0);
        w.write_u8(0);
    }
    let original = w.into_vec();

    let widened = run(|w| sub_chunk(w, true, SubChunkShape::DOCUMENTED), &original);
    let back = run(|w| sub_chunk(w, false, SubChunkShape::DOCUMENTED), &widened);
    assert_eq!(back, original);
}

#[test]
fn every_sub_chunk_shape_round_trips() {
    let original = v1001_sub_chunk(&[
        (1, vec![9; 40], true),
        (SUB_CHUNK_RESULT_SUCCESS_ALL_AIR, vec![], false),
        (1, vec![7; 12], false),
    ]);
    for type_bytes in [true, false] {
        for announce_blob_hash in [true, false] {
            for zero_blob_hash in [true, false] {
                for empty_payload in [true, false] {
                    let shape = SubChunkShape {
                        type_bytes,
                        announce_blob_hash,
                        zero_blob_hash,
                        empty_payload,
                        strip_content: false,
                    };
                    let widened = run(|w| sub_chunk(w, true, shape), &original);
                    let back = run(|w| sub_chunk(w, false, shape), &widened);
                    if announce_blob_hash {
                        assert_eq!(back, original, "shape {shape:?} is not its own inverse");
                    } else {
                        assert_eq!(
                            back.len(),
                            original.len(),
                            "shape {shape:?} moved something other than the hash"
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn air_mode_empties_every_entry_and_keeps_the_framing() {
    let original = v1001_sub_chunk(&[
        (1, vec![9; 40], true),
        (SUB_CHUNK_RESULT_SUCCESS_ALL_AIR, vec![], false),
        (1, vec![7; 12], false),
    ]);
    let air = SubChunkShape {
        strip_content: true,
        ..SubChunkShape::DOCUMENTED
    };
    let stripped = run(|w| sub_chunk(w, true, air), &original);

    assert_eq!(stripped.len(), 3 + 12 + 1 + 3 * 10);
    for entry in 0..3 {
        let at = 16 + entry * 10;
        assert_eq!(
            &stripped[at..at + 10],
            &[
                stripped[at],
                stripped[at + 1],
                stripped[at + 2],
                SUB_CHUNK_RESULT_SUCCESS_ALL_AIR,
                0,
                HEIGHT_MAP_NONE,
                0,
                HEIGHT_MAP_NONE,
                0,
                0,
            ]
        );
    }
}

#[test]
fn sub_chunk_matches_the_captured_packet_byte_for_byte() {
    let entry = |y: u8| [0xfe, y, 0x03, 0x06, 0x02, 0x02, 0, 0, 0, 0, 0, 0, 0, 0];
    let mut input = vec![0x01, 0xd0, 0x0f, 0x07, 0x00, 0x09, 0x02, 0x00, 0x00, 0x00];
    input.extend_from_slice(&entry(0xe0));
    input.extend_from_slice(&entry(0xe1));

    let widened = run(|w| sub_chunk(w, true, SubChunkShape::DOCUMENTED), &input);

    let out_entry = |y: u8| {
        [
            0xfe, y, 0x03, 0x06, 0x00, 0x02, 0x00, 0x02, 0x00, 0x01, 0, 0, 0, 0, 0, 0, 0, 0,
        ]
    };
    let mut expected = vec![
        0x01, 0xd0, 0x0f, 0xfc, 0xff, 0xff, 0xff, 0x00, 0x00, 0x00, 0x00, 0xfb, 0xff, 0xff, 0xff,
        0x02,
    ];
    expected.extend_from_slice(&out_entry(0xe0));
    expected.extend_from_slice(&out_entry(0xe1));
    assert_eq!(widened, expected);

    let lean = run(
        |w| {
            sub_chunk(
                w,
                true,
                SubChunkShape {
                    type_bytes: false,
                    announce_blob_hash: true,
                    zero_blob_hash: false,
                    empty_payload: true,
                    strip_content: false,
                },
            )
        },
        &input,
    );
    assert_eq!(lean.len(), widened.len() - 2 * (2 + 8));
}

#[test]
fn the_default_shape_announces_a_real_hash_and_hides_a_zero_one() {
    const {
        assert!(
            SubChunkShape::DEFAULT.announce_blob_hash,
            "a v2168 client leaves the world when an entry with a payload \
             carries no blob hash; see the capture in the devdocs"
        );
        assert!(!SubChunkShape::DEFAULT.zero_blob_hash);
        assert!(SubChunkShape::DEFAULT.type_bytes);
        assert!(!SubChunkShape::DEFAULT.strip_content);
    }

    let original = v1001_sub_chunk(&[(SUB_CHUNK_RESULT_SUCCESS_ALL_AIR, vec![], false)]);
    let kept = run(|w| sub_chunk(w, true, SubChunkShape::DEFAULT), &original);
    assert_eq!(kept[kept.len() - 9], 0x01, "a real hash stays present");

    let mut all_air = original.clone();
    let len = all_air.len();
    all_air[len - 8..].fill(0);
    let dropped = run(|w| sub_chunk(w, true, SubChunkShape::DEFAULT), &all_air);
    assert_eq!(*dropped.last().unwrap(), 0x00, "a zero hash goes absent");
    assert_eq!(dropped.len(), kept.len() - 8);
}

#[test]
fn an_empty_payload_is_written_absent_by_default() {
    const {
        assert!(!SubChunkShape::DEFAULT.empty_payload);
        assert!(SubChunkShape::SUPPRESS_ZERO_HASH.empty_payload);
    }

    let mut w = Writer::new();
    w.write_bool(true);
    w.write_varint(1000);
    w.write_varint(0);
    w.write_varint(0);
    w.write_varint(0);
    w.write_u32_le(1);
    w.write_i8(-114);
    w.write_i8(12);
    w.write_i8(-114);
    w.write_u8(SUB_CHUNK_RESULT_LEVEL_CHUNK_DOESNT_EXIST);
    w.write_count(0);
    w.write_u8(HEIGHT_MAP_NONE);
    w.write_u8(HEIGHT_MAP_NONE);
    w.write_u64_le(0);
    let original = w.into_vec();

    let absent = run(|w| sub_chunk(w, true, SubChunkShape::DEFAULT), &original);
    let announced = run(
        |w| sub_chunk(w, true, SubChunkShape::SUPPRESS_ZERO_HASH),
        &original,
    );

    assert_eq!(
        absent.len(),
        announced.len() - 1,
        "the only difference is the zero-length byte array behind the presence bool"
    );
    assert_eq!(
        absent[20], 0x00,
        "a v975 chunk-doesn't-exist entry hands over no sub-chunk, so the optional is None"
    );
    assert_eq!(announced[20], 0x01);
    assert_eq!(absent[19], SUB_CHUNK_RESULT_LEVEL_CHUNK_DOESNT_EXIST);

    let back = run(|w| sub_chunk(w, false, SubChunkShape::DEFAULT), &absent);
    assert_eq!(
        back, original,
        "None widens back into the empty array v975 wrote"
    );
}

#[test]
fn a_real_payload_survives_the_empty_payload_rule() {
    let original = v1001_sub_chunk(&[(1, vec![9; 40], true), (1, vec![7; 12], false)]);
    let widened = run(|w| sub_chunk(w, true, SubChunkShape::DEFAULT), &original);
    let back = run(|w| sub_chunk(w, false, SubChunkShape::DEFAULT), &widened);
    assert_eq!(back, original);
}

#[test]
fn mode_e_drops_every_hash_and_is_not_the_default() {
    const {
        assert!(!SubChunkShape::NO_BLOB_HASH.announce_blob_hash);
    }
    assert_ne!(
        SubChunkShape::DEFAULT,
        SubChunkShape::NO_BLOB_HASH,
        "mode e is opt-in: it costs 8 bytes per entry and a v2168 client \
         refuses the result"
    );

    let original = v1001_sub_chunk(&[(SUB_CHUNK_RESULT_SUCCESS_ALL_AIR, vec![], false)]);
    let lean = run(
        |w| sub_chunk(w, true, SubChunkShape::NO_BLOB_HASH),
        &original,
    );
    let announced = run(|w| sub_chunk(w, true, SubChunkShape::DEFAULT), &original);

    assert_eq!(*lean.last().unwrap(), 0x00);
    assert_eq!(
        lean.len(),
        announced.len() - 8,
        "8 bytes per entry, which is what the -617 in the capture was"
    );

    let back = run(|w| sub_chunk(w, false, SubChunkShape::NO_BLOB_HASH), &lean);
    assert_eq!(
        back.len(),
        original.len(),
        "only the hash is lost; the framing has to survive"
    );
}
