use super::*;
use bedrock_codec::prelude::*;
use bedrock_codec::PacketWrapper;

#[allow(unused)]
const SUB_CHUNK_RESULT_LEVEL_CHUNK_DOESNT_EXIST: u8 = 2;

#[test]
fn creative_content_round_trips_including_an_air_icon() {
    let mut w = PacketWrapper::new(&[]);
    w.write::<UVarInt>(2);
    for (category, air) in [(0i32, true), (3, false)] {
        w.write::<IntLe>(category);
        w.write::<Str>("itemGroup.name.planks".into());
        w.write::<NetworkItemInstanceDescriptor>(if air {
            Item::default()
        } else {
            Item {
                network_id: 5,
                count: 1,
                aux_value: 0,
                block_runtime_id: 7,
                ..Item::default()
            }
        });
    }
    w.write::<UVarInt>(1);
    w.write::<UVarInt>(101);
    w.write::<NetworkItemInstanceDescriptor>(Item {
        network_id: 9,
        count: 64,
        aux_value: 2,
        block_runtime_id: 0,
        ..Item::default()
    });
    w.write::<UVarInt>(1);
    let original = w.finish();

    let widened = run(|w| creative_content(w, true), &original);

    assert_eq!(
        widened.len(),
        original.len() - 2 * 3 + 5,
        "category narrowing and air widening must both show up in the length"
    );

    let back = run(|w| creative_content(w, false), &widened);
    assert_eq!(back, original);
}
