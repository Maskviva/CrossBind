mod action;
mod descriptor;
mod registry;
mod response;

pub(crate) use registry::*;
pub(crate) use response::*;

#[cfg(test)]
mod tests;

use action::{action, ActionOutcome};
use bedrock_codec::prelude::*;
use std::collections::HashMap;

pub fn item_stack_request(
    w: &mut PacketWrapper,
    to_v2168: bool,
    names: &HashMap<String, i32>,
    ids: &HashMap<i32, String>,
) -> Result<bool> {
    let mut out = Writer::new();
    let requests = w.reader().read_count()?;
    out.write_count(requests);

    for _ in 0..requests {
        out.write_varint(w.reader().read_varint()?);

        let count = w.reader().read_count()?;
        out.write_count(count);
        for _ in 0..count {
            if action(w.reader(), &mut out, to_v2168, names, ids)? == ActionOutcome::Blocked {
                return Ok(false);
            }
        }

        let filters = w.reader().read_count()?;
        out.write_count(filters);
        for _ in 0..filters {
            let s = Str::read(w.reader())?;
            Str::write(&mut out, &s);
        }

        out.write_i32_le(w.reader().read_i32_le()?);
    }

    w.writer().write_bytes(&out.into_vec());
    w.passthrough_all();
    Ok(true)
}
