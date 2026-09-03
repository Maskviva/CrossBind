use super::creative::map_item;
use bedrock_codec::prelude::*;

pub(crate) fn inventory_transaction(w: &mut PacketWrapper, to_v2168: bool) -> Result<()> {
    w.passthrough::<VarInt>()?;
    if w.passthrough::<Bool>()? {
        copy_legacy_set_item_slots(w)?;
    }
    w.passthrough::<Bool>()?;
    let transaction_type = w.passthrough::<UVarInt>()?;
    w.passthrough::<Bool>()?;
    w.passthrough_each(|w| inventory_action(w, to_v2168))?;
    match transaction_type {
        TRANSACTION_ITEM_USE => use_item_transaction(w, to_v2168)?,
        TRANSACTION_ITEM_USE_ON_ENTITY => use_item_on_entity_transaction(w, to_v2168)?,
        TRANSACTION_ITEM_RELEASE => release_item_transaction(w, to_v2168)?,
        _ => {}
    }
    Ok(())
}

fn use_item_transaction(w: &mut PacketWrapper, to_v2168: bool) -> Result<()> {
    w.passthrough::<VarInt>()?;
    w.passthrough::<Byte>()?;
    w.passthrough::<BlockPos>()?;
    w.passthrough::<Byte>()?;
    w.passthrough::<VarInt>()?;
    convert_item(w, to_v2168)?;
    w.passthrough::<Vec3>()?;
    w.passthrough::<Vec3>()?;
    w.passthrough::<UVarInt>()?;
    w.passthrough::<Byte>()?;
    w.passthrough::<Byte>()?;
    Ok(())
}

fn use_item_on_entity_transaction(w: &mut PacketWrapper, to_v2168: bool) -> Result<()> {
    w.passthrough::<UVarInt64>()?;
    w.passthrough::<VarInt>()?;
    w.passthrough::<VarInt>()?;
    convert_item(w, to_v2168)?;
    w.passthrough::<Vec3>()?;
    w.passthrough::<Vec3>()?;
    Ok(())
}

fn release_item_transaction(w: &mut PacketWrapper, to_v2168: bool) -> Result<()> {
    w.passthrough::<VarInt>()?;
    w.passthrough::<VarInt>()?;
    convert_item(w, to_v2168)?;
    w.passthrough::<Vec3>()?;
    Ok(())
}

fn inventory_action(w: &mut PacketWrapper, to_v2168: bool) -> Result<()> {
    w.passthrough::<UVarInt>()?;
    double_optional(w, !to_v2168, |w| {
        w.passthrough::<SByte>()?;
        Ok(())
    })?;
    double_optional(w, !to_v2168, |w| {
        w.passthrough::<UVarInt>()?;
        Ok(())
    })?;
    w.passthrough::<UVarInt>()?;
    convert_item(w, to_v2168)?;
    convert_item(w, to_v2168)?;
    Ok(())
}

fn copy_legacy_set_item_slots(w: &mut PacketWrapper) -> Result<()> {
    w.passthrough_each(|w| {
        w.passthrough::<Byte>()?;
        w.passthrough_each(|w| {
            w.passthrough::<Byte>()?;
            Ok(())
        })?;
        Ok(())
    })?;
    Ok(())
}

fn double_optional<'a>(
    w: &mut PacketWrapper<'a>,
    from_v2168: bool,
    value: impl FnOnce(&mut PacketWrapper<'a>) -> Result<()>,
) -> Result<()> {
    let outer = w.read::<Bool>()?;
    let inner = if outer || !from_v2168 {
        w.read::<Bool>()?
    } else {
        false
    };
    w.write::<Bool>(true);
    w.write::<Bool>(inner);
    if inner {
        value(w)?;
    }
    Ok(())
}

pub(crate) fn inventory_content(w: &mut PacketWrapper, to_v2168: bool) -> Result<()> {
    w.passthrough::<UVarInt>()?;
    let count = w.passthrough::<UVarInt>()?;
    for _ in 0..count {
        convert_item(w, to_v2168)?;
    }
    w.passthrough::<Byte>()?;
    w.passthrough::<Optional<UIntLe>>()?;
    convert_item(w, to_v2168)?;
    w.passthrough_all();
    Ok(())
}

pub(crate) fn inventory_slot(w: &mut PacketWrapper, to_v2168: bool) -> Result<()> {
    w.passthrough::<Byte>()?;
    w.passthrough::<UVarInt>()?;
    if w.passthrough::<Bool>()? {
        w.passthrough::<Byte>()?;
        w.passthrough::<Optional<UIntLe>>()?;
    }
    if w.passthrough::<Bool>()? {
        convert_item(w, to_v2168)?;
    }
    convert_item(w, to_v2168)?;
    w.passthrough_all();
    Ok(())
}

pub(crate) fn player_equipment(w: &mut PacketWrapper, to_v2168: bool) -> Result<()> {
    w.passthrough::<UVarInt64>()?;
    convert_item(w, to_v2168)?;
    w.passthrough_all();
    Ok(())
}

pub(crate) fn mob_armor_equipment(w: &mut PacketWrapper, to_v2168: bool) -> Result<()> {
    w.passthrough::<UVarInt64>()?;
    for _ in 0..5 {
        convert_item(w, to_v2168)?;
    }
    w.passthrough_all();
    Ok(())
}

fn convert_item(w: &mut PacketWrapper, to_v2168: bool) -> Result<()> {
    if to_v2168 {
        map_item::<ItemInstanceV975, ItemInstanceV2168>(w, true)
    } else {
        map_item::<ItemInstanceV2168, ItemInstanceV975>(w, false)
    }
}

pub(crate) fn convert_legacy_item(w: &mut PacketWrapper, to_v2168: bool) -> Result<()> {
    if to_v2168 {
        map_item::<ItemInstance, ItemInstanceV2168>(w, true)
    } else {
        map_item::<ItemInstanceV2168, ItemInstance>(w, false)
    }
}

pub(crate) const TRANSACTION_ITEM_USE: u32 = 2;
pub(crate) const TRANSACTION_ITEM_USE_ON_ENTITY: u32 = 3;
pub(crate) const TRANSACTION_ITEM_RELEASE: u32 = 4;
