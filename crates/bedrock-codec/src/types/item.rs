use crate::{Codec, Reader, Result, Writer};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Item {
    pub network_id: i32,
    pub count: u16,
    pub aux_value: u32,
    pub has_net_id: bool,
    pub stack_net_id: i32,
    pub net_id_variant: u32,
    pub block_runtime_id: i32,
    pub user_data: Vec<u8>,
}

impl Item {
    pub fn is_air(&self) -> bool {
        self.network_id == 0
    }
}

pub struct ItemInstance;

impl Codec for ItemInstance {
    type Value = Item;

    fn read(r: &mut Reader<'_>) -> Result<Item> {
        let network_id = r.read_varint()?;
        if network_id == 0 {
            return Ok(Item::default());
        }
        let count = r.read_u16_le()?;
        let aux_value = r.read_uvarint()?;
        let has_net_id = r.read_bool()?;
        let stack_net_id = if has_net_id { r.read_varint()? } else { 0 };
        let block_runtime_id = r.read_varint()?;
        let extra_len = r.read_count()?;
        let user_data = r.read_bytes(extra_len)?.to_vec();
        Ok(Item {
            network_id,
            count,
            aux_value,
            has_net_id,
            stack_net_id,
            net_id_variant: 0,
            block_runtime_id,
            user_data,
        })
    }

    fn write(w: &mut Writer, v: &Item) {
        w.write_varint(v.network_id);
        if v.network_id == 0 {
            return;
        }
        w.write_u16_le(v.count);
        w.write_uvarint(v.aux_value);
        w.write_bool(v.has_net_id);
        if v.has_net_id {
            w.write_varint(v.stack_net_id);
        }
        w.write_varint(v.block_runtime_id);
        w.write_count(v.user_data.len());
        w.write_bytes(&v.user_data);
    }
}

pub struct NetworkItemInstanceDescriptor;

impl Codec for NetworkItemInstanceDescriptor {
    type Value = Item;

    fn read(r: &mut Reader<'_>) -> Result<Item> {
        let network_id = r.read_varint()?;
        if network_id == 0 {
            return Ok(Item::default());
        }
        let count = r.read_u16_le()?;
        let aux_value = r.read_uvarint()?;
        let block_runtime_id = r.read_varint()?;
        let extra_len = r.read_count()?;
        let user_data = r.read_bytes(extra_len)?.to_vec();
        Ok(Item {
            network_id,
            count,
            aux_value,
            has_net_id: false,
            stack_net_id: 0,
            net_id_variant: 0,
            block_runtime_id,
            user_data,
        })
    }

    fn write(w: &mut Writer, v: &Item) {
        w.write_varint(v.network_id);
        if v.network_id == 0 {
            return;
        }
        w.write_u16_le(v.count);
        w.write_uvarint(v.aux_value);
        w.write_varint(v.block_runtime_id);
        w.write_count(v.user_data.len());
        w.write_bytes(&v.user_data);
    }
}

pub struct ItemInstanceV975;

impl Codec for ItemInstanceV975 {
    type Value = Item;

    fn read(r: &mut Reader<'_>) -> Result<Item> {
        let network_id = r.read_i16_le()? as i32;
        let count = r.read_u16_le()?;
        let aux_value = r.read_uvarint()?;
        let has_net_id = r.read_bool()?;
        let (net_id_variant, stack_net_id) = if has_net_id {
            (r.read_uvarint()?, r.read_varint()?)
        } else {
            (0, 0)
        };
        let block_runtime_id = r.read_uvarint()? as i32;
        let extra_len = r.read_count()?;
        let user_data = r.read_bytes(extra_len)?.to_vec();
        Ok(Item {
            network_id,
            count,
            aux_value,
            has_net_id,
            stack_net_id,
            net_id_variant,
            block_runtime_id,
            user_data,
        })
    }

    fn write(w: &mut Writer, v: &Item) {
        w.write_i16_le(v.network_id as i16);
        w.write_u16_le(v.count);
        w.write_uvarint(v.aux_value);
        w.write_bool(v.has_net_id);
        if v.has_net_id {
            w.write_uvarint(v.net_id_variant);
            w.write_varint(v.stack_net_id);
        }
        w.write_uvarint(v.block_runtime_id as u32);
        w.write_count(v.user_data.len());
        w.write_bytes(&v.user_data);
    }
}

pub struct NetworkItemInstanceDescriptorV2168;

impl Codec for NetworkItemInstanceDescriptorV2168 {
    type Value = Item;

    fn read(r: &mut Reader<'_>) -> Result<Item> {
        let network_id = r.read_varint()?;
        let count = r.read_u16_le()?;
        let aux_value = r.read_uvarint()?;
        let block_runtime_id = r.read_varint()?;
        let extra_len = r.read_count()?;
        let user_data = r.read_bytes(extra_len)?.to_vec();
        Ok(Item {
            network_id,
            count,
            aux_value,
            has_net_id: false,
            stack_net_id: 0,
            net_id_variant: 0,
            block_runtime_id,
            user_data,
        })
    }

    fn write(w: &mut Writer, v: &Item) {
        w.write_varint(v.network_id);
        w.write_u16_le(v.count);
        w.write_uvarint(v.aux_value);
        w.write_varint(v.block_runtime_id);
        w.write_count(v.user_data.len());
        w.write_bytes(&v.user_data);
    }
}

pub struct ItemInstanceV2168;

impl Codec for ItemInstanceV2168 {
    type Value = Item;

    fn read(r: &mut Reader<'_>) -> Result<Item> {
        let network_id = r.read_i16_le()? as i32;
        let count = r.read_u16_le()?;
        let aux_value = r.read_uvarint()?;
        let has_net_id = r.read_bool()?;
        let stack_net_id = if has_net_id { r.read_varint()? } else { 0 };
        let block_runtime_id = r.read_uvarint()? as i32;
        let extra_len = r.read_count()?;
        let user_data = r.read_bytes(extra_len)?.to_vec();
        Ok(Item {
            network_id,
            count,
            aux_value,
            has_net_id,
            stack_net_id,
            net_id_variant: 0,
            block_runtime_id,
            user_data,
        })
    }

    fn write(w: &mut Writer, v: &Item) {
        w.write_i16_le(v.network_id as i16);
        w.write_u16_le(v.count);
        w.write_uvarint(v.aux_value);
        w.write_bool(v.has_net_id);
        if v.has_net_id {
            w.write_varint(v.stack_net_id);
        }
        w.write_uvarint(v.block_runtime_id as u32);
        w.write_count(v.user_data.len());
        w.write_bytes(&v.user_data);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn air_round_trips_as_a_single_byte_pre_v975() {
        let mut w = Writer::new();
        ItemInstance::write(&mut w, &Item::default());
        assert_eq!(w.as_slice(), &[0x00]);
    }

    #[test]
    fn v975_air_still_writes_every_field() {
        let mut w = Writer::new();
        ItemInstanceV975::write(&mut w, &Item::default());
        assert_eq!(w.len(), 2 + 2 + 1 + 1 + 1 + 1);
    }

    #[test]
    fn v2168_differs_from_v975_by_exactly_the_variant_tag() {
        let with_net_id = Item {
            network_id: 5,
            count: 1,
            aux_value: 0,
            has_net_id: true,
            stack_net_id: 7,
            net_id_variant: 0,
            block_runtime_id: 0,
            user_data: Vec::new(),
        };
        let mut old = Writer::new();
        ItemInstanceV975::write(&mut old, &with_net_id);
        let mut new = Writer::new();
        ItemInstanceV2168::write(&mut new, &with_net_id);
        assert_eq!(old.len(), new.len() + 1);

        let air = Item::default();
        let mut old_air = Writer::new();
        ItemInstanceV975::write(&mut old_air, &air);
        let mut new_air = Writer::new();
        ItemInstanceV2168::write(&mut new_air, &air);
        assert_eq!(old_air.as_slice(), new_air.as_slice());
    }

    #[test]
    fn descriptor_air_is_one_byte_before_v2168_and_six_after() {
        let mut old = Writer::new();
        NetworkItemInstanceDescriptor::write(&mut old, &Item::default());
        assert_eq!(old.as_slice(), &[0x00]);

        let mut new = Writer::new();
        NetworkItemInstanceDescriptorV2168::write(&mut new, &Item::default());
        assert_eq!(new.len(), 1 + 2 + 1 + 1 + 1);
    }

    #[test]
    fn descriptor_v2168_round_trips_air_and_a_real_stack() {
        for item in [
            Item::default(),
            Item {
                network_id: 17,
                count: 12,
                aux_value: 4,
                has_net_id: false,
                stack_net_id: 0,
                net_id_variant: 0,
                block_runtime_id: -88,
                user_data: vec![4, 5],
            },
        ] {
            let mut w = Writer::new();
            NetworkItemInstanceDescriptorV2168::write(&mut w, &item);
            let bytes = w.into_vec();
            let mut r = Reader::new(&bytes);
            assert_eq!(
                NetworkItemInstanceDescriptorV2168::read(&mut r).unwrap(),
                item
            );
            assert_eq!(r.remaining(), 0);
        }
    }

    #[test]
    fn v2168_round_trips() {
        let item = Item {
            network_id: -3,
            count: 64,
            aux_value: 11,
            has_net_id: true,
            stack_net_id: -9,
            net_id_variant: 0,
            block_runtime_id: 1234,
            user_data: vec![9, 8, 7],
        };
        let mut w = Writer::new();
        ItemInstanceV2168::write(&mut w, &item);
        let bytes = w.into_vec();
        let mut r = Reader::new(&bytes);
        assert_eq!(ItemInstanceV2168::read(&mut r).unwrap(), item);
        assert_eq!(r.remaining(), 0);
    }

    #[test]
    fn v975_round_trip_preserves_the_net_id_variant() {
        let item = Item {
            network_id: 12,
            count: 3,
            aux_value: 7,
            has_net_id: true,
            stack_net_id: -4,
            net_id_variant: 2,
            block_runtime_id: 99,
            user_data: vec![1, 2, 3],
        };
        let mut w = Writer::new();
        ItemInstanceV975::write(&mut w, &item);
        let bytes = w.into_vec();
        let mut r = Reader::new(&bytes);
        assert_eq!(ItemInstanceV975::read(&mut r).unwrap(), item);
        assert_eq!(r.remaining(), 0);
    }
}
