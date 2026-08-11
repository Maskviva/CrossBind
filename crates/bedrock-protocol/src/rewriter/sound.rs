use bedrock_codec::prelude::*;

use crate::direction::Direction;
use crate::mapping::IdShift;
use crate::packet_ids::ids;
use crate::translator::Translator;

#[derive(Clone, Copy)]
pub struct SoundRewriter {
    shift: IdShift,
    client_is_newer: bool,
    sound_key: u32,
    dropped_keys: &'static [u32],
    skip_level_sound_event: bool,
    item_encoding_changed: bool,
}

impl SoundRewriter {
    pub fn new(shift: IdShift, client_is_newer: bool, sound_key: u32) -> SoundRewriter {
        SoundRewriter {
            shift,
            client_is_newer,
            sound_key,
            dropped_keys: &[],
            skip_level_sound_event: false,
            item_encoding_changed: false,
        }
    }

    pub fn without_level_sound_event(mut self) -> SoundRewriter {
        self.skip_level_sound_event = true;
        self
    }

    pub fn with_item_encoding_change(mut self) -> SoundRewriter {
        self.item_encoding_changed = true;
        self
    }

    pub fn with_dropped_keys(mut self, keys: &'static [u32]) -> SoundRewriter {
        self.dropped_keys = keys;
        self
    }

    fn going_to_newer(&self, direction: Direction) -> bool {
        match direction {
            Direction::Clientbound => self.client_is_newer,
            Direction::Serverbound => !self.client_is_newer,
        }
    }

    fn remap(&self, direction: Direction, id: u32) -> u32 {
        if self.going_to_newer(direction) {
            self.shift.up(id)
        } else {
            self.shift.down(id)
        }
    }

    fn item(&self, w: &mut PacketWrapper, direction: Direction) -> Result<()> {
        if !self.item_encoding_changed {
            w.passthrough::<ItemInstance>()?;
            return Ok(());
        }
        if self.going_to_newer(direction) {
            w.map::<ItemInstance, ItemInstanceV975>()?;
        } else {
            w.map::<ItemInstanceV975, ItemInstance>()?;
        }
        Ok(())
    }

    fn should_drop_keys(&self, direction: Direction) -> bool {
        if self.dropped_keys.is_empty() {
            return false;
        }
        !self.going_to_newer(direction)
    }

    fn rewrite_actor_data(&self, w: &mut PacketWrapper, direction: Direction) -> Result<()> {
        let mut entries = w.read::<ActorDataList>()?;
        if self.should_drop_keys(direction) {
            entries.retain(|entry| !self.dropped_keys.contains(&entry.key));
        }
        for entry in &mut entries {
            if entry.key != self.sound_key {
                continue;
            }
            if let Some(current) = entry.value.as_int() {
                let remapped = self.remap(direction, current as u32);
                entry.value.set_int(remapped as i64);
            }
        }
        w.write::<ActorDataList>(entries);
        Ok(())
    }

    fn level_sound_event(&self, w: &mut PacketWrapper, direction: Direction) -> Result<()> {
        let event_id = w.read::<UVarInt>()?;
        w.write::<UVarInt>(self.remap(direction, event_id));
        Ok(())
    }

    fn set_actor_data(&self, w: &mut PacketWrapper, direction: Direction) -> Result<()> {
        w.passthrough::<UVarInt64>()?;
        self.rewrite_actor_data(w, direction)
    }

    fn add_actor(&self, w: &mut PacketWrapper, direction: Direction) -> Result<()> {
        w.passthrough::<VarInt64>()?;
        w.passthrough::<UVarInt64>()?;
        w.passthrough::<Str>()?;
        w.passthrough::<Vec3>()?;
        w.passthrough::<Vec3>()?;
        w.passthrough::<Vec2>()?;
        w.passthrough::<FloatLe>()?;
        w.passthrough::<FloatLe>()?;
        w.passthrough_each(|w| {
            w.passthrough::<Str>()?;
            w.passthrough::<FloatLe>()?;
            w.passthrough::<FloatLe>()?;
            w.passthrough::<FloatLe>()?;
            Ok(())
        })?;
        self.rewrite_actor_data(w, direction)
    }

    fn add_item_actor(&self, w: &mut PacketWrapper, direction: Direction) -> Result<()> {
        w.passthrough::<VarInt64>()?;
        w.passthrough::<UVarInt64>()?;
        self.item(w, direction)?;
        w.passthrough::<Vec3>()?;
        w.passthrough::<Vec3>()?;
        self.rewrite_actor_data(w, direction)
    }

    fn add_player(&self, w: &mut PacketWrapper, direction: Direction) -> Result<()> {
        w.passthrough::<Uuid>()?;
        w.passthrough::<Str>()?;
        w.passthrough::<UVarInt64>()?;
        w.passthrough::<Str>()?;
        w.passthrough::<Vec3>()?;
        w.passthrough::<Vec3>()?;
        w.passthrough::<Vec2>()?;
        w.passthrough::<FloatLe>()?;
        self.item(w, direction)?;
        w.passthrough::<VarInt>()?;
        self.rewrite_actor_data(w, direction)
    }

    pub fn register(self, mut step: Translator) -> Translator {
        if !self.skip_level_sound_event {
            for direction in [Direction::Clientbound, Direction::Serverbound] {
                step = step.register(direction, ids::LEVEL_SOUND_EVENT, move |w, _| {
                    self.level_sound_event(w, direction)
                });
            }
        }
        step = step
            .clientbound(ids::SET_ACTOR_DATA, move |w, _| {
                self.set_actor_data(w, Direction::Clientbound)
            })
            .clientbound(ids::ADD_ACTOR, move |w, _| {
                self.add_actor(w, Direction::Clientbound)
            })
            .clientbound(ids::ADD_ITEM_ACTOR, move |w, _| {
                self.add_item_actor(w, Direction::Clientbound)
            })
            .clientbound(ids::ADD_PLAYER, move |w, _| {
                self.add_player(w, Direction::Clientbound)
            });
        step
    }
}
