use crate::{Codec, Reader, Result, Writer};

pub struct PacketWrapper<'a> {
    reader: Reader<'a>,
    writer: Writer,
    cancelled: bool,
}

impl<'a> PacketWrapper<'a> {
    pub fn new(payload: &'a [u8]) -> PacketWrapper<'a> {
        PacketWrapper {
            writer: Writer::with_capacity(payload.len()),
            reader: Reader::new(payload),
            cancelled: false,
        }
    }

    pub fn reader(&mut self) -> &mut Reader<'a> {
        &mut self.reader
    }

    pub fn writer(&mut self) -> &mut Writer {
        &mut self.writer
    }

    pub fn has_remaining(&self) -> bool {
        self.reader.has_remaining()
    }

    pub fn remaining(&self) -> usize {
        self.reader.remaining()
    }

    pub fn cancel(&mut self) {
        self.cancelled = true;
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }

    pub fn passthrough<C: Codec>(&mut self) -> Result<C::Value> {
        let value = C::read(&mut self.reader)?;
        C::write(&mut self.writer, &value);
        Ok(value)
    }

    pub fn read<C: Codec>(&mut self) -> Result<C::Value> {
        C::read(&mut self.reader)
    }

    pub fn write<C: Codec>(&mut self, value: C::Value) {
        C::write(&mut self.writer, &value);
    }

    pub fn map<From, To>(&mut self) -> Result<From::Value>
    where
        From: Codec,
        To: Codec<Value = From::Value>,
    {
        let value = From::read(&mut self.reader)?;
        To::write(&mut self.writer, &value);
        Ok(value)
    }

    pub fn passthrough_all(&mut self) {
        let rest = self.reader.read_remaining();
        self.writer.write_bytes(rest);
    }

    pub fn passthrough_each(
        &mut self,
        mut each: impl FnMut(&mut Self) -> Result<()>,
    ) -> Result<usize> {
        let count = self.reader.read_count()?;
        self.writer.write_count(count);
        for _ in 0..count {
            each(self)?;
        }
        Ok(count)
    }

    pub fn finish(mut self) -> Vec<u8> {
        let rest = self.reader.read_remaining();
        self.writer.write_bytes(rest);
        self.writer.into_vec()
    }
}
