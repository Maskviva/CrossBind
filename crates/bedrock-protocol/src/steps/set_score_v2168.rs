use std::sync::OnceLock;

use bedrock_codec::prelude::*;

use crate::connection::ConnState;
use crate::direction::Direction;
use crate::packet_ids::ids;

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

struct Entry {
    scoreboard_id: i64,
    objective: Option<String>,
    score: i32,
    identity: Option<Identity>,
}

enum Identity {
    Player(i64),
    Entity(i64),
    FakePlayer(String),
}

impl Identity {
    fn variant(&self) -> u32 {
        match self {
            Identity::Player(_) => VARIANT_CHANGE_PLAYER,
            Identity::Entity(_) => VARIANT_CHANGE_ENTITY,
            Identity::FakePlayer(_) => VARIANT_CHANGE_FAKE_PLAYER,
        }
    }

    fn name(&self) -> &'static str {
        match self {
            Identity::Player(_) => NAME_CHANGE_PLAYER,
            Identity::Entity(_) => NAME_CHANGE_ENTITY,
            Identity::FakePlayer(_) => NAME_CHANGE_FAKE_PLAYER,
        }
    }

    fn legacy_type(&self) -> u8 {
        match self {
            Identity::Player(_) => IDENTITY_PLAYER,
            Identity::Entity(_) => IDENTITY_ENTITY,
            Identity::FakePlayer(_) => IDENTITY_FAKE_PLAYER,
        }
    }
}

struct IdentityEntry {
    scoreboard_id: i64,
    player_id: Option<i64>,
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
        return Err(Error::Invalid("trailing bytes after set scoreboard identity"));
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

fn read_score_v1001(r: &mut Reader<'_>) -> Result<Vec<Entry>> {
    let action = r.read_u8()?;
    if action != ACTION_CHANGE && action != ACTION_REMOVE {
        return Err(Error::BadDiscriminant {
            what: "set score action type",
            value: i64::from(action),
        });
    }
    let count = r.read_count()?;
    let mut entries = Vec::with_capacity(count);
    for _ in 0..count {
        let scoreboard_id = r.read_varint64()?;
        let objective = Str::read(r)?;
        let score = r.read_i32_le()?;
        let identity = if action == ACTION_REMOVE {
            None
        } else {
            Some(read_identity_v1001_body(r)?)
        };
        entries.push(Entry {
            scoreboard_id,
            objective: Some(objective),
            score,
            identity,
        });
    }
    Ok(entries)
}

fn read_identity_v1001_body(r: &mut Reader<'_>) -> Result<Identity> {
    let kind = r.read_u8()?;
    match kind {
        IDENTITY_PLAYER => Ok(Identity::Player(r.read_varint64()?)),
        IDENTITY_ENTITY => Ok(Identity::Entity(r.read_varint64()?)),
        IDENTITY_FAKE_PLAYER => Ok(Identity::FakePlayer(Str::read(r)?)),
        other => Err(Error::BadDiscriminant {
            what: "scoreboard identity type",
            value: i64::from(other),
        }),
    }
}

fn read_score_v2168(r: &mut Reader<'_>) -> Result<Vec<Entry>> {
    let count = r.read_count()?;
    let mut entries = Vec::with_capacity(count);
    for _ in 0..count {
        let variant = r.read_uvarint()?;
        let name = Str::read(r)?;
        if !name.eq_ignore_ascii_case(expected_name(variant)?) {
            return Err(Error::Invalid("set score entry action name disagrees with its variant"));
        }
        let scoreboard_id = r.read_varint64()?;
        if variant == VARIANT_REMOVE {
            let objective = if r.read_bool()? {
                Some(Str::read(r)?)
            } else {
                None
            };
            entries.push(Entry {
                scoreboard_id,
                objective,
                score: 0,
                identity: None,
            });
            continue;
        }
        let objective = Str::read(r)?;
        let score = r.read_i32_le()?;
        let identity = match variant {
            VARIANT_CHANGE_PLAYER => Identity::Player(r.read_varint64()?),
            VARIANT_CHANGE_ENTITY => Identity::Entity(r.read_varint64()?),
            _ => Identity::FakePlayer(Str::read(r)?),
        };
        entries.push(Entry {
            scoreboard_id,
            objective: Some(objective),
            score,
            identity: Some(identity),
        });
    }
    Ok(entries)
}

fn expected_name(variant: u32) -> Result<&'static str> {
    match variant {
        VARIANT_REMOVE => Ok(NAME_REMOVE),
        VARIANT_CHANGE_PLAYER => Ok(NAME_CHANGE_PLAYER),
        VARIANT_CHANGE_ENTITY => Ok(NAME_CHANGE_ENTITY),
        VARIANT_CHANGE_FAKE_PLAYER => Ok(NAME_CHANGE_FAKE_PLAYER),
        other => Err(Error::BadDiscriminant {
            what: "set score entry variant",
            value: i64::from(other),
        }),
    }
}

fn write_score_v2168(w: &mut Writer, entries: &[Entry]) {
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

fn write_score_v1001(w: &mut Writer, entries: &[Entry]) -> bool {
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

fn read_identity_v1001(r: &mut Reader<'_>) -> Result<(bool, Vec<IdentityEntry>)> {
    let action = r.read_u8()?;
    if action != ACTION_CHANGE && action != ACTION_REMOVE {
        return Err(Error::BadDiscriminant {
            what: "scoreboard identity action type",
            value: i64::from(action),
        });
    }
    let update = action == ACTION_CHANGE;
    let count = r.read_count()?;
    let mut entries = Vec::with_capacity(count);
    for _ in 0..count {
        let scoreboard_id = r.read_varint64()?;
        let player_id = if update {
            Some(r.read_varint64()?)
        } else {
            None
        };
        entries.push(IdentityEntry {
            scoreboard_id,
            player_id,
        });
    }
    Ok((update, entries))
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

#[cfg(test)]
mod tests {
    use super::*;

    const GOLDEN_V1001_CHANGE: &[u8] = &[
        0x00, 0x03, 0x02, 0x03, 0x6F, 0x62, 0x6A, 0x05, 0x00, 0x00, 0x00, 0x01, 0x12, 0x04, 0x03,
        0x6F, 0x62, 0x6A, 0x06, 0x00, 0x00, 0x00, 0x02, 0x16, 0x06, 0x03, 0x6F, 0x62, 0x6A, 0x07,
        0x00, 0x00, 0x00, 0x03, 0x04, 0x66, 0x61, 0x6B, 0x65,
    ];

    const GOLDEN_V1001_REMOVE: &[u8] = &[
        0x01, 0x01, 0x02, 0x02, 0x72, 0x6D, 0x00, 0x00, 0x00, 0x00,
    ];

    const GOLDEN_V2168: &[u8] = &[
        0x04, 0x00, 0x06, 0x52, 0x65, 0x6D, 0x6F, 0x76, 0x65, 0x02, 0x01, 0x02, 0x72, 0x6D, 0x01,
        0x0C, 0x43, 0x68, 0x61, 0x6E, 0x67, 0x65, 0x50, 0x6C, 0x61, 0x79, 0x65, 0x72, 0x04, 0x03,
        0x6F, 0x62, 0x6A, 0x05, 0x00, 0x00, 0x00, 0x12, 0x02, 0x0C, 0x43, 0x68, 0x61, 0x6E, 0x67,
        0x65, 0x45, 0x6E, 0x74, 0x69, 0x74, 0x79, 0x06, 0x03, 0x6F, 0x62, 0x6A, 0x06, 0x00, 0x00,
        0x00, 0x16, 0x03, 0x10, 0x43, 0x68, 0x61, 0x6E, 0x67, 0x65, 0x46, 0x61, 0x6B, 0x65, 0x50,
        0x6C, 0x61, 0x79, 0x65, 0x72, 0x08, 0x03, 0x6F, 0x62, 0x6A, 0x07, 0x00, 0x00, 0x00, 0x04,
        0x66, 0x61, 0x6B, 0x65,
    ];

    const GOLDEN_IDENTITY_V1001_UPDATE: &[u8] = &[0x00, 0x02, 0x02, 0x0E, 0x04, 0x10];
    const GOLDEN_IDENTITY_V1001_REMOVE: &[u8] = &[0x01, 0x01, 0x02];
    const GOLDEN_IDENTITY_V2168_UPDATE: &[u8] = &[0x00, 0x02, 0x02, 0x01, 0x0E, 0x04, 0x01, 0x10];
    const GOLDEN_IDENTITY_V2168_REMOVE: &[u8] = &[0x01, 0x01, 0x02, 0x00];

    fn run(input: &[u8], to_v2168: bool, score: bool) -> Option<Vec<u8>> {
        let mut w = PacketWrapper::new(input);
        let body: fn(&mut PacketWrapper, bool) -> Result<bool> =
            if score { score_body } else { identity_body };
        match body(&mut w, to_v2168) {
            Ok(true) => Some(w.finish()),
            Ok(false) => None,
            Err(err) => panic!("failed to translate: {err}"),
        }
    }

    #[test]
    fn the_v2168_writer_reproduces_the_reference_bytes() {
        let entries = read_score_v2168(&mut Reader::new(GOLDEN_V2168)).expect("decode golden");
        assert_eq!(entries.len(), 4);
        let mut out = Writer::new();
        write_score_v2168(&mut out, &entries);
        assert_eq!(out.into_vec(), GOLDEN_V2168);
    }

    #[test]
    fn the_v2168_reader_pulls_the_reference_fields_out() {
        let entries = read_score_v2168(&mut Reader::new(GOLDEN_V2168)).expect("decode golden");
        assert_eq!(entries[0].scoreboard_id, 1);
        assert_eq!(entries[0].objective.as_deref(), Some("rm"));
        assert!(entries[0].identity.is_none());
        assert!(matches!(entries[1].identity, Some(Identity::Player(9))));
        assert_eq!(entries[1].score, 5);
        assert!(matches!(entries[2].identity, Some(Identity::Entity(11))));
        match entries[3].identity.as_ref().expect("fake player") {
            Identity::FakePlayer(name) => assert_eq!(name, "fake"),
            _ => panic!("entry 3 is not a fake player"),
        }
        assert_eq!(entries[3].scoreboard_id, 4);
        assert_eq!(entries[3].score, 7);
    }

    #[test]
    fn a_v1001_change_packet_upgrades_to_the_reference_shape() {
        let up = run(GOLDEN_V1001_CHANGE, true, true).expect("upgrade");
        let mut expected = Writer::new();
        expected.write_count(3);
        expected.write_uvarint(VARIANT_CHANGE_PLAYER);
        expected.write_string(NAME_CHANGE_PLAYER);
        expected.write_varint64(1);
        expected.write_string("obj");
        expected.write_i32_le(5);
        expected.write_varint64(9);
        expected.write_uvarint(VARIANT_CHANGE_ENTITY);
        expected.write_string(NAME_CHANGE_ENTITY);
        expected.write_varint64(2);
        expected.write_string("obj");
        expected.write_i32_le(6);
        expected.write_varint64(11);
        expected.write_uvarint(VARIANT_CHANGE_FAKE_PLAYER);
        expected.write_string(NAME_CHANGE_FAKE_PLAYER);
        expected.write_varint64(3);
        expected.write_string("obj");
        expected.write_i32_le(7);
        expected.write_string("fake");
        assert_eq!(up, expected.into_vec());
        assert_eq!(run(&up, false, true).expect("downgrade"), GOLDEN_V1001_CHANGE);
    }

    #[test]
    fn a_v1001_remove_packet_round_trips() {
        let up = run(GOLDEN_V1001_REMOVE, true, true).expect("upgrade");
        let mut expected = Writer::new();
        expected.write_count(1);
        expected.write_uvarint(VARIANT_REMOVE);
        expected.write_string(NAME_REMOVE);
        expected.write_varint64(1);
        expected.write_bool(true);
        expected.write_string("rm");
        assert_eq!(up, expected.into_vec());
        assert_eq!(run(&up, false, true).expect("downgrade"), GOLDEN_V1001_REMOVE);
    }

    #[test]
    fn a_real_sidebar_entry_upgrades_byte_for_byte() {
        let mut w = Writer::new();
        w.write_u8(ACTION_CHANGE);
        w.write_count(1);
        w.write_varint64(1);
        w.write_string("rcc_bar");
        w.write_i32_le(12);
        w.write_u8(IDENTITY_FAKE_PLAYER);
        w.write_string("§f§l个人信息");
        let captured = w.into_vec();

        let up = run(&captured, true, true).expect("upgrade");
        let mut expected = Writer::new();
        expected.write_count(1);
        expected.write_uvarint(VARIANT_CHANGE_FAKE_PLAYER);
        expected.write_string(NAME_CHANGE_FAKE_PLAYER);
        expected.write_varint64(1);
        expected.write_string("rcc_bar");
        expected.write_i32_le(12);
        expected.write_string("§f§l个人信息");
        assert_eq!(up, expected.into_vec());
        assert_eq!(run(&up, false, true).expect("downgrade"), captured);
    }

    #[test]
    fn neither_body_survives_the_other_version_reader() {
        let mut w = PacketWrapper::new(GOLDEN_V1001_CHANGE);
        assert!(
            score_body(&mut w, false).is_err(),
            "a v1001 body read as v2168 gives count 0 and a tail of leftover bytes"
        );
        let mut w = PacketWrapper::new(GOLDEN_V2168);
        assert!(score_body(&mut w, true).is_err());
    }

    #[test]
    fn a_mixed_v2168_list_cannot_go_down() {
        assert!(run(GOLDEN_V2168, false, true).is_none());
    }

    #[test]
    fn a_removal_without_an_objective_name_cannot_go_down() {
        let mut w = Writer::new();
        w.write_count(1);
        w.write_uvarint(VARIANT_REMOVE);
        w.write_string(NAME_REMOVE);
        w.write_varint64(1);
        w.write_bool(false);
        assert!(run(&w.into_vec(), false, true).is_none());
    }

    #[test]
    fn an_action_name_that_disagrees_with_its_variant_is_refused() {
        let mut w = Writer::new();
        w.write_count(1);
        w.write_uvarint(VARIANT_CHANGE_FAKE_PLAYER);
        w.write_string(NAME_CHANGE_PLAYER);
        w.write_varint64(1);
        assert!(read_score_v2168(&mut Reader::new(&w.into_vec())).is_err());
    }

    #[test]
    fn the_action_name_is_matched_case_insensitively() {
        let mut w = Writer::new();
        w.write_count(1);
        w.write_uvarint(VARIANT_CHANGE_FAKE_PLAYER);
        w.write_string("changefakeplayer");
        w.write_varint64(1);
        w.write_string("obj");
        w.write_i32_le(3);
        w.write_string("fake");
        let entries = read_score_v2168(&mut Reader::new(&w.into_vec())).expect("lower case name");
        assert_eq!(entries.len(), 1);
    }

    #[test]
    fn scoreboard_identities_match_the_reference_bytes_both_ways() {
        assert_eq!(
            run(GOLDEN_IDENTITY_V1001_UPDATE, true, false).expect("update up"),
            GOLDEN_IDENTITY_V2168_UPDATE
        );
        assert_eq!(
            run(GOLDEN_IDENTITY_V2168_UPDATE, false, false).expect("update down"),
            GOLDEN_IDENTITY_V1001_UPDATE
        );
        assert_eq!(
            run(GOLDEN_IDENTITY_V1001_REMOVE, true, false).expect("remove up"),
            GOLDEN_IDENTITY_V2168_REMOVE
        );
        assert_eq!(
            run(GOLDEN_IDENTITY_V2168_REMOVE, false, false).expect("remove down"),
            GOLDEN_IDENTITY_V1001_REMOVE
        );
    }

    #[test]
    fn only_an_explicit_off_stops_the_translation() {
        assert!(parse_enabled(None));
        assert!(parse_enabled(Some("")));
        assert!(parse_enabled(Some("on")));
        assert!(parse_enabled(Some("probe")));
        assert!(!parse_enabled(Some("off")));
        assert!(!parse_enabled(Some(" OFF ")));
        assert!(!parse_enabled(Some("drop")));
    }
}
