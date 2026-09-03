use crate::types::enums::data_item_type as dit;
use crate::types::nbt::NamedCompoundTag;
use crate::types::primitives::{BlockPos, Vec3};
use crate::{Codec, Error, Reader, Result, Writer};

#[derive(Debug, Clone, PartialEq)]
pub enum ActorDataValue {
    Byte(u8),
    Short(i16),
    Int(i32),
    Float(f32),
    Str(String),
    Compound(Vec<u8>),
    Pos((i32, i32, i32)),
    Int64(i64),
    Vec3((f32, f32, f32)),
}

impl ActorDataValue {
    pub fn type_id(&self) -> u32 {
        match self {
            ActorDataValue::Byte(_) => dit::BYTE,
            ActorDataValue::Short(_) => dit::SHORT,
            ActorDataValue::Int(_) => dit::INT,
            ActorDataValue::Float(_) => dit::FLOAT,
            ActorDataValue::Str(_) => dit::STRING,
            ActorDataValue::Compound(_) => dit::COMPOUND_TAG,
            ActorDataValue::Pos(_) => dit::POS,
            ActorDataValue::Int64(_) => dit::INT64,
            ActorDataValue::Vec3(_) => dit::VEC3,
        }
    }

    pub fn as_int(&self) -> Option<i64> {
        match self {
            ActorDataValue::Int(v) => Some(*v as i64),
            ActorDataValue::Int64(v) => Some(*v),
            _ => None,
        }
    }

    pub fn set_int(&mut self, value: i64) {
        match self {
            ActorDataValue::Int(slot) => *slot = value as i32,
            ActorDataValue::Int64(slot) => *slot = value,
            _ => {}
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActorDataItem {
    pub key: u32,
    pub value: ActorDataValue,
}

fn read_value(r: &mut Reader<'_>, type_id: u32) -> Result<ActorDataValue> {
    Ok(match type_id {
        dit::BYTE => ActorDataValue::Byte(r.read_u8()?),
        dit::SHORT => ActorDataValue::Short(r.read_i16_le()?),
        dit::INT => ActorDataValue::Int(r.read_varint()?),
        dit::FLOAT => ActorDataValue::Float(r.read_f32_le()?),
        dit::STRING => ActorDataValue::Str(r.read_string()?),
        dit::COMPOUND_TAG => ActorDataValue::Compound(NamedCompoundTag::read(r)?),
        dit::POS => ActorDataValue::Pos(BlockPos::read(r)?),
        dit::INT64 => ActorDataValue::Int64(r.read_varint64()?),
        dit::VEC3 => ActorDataValue::Vec3(Vec3::read(r)?),
        other => {
            return Err(Error::BadDiscriminant {
                what: "ActorData value type",
                value: other as i64,
            });
        }
    })
}

fn write_value(w: &mut Writer, v: &ActorDataValue) {
    match v {
        ActorDataValue::Byte(x) => w.write_u8(*x),
        ActorDataValue::Short(x) => w.write_i16_le(*x),
        ActorDataValue::Int(x) => w.write_varint(*x),
        ActorDataValue::Float(x) => w.write_f32_le(*x),
        ActorDataValue::Str(x) => w.write_string(x),
        ActorDataValue::Compound(x) => NamedCompoundTag::write(w, x),
        ActorDataValue::Pos(x) => BlockPos::write(w, x),
        ActorDataValue::Int64(x) => w.write_varint64(*x),
        ActorDataValue::Vec3(x) => Vec3::write(w, x),
    }
}

pub struct ActorDataEntry;

impl Codec for ActorDataEntry {
    type Value = ActorDataItem;

    fn read(r: &mut Reader<'_>) -> Result<ActorDataItem> {
        let key = r.read_uvarint()?;
        let type_id = r.read_uvarint()?;
        Ok(ActorDataItem {
            key,
            value: read_value(r, type_id)?,
        })
    }

    fn write(w: &mut Writer, v: &ActorDataItem) {
        w.write_uvarint(v.key);
        w.write_uvarint(v.value.type_id());
        write_value(w, &v.value);
    }
}

pub struct ActorDataEntryV2168;

impl Codec for ActorDataEntryV2168 {
    type Value = ActorDataItem;

    fn read(r: &mut Reader<'_>) -> Result<ActorDataItem> {
        let key = r.read_uvarint()?;
        let type_id = r.read_uvarint()?;
        let legacy_type = r.read_u8()? as u32;
        if legacy_type != type_id {
            return Err(Error::BadDiscriminant {
                what: "ActorData legacy value type",
                value: legacy_type as i64,
            });
        }
        Ok(ActorDataItem {
            key,
            value: read_value(r, type_id)?,
        })
    }

    fn write(w: &mut Writer, v: &ActorDataItem) {
        let type_id = v.value.type_id();
        w.write_uvarint(v.key);
        w.write_uvarint(type_id);
        w.write_u8(type_id as u8);
        write_value(w, &v.value);
    }
}

fn read_list<E: Codec<Value = ActorDataItem>>(r: &mut Reader<'_>) -> Result<Vec<ActorDataItem>> {
    let count = r.read_count()?;
    let mut out = Vec::with_capacity(count);
    for _ in 0..count {
        out.push(E::read(r)?);
    }
    Ok(out)
}

fn write_list<E: Codec<Value = ActorDataItem>>(w: &mut Writer, v: &[ActorDataItem]) {
    w.write_count(v.len());
    for entry in v {
        E::write(w, entry);
    }
}

pub struct ActorDataList;

impl Codec for ActorDataList {
    type Value = Vec<ActorDataItem>;

    fn read(r: &mut Reader<'_>) -> Result<Vec<ActorDataItem>> {
        read_list::<ActorDataEntry>(r)
    }

    fn write(w: &mut Writer, v: &Vec<ActorDataItem>) {
        write_list::<ActorDataEntry>(w, v);
    }
}

pub struct ActorDataListV2168;

impl Codec for ActorDataListV2168 {
    type Value = Vec<ActorDataItem>;

    fn read(r: &mut Reader<'_>) -> Result<Vec<ActorDataItem>> {
        read_list::<ActorDataEntryV2168>(r)
    }

    fn write(w: &mut Writer, v: &Vec<ActorDataItem>) {
        write_list::<ActorDataEntryV2168>(w, v);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_a_mixed_list() {
        let list = vec![
            ActorDataItem {
                key: 1,
                value: ActorDataValue::Byte(7),
            },
            ActorDataItem {
                key: 126,
                value: ActorDataValue::Int(566),
            },
            ActorDataItem {
                key: 9,
                value: ActorDataValue::Str("hello".into()),
            },
            ActorDataItem {
                key: 12,
                value: ActorDataValue::Vec3((1.0, -2.0, 3.5)),
            },
        ];
        let mut w = Writer::new();
        ActorDataList::write(&mut w, &list);
        let bytes = w.into_vec();
        let mut r = Reader::new(&bytes);
        assert_eq!(ActorDataList::read(&mut r).unwrap(), list);
        assert_eq!(r.remaining(), 0);
    }

    #[test]
    fn unknown_value_type_errors() {
        let bytes = [0x01, 0x01, 0xC8, 0x01];
        let mut r = Reader::new(&bytes);
        assert!(ActorDataList::read(&mut r).is_err());
    }

    #[test]
    fn v2168_adds_exactly_one_byte_per_entry() {
        let list = vec![
            ActorDataItem {
                key: 0,
                value: ActorDataValue::Byte(1),
            },
            ActorDataItem {
                key: 38,
                value: ActorDataValue::Int64(-7),
            },
            ActorDataItem {
                key: 81,
                value: ActorDataValue::Float(0.5),
            },
        ];
        let mut old = Writer::new();
        ActorDataList::write(&mut old, &list);
        let mut new = Writer::new();
        ActorDataListV2168::write(&mut new, &list);
        assert_eq!(new.len(), old.len() + list.len());
    }

    #[test]
    fn v2168_writes_the_type_twice_and_reads_it_back() {
        let list = vec![ActorDataItem {
            key: 4,
            value: ActorDataValue::Str("nametag".into()),
        }];
        let mut w = Writer::new();
        ActorDataListV2168::write(&mut w, &list);
        let bytes = w.into_vec();
        assert_eq!(
            &bytes[..4],
            &[0x01, 0x04, dit::STRING as u8, dit::STRING as u8]
        );

        let mut r = Reader::new(&bytes);
        assert_eq!(ActorDataListV2168::read(&mut r).unwrap(), list);
        assert_eq!(r.remaining(), 0);
    }

    #[test]
    fn v2168_rejects_a_list_that_is_really_the_older_shape() {
        let list = vec![ActorDataItem {
            key: 1,
            value: ActorDataValue::Int(1000),
        }];
        let mut w = Writer::new();
        ActorDataList::write(&mut w, &list);
        let bytes = w.into_vec();
        let mut r = Reader::new(&bytes);
        assert!(ActorDataListV2168::read(&mut r).is_err());
    }

    #[test]
    fn the_two_shapes_round_trip_through_each_other() {
        let list = vec![
            ActorDataItem {
                key: 1,
                value: ActorDataValue::Int(566),
            },
            ActorDataItem {
                key: 12,
                value: ActorDataValue::Vec3((1.0, -2.0, 3.5)),
            },
        ];
        let mut old = Writer::new();
        ActorDataList::write(&mut old, &list);
        let original = old.into_vec();

        let mut r = Reader::new(&original);
        let decoded = ActorDataList::read(&mut r).unwrap();
        let mut new = Writer::new();
        ActorDataListV2168::write(&mut new, &decoded);
        let widened = new.into_vec();

        let mut r = Reader::new(&widened);
        let decoded = ActorDataListV2168::read(&mut r).unwrap();
        let mut back = Writer::new();
        ActorDataList::write(&mut back, &decoded);
        assert_eq!(back.into_vec(), original);
    }

    #[test]
    fn set_int_only_touches_integer_cases() {
        let mut b = ActorDataValue::Byte(3);
        b.set_int(99);
        assert_eq!(b, ActorDataValue::Byte(3));

        let mut i = ActorDataValue::Int(3);
        i.set_int(99);
        assert_eq!(i, ActorDataValue::Int(99));
    }
}
