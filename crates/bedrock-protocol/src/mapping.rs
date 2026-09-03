#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IdShift {
    pub insert_at: u32,
    pub count: u32,
}

impl IdShift {
    pub const fn inserted(count: u32, at: u32) -> IdShift {
        IdShift {
            insert_at: at,
            count,
        }
    }

    pub fn up(&self, id: u32) -> u32 {
        if id >= self.insert_at {
            id + self.count
        } else {
            id
        }
    }

    pub fn down(&self, id: u32) -> u32 {
        if id >= self.insert_at + self.count {
            id - self.count
        } else if id >= self.insert_at {
            self.insert_at
        } else {
            id
        }
    }

    pub fn cap(&self, id: u32) -> u32 {
        if id >= self.insert_at {
            self.insert_at
        } else {
            id
        }
    }
}

#[derive(Debug, Clone)]
pub struct MappingData {
    pub sound: IdShift,
    pub actor_data_sound_key: u32,
    pub actor_event: Option<IdShift>,
    pub note_instrument: Option<IdShift>,
    pub dropped_actor_data_keys: &'static [u32],
}

impl MappingData {
    pub const fn new(sound: IdShift, actor_data_sound_key: u32) -> MappingData {
        MappingData {
            sound,
            actor_data_sound_key,
            actor_event: None,
            note_instrument: None,
            dropped_actor_data_keys: &[],
        }
    }

    pub const fn with_actor_event(mut self, shift: IdShift) -> MappingData {
        self.actor_event = Some(shift);
        self
    }

    pub const fn with_note_instrument(mut self, shift: IdShift) -> MappingData {
        self.note_instrument = Some(shift);
        self
    }

    pub const fn with_dropped_actor_data_keys(mut self, keys: &'static [u32]) -> MappingData {
        self.dropped_actor_data_keys = keys;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SOUND: IdShift = IdShift::inserted(12, 566);

    #[test]
    fn values_below_the_insertion_are_untouched() {
        for id in [0u32, 1, 100, 565] {
            assert_eq!(SOUND.up(id), id);
            assert_eq!(SOUND.down(id), id);
        }
    }

    #[test]
    fn inserted_values_collapse_onto_the_sentinel() {
        for id in 566..578 {
            assert_eq!(SOUND.down(id), 566, "id {id} should map to Undefined");
        }
    }

    #[test]
    fn values_above_the_run_shift_by_the_count() {
        assert_eq!(SOUND.down(578), 566);
        assert_eq!(SOUND.down(579), 567);
        assert_eq!(SOUND.up(566), 578);
        assert_eq!(SOUND.up(567), 579);
    }

    #[test]
    fn every_old_value_survives_a_round_trip() {
        for old in 0..1200u32 {
            assert_eq!(
                SOUND.down(SOUND.up(old)),
                old,
                "old id {old} did not survive"
            );
        }
    }

    #[test]
    fn up_never_lands_inside_the_inserted_run() {
        for old in 0..1200u32 {
            let new = SOUND.up(old);
            assert!(
                !(566..578).contains(&new),
                "old {old} mapped onto inserted id {new}"
            );
        }
    }

    #[test]
    fn chained_shifts_compose() {
        let a = IdShift::inserted(12, 566);
        let b = IdShift::inserted(19, 578);
        for old in 0..1200u32 {
            let up = b.up(a.up(old));
            assert_eq!(a.down(b.down(up)), old);
        }
    }

    #[test]
    fn cap_is_lossier_than_down() {
        let shift = IdShift::inserted(4, 16);
        assert_eq!(shift.down(20), 16);
        assert_eq!(shift.cap(20), 16);
        assert_eq!(shift.down(21), 17);
        assert_eq!(shift.cap(21), 16);
    }
}
