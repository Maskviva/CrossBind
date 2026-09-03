use super::codes::{arm_size_to_v2168, parse_hex_colour, parse_uuid, piece_type_to_v2168};
use super::model::{Added, Animation, Entry, PersonaPiece, Skin, TintColour};
use super::{ACTION_ADD, ACTION_REMOVE};
use bedrock_codec::prelude::*;

pub(super) fn read_v1001(r: &mut Reader<'_>) -> Result<Vec<Entry>> {
    let action = r.read_u8()?;
    let count = r.read_count()?;
    let mut entries = Vec::with_capacity(count);
    match action {
        ACTION_REMOVE => {
            for _ in 0..count {
                entries.push(Entry {
                    uuid: Uuid::read(r)?,
                    added: None,
                });
            }
        }
        ACTION_ADD => {
            for _ in 0..count {
                entries.push(read_v1001_added(r)?);
            }
            for entry in entries.iter_mut() {
                let trusted = r.read_bool()?;
                if let Some(added) = entry.added.as_mut() {
                    added.skin.trusted = trusted;
                }
            }
        }
        other => {
            return Err(Error::BadDiscriminant {
                what: "player list action type",
                value: i64::from(other),
            })
        }
    }
    Ok(entries)
}

fn read_v1001_added(r: &mut Reader<'_>) -> Result<Entry> {
    let uuid = Uuid::read(r)?;
    Ok(Entry {
        uuid,
        added: Some(Added {
            entity_unique_id: r.read_varint64()?,
            username: Str::read(r)?,
            xuid: Str::read(r)?,
            platform_chat_id: Str::read(r)?,
            build_platform: r.read_i32_le()?,
            skin: read_skin_v1001(r)?,
            teacher: r.read_bool()?,
            host: r.read_bool()?,
            sub_client: r.read_bool()?,
            player_colour: r.read_u32_le()?,
        }),
    })
}

fn read_skin_v1001(r: &mut Reader<'_>) -> Result<Skin> {
    let skin_id = Str::read(r)?;
    let play_fab_id = Str::read(r)?;
    let resource_patch = ByteArray::read(r)?;
    let skin_image_width = r.read_u32_le()?;
    let skin_image_height = r.read_u32_le()?;
    let skin_data = ByteArray::read(r)?;

    let mut animations = Vec::new();
    for _ in 0..read_u32_count(r)? {
        animations.push(Animation {
            image_width: r.read_u32_le()?,
            image_height: r.read_u32_le()?,
            image_data: ByteArray::read(r)?,
            animation_type: r.read_u32_le()?,
            frame_count: r.read_f32_le()?,
            expression_type: r.read_u32_le()?,
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
    let arm_size = arm_size_to_v2168(&Str::read(r)?);
    let skin_colour = parse_hex_colour(&Str::read(r)?);

    let mut persona_pieces = Vec::new();
    for _ in 0..read_u32_count(r)? {
        persona_pieces.push(PersonaPiece {
            piece_id: Str::read(r)?,
            piece_type: piece_type_to_v2168(&Str::read(r)?),
            pack_id: parse_uuid(&Str::read(r)?),
            default: r.read_bool()?,
            product_id: Str::read(r)?,
        });
    }

    let mut tint_colours = Vec::new();
    for _ in 0..read_u32_count(r)? {
        let piece_type = Str::read(r)?;
        let mut colours = [0u32; 4];
        #[allow(clippy::needless_range_loop)]
        for i in 0..read_u32_count(r)? {
            let text = Str::read(r)?;
            if i < 4 {
                colours[i] = parse_hex_colour(&text);
            }
        }
        tint_colours.push(TintColour {
            piece_type,
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
        trusted: false,
        profile_hash: String::new(),
    })
}

fn read_u32_count(r: &mut Reader<'_>) -> Result<usize> {
    let count = r.read_u32_le()? as usize;
    if count > r.remaining() {
        return Err(Error::LengthLimit {
            got: count,
            limit: r.remaining(),
        });
    }
    Ok(count)
}
