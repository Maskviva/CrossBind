use super::*;

#[test]
fn a_response_round_trips_and_moves_the_optional() {
    let mut w = Writer::new();
    w.write_count(1);
    w.write_u8(RESPONSE_STATUS_OK);
    w.write_varint(-3);
    w.write_count(1);
    w.write_u8(0);
    Optional::<UIntLe>::write(&mut w, &None);
    w.write_count(1);
    w.write_u8(50);
    w.write_u8(50);
    w.write_u8(64);
    w.write_varint(12);
    Str::write(&mut w, &String::new());
    Str::write(&mut w, &String::new());
    w.write_varint(0);
    let original = w.into_vec();

    let mut wrapper = PacketWrapper::new(&original);
    item_stack_response(&mut wrapper, true).unwrap();
    let widened = wrapper.finish();
    assert_eq!(widened.len(), original.len() + 4, "two optional bool pairs");

    let mut wrapper = PacketWrapper::new(&widened);
    item_stack_response(&mut wrapper, false).unwrap();
    assert_eq!(wrapper.finish(), original);
}
