use super::movement::byte_width;
use bedrock_codec::prelude::*;

pub(crate) fn inventory_slot(w: &mut PacketWrapper, to_v975: bool) -> Result<()> {
    w.passthrough::<UVarInt>()?;
    w.passthrough::<UVarInt>()?;

    if to_v975 {
        let name = w.read::<Byte>()?;
        let has_dynamic = w.read::<Bool>()?;
        let dynamic = if has_dynamic { w.read::<UIntLe>()? } else { 0 };
        w.write::<Bool>(true);
        w.write::<Byte>(name);
        w.write::<Bool>(has_dynamic);
        if has_dynamic {
            w.write::<UIntLe>(dynamic);
        }

        let storage = w.read::<ItemInstance>()?;
        if storage.is_air() {
            w.write::<Bool>(false);
        } else {
            w.write::<Bool>(true);
            w.write::<ItemInstanceV975>(storage);
        }
    } else {
        if w.read::<Bool>()? {
            let name = w.read::<Byte>()?;
            let has_dynamic = w.read::<Bool>()?;
            let dynamic = if has_dynamic { w.read::<UIntLe>()? } else { 0 };
            w.write::<Byte>(name);
            w.write::<Bool>(has_dynamic);
            if has_dynamic {
                w.write::<UIntLe>(dynamic);
            }
        } else {
            w.write::<Byte>(0);
            w.write::<Bool>(false);
        }

        if w.read::<Bool>()? {
            let storage = w.read::<ItemInstanceV975>()?;
            w.write::<ItemInstance>(storage);
        } else {
            w.write::<ItemInstance>(air());
        }
    }

    convert_item(w, to_v975)?;
    Ok(())
}

pub(crate) fn player_equipment(w: &mut PacketWrapper, to_v975: bool) -> Result<()> {
    w.passthrough::<UVarInt64>()?;
    convert_item(w, to_v975)?;
    byte_width(w, to_v975)?;
    byte_width(w, to_v975)?;
    byte_width(w, to_v975)?;
    Ok(())
}

pub(crate) fn player_enchant_options(w: &mut PacketWrapper, to_v975: bool) -> Result<()> {
    w.passthrough_each(|w| {
        if to_v975 {
            let cost = w.read::<UVarInt>()?;
            w.write::<Byte>(cost.min(0xFF) as u8);
        } else {
            let cost = w.read::<Byte>()?;
            w.write::<UVarInt>(u32::from(cost));
        }
        w.passthrough::<IntLe>()?;
        for _ in 0..3 {
            w.passthrough_each(|w| {
                w.passthrough::<Byte>()?;
                w.passthrough::<Byte>()?;
                Ok(())
            })?;
        }
        w.passthrough::<Str>()?;
        w.passthrough::<UVarInt>()?;
        Ok(())
    })?;
    Ok(())
}

pub(crate) fn convert_item(w: &mut PacketWrapper, to_v975: bool) -> Result<()> {
    if to_v975 {
        w.map::<ItemInstance, ItemInstanceV975>()?;
    } else {
        w.map::<ItemInstanceV975, ItemInstance>()?;
    }
    Ok(())
}

pub(crate) fn air() -> Item {
    Item {
        network_id: 0,
        count: 0,
        aux_value: 0,
        has_net_id: false,
        stack_net_id: 0,
        net_id_variant: 0,
        block_runtime_id: 0,
        user_data: Vec::new(),
    }
}
