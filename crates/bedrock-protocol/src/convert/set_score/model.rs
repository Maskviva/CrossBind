use super::{
    IDENTITY_ENTITY, IDENTITY_FAKE_PLAYER, IDENTITY_PLAYER, NAME_CHANGE_ENTITY,
    NAME_CHANGE_FAKE_PLAYER, NAME_CHANGE_PLAYER, VARIANT_CHANGE_ENTITY, VARIANT_CHANGE_FAKE_PLAYER,
    VARIANT_CHANGE_PLAYER,
};

pub(super) struct Entry {
    pub(super) scoreboard_id: i64,
    pub(super) objective: Option<String>,
    pub(super) score: i32,
    pub(super) identity: Option<Identity>,
}

pub(super) enum Identity {
    Player(i64),
    Entity(i64),
    FakePlayer(String),
}

impl Identity {
    pub(super) fn variant(&self) -> u32 {
        match self {
            Identity::Player(_) => VARIANT_CHANGE_PLAYER,
            Identity::Entity(_) => VARIANT_CHANGE_ENTITY,
            Identity::FakePlayer(_) => VARIANT_CHANGE_FAKE_PLAYER,
        }
    }

    pub(super) fn name(&self) -> &'static str {
        match self {
            Identity::Player(_) => NAME_CHANGE_PLAYER,
            Identity::Entity(_) => NAME_CHANGE_ENTITY,
            Identity::FakePlayer(_) => NAME_CHANGE_FAKE_PLAYER,
        }
    }

    pub(super) fn legacy_type(&self) -> u8 {
        match self {
            Identity::Player(_) => IDENTITY_PLAYER,
            Identity::Entity(_) => IDENTITY_ENTITY,
            Identity::FakePlayer(_) => IDENTITY_FAKE_PLAYER,
        }
    }
}

pub(super) struct IdentityEntry {
    pub(super) scoreboard_id: i64,
    pub(super) player_id: Option<i64>,
}
