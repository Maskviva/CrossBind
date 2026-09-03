use crate::item_remap;
use bedrock_codec::prelude::*;

pub(crate) fn creative_content(w: &mut PacketWrapper, to_v2168: bool) -> Result<()> {
    let groups = w.passthrough::<UVarInt>()?;
    for _ in 0..groups {
        if to_v2168 {
            let category = w.reader().read_i32_le()?;
            w.writer().write_u8(category as u8);
        } else {
            let category = w.reader().read_u8()?;
            w.writer().write_i32_le(category as i32);
        }
        w.passthrough::<Str>()?;
        creative_item_stack(w, to_v2168)?;
    }

    let items = w.passthrough::<UVarInt>()?;
    for _ in 0..items {
        w.passthrough::<UVarInt>()?;
        creative_item_stack(w, to_v2168)?;
        w.passthrough::<UVarInt>()?;
    }

    w.passthrough_all();
    Ok(())
}

fn creative_item_stack(w: &mut PacketWrapper, to_v2168: bool) -> Result<()> {
    if to_v2168 {
        map_item::<NetworkItemInstanceDescriptor, NetworkItemInstanceDescriptorV2168>(w, true)
    } else {
        map_item::<NetworkItemInstanceDescriptorV2168, NetworkItemInstanceDescriptor>(w, false)
    }
}

pub(crate) fn map_item<A, B>(w: &mut PacketWrapper, to_v2168: bool) -> Result<()>
where
    A: Codec<Value = Item>,
    B: Codec<Value = Item>,
{
    let mut item = w.read::<A>()?;
    item.network_id = if to_v2168 {
        item_remap::to_client(item.network_id)
    } else {
        item_remap::to_server(item.network_id)
    };
    w.write::<B>(item);
    Ok(())
}
