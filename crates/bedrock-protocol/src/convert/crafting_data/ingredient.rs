use super::model::{Ctx, Emit, Ingredient, IngredientCount, Unrepresentable};
use super::{
    DESCRIPTOR_AUX, DESC_COMPLEX_ALIAS, DESC_DEFAULT, DESC_DEFERRED, DESC_INVALID, DESC_ITEM_TAG,
    DESC_MOLANG,
};
use crate::item_remap;
use bedrock_codec::prelude::*;
use std::collections::HashMap;

pub(super) fn read_ingredient_slice(
    r: &mut Reader<'_>,
    ctx: &Ctx<'_>,
) -> Result<Vec<IngredientCount>> {
    let n = r.read_count()?;
    let mut v = Vec::with_capacity(n);
    for _ in 0..n {
        v.push(read_ingredient(r, ctx)?);
    }
    Ok(v)
}

pub(super) fn read_ingredient(r: &mut Reader<'_>, ctx: &Ctx<'_>) -> Result<IngredientCount> {
    let what = if ctx.to_v2168 {
        read_ingredient_v1001(r)?
    } else {
        read_ingredient_v2168(r)?
    };
    Ok(IngredientCount {
        what,
        count: r.read_varint()?,
    })
}

fn read_ingredient_v1001(r: &mut Reader<'_>) -> Result<Ingredient> {
    Ok(match r.read_u8()? {
        DESC_INVALID => Ingredient::Invalid,
        DESC_DEFAULT => {
            let id = i32::from(r.read_i16_le()?);
            let meta = if id != 0 {
                i32::from(r.read_i16_le()?)
            } else {
                0
            };
            Ingredient::Item {
                id: Some(id),
                name: None,
                meta,
            }
        }
        DESC_MOLANG => Ingredient::MoLang {
            expression: Str::read(r)?,
            version: i16::from(r.read_u8()?),
        },
        DESC_ITEM_TAG => Ingredient::ItemTag { tag: Str::read(r)? },
        DESC_DEFERRED => {
            let name = Str::read(r)?;
            Ingredient::Item {
                id: None,
                name: Some(name),
                meta: i32::from(r.read_i16_le()?),
            }
        }
        DESC_COMPLEX_ALIAS => Ingredient::ComplexAlias {
            name: Str::read(r)?,
        },
        other => {
            return Err(Error::BadDiscriminant {
                what: "item descriptor type",
                value: i64::from(other),
            })
        }
    })
}

fn read_ingredient_v2168(r: &mut Reader<'_>) -> Result<Ingredient> {
    let variant = r.read_uvarint()?;
    if variant == u32::from(DESC_INVALID) {
        r.read_varint()?;
        return Ok(Ingredient::Invalid);
    }
    if variant != u32::from(DESC_DEFAULT) {
        return Err(Error::BadDiscriminant {
            what: "item descriptor variant",
            value: i64::from(variant),
        });
    }
    let kind = Str::read(r)?;
    Ok(match kind.as_str() {
        "name" => Ingredient::Item {
            id: None,
            name: Some(Str::read(r)?),
            meta: r.read_varint()?,
        },
        "molang" => {
            let expression = Str::read(r)?;
            let _aux_value = r.read_varint()?;
            Ingredient::MoLang {
                expression,
                version: 0,
            }
        }
        "item_tag" => {
            let tag = Str::read(r)?;
            r.read_varint()?;
            Ingredient::ItemTag { tag }
        }
        _ => return Err(Error::Invalid("unknown item descriptor kind")),
    })
}

pub(super) fn write_ingredient_slice(
    w: &mut Writer,
    items: &[IngredientCount],
    ctx: &Ctx<'_>,
) -> Result<Emit<()>> {
    let mut body = Writer::new();
    for item in items {
        if let Err(why) = write_ingredient(&mut body, item, ctx)? {
            return Ok(Err(why));
        }
    }
    w.write_count(items.len());
    w.write_bytes(&body.into_vec());
    Ok(Ok(()))
}

pub(super) fn write_ingredient(
    w: &mut Writer,
    d: &IngredientCount,
    ctx: &Ctx<'_>,
) -> Result<Emit<()>> {
    let mut body = Writer::new();
    let outcome = if ctx.to_v2168 {
        write_ingredient_v2168(&mut body, &d.what, ctx.ids)
    } else {
        write_ingredient_v1001(&mut body, &d.what, ctx.names)
    };
    if let Err(why) = outcome {
        return Ok(Err(why));
    }
    w.write_bytes(&body.into_vec());
    w.write_varint(d.count);
    Ok(Ok(()))
}

fn write_ingredient_v2168(w: &mut Writer, d: &Ingredient, ids: &HashMap<i32, String>) -> Emit<()> {
    match d {
        Ingredient::Invalid => {
            w.write_uvarint(u32::from(DESC_INVALID));
            w.write_varint(DESCRIPTOR_AUX);
        }
        Ingredient::Item { id, name, meta } => {
            if name.is_none() && *id == Some(0) {
                w.write_uvarint(u32::from(DESC_INVALID));
                w.write_varint(DESCRIPTOR_AUX);
            } else {
                let resolved = match name {
                    Some(n) => n.clone(),
                    None => match id.and_then(|i| ids.get(&i)) {
                        Some(n) => n.clone(),
                        None => {
                            return Err(Unrepresentable::UnknownItemId(id.unwrap_or(0)));
                        }
                    },
                };
                w.write_uvarint(u32::from(DESC_DEFAULT));
                w.write_string("name");
                Str::write(w, &resolved);
                w.write_varint(*meta);
            }
        }
        Ingredient::MoLang {
            expression,
            version: _,
        } => {
            w.write_uvarint(u32::from(DESC_DEFAULT));
            w.write_string("molang");
            Str::write(w, expression);
            w.write_varint(0);
        }
        Ingredient::ItemTag { tag } => {
            w.write_uvarint(u32::from(DESC_DEFAULT));
            w.write_string("item_tag");
            Str::write(w, tag);
            w.write_varint(DESCRIPTOR_AUX);
        }
        Ingredient::ComplexAlias { name } => {
            w.write_uvarint(u32::from(DESC_DEFAULT));
            w.write_string("name");
            Str::write(w, name);
            w.write_varint(DESCRIPTOR_AUX);
        }
    }
    Ok(())
}

pub(super) fn write_ingredient_v1001(
    w: &mut Writer,
    d: &Ingredient,
    names: &HashMap<String, i32>,
) -> Emit<()> {
    match d {
        Ingredient::Invalid => w.write_u8(DESC_INVALID),
        Ingredient::Item { id, name, meta } => {
            let resolved = match id {
                Some(i) => Some(*i),
                None => name.as_ref().and_then(|n| names.get(n)).copied(),
            };
            match resolved {
                Some(i) => {
                    w.write_u8(DESC_DEFAULT);
                    w.write_i16_le(
                        i16::try_from(i).map_err(|_| Unrepresentable::OutOfRange("item id"))?,
                    );
                    if i != 0 {
                        w.write_i16_le(
                            i16::try_from(*meta)
                                .map_err(|_| Unrepresentable::OutOfRange("item meta"))?,
                        );
                    }
                }
                None => {
                    let Some(n) = name.as_ref() else {
                        return Err(Unrepresentable::UnknownItemId(0));
                    };
                    w.write_u8(DESC_DEFERRED);
                    Str::write(w, n);
                    w.write_i16_le(
                        i16::try_from(*meta)
                            .map_err(|_| Unrepresentable::OutOfRange("item meta"))?,
                    );
                }
            }
        }
        Ingredient::MoLang {
            expression,
            version,
        } => {
            w.write_u8(DESC_MOLANG);
            Str::write(w, expression);
            w.write_u8(
                u8::try_from(*version)
                    .map_err(|_| Unrepresentable::OutOfRange("molang version"))?,
            );
        }
        Ingredient::ItemTag { tag } => {
            w.write_u8(DESC_ITEM_TAG);
            Str::write(w, tag);
        }
        Ingredient::ComplexAlias { name } => {
            w.write_u8(DESC_COMPLEX_ALIAS);
            Str::write(w, name);
        }
    }
    Ok(())
}

pub(super) fn read_output_slice(r: &mut Reader<'_>, ctx: &Ctx<'_>) -> Result<Vec<Item>> {
    let n = r.read_count()?;
    let mut v = Vec::with_capacity(n);
    for _ in 0..n {
        v.push(read_output(r, ctx)?);
    }
    Ok(v)
}

pub(super) fn read_output(r: &mut Reader<'_>, ctx: &Ctx<'_>) -> Result<Item> {
    if ctx.to_v2168 {
        NetworkItemInstanceDescriptor::read(r)
    } else {
        NetworkItemInstanceDescriptorV2168::read(r)
    }
}

pub(super) fn write_output_slice(w: &mut Writer, items: &[Item], ctx: &Ctx<'_>) {
    w.write_count(items.len());
    for item in items {
        write_output(w, item, ctx);
    }
}

pub(super) fn write_output(w: &mut Writer, item: &Item, ctx: &Ctx<'_>) {
    let renumbered = if ctx.to_v2168 {
        item_remap::to_client(item.network_id)
    } else {
        item_remap::to_server(item.network_id)
    };

    if renumbered == item.network_id {
        if ctx.to_v2168 {
            NetworkItemInstanceDescriptorV2168::write(w, item);
        } else {
            NetworkItemInstanceDescriptor::write(w, item);
        }
        return;
    }

    let mut moved = item.clone();
    moved.network_id = renumbered;
    if ctx.to_v2168 {
        NetworkItemInstanceDescriptorV2168::write(w, &moved);
    } else {
        NetworkItemInstanceDescriptor::write(w, &moved);
    }
}
