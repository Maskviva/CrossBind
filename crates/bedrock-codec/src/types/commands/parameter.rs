use crate::types::primitives::Array;
use crate::{Codec, Reader, Result, Writer};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandParameter {
    pub name: String,
    pub symbol: i32,
    pub optional: bool,
    pub options: u8,
}

pub struct CommandParameterCodec;

impl Codec for CommandParameterCodec {
    type Value = CommandParameter;
    fn read(r: &mut Reader<'_>) -> Result<CommandParameter> {
        Ok(CommandParameter {
            name: r.read_string()?,
            symbol: r.read_i32_le()?,
            optional: r.read_bool()?,
            options: r.read_u8()?,
        })
    }
    fn write(w: &mut Writer, v: &CommandParameter) {
        w.write_string(&v.name);
        w.write_i32_le(v.symbol);
        w.write_bool(v.optional);
        w.write_u8(v.options);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandOverload {
    pub chaining: bool,
    pub parameters: Vec<CommandParameter>,
}

pub struct CommandOverloadCodec;

impl Codec for CommandOverloadCodec {
    type Value = CommandOverload;
    fn read(r: &mut Reader<'_>) -> Result<CommandOverload> {
        let chaining = r.read_bool()?;
        let parameters = Array::<CommandParameterCodec>::read(r)?;
        Ok(CommandOverload {
            chaining,
            parameters,
        })
    }
    fn write(w: &mut Writer, v: &CommandOverload) {
        w.write_bool(v.chaining);
        Array::<CommandParameterCodec>::write(w, &v.parameters);
    }
}
