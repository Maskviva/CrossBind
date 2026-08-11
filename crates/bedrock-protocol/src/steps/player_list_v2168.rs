use bedrock_codec::prelude::*;

use crate::connection::ConnState;

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

struct Entry {
    uuid: MceUuid,
    added: Option<Added>,
}

struct Added {
    entity_unique_id: i64,
    username: String,
    xuid: String,
    platform_chat_id: String,
    build_platform: i32,
    skin: Skin,
    teacher: bool,
    host: bool,
    sub_client: bool,
    player_colour: u32,
}

struct Skin {
    skin_id: String,
    play_fab_id: String,
    resource_patch: Vec<u8>,
    skin_image_width: u32,
    skin_image_height: u32,
    skin_data: Vec<u8>,
    animations: Vec<Animation>,
    cape_image_width: u32,
    cape_image_height: u32,
    cape_data: Vec<u8>,
    geometry: Vec<u8>,
    geometry_engine_version: Vec<u8>,
    animation_data: Vec<u8>,
    cape_id: String,
    full_id: String,
    arm_size: u8,
    skin_colour: u32,
    persona_pieces: Vec<PersonaPiece>,
    tint_colours: Vec<TintColour>,
    premium: bool,
    persona: bool,
    persona_cape_on_classic: bool,
    primary_user: bool,
    override_appearance: bool,
    trusted: bool,
    profile_hash: String,
}

struct Animation {
    image_width: u32,
    image_height: u32,
    image_data: Vec<u8>,
    animation_type: u32,
    frame_count: f32,
    expression_type: u32,
}

struct PersonaPiece {
    piece_id: String,
    piece_type: u32,
    pack_id: MceUuid,
    default: bool,
    product_id: String,
}

struct TintColour {
    piece_type: String,
    colours: [u32; 4],
}

fn read_v1001(r: &mut Reader<'_>) -> Result<Vec<Entry>> {
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

fn read_v2168(r: &mut Reader<'_>) -> Result<Vec<Entry>> {
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

fn write_v2168(w: &mut Writer, entries: &[Entry]) {
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

fn write_v1001(w: &mut Writer, entries: &[Entry]) -> bool {
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

fn arm_size_to_v2168(name: &str) -> u8 {
    if name.eq_ignore_ascii_case("slim") {
        ARM_SIZE_SLIM
    } else {
        ARM_SIZE_WIDE
    }
}

fn arm_size_to_v1001(size: u8) -> &'static str {
    if size == ARM_SIZE_SLIM {
        "slim"
    } else {
        "wide"
    }
}

fn piece_type_to_v2168(name: &str) -> u32 {
    let name = if name == "persona_hands" {
        "persona_hand"
    } else {
        name
    };
    match PIECE_TYPES.iter().position(|n| *n == name) {
        Some(index) => index as u32 + 1,
        None => 0,
    }
}

fn piece_type_to_v1001(value: u32) -> String {
    match value.checked_sub(1).and_then(|i| PIECE_TYPES.get(i as usize)) {
        Some(name) => (*name).to_string(),
        None => String::new(),
    }
}

fn tint_type_to_v2168(name: &str) -> String {
    if name == "persona_hand" || name == "persona_hands" {
        return "hands".to_string();
    }
    name.strip_prefix("persona_").unwrap_or(name).to_string()
}

fn tint_type_to_v1001(wire: &str) -> String {
    match wire {
        "hands" => "persona_hand".to_string(),
        "unsupported" => wire.to_string(),
        _ => format!("persona_{wire}"),
    }
}

fn parse_hex_colour(text: &str) -> u32 {
    let hex = text.strip_prefix('#').unwrap_or(text);
    let value = match u32::from_str_radix(hex, 16) {
        Ok(v) => v,
        Err(_) => return 0,
    };
    let (a, rgb) = match hex.len() {
        6 => (0xffu32, value),
        8 => (value >> 24, value & 0x00ff_ffff),
        _ => return 0,
    };
    let r = (rgb >> 16) & 0xff;
    let g = (rgb >> 8) & 0xff;
    let b = rgb & 0xff;
    a | (r << 8) | (g << 16) | (b << 24)
}

fn format_rgb_colour(packed: u32) -> String {
    let r = (packed >> 8) & 0xff;
    let g = (packed >> 16) & 0xff;
    let b = (packed >> 24) & 0xff;
    format!("#{r:02x}{g:02x}{b:02x}")
}

fn format_argb_colour(packed: u32) -> String {
    if packed == 0 {
        return "#0".to_string();
    }
    let a = packed & 0xff;
    let r = (packed >> 8) & 0xff;
    let g = (packed >> 16) & 0xff;
    let b = (packed >> 24) & 0xff;
    format!("#{a:02x}{r:02x}{g:02x}{b:02x}")
}

fn parse_uuid(text: &str) -> MceUuid {
    let mut nibbles = [0u8; 32];
    let mut seen = 0;
    for ch in text.chars() {
        if ch == '-' {
            continue;
        }
        match ch.to_digit(16) {
            Some(d) if seen < 32 => {
                nibbles[seen] = d as u8;
                seen += 1;
            }
            _ => return MceUuid::default(),
        }
    }
    if seen != 32 {
        return MceUuid::default();
    }
    let half = |start: usize| {
        nibbles[start..start + 16]
            .iter()
            .fold(0u64, |acc, n| (acc << 4) | u64::from(*n))
    };
    MceUuid {
        msb: half(0),
        lsb: half(16),
    }
}

fn format_uuid(id: &MceUuid) -> String {
    if id.msb == 0 && id.lsb == 0 {
        return String::new();
    }
    let hex = format!("{:016x}{:016x}", id.msb, id.lsb);
    format!(
        "{}-{}-{}-{}-{}",
        &hex[0..8],
        &hex[8..12],
        &hex[12..16],
        &hex[16..20],
        &hex[20..32]
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use bedrock_codec::PacketWrapper;

    fn skin_bytes_v1001(w: &mut Writer) {
        Str::write(w, &"skin-id".to_string());
        Str::write(w, &"playfab".to_string());
        ByteArray::write(w, &b"{patch}".to_vec());
        w.write_u32_le(64);
        w.write_u32_le(32);
        ByteArray::write(w, &vec![1u8, 2, 3, 4]);
        w.write_u32_le(1);
        w.write_u32_le(8);
        w.write_u32_le(8);
        ByteArray::write(w, &vec![9u8]);
        w.write_u32_le(2);
        w.write_f32_le(1.5);
        w.write_u32_le(3);
        w.write_u32_le(0);
        w.write_u32_le(0);
        ByteArray::write(w, &Vec::new());
        ByteArray::write(w, &b"{geo}".to_vec());
        ByteArray::write(w, &b"1.8.0".to_vec());
        ByteArray::write(w, &b"{anim}".to_vec());
        Str::write(w, &"cape-id".to_string());
        Str::write(w, &"full-id".to_string());
        Str::write(w, &"wide".to_string());
        Str::write(w, &"#b37b62".to_string());
        w.write_u32_le(1);
        Str::write(w, &"piece-id".to_string());
        Str::write(w, &"persona_eyes".to_string());
        Str::write(w, &"3d29a1a4-1c1e-4e4d-9b2f-000000000001".to_string());
        w.write_bool(true);
        Str::write(w, &"product".to_string());
        w.write_u32_le(1);
        Str::write(w, &"persona_eyes".to_string());
        w.write_u32_le(4);
        for c in ["#ffa12722", "#ff2f1f0f", "#ff3aafd9", "#ff000000"] {
            Str::write(w, &c.to_string());
        }
        w.write_bool(false);
        w.write_bool(true);
        w.write_bool(false);
        w.write_bool(true);
        w.write_bool(false);
    }

    fn v1001_add() -> Vec<u8> {
        let mut w = Writer::new();
        w.write_u8(ACTION_ADD);
        w.write_count(1);
        Uuid::write(
            &mut w,
            &MceUuid {
                msb: 0x0123_4567_89ab_cdef,
                lsb: 0xfedc_ba98_7654_3210,
            },
        );
        w.write_varint64(42);
        Str::write(&mut w, &"RSxiaotong".to_string());
        Str::write(&mut w, &"2535424392028628".to_string());
        Str::write(&mut w, &String::new());
        w.write_i32_le(7);
        skin_bytes_v1001(&mut w);
        w.write_bool(false);
        w.write_bool(true);
        w.write_bool(false);
        w.write_u32_le(0xdd_cc_bb_aa);
        w.write_bool(true);
        w.into_vec()
    }

    #[test]
    fn v1001_add_becomes_a_v2168_entry_with_the_name_intact() {
        let input = v1001_add();
        let mut state = ConnState::new(975);
        let mut wrapper = PacketWrapper::new(&input);
        assert!(player_list(&mut wrapper, &mut state, true).unwrap());
        assert!(state.notices.is_empty());

        let out = wrapper.finish();
        let mut r = Reader::new(&out);
        assert_eq!(r.read_count().unwrap(), 1);
        assert_eq!(r.read_uvarint().unwrap(), VARIANT_ADD);
        assert_eq!(r.read_u8().unwrap(), ACTION_ADD);
        Uuid::read(&mut r).unwrap();
        assert_eq!(r.read_varint64().unwrap(), 42);
        assert_eq!(Str::read(&mut r).unwrap(), "RSxiaotong");
        assert_eq!(Str::read(&mut r).unwrap(), "2535424392028628");
    }

    #[test]
    fn add_round_trips_through_v2168_and_back() {
        let input = v1001_add();
        let mut state = ConnState::new(975);

        let mut up = PacketWrapper::new(&input);
        assert!(player_list(&mut up, &mut state, true).unwrap());
        let v2168_bytes = up.finish();

        let mut down = PacketWrapper::new(&v2168_bytes);
        assert!(player_list(&mut down, &mut state, false).unwrap());
        assert_eq!(down.finish(), input, "v1001 -> v2168 -> v1001 must be exact");
        assert!(state.notices.is_empty());
    }

    #[test]
    fn remove_lists_carry_only_uuids() {
        let mut w = Writer::new();
        w.write_u8(ACTION_REMOVE);
        w.write_count(2);
        Uuid::write(&mut w, &MceUuid { msb: 1, lsb: 2 });
        Uuid::write(&mut w, &MceUuid { msb: 3, lsb: 4 });
        let input = w.into_vec();

        let mut state = ConnState::new(975);
        let mut wrapper = PacketWrapper::new(&input);
        assert!(player_list(&mut wrapper, &mut state, true).unwrap());
        let out = wrapper.finish();

        let mut r = Reader::new(&out);
        assert_eq!(r.read_count().unwrap(), 2);
        for expected in [MceUuid { msb: 1, lsb: 2 }, MceUuid { msb: 3, lsb: 4 }] {
            assert_eq!(r.read_uvarint().unwrap(), VARIANT_REMOVE);
            assert_eq!(r.read_u8().unwrap(), ACTION_REMOVE);
            assert_eq!(Uuid::read(&mut r).unwrap(), expected);
        }
        assert!(!r.has_remaining(), "a removal carries nothing past the UUID");

        let mut down = PacketWrapper::new(&out);
        assert!(player_list(&mut down, &mut state, false).unwrap());
        assert_eq!(down.finish(), input);
    }

    #[test]
    fn player_colour_is_a_byte_reversal_not_a_repack() {
        let input = v1001_add();
        let mut state = ConnState::new(975);
        let mut up = PacketWrapper::new(&input);
        player_list(&mut up, &mut state, true).unwrap();
        let out = up.finish();
        let tail = &out[out.len() - 4..];
        assert_eq!(tail, &[0xdd, 0xcc, 0xbb, 0xaa]);
    }

    #[test]
    fn a_mixed_v2168_list_is_cancelled_rather_than_half_applied() {
        let mut w = Writer::new();
        w.write_count(2);
        w.write_uvarint(VARIANT_REMOVE);
        w.write_u8(ACTION_REMOVE);
        Uuid::write(&mut w, &MceUuid { msb: 1, lsb: 2 });
        w.write_uvarint(VARIANT_ADD);
        w.write_u8(ACTION_ADD);
        Uuid::write(&mut w, &MceUuid { msb: 3, lsb: 4 });
        w.write_varint64(1);
        Str::write(&mut w, &"Someone".to_string());
        Str::write(&mut w, &String::new());
        Str::write(&mut w, &String::new());
        w.write_i32_le(0);
        let mut skin = Writer::new();
        skin_bytes_v1001(&mut skin);
        let _ = skin;
        let input = w.into_vec();

        let mut state = ConnState::new(2168);
        let mut wrapper = PacketWrapper::new(&input);
        assert!(!player_list(&mut wrapper, &mut state, false).unwrap());
        assert_eq!(state.notices.len(), 1);
    }

    #[test]
    fn colour_and_uuid_helpers_round_trip() {
        assert_eq!(format_rgb_colour(parse_hex_colour("#b37b62")), "#b37b62");
        assert_eq!(format_argb_colour(parse_hex_colour("#ffa12722")), "#ffa12722");
        assert_eq!(format_argb_colour(parse_hex_colour("#0")), "#0");
        assert_eq!(parse_hex_colour("#0"), 0);
        assert_eq!(parse_hex_colour(""), 0);
        let id = "3d29a1a4-1c1e-4e4d-9b2f-000000000001";
        assert_eq!(format_uuid(&parse_uuid(id)), id);
        assert_eq!(format_uuid(&parse_uuid("not a uuid")), "");
    }

    #[test]
    fn persona_piece_names_map_onto_the_documented_enum_positions() {
        for (name, expected) in [
            ("persona_skeleton", 1u32),
            ("persona_body", 2),
            ("persona_skin", 3),
            ("persona_bottom", 4),
            ("persona_feet", 5),
            ("persona_top", 7),
            ("persona_mouth", 12),
            ("persona_eyes", 13),
            ("persona_hair", 14),
            ("persona_facial_hair", 11),
        ] {
            assert_eq!(piece_type_to_v2168(name), expected, "{name}");
            assert_eq!(piece_type_to_v1001(expected), name);
        }
        assert_eq!(piece_type_to_v2168("persona_something_new"), 0);
    }

    #[test]
    fn tint_piece_names_lose_and_regain_their_prefix() {
        assert_eq!(tint_type_to_v2168("persona_eyes"), "eyes");
        assert_eq!(tint_type_to_v1001("eyes"), "persona_eyes");
        assert_eq!(tint_type_to_v2168("persona_hand"), "hands");
        assert_eq!(tint_type_to_v1001("hands"), "persona_hand");
        assert_eq!(tint_type_to_v1001("unsupported"), "unsupported");
    }
}
