pub mod data_item_type {
    pub const BYTE: u32 = 0;
    pub const SHORT: u32 = 1;
    pub const INT: u32 = 2;
    pub const FLOAT: u32 = 3;
    pub const STRING: u32 = 4;
    pub const COMPOUND_TAG: u32 = 5;
    pub const POS: u32 = 6;
    pub const INT64: u32 = 7;
    pub const VEC3: u32 = 8;
}

pub mod actor_data_ids {
    pub const HEARTBEAT_SOUND_EVENT: u32 = 126;
    pub const AIM_ASSIST_PRIORITY_PRESET_ID: u32 = 136;
    pub const AIM_ASSIST_PRIORITY_CATEGORY_ID: u32 = 137;
    pub const AIM_ASSIST_PRIORITY_ACTOR_ID: u32 = 138;
}

pub mod level_sound_event {
    pub const UNDEFINED_V860: u32 = 566;
    pub const UNDEFINED_V898: u32 = 578;
    pub const UNDEFINED_V924: u32 = 597;
    pub const UNDEFINED_V944: u32 = 599;
}

pub mod actor_event {
    pub const KINETIC_DAMAGE_DEALT: u32 = 80;
}

pub mod note_block_instrument {
    pub const TRUMPET: u32 = 16;
}

pub mod interact_action {
    pub const INVALID: u8 = 0;
    pub const STOP_RIDING: u8 = 3;
    pub const INTERACT_UPDATE: u8 = 4;
    pub const NPC_OPEN: u8 = 5;
    pub const OPEN_INVENTORY: u8 = 6;
}

pub mod command_origin_type {
    pub const PLAYER: u32 = 0;
    pub const COMMAND_BLOCK: u32 = 1;
    pub const MINECART_COMMAND_BLOCK: u32 = 2;
    pub const DEV_CONSOLE: u32 = 3;
    pub const TEST: u32 = 4;
    pub const AUTOMATION_PLAYER: u32 = 5;
    pub const CLIENT_AUTOMATION: u32 = 6;
    pub const DEDICATED_SERVER: u32 = 7;
    pub const ENTITY: u32 = 8;
    pub const VIRTUAL: u32 = 9;
    pub const GAME_ARGUMENT: u32 = 10;
    pub const ENTITY_SERVER: u32 = 11;
    pub const PRECOMPILED: u32 = 12;
    pub const GAME_DIRECTOR_ENTITY_SERVER: u32 = 13;
    pub const SCRIPTING: u32 = 14;
    pub const EXECUTE_CONTEXT: u32 = 15;

    pub const LABELS: [&str; 16] = [
        "player",
        "commandblock",
        "minecartcommandblock",
        "devconsole",
        "test",
        "automationplayer",
        "clientautomation",
        "dedicatedserver",
        "entity",
        "virtual",
        "gameargument",
        "entityserver",
        "precompiled",
        "gamedirectorentityserver",
        "scripting",
        "executecontext",
    ];
}

pub mod command_permission_level {
    pub const ANY: u32 = 0;
    pub const LABELS: [&str; 6] = ["any", "gamedirectors", "admin", "host", "owner", "internal"];
}

pub mod command_output_type {
    pub const NONE: u32 = 0;
    pub const LABELS: [&str; 5] = ["none", "lastoutput", "silent", "alloutput", "dataset"];
}

pub mod game_rule_type {
    pub const NULL: u32 = 0;
    pub const BOOL: u32 = 1;
    pub const INT: u32 = 2;
    pub const FLOAT: u32 = 3;
}

pub fn value_from_label(labels: &[&str], label: &str, default: u32) -> u32 {
    labels
        .iter()
        .position(|candidate| *candidate == label)
        .map(|index| index as u32)
        .unwrap_or(default)
}

pub fn label_from_value(labels: &[&'static str], value: u32) -> &'static str {
    let index = value as usize;
    labels
        .get(index)
        .copied()
        .unwrap_or_else(|| labels.first().copied().unwrap_or(""))
}
