use crate::translator::Translator;

pub fn downgrade() -> Translator {
    Translator::new("v860->v859", 859, 860)
}

pub fn upgrade() -> Translator {
    Translator::new("v859->v860", 860, 859)
}
