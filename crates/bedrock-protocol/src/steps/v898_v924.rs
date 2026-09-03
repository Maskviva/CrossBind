use crate::mapping::IdShift;
use crate::rewriter::SoundRewriter;
use crate::translator::Translator;

const SOUND: IdShift = IdShift::inserted(19, 578);
const HEARTBEAT_KEY: u32 = 126;

const AIM_ASSIST_KEYS: &[u32] = &[136, 137, 138];

pub fn downgrade() -> Translator {
    let step = Translator::new("v924->v898", 898, 924);
    SoundRewriter::new(SOUND, true, HEARTBEAT_KEY)
        .with_dropped_keys(AIM_ASSIST_KEYS)
        .register(step)
}

pub fn upgrade() -> Translator {
    let step = Translator::new("v898->v924", 924, 898);
    SoundRewriter::new(SOUND, false, HEARTBEAT_KEY)
        .with_dropped_keys(AIM_ASSIST_KEYS)
        .register(step)
}
