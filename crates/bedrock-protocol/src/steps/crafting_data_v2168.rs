use std::collections::HashMap;

use bedrock_codec::prelude::*;

use crate::connection::ConnState;

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
    let result = translate(w, &names, &ids, to_v2168);
    state.item_ids = names;
    state.item_names = ids;

    match result {
        Ok(report) => {
            if report.skipped > 0 {
                state.notices.push(format!(
                    "CraftingData: {} of {} recipes dropped (ingredient not in item registry)",
                    report.skipped, report.total,
                ));
            }
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

struct Report {
    total: usize,
    skipped: usize,
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

    let mut out = Writer::new();
    let report = if to_v2168 {
        write_v2168_vectors(&mut out, &recipes, &ctx)?
    } else {
        write_v1001_list(&mut out, &recipes, &ctx)?
    };
    out.write_bytes(&tail);

    w.writer().write_bytes(&out.into_vec());
    Ok(report)
}

struct Ctx<'a> {
    to_v2168: bool,
    names: &'a HashMap<String, i32>,
    ids: &'a HashMap<i32, String>,
}

#[derive(Default)]
struct Recipes {
    shaped: Vec<Shaped>,
    shapeless: Vec<Shapeless>,
    multi: Vec<Multi>,
    user_data_shapeless: Vec<Shapeless>,
    shapeless_chemistry: Vec<Shapeless>,
    shaped_chemistry: Vec<Shaped>,
    smithing_transform: Vec<SmithingTransform>,
    smithing_trim: Vec<SmithingTrim>,
}

impl Recipes {
    fn total(&self) -> usize {
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

struct Shapeless {
    recipe_id: String,
    inputs: Vec<IngredientCount>,
    outputs: Vec<Item>,
    uuid: MceUuid,
    block: String,
    priority: i32,
    unlock: Unlock,
    network_id: u32,
}

struct Shaped {
    recipe_id: String,
    width: i32,
    height: i32,
    inputs: Vec<IngredientCount>,
    outputs: Vec<Item>,
    uuid: MceUuid,
    block: String,
    priority: i32,
    assume_symmetry: bool,
    unlock: Unlock,
    network_id: u32,
}

struct Multi {
    uuid: MceUuid,
    network_id: u32,
}

struct SmithingTransform {
    recipe_id: String,
    template: IngredientCount,
    base: IngredientCount,
    addition: IngredientCount,
    result: Item,
    block: String,
    network_id: u32,
}

struct SmithingTrim {
    recipe_id: String,
    template: IngredientCount,
    base: IngredientCount,
    addition: IngredientCount,
    block: String,
    network_id: u32,
}

struct Unlock {
    present: bool,
    context: i32,
    has_ingredients: bool,
    ingredients: Vec<IngredientCount>,
}

struct IngredientCount {
    what: Ingredient,
    count: i32,
}

enum Ingredient {
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

struct Unrepresentable;

type Emit<T> = std::result::Result<T, Unrepresentable>;

fn read_v1001_list(r: &mut Reader<'_>, ctx: &Ctx<'_>) -> Result<Recipes> {
    let mut out = Recipes::default();
    let count = r.read_count()?;
    for _ in 0..count {
        match r.read_varint()? {
            TYPE_SHAPELESS => out.shapeless.push(read_shapeless(r, ctx)?),
            TYPE_SHAPED => out.shaped.push(read_shaped(r, ctx)?),
            TYPE_MULTI => out.multi.push(read_multi(r)?),
            TYPE_USER_DATA_SHAPELESS => out.user_data_shapeless.push(read_shapeless(r, ctx)?),
            TYPE_SHAPELESS_CHEMISTRY => out.shapeless_chemistry.push(read_shapeless(r, ctx)?),
            TYPE_SHAPED_CHEMISTRY => out.shaped_chemistry.push(read_shaped(r, ctx)?),
            TYPE_SMITHING_TRANSFORM => {
                out.smithing_transform.push(read_smithing_transform(r, ctx)?)
            }
            TYPE_SMITHING_TRIM => out.smithing_trim.push(read_smithing_trim(r, ctx)?),
            other => {
                return Err(Error::BadDiscriminant {
                    what: "crafting data recipe type",
                    value: other as i64,
                })
            }
        }
    }
    Ok(out)
}

fn read_v2168_vectors(r: &mut Reader<'_>, ctx: &Ctx<'_>) -> Result<Recipes> {
    Ok(Recipes {
        shaped: read_vec(r, ctx, read_shaped)?,
        shapeless: read_vec(r, ctx, read_shapeless)?,
        multi: {
            let n = r.read_count()?;
            let mut v = Vec::with_capacity(n);
            for _ in 0..n {
                v.push(read_multi(r)?);
            }
            v
        },
        user_data_shapeless: read_vec(r, ctx, read_shapeless)?,
        shapeless_chemistry: read_vec(r, ctx, read_shapeless)?,
        shaped_chemistry: read_vec(r, ctx, read_shaped)?,
        smithing_transform: read_vec(r, ctx, read_smithing_transform)?,
        smithing_trim: read_vec(r, ctx, read_smithing_trim)?,
    })
}

fn read_vec<T>(
    r: &mut Reader<'_>,
    ctx: &Ctx<'_>,
    each: fn(&mut Reader<'_>, &Ctx<'_>) -> Result<T>,
) -> Result<Vec<T>> {
    let n = r.read_count()?;
    let mut v = Vec::with_capacity(n);
    for _ in 0..n {
        v.push(each(r, ctx)?);
    }
    Ok(v)
}

fn read_shapeless(r: &mut Reader<'_>, ctx: &Ctx<'_>) -> Result<Shapeless> {
    Ok(Shapeless {
        recipe_id: Str::read(r)?,
        inputs: read_ingredient_slice(r, ctx)?,
        outputs: read_output_slice(r, ctx)?,
        uuid: Uuid::read(r)?,
        block: Str::read(r)?,
        priority: r.read_varint()?,
        unlock: read_unlock(r, ctx)?,
        network_id: r.read_uvarint()?,
    })
}

fn read_shaped(r: &mut Reader<'_>, ctx: &Ctx<'_>) -> Result<Shaped> {
    let recipe_id = Str::read(r)?;
    let width = r.read_varint()?;
    let height = r.read_varint()?;
    let inputs = if ctx.to_v2168 {
        let cells = i64::from(width) * i64::from(height);
        if !(0..=MAX_GRID).contains(&cells) {
            return Err(Error::Invalid("shaped recipe grid out of range"));
        }
        let mut v = Vec::with_capacity(cells as usize);
        for _ in 0..cells {
            v.push(read_ingredient(r, ctx)?);
        }
        v
    } else {
        read_ingredient_slice(r, ctx)?
    };
    Ok(Shaped {
        recipe_id,
        width,
        height,
        inputs,
        outputs: read_output_slice(r, ctx)?,
        uuid: Uuid::read(r)?,
        block: Str::read(r)?,
        priority: r.read_varint()?,
        assume_symmetry: r.read_bool()?,
        unlock: read_unlock(r, ctx)?,
        network_id: r.read_uvarint()?,
    })
}

fn read_multi(r: &mut Reader<'_>) -> Result<Multi> {
    Ok(Multi {
        uuid: Uuid::read(r)?,
        network_id: r.read_uvarint()?,
    })
}

fn read_smithing_transform(r: &mut Reader<'_>, ctx: &Ctx<'_>) -> Result<SmithingTransform> {
    Ok(SmithingTransform {
        recipe_id: Str::read(r)?,
        template: read_ingredient(r, ctx)?,
        base: read_ingredient(r, ctx)?,
        addition: read_ingredient(r, ctx)?,
        result: read_output(r, ctx)?,
        block: Str::read(r)?,
        network_id: r.read_uvarint()?,
    })
}

fn read_smithing_trim(r: &mut Reader<'_>, ctx: &Ctx<'_>) -> Result<SmithingTrim> {
    Ok(SmithingTrim {
        recipe_id: Str::read(r)?,
        template: read_ingredient(r, ctx)?,
        base: read_ingredient(r, ctx)?,
        addition: read_ingredient(r, ctx)?,
        block: Str::read(r)?,
        network_id: r.read_uvarint()?,
    })
}

fn read_unlock(r: &mut Reader<'_>, ctx: &Ctx<'_>) -> Result<Unlock> {
    if ctx.to_v2168 {
        let context = i32::from(r.read_u8()?);
        let has_ingredients = context == 0;
        let ingredients = if has_ingredients {
            read_ingredient_slice(r, ctx)?
        } else {
            Vec::new()
        };
        return Ok(Unlock {
            present: true,
            context,
            has_ingredients,
            ingredients,
        });
    }
    if !r.read_bool()? {
        return Ok(Unlock {
            present: false,
            context: 0,
            has_ingredients: false,
            ingredients: Vec::new(),
        });
    }
    let context = r.read_varint()?;
    let has_ingredients = r.read_bool()?;
    let ingredients = if has_ingredients {
        read_ingredient_slice(r, ctx)?
    } else {
        Vec::new()
    };
    Ok(Unlock {
        present: true,
        context,
        has_ingredients,
        ingredients,
    })
}

fn read_ingredient_slice(r: &mut Reader<'_>, ctx: &Ctx<'_>) -> Result<Vec<IngredientCount>> {
    let n = r.read_count()?;
    let mut v = Vec::with_capacity(n);
    for _ in 0..n {
        v.push(read_ingredient(r, ctx)?);
    }
    Ok(v)
}

fn read_ingredient(r: &mut Reader<'_>, ctx: &Ctx<'_>) -> Result<IngredientCount> {
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
        DESC_COMPLEX_ALIAS => Ingredient::ComplexAlias { name: Str::read(r)? },
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
        "molang" => Ingredient::MoLang {
            expression: Str::read(r)?,
            version: r.read_i16_le()?,
        },
        "item_tag" => {
            let tag = Str::read(r)?;
            r.read_varint()?;
            Ingredient::ItemTag { tag }
        }
        _ => return Err(Error::Invalid("unknown item descriptor kind")),
    })
}

fn read_output_slice(r: &mut Reader<'_>, ctx: &Ctx<'_>) -> Result<Vec<Item>> {
    let n = r.read_count()?;
    let mut v = Vec::with_capacity(n);
    for _ in 0..n {
        v.push(read_output(r, ctx)?);
    }
    Ok(v)
}

fn read_output(r: &mut Reader<'_>, ctx: &Ctx<'_>) -> Result<Item> {
    if ctx.to_v2168 {
        NetworkItemInstanceDescriptor::read(r)
    } else {
        NetworkItemInstanceDescriptorV2168::read(r)
    }
}

fn write_v2168_vectors(out: &mut Writer, r: &Recipes, ctx: &Ctx<'_>) -> Result<Report> {
    let mut skipped = 0;
    write_vec(out, &r.shaped, ctx, &mut skipped, encode_shaped)?;
    write_vec(out, &r.shapeless, ctx, &mut skipped, encode_shapeless)?;
    out.write_count(r.multi.len());
    for m in &r.multi {
        write_multi(out, m);
    }
    write_vec(out, &r.user_data_shapeless, ctx, &mut skipped, encode_shapeless)?;
    write_vec(out, &r.shapeless_chemistry, ctx, &mut skipped, encode_shapeless)?;
    write_vec(out, &r.shaped_chemistry, ctx, &mut skipped, encode_shaped)?;
    write_vec(
        out,
        &r.smithing_transform,
        ctx,
        &mut skipped,
        encode_smithing_transform,
    )?;
    write_vec(out, &r.smithing_trim, ctx, &mut skipped, encode_smithing_trim)?;
    Ok(Report {
        total: r.total(),
        skipped,
    })
}

fn write_vec<T>(
    out: &mut Writer,
    items: &[T],
    ctx: &Ctx<'_>,
    skipped: &mut usize,
    encode: fn(&T, &Ctx<'_>) -> Result<Emit<Vec<u8>>>,
) -> Result<()> {
    let mut bodies = Vec::with_capacity(items.len());
    for item in items {
        match encode(item, ctx)? {
            Ok(bytes) => bodies.push(bytes),
            Err(Unrepresentable) => *skipped += 1,
        }
    }
    out.write_count(bodies.len());
    for body in &bodies {
        out.write_bytes(body);
    }
    Ok(())
}

fn write_v1001_list(out: &mut Writer, r: &Recipes, ctx: &Ctx<'_>) -> Result<Report> {
    let mut skipped = 0;
    let mut tagged: Vec<(i32, Vec<u8>)> = Vec::with_capacity(r.total());

    collect(&mut tagged, TYPE_SHAPED, &r.shaped, ctx, &mut skipped, encode_shaped)?;
    collect(
        &mut tagged,
        TYPE_SHAPELESS,
        &r.shapeless,
        ctx,
        &mut skipped,
        encode_shapeless,
    )?;
    for m in &r.multi {
        let mut w = Writer::new();
        write_multi(&mut w, m);
        tagged.push((TYPE_MULTI, w.into_vec()));
    }
    collect(
        &mut tagged,
        TYPE_USER_DATA_SHAPELESS,
        &r.user_data_shapeless,
        ctx,
        &mut skipped,
        encode_shapeless,
    )?;
    collect(
        &mut tagged,
        TYPE_SHAPELESS_CHEMISTRY,
        &r.shapeless_chemistry,
        ctx,
        &mut skipped,
        encode_shapeless,
    )?;
    collect(
        &mut tagged,
        TYPE_SHAPED_CHEMISTRY,
        &r.shaped_chemistry,
        ctx,
        &mut skipped,
        encode_shaped,
    )?;
    collect(
        &mut tagged,
        TYPE_SMITHING_TRANSFORM,
        &r.smithing_transform,
        ctx,
        &mut skipped,
        encode_smithing_transform,
    )?;
    collect(
        &mut tagged,
        TYPE_SMITHING_TRIM,
        &r.smithing_trim,
        ctx,
        &mut skipped,
        encode_smithing_trim,
    )?;

    out.write_count(tagged.len());
    for (ty, body) in &tagged {
        out.write_varint(*ty);
        out.write_bytes(body);
    }
    Ok(Report {
        total: r.total(),
        skipped,
    })
}

fn collect<T>(
    into: &mut Vec<(i32, Vec<u8>)>,
    ty: i32,
    items: &[T],
    ctx: &Ctx<'_>,
    skipped: &mut usize,
    encode: fn(&T, &Ctx<'_>) -> Result<Emit<Vec<u8>>>,
) -> Result<()> {
    for item in items {
        match encode(item, ctx)? {
            Ok(bytes) => into.push((ty, bytes)),
            Err(Unrepresentable) => *skipped += 1,
        }
    }
    Ok(())
}

fn encode_shapeless(r: &Shapeless, ctx: &Ctx<'_>) -> Result<Emit<Vec<u8>>> {
    let mut w = Writer::new();
    Str::write(&mut w, &r.recipe_id);
    if write_ingredient_slice(&mut w, &r.inputs, ctx)?.is_err() {
        return Ok(Err(Unrepresentable));
    }
    write_output_slice(&mut w, &r.outputs, ctx);
    Uuid::write(&mut w, &r.uuid);
    Str::write(&mut w, &r.block);
    w.write_varint(r.priority);
    if write_unlock(&mut w, &r.unlock, ctx)?.is_err() {
        return Ok(Err(Unrepresentable));
    }
    w.write_uvarint(r.network_id);
    Ok(Ok(w.into_vec()))
}

fn encode_shaped(r: &Shaped, ctx: &Ctx<'_>) -> Result<Emit<Vec<u8>>> {
    if !ctx.to_v2168 && i64::from(r.width) * i64::from(r.height) != r.inputs.len() as i64 {
        return Err(Error::Invalid(
            "shaped recipe ingredients do not match width times height",
        ));
    }
    let mut w = Writer::new();
    Str::write(&mut w, &r.recipe_id);
    w.write_varint(r.width);
    w.write_varint(r.height);
    if ctx.to_v2168 {
        if write_ingredient_slice(&mut w, &r.inputs, ctx)?.is_err() {
            return Ok(Err(Unrepresentable));
        }
    } else {
        for ingredient in &r.inputs {
            if write_ingredient(&mut w, ingredient, ctx)?.is_err() {
                return Ok(Err(Unrepresentable));
            }
        }
    }
    write_output_slice(&mut w, &r.outputs, ctx);
    Uuid::write(&mut w, &r.uuid);
    Str::write(&mut w, &r.block);
    w.write_varint(r.priority);
    w.write_bool(r.assume_symmetry);
    if write_unlock(&mut w, &r.unlock, ctx)?.is_err() {
        return Ok(Err(Unrepresentable));
    }
    w.write_uvarint(r.network_id);
    Ok(Ok(w.into_vec()))
}

fn write_multi(w: &mut Writer, r: &Multi) {
    Uuid::write(w, &r.uuid);
    w.write_uvarint(r.network_id);
}

fn encode_smithing_transform(r: &SmithingTransform, ctx: &Ctx<'_>) -> Result<Emit<Vec<u8>>> {
    let mut w = Writer::new();
    Str::write(&mut w, &r.recipe_id);
    for ingredient in [&r.template, &r.base, &r.addition] {
        if write_ingredient(&mut w, ingredient, ctx)?.is_err() {
            return Ok(Err(Unrepresentable));
        }
    }
    write_output(&mut w, &r.result, ctx);
    Str::write(&mut w, &r.block);
    w.write_uvarint(r.network_id);
    Ok(Ok(w.into_vec()))
}

fn encode_smithing_trim(r: &SmithingTrim, ctx: &Ctx<'_>) -> Result<Emit<Vec<u8>>> {
    let mut w = Writer::new();
    Str::write(&mut w, &r.recipe_id);
    for ingredient in [&r.template, &r.base, &r.addition] {
        if write_ingredient(&mut w, ingredient, ctx)?.is_err() {
            return Ok(Err(Unrepresentable));
        }
    }
    Str::write(&mut w, &r.block);
    w.write_uvarint(r.network_id);
    Ok(Ok(w.into_vec()))
}

fn write_unlock(w: &mut Writer, u: &Unlock, ctx: &Ctx<'_>) -> Result<Emit<()>> {
    if ctx.to_v2168 {
        w.write_bool(u.present);
        if !u.present {
            return Ok(Ok(()));
        }
        w.write_varint(u.context);
        w.write_bool(u.has_ingredients);
        if u.has_ingredients {
            return write_ingredient_slice(w, &u.ingredients, ctx);
        }
        return Ok(Ok(()));
    }
    let context = if u.present { u.context } else { 0 };
    let byte = u8::try_from(context).map_err(|_| Error::Invalid("recipe unlock context"))?;
    w.write_u8(byte);
    if context == 0 {
        let ingredients: &[IngredientCount] = if u.has_ingredients {
            &u.ingredients
        } else {
            &[]
        };
        return write_ingredient_slice(w, ingredients, ctx);
    }
    Ok(Ok(()))
}

fn write_ingredient_slice(
    w: &mut Writer,
    items: &[IngredientCount],
    ctx: &Ctx<'_>,
) -> Result<Emit<()>> {
    let mut body = Writer::new();
    for item in items {
        if write_ingredient(&mut body, item, ctx)?.is_err() {
            return Ok(Err(Unrepresentable));
        }
    }
    w.write_count(items.len());
    w.write_bytes(&body.into_vec());
    Ok(Ok(()))
}

fn write_ingredient(w: &mut Writer, d: &IngredientCount, ctx: &Ctx<'_>) -> Result<Emit<()>> {
    let mut body = Writer::new();
    let outcome = if ctx.to_v2168 {
        write_ingredient_v2168(&mut body, &d.what, ctx.ids)
    } else {
        write_ingredient_v1001(&mut body, &d.what, ctx.names)
    };
    if outcome.is_err() {
        return Ok(Err(Unrepresentable));
    }
    w.write_bytes(&body.into_vec());
    w.write_varint(d.count);
    Ok(Ok(()))
}

fn write_ingredient_v2168(
    w: &mut Writer,
    d: &Ingredient,
    ids: &HashMap<i32, String>,
) -> Emit<()> {
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
                        None => return Err(Unrepresentable),
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
            version,
        } => {
            w.write_uvarint(u32::from(DESC_DEFAULT));
            w.write_string("molang");
            Str::write(w, expression);
            w.write_i16_le(*version);
        }
        Ingredient::ItemTag { tag } => {
            w.write_uvarint(u32::from(DESC_DEFAULT));
            w.write_string("item_tag");
            Str::write(w, tag);
            w.write_varint(DESCRIPTOR_AUX);
        }
        Ingredient::ComplexAlias { .. } => return Err(Unrepresentable),
    }
    Ok(())
}

fn write_ingredient_v1001(
    w: &mut Writer,
    d: &Ingredient,
    names: &HashMap<String, i32>,
) -> Emit<()> {
    match d {
        Ingredient::Invalid => w.write_u8(DESC_INVALID),
        Ingredient::Item { id, name, meta } => {
            let resolved = match id {
                Some(i) => *i,
                None => match name.as_ref().and_then(|n| names.get(n)) {
                    Some(i) => *i,
                    None => return Err(Unrepresentable),
                },
            };
            w.write_u8(DESC_DEFAULT);
            w.write_i16_le(i16::try_from(resolved).map_err(|_| Unrepresentable)?);
            if resolved != 0 {
                w.write_i16_le(i16::try_from(*meta).map_err(|_| Unrepresentable)?);
            }
        }
        Ingredient::MoLang {
            expression,
            version,
        } => {
            w.write_u8(DESC_MOLANG);
            Str::write(w, expression);
            w.write_u8(u8::try_from(*version).map_err(|_| Unrepresentable)?);
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

fn write_output_slice(w: &mut Writer, items: &[Item], ctx: &Ctx<'_>) {
    w.write_count(items.len());
    for item in items {
        write_output(w, item, ctx);
    }
}

fn write_output(w: &mut Writer, item: &Item, ctx: &Ctx<'_>) {
    if ctx.to_v2168 {
        NetworkItemInstanceDescriptorV2168::write(w, item);
    } else {
        NetworkItemInstanceDescriptor::write(w, item);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bedrock_codec::PacketWrapper;

    fn registry() -> (HashMap<String, i32>, HashMap<i32, String>) {
        let mut names = HashMap::new();
        let mut ids = HashMap::new();
        names.insert("minecraft:stone".to_string(), 5);
        ids.insert(5, "minecraft:stone".to_string());
        names.insert("minecraft:iron_ingot".to_string(), 9);
        ids.insert(9, "minecraft:iron_ingot".to_string());
        (names, ids)
    }

    fn state_with_registry() -> ConnState {
        let mut s = ConnState::new(2168);
        let (names, ids) = registry();
        s.item_ids = names;
        s.item_names = ids;
        s
    }

    fn plain_output(id: i32) -> Item {
        Item {
            network_id: id,
            count: 1,
            aux_value: 0,
            has_net_id: false,
            stack_net_id: 0,
            net_id_variant: 0,
            block_runtime_id: 0,
            user_data: Vec::new(),
        }
    }

    fn v1001_bytes() -> Vec<u8> {
        let mut w = Writer::new();
        w.write_count(1);
        w.write_varint(TYPE_SHAPELESS);
        Str::write(&mut w, &"recipe:stone_from_iron".to_string());
        w.write_count(1);
        w.write_u8(DESC_DEFAULT);
        w.write_i16_le(9);
        w.write_i16_le(0);
        w.write_varint(1);
        w.write_count(1);
        NetworkItemInstanceDescriptor::write(&mut w, &plain_output(5));
        Uuid::write(&mut w, &MceUuid::default());
        Str::write(&mut w, &"crafting_table".to_string());
        w.write_varint(0);
        w.write_u8(1);
        w.write_uvarint(7);
        w.write_count(0);
        w.write_count(0);
        w.write_count(0);
        w.write_bool(false);
        w.into_vec()
    }

    #[test]
    fn v1001_to_v2168_round_trips_a_shapeless_recipe() {
        let mut state = state_with_registry();
        let input = v1001_bytes();
        let mut wrapper = PacketWrapper::new(&input);
        let ok = crafting_data(&mut wrapper, &mut state, true).unwrap();
        assert!(ok, "recipe with a known ingredient must not be dropped");
        assert!(state.notices.is_empty(), "no ingredient should have been skipped");

        let out = wrapper.finish();
        let mut r = Reader::new(&out);

        assert_eq!(r.read_count().unwrap(), 0);
        assert_eq!(r.read_count().unwrap(), 1);
        assert_eq!(Str::read(&mut r).unwrap(), "recipe:stone_from_iron");
        assert_eq!(r.read_count().unwrap(), 1);
        assert_eq!(r.read_uvarint().unwrap(), u32::from(DESC_DEFAULT));
        assert_eq!(Str::read(&mut r).unwrap(), "name");
        assert_eq!(Str::read(&mut r).unwrap(), "minecraft:iron_ingot");
        assert_eq!(r.read_varint().unwrap(), 0);
        assert_eq!(r.read_varint().unwrap(), 1);
    }

    #[test]
    fn v2168_to_v1001_round_trips_the_same_recipe() {
        let mut state = state_with_registry();
        let input = v1001_bytes();

        let mut up = PacketWrapper::new(&input);
        crafting_data(&mut up, &mut state, true).unwrap();
        let v2168_bytes = up.finish();

        let mut down = PacketWrapper::new(&v2168_bytes);
        let ok = crafting_data(&mut down, &mut state, false).unwrap();
        assert!(ok);
        assert!(state.notices.is_empty());

        let round_tripped = down.finish();
        assert_eq!(round_tripped, input, "v1001 -> v2168 -> v1001 must be the identity");
    }

    #[test]
    fn unknown_ingredient_drops_only_that_recipe() {
        let mut state = state_with_registry();
        let mut w = Writer::new();
        w.write_count(1);
        w.write_varint(TYPE_SHAPELESS);
        Str::write(&mut w, &"recipe:unknown".to_string());
        w.write_count(1);
        w.write_u8(DESC_DEFAULT);
        w.write_i16_le(999);
        w.write_i16_le(0);
        w.write_varint(1);
        w.write_count(0);
        Uuid::write(&mut w, &MceUuid::default());
        Str::write(&mut w, &"crafting_table".to_string());
        w.write_varint(0);
        w.write_u8(1);
        w.write_uvarint(1);
        w.write_count(0);
        w.write_count(0);
        w.write_count(0);
        w.write_bool(false);
        let input = w.into_vec();

        let mut wrapper = PacketWrapper::new(&input);
        let ok = crafting_data(&mut wrapper, &mut state, true).unwrap();
        assert!(ok, "a decodable packet with one bad ingredient is still forwarded");
        assert_eq!(state.notices.len(), 1);
        assert!(state.notices[0].contains("1 of 1 recipes dropped"));

        let out = wrapper.finish();
        let mut r = Reader::new(&out);
        assert_eq!(r.read_count().unwrap(), 0);
        assert_eq!(r.read_count().unwrap(), 0);
    }

    #[test]
    fn recipe_type_discriminants_are_not_dense() {
        assert_eq!(TYPE_MULTI, 4);
        assert_eq!(TYPE_USER_DATA_SHAPELESS, 5);
        assert_eq!(TYPE_SHAPELESS_CHEMISTRY, 6);
        assert_eq!(TYPE_SHAPED_CHEMISTRY, 7);
        assert_eq!(TYPE_SMITHING_TRANSFORM, 8);
        assert_eq!(TYPE_SMITHING_TRIM, 9);
    }

    #[test]
    fn multi_recipe_round_trips_through_the_correct_vector() {
        let mut state = state_with_registry();
        let mut w = Writer::new();
        w.write_count(1);
        w.write_varint(TYPE_MULTI);
        Uuid::write(&mut w, &MceUuid::default());
        w.write_uvarint(42);
        w.write_count(0);
        w.write_count(0);
        w.write_count(0);
        w.write_bool(false);
        let input = w.into_vec();

        let mut wrapper = PacketWrapper::new(&input);
        let ok = crafting_data(&mut wrapper, &mut state, true).unwrap();
        assert!(ok);
        assert!(state.notices.is_empty());

        let out = wrapper.finish();
        let mut r = Reader::new(&out);
        assert_eq!(r.read_count().unwrap(), 0);
        assert_eq!(r.read_count().unwrap(), 0);
        assert_eq!(r.read_count().unwrap(), 1);
        Uuid::read(&mut r).unwrap();
        assert_eq!(r.read_uvarint().unwrap(), 42);
    }

    #[test]
    fn shaped_recipe_grid_has_no_length_prefix_on_v1001() {
        let mut state = state_with_registry();
        let mut w = Writer::new();
        w.write_count(1);
        w.write_varint(TYPE_SHAPED);
        Str::write(&mut w, &"recipe:shaped".to_string());
        w.write_varint(2);
        w.write_varint(1);
        for _ in 0..2 {
            w.write_u8(DESC_DEFAULT);
            w.write_i16_le(5);
            w.write_i16_le(0);
            w.write_varint(1);
        }
        w.write_count(1);
        NetworkItemInstanceDescriptor::write(&mut w, &plain_output(5));
        Uuid::write(&mut w, &MceUuid::default());
        Str::write(&mut w, &"crafting_table".to_string());
        w.write_varint(0);
        w.write_bool(false);
        w.write_u8(1);
        w.write_uvarint(3);
        w.write_count(0);
        w.write_count(0);
        w.write_count(0);
        w.write_bool(false);
        let input = w.into_vec();

        let mut wrapper = PacketWrapper::new(&input);
        let ok = crafting_data(&mut wrapper, &mut state, true).unwrap();
        assert!(ok);
        assert!(state.notices.is_empty());

        let out = wrapper.finish();
        let mut r = Reader::new(&out);
        assert_eq!(r.read_count().unwrap(), 1);
        Str::read(&mut r).unwrap();
        assert_eq!(r.read_varint().unwrap(), 2);
        assert_eq!(r.read_varint().unwrap(), 1);
        assert_eq!(r.read_count().unwrap(), 2);
    }

    #[test]
    fn unlock_requirement_absent_on_v1001_has_no_outer_flag() {
        let mut state = state_with_registry();
        let mut w = Writer::new();
        w.write_count(1);
        w.write_varint(TYPE_SHAPELESS);
        Str::write(&mut w, &"recipe:no_unlock".to_string());
        w.write_count(0);
        w.write_count(0);
        Uuid::write(&mut w, &MceUuid::default());
        Str::write(&mut w, &"crafting_table".to_string());
        w.write_varint(0);
        w.write_u8(2);
        w.write_uvarint(9);
        w.write_count(0);
        w.write_count(0);
        w.write_count(0);
        w.write_bool(false);
        let input = w.into_vec();

        let mut up = PacketWrapper::new(&input);
        crafting_data(&mut up, &mut state, true).unwrap();
        let v2168_bytes = up.finish();
        let mut r = Reader::new(&v2168_bytes);
        r.read_count().unwrap();
        assert_eq!(r.read_count().unwrap(), 1);
        Str::read(&mut r).unwrap();
        r.read_count().unwrap();
        r.read_count().unwrap();
        Uuid::read(&mut r).unwrap();
        Str::read(&mut r).unwrap();
        r.read_varint().unwrap();
        assert!(r.read_bool().unwrap(), "v2168 outer Optional must be present");
        assert_eq!(r.read_varint().unwrap(), 2);
        assert!(!r.read_bool().unwrap(), "context != 0 means no ingredient table");
    }

    #[test]
    fn malformed_packet_is_cancelled_not_forwarded() {
        let mut state = state_with_registry();
        let input = vec![0xFFu8];
        let mut wrapper = PacketWrapper::new(&input);
        let ok = crafting_data(&mut wrapper, &mut state, true).unwrap();
        assert!(!ok, "an undecodable packet must be reported as cancel, not forwarded");
        assert!(state.notices.iter().any(|n| n.contains("cannot decode")));
    }
}
