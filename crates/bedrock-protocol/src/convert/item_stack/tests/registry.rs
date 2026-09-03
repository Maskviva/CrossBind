use super::*;

#[test]
fn the_registry_is_forwarded_verbatim_and_cached() {
    let mut w = Writer::new();
    w.write_count(2);
    for (name, id) in [("minecraft:stone", 1i16), ("minecraft:dirt", 3)] {
        Str::write(&mut w, &name.to_owned());
        w.write_i16_le(id);
        w.write_bool(false);
        w.write_varint(0);
        NamedCompoundTag::write(&mut w, &EMPTY_NAMED_COMPOUND.to_vec());
    }
    let body = w.into_vec();

    let mut wrapper = PacketWrapper::new(&body);
    let mut state = ConnState::new(975);
    cache_item_registry(&mut wrapper, &mut state).unwrap();
    assert_eq!(wrapper.finish(), body, "the packet must not be altered");
    assert_eq!(state.item_ids.get("minecraft:dirt"), Some(&3));
    assert_eq!(
        state.item_names.get(&1).map(String::as_str),
        Some("minecraft:stone")
    );
}

#[test]
fn a_server_id_that_disagrees_with_the_client_is_rewritten() {
    let mut w = Writer::new();
    w.write_count(1);
    Str::write(&mut w, &"minecraft:netherite_axe".to_owned());
    w.write_i16_le(609);
    w.write_bool(false);
    w.write_varint(0);
    NamedCompoundTag::write(&mut w, &EMPTY_NAMED_COMPOUND.to_vec());
    let body = w.into_vec();

    let mut wrapper = PacketWrapper::new(&body);
    let mut state = ConnState::new(975);
    cache_item_registry(&mut wrapper, &mut state).unwrap();

    let out = wrapper.finish();
    assert_ne!(out, body);

    let mut r = Reader::new(&out);
    assert_eq!(r.read_count().unwrap(), 1);
    assert_eq!(Str::read(&mut r).unwrap(), "minecraft:netherite_axe");
    assert_eq!(r.read_i16_le().unwrap(), 620);
}
