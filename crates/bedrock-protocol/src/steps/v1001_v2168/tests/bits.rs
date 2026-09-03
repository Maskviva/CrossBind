use crate::steps::v1001_v2168::auth_input::*;
use crate::steps::v1001_v2168::bits::*;
use bedrock_codec::PacketWrapper;

#[allow(unused)]
const SUB_CHUNK_RESULT_LEVEL_CHUNK_DOESNT_EXIST: u8 = 2;

#[test]
fn bitset_and_flag_list_round_trip() {
    for set in [
        vec![],
        vec![0u32],
        vec![6, 7],
        vec![0, 34, 64],
        vec![1, 2, 3, 45],
    ] {
        let mut w = PacketWrapper::new(&[]);
        write_bitset(&mut w, &set, INPUT_FLAG_BITSET_SIZE_V1001);
        let bytes = w.finish();
        let mut w = PacketWrapper::new(&bytes);
        let decoded = read_bitset(&mut w, INPUT_FLAG_BITSET_SIZE_V1001).unwrap();
        assert_eq!(decoded, set, "bitset round trip failed for {set:?}");
    }
}

#[test]
fn bitset_rejects_a_flag_past_the_end() {
    let body = [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x7F];
    let mut w = PacketWrapper::new(&body);
    assert!(read_bitset(&mut w, INPUT_FLAG_BITSET_SIZE_V1001).is_err());
}
