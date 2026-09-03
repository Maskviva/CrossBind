use super::action::write_stack_u16;
use bedrock_codec::prelude::*;
use std::collections::HashMap;

pub(super) fn recipe_ingredient(
    r: &mut Reader<'_>,
    w: &mut Writer,
    to_v2168: bool,
    names: &HashMap<String, i32>,
    ids: &HashMap<i32, String>,
) -> Result<bool> {
    if to_v2168 {
        let internal_type = r.read_u8()?;
        match internal_type {
            INTERNAL_TYPE_INVALID => {
                let stack = r.read_varint()?;
                w.write_u8(RECIPE_DESC_EMPTY);
                write_stack_u16(w, stack)?;
            }
            INTERNAL_TYPE_DEFAULT => {
                let id = i32::from(r.read_i16_le()?);
                let aux = if id != 0 {
                    i32::from(r.read_i16_le()?)
                } else {
                    0
                };
                let stack = r.read_varint()?;
                if id == 0 {
                    w.write_u8(RECIPE_DESC_EMPTY);
                } else {
                    let Some(name) = ids.get(&id) else {
                        return Ok(false);
                    };
                    w.write_u8(RECIPE_DESC_ITEM_NAME);
                    Str::write(w, name);
                    w.write_varint(aux);
                }
                write_stack_u16(w, stack)?;
            }
            INTERNAL_TYPE_MOLANG => {
                let expression = Str::read(r)?;
                let version = i16::from(r.read_u8()?);
                let stack = r.read_varint()?;
                w.write_u8(RECIPE_DESC_MOLANG);
                Str::write(w, &expression);
                w.write_i16_le(version);
                write_stack_u16(w, stack)?;
            }
            INTERNAL_TYPE_ITEM_TAG => {
                let tag = Str::read(r)?;
                let stack = r.read_varint()?;
                w.write_u8(RECIPE_DESC_ITEM_TAG);
                Str::write(w, &tag);
                write_stack_u16(w, stack)?;
            }
            INTERNAL_TYPE_DEFERRED => {
                let name = Str::read(r)?;
                let aux = i32::from(r.read_i16_le()?);
                let stack = r.read_varint()?;
                w.write_u8(RECIPE_DESC_ITEM_NAME);
                Str::write(w, &name);
                w.write_varint(aux);
                write_stack_u16(w, stack)?;
            }
            INTERNAL_TYPE_COMPLEX_ALIAS => {
                let name = Str::read(r)?;
                let stack = r.read_varint()?;
                w.write_u8(RECIPE_DESC_ITEM_NAME);
                Str::write(w, &name);
                w.write_varint(0);
                write_stack_u16(w, stack)?;
            }
            other => {
                return Err(Error::BadDiscriminant {
                    what: "item descriptor internal type",
                    value: i64::from(other),
                });
            }
        }
    } else {
        let dt = r.read_u8()?;
        match dt {
            RECIPE_DESC_EMPTY => {
                let stack = r.read_u16_le()?;
                w.write_u8(INTERNAL_TYPE_INVALID);
                w.write_varint(i32::from(stack));
            }
            RECIPE_DESC_ITEM_NAME => {
                let full_name = Str::read(r)?;
                let aux = r.read_varint()?;
                let stack = r.read_u16_le()?;
                match names.get(&full_name).copied() {
                    Some(0) => {
                        w.write_u8(INTERNAL_TYPE_INVALID);
                    }
                    Some(id) => {
                        let id16 = i16::try_from(id)
                            .map_err(|_| Error::Invalid("item registry id exceeds i16 range"))?;
                        w.write_u8(INTERNAL_TYPE_DEFAULT);
                        w.write_i16_le(id16);
                        let aux16 = i16::try_from(aux).unwrap_or(0);
                        w.write_i16_le(aux16);
                    }
                    None => {
                        return Ok(false);
                    }
                }
                w.write_varint(i32::from(stack));
            }
            RECIPE_DESC_MOLANG => {
                let expression = Str::read(r)?;
                let version = r.read_i16_le()?;
                let stack = r.read_u16_le()?;
                let version_u8 = u8::try_from(version)
                    .map_err(|_| Error::Invalid("molang version exceeds u8 range"))?;
                w.write_u8(INTERNAL_TYPE_MOLANG);
                Str::write(w, &expression);
                w.write_u8(version_u8);
                w.write_varint(i32::from(stack));
            }
            RECIPE_DESC_ITEM_TAG => {
                let tag = Str::read(r)?;
                let stack = r.read_u16_le()?;
                w.write_u8(INTERNAL_TYPE_ITEM_TAG);
                Str::write(w, &tag);
                w.write_varint(i32::from(stack));
            }
            other => {
                return Err(Error::BadDiscriminant {
                    what: "recipe ingredient descriptor variant",
                    value: i64::from(other),
                });
            }
        }
    }
    Ok(true)
}

const RECIPE_DESC_EMPTY: u8 = 0;

pub(super) const RECIPE_DESC_ITEM_NAME: u8 = 1;

pub(super) const RECIPE_DESC_MOLANG: u8 = 2;

const RECIPE_DESC_ITEM_TAG: u8 = 3;

pub(super) const INTERNAL_TYPE_INVALID: u8 = 0;

pub(super) const INTERNAL_TYPE_DEFAULT: u8 = 1;

pub(super) const INTERNAL_TYPE_MOLANG: u8 = 2;

const INTERNAL_TYPE_ITEM_TAG: u8 = 3;

const INTERNAL_TYPE_DEFERRED: u8 = 4;

pub(super) const INTERNAL_TYPE_COMPLEX_ALIAS: u8 = 5;

pub(super) const DESCRIPTOR_INVALID: u32 = 0;

pub(super) const DESCRIPTOR_DEFAULT: u32 = 1;
