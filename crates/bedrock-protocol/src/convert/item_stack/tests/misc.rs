use super::*;

pub fn v1001_slot(w: &mut Writer, slot: u8, net: i32) {
    w.write_u8(0);
    Optional::<UIntLe>::write(w, &None);
    w.write_u8(slot);
    w.write_varint(net);
}

pub fn v1001_creative_pull() -> Vec<u8> {
    let mut w = Writer::new();
    w.write_count(1);
    w.write_varint(-3);
    w.write_count(3);

    w.write_u8(14);
    w.write_uvarint(755);
    w.write_u8(1);

    w.write_u8(4);
    w.write_u8(64);
    v1001_slot(&mut w, 50, 12);

    w.write_u8(19);
    w.write_count(0);
    w.write_u8(1);

    w.write_count(0);
    w.write_i32_le(0);
    w.into_vec()
}

pub fn empty_tables() -> (HashMap<String, i32>, HashMap<i32, String>) {
    (HashMap::new(), HashMap::new())
}

pub fn run(input: &[u8], to_v2168: bool) -> Vec<u8> {
    let (names, ids) = empty_tables();
    let mut w = PacketWrapper::new(input);
    assert!(item_stack_request(&mut w, to_v2168, &names, &ids).expect("handler failed"));
    w.finish()
}

#[test]
fn a_creative_pull_round_trips() {
    let original = v1001_creative_pull();
    let widened = run(&original, true);
    assert_ne!(widened, original, "the header and slot id must change");
    let back = run(&widened, false);
    assert_eq!(back, original);
}

#[test]
fn the_slot_network_id_becomes_fixed_width() {
    let original = v1001_creative_pull();
    let widened = run(&original, true);
    assert_eq!(widened.len(), original.len() + 3 + 3);
}

#[test]
fn craft_recipe_auto_round_trips_through_the_item_registry() {
    let (names, ids) = stone_tables();
    let original = v1001_craft_recipe_auto_default(200, 4, 1, 3);

    let mut up = PacketWrapper::new(&original);
    assert!(
        item_stack_request(&mut up, true, &names, &ids).unwrap(),
        "the up-translation must not cancel a known item"
    );
    let widened = up.finish();
    assert_ne!(
        widened, original,
        "the ingredient encoding must actually change"
    );

    let mut down = PacketWrapper::new(&widened);
    assert!(
        item_stack_request(&mut down, false, &names, &ids).unwrap(),
        "the down-translation must not cancel either"
    );
    assert_eq!(
        down.finish(),
        original,
        "the packet must round-trip byte-for-byte"
    );
}

#[test]
fn craft_recipe_auto_survives_a_molang_ingredient() {
    let (names, ids) = empty_tables();
    let original = v1001_craft_recipe_auto_molang("q.foo", 7);

    let mut up = PacketWrapper::new(&original);
    assert!(item_stack_request(&mut up, true, &names, &ids).unwrap());
    let widened = up.finish();

    let mut down = PacketWrapper::new(&widened);
    assert!(item_stack_request(&mut down, false, &names, &ids).unwrap());
    assert_eq!(
        down.finish(),
        original,
        "molang expression + version must round-trip"
    );
}

#[test]
fn an_out_of_range_variant_is_an_error_not_a_silent_shift() {
    let mut w = Writer::new();
    w.write_count(1);
    w.write_varint(1);
    w.write_count(1);
    w.write_uvarint(99);
    w.write_u8(99);
    let body = w.into_vec();

    let mut wrapper = PacketWrapper::new(&body);
    let (names, ids) = empty_tables();
    assert!(item_stack_request(&mut wrapper, false, &names, &ids).is_err());
}

pub fn stone_tables() -> (HashMap<String, i32>, HashMap<i32, String>) {
    let mut names = HashMap::new();
    let mut ids = HashMap::new();
    names.insert("minecraft:stone".to_owned(), 1);
    ids.insert(1, "minecraft:stone".to_owned());
    (names, ids)
}

#[test]
fn a_result_list_never_comes_out_empty() {
    let (names, ids) = stone_tables();
    let input = v2168_pull_with_result("minecraft:stone");
    let mut w = PacketWrapper::new(&input);
    assert!(item_stack_request(&mut w, false, &names, &ids).unwrap());
    let out = w.finish();

    let mut r = Reader::new(&out);
    assert_eq!(r.read_count().unwrap(), 1);
    assert_eq!(r.read_varint().unwrap(), -3);
    assert_eq!(r.read_count().unwrap(), 1);
    assert_eq!(r.read_u8().unwrap(), 19);
    assert_eq!(r.read_count().unwrap(), 1, "the result list must survive");
    assert_eq!(r.read_varint().unwrap(), 1, "resolved to the registry id");
    assert_eq!(r.read_u16_le().unwrap(), 64);
}

#[test]
fn an_unknown_item_refuses_instead_of_emitting_a_hole() {
    let (names, ids) = stone_tables();
    let input = v2168_pull_with_result("modded:widget");
    let mut w = PacketWrapper::new(&input);
    assert!(
        !item_stack_request(&mut w, false, &names, &ids).unwrap(),
        "a name the server never registered must block the request"
    );
}

#[test]
fn an_empty_registry_blocks_rather_than_crashing() {
    let (names, ids) = empty_tables();
    let input = v2168_pull_with_result("minecraft:stone");
    let mut w = PacketWrapper::new(&input);
    assert!(!item_stack_request(&mut w, false, &names, &ids).unwrap());
}

#[test]
fn a_result_item_round_trips_through_the_registry() {
    let (names, ids) = stone_tables();
    let input = v2168_pull_with_result("minecraft:stone");
    let mut w = PacketWrapper::new(&input);
    assert!(item_stack_request(&mut w, false, &names, &ids).unwrap());
    let down = w.finish();

    let mut w = PacketWrapper::new(&down);
    assert!(item_stack_request(&mut w, true, &names, &ids).unwrap());
    assert_eq!(w.finish(), input);
}
