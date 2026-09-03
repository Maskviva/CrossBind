use super::descriptor::{recipe_ingredient, DESCRIPTOR_DEFAULT, DESCRIPTOR_INVALID};
use bedrock_codec::prelude::*;
use std::collections::HashMap;

pub(super) fn action(
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
        13 => {
            w.write_varint(r.read_varint()?);
            w.write_u8(r.read_u8()?);
            if to_v2168 {
                let _num_ingredients = r.read_u8()?;
            }
            let count = r.read_count()?;
            if !to_v2168 {
                let num_u8 = u8::try_from(count).map_err(|_| {
                    Error::Invalid("CraftRecipeAuto ingredient count exceeds uint8")
                })?;
                w.write_u8(num_u8);
            }
            w.write_count(count);
            for _ in 0..count {
                if !recipe_ingredient(r, w, to_v2168, names, ids)? {
                    return Ok(ActionOutcome::Blocked);
                }
            }
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

#[derive(Debug, PartialEq)]
pub(super) enum ActionOutcome {
    Written,
    Blocked,
}

pub(super) fn action_variant(id: u32) -> u32 {
    if id > ACTION_TAKE_OUT_CONTAINER {
        id - 2
    } else {
        id
    }
}

pub(super) fn action_id(variant: u32) -> u32 {
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

pub(super) fn full_container_name(r: &mut Reader<'_>, w: &mut Writer) -> Result<()> {
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

pub(super) fn write_stack_u16(w: &mut Writer, stack: i32) -> Result<()> {
    let clamped = if stack < 0 {
        0u16
    } else {
        u16::try_from(stack)
            .map_err(|_| Error::Invalid("recipe ingredient stack_size exceeds uint16"))?
    };
    w.write_u16_le(clamped);
    Ok(())
}

pub(super) const ACTION_PLACE_IN_CONTAINER: u32 = 7;

pub(super) const ACTION_TAKE_OUT_CONTAINER: u32 = 8;

pub(super) const ACTION_MAX: u32 = 19;

pub(super) fn read_double_optional_from(r: &mut Reader<'_>) -> Result<bool> {
    if !r.read_bool()? {
        return Ok(false);
    }
    r.read_bool()
}
