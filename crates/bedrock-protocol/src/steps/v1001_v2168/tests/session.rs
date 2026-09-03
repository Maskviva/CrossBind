use super::*;
use crate::steps::v1001_v2168::session::server_join_information;
use bedrock_codec::prelude::*;
use bedrock_codec::PacketWrapper;

#[allow(unused)]
const SUB_CHUNK_RESULT_LEVEL_CHUNK_DOESNT_EXIST: u8 = 2;

#[test]
fn pack_response_round_trips_and_shifts_the_enum() {
    let mut w = PacketWrapper::new(&[]);
    w.write::<Byte>(2);
    w.write::<UShortLe>(2);
    w.write::<Str>("a_1.0.0".into());
    w.write::<Str>("b_2.0.0".into());
    let original = w.finish();

    let widened = run(|w| resource_pack_client_response(w, true), &original);
    assert_eq!(widened[0], 1, "v2168 numbering is one lower");

    let back = run(|w| resource_pack_client_response(w, false), &widened);
    assert_eq!(back, original);
}

#[test]
fn pack_response_without_packs_still_round_trips() {
    let mut w = PacketWrapper::new(&[]);
    w.write::<Byte>(4);
    w.write::<UShortLe>(0);
    let original = w.finish();
    let widened = run(|w| resource_pack_client_response(w, true), &original);
    let back = run(|w| resource_pack_client_response(w, false), &widened);
    assert_eq!(back, original);
}

#[test]
fn pack_response_rejects_an_out_of_range_value() {
    let body = [0u8, 0, 0];
    let mut w = PacketWrapper::new(&body);
    assert!(resource_pack_client_response(&mut w, true).is_err());
}

#[test]
fn join_info_up_then_down_is_the_identity() {
    let original = join_info_v1001();
    let widened = run(|w| server_join_information(w, true), &original);
    assert_ne!(widened, original, "the optionals must change the encoding");
    let back = run(|w| server_join_information(w, false), &widened);
    assert_eq!(back, original);
}

#[test]
fn an_absent_join_info_leaves_the_trailing_ids_alone() {
    let ids: [u8; 4] = [0, 0, 0, 0];

    for inner in [vec![0u8], vec![1u8, 0, 0, 0]] {
        let mut original = inner.clone();
        original.extend_from_slice(&ids);

        for to_v2168 in [true, false] {
            let mut w = PacketWrapper::new(&original);
            server_join_information(&mut w, to_v2168).expect("join info");
            w.passthrough_all();
            let out = w.finish();
            assert_eq!(
                out, original,
                "absent join info must be byte-identical in both directions"
            );
        }

        let mut w = PacketWrapper::new(&original);
        server_join_information(&mut w, true).expect("join info");
        assert_eq!(
            w.finish().len(),
            inner.len() + ids.len(),
            "handler must not consume the trailing IDs"
        );
    }
}

#[test]
fn dimension_data_gains_and_loses_the_pack_id() {
    let mut w = PacketWrapper::new(&[]);
    w.write::<UVarInt>(1);
    w.write::<Str>("overworld".into());
    for v in [320i32, -64, 0, 0] {
        w.write::<VarInt>(v);
    }
    let original = w.finish();
    let widened = run(|w| dimension_data(w, true), &original);
    assert_eq!(widened.len(), original.len() + 16);
    let back = run(|w| dimension_data(w, false), &widened);
    assert_eq!(back, original);
}
