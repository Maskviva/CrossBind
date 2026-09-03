use super::{CONTAINER_INVENTORY, NON_IMPLEMENTED_FEATURE_TODO, WORLD_INTERACTION};
use bedrock_codec::prelude::*;

#[derive(Clone)]
pub(crate) struct OldAction {
    source_type: u32,
    window_id: Option<i32>,
    source_flags: Option<u32>,
    slot: u32,
    old_item: Item,
    new_item: Item,
}

#[derive(Clone)]
pub(crate) struct NewAction {
    source_type: u32,
    window_id: Option<i8>,
    source_flags: Option<u32>,
    slot: u32,
    old_item: Item,
    new_item: Item,
}

pub(crate) fn read_old_action(w: &mut PacketWrapper) -> Result<OldAction> {
    let source_type = w.read::<UVarInt>()?;
    let mut window_id = None;
    let mut source_flags = None;
    if source_type == CONTAINER_INVENTORY || source_type == NON_IMPLEMENTED_FEATURE_TODO {
        window_id = Some(w.read::<VarInt>()?);
    } else if source_type == WORLD_INTERACTION {
        source_flags = Some(w.read::<UVarInt>()?);
    }
    let slot = w.read::<UVarInt>()?;
    let old_item = w.read::<ItemInstance>()?;
    let new_item = w.read::<ItemInstance>()?;
    Ok(OldAction {
        source_type,
        window_id,
        source_flags,
        slot,
        old_item,
        new_item,
    })
}

pub(crate) fn write_old_action(w: &mut PacketWrapper, a: &OldAction) -> Result<()> {
    w.write::<UVarInt>(a.source_type);
    if a.source_type == CONTAINER_INVENTORY || a.source_type == NON_IMPLEMENTED_FEATURE_TODO {
        w.write::<VarInt>(a.window_id.ok_or(Error::Invalid("missing WindowID"))?);
    } else if a.source_type == WORLD_INTERACTION {
        w.write::<UVarInt>(
            a.source_flags
                .ok_or(Error::Invalid("missing SourceFlags"))?,
        );
    }
    w.write::<UVarInt>(a.slot);
    w.write::<ItemInstance>(a.old_item.clone());
    w.write::<ItemInstance>(a.new_item.clone());
    Ok(())
}

pub(crate) fn read_new_action(w: &mut PacketWrapper) -> Result<NewAction> {
    let source_type = w.read::<UVarInt>()?;
    w.read::<Bool>()?;
    let has_container = w.read::<Bool>()?;
    let window_id = if has_container {
        Some(w.read::<SByte>()?)
    } else {
        None
    };
    w.read::<Bool>()?;
    let has_flags = w.read::<Bool>()?;
    let source_flags = if has_flags {
        Some(w.read::<UVarInt>()?)
    } else {
        None
    };
    let slot = w.read::<UVarInt>()?;
    let old_item = w.read::<ItemInstanceV975>()?;
    let new_item = w.read::<ItemInstanceV975>()?;
    Ok(NewAction {
        source_type,
        window_id,
        source_flags,
        slot,
        old_item,
        new_item,
    })
}

pub(crate) fn write_new_action(w: &mut PacketWrapper, a: &NewAction) -> Result<()> {
    let has_container =
        a.source_type == CONTAINER_INVENTORY || a.source_type == NON_IMPLEMENTED_FEATURE_TODO;
    let has_flags = a.source_type == WORLD_INTERACTION;
    w.write::<UVarInt>(a.source_type);
    w.write::<Bool>(true);
    w.write::<Bool>(has_container);
    if has_container {
        w.write::<SByte>(a.window_id.ok_or(Error::Invalid("missing WindowID"))?);
    }
    w.write::<Bool>(true);
    w.write::<Bool>(has_flags);
    if has_flags {
        w.write::<UVarInt>(
            a.source_flags
                .ok_or(Error::Invalid("missing SourceFlags"))?,
        );
    }
    w.write::<UVarInt>(a.slot);
    w.write::<ItemInstanceV975>(a.old_item.clone());
    w.write::<ItemInstanceV975>(a.new_item.clone());
    Ok(())
}

pub(crate) fn old_to_new_action(a: OldAction) -> NewAction {
    NewAction {
        source_type: a.source_type,
        window_id: a.window_id.map(|v| v as i8),
        source_flags: a.source_flags,
        slot: a.slot,
        old_item: a.old_item,
        new_item: a.new_item,
    }
}

pub(crate) fn new_to_old_action(a: NewAction) -> OldAction {
    OldAction {
        source_type: a.source_type,
        window_id: a.window_id.map(|v| v as i32),
        source_flags: a.source_flags,
        slot: a.slot,
        old_item: a.old_item,
        new_item: a.new_item,
    }
}
