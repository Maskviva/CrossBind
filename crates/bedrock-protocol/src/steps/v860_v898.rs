use bedrock_codec::prelude::*;

use crate::mapping::IdShift;
use crate::packet_ids::ids;
use crate::rewriter::{ActorEventRewriter, SoundRewriter};
use crate::translator::Translator;

const SOUND: IdShift = IdShift::inserted(12, 566);
const ACTOR_EVENT: IdShift = IdShift::inserted(1, 80);
const HEARTBEAT_KEY: u32 = 126;

fn mob_effect_add_ambient(w: &mut PacketWrapper, _: &mut crate::connection::ConnState) -> Result<()> {
    w.passthrough::<UVarInt64>()?;
    w.passthrough::<Byte>()?;
    w.passthrough::<VarInt>()?;
    w.passthrough::<VarInt>()?;
    w.passthrough::<Bool>()?;
    w.passthrough::<VarInt>()?;
    w.passthrough::<UVarInt64>()?;
    w.write::<Bool>(false);
    Ok(())
}

fn mob_effect_drop_ambient(w: &mut PacketWrapper, _: &mut crate::connection::ConnState) -> Result<()> {
    w.passthrough::<UVarInt64>()?;
    w.passthrough::<Byte>()?;
    w.passthrough::<VarInt>()?;
    w.passthrough::<VarInt>()?;
    w.passthrough::<Bool>()?;
    w.passthrough::<VarInt>()?;
    w.passthrough::<UVarInt64>()?;
    w.read::<Bool>()?;
    Ok(())
}

fn animate_add_data(w: &mut PacketWrapper, _: &mut crate::connection::ConnState) -> Result<()> {
    w.passthrough::<VarInt>()?;
    w.passthrough::<UVarInt64>()?;
    w.write::<FloatLe>(0.0);
    Ok(())
}

fn animate_drop_data(w: &mut PacketWrapper, _: &mut crate::connection::ConnState) -> Result<()> {
    w.passthrough::<VarInt>()?;
    w.passthrough::<UVarInt64>()?;
    w.read::<FloatLe>()?;
    Ok(())
}

fn interact_add_position(w: &mut PacketWrapper, _: &mut crate::connection::ConnState) -> Result<()> {
    use bedrock_codec::types::enums::interact_action as ia;
    let action = w.passthrough::<Byte>()?;
    w.passthrough::<UVarInt64>()?;
    if action == ia::INTERACT_UPDATE || action == ia::STOP_RIDING {
        w.passthrough::<Vec3>()?;
    } else if action == ia::OPEN_INVENTORY {
        w.write::<Vec3>((0.0, 0.0, 0.0));
    }
    Ok(())
}

fn interact_drop_position(w: &mut PacketWrapper, _: &mut crate::connection::ConnState) -> Result<()> {
    use bedrock_codec::types::enums::interact_action as ia;
    let action = w.passthrough::<Byte>()?;
    w.passthrough::<UVarInt64>()?;
    if action == ia::INTERACT_UPDATE || action == ia::STOP_RIDING {
        w.passthrough::<Vec3>()?;
    } else if action == ia::OPEN_INVENTORY {
        w.read::<Vec3>()?;
    }
    Ok(())
}

fn resource_pack_stack_add_name(
    w: &mut PacketWrapper,
    _: &mut crate::connection::ConnState,
) -> Result<()> {
    w.passthrough::<Bool>()?;
    for _ in 0..2 {
        w.passthrough_each(|w| {
            w.passthrough::<Str>()?;
            w.passthrough::<Str>()?;
            w.passthrough::<Str>()?;
            Ok(())
        })?;
    }
    w.passthrough::<Str>()?;
    w.passthrough_each(|w| {
        w.passthrough::<Str>()?;
        w.passthrough::<Bool>()?;
        Ok(())
    })?;
    w.passthrough::<Bool>()?;
    w.write::<Str>(String::new());
    Ok(())
}

fn resource_pack_stack_drop_name(
    w: &mut PacketWrapper,
    _: &mut crate::connection::ConnState,
) -> Result<()> {
    w.passthrough::<Bool>()?;
    for _ in 0..2 {
        w.passthrough_each(|w| {
            w.passthrough::<Str>()?;
            w.passthrough::<Str>()?;
            w.passthrough::<Str>()?;
            Ok(())
        })?;
    }
    w.passthrough::<Str>()?;
    w.passthrough_each(|w| {
        w.passthrough::<Str>()?;
        w.passthrough::<Bool>()?;
        Ok(())
    })?;
    w.passthrough::<Bool>()?;
    w.read::<Str>()?;
    Ok(())
}

fn telemetry_widen_type(w: &mut PacketWrapper, _: &mut crate::connection::ConnState) -> Result<()> {
    w.passthrough::<VarInt64>()?;
    w.map::<SByte, VarInt32FromI8>()?;
    Ok(())
}

fn telemetry_narrow_type(w: &mut PacketWrapper, _: &mut crate::connection::ConnState) -> Result<()> {
    w.passthrough::<VarInt64>()?;
    w.map::<VarInt32FromI8, SByte>()?;
    Ok(())
}

struct VarInt32FromI8;
impl Codec for VarInt32FromI8 {
    type Value = i8;
    fn read(r: &mut Reader<'_>) -> Result<i8> {
        Ok(r.read_varint()? as i8)
    }
    fn write(w: &mut Writer, v: &i8) {
        w.write_varint(*v as i32)
    }
}

pub fn downgrade() -> Translator {
    let step = Translator::new("v898->v860", 860, 898)
        .clientbound(ids::MOB_EFFECT, mob_effect_add_ambient)
        .clientbound(ids::ANIMATE, animate_add_data)
        .clientbound(ids::RESOURCE_PACK_STACK, resource_pack_stack_add_name)
        .clientbound(ids::LEGACY_TELEMETRY_EVENT, telemetry_widen_type)
        .serverbound(ids::MOB_EFFECT, mob_effect_drop_ambient)
        .serverbound(ids::ANIMATE, animate_drop_data)
        .serverbound(ids::INTERACT, interact_drop_position);

    let step = SoundRewriter::new(SOUND,  true, HEARTBEAT_KEY).register(step);
    ActorEventRewriter::new(ACTOR_EVENT, true).register(step)
}

pub fn upgrade() -> Translator {
    let step = Translator::new("v860->v898", 898, 860)
        .clientbound(ids::MOB_EFFECT, mob_effect_drop_ambient)
        .clientbound(ids::ANIMATE, animate_drop_data)
        .clientbound(ids::RESOURCE_PACK_STACK, resource_pack_stack_drop_name)
        .clientbound(ids::LEGACY_TELEMETRY_EVENT, telemetry_narrow_type)
        .serverbound(ids::MOB_EFFECT, mob_effect_add_ambient)
        .serverbound(ids::ANIMATE, animate_add_data)
        .serverbound(ids::INTERACT, interact_add_position);

    let step = SoundRewriter::new(SOUND,  false, HEARTBEAT_KEY).register(step);
    ActorEventRewriter::new(ACTOR_EVENT, false).register(step)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connection::ConnState;
    use crate::direction::Direction;

    fn run(
        handler: fn(&mut PacketWrapper, &mut ConnState) -> Result<()>,
        input: &[u8],
    ) -> Vec<u8> {
        let mut state = ConnState::new(860);
        let mut w = PacketWrapper::new(input);
        handler(&mut w, &mut state).expect("handler failed");
        w.finish()
    }

    #[test]
    fn mob_effect_add_then_drop_is_the_identity() {
        let original = [0x01u8, 0x02, 0x04, 0x06, 0x01, 0x08, 0x0A];
        let widened = run(mob_effect_add_ambient, &original);
        assert_eq!(widened.len(), original.len() + 1);
        assert_eq!(*widened.last().unwrap(), 0);
        assert_eq!(run(mob_effect_drop_ambient, &widened), original.to_vec());
    }

    #[test]
    fn interact_only_touches_open_inventory() {
        let original = [6u8, 0x2A];
        let widened = run(interact_add_position, &original);
        assert_eq!(widened.len(), original.len() + 12);
        assert_eq!(run(interact_drop_position, &widened), original.to_vec());

        let mut with_pos = vec![4u8, 0x2A];
        with_pos.extend_from_slice(&[0u8; 12]);
        assert_eq!(run(interact_add_position, &with_pos), with_pos);
        assert_eq!(run(interact_drop_position, &with_pos), with_pos);

        let plain = [0u8, 0x2A];
        assert_eq!(run(interact_add_position, &plain), plain.to_vec());
    }

    #[test]
    fn animate_gains_and_loses_the_float() {
        let original = [0x02u8, 0x05];
        let widened = run(animate_add_data, &original);
        assert_eq!(widened.len(), original.len() + 4);
        assert_eq!(run(animate_drop_data, &widened), original.to_vec());
    }

    #[test]
    fn telemetry_type_width_round_trips_including_negatives() {
        for raw in [0i8, 1, -1, 127, -128] {
            let original = [0x00u8, raw as u8];
            let widened = run(telemetry_widen_type, &original);
            assert_eq!(run(telemetry_narrow_type, &widened), original.to_vec());
        }
    }

    #[test]
    fn both_directions_register_the_same_packets() {
        let down = downgrade();
        let up = upgrade();
        for id in [
            ids::MOB_EFFECT,
            ids::ANIMATE,
            ids::LEVEL_SOUND_EVENT,
            ids::ACTOR_EVENT,
        ] {
            assert!(down.touches(Direction::Clientbound, id));
            assert!(up.touches(Direction::Clientbound, id));
        }
    }
}
