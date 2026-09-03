mod audit;
mod ingredient;
mod model;
mod read;
mod write;

use crate::connection::ConnState;
use audit::{audit_results, dump_recipes, ResultAudit};
use bedrock_codec::prelude::*;
use model::{Ctx, Report};
use read::{read_v1001_list, read_v2168_vectors};
use std::collections::HashMap;
use write::{write_v1001_list, write_v2168_vectors};

const TYPE_SHAPELESS: i32 = 0;

const TYPE_SHAPED: i32 = 1;

const TYPE_MULTI: i32 = 4;

const TYPE_USER_DATA_SHAPELESS: i32 = 5;

const TYPE_SHAPELESS_CHEMISTRY: i32 = 6;

const TYPE_SHAPED_CHEMISTRY: i32 = 7;

const TYPE_SMITHING_TRANSFORM: i32 = 8;

const TYPE_SMITHING_TRIM: i32 = 9;

const DESC_INVALID: u8 = 0;

const DESC_DEFAULT: u8 = 1;

const DESC_MOLANG: u8 = 2;

const DESC_ITEM_TAG: u8 = 3;

const DESC_DEFERRED: u8 = 4;

const DESC_COMPLEX_ALIAS: u8 = 5;

const DESCRIPTOR_AUX: i32 = 32767;

const MAX_GRID: i64 = 1024;

pub(crate) fn crafting_data(
    w: &mut PacketWrapper,
    state: &mut ConnState,
    to_v2168: bool,
) -> Result<bool> {
    let names = std::mem::take(&mut state.item_ids);
    let ids = std::mem::take(&mut state.item_names);
    let registry_len = if to_v2168 { ids.len() } else { names.len() };
    let result = translate(w, &names, &ids, to_v2168);
    state.item_ids = names;
    state.item_names = ids;

    match result {
        Ok(report) => {
            if let Some(notice) = report.skipped.describe(report.total, registry_len) {
                state.notices.push(notice);
            }
            if let Some(notice) = report.results.describe(registry_len) {
                state.notices.push(notice);
            }
            state.notices.extend(report.dumps);
            Ok(true)
        }
        Err(err) => {
            state
                .notices
                .push(format!("CraftingData: cancelled, cannot decode: {err}"));
            Ok(false)
        }
    }
}

fn translate(
    w: &mut PacketWrapper,
    names: &HashMap<String, i32>,
    ids: &HashMap<i32, String>,
    to_v2168: bool,
) -> Result<Report> {
    let ctx = Ctx {
        to_v2168,
        names,
        ids,
    };
    let recipes = if to_v2168 {
        read_v1001_list(w.reader(), &ctx)?
    } else {
        read_v2168_vectors(w.reader(), &ctx)?
    };

    let tail = w.reader().read_remaining().to_vec();

    let audit = if to_v2168 {
        audit_results(&recipes, ids)
    } else {
        ResultAudit::default()
    };

    let mut out = Writer::new();
    let mut report = if to_v2168 {
        write_v2168_vectors(&mut out, &recipes, &ctx)?
    } else {
        write_v1001_list(&mut out, &recipes, &ctx)?
    };
    report.results = audit;
    report.dumps = dump_recipes(&recipes, &ctx);
    out.write_bytes(&tail);

    w.writer().write_bytes(&out.into_vec());
    Ok(report)
}

const SKIP_SAMPLE_LIMIT: usize = 12;

const RESULT_SAMPLE_LIMIT: usize = 16;

const DUMP_HEX_LIMIT: usize = 220;

const RECIPE_DUMP_DEFAULT: &str = concat!(
    "minecraft:smithing_netherite_sword,",
    "minecraft:smithing_netherite_axe,",
    "minecraft:warped_door,",
    "minecraft:crimson_door"
);

#[cfg(test)]
mod tests;
