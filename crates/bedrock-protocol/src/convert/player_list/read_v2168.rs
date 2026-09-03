use super::codes::tint_type_to_v1001;
use super::model::{Added, Animation, Entry, PersonaPiece, Skin, TintColour};
use super::{VARIANT_ADD, VARIANT_REMOVE};
use bedrock_codec::prelude::*;

pub(super) fn read_v2168(r: &mut Reader<'_>) -> Result<Vec<Entry>> {
    let count = r.read_count()?;
    let mut entries = Vec::with_capacity(count);
    for _ in 0..count {
        let variant = r.read_uvarint()?;
        r.read_u8()?;
        let uuid = Uuid::read(r)?;
        if variant == VARIANT_REMOVE {
            entries.push(Entry { uuid, added: None });
            continue;
        }
        if variant != VARIANT_ADD {
            return Err(Error::BadDiscriminant {
                what: "player list entry variant",
                value: i64::from(variant),
            });
        }
        entries.push(Entry {
            uuid,
            added: Some(Added {
                entity_unique_id: r.read_varint64()?,
                username: Str::read(r)?,
                xuid: Str::read(r)?,
                platform_chat_id: Str::read(r)?,
                build_platform: r.read_i32_le()?,
                skin: read_skin_v2168(r)?,
                teacher: r.read_bool()?,
                host: r.read_bool()?,
                sub_client: r.read_bool()?,
                player_colour: r.read_i32_be()? as u32,
            }),
        });
    }
    Ok(entries)
}

fn read_skin_v2168(r: &mut Reader<'_>) -> Result<Skin> {
    let skin_id = Str::read(r)?;
    let play_fab_id = Str::read(r)?;
    let resource_patch = ByteArray::read(r)?;
    let skin_image_width = r.read_u32_le()?;
    let skin_image_height = r.read_u32_le()?;
    let skin_data = ByteArray::read(r)?;

    let mut animations = Vec::new();
    for _ in 0..r.read_count()? {
        animations.push(Animation {
            image_width: r.read_u32_le()?,
            image_height: r.read_u32_le()?,
            image_data: ByteArray::read(r)?,
            animation_type: r.read_uvarint()?,
            frame_count: r.read_f32_le()?,
            expression_type: r.read_uvarint()?,
        });
    }

    let cape_image_width = r.read_u32_le()?;
    let cape_image_height = r.read_u32_le()?;
    let cape_data = ByteArray::read(r)?;
    let geometry = ByteArray::read(r)?;
    let geometry_engine_version = ByteArray::read(r)?;
    let animation_data = ByteArray::read(r)?;
    let cape_id = Str::read(r)?;
    let full_id = Str::read(r)?;
    let arm_size = r.read_u8()?;
    let skin_colour = r.read_i32_be()? as u32;

    let mut persona_pieces = Vec::new();
    for _ in 0..r.read_count()? {
        persona_pieces.push(PersonaPiece {
            piece_id: Str::read(r)?,
            piece_type: r.read_u32_le()?,
            pack_id: Uuid::read(r)?,
            default: r.read_bool()?,
            product_id: Str::read(r)?,
        });
    }

    let mut tint_colours = Vec::new();
    for _ in 0..r.read_count()? {
        let wire_type = Str::read(r)?;
        let mut colours = [0u32; 4];
        for slot in colours.iter_mut() {
            *slot = r.read_i32_be()? as u32;
        }
        tint_colours.push(TintColour {
            piece_type: tint_type_to_v1001(&wire_type),
            colours,
        });
    }

    Ok(Skin {
        skin_id,
        play_fab_id,
        resource_patch,
        skin_image_width,
        skin_image_height,
        skin_data,
        animations,
        cape_image_width,
        cape_image_height,
        cape_data,
        geometry,
        geometry_engine_version,
        animation_data,
        cape_id,
        full_id,
        arm_size,
        skin_colour,
        persona_pieces,
        tint_colours,
        premium: r.read_bool()?,
        persona: r.read_bool()?,
        persona_cape_on_classic: r.read_bool()?,
        primary_user: r.read_bool()?,
        override_appearance: r.read_bool()?,
        trusted: {
            let text = Str::read(r)?;
            text.eq_ignore_ascii_case("true")
        },
        profile_hash: Str::read(r)?,
    })
}
