use crate::translator::Translator;

pub fn downgrade() -> Translator {
    Translator::new("v2169->v2168", 2168, 2169)
}

pub fn upgrade() -> Translator {
    Translator::new("v2168->v2169", 2169, 2168)
}
