use super::audit::{ResultAudit, Skipped};
use super::ingredient::{
    write_ingredient, write_ingredient_slice, write_output, write_output_slice,
};
use super::model::{
    Ctx, Emit, IngredientCount, Multi, Recipes, Report, Shaped, Shapeless, SmithingTransform,
    SmithingTrim, Unlock,
};
use super::{
    TYPE_MULTI, TYPE_SHAPED, TYPE_SHAPED_CHEMISTRY, TYPE_SHAPELESS, TYPE_SHAPELESS_CHEMISTRY,
    TYPE_SMITHING_TRANSFORM, TYPE_SMITHING_TRIM, TYPE_USER_DATA_SHAPELESS,
};
use bedrock_codec::prelude::*;

pub(super) fn write_v2168_vectors(out: &mut Writer, r: &Recipes, ctx: &Ctx<'_>) -> Result<Report> {
    let mut skipped = Skipped::default();
    write_vec(out, &r.shaped, ctx, &mut skipped, encode_shaped)?;
    write_vec(out, &r.shapeless, ctx, &mut skipped, encode_shapeless)?;
    out.write_count(r.multi.len());
    for m in &r.multi {
        write_multi(out, m);
    }
    write_vec(
        out,
        &r.user_data_shapeless,
        ctx,
        &mut skipped,
        encode_shapeless,
    )?;
    write_vec(
        out,
        &r.shapeless_chemistry,
        ctx,
        &mut skipped,
        encode_shapeless,
    )?;
    write_vec(out, &r.shaped_chemistry, ctx, &mut skipped, encode_shaped)?;
    write_vec(
        out,
        &r.smithing_transform,
        ctx,
        &mut skipped,
        encode_smithing_transform,
    )?;
    write_vec(
        out,
        &r.smithing_trim,
        ctx,
        &mut skipped,
        encode_smithing_trim,
    )?;
    Ok(Report {
        total: r.total(),
        skipped,
        results: ResultAudit::default(),
        dumps: Vec::new(),
    })
}

fn write_vec<T>(
    out: &mut Writer,
    items: &[T],
    ctx: &Ctx<'_>,
    skipped: &mut Skipped,
    encode: fn(&T, &Ctx<'_>) -> Result<Emit<Vec<u8>>>,
) -> Result<()> {
    let mut bodies = Vec::with_capacity(items.len());
    for item in items {
        match encode(item, ctx)? {
            Ok(bytes) => bodies.push(bytes),
            Err(why) => skipped.record(why),
        }
    }
    out.write_count(bodies.len());
    for body in &bodies {
        out.write_bytes(body);
    }
    Ok(())
}

pub(super) fn write_v1001_list(out: &mut Writer, r: &Recipes, ctx: &Ctx<'_>) -> Result<Report> {
    let mut skipped = Skipped::default();
    let mut tagged: Vec<(i32, Vec<u8>)> = Vec::with_capacity(r.total());

    collect(
        &mut tagged,
        TYPE_SHAPED,
        &r.shaped,
        ctx,
        &mut skipped,
        encode_shaped,
    )?;
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
        results: ResultAudit::default(),
        dumps: Vec::new(),
    })
}

pub(super) fn collect<T>(
    into: &mut Vec<(i32, Vec<u8>)>,
    ty: i32,
    items: &[T],
    ctx: &Ctx<'_>,
    skipped: &mut Skipped,
    encode: fn(&T, &Ctx<'_>) -> Result<Emit<Vec<u8>>>,
) -> Result<()> {
    for item in items {
        match encode(item, ctx)? {
            Ok(bytes) => into.push((ty, bytes)),
            Err(why) => skipped.record(why),
        }
    }
    Ok(())
}

pub(super) fn encode_shapeless(r: &Shapeless, ctx: &Ctx<'_>) -> Result<Emit<Vec<u8>>> {
    let mut w = Writer::new();
    Str::write(&mut w, &r.recipe_id);
    if let Err(why) = write_ingredient_slice(&mut w, &r.inputs, ctx)? {
        return Ok(Err(why));
    }
    write_output_slice(&mut w, &r.outputs, ctx);
    Uuid::write(&mut w, &r.uuid);
    Str::write(&mut w, &r.block);
    w.write_varint(r.priority);
    if let Err(why) = write_unlock(&mut w, &r.unlock, ctx)? {
        return Ok(Err(why));
    }
    w.write_uvarint(r.network_id);
    Ok(Ok(w.into_vec()))
}

pub(super) fn encode_shaped(r: &Shaped, ctx: &Ctx<'_>) -> Result<Emit<Vec<u8>>> {
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
        if let Err(why) = write_ingredient_slice(&mut w, &r.inputs, ctx)? {
            return Ok(Err(why));
        }
    } else {
        for ingredient in &r.inputs {
            if let Err(why) = write_ingredient(&mut w, ingredient, ctx)? {
                return Ok(Err(why));
            }
        }
    }
    write_output_slice(&mut w, &r.outputs, ctx);
    Uuid::write(&mut w, &r.uuid);
    Str::write(&mut w, &r.block);
    w.write_varint(r.priority);
    w.write_bool(r.assume_symmetry);
    if let Err(why) = write_unlock(&mut w, &r.unlock, ctx)? {
        return Ok(Err(why));
    }
    w.write_uvarint(r.network_id);
    Ok(Ok(w.into_vec()))
}

fn write_multi(w: &mut Writer, r: &Multi) {
    Uuid::write(w, &r.uuid);
    w.write_uvarint(r.network_id);
}

pub(super) fn encode_smithing_transform(
    r: &SmithingTransform,
    ctx: &Ctx<'_>,
) -> Result<Emit<Vec<u8>>> {
    let mut w = Writer::new();
    Str::write(&mut w, &r.recipe_id);
    for ingredient in [&r.template, &r.base, &r.addition] {
        if let Err(why) = write_ingredient(&mut w, ingredient, ctx)? {
            return Ok(Err(why));
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
        if let Err(why) = write_ingredient(&mut w, ingredient, ctx)? {
            return Ok(Err(why));
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
