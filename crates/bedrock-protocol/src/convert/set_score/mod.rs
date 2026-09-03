mod model;
mod read;
#[cfg(test)]
mod tests;
mod write;

use crate::connection::ConnState;
use crate::direction::Direction;
use crate::packet_ids::ids;
use bedrock_codec::prelude::*;
use model::IdentityEntry;
use read::{read_identity_v1001, read_score_v1001, read_score_v2168};
use std::sync::OnceLock;
use write::{write_score_v1001, write_score_v2168};

const ACTION_CHANGE: u8 = 0;

const ACTION_REMOVE: u8 = 1;

const IDENTITY_PLAYER: u8 = 1;

const IDENTITY_ENTITY: u8 = 2;

const IDENTITY_FAKE_PLAYER: u8 = 3;

const VARIANT_REMOVE: u32 = 0;

const VARIANT_CHANGE_PLAYER: u32 = 1;

const VARIANT_CHANGE_ENTITY: u32 = 2;

const VARIANT_CHANGE_FAKE_PLAYER: u32 = 3;

const NAME_REMOVE: &str = "Remove";

const NAME_CHANGE_PLAYER: &str = "ChangePlayer";

const NAME_CHANGE_ENTITY: &str = "ChangeEntity";

const NAME_CHANGE_FAKE_PLAYER: &str = "ChangeFakePlayer";

pub fn enabled() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| parse_enabled(std::env::var("CROSSBIND_SET_SCORE").ok().as_deref()))
}

pub fn parse_enabled(raw: Option<&str>) -> bool {
    match raw {
        None => true,
        Some(v) => !matches!(
            v.trim().to_ascii_lowercase().as_str(),
            "off" | "0" | "false" | "drop" | "cancel"
        ),
    }
}

pub fn describe_layout() -> String {
    if enabled() {
        "SetScore/SetScoreboardIdentity toward v2168: translated (1.26.40 moved each score \
         entry to a name-coded variant; set CROSSBIND_SET_SCORE=off to drop them instead)"
            .to_string()
    } else {
        "SetScore/SetScoreboardIdentity toward v2168: dropped (CROSSBIND_SET_SCORE=off); a \
         sidebar on a 1.26.40 client will show its title and no lines"
            .to_string()
    }
}

pub(crate) fn set_score(
    w: &mut PacketWrapper,
    state: &mut ConnState,
    to_v2168: bool,
) -> Result<bool> {
    dispatch(w, state, to_v2168, ids::SET_SCORE, "SetScore", score_body)
}

pub(crate) fn set_scoreboard_identity(
    w: &mut PacketWrapper,
    state: &mut ConnState,
    to_v2168: bool,
) -> Result<bool> {
    dispatch(
        w,
        state,
        to_v2168,
        ids::SET_SCOREBOARD_IDENTITY,
        "SetScoreboardIdentity",
        identity_body,
    )
}

fn dispatch(
    w: &mut PacketWrapper,
    state: &mut ConnState,
    to_v2168: bool,
    packet_id: u16,
    name: &'static str,
    body: fn(&mut PacketWrapper, bool) -> Result<bool>,
) -> Result<bool> {
    if !enabled() {
        if state.first_failure(Direction::Clientbound, packet_id) {
            state.notices.push(format!(
                "{name}: dropped, CROSSBIND_SET_SCORE=off. A sidebar on a 1.26.40 client shows \
                 its title and no lines while this is off"
            ));
        }
        return Ok(false);
    }
    match body(w, to_v2168) {
        Ok(true) => Ok(true),
        Ok(false) => {
            if state.first_failure(Direction::Clientbound, packet_id) {
                state.notices.push(format!(
                    "{name}: dropped, this packet cannot be expressed in the older form"
                ));
            }
            Ok(false)
        }
        Err(err) => {
            if state.first_failure(Direction::Clientbound, packet_id) {
                state
                    .notices
                    .push(format!("{name}: dropped, cannot decode: {err}"));
            }
            Ok(false)
        }
    }
}

fn score_body(w: &mut PacketWrapper, to_v2168: bool) -> Result<bool> {
    let entries = if to_v2168 {
        read_score_v1001(w.reader())?
    } else {
        read_score_v2168(w.reader())?
    };
    if w.reader().has_remaining() {
        return Err(Error::Invalid("trailing bytes after set score"));
    }
    let mut out = Writer::new();
    if to_v2168 {
        write_score_v2168(&mut out, &entries);
    } else if !write_score_v1001(&mut out, &entries) {
        return Ok(false);
    }
    w.writer().write_bytes(&out.into_vec());
    Ok(true)
}

fn identity_body(w: &mut PacketWrapper, to_v2168: bool) -> Result<bool> {
    let (update, entries) = if to_v2168 {
        read_identity_v1001(w.reader())?
    } else {
        read_identity_v2168(w.reader())?
    };
    if w.reader().has_remaining() {
        return Err(Error::Invalid(
            "trailing bytes after set scoreboard identity",
        ));
    }
    let mut out = Writer::new();
    if to_v2168 {
        write_identity_v2168(&mut out, update, &entries);
    } else if !write_identity_v1001(&mut out, update, &entries) {
        return Ok(false);
    }
    w.writer().write_bytes(&out.into_vec());
    Ok(true)
}

fn read_identity_v2168(r: &mut Reader<'_>) -> Result<(bool, Vec<IdentityEntry>)> {
    let action = r.read_u8()?;
    if action != ACTION_CHANGE && action != ACTION_REMOVE {
        return Err(Error::BadDiscriminant {
            what: "scoreboard identity action type",
            value: i64::from(action),
        });
    }
    let count = r.read_count()?;
    let mut entries = Vec::with_capacity(count);
    for _ in 0..count {
        let scoreboard_id = r.read_varint64()?;
        let player_id = if r.read_bool()? {
            Some(r.read_varint64()?)
        } else {
            None
        };
        entries.push(IdentityEntry {
            scoreboard_id,
            player_id,
        });
    }
    Ok((action == ACTION_CHANGE, entries))
}

fn write_identity_v2168(w: &mut Writer, update: bool, entries: &[IdentityEntry]) {
    w.write_u8(if update { ACTION_CHANGE } else { ACTION_REMOVE });
    w.write_count(entries.len());
    for entry in entries {
        w.write_varint64(entry.scoreboard_id);
        match entry.player_id {
            Some(player_id) => {
                w.write_bool(true);
                w.write_varint64(player_id);
            }
            None => w.write_bool(false),
        }
    }
}

fn write_identity_v1001(w: &mut Writer, update: bool, entries: &[IdentityEntry]) -> bool {
    if update && entries.iter().any(|e| e.player_id.is_none()) {
        return false;
    }
    if !update && entries.iter().any(|e| e.player_id.is_some()) {
        return false;
    }
    w.write_u8(if update { ACTION_CHANGE } else { ACTION_REMOVE });
    w.write_count(entries.len());
    for entry in entries {
        w.write_varint64(entry.scoreboard_id);
        if let Some(player_id) = entry.player_id {
            w.write_varint64(player_id);
        }
    }
    true
}
