use super::action::{full_container_name, read_double_optional_from};
use bedrock_codec::prelude::*;

pub fn item_stack_response(w: &mut PacketWrapper, to_v2168: bool) -> Result<()> {
    let mut out = Writer::new();
    let entries = w.reader().read_count()?;
    out.write_count(entries);

    for _ in 0..entries {
        let status = w.reader().read_u8()?;
        out.write_u8(status);
        out.write_varint(w.reader().read_varint()?);

        if to_v2168 {
            let present = status == RESPONSE_STATUS_OK;
            out.write_bool(true);
            out.write_bool(present);
            if present {
                response_containers(w.reader(), &mut out, true)?;
            }
        } else {
            let present = read_double_optional_from(w.reader())?;
            let writes = status == RESPONSE_STATUS_OK;
            if present {
                let mut buffered = Writer::new();
                response_containers(w.reader(), &mut buffered, false)?;
                if writes {
                    out.write_bytes(&buffered.into_vec());
                }
            } else if writes {
                out.write_count(0);
            }
        }
    }

    w.writer().write_bytes(&out.into_vec());
    w.passthrough_all();
    Ok(())
}

fn response_containers(r: &mut Reader<'_>, w: &mut Writer, to_v2168: bool) -> Result<()> {
    let containers = r.read_count()?;
    w.write_count(containers);
    for _ in 0..containers {
        full_container_name(r, w)?;
        let slots = r.read_count()?;
        w.write_count(slots);
        for _ in 0..slots {
            w.write_u8(r.read_u8()?);
            w.write_u8(r.read_u8()?);
            w.write_u8(r.read_u8()?);

            if to_v2168 {
                let net = r.read_varint()?;
                w.write_bool(true);
                w.write_bool(true);
                w.write_varint(net);
            } else if read_double_optional_from(r)? {
                w.write_varint(r.read_varint()?);
            } else {
                w.write_varint(0);
            }

            let name = Str::read(r)?;
            Str::write(w, &name);
            let filtered = Str::read(r)?;
            Str::write(w, &filtered);
            w.write_varint(r.read_varint()?);
        }
    }
    Ok(())
}

pub(super) const RESPONSE_STATUS_OK: u8 = 0;
