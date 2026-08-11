pub struct Writer {
    buf: Vec<u8>,
}

impl Default for Writer {
    fn default() -> Self {
        Writer::new()
    }
}

impl Writer {
    pub fn new() -> Writer {
        Writer { buf: Vec::new() }
    }

    pub fn with_capacity(cap: usize) -> Writer {
        Writer {
            buf: Vec::with_capacity(cap),
        }
    }

    pub fn len(&self) -> usize {
        self.buf.len()
    }

    pub fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.buf
    }

    pub fn into_vec(self) -> Vec<u8> {
        self.buf
    }

    pub fn write_bytes(&mut self, bytes: &[u8]) {
        self.buf.extend_from_slice(bytes);
    }

    pub fn write_u8(&mut self, v: u8) {
        self.buf.push(v);
    }

    pub fn write_i8(&mut self, v: i8) {
        self.buf.push(v as u8);
    }

    pub fn write_bool(&mut self, v: bool) {
        self.buf.push(u8::from(v));
    }

    pub fn write_i16_le(&mut self, v: i16) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }

    pub fn write_u16_le(&mut self, v: u16) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }

    pub fn write_i32_le(&mut self, v: i32) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }

    pub fn write_u32_le(&mut self, v: u32) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }

    pub fn write_i32_be(&mut self, v: i32) {
        self.buf.extend_from_slice(&v.to_be_bytes());
    }

    pub fn write_i64_le(&mut self, v: i64) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }

    pub fn write_u64_le(&mut self, v: u64) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }

    pub fn write_f32_le(&mut self, v: f32) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }

    pub fn write_f64_le(&mut self, v: f64) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }

    pub fn write_uvarint(&mut self, mut v: u32) {
        while v >= 0x80 {
            self.buf.push((v as u8 & 0x7F) | 0x80);
            v >>= 7;
        }
        self.buf.push(v as u8);
    }

    pub fn write_varint(&mut self, v: i32) {
        self.write_uvarint(((v << 1) ^ (v >> 31)) as u32);
    }

    pub fn write_uvarint64(&mut self, mut v: u64) {
        while v >= 0x80 {
            self.buf.push((v as u8 & 0x7F) | 0x80);
            v >>= 7;
        }
        self.buf.push(v as u8);
    }

    pub fn write_varint64(&mut self, v: i64) {
        self.write_uvarint64(((v << 1) ^ (v >> 63)) as u64);
    }

    pub fn write_string(&mut self, v: &str) {
        self.write_uvarint(v.len() as u32);
        self.write_bytes(v.as_bytes());
    }

    pub fn write_count(&mut self, count: usize) {
        self.write_uvarint(count as u32);
    }
}
