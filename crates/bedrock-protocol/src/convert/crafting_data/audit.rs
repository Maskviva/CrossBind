use super::model::{Ctx, Emit, Ingredient, IngredientCount, Recipes, Unrepresentable};
use super::write::{encode_shaped, encode_shapeless, encode_smithing_transform};
use super::{DUMP_HEX_LIMIT, RECIPE_DUMP_DEFAULT, RESULT_SAMPLE_LIMIT, SKIP_SAMPLE_LIMIT};
use crate::diag;
use bedrock_codec::prelude::*;
use std::collections::HashMap;

#[derive(Default)]
pub(super) struct Skipped {
    count: usize,
    unknown_ids: Vec<i32>,
    out_of_range: Vec<&'static str>,
}

impl Skipped {
    pub(super) fn record(&mut self, why: Unrepresentable) {
        self.count += 1;
        match why {
            Unrepresentable::UnknownItemId(id) => {
                if !self.unknown_ids.contains(&id) && self.unknown_ids.len() < SKIP_SAMPLE_LIMIT {
                    self.unknown_ids.push(id);
                }
            }
            Unrepresentable::OutOfRange(what) => {
                if !self.out_of_range.contains(&what) && self.out_of_range.len() < SKIP_SAMPLE_LIMIT
                {
                    self.out_of_range.push(what);
                }
            }
        }
    }

    pub(super) fn describe(&self, total: usize, registry_len: usize) -> Option<String> {
        if self.count == 0 {
            return None;
        }
        let mut msg = format!("CraftingData: {} of {total} recipes dropped", self.count);
        if !self.unknown_ids.is_empty() {
            let ids: Vec<String> = self.unknown_ids.iter().map(i32::to_string).collect();
            msg.push_str(&format!(
                "; ingredient ids missing from the {registry_len}-entry item registry: {}",
                ids.join(", ")
            ));
            if self.unknown_ids.len() == SKIP_SAMPLE_LIMIT {
                msg.push_str(" (sample truncated)");
            }
        }
        if !self.out_of_range.is_empty() {
            msg.push_str(&format!(
                "; value too wide for the pre-2168 field: {}",
                self.out_of_range.join(", ")
            ));
        }
        Some(msg)
    }
}

#[derive(Default)]
pub(super) struct ResultAudit {
    checked: usize,
    empty_count: usize,
    unresolved_count: usize,
    empty: Vec<String>,
    unresolved: Vec<(String, i32)>,
}

impl ResultAudit {
    fn check(&mut self, recipe_id: &str, item: &Item, ids: &HashMap<i32, String>) {
        self.checked += 1;
        if item.network_id == 0 || item.count == 0 {
            self.empty_count += 1;
            if self.empty.len() < RESULT_SAMPLE_LIMIT {
                self.empty.push(recipe_id.to_owned());
            }
            return;
        }
        if !ids.contains_key(&item.network_id) {
            self.unresolved_count += 1;
            if self.unresolved.len() < RESULT_SAMPLE_LIMIT {
                self.unresolved
                    .push((recipe_id.to_owned(), item.network_id));
            }
        }
    }

    pub(super) fn describe(&self, registry_len: usize) -> Option<String> {
        if self.empty_count == 0 && self.unresolved_count == 0 {
            return None;
        }
        let mut msg = format!(
            "CraftingData: {} of {} recipe results are unusable as they arrive from \
             the server (results are passed through untouched, so this is upstream \
             of crossbind)",
            self.empty_count + self.unresolved_count,
            self.checked
        );
        if self.empty_count != 0 {
            msg.push_str(&format!(
                "; {} arrive empty (id 0 or stack size 0): {}",
                self.empty_count,
                self.empty.join(", ")
            ));
            if self.empty.len() < self.empty_count {
                msg.push_str(", …");
            }
        }
        if self.unresolved_count != 0 {
            let listed: Vec<String> = self
                .unresolved
                .iter()
                .map(|(name, id)| format!("{name}(id {id})"))
                .collect();
            msg.push_str(&format!(
                "; {} name an id absent from the {registry_len}-entry registry: {}",
                self.unresolved_count,
                listed.join(", ")
            ));
            if self.unresolved.len() < self.unresolved_count {
                msg.push_str(", …");
            }
        }
        Some(msg)
    }
}

pub(super) fn audit_results(r: &Recipes, ids: &HashMap<i32, String>) -> ResultAudit {
    let mut audit = ResultAudit::default();
    if ids.is_empty() {
        return audit;
    }
    for group in [&r.shaped, &r.shaped_chemistry] {
        for recipe in group.iter() {
            for item in &recipe.outputs {
                audit.check(&recipe.recipe_id, item, ids);
            }
        }
    }
    for group in [&r.shapeless, &r.user_data_shapeless, &r.shapeless_chemistry] {
        for recipe in group.iter() {
            for item in &recipe.outputs {
                audit.check(&recipe.recipe_id, item, ids);
            }
        }
    }
    for recipe in &r.smithing_transform {
        audit.check(&recipe.recipe_id, &recipe.result, ids);
    }
    audit
}

fn recipe_dump_names() -> Vec<String> {
    if !diag::enabled() {
        return Vec::new();
    }
    let Ok(raw) = std::env::var("CROSSBIND_RECIPE_DUMP") else {
        return Vec::new();
    };
    let trimmed = raw.trim();
    if trimmed.is_empty()
        || trimmed.eq_ignore_ascii_case("off")
        || trimmed.eq_ignore_ascii_case("none")
        || trimmed == "0"
    {
        return Vec::new();
    }
    let list = if trimmed == "1"
        || trimmed.eq_ignore_ascii_case("on")
        || trimmed.eq_ignore_ascii_case("default")
    {
        RECIPE_DUMP_DEFAULT
    } else {
        trimmed
    };
    list.split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
        .collect()
}

fn hex(bytes: &[u8]) -> String {
    let shown = bytes.len().min(DUMP_HEX_LIMIT);
    let parts: Vec<String> = bytes[..shown].iter().map(|b| format!("{b:02x}")).collect();
    let mut s = parts.join(" ");
    if bytes.len() > shown {
        s.push_str(" ...");
    }
    s
}

fn describe_ingredient(d: &IngredientCount) -> String {
    let what = match &d.what {
        Ingredient::Invalid => "invalid".to_owned(),
        Ingredient::Item { id, name, meta } => match (id, name) {
            (Some(i), _) => format!("id {i} meta {meta}"),
            (None, Some(n)) => format!("name {n} meta {meta}"),
            (None, None) => "empty".to_owned(),
        },
        Ingredient::MoLang {
            expression,
            version,
        } => format!("molang {expression:?} v{version}"),
        Ingredient::ItemTag { tag } => format!("tag {tag}"),
        Ingredient::ComplexAlias { name } => format!("alias {name}"),
    };
    format!("{what} x{}", d.count)
}

fn describe_item(it: &Item) -> String {
    format!(
        "id {} x{} aux {} block_runtime_id {} nbt {} B",
        it.network_id,
        it.count,
        it.aux_value,
        it.block_runtime_id,
        it.user_data.len()
    )
}

fn wanted(list: &[String], recipe_id: &str) -> bool {
    list.iter().any(|w| w == recipe_id)
}

fn dump_body(out: &mut Vec<String>, encoded: Result<Emit<Vec<u8>>>) {
    match encoded {
        Ok(Ok(body)) => out.push(format!("    wire {} B: {}", body.len(), hex(&body))),
        Ok(Err(why)) => out.push(format!("    wire: recipe dropped ({why:?})")),
        Err(err) => out.push(format!("    wire: re-encode failed ({err})")),
    }
}

pub(super) fn dump_recipes(r: &Recipes, ctx: &Ctx<'_>) -> Vec<String> {
    let list = recipe_dump_names();
    let mut out = Vec::new();
    if list.is_empty() {
        return out;
    }

    for (label, group) in [
        ("shaped", &r.shaped),
        ("shaped_chemistry", &r.shaped_chemistry),
    ] {
        for rec in group.iter() {
            if !wanted(&list, &rec.recipe_id) {
                continue;
            }
            out.push(format!(
                "recipe dump {} [{label} {}x{}] block={} priority={} symmetry={} \
                 unlock(present={} context={} ingredients={})",
                rec.recipe_id,
                rec.width,
                rec.height,
                rec.block,
                rec.priority,
                rec.assume_symmetry,
                rec.unlock.present,
                rec.unlock.context,
                rec.unlock.ingredients.len()
            ));
            for (n, ing) in rec.inputs.iter().enumerate() {
                out.push(format!("    in[{n}] {}", describe_ingredient(ing)));
            }
            for (n, item) in rec.outputs.iter().enumerate() {
                out.push(format!("    out[{n}] {}", describe_item(item)));
            }
            dump_body(&mut out, encode_shaped(rec, ctx));
        }
    }

    for (label, group) in [
        ("shapeless", &r.shapeless),
        ("user_data_shapeless", &r.user_data_shapeless),
        ("shapeless_chemistry", &r.shapeless_chemistry),
    ] {
        for rec in group.iter() {
            if !wanted(&list, &rec.recipe_id) {
                continue;
            }
            out.push(format!(
                "recipe dump {} [{label}] block={} priority={} \
                 unlock(present={} context={} ingredients={})",
                rec.recipe_id,
                rec.block,
                rec.priority,
                rec.unlock.present,
                rec.unlock.context,
                rec.unlock.ingredients.len()
            ));
            for (n, ing) in rec.inputs.iter().enumerate() {
                out.push(format!("    in[{n}] {}", describe_ingredient(ing)));
            }
            for (n, item) in rec.outputs.iter().enumerate() {
                out.push(format!("    out[{n}] {}", describe_item(item)));
            }
            dump_body(&mut out, encode_shapeless(rec, ctx));
        }
    }

    for rec in &r.smithing_transform {
        if !wanted(&list, &rec.recipe_id) {
            continue;
        }
        out.push(format!(
            "recipe dump {} [smithing_transform] block={}",
            rec.recipe_id, rec.block
        ));
        out.push(format!(
            "    template {}",
            describe_ingredient(&rec.template)
        ));
        out.push(format!("    base     {}", describe_ingredient(&rec.base)));
        out.push(format!(
            "    addition {}",
            describe_ingredient(&rec.addition)
        ));
        out.push(format!("    out[0] {}", describe_item(&rec.result)));
        dump_body(&mut out, encode_smithing_transform(rec, ctx));
    }

    for name in &list {
        if !out
            .iter()
            .any(|l| l.starts_with(&format!("recipe dump {name} ")))
        {
            out.push(format!(
                "recipe dump {name}: the server never sent a recipe with this id"
            ));
        }
    }

    out
}
