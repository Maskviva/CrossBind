use bedrock_codec::prelude::*;

use crate::direction::Direction;
use crate::mapping::IdShift;
use crate::packet_ids::ids;
use crate::translator::Translator;

#[derive(Clone, Copy)]
pub struct ActorEventRewriter {
    shift: IdShift,
    client_is_newer: bool,
}

impl ActorEventRewriter {
    pub fn new(shift: IdShift, client_is_newer: bool) -> ActorEventRewriter {
        ActorEventRewriter {
            shift,
            client_is_newer,
        }
    }

    fn going_to_newer(&self, direction: Direction) -> bool {
        match direction {
            Direction::Clientbound => self.client_is_newer,
            Direction::Serverbound => !self.client_is_newer,
        }
    }

    fn actor_event(&self, w: &mut PacketWrapper, direction: Direction) -> Result<()> {
        w.passthrough::<UVarInt64>()?;
        let event_id = w.read::<Byte>()? as u32;

        if self.going_to_newer(direction) {
            w.write::<Byte>(self.shift.up(event_id) as u8);
            return Ok(());
        }

        let inserted_range = self.shift.insert_at..self.shift.insert_at + self.shift.count;
        if inserted_range.contains(&event_id) {
            w.cancel();
            return Ok(());
        }
        w.write::<Byte>(self.shift.down(event_id) as u8);
        Ok(())
    }

    pub fn register(self, step: Translator) -> Translator {
        step.clientbound(ids::ACTOR_EVENT, move |w, _| {
            self.actor_event(w, Direction::Clientbound)
        })
    }
}
