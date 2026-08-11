use bedrock_codec::prelude::*;

use crate::connection::ConnState;
use crate::mapping::IdShift;
use crate::packet_ids::ids;
use crate::rewriter::SoundRewriter;
use crate::translator::Translator;

const SOUND: IdShift = IdShift::inserted(2, 597);
const NOTE_INSTRUMENT: IdShift = IdShift::inserted(4, 16);
const HEARTBEAT_KEY: u32 = 126;

fn spawn_position_to_signed(w: &mut PacketWrapper, _: &mut ConnState) -> Result<()> {
    w.passthrough::<VarInt>()?;
    w.map::<NetworkBlockPos, BlockPos>()?;
    w.passthrough::<VarInt>()?;
    w.map::<NetworkBlockPos, BlockPos>()?;
    Ok(())
}

fn spawn_position_to_unsigned(w: &mut PacketWrapper, _: &mut ConnState) -> Result<()> {
    w.passthrough::<VarInt>()?;
    w.map::<BlockPos, NetworkBlockPos>()?;
    w.passthrough::<VarInt>()?;
    w.map::<BlockPos, NetworkBlockPos>()?;
    Ok(())
}

fn tile_event_remap(w: &mut PacketWrapper, shift_up: bool) -> Result<()> {
    w.map::<NetworkBlockPos, BlockPos>()?;
    let event_type = w.passthrough::<VarInt>()?;
    const NOTE_BLOCK_EVENT: i32 = 1;
    if event_type == NOTE_BLOCK_EVENT {
        let instrument = w.read::<VarInt>()? as u32;
        let remapped = if shift_up {
            NOTE_INSTRUMENT.up(instrument)
        } else {
            NOTE_INSTRUMENT.down(instrument)
        };
        w.write::<VarInt>(remapped as i32);
    }
    Ok(())
}

pub fn downgrade() -> Translator {
    let step = Translator::new("v944->v924", 924, 944)
        .clientbound(ids::SET_SPAWN_POSITION, spawn_position_to_signed)
        .clientbound(ids::TILE_EVENT, |w, _| tile_event_remap(w, true))
        .serverbound(ids::SET_SPAWN_POSITION, spawn_position_to_unsigned);
    SoundRewriter::new(SOUND,  true, HEARTBEAT_KEY).register(step)
}

pub fn upgrade() -> Translator {
    let step = Translator::new("v924->v944", 944, 924)
        .clientbound(ids::SET_SPAWN_POSITION, spawn_position_to_unsigned)
        .clientbound(ids::TILE_EVENT, |w, _| tile_event_remap(w, false))
        .serverbound(ids::SET_SPAWN_POSITION, spawn_position_to_signed);
    SoundRewriter::new(SOUND,  false, HEARTBEAT_KEY).register(step)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn negative_y_survives_the_encoding_change() {
        let mut w = Writer::new();
        BlockPos::write(&mut w, &(10, -64, -3));
        let signed = w.into_vec();

        let mut wrapper = PacketWrapper::new(&signed);
        wrapper.map::<BlockPos, NetworkBlockPos>().unwrap();
        let unsigned = wrapper.finish();

        let mut back = PacketWrapper::new(&unsigned);
        back.map::<NetworkBlockPos, BlockPos>().unwrap();
        assert_eq!(back.finish(), signed);
    }

    #[test]
    fn note_instrument_shift_skips_non_note_events() {
        let mut w = Writer::new();
        NetworkBlockPos::write(&mut w, &(1, 2, 3));
        w.write_varint(2);
        w.write_varint(999);
        let input = w.into_vec();

        let mut wrapper = PacketWrapper::new(&input);
        tile_event_remap(&mut wrapper, true).unwrap();
        let out = wrapper.finish();

        let mut check = PacketWrapper::new(&out);
        check.map::<BlockPos, NetworkBlockPos>().unwrap();
        assert_eq!(check.finish(), input);
    }
}
