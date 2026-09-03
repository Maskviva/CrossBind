use super::*;
use crate::ClientItems;
use std::collections::HashSet;

fn client(rows: &[(&str, i32)]) -> ClientItems {
    let mut text = String::from("name\tid\tcomponent_based\titem_version\tcomponent_nbt_bytes\n");
    for (name, id) in rows {
        text.push_str(&format!("{name}\t{id}\tfalse\tNONE\t3\n"));
    }
    ClientItems::from_registry_tsv(&text).unwrap()
}

fn server(rows: &[(&str, i32)]) -> Vec<(String, i32)> {
    rows.iter().map(|(n, i)| ((*n).to_owned(), *i)).collect()
}

#[test]
fn shared_names_take_the_clients_id() {
    let c = client(&[("minecraft:netherite_sword", 617), ("minecraft:stone", 1)]);
    let remap = ItemRemap::build(
        &c,
        &server(&[("minecraft:netherite_sword", 774), ("minecraft:stone", 1)]),
    );
    assert_eq!(remap.to_client(774), 617);
    assert_eq!(remap.to_server(617), 774);
    assert_eq!(remap.to_client(1), 1);
    assert_eq!(remap.report().renumbered, 1);
    assert_eq!(remap.report().agreed, 1);
}

#[test]
fn a_server_only_item_keeps_its_id_when_nothing_wants_it() {
    let c = client(&[("minecraft:stone", 1)]);
    let remap = ItemRemap::build(
        &c,
        &server(&[("minecraft:stone", 1), ("bedwars:blue_bed", 300)]),
    );
    assert_eq!(remap.to_client(300), 300);
    assert_eq!(remap.report().kept, 1);
    assert_eq!(remap.report().relocated, 0);
}

#[test]
fn a_server_only_item_is_moved_off_a_client_id() {
    let c = client(&[("minecraft:golden_apple", 258), ("minecraft:stone", 1)]);
    let remap = ItemRemap::build(
        &c,
        &server(&[
            ("minecraft:golden_apple", 500),
            ("minecraft:stone", 1),
            ("bedwars:brown_boots", 258),
        ]),
    );
    assert_eq!(
        remap.to_client(500),
        258,
        "the vanilla name gets the client id"
    );
    let moved = remap.to_client(258);
    assert_ne!(moved, 258, "the addon item must not stay on 258");
    assert_eq!(remap.to_server(moved), 258, "and must stay reversible");
    assert_eq!(remap.report().relocated, 1);
}

#[test]
fn no_two_server_ids_reach_the_same_client_id() {
    let c = client(&[
        ("minecraft:a", 258),
        ("minecraft:b", 259),
        ("minecraft:c", 3),
    ]);
    let remap = ItemRemap::build(
        &c,
        &server(&[
            ("minecraft:a", 500),
            ("minecraft:b", 501),
            ("minecraft:c", 3),
            ("addon:x", 258),
            ("addon:y", 259),
            ("addon:z", 260),
        ]),
    );
    let targets: Vec<i32> = [500, 501, 3, 258, 259, 260]
        .iter()
        .map(|id| remap.to_client(*id))
        .collect();
    let mut seen = HashSet::new();
    for id in &targets {
        assert!(seen.insert(*id), "duplicate client id {id} in {targets:?}");
    }
}

#[test]
fn a_relocated_item_keeps_its_sign() {
    let c = client(&[("minecraft:stone", -270), ("minecraft:dirt", 1)]);
    let remap = ItemRemap::build(
        &c,
        &server(&[
            ("minecraft:stone", -270),
            ("minecraft:dirt", 1),
            ("addon:block", -270),
        ]),
    );
    let moved = remap.to_client(-270);
    assert!(moved < 0, "relocated block item went positive: {moved}");
    assert_eq!(remap.report().relocated, 1);
}

#[test]
fn round_trip_is_lossless_for_every_mapped_id() {
    let c = client(&[
        ("minecraft:a", 617),
        ("minecraft:b", 620),
        ("minecraft:c", 5),
    ]);
    let remap = ItemRemap::build(
        &c,
        &server(&[
            ("minecraft:a", 774),
            ("minecraft:b", 777),
            ("minecraft:c", 5),
            ("addon:x", 617),
        ]),
    );
    for server_id in [774, 777, 5, 617] {
        assert_eq!(remap.to_server(remap.to_client(server_id)), server_id);
    }
}

#[test]
fn air_is_never_remapped() {
    let c = client(&[("minecraft:stone", 1)]);
    let remap = ItemRemap::build(&c, &server(&[("minecraft:stone", 1)]));
    assert_eq!(remap.to_client(AIR), AIR);
    assert_eq!(remap.to_server(AIR), AIR);
}

#[test]
fn an_unmapped_id_passes_through() {
    let c = client(&[("minecraft:stone", 1)]);
    let remap = ItemRemap::build(&c, &server(&[("minecraft:stone", 1)]));
    assert_eq!(remap.to_client(9999), 9999);
}

#[test]
fn a_two_column_table_also_parses() {
    let items = ClientItems::from_registry_tsv("name\tid\nminecraft:stone\t1\n").unwrap();
    assert_eq!(items.len(), 1);
}

#[test]
fn a_duplicate_client_id_is_refused() {
    assert!(matches!(
        ClientItems::from_registry_tsv("name\tid\nminecraft:a\t1\nminecraft:b\t1\n").unwrap_err(),
        RemapLoadError::DuplicateId { id: 1, .. }
    ));
}

#[test]
fn a_header_without_an_id_column_is_refused() {
    assert!(matches!(
        ClientItems::from_registry_tsv("name\tvalue\nminecraft:a\t1\n").unwrap_err(),
        RemapLoadError::BadHeader(_)
    ));
}

#[test]
fn a_header_with_no_rows_is_refused() {
    assert!(matches!(
        ClientItems::from_registry_tsv("name\tid\n").unwrap_err(),
        RemapLoadError::Empty
    ));
}

#[test]
fn the_embedded_client_table_parses_and_looks_right() {
    let items = client_items().expect("embedded client table must parse");
    assert!(items.len() > 1500, "only {} entries", items.len());
    for (name, id) in [
        ("minecraft:netherite_sword", 617),
        ("minecraft:netherite_axe", 620),
        ("minecraft:warped_door", 631),
    ] {
        assert_eq!(
            items.by_name.get(name).copied(),
            Some(id),
            "{name} moved; the table no longer matches the target client"
        );
    }
}
