use super::{ARM_SIZE_SLIM, ARM_SIZE_WIDE, PIECE_TYPES};
use bedrock_codec::prelude::*;

pub(super) fn arm_size_to_v2168(name: &str) -> u8 {
    if name.eq_ignore_ascii_case("slim") {
        ARM_SIZE_SLIM
    } else {
        ARM_SIZE_WIDE
    }
}

pub(super) fn arm_size_to_v1001(size: u8) -> &'static str {
    if size == ARM_SIZE_SLIM {
        "slim"
    } else {
        "wide"
    }
}

pub(super) fn piece_type_to_v2168(name: &str) -> u32 {
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

pub(super) fn piece_type_to_v1001(value: u32) -> String {
    match value
        .checked_sub(1)
        .and_then(|i| PIECE_TYPES.get(i as usize))
    {
        Some(name) => (*name).to_string(),
        None => String::new(),
    }
}

pub(super) fn tint_type_to_v2168(name: &str) -> String {
    if name == "persona_hand" || name == "persona_hands" {
        return "hands".to_string();
    }
    name.strip_prefix("persona_").unwrap_or(name).to_string()
}

pub(super) fn tint_type_to_v1001(wire: &str) -> String {
    match wire {
        "hands" => "persona_hand".to_string(),
        "unsupported" => wire.to_string(),
        _ => format!("persona_{wire}"),
    }
}

pub(super) fn parse_hex_colour(text: &str) -> u32 {
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

pub(super) fn format_rgb_colour(packed: u32) -> String {
    let r = (packed >> 8) & 0xff;
    let g = (packed >> 16) & 0xff;
    let b = (packed >> 24) & 0xff;
    format!("#{r:02x}{g:02x}{b:02x}")
}

pub(super) fn format_argb_colour(packed: u32) -> String {
    if packed == 0 {
        return "#0".to_string();
    }
    let a = packed & 0xff;
    let r = (packed >> 8) & 0xff;
    let g = (packed >> 16) & 0xff;
    let b = (packed >> 24) & 0xff;
    format!("#{a:02x}{r:02x}{g:02x}{b:02x}")
}

pub(super) fn parse_uuid(text: &str) -> MceUuid {
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

pub(super) fn format_uuid(id: &MceUuid) -> String {
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
