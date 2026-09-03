use crate::connection::ConnState;
use bedrock_codec::prelude::*;
use std::collections::HashMap;

use super::crafting_data;
use super::ingredient::write_ingredient_v1001;
use super::model::Ingredient;
use super::{
    DESC_COMPLEX_ALIAS, DESC_DEFAULT, DESC_DEFERRED, DESC_MOLANG, TYPE_MULTI, TYPE_SHAPED,
    TYPE_SHAPED_CHEMISTRY, TYPE_SHAPELESS, TYPE_SHAPELESS_CHEMISTRY, TYPE_SMITHING_TRANSFORM,
    TYPE_SMITHING_TRIM, TYPE_USER_DATA_SHAPELESS,
};

mod misc;
mod model;
