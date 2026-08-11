use crate::{Error, Result};

pub const MAX_STRING_BYTES: usize = 32767 * 4;

pub struct Reader<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    pub fn new(data: &'a [u8]) -> Reader<'a> {
        Reader { data, pos: 0 }
    }

    pub fn position(&self) -> usize {
        self.pos
    }

    pub fn set_position(&mut self, pos: usize) {
        self.pos = pos.min(self.data.len());
    }

    pub fn remaining(&self) -> usize {
        self.data.len() - self.pos
    }

    pub fn has_remaining(&self) -> bool {
        self.pos < self.data.len()
    }

    fn need(&self, n: usize) -> Result<()> {
        if n > self.remaining() {
            return Err(Error::Eof {
                needed: n,
                remaining: self.remaining(),
            });
        }
        Ok(())
    }

    pub fn skip(&mut self, n: usize) -> Result<()> {
        self.need(n)?;
        self.pos += n;
        Ok(())
    }

    pub fn read_bytes(&mut self, n: usize) -> Result<&'a [u8]> {
        self.need(n)?;
        let out = &self.data[self.pos..self.pos + n];
        self.pos += n;
        Ok(out)
    }

    pub fn bytes_from(&self, start: usize) -> &'a [u8] {
        let start = start.min(self.pos);
        &self.data[start..self.pos]
    }

    pub fn read_remaining(&mut self) -> &'a [u8] {
        let out = &self.data[self.pos..];
        self.pos = self.data.len();
        out
    }

    fn read_array<const N: usize>(&mut self) -> Result<[u8; N]> {
        self.need(N)?;
        let mut buf = [0u8; N];
        buf.copy_from_slice(&self.data[self.pos..self.pos + N]);
        self.pos += N;
        Ok(buf)
    }

    pub fn read_u8(&mut self) -> Result<u8> {
        self.need(1)?;
        let v = self.data[self.pos];
        self.pos += 1;
        Ok(v)
    }

    pub fn read_i8(&mut self) -> Result<i8> {
        Ok(self.read_u8()? as i8)
    }

    pub fn read_bool(&mut self) -> Result<bool> {
        Ok(self.read_u8()? != 0)
    }

    pub fn read_i16_le(&mut self) -> Result<i16> {
        Ok(i16::from_le_bytes(self.read_array::<2>()?))
    }

    pub fn read_u16_le(&mut self) -> Result<u16> {
        Ok(u16::from_le_bytes(self.read_array::<2>()?))
    }

    pub fn read_i32_le(&mut self) -> Result<i32> {
        Ok(i32::from_le_bytes(self.read_array::<4>()?))
    }

    pub fn read_u32_le(&mut self) -> Result<u32> {
        Ok(u32::from_le_bytes(self.read_array::<4>()?))
    }

    pub fn read_i32_be(&mut self) -> Result<i32> {
        Ok(i32::from_be_bytes(self.read_array::<4>()?))
    }

    pub fn read_i64_le(&mut self) -> Result<i64> {
        Ok(i64::from_le_bytes(self.read_array::<8>()?))
    }

    pub fn read_u64_le(&mut self) -> Result<u64> {
        Ok(u64::from_le_bytes(self.read_array::<8>()?))
    }

    pub fn read_f32_le(&mut self) -> Result<f32> {
        Ok(f32::from_le_bytes(self.read_array::<4>()?))
    }

    pub fn read_f64_le(&mut self) -> Result<f64> {
        Ok(f64::from_le_bytes(self.read_array::<8>()?))
    }

    pub fn read_uvarint(&mut self) -> Result<u32> {
        let mut result: u32 = 0;
        let mut shift = 0u32;
        loop {
            let b = self.read_u8()?;
            result |= ((b & 0x7F) as u32) << shift;
            if b & 0x80 == 0 {
                return Ok(result);
            }
            shift += 7;
            if shift >= 35 {
                return Err(Error::VarIntTooLong);
            }
        }
    }

    pub fn read_varint(&mut self) -> Result<i32> {
        let raw = self.read_uvarint()?;
        Ok(((raw >> 1) as i32) ^ -((raw & 1) as i32))
    }

    pub fn read_uvarint64(&mut self) -> Result<u64> {
        let mut result: u64 = 0;
        let mut shift = 0u32;
        loop {
            let b = self.read_u8()?;
            result |= ((b & 0x7F) as u64) << shift;
            if b & 0x80 == 0 {
                return Ok(result);
            }
            shift += 7;
            if shift >= 70 {
                return Err(Error::VarIntTooLong);
            }
        }
    }

    pub fn read_varint64(&mut self) -> Result<i64> {
        let raw = self.read_uvarint64()?;
        Ok(((raw >> 1) as i64) ^ -((raw & 1) as i64))
    }

    pub fn read_string(&mut self) -> Result<String> {
        let len = self.read_uvarint()? as usize;
        if len > MAX_STRING_BYTES {
            return Err(Error::LengthLimit {
                got: len,
                limit: MAX_STRING_BYTES,
            });
        }
        let bytes = self.read_bytes(len)?;
        String::from_utf8(bytes.to_vec()).map_err(|_| Error::BadUtf8)
    }

    pub fn read_count(&mut self) -> Result<usize> {
        let count = self.read_uvarint()? as usize;
        if count > self.remaining() {
            return Err(Error::LengthLimit {
                got: count,
                limit: self.remaining(),
            });
        }
        Ok(count)
    }
}
