use super::model::{Entry, Identity};
use super::{ACTION_CHANGE, ACTION_REMOVE, NAME_REMOVE, VARIANT_REMOVE};
use bedrock_codec::prelude::*;

pub(super) fn write_score_v2168(w: &mut Writer, entries: &[Entry]) {
    w.write_count(entries.len());
    for entry in entries {
        match &entry.identity {
            None => {
                w.write_uvarint(VARIANT_REMOVE);
                w.write_string(NAME_REMOVE);
                w.write_varint64(entry.scoreboard_id);
                match &entry.objective {
                    Some(objective) => {
                        w.write_bool(true);
                        w.write_string(objective);
                    }
                    None => w.write_bool(false),
                }
            }
            Some(identity) => {
                w.write_uvarint(identity.variant());
                w.write_string(identity.name());
                w.write_varint64(entry.scoreboard_id);
                w.write_string(entry.objective.as_deref().unwrap_or_default());
                w.write_i32_le(entry.score);
                match identity {
                    Identity::Player(id) | Identity::Entity(id) => w.write_varint64(*id),
                    Identity::FakePlayer(name) => w.write_string(name),
                }
            }
        }
    }
}

pub(super) fn write_score_v1001(w: &mut Writer, entries: &[Entry]) -> bool {
    let changes = entries.iter().filter(|e| e.identity.is_some()).count();
    if changes != 0 && changes != entries.len() {
        return false;
    }
    if entries.iter().any(|e| e.objective.is_none()) {
        return false;
    }
    let action = if changes == 0 && !entries.is_empty() {
        ACTION_REMOVE
    } else {
        ACTION_CHANGE
    };
    w.write_u8(action);
    w.write_count(entries.len());
    for entry in entries {
        w.write_varint64(entry.scoreboard_id);
        w.write_string(entry.objective.as_deref().unwrap_or_default());
        w.write_i32_le(entry.score);
        if let Some(identity) = &entry.identity {
            w.write_u8(identity.legacy_type());
            match identity {
                Identity::Player(id) | Identity::Entity(id) => w.write_varint64(*id),
                Identity::FakePlayer(name) => w.write_string(name),
            }
        }
    }
    true
}
