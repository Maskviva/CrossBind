use super::bits::{read_bitset, write_bitset};
use super::movement::read_double_optional;
use crate::connection::ConnState;
use crate::item_remap;
use crate::pipeline::trace_limit;
use bedrock_codec::prelude::*;

pub(crate) fn player_auth_input(
    w: &mut PacketWrapper,
    state: &mut ConnState,
    to_v2168: bool,
) -> Result<()> {
    if to_v2168 {
        player_auth_input_to_v2168(w)
    } else {
        player_auth_input_to_v1001(w, Some(state))
    }
}

pub(crate) fn player_auth_input_to_v1001(
    w: &mut PacketWrapper,
    mut state: Option<&mut ConnState>,
) -> Result<()> {
    w.passthrough::<FloatLe>()?;
    w.passthrough::<FloatLe>()?;
    w.passthrough::<Vec3>()?;
    w.passthrough::<Vec2>()?;
    w.passthrough::<FloatLe>()?;

    let mut flags: Vec<u32> = Vec::new();
    if w.reader().read_bool()? {
        let count = w.reader().read_count()?;
        for _ in 0..count {
            let id = w.reader().read_varint()?;
            if id < 0 || id as u32 >= INPUT_FLAG_COUNT_V2168 {
                return Err(Error::BadDiscriminant {
                    what: "player auth input flag",
                    value: id as i64,
                });
            }
            if (id as u32) < INPUT_FLAG_BITSET_SIZE_V1001 {
                flags.push(id as u32);
            }
        }
    }

    let input_mode = w.reader().read_uvarint()?;
    let play_mode = w.reader().read_uvarint()?;
    let interaction_model = w.reader().read_varint()?;
    let interact_pitch = w.reader().read_f32_le()?;
    let interact_yaw = w.reader().read_f32_le()?;
    let tick = w.reader().read_uvarint64()?;
    let delta = Vec3::read(w.reader())?;

    let mut payload = Writer::new();

    let has_item_interaction = read_double_optional(w)?;
    if has_item_interaction {
        item_interaction_to_v1001(w.reader(), &mut payload)?;
    }

    if read_double_optional(w)? {
        if trace_limit() != 0 {
            if let Some(state) = state.as_deref_mut() {
                state
                    .notices
                    .push("auth input: tick dropped, carries an ItemStackRequest".to_owned());
            }
        }
        w.cancel();
        return Ok(());
    }

    let has_block_actions = read_double_optional(w)?;
    if has_block_actions {
        block_actions_to_v1001(w.reader(), &mut payload)?;
    }

    let rotation = if read_double_optional(w)? {
        Some(Vec2::read(w.reader())?)
    } else {
        None
    };
    let vehicle = if read_double_optional(w)? {
        Some(w.reader().read_varint64()?)
    } else {
        None
    };
    let has_vehicle = match (rotation, vehicle) {
        (Some(rotation), Some(vehicle)) => {
            Vec2::write(&mut payload, &rotation);
            payload.write_varint64(vehicle);
            true
        }
        _ => false,
    };

    set_flag(
        &mut flags,
        FLAG_PERFORM_ITEM_INTERACTION,
        has_item_interaction,
    );
    set_flag(&mut flags, FLAG_PERFORM_ITEM_STACK_REQUEST, false);
    set_flag(&mut flags, FLAG_PERFORM_BLOCK_ACTIONS, has_block_actions);
    set_flag(&mut flags, FLAG_CLIENT_PREDICTED_VEHICLE, has_vehicle);

    if (has_item_interaction || has_block_actions) && trace_limit() != 0 {
        if let Some(state) = state {
            if trace_limit() != 0 {
                let bytes = payload.len();
                state.notices.push(format!(
                        "auth input: interaction={has_item_interaction} block_actions={has_block_actions} flags={flags:?} payload={bytes} B"
                    ));
            }
        }
    }

    write_bitset(w, &flags, INPUT_FLAG_BITSET_SIZE_V1001);
    w.writer().write_uvarint(input_mode);
    w.writer().write_uvarint(play_mode);
    w.writer().write_uvarint(interaction_model as u32);
    w.writer().write_f32_le(interact_pitch);
    w.writer().write_f32_le(interact_yaw);
    w.writer().write_uvarint64(tick);
    Vec3::write(w.writer(), &delta);
    let payload = payload.into_vec();
    w.writer().write_bytes(&payload);

    w.passthrough_all();
    Ok(())
}

pub(crate) fn player_auth_input_to_v2168(w: &mut PacketWrapper) -> Result<()> {
    w.passthrough::<FloatLe>()?;
    w.passthrough::<FloatLe>()?;
    w.passthrough::<Vec3>()?;
    w.passthrough::<Vec2>()?;
    w.passthrough::<FloatLe>()?;

    let flags = read_bitset(w, INPUT_FLAG_BITSET_SIZE_V1001)?;
    w.writer().write_bool(true);
    w.writer().write_count(flags.len());
    for id in &flags {
        w.writer().write_varint(*id as i32);
    }

    w.passthrough::<UVarInt>()?;
    w.passthrough::<UVarInt>()?;

    let model = w.reader().read_uvarint()?;
    w.writer().write_varint(model as i32);

    w.passthrough::<FloatLe>()?;
    w.passthrough::<FloatLe>()?;
    w.passthrough::<UVarInt64>()?;
    w.passthrough::<Vec3>()?;

    let has = |flag: u32| flags.contains(&flag);

    if has(FLAG_PERFORM_ITEM_INTERACTION)
        || has(FLAG_PERFORM_ITEM_STACK_REQUEST)
        || has(FLAG_PERFORM_BLOCK_ACTIONS)
    {
        w.cancel();
        return Ok(());
    }
    for _ in 0..3 {
        w.writer().write_bool(true);
        w.writer().write_bool(false);
    }

    if has(FLAG_CLIENT_PREDICTED_VEHICLE) {
        let rotation = w.read::<Vec2>()?;
        let vehicle = w.reader().read_varint64()?;
        w.writer().write_bool(true);
        w.writer().write_bool(true);
        w.write::<Vec2>(rotation);
        w.writer().write_bool(true);
        w.writer().write_bool(true);
        w.writer().write_varint64(vehicle);
    } else {
        for _ in 0..2 {
            w.writer().write_bool(true);
            w.writer().write_bool(false);
        }
    }

    w.passthrough_all();
    Ok(())
}

fn block_actions_to_v1001(r: &mut Reader<'_>, out: &mut Writer) -> Result<()> {
    let count = r.read_count()?;
    out.write_varint(count as i32);
    for _ in 0..count {
        let action = r.read_varint()?;
        let position = BlockPos::read(r)?;
        let face = r.read_varint()?;
        out.write_varint(action);
        if block_action_has_position(action) {
            BlockPos::write(out, &position);
            out.write_varint(face);
        }
    }
    Ok(())
}

fn inventory_action_to_v1001(r: &mut Reader<'_>, out: &mut Writer) -> Result<()> {
    let source = r.read_uvarint()?;
    out.write_uvarint(source);

    let window_id = if read_double_optional_from(r)? {
        Some(r.read_i8()?)
    } else {
        None
    };
    let source_flags = if read_double_optional_from(r)? {
        Some(r.read_uvarint()?)
    } else {
        None
    };
    match source {
        INVENTORY_SOURCE_CONTAINER | INVENTORY_SOURCE_TODO => {
            out.write_varint(window_id.unwrap_or(0) as i32)
        }
        INVENTORY_SOURCE_WORLD => out.write_uvarint(source_flags.unwrap_or(0)),
        _ => {}
    }

    out.write_uvarint(r.read_uvarint()?);
    item_stack_to_v1001(r, out)?;
    item_stack_to_v1001(r, out)?;
    Ok(())
}

fn item_interaction_to_v1001(r: &mut Reader<'_>, out: &mut Writer) -> Result<()> {
    let legacy_request_id = r.read_varint()?;
    out.write_varint(legacy_request_id);

    let mut slots: Vec<(u8, Vec<u8>)> = Vec::new();
    if r.read_bool()? {
        let count = r.read_count()?;
        for _ in 0..count {
            let container = r.read_u8()?;
            slots.push((container, ByteArray::read(r)?));
        }
    }
    if legacy_request_id < -1 && (legacy_request_id & 1) == 0 {
        out.write_count(slots.len());
        for (container, payload) in &slots {
            out.write_u8(*container);
            out.write_count(payload.len());
            out.write_bytes(payload);
        }
    }

    let action_count = if read_double_optional_from(r)? {
        r.read_count()?
    } else {
        0
    };
    out.write_count(action_count);
    for _ in 0..action_count {
        inventory_action_to_v1001(r, out)?;
    }

    out.write_uvarint(r.read_varint()? as u32);
    out.write_uvarint(r.read_u8()? as u32);
    BlockPos::write(out, &BlockPos::read(r)?);
    out.write_varint(r.read_u8()? as i32);
    out.write_varint(r.read_varint()?);
    item_stack_to_v1001(r, out)?;
    Vec3::write(out, &Vec3::read(r)?);
    Vec3::write(out, &Vec3::read(r)?);
    out.write_uvarint(r.read_uvarint()?);
    out.write_u8(r.read_u8()?);
    out.write_u8(r.read_u8()?);
    Ok(())
}

fn item_stack_to_v1001(r: &mut Reader<'_>, out: &mut Writer) -> Result<()> {
    let mut item = ItemInstanceV2168::read(r)?;
    item.network_id = item_remap::to_server(item.network_id);
    ItemInstance::write(out, &item);
    Ok(())
}

fn block_action_has_position(action: i32) -> bool {
    matches!(
        action,
        PLAYER_ACTION_START_BREAK
            | PLAYER_ACTION_ABORT_BREAK
            | PLAYER_ACTION_CRACK_BREAK
            | PLAYER_ACTION_PREDICT_DESTROY_BLOCK
            | PLAYER_ACTION_CONTINUE_DESTROY_BLOCK
    )
}

fn read_double_optional_from(r: &mut Reader<'_>) -> Result<bool> {
    if !r.read_bool()? {
        return Ok(false);
    }
    r.read_bool()
}

fn set_flag(flags: &mut Vec<u32>, flag: u32, present: bool) {
    match (flags.iter().position(|f| *f == flag), present) {
        (None, true) => flags.push(flag),
        (Some(at), false) => {
            flags.remove(at);
        }
        _ => {}
    }
}

pub(crate) const INPUT_FLAG_BITSET_SIZE_V1001: u32 = 65;
pub(crate) const INPUT_FLAG_COUNT_V2168: u32 = 66;
pub(crate) const FLAG_PERFORM_ITEM_INTERACTION: u32 = 34;
pub(crate) const FLAG_PERFORM_BLOCK_ACTIONS: u32 = 35;
pub(crate) const FLAG_PERFORM_ITEM_STACK_REQUEST: u32 = 36;
pub(crate) const FLAG_CLIENT_PREDICTED_VEHICLE: u32 = 45;

pub(crate) const PLAYER_ACTION_START_BREAK: i32 = 0;
pub(crate) const PLAYER_ACTION_ABORT_BREAK: i32 = 1;
pub(crate) const PLAYER_ACTION_CRACK_BREAK: i32 = 18;
pub(crate) const PLAYER_ACTION_PREDICT_DESTROY_BLOCK: i32 = 26;
pub(crate) const PLAYER_ACTION_CONTINUE_DESTROY_BLOCK: i32 = 27;
pub(crate) const INVENTORY_SOURCE_CONTAINER: u32 = 0;
pub(crate) const INVENTORY_SOURCE_WORLD: u32 = 2;
pub(crate) const INVENTORY_SOURCE_TODO: u32 = 99999;
