use bedrock_codec::prelude::*;

pub(crate) fn read_bitset(w: &mut PacketWrapper, size: u32) -> Result<Vec<u32>> {
    let mut set = Vec::new();
    let mut base = 0u32;
    while base < size {
        let byte = w.reader().read_u8()?;
        for bit in 0..7u32 {
            if byte & (1 << bit) != 0 {
                let id = base + bit;
                if id >= size {
                    return Err(Error::BadDiscriminant {
                        what: "player auth input flag",
                        value: id as i64,
                    });
                }
                set.push(id);
            }
        }
        if byte & 0x80 == 0 {
            return Ok(set);
        }
        base += 7;
    }
    Err(Error::BadDiscriminant {
        what: "player auth input bitset",
        value: size as i64,
    })
}

pub(crate) fn write_bitset(w: &mut PacketWrapper, set: &[u32], size: u32) {
    let highest = set.iter().copied().max().unwrap_or(0);
    let bytes = ((highest / 7) + 1).min(size.div_ceil(7));
    for group in 0..bytes {
        let mut byte = 0u8;
        for bit in 0..7u32 {
            if set.contains(&(group * 7 + bit)) {
                byte |= 1 << bit;
            }
        }
        if group + 1 < bytes {
            byte |= 0x80;
        }
        w.writer().write_u8(byte);
    }
}
