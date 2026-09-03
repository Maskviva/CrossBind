use crate::connection::ConnState;
use crate::diag;
use crate::item_remap;
use bedrock_codec::prelude::*;
use std::collections::{HashMap, HashSet};

pub(crate) fn cache_item_registry(w: &mut PacketWrapper, state: &mut ConnState) -> Result<()> {
    let body = w.reader().read_remaining().to_vec();

    let probe = if diag::enabled() {
        item_probe_names()
    } else {
        Vec::new()
    };
    let mut probe_lines: Vec<String> = Vec::new();
    let mut component_based = 0usize;
    let mut by_version = [0usize; 4];
    let mut collisions: Vec<String> = Vec::new();
    let mut collision_count = 0usize;
    let dump_path = item_dump_path();
    let mut dump_rows: Vec<String> = Vec::new();

    let mut r = Reader::new(&body);
    let count = match r.read_count() {
        Ok(count) => count,
        Err(e) => {
            w.writer().write_bytes(&body);
            state.notices.push(format!(
                "item registry: cannot read entry count ({e}); forwarded unchanged"
            ));
            return Ok(());
        }
    };
    let mut names = HashMap::with_capacity(count);
    let mut ids = HashMap::with_capacity(count);
    let mut entries: Vec<(String, i32, bool, i32, Vec<u8>)> = Vec::with_capacity(count.min(8192));
    for _ in 0..count {
        let name = Str::read(&mut r)?;
        let network_id = r.read_i16_le()? as i32;
        let is_component_based = r.read_bool()?;
        let item_version = r.read_varint()?;
        let component = NamedCompoundTag::read(&mut r)?;
        let nbt_len = component.len();

        if is_component_based {
            component_based += 1;
        }
        by_version[match item_version {
            0 => 0,
            1 => 1,
            2 => 2,
            _ => 3,
        }] += 1;

        if probe.iter().any(|p| p == &name) {
            probe_lines.push(format!(
                "item probe: {name} present id={network_id} component_based={is_component_based} \
                 item_version={} component_nbt={nbt_len} B",
                item_version_label(item_version)
            ));
        }

        if dump_path.is_some() {
            dump_rows.push(format!(
                "{name}\t{network_id}\t{is_component_based}\t{}\t{nbt_len}",
                item_version_label(item_version)
            ));
        }

        entries.push((
            name.clone(),
            network_id,
            is_component_based,
            item_version,
            component,
        ));

        let displaced_name = ids.insert(network_id, name.clone());
        let displaced_id = names.insert(name.clone(), network_id);
        if let Some(prev) = displaced_name {
            collision_count += 1;
            if collisions.len() < COLLISION_SAMPLE_LIMIT {
                collisions.push(format!("id {network_id} claimed by both {prev} and {name}"));
            }
        }
        if let Some(prev_id) = displaced_id {
            collision_count += 1;
            if collisions.len() < COLLISION_SAMPLE_LIMIT {
                collisions.push(format!(
                    "{name} listed twice, as id {prev_id} and id {network_id}"
                ));
            }
        }
    }
    let leftover = r.remaining();
    if leftover != 0 {
        state.notices.push(format!(
            "WARNING: item registry has {leftover} trailing bytes after {count} entries; \
             item ids may be wrong"
        ));
    }

    if diag::enabled() {
        state.notices.push(format!(
            "item registry cached: {count} entries ({component_based} component-based; \
             item_version legacy={} data_driven={} none={} other={})",
            by_version[0], by_version[1], by_version[2], by_version[3]
        ));
    }

    if collision_count != 0 {
        state.notices.push(format!(
            "WARNING: the server's item registry has {collision_count} name/id collisions; \
             one item per collision has no definition behind it{}",
            if diag::enabled() {
                format!(" ({})", collisions.join("; "))
            } else {
                String::new()
            }
        ));
    }

    if !probe.is_empty() {
        let missing: Vec<&str> = probe
            .iter()
            .filter(|p| !names.contains_key(p.as_str()))
            .map(String::as_str)
            .collect();
        state.notices.extend(probe_lines);
        if !missing.is_empty() {
            state.notices.push(format!(
                "item probe: {} of {} probed names are absent from the registry the \
                 server sent: {}",
                missing.len(),
                probe.len(),
                missing.join(", ")
            ));
        }
    }

    if let Some(path) = dump_path {
        let mut contents =
            String::from("name\tid\tcomponent_based\titem_version\tcomponent_nbt_bytes\n");
        for row in &dump_rows {
            contents.push_str(row);
            contents.push('\n');
        }
        match std::fs::write(&path, contents) {
            Ok(()) => state
                .notices
                .push(format!("item registry: dumped {count} entries to {path}")),
            Err(e) => state.notices.push(format!(
                "item registry: could not write dump to {path}: {e}"
            )),
        }
    }

    emit_registry(w, state, &body, &entries);

    state.item_ids = names;
    state.item_names = ids;
    Ok(())
}

fn emit_registry(
    w: &mut PacketWrapper,
    state: &mut ConnState,
    original: &[u8],
    entries: &[(String, i32, bool, i32, Vec<u8>)],
) {
    let pairs: Vec<(String, i32)> = entries
        .iter()
        .map(|(name, id, _, _, _)| (name.clone(), *id))
        .collect();
    let Some((remap, first_time)) = item_remap::build_once(&pairs) else {
        w.writer().write_bytes(original);
        return;
    };

    let mut out = Writer::new();
    out.write_count(entries.len());
    let mut changed = 0usize;
    let mut seen: HashSet<i32> = HashSet::with_capacity(entries.len());
    let mut duplicates = 0usize;
    for (name, server_id, component_based, item_version, component) in entries {
        let client_id = remap.to_client(*server_id);
        if client_id != *server_id {
            changed += 1;
        }
        if !seen.insert(client_id) {
            duplicates += 1;
        }
        Str::write(&mut out, name);
        out.write_i16_le(client_id as i16);
        out.write_bool(*component_based);
        out.write_varint(*item_version);
        NamedCompoundTag::write(&mut out, component);
    }
    w.writer().write_bytes(&out.into_vec());

    if first_time {
        state.notices.push(format!(
            "item ids: {changed} of {} renumbered for the client",
            entries.len()
        ));
        if diag::enabled() {
            state
                .notices
                .push(format!("item id mapping: {}", remap.report()));
        }
        let unplaceable = remap.report().unplaceable;
        if unplaceable != 0 {
            state.notices.push(format!(
                "WARNING: {unplaceable} server items could not be given a free id and \
                 will not appear correctly"
            ));
        }
    }

    if duplicates != 0 && first_time {
        state.notices.push(format!(
            "WARNING: {duplicates} item ids collide after renumbering; the client will \
             lose one item per collision"
        ));
    }
}

fn item_probe_names() -> Vec<String> {
    let raw = std::env::var("CROSSBIND_ITEM_PROBE").unwrap_or_default();
    let trimmed = raw.trim();
    if trimmed.eq_ignore_ascii_case("off") || trimmed.eq_ignore_ascii_case("none") {
        return Vec::new();
    }
    let list = if trimmed.is_empty() {
        ITEM_PROBE_DEFAULT
    } else {
        trimmed
    };
    list.split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
        .collect()
}

fn item_version_label(v: i32) -> &'static str {
    match v {
        0 => "LEGACY",
        1 => "DATA_DRIVEN",
        2 => "NONE",
        _ => "unknown",
    }
}

fn item_dump_path() -> Option<String> {
    std::env::var("CROSSBIND_ITEM_DUMP")
        .ok()
        .filter(|s| !s.trim().is_empty())
}

const ITEM_PROBE_DEFAULT: &str = concat!(
    "minecraft:netherite_ingot,",
    "minecraft:netherite_sword,",
    "minecraft:netherite_pickaxe,",
    "minecraft:netherite_shovel,",
    "minecraft:netherite_hoe,",
    "minecraft:netherite_helmet,",
    "minecraft:netherite_chestplate,",
    "minecraft:netherite_leggings,",
    "minecraft:netherite_boots,",
    "minecraft:warped_door,",
    "minecraft:warped_sign,",
    "minecraft:netherite_axe,",
    "minecraft:crimson_door,",
    "minecraft:crimson_sign,",
    "minecraft:netherite_block,",
    "minecraft:netherite_scrap"
);

const COLLISION_SAMPLE_LIMIT: usize = 8;
