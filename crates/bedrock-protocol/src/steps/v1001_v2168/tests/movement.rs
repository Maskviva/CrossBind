use super::*;
use crate::steps::v1001_v2168::movement::*;
use bedrock_codec::prelude::*;
use bedrock_codec::PacketWrapper;

#[allow(unused)]
const SUB_CHUNK_RESULT_LEVEL_CHUNK_DOESNT_EXIST: u8 = 2;

#[test]
fn move_delta_up_then_down_is_the_identity() {
    for flags in [
        0u16,
        MOVE_DELTA_HAS_X | MOVE_DELTA_HAS_Z,
        MOVE_DELTA_HAS_X | MOVE_DELTA_HAS_Y | MOVE_DELTA_HAS_Z | MOVE_DELTA_ON_GROUND,
        MOVE_DELTA_HAS_ROT_Y | MOVE_DELTA_FORCE_MOVE,
    ] {
        let mut w = PacketWrapper::new(&[]);
        w.write::<UVarInt64>(42);
        w.writer().write_u16_le(flags);
        for bit in [MOVE_DELTA_HAS_X, MOVE_DELTA_HAS_Y, MOVE_DELTA_HAS_Z] {
            if flags & bit != 0 {
                w.writer().write_f32_le(3.5);
            }
        }
        for bit in [
            MOVE_DELTA_HAS_ROT_X,
            MOVE_DELTA_HAS_ROT_Y,
            MOVE_DELTA_HAS_ROT_Z,
        ] {
            if flags & bit != 0 {
                w.writer().write_u8(200);
            }
        }
        let original = w.finish();

        let widened = run(|w| move_delta_actor(w, true), &original);
        let back = run(|w| move_delta_actor(w, false), &widened);
        assert_eq!(back, original, "round trip failed for flags {flags:#x}");
    }
}
