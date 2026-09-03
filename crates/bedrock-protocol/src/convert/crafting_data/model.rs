use super::audit::{ResultAudit, Skipped};
use bedrock_codec::prelude::*;
use std::collections::HashMap;

pub(super) struct Report {
    pub(super) total: usize,
    pub(super) skipped: Skipped,
    pub(super) results: ResultAudit,
    pub(super) dumps: Vec<String>,
}

pub(super) struct Ctx<'a> {
    pub(super) to_v2168: bool,
    pub(super) names: &'a HashMap<String, i32>,
    pub(super) ids: &'a HashMap<i32, String>,
}

#[derive(Default)]
pub(super) struct Recipes {
    pub(super) shaped: Vec<Shaped>,
    pub(super) shapeless: Vec<Shapeless>,
    pub(super) multi: Vec<Multi>,
    pub(super) user_data_shapeless: Vec<Shapeless>,
    pub(super) shapeless_chemistry: Vec<Shapeless>,
    pub(super) shaped_chemistry: Vec<Shaped>,
    pub(super) smithing_transform: Vec<SmithingTransform>,
    pub(super) smithing_trim: Vec<SmithingTrim>,
}

impl Recipes {
    pub(super) fn total(&self) -> usize {
        self.shaped.len()
            + self.shapeless.len()
            + self.multi.len()
            + self.user_data_shapeless.len()
            + self.shapeless_chemistry.len()
            + self.shaped_chemistry.len()
            + self.smithing_transform.len()
            + self.smithing_trim.len()
    }
}

pub(super) struct Shapeless {
    pub(super) recipe_id: String,
    pub(super) inputs: Vec<IngredientCount>,
    pub(super) outputs: Vec<Item>,
    pub(super) uuid: MceUuid,
    pub(super) block: String,
    pub(super) priority: i32,
    pub(super) unlock: Unlock,
    pub(super) network_id: u32,
}

pub(super) struct Shaped {
    pub(super) recipe_id: String,
    pub(super) width: i32,
    pub(super) height: i32,
    pub(super) inputs: Vec<IngredientCount>,
    pub(super) outputs: Vec<Item>,
    pub(super) uuid: MceUuid,
    pub(super) block: String,
    pub(super) priority: i32,
    pub(super) assume_symmetry: bool,
    pub(super) unlock: Unlock,
    pub(super) network_id: u32,
}

pub(super) struct Multi {
    pub(super) uuid: MceUuid,
    pub(super) network_id: u32,
}

pub(super) struct SmithingTransform {
    pub(super) recipe_id: String,
    pub(super) template: IngredientCount,
    pub(super) base: IngredientCount,
    pub(super) addition: IngredientCount,
    pub(super) result: Item,
    pub(super) block: String,
    pub(super) network_id: u32,
}

pub(super) struct SmithingTrim {
    pub(super) recipe_id: String,
    pub(super) template: IngredientCount,
    pub(super) base: IngredientCount,
    pub(super) addition: IngredientCount,
    pub(super) block: String,
    pub(super) network_id: u32,
}

pub(super) struct Unlock {
    pub(super) present: bool,
    pub(super) context: i32,
    pub(super) has_ingredients: bool,
    pub(super) ingredients: Vec<IngredientCount>,
}

pub(super) struct IngredientCount {
    pub(super) what: Ingredient,
    pub(super) count: i32,
}

pub(super) enum Ingredient {
    Invalid,
    Item {
        id: Option<i32>,
        name: Option<String>,
        meta: i32,
    },
    MoLang {
        expression: String,
        version: i16,
    },
    ItemTag {
        tag: String,
    },
    ComplexAlias {
        name: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Unrepresentable {
    UnknownItemId(i32),
    OutOfRange(&'static str),
}

pub(super) type Emit<T> = std::result::Result<T, Unrepresentable>;
