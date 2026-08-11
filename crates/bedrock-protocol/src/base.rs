use bedrock_codec::prelude::*;

use crate::connection::ConnState;
use crate::packet_ids::ids;
use crate::translator::Translator;
use crate::versions;

fn detect_client_protocol(w: &mut PacketWrapper, state: &mut ConnState) -> Result<()> {
    let client_protocol = w.read::<IntBe>()?;
    w.write::<IntBe>(state.server_protocol as i32);

    if state.client_protocol != client_protocol as u32 {
        state.client_protocol = client_protocol as u32;
        state.notices.push(format!(
            "client connected as {}, server speaks {}",
            versions::describe(state.client_protocol),
            versions::describe(state.server_protocol),
        ));
        if !state.is_supported() {
            state.notices.push(format!(
                "{} cannot be translated{}",
                versions::describe(state.client_protocol),
                match versions::nearest_translatable_below(state.client_protocol) {
                    Some(near) => format!("; closest supported is {}", versions::describe(near)),
                    None => String::new(),
                }
            ));
        }
    }
    Ok(())
}

fn rewrite_login(w: &mut PacketWrapper, state: &mut ConnState) -> Result<()> {
    let client_protocol = w.read::<IntBe>()?;
    w.write::<IntBe>(state.server_protocol as i32);
    if state.client_protocol == 0 {
        state.client_protocol = client_protocol as u32;
    }
    Ok(())
}

fn log_packet_violation(w: &mut PacketWrapper, state: &mut ConnState) -> Result<()> {
    let violation_type = w.passthrough::<VarInt>()?;
    let severity = w.passthrough::<VarInt>()?;
    let offending = w.passthrough::<VarInt>()?;
    let context = w.passthrough::<Str>()?;
    state.notices.push(format!(
        "packet violation reported for {}: type={violation_type} severity={severity} context={context:?}",
        crate::packet_ids::label(offending.clamp(0, u16::MAX as i32) as u16),
    ));
    Ok(())
}

pub fn create(server_protocol: u32) -> Translator {
    Translator::new("base", server_protocol, 0)
        .serverbound(ids::REQUEST_NETWORK_SETTINGS, detect_client_protocol)
        .serverbound(ids::LOGIN, rewrite_login)
        .clientbound(ids::PACKET_VIOLATION_WARNING, log_packet_violation)
        .serverbound(ids::PACKET_VIOLATION_WARNING, log_packet_violation)
}
