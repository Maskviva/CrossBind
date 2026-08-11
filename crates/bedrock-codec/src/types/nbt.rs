use crate::{Codec, Error, Reader, Result, Writer};

pub const TAG_END: u8 = 0;
pub const TAG_BYTE: u8 = 1;
pub const TAG_SHORT: u8 = 2;
pub const TAG_INT: u8 = 3;
pub const TAG_LONG: u8 = 4;
pub const TAG_FLOAT: u8 = 5;
pub const TAG_DOUBLE: u8 = 6;
pub const TAG_BYTE_ARRAY: u8 = 7;
pub const TAG_STRING: u8 = 8;
pub const TAG_LIST: u8 = 9;
pub const TAG_COMPOUND: u8 = 10;
pub const TAG_INT_ARRAY: u8 = 11;
pub const TAG_LONG_ARRAY: u8 = 12;

const MAX_DEPTH: u32 = 512;

pub const EMPTY_NAMED_COMPOUND: [u8; 3] = [TAG_COMPOUND, 0x00, TAG_END];

fn skip_nbt_string(r: &mut Reader<'_>) -> Result<()> {
    let len = r.read_count()?;
    r.skip(len)
}

fn skip_payload(r: &mut Reader<'_>, tag_type: u8, depth: u32) -> Result<()> {
    if depth > MAX_DEPTH {
        return Err(Error::Invalid("NBT nesting too deep"));
    }
    match tag_type {
        TAG_BYTE => r.skip(1),
        TAG_SHORT => r.skip(2),
        TAG_INT => {
            r.read_varint()?;
            Ok(())
        }
        TAG_LONG => {
            r.read_varint64()?;
            Ok(())
        }
        TAG_FLOAT => r.skip(4),
        TAG_DOUBLE => r.skip(8),
        TAG_BYTE_ARRAY => {
            let len = r.read_varint()?;
            if len < 0 {
                return Err(Error::Invalid("negative NBT byte-array length"));
            }
            r.skip(len as usize)
        }
        TAG_STRING => skip_nbt_string(r),
        TAG_LIST => {
            let elem_type = r.read_u8()?;
            let count = r.read_varint()?;
            if count < 0 {
                return Err(Error::Invalid("negative NBT list length"));
            }
            if elem_type != TAG_END {
                for _ in 0..count {
                    skip_payload(r, elem_type, depth + 1)?;
                }
            }
            Ok(())
        }
        TAG_COMPOUND => {
            loop {
                let child = r.read_u8()?;
                if child == TAG_END {
                    return Ok(());
                }
                skip_nbt_string(r)?;
                skip_payload(r, child, depth + 1)?;
            }
        }
        TAG_INT_ARRAY => {
            let count = r.read_varint()?;
            if count < 0 {
                return Err(Error::Invalid("negative NBT int-array length"));
            }
            for _ in 0..count {
                r.read_varint()?;
            }
            Ok(())
        }
        TAG_LONG_ARRAY => {
            let count = r.read_varint()?;
            if count < 0 {
                return Err(Error::Invalid("negative NBT long-array length"));
            }
            for _ in 0..count {
                r.read_varint64()?;
            }
            Ok(())
        }
        other => Err(Error::BadDiscriminant {
            what: "NBT tag",
            value: other as i64,
        }),
    }
}

fn skip_named_tag(r: &mut Reader<'_>) -> Result<()> {
    let tag_type = r.read_u8()?;
    if tag_type == TAG_END {
        return Ok(());
    }
    if tag_type != TAG_COMPOUND {
        return Err(Error::BadDiscriminant {
            what: "NBT root tag",
            value: tag_type as i64,
        });
    }
    skip_nbt_string(r)?;
    skip_payload(r, TAG_COMPOUND, 0)
}

pub struct NamedCompoundTag;

impl Codec for NamedCompoundTag {
    type Value = Vec<u8>;

    fn read(r: &mut Reader<'_>) -> Result<Vec<u8>> {
        let start = r.position();
        skip_named_tag(r)?;
        Ok(r.bytes_from(start).to_vec())
    }

    fn write(w: &mut Writer, v: &Vec<u8>) {
        if v.is_empty() {
            w.write_bytes(&EMPTY_NAMED_COMPOUND);
        } else {
            w.write_bytes(v);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn roundtrip(bytes: &[u8]) -> Result<Vec<u8>> {
        let mut r = Reader::new(bytes);
        NamedCompoundTag::read(&mut r)
    }

    #[test]
    fn empty_compound_is_three_bytes() {
        let out = roundtrip(&EMPTY_NAMED_COMPOUND).unwrap();
        assert_eq!(out, EMPTY_NAMED_COMPOUND.to_vec());
    }

    #[test]
    fn root_end_is_one_byte() {
        let out = roundtrip(&[TAG_END, 0xAA]).unwrap();
        assert_eq!(out, vec![TAG_END]);
    }

    #[test]
    fn stops_exactly_at_tag_end_and_leaves_the_tail() {
        let tag = [
            TAG_COMPOUND, 0x00,
            TAG_BYTE, 0x01, b'a', 0x2A,
            TAG_END,
        ];
        let mut buf = tag.to_vec();
        buf.push(0xEE);
        let mut r = Reader::new(&buf);
        let out = NamedCompoundTag::read(&mut r).unwrap();
        assert_eq!(out, tag.to_vec());
        assert_eq!(r.remaining(), 1);
    }

    #[test]
    fn nested_list_of_compounds() {
        let tag = [
            TAG_COMPOUND, 0x00,
            TAG_LIST, 0x01, b'l',
            TAG_COMPOUND, 0x04,
            TAG_END,
            TAG_END,
            TAG_END,
        ];
        assert_eq!(roundtrip(&tag).unwrap(), tag.to_vec());
    }

    #[test]
    fn truncated_input_is_an_error_not_a_panic() {
        assert!(roundtrip(&[TAG_COMPOUND, 0x00]).is_err());
        assert!(roundtrip(&[TAG_COMPOUND]).is_err());
        assert!(roundtrip(&[]).is_err());
    }

    #[test]
    fn non_compound_root_is_rejected() {
        assert!(roundtrip(&[TAG_STRING, 0x00]).is_err());
    }
}
