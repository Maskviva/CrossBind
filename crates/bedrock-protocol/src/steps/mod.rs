use std::sync::Arc;

use crate::translator::Translator;

pub mod v859_v860;
pub mod v860_v898;
pub mod v898_v924;
pub mod v924_v944;
pub mod v944_v975;
pub mod v975_v1001;
pub mod item_stack_v2168;
pub mod crafting_data_v2168;
pub mod player_list_v2168;
pub mod set_score_v2168;
pub mod v1001_v2168;

pub fn all() -> Vec<Arc<Translator>> {
    vec![
        Arc::new(v859_v860::downgrade()),
        Arc::new(v859_v860::upgrade()),
        Arc::new(v860_v898::downgrade()),
        Arc::new(v860_v898::upgrade()),
        Arc::new(v898_v924::downgrade()),
        Arc::new(v898_v924::upgrade()),
        Arc::new(v924_v944::downgrade()),
        Arc::new(v924_v944::upgrade()),
        Arc::new(v944_v975::downgrade()),
        Arc::new(v944_v975::upgrade()),
        Arc::new(v975_v1001::downgrade()),
        Arc::new(v975_v1001::upgrade()),
        Arc::new(v1001_v2168::downgrade()),
        Arc::new(v1001_v2168::upgrade()),
    ]
}

#[cfg(test)]
mod tests {
    use crate::versions::TRANSLATABLE;

    #[test]
    fn every_adjacent_pair_has_both_directions() {
        let steps = super::all();
        for pair in TRANSLATABLE.windows(2) {
            let (older, newer) = (pair[0], pair[1]);
            assert!(
                steps
                    .iter()
                    .any(|s| s.server_protocol == older && s.client_protocol == newer),
                "missing downgrade step {older} <- {newer}"
            );
            assert!(
                steps
                    .iter()
                    .any(|s| s.server_protocol == newer && s.client_protocol == older),
                "missing upgrade step {newer} <- {older}"
            );
        }
    }

    #[test]
    fn no_step_claims_a_version_outside_the_translatable_set() {
        for step in super::all() {
            assert!(
                TRANSLATABLE.contains(&step.server_protocol),
                "{} names untranslatable server protocol {}",
                step.name,
                step.server_protocol
            );
            assert!(
                TRANSLATABLE.contains(&step.client_protocol),
                "{} names untranslatable client protocol {}",
                step.name,
                step.client_protocol
            );
        }
    }
}
