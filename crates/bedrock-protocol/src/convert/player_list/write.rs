use super::codes::{
    arm_size_to_v1001, format_argb_colour, format_rgb_colour, format_uuid, piece_type_to_v1001,
    tint_type_to_v2168,
};
use super::model::{Entry, Skin};
use super::{ACTION_ADD, ACTION_REMOVE, VARIANT_ADD, VARIANT_REMOVE};
use bedrock_codec::prelude::*;

pub(super) fn write_v2168(w: &mut Writer, entries: &[Entry]) {
    w.write_count(entries.len());
    for entry in entries {
        match &entry.added {
            None => {
                w.write_uvarint(VARIANT_REMOVE);
                w.write_u8(ACTION_REMOVE);
                Uuid::write(w, &entry.uuid);
            }
            Some(added) => {
                w.write_uvarint(VARIANT_ADD);
                w.write_u8(ACTION_ADD);
                Uuid::write(w, &entry.uuid);
                w.write_varint64(added.entity_unique_id);
                Str::write(w, &added.username);
                Str::write(w, &added.xuid);
                Str::write(w, &added.platform_chat_id);
                w.write_i32_le(added.build_platform);
                write_skin_v2168(w, &added.skin);
                w.write_bool(added.teacher);
                w.write_bool(added.host);
                w.write_bool(added.sub_client);
                w.write_i32_be(added.player_colour as i32);
            }
        }
    }
}

fn write_skin_v2168(w: &mut Writer, s: &Skin) {
    Str::write(w, &s.skin_id);
    Str::write(w, &s.play_fab_id);
    ByteArray::write(w, &s.resource_patch);
    w.write_u32_le(s.skin_image_width);
    w.write_u32_le(s.skin_image_height);
    ByteArray::write(w, &s.skin_data);
    w.write_count(s.animations.len());
    for a in &s.animations {
        w.write_u32_le(a.image_width);
        w.write_u32_le(a.image_height);
        ByteArray::write(w, &a.image_data);
        w.write_uvarint(a.animation_type);
        w.write_f32_le(a.frame_count);
        w.write_uvarint(a.expression_type);
    }
    w.write_u32_le(s.cape_image_width);
    w.write_u32_le(s.cape_image_height);
    ByteArray::write(w, &s.cape_data);
    ByteArray::write(w, &s.geometry);
    ByteArray::write(w, &s.geometry_engine_version);
    ByteArray::write(w, &s.animation_data);
    Str::write(w, &s.cape_id);
    Str::write(w, &s.full_id);
    w.write_u8(s.arm_size);
    w.write_i32_be(s.skin_colour as i32);
    w.write_count(s.persona_pieces.len());
    for p in &s.persona_pieces {
        Str::write(w, &p.piece_id);
        w.write_u32_le(p.piece_type);
        Uuid::write(w, &p.pack_id);
        w.write_bool(p.default);
        Str::write(w, &p.product_id);
    }
    w.write_count(s.tint_colours.len());
    for t in &s.tint_colours {
        Str::write(w, &tint_type_to_v2168(&t.piece_type));
        for colour in t.colours {
            w.write_i32_be(colour as i32);
        }
    }
    w.write_bool(s.premium);
    w.write_bool(s.persona);
    w.write_bool(s.persona_cape_on_classic);
    w.write_bool(s.primary_user);
    w.write_bool(s.override_appearance);
    Str::write(w, &if s.trusted { "true" } else { "false" }.to_string());
    Str::write(w, &s.profile_hash);
}

pub(super) fn write_v1001(w: &mut Writer, entries: &[Entry]) -> bool {
    let adds = entries.iter().filter(|e| e.added.is_some()).count();
    if adds != 0 && adds != entries.len() {
        return false;
    }

    let action = if adds == 0 && !entries.is_empty() {
        ACTION_REMOVE
    } else {
        ACTION_ADD
    };
    w.write_u8(action);
    w.write_count(entries.len());

    if action == ACTION_REMOVE {
        for entry in entries {
            Uuid::write(w, &entry.uuid);
        }
        return true;
    }

    for entry in entries {
        Uuid::write(w, &entry.uuid);
        let added = match &entry.added {
            Some(a) => a,
            None => continue,
        };
        w.write_varint64(added.entity_unique_id);
        Str::write(w, &added.username);
        Str::write(w, &added.xuid);
        Str::write(w, &added.platform_chat_id);
        w.write_i32_le(added.build_platform);
        write_skin_v1001(w, &added.skin);
        w.write_bool(added.teacher);
        w.write_bool(added.host);
        w.write_bool(added.sub_client);
        w.write_u32_le(added.player_colour);
    }
    for entry in entries {
        w.write_bool(entry.added.as_ref().is_some_and(|a| a.skin.trusted));
    }
    true
}

fn write_skin_v1001(w: &mut Writer, s: &Skin) {
    Str::write(w, &s.skin_id);
    Str::write(w, &s.play_fab_id);
    ByteArray::write(w, &s.resource_patch);
    w.write_u32_le(s.skin_image_width);
    w.write_u32_le(s.skin_image_height);
    ByteArray::write(w, &s.skin_data);
    w.write_u32_le(s.animations.len() as u32);
    for a in &s.animations {
        w.write_u32_le(a.image_width);
        w.write_u32_le(a.image_height);
        ByteArray::write(w, &a.image_data);
        w.write_u32_le(a.animation_type);
        w.write_f32_le(a.frame_count);
        w.write_u32_le(a.expression_type);
    }
    w.write_u32_le(s.cape_image_width);
    w.write_u32_le(s.cape_image_height);
    ByteArray::write(w, &s.cape_data);
    ByteArray::write(w, &s.geometry);
    ByteArray::write(w, &s.geometry_engine_version);
    ByteArray::write(w, &s.animation_data);
    Str::write(w, &s.cape_id);
    Str::write(w, &s.full_id);
    Str::write(w, &arm_size_to_v1001(s.arm_size).to_string());
    Str::write(w, &format_rgb_colour(s.skin_colour));
    w.write_u32_le(s.persona_pieces.len() as u32);
    for p in &s.persona_pieces {
        Str::write(w, &p.piece_id);
        Str::write(w, &piece_type_to_v1001(p.piece_type));
        Str::write(w, &format_uuid(&p.pack_id));
        w.write_bool(p.default);
        Str::write(w, &p.product_id);
    }
    w.write_u32_le(s.tint_colours.len() as u32);
    for t in &s.tint_colours {
        Str::write(w, &t.piece_type);
        w.write_u32_le(4);
        for colour in t.colours {
            Str::write(w, &format_argb_colour(colour));
        }
    }
    w.write_bool(s.premium);
    w.write_bool(s.persona);
    w.write_bool(s.persona_cape_on_classic);
    w.write_bool(s.primary_user);
    w.write_bool(s.override_appearance);
}
