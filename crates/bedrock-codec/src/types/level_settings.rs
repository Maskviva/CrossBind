use core::marker::PhantomData;

use crate::types::gameplay::{Experiment, ExperimentEntry, GameRule, GameRules, GameRulesV2168};
use crate::types::primitives::{ArrayI32, ArrayU32, Optional, UVarInt, VarInt};
use crate::{Codec, Reader, Result, Writer};

#[derive(Debug, Clone, PartialEq)]
pub struct LevelSettings {
    pub seed: i64,
    pub spawn_biome_type: i16,
    pub spawn_biome_name: String,
    pub spawn_dimension: i32,
    pub generator: i32,
    pub game_type: i32,
    pub is_hardcore: bool,
    pub game_difficulty: i32,
    pub default_spawn_x: i32,
    pub default_spawn_y: i32,
    pub default_spawn_z: i32,
    pub achievements_disabled: bool,
    pub editor_world_type: i32,
    pub created_in_editor: bool,
    pub exported_from_editor: bool,
    pub day_cycle_stop_time: i32,
    pub education_edition_offer: i32,
    pub education_features_enabled: bool,
    pub education_product_id: String,
    pub rain_level: f32,
    pub lightning_level: f32,
    pub has_confirmed_platform_locked_content: bool,
    pub multiplayer_intended: bool,
    pub lan_broadcasting_intended: bool,
    pub xbox_live_broadcast_setting: i32,
    pub platform_broadcast_setting: i32,
    pub commands_enabled: bool,
    pub texture_packs_required: bool,
    pub game_rules: Vec<GameRule>,
    pub experiments: Vec<Experiment>,
    pub ever_toggled: bool,
    pub has_bonus_chest: bool,
    pub start_with_map: bool,
    pub player_permissions: i32,
    pub server_chunk_tick_range: i32,
    pub has_locked_behavior_pack: bool,
    pub has_locked_resource_pack: bool,
    pub is_from_locked_template: bool,
    pub use_msa_gamertags_only: bool,
    pub created_from_template: bool,
    pub template_with_locked_settings: bool,
    pub only_spawn_v1_villagers: bool,
    pub persona_disabled: bool,
    pub custom_skins_disabled: bool,
    pub emote_chat_muted: bool,
    pub base_game_version: String,
    pub limited_world_width: i32,
    pub limited_world_depth: i32,
    pub nether_type: bool,
    pub edu_shared_uri_button_name: String,
    pub edu_shared_uri_link_uri: String,
    pub override_force_experimental: Option<bool>,
    pub chat_restriction_level: u8,
    pub disable_player_interactions: bool,
}

pub struct LevelSettingsWith<Y, E, O = VarInt, P = VarInt, G = GameRules>(
    PhantomData<(Y, E, O, P, G)>,
);

impl<Y, E, O, P, G> Codec for LevelSettingsWith<Y, E, O, P, G>
where
    Y: Codec<Value = i32>,
    E: Codec<Value = Vec<Experiment>>,
    O: Codec<Value = i32>,
    P: Codec<Value = i32>,
    G: Codec<Value = Vec<GameRule>>,
{
    type Value = LevelSettings;

    fn read(r: &mut Reader<'_>) -> Result<LevelSettings> {
        Ok(LevelSettings {
            seed: r.read_i64_le()?,
            spawn_biome_type: r.read_i16_le()?,
            spawn_biome_name: r.read_string()?,
            spawn_dimension: r.read_varint()?,
            generator: r.read_varint()?,
            game_type: r.read_varint()?,
            is_hardcore: r.read_bool()?,
            game_difficulty: r.read_varint()?,
            default_spawn_x: r.read_varint()?,
            default_spawn_y: Y::read(r)?,
            default_spawn_z: r.read_varint()?,
            achievements_disabled: r.read_bool()?,
            editor_world_type: r.read_varint()?,
            created_in_editor: r.read_bool()?,
            exported_from_editor: r.read_bool()?,
            day_cycle_stop_time: r.read_varint()?,
            education_edition_offer: O::read(r)?,
            education_features_enabled: r.read_bool()?,
            education_product_id: r.read_string()?,
            rain_level: r.read_f32_le()?,
            lightning_level: r.read_f32_le()?,
            has_confirmed_platform_locked_content: r.read_bool()?,
            multiplayer_intended: r.read_bool()?,
            lan_broadcasting_intended: r.read_bool()?,
            xbox_live_broadcast_setting: r.read_varint()?,
            platform_broadcast_setting: r.read_varint()?,
            commands_enabled: r.read_bool()?,
            texture_packs_required: r.read_bool()?,
            game_rules: G::read(r)?,
            experiments: E::read(r)?,
            ever_toggled: r.read_bool()?,
            has_bonus_chest: r.read_bool()?,
            start_with_map: r.read_bool()?,
            player_permissions: P::read(r)?,
            server_chunk_tick_range: r.read_i32_le()?,
            has_locked_behavior_pack: r.read_bool()?,
            has_locked_resource_pack: r.read_bool()?,
            is_from_locked_template: r.read_bool()?,
            use_msa_gamertags_only: r.read_bool()?,
            created_from_template: r.read_bool()?,
            template_with_locked_settings: r.read_bool()?,
            only_spawn_v1_villagers: r.read_bool()?,
            persona_disabled: r.read_bool()?,
            custom_skins_disabled: r.read_bool()?,
            emote_chat_muted: r.read_bool()?,
            base_game_version: r.read_string()?,
            limited_world_width: r.read_i32_le()?,
            limited_world_depth: r.read_i32_le()?,
            nether_type: r.read_bool()?,
            edu_shared_uri_button_name: r.read_string()?,
            edu_shared_uri_link_uri: r.read_string()?,
            override_force_experimental: Optional::<crate::types::primitives::Bool>::read(r)?,
            chat_restriction_level: r.read_u8()?,
            disable_player_interactions: r.read_bool()?,
        })
    }

    fn write(w: &mut Writer, v: &LevelSettings) {
        w.write_i64_le(v.seed);
        w.write_i16_le(v.spawn_biome_type);
        w.write_string(&v.spawn_biome_name);
        w.write_varint(v.spawn_dimension);
        w.write_varint(v.generator);
        w.write_varint(v.game_type);
        w.write_bool(v.is_hardcore);
        w.write_varint(v.game_difficulty);
        w.write_varint(v.default_spawn_x);
        Y::write(w, &v.default_spawn_y);
        w.write_varint(v.default_spawn_z);
        w.write_bool(v.achievements_disabled);
        w.write_varint(v.editor_world_type);
        w.write_bool(v.created_in_editor);
        w.write_bool(v.exported_from_editor);
        w.write_varint(v.day_cycle_stop_time);
        O::write(w, &v.education_edition_offer);
        w.write_bool(v.education_features_enabled);
        w.write_string(&v.education_product_id);
        w.write_f32_le(v.rain_level);
        w.write_f32_le(v.lightning_level);
        w.write_bool(v.has_confirmed_platform_locked_content);
        w.write_bool(v.multiplayer_intended);
        w.write_bool(v.lan_broadcasting_intended);
        w.write_varint(v.xbox_live_broadcast_setting);
        w.write_varint(v.platform_broadcast_setting);
        w.write_bool(v.commands_enabled);
        w.write_bool(v.texture_packs_required);
        G::write(w, &v.game_rules);
        E::write(w, &v.experiments);
        w.write_bool(v.ever_toggled);
        w.write_bool(v.has_bonus_chest);
        w.write_bool(v.start_with_map);
        P::write(w, &v.player_permissions);
        w.write_i32_le(v.server_chunk_tick_range);
        w.write_bool(v.has_locked_behavior_pack);
        w.write_bool(v.has_locked_resource_pack);
        w.write_bool(v.is_from_locked_template);
        w.write_bool(v.use_msa_gamertags_only);
        w.write_bool(v.created_from_template);
        w.write_bool(v.template_with_locked_settings);
        w.write_bool(v.only_spawn_v1_villagers);
        w.write_bool(v.persona_disabled);
        w.write_bool(v.custom_skins_disabled);
        w.write_bool(v.emote_chat_muted);
        w.write_string(&v.base_game_version);
        w.write_i32_le(v.limited_world_width);
        w.write_i32_le(v.limited_world_depth);
        w.write_bool(v.nether_type);
        w.write_string(&v.edu_shared_uri_button_name);
        w.write_string(&v.edu_shared_uri_link_uri);
        Optional::<crate::types::primitives::Bool>::write(w, &v.override_force_experimental);
        w.write_u8(v.chat_restriction_level);
        w.write_bool(v.disable_player_interactions);
    }
}

pub type LevelSettingsV860 = LevelSettingsWith<UVarInt32AsI32, ArrayI32<ExperimentEntry>>;
pub type LevelSettingsV924 = LevelSettingsWith<UVarInt32AsI32, ArrayU32<ExperimentEntry>>;
pub type LevelSettingsV944 = LevelSettingsWith<VarInt, ArrayU32<ExperimentEntry>>;
pub type LevelSettingsV2168 =
    LevelSettingsWith<VarInt, ArrayU32<ExperimentEntry>, UVarInt32AsI32, ByteAsI32, GameRulesV2168>;

pub struct UVarInt32AsI32;

impl Codec for UVarInt32AsI32 {
    type Value = i32;
    fn read(r: &mut Reader<'_>) -> Result<i32> {
        Ok(UVarInt::read(r)? as i32)
    }
    fn write(w: &mut Writer, v: &i32) {
        UVarInt::write(w, &(*v as u32))
    }
}

pub struct ByteAsI32;

impl Codec for ByteAsI32 {
    type Value = i32;
    fn read(r: &mut Reader<'_>) -> Result<i32> {
        Ok(r.read_u8()? as i32)
    }
    fn write(w: &mut Writer, v: &i32) {
        w.write_u8(*v as u8)
    }
}
