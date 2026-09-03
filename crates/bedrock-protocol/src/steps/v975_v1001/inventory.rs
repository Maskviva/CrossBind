use super::action::{
    new_to_old_action, old_to_new_action, read_new_action, read_old_action, write_new_action,
    write_old_action,
};
use super::{ITEM_RELEASE_TRANSACTION, ITEM_USE_ON_ENTITY_TRANSACTION, ITEM_USE_TRANSACTION};
use bedrock_codec::prelude::*;

pub(crate) fn inventory_content(w: &mut PacketWrapper, to_v1001: bool) -> Result<()> {
    w.passthrough::<UVarInt>()?;

    let content_count = w.passthrough::<UVarInt>()?;
    for _ in 0..content_count {
        convert_item(w, to_v1001)?;
    }

    w.passthrough::<Byte>()?;
    let has_dynamic = w.passthrough::<Bool>()?;
    if has_dynamic {
        w.passthrough::<UIntLe>()?;
    }

    convert_item(w, to_v1001)?;
    Ok(())
}

pub(crate) fn mob_armour(w: &mut PacketWrapper, to_v1001: bool) -> Result<()> {
    w.passthrough::<UVarInt64>()?;
    for _ in 0..5 {
        convert_item(w, to_v1001)?;
    }
    Ok(())
}

fn convert_item(w: &mut PacketWrapper, to_v1001: bool) -> Result<()> {
    if to_v1001 {
        w.map::<ItemInstance, ItemInstanceV975>()?;
    } else {
        w.map::<ItemInstanceV975, ItemInstance>()?;
    }
    Ok(())
}

fn copy_legacy_set_item_slots(w: &mut PacketWrapper) -> Result<()> {
    let slot_count = w.passthrough::<UVarInt>()?;
    for _ in 0..slot_count {
        w.passthrough::<Byte>()?;
        let slots_len = w.passthrough::<UVarInt>()?;
        for _ in 0..slots_len {
            w.passthrough::<Byte>()?;
        }
    }
    Ok(())
}

pub(crate) fn inventory_transaction(w: &mut PacketWrapper, to_v1001: bool) -> Result<()> {
    if to_v1001 {
        let legacy_request_id = w.passthrough::<VarInt>()?;
        let has_legacy = legacy_request_id != 0;
        w.write::<Bool>(has_legacy);
        if has_legacy {
            copy_legacy_set_item_slots(w)?;
        }
        w.write::<Bool>(true);
        let transaction_type = w.passthrough::<UVarInt>()?;
        w.write::<Bool>(true);
        let action_count = w.passthrough::<UVarInt>()?;
        for _ in 0..action_count {
            let action = read_old_action(w)?;
            write_new_action(w, &old_to_new_action(action))?;
        }
        match transaction_type {
            ITEM_USE_TRANSACTION => use_item_old_to_new(w)?,
            ITEM_USE_ON_ENTITY_TRANSACTION => use_item_on_entity_old_to_new(w)?,
            ITEM_RELEASE_TRANSACTION => release_item_old_to_new(w)?,
            _ => {}
        }
    } else {
        w.passthrough::<VarInt>()?;
        let has_legacy = w.read::<Bool>()?;
        if has_legacy {
            copy_legacy_set_item_slots(w)?;
        }
        w.read::<Bool>()?;
        let transaction_type = w.passthrough::<UVarInt>()?;
        w.read::<Bool>()?;
        let action_count = w.passthrough::<UVarInt>()?;
        for _ in 0..action_count {
            let action = read_new_action(w)?;
            write_old_action(w, &new_to_old_action(action))?;
        }
        match transaction_type {
            ITEM_USE_TRANSACTION => use_item_new_to_old(w)?,
            ITEM_USE_ON_ENTITY_TRANSACTION => use_item_on_entity_new_to_old(w)?,
            ITEM_RELEASE_TRANSACTION => release_item_new_to_old(w)?,
            _ => {}
        }
    }
    Ok(())
}

fn use_item_old_to_new(w: &mut PacketWrapper) -> Result<()> {
    let action_type = w.read::<UVarInt>()?;
    w.write::<VarInt>(action_type as i32);
    let trigger_type = w.read::<UVarInt>()?;
    w.write::<Byte>(trigger_type as u8);
    w.passthrough::<BlockPos>()?;
    let block_face = w.read::<VarInt>()?;
    w.write::<SByte>(block_face as i8);
    w.passthrough::<VarInt>()?;
    w.map::<ItemInstance, ItemInstanceV975>()?;
    w.passthrough::<Vec3>()?;
    w.passthrough::<Vec3>()?;
    w.passthrough::<UVarInt>()?;
    let client_prediction = w.read::<UVarInt>()?;
    w.write::<Byte>(client_prediction as u8);
    w.passthrough::<Byte>()?;
    Ok(())
}

fn use_item_new_to_old(w: &mut PacketWrapper) -> Result<()> {
    let action_type = w.read::<VarInt>()?;
    w.write::<UVarInt>(action_type as u32);
    let trigger_type = w.read::<Byte>()?;
    w.write::<UVarInt>(trigger_type as u32);
    w.passthrough::<BlockPos>()?;
    let block_face = w.read::<SByte>()?;
    w.write::<VarInt>(block_face as i32);
    w.passthrough::<VarInt>()?;
    w.map::<ItemInstanceV975, ItemInstance>()?;
    w.passthrough::<Vec3>()?;
    w.passthrough::<Vec3>()?;
    w.passthrough::<UVarInt>()?;
    let client_prediction = w.read::<Byte>()?;
    w.write::<UVarInt>(client_prediction as u32);
    w.passthrough::<Byte>()?;
    Ok(())
}

fn use_item_on_entity_old_to_new(w: &mut PacketWrapper) -> Result<()> {
    w.passthrough::<UVarInt64>()?;
    let action_type = w.read::<UVarInt>()?;
    w.write::<VarInt>(action_type as i32);
    w.passthrough::<VarInt>()?;
    w.map::<ItemInstance, ItemInstanceV975>()?;
    w.passthrough::<Vec3>()?;
    w.passthrough::<Vec3>()?;
    Ok(())
}

fn use_item_on_entity_new_to_old(w: &mut PacketWrapper) -> Result<()> {
    w.passthrough::<UVarInt64>()?;
    let action_type = w.read::<VarInt>()?;
    w.write::<UVarInt>(action_type as u32);
    w.passthrough::<VarInt>()?;
    w.map::<ItemInstanceV975, ItemInstance>()?;
    w.passthrough::<Vec3>()?;
    w.passthrough::<Vec3>()?;
    Ok(())
}

fn release_item_old_to_new(w: &mut PacketWrapper) -> Result<()> {
    let action_type = w.read::<UVarInt>()?;
    w.write::<VarInt>(action_type as i32);
    w.passthrough::<VarInt>()?;
    w.map::<ItemInstance, ItemInstanceV975>()?;
    w.passthrough::<Vec3>()?;
    Ok(())
}

fn release_item_new_to_old(w: &mut PacketWrapper) -> Result<()> {
    let action_type = w.read::<VarInt>()?;
    w.write::<UVarInt>(action_type as u32);
    w.passthrough::<VarInt>()?;
    w.map::<ItemInstanceV975, ItemInstance>()?;
    w.passthrough::<Vec3>()?;
    Ok(())
}
