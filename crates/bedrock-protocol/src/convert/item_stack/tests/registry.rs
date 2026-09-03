use super::*;

#[test]
fn the_registry_is_forwarded_verbatim_and_cached() {
    let mut w = Writer::new();
    w.write_count(2);
    for (name, id) in [("minecraft:stone", 1i16), ("minecraft:dirt", 2)] {
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
    assert_eq!(state.item_ids.get("minecraft:dirt"), Some(&2));
    assert_eq!(
        state.item_names.get(&1).map(String::as_str),
        Some("minecraft:stone")
    );
}

#[test]
fn the_probe_separates_a_present_name_from_an_absent_one() {
    if std::env::var_os("CROSSBIND_ITEM_PROBE").is_some() {
        return;
    }

    let mut w = Writer::new();
    w.write_count(2);
    for (name, id, component, version) in [
        ("minecraft:netherite_axe", 609i16, false, 0i32),
        ("minecraft:stone", 1, false, 0),
    ] {
        Str::write(&mut w, &name.to_owned());
        w.write_i16_le(id);
        w.write_bool(component);
        w.write_varint(version);
        NamedCompoundTag::write(&mut w, &EMPTY_NAMED_COMPOUND.to_vec());
    }
    let body = w.into_vec();

    let mut wrapper = PacketWrapper::new(&body);
    let mut state = ConnState::new(975);
    cache_item_registry(&mut wrapper, &mut state).unwrap();
    assert_eq!(wrapper.finish(), body, "probing must not alter the packet");

    let joined = state.notices.join("\n");
    assert!(
        joined.contains("minecraft:netherite_axe present id=609"),
        "a probed name that is in the registry should be reported with its \
         id and flags: {joined}"
    );
    assert!(
        joined.contains("item_version=LEGACY"),
        "the item_version has to be spelled out, not printed as a number: {joined}"
    );
    assert!(
        joined.contains("absent from the registry") && joined.contains("minecraft:netherite_sword"),
        "names on the probe list that the registry doesn't carry are the \
         whole point of the probe: {joined}"
    );
}
