use std::collections::HashMap;

use crate::connection::ConnState;

use bedrock_codec::prelude::*;

const ACTION_PLACE_IN_CONTAINER: u32 = 7;
const ACTION_TAKE_OUT_CONTAINER: u32 = 8;
const ACTION_CRAFT_RECIPE_AUTO: u32 = 13;
const ACTION_MAX: u32 = 19;

const RESPONSE_STATUS_OK: u8 = 0;

const DESCRIPTOR_INVALID: u32 = 0;
const DESCRIPTOR_DEFAULT: u32 = 1;

fn action_variant(id: u32) -> u32 {
    if id > ACTION_TAKE_OUT_CONTAINER {
        id - 2
    } else {
        id
    }
}

fn action_id(variant: u32) -> u32 {
    if variant >= ACTION_PLACE_IN_CONTAINER {
        variant + 2
    } else {
        variant
    }
}

fn bad_action(value: u32) -> Error {
    Error::BadDiscriminant {
        what: "stack request action",
        value: value as i64,
    }
}

fn full_container_name(r: &mut Reader<'_>, w: &mut Writer) -> Result<()> {
    w.write_u8(r.read_u8()?);
    let dynamic = Optional::<UIntLe>::read(r)?;
    Optional::<UIntLe>::write(w, &dynamic);
    Ok(())
}

fn slot_info(r: &mut Reader<'_>, w: &mut Writer, to_v2168: bool) -> Result<()> {
    full_container_name(r, w)?;
    w.write_u8(r.read_u8()?);
    if to_v2168 {
        let id = r.read_varint()?;
        w.write_i32_le(id);
    } else {
        let id = r.read_i32_le()?;
        w.write_varint(id);
    }
    Ok(())
}

fn result_item(
    r: &mut Reader<'_>,
    w: &mut Writer,
    to_v2168: bool,
    names: &HashMap<String, i32>,
    ids: &HashMap<i32, String>,
) -> Result<bool> {
    if to_v2168 {
        let network_id = r.read_varint()?;
        if network_id == 0 {
            w.write_uvarint(DESCRIPTOR_INVALID);
            w.write_u8(DESCRIPTOR_INVALID as u8);
            w.write_i16_le(0);
            w.write_uvarint(0);
            w.write_count(0);
            return Ok(true);
        }
        let Some(name) = ids.get(&network_id) else {
            return Ok(false);
        };
        let count = r.read_u16_le()?;
        let metadata = r.read_uvarint()?;
        let block_runtime_id = r.read_varint()?;
        let extra = r.read_count()?;
        let user_data = r.read_bytes(extra)?.to_vec();

        w.write_uvarint(DESCRIPTOR_DEFAULT);
        w.write_u8(DESCRIPTOR_DEFAULT as u8);
        Str::write(w, name);
        w.write_varint(metadata as i32);
        w.write_i16_le(count as i16);
        w.write_uvarint(block_runtime_id as u32);
        w.write_count(user_data.len());
        w.write_bytes(&user_data);
        return Ok(true);
    }

    let variant = r.read_uvarint()?;
    r.read_u8()?;
    if variant == DESCRIPTOR_DEFAULT {
        let name = Str::read(r)?;
        let metadata = r.read_varint()?;
        let count = r.read_i16_le()?;
        let block_runtime_id = r.read_uvarint()?;
        let extra = r.read_count()?;
        let user_data = r.read_bytes(extra)?.to_vec();

        let Some(network_id) = names.get(&name).copied() else {
            return Ok(false);
        };
        if network_id == 0 {
            w.write_varint(0);
            return Ok(true);
        }
        w.write_varint(network_id);
        w.write_u16_le(count as u16);
        w.write_uvarint(metadata as u32);
        w.write_varint(block_runtime_id as i32);
        w.write_count(user_data.len());
        w.write_bytes(&user_data);
        return Ok(true);
    }
    if variant != DESCRIPTOR_INVALID {
        return Ok(false);
    }
    r.read_i16_le()?;
    r.read_uvarint()?;
    let extra = r.read_count()?;
    r.read_bytes(extra)?;
    w.write_varint(0);
    Ok(true)
}

#[derive(Debug, PartialEq)]
pub(crate) enum ActionOutcome {
    Written,
    Blocked,
}

fn action(
    r: &mut Reader<'_>,
    w: &mut Writer,
    to_v2168: bool,
    names: &HashMap<String, i32>,
    ids: &HashMap<i32, String>,
) -> Result<ActionOutcome> {
    let id = if to_v2168 {
        r.read_u8()? as u32
    } else {
        let variant = r.read_uvarint()?;
        r.read_u8()?;
        if variant > action_variant(ACTION_MAX) {
            return Err(bad_action(variant));
        }
        action_id(variant)
    };
    if id > ACTION_MAX {
        return Err(bad_action(id));
    }

    if id == ACTION_CRAFT_RECIPE_AUTO {
        return Ok(ActionOutcome::Blocked);
    }
    if id == ACTION_PLACE_IN_CONTAINER || id == ACTION_TAKE_OUT_CONTAINER {
        return Err(bad_action(id));
    }

    if to_v2168 {
        w.write_uvarint(action_variant(id));
        w.write_u8(id as u8);
    } else {
        w.write_u8(id as u8);
    }

    match id {
        0 | 1 => {
            w.write_u8(r.read_u8()?);
            slot_info(r, w, to_v2168)?;
            slot_info(r, w, to_v2168)?;
        }
        2 => {
            slot_info(r, w, to_v2168)?;
            slot_info(r, w, to_v2168)?;
        }
        3 => {
            w.write_u8(r.read_u8()?);
            slot_info(r, w, to_v2168)?;
            w.write_bool(r.read_bool()?);
        }
        4 | 5 => {
            w.write_u8(r.read_u8()?);
            slot_info(r, w, to_v2168)?;
        }
        6 => w.write_u8(r.read_u8()?),
        9 | 18 => {}
        10 => {
            w.write_varint(r.read_varint()?);
            w.write_varint(r.read_varint()?);
        }
        11 => {
            w.write_varint(r.read_varint()?);
            w.write_varint(r.read_varint()?);
            if to_v2168 {
                let id = r.read_varint()?;
                w.write_i32_le(id);
            } else {
                let id = r.read_i32_le()?;
                w.write_varint(id);
            }
        }
        12 | 14 => {
            w.write_uvarint(r.read_uvarint()?);
            w.write_u8(r.read_u8()?);
        }
        15 => {
            w.write_uvarint(r.read_uvarint()?);
            w.write_i32_le(r.read_i32_le()?);
        }
        16 => {
            if to_v2168 {
                let recipe = r.read_uvarint()?;
                w.write_i32_le(recipe as i32);
            } else {
                let recipe = r.read_i32_le()?;
                w.write_uvarint(recipe as u32);
            }
            w.write_u8(r.read_u8()?);
            w.write_varint(r.read_varint()?);
        }
        17 => {
            let pattern = Str::read(r)?;
            Str::write(w, &pattern);
            w.write_u8(r.read_u8()?);
        }
        19 => {
            let items = r.read_count()?;
            w.write_count(items);
            for _ in 0..items {
                if !result_item(r, w, to_v2168, names, ids)? {
                    return Ok(ActionOutcome::Blocked);
                }
            }
            w.write_u8(r.read_u8()?);
        }
        other => return Err(bad_action(other)),
    }

    Ok(ActionOutcome::Written)
}

pub(crate) fn item_stack_request(
    w: &mut PacketWrapper,
    to_v2168: bool,
    names: &HashMap<String, i32>,
    ids: &HashMap<i32, String>,
) -> Result<bool> {
    let mut out = Writer::new();
    let requests = w.reader().read_count()?;
    out.write_count(requests);

    for _ in 0..requests {
        out.write_varint(w.reader().read_varint()?);

        let count = w.reader().read_count()?;
        out.write_count(count);
        for _ in 0..count {
            if action(w.reader(), &mut out, to_v2168, names, ids)? == ActionOutcome::Blocked {
                return Ok(false);
            }
        }

        let filters = w.reader().read_count()?;
        out.write_count(filters);
        for _ in 0..filters {
            let s = Str::read(w.reader())?;
            Str::write(&mut out, &s);
        }

        out.write_i32_le(w.reader().read_i32_le()?);
    }

    w.writer().write_bytes(&out.into_vec());
    w.passthrough_all();
    Ok(true)
}

fn response_containers(r: &mut Reader<'_>, w: &mut Writer, to_v2168: bool) -> Result<()> {
    let containers = r.read_count()?;
    w.write_count(containers);
    for _ in 0..containers {
        full_container_name(r, w)?;
        let slots = r.read_count()?;
        w.write_count(slots);
        for _ in 0..slots {
            w.write_u8(r.read_u8()?);
            w.write_u8(r.read_u8()?);
            w.write_u8(r.read_u8()?);

            if to_v2168 {
                let net = r.read_varint()?;
                w.write_bool(true);
                w.write_bool(true);
                w.write_varint(net);
            } else if read_double_optional_from(r)? {
                w.write_varint(r.read_varint()?);
            } else {
                w.write_varint(0);
            }

            let name = Str::read(r)?;
            Str::write(w, &name);
            let filtered = Str::read(r)?;
            Str::write(w, &filtered);
            w.write_varint(r.read_varint()?);
        }
    }
    Ok(())
}

fn read_double_optional_from(r: &mut Reader<'_>) -> Result<bool> {
    if !r.read_bool()? {
        return Ok(false);
    }
    r.read_bool()
}

pub(crate) fn item_stack_response(w: &mut PacketWrapper, to_v2168: bool) -> Result<()> {
    let mut out = Writer::new();
    let entries = w.reader().read_count()?;
    out.write_count(entries);

    for _ in 0..entries {
        let status = w.reader().read_u8()?;
        out.write_u8(status);
        out.write_varint(w.reader().read_varint()?);

        if to_v2168 {
            let present = status == RESPONSE_STATUS_OK;
            out.write_bool(true);
            out.write_bool(present);
            if present {
                response_containers(w.reader(), &mut out, true)?;
            }
        } else {
            let present = read_double_optional_from(w.reader())?;
            let writes = status == RESPONSE_STATUS_OK;
            if present {
                let mut buffered = Writer::new();
                response_containers(w.reader(), &mut buffered, false)?;
                if writes {
                    out.write_bytes(&buffered.into_vec());
                }
            } else if writes {
                out.write_count(0);
            }
        }
    }

    w.writer().write_bytes(&out.into_vec());
    w.passthrough_all();
    Ok(())
}

pub(crate) fn cache_item_registry(w: &mut PacketWrapper, state: &mut ConnState) -> Result<()> {
    let body = w.reader().read_remaining().to_vec();
    w.writer().write_bytes(&body);

    let mut r = Reader::new(&body);
    let count = r.read_count()?;
    let mut names = HashMap::with_capacity(count);
    let mut ids = HashMap::with_capacity(count);
    for _ in 0..count {
        let name = Str::read(&mut r)?;
        let network_id = r.read_i16_le()? as i32;
        r.read_bool()?;
        r.read_varint()?;
        NamedCompoundTag::read(&mut r)?;
        ids.insert(network_id, name.clone());
        names.insert(name, network_id);
    }
    state
        .notices
        .push(format!("item registry cached: {count} entries"));
    state.item_ids = names;
    state.item_names = ids;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v1001_slot(w: &mut Writer, slot: u8, net: i32) {
        w.write_u8(0);
        Optional::<UIntLe>::write(w, &None);
        w.write_u8(slot);
        w.write_varint(net);
    }

    fn v1001_creative_pull() -> Vec<u8> {
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

    fn empty_tables() -> (HashMap<String, i32>, HashMap<i32, String>) {
        (HashMap::new(), HashMap::new())
    }

    fn run(input: &[u8], to_v2168: bool) -> Vec<u8> {
        let (names, ids) = empty_tables();
        let mut w = PacketWrapper::new(input);
        assert!(item_stack_request(&mut w, to_v2168, &names, &ids).expect("handler failed"));
        w.finish()
    }

    #[test]
    fn variant_mapping_is_invertible_and_skips_the_container_pair() {
        for id in 0..=ACTION_MAX {
            if id == ACTION_PLACE_IN_CONTAINER || id == ACTION_TAKE_OUT_CONTAINER {
                continue;
            }
            assert_eq!(action_id(action_variant(id)), id, "id {id} did not survive");
        }
        assert_eq!(action_variant(6), 6);
        assert_eq!(action_variant(9), 7);
        assert_eq!(action_id(7), 9);
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
    fn shift_click_crafting_is_refused_rather_than_guessed() {
        let mut w = Writer::new();
        w.write_count(1);
        w.write_varint(1);
        w.write_count(1);
        w.write_u8(13);
        let body = w.into_vec();

        let mut wrapper = PacketWrapper::new(&body);
        let (names, ids) = empty_tables();
        assert!(!item_stack_request(&mut wrapper, true, &names, &ids).unwrap());
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

    fn stone_tables() -> (HashMap<String, i32>, HashMap<i32, String>) {
        let mut names = HashMap::new();
        let mut ids = HashMap::new();
        names.insert("minecraft:stone".to_owned(), 1);
        ids.insert(1, "minecraft:stone".to_owned());
        (names, ids)
    }

    fn v2168_pull_with_result(name: &str) -> Vec<u8> {
        let mut w = Writer::new();
        w.write_count(1);
        w.write_varint(-3);
        w.write_count(1);
        w.write_uvarint(action_variant(19));
        w.write_u8(19);
        w.write_count(1);
        w.write_uvarint(DESCRIPTOR_DEFAULT);
        w.write_u8(DESCRIPTOR_DEFAULT as u8);
        Str::write(&mut w, &name.to_owned());
        w.write_varint(0);
        w.write_i16_le(64);
        w.write_uvarint(0);
        w.write_count(0);
        w.write_u8(1);
        w.write_count(0);
        w.write_i32_le(0);
        w.into_vec()
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
        assert_eq!(state.item_names.get(&1).map(String::as_str), Some("minecraft:stone"));
    }

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

    #[test]
    fn a_rejected_response_carries_no_container_list_down() {
        let mut w = Writer::new();
        w.write_count(1);
        w.write_u8(1);
        w.write_varint(7);
        w.write_bool(true);
        w.write_bool(true);
        w.write_count(0);
        let body = w.into_vec();

        let mut wrapper = PacketWrapper::new(&body);
        item_stack_response(&mut wrapper, false).unwrap();
        let out = wrapper.finish();

        let mut r = Reader::new(&out);
        assert_eq!(r.read_count().unwrap(), 1);
        assert_eq!(r.read_u8().unwrap(), 1);
        assert_eq!(r.read_varint().unwrap(), 7);
        assert_eq!(r.remaining(), 0, "nothing follows a non-OK status");
    }
}
