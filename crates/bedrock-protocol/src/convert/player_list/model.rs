use bedrock_codec::prelude::*;

pub(super) struct Entry {
    pub(super) uuid: MceUuid,
    pub(super) added: Option<Added>,
}

pub(super) struct Added {
    pub(super) entity_unique_id: i64,
    pub(super) username: String,
    pub(super) xuid: String,
    pub(super) platform_chat_id: String,
    pub(super) build_platform: i32,
    pub(super) skin: Skin,
    pub(super) teacher: bool,
    pub(super) host: bool,
    pub(super) sub_client: bool,
    pub(super) player_colour: u32,
}

pub(super) struct Skin {
    pub(super) skin_id: String,
    pub(super) play_fab_id: String,
    pub(super) resource_patch: Vec<u8>,
    pub(super) skin_image_width: u32,
    pub(super) skin_image_height: u32,
    pub(super) skin_data: Vec<u8>,
    pub(super) animations: Vec<Animation>,
    pub(super) cape_image_width: u32,
    pub(super) cape_image_height: u32,
    pub(super) cape_data: Vec<u8>,
    pub(super) geometry: Vec<u8>,
    pub(super) geometry_engine_version: Vec<u8>,
    pub(super) animation_data: Vec<u8>,
    pub(super) cape_id: String,
    pub(super) full_id: String,
    pub(super) arm_size: u8,
    pub(super) skin_colour: u32,
    pub(super) persona_pieces: Vec<PersonaPiece>,
    pub(super) tint_colours: Vec<TintColour>,
    pub(super) premium: bool,
    pub(super) persona: bool,
    pub(super) persona_cape_on_classic: bool,
    pub(super) primary_user: bool,
    pub(super) override_appearance: bool,
    pub(super) trusted: bool,
    pub(super) profile_hash: String,
}

pub(super) struct Animation {
    pub(super) image_width: u32,
    pub(super) image_height: u32,
    pub(super) image_data: Vec<u8>,
    pub(super) animation_type: u32,
    pub(super) frame_count: f32,
    pub(super) expression_type: u32,
}

pub(super) struct PersonaPiece {
    pub(super) piece_id: String,
    pub(super) piece_type: u32,
    pub(super) pack_id: MceUuid,
    pub(super) default: bool,
    pub(super) product_id: String,
}

pub(super) struct TintColour {
    pub(super) piece_type: String,
    pub(super) colours: [u32; 4],
}
