use super::*;

#[test]
fn an_unresolvable_name_becomes_deferred_rather_than_a_dropped_recipe() {
    let mut w = Writer::new();
    let names: HashMap<String, i32> = HashMap::new();
    let ingredient = Ingredient::Item {
        id: None,
        name: Some("modded:widget".to_string()),
        meta: 0,
    };
    write_ingredient_v1001(&mut w, &ingredient, &names)
        .expect("an unresolved name must not be fatal");

    let bytes = w.into_vec();
    let mut r = Reader::new(&bytes);
    assert_eq!(r.read_u8().unwrap(), DESC_DEFERRED);
    assert_eq!(Str::read(&mut r).unwrap(), "modded:widget");
    assert_eq!(r.read_i16_le().unwrap(), 0);
    assert_eq!(r.remaining(), 0);
}
