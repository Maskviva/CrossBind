use crate::connection::ConnState;
use bedrock_codec::prelude::*;
use std::collections::HashMap;

use super::action::{
    action_id, action_variant, ACTION_MAX, ACTION_PLACE_IN_CONTAINER, ACTION_TAKE_OUT_CONTAINER,
};
use super::descriptor::{
    DESCRIPTOR_DEFAULT, INTERNAL_TYPE_COMPLEX_ALIAS, INTERNAL_TYPE_DEFAULT, INTERNAL_TYPE_INVALID,
    INTERNAL_TYPE_MOLANG, RECIPE_DESC_ITEM_NAME,
};
use super::item_stack_request;
use super::item_stack_response;
use super::response::RESPONSE_STATUS_OK;
use crate::convert::cache_item_registry;

use action::{v1001_craft_recipe_auto_default, v2168_pull_with_result};
use descriptor::{v1001_craft_recipe_auto_complex_alias, v1001_craft_recipe_auto_molang};
use misc::{empty_tables, stone_tables};

mod action;
mod descriptor;
mod misc;
mod registry;
mod response;
