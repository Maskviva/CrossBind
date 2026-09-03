mod codes;
mod model;
mod read_v1001;
mod read_v2168;
mod write;
#[cfg(test)]
mod tests;

use crate::connection::ConnState;
use bedrock_codec::prelude::*;
use read_v1001::read_v1001;
use read_v2168::read_v2168;
use write::{write_v1001, write_v2168};

const ACTION_ADD: u8 = 0;

const ACTION_REMOVE: u8 = 1;

const VARIANT_ADD: u32 = 1;

const VARIANT_REMOVE: u32 = 0;

const ARM_SIZE_SLIM: u8 = 0;

const ARM_SIZE_WIDE: u8 = 1;

const PIECE_TYPES: &[&str] = &[
    "persona_skeleton",
    "persona_body",
    "persona_skin",
    "persona_bottom",
    "persona_feet",
    "persona_dress",
    "persona_top",
    "persona_high_pants",
    "persona_hand",
    "persona_outerwear",
    "persona_facial_hair",
    "persona_mouth",
    "persona_eyes",
    "persona_hair",
    "persona_hood",
    "persona_back",
    "persona_face_accessory",
    "persona_head",
    "persona_legs",
    "persona_left_leg",
    "persona_right_leg",
    "persona_arms",
    "persona_left_arm",
    "persona_right_arm",
    "persona_capes",
    "persona_classic_skin",
    "persona_emote",
    "persona_unsupported",
];

pub(crate) fn player_list(
    w: &mut PacketWrapper,
    state: &mut ConnState,
    to_v2168: bool,
) -> Result<bool> {
    match translate(w, to_v2168) {
        Ok(true) => Ok(true),
        Ok(false) => {
            state.notices.push(
                "PlayerList: cancelled, a v2168 list mixes added and removed \
                 entries and v1001 has one action byte for the whole packet"
                    .to_string(),
            );
            Ok(false)
        }
        Err(err) => {
            state
                .notices
                .push(format!("PlayerList: cancelled, cannot decode: {err}"));
            Ok(false)
        }
    }
}

fn translate(w: &mut PacketWrapper, to_v2168: bool) -> Result<bool> {
    let entries = if to_v2168 {
        read_v1001(w.reader())?
    } else {
        read_v2168(w.reader())?
    };
    if w.reader().has_remaining() {
        return Err(Error::Invalid("trailing bytes after player list"));
    }

    let mut out = Writer::new();
    if to_v2168 {
        write_v2168(&mut out, &entries);
    } else if !write_v1001(&mut out, &entries) {
        return Ok(false);
    }
    w.writer().write_bytes(&out.into_vec());
    Ok(true)
}
