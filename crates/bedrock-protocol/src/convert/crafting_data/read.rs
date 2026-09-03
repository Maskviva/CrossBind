use super::ingredient::{read_ingredient, read_ingredient_slice, read_output, read_output_slice};
use super::model::{
    Ctx, Multi, Recipes, Shaped, Shapeless, SmithingTransform, SmithingTrim, Unlock,
};
use super::{
    MAX_GRID, TYPE_MULTI, TYPE_SHAPED, TYPE_SHAPED_CHEMISTRY, TYPE_SHAPELESS,
    TYPE_SHAPELESS_CHEMISTRY, TYPE_SMITHING_TRANSFORM, TYPE_SMITHING_TRIM,
    TYPE_USER_DATA_SHAPELESS,
};
use bedrock_codec::prelude::*;

pub(super) fn read_v1001_list(r: &mut Reader<'_>, ctx: &Ctx<'_>) -> Result<Recipes> {
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
            TYPE_SMITHING_TRANSFORM => out
                .smithing_transform
                .push(read_smithing_transform(r, ctx)?),
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

pub(super) fn read_v2168_vectors(r: &mut Reader<'_>, ctx: &Ctx<'_>) -> Result<Recipes> {
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
