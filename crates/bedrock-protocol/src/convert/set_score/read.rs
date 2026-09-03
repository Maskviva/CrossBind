use super::model::{Entry, Identity, IdentityEntry};
use super::{
    ACTION_CHANGE, ACTION_REMOVE, IDENTITY_ENTITY, IDENTITY_FAKE_PLAYER, IDENTITY_PLAYER,
    NAME_CHANGE_ENTITY, NAME_CHANGE_FAKE_PLAYER, NAME_CHANGE_PLAYER, NAME_REMOVE,
    VARIANT_CHANGE_ENTITY, VARIANT_CHANGE_FAKE_PLAYER, VARIANT_CHANGE_PLAYER, VARIANT_REMOVE,
};
use bedrock_codec::prelude::*;

pub(super) fn read_score_v1001(r: &mut Reader<'_>) -> Result<Vec<Entry>> {
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

pub(super) fn read_identity_v1001(r: &mut Reader<'_>) -> Result<(bool, Vec<IdentityEntry>)> {
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

pub(super) fn read_score_v2168(r: &mut Reader<'_>) -> Result<Vec<Entry>> {
    let count = r.read_count()?;
    let mut entries = Vec::with_capacity(count);
    for _ in 0..count {
        let variant = r.read_uvarint()?;
        let name = Str::read(r)?;
        if !name.eq_ignore_ascii_case(expected_name(variant)?) {
            return Err(Error::Invalid(
                "set score entry action name disagrees with its variant",
            ));
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
